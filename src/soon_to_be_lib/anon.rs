use crate::soon_to_be_lib::spec::Spec;
use csv::{ReaderBuilder, StringRecord, WriterBuilder};
use std::{
    collections::btree_map::Entry, collections::BTreeMap, collections::BTreeSet, io::Read,
    io::Write,
};

#[derive(Debug, Default)]
pub struct RewriteInfo {
    pub rows: u64,
    pub cells: u64,
}

fn validated(specs: &[Spec]) -> Result<&[Spec], failure::Error> {
    let seen = specs
        .iter()
        .fold(BTreeSet::<usize>::new(), |mut memo, spec| {
            memo.insert(spec.column);
            memo
        });
    if seen.len() != specs.len() {
        bail!(
            "rewrite specifications contained {} duplicate column(s)",
            specs.len() - seen.len()
        );
    }
    Ok(specs)
}

pub fn anonymise(
    input: impl Read,
    delimiter: u8,
    has_header: bool,
    specs: &[Spec],
    output: impl Write,
) -> Result<RewriteInfo, failure::Error> {
    let specs = validated(specs)?;
    let mut csv = ReaderBuilder::new()
        .has_headers(has_header)
        .delimiter(delimiter)
        .from_reader(input);
    let mut out_csv = WriterBuilder::new()
        .has_headers(has_header)
        .delimiter(delimiter)
        .from_writer(output);
    let mut info = RewriteInfo::default();
    let mut memo = BTreeMap::<String, String>::new();
    let mut memoized = |cell: &str, spec: &Spec| match memo.entry(cell.to_owned()) {
        Entry::Occupied(v) => v.get().to_owned(),
        Entry::Vacant(e) => e.insert(spec.kind.fake().into_owned()).to_owned(),
    };
    if has_header {
        out_csv.write_record(csv.headers()?)?;
    }
    for record in csv.records() {
        let record = record?;
        info.rows += 1;
        let mut anon_record =
            StringRecord::with_capacity(record.as_slice().as_bytes().len(), record.len());
        let mut last_cell: Option<usize> = None;
        let push_fields = |target: &mut StringRecord, from: Option<usize>, to| {
            for index in (from.map(|index| index + 1).unwrap_or(0))..to {
                target.push_field(&record[index])
            }
        };
        for spec in specs {
            info.cells += 1;
            let cell = record.get(spec.column).ok_or_else(|| {
                format_err!(
                    "Invalid column index {} - rows have no more than {} columns",
                    spec.column,
                    record.len()
                )
            })?;
            push_fields(&mut anon_record, last_cell, spec.column);
            anon_record.push_field(&memoized(cell, spec));
            last_cell = Some(spec.column);
        }
        push_fields(&mut anon_record, last_cell, record.len());
        out_csv.write_record(&anon_record)?;
    }
    Ok(info)
}
