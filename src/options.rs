use crate::soon_to_be_lib::spec::Spec;
use std::path::PathBuf;

fn parse_spec(src: &str) -> Result<Spec, failure::Error> {
    src.parse()
}

#[derive(Debug, StructOpt)]
#[structopt(name = "anon-csv", about = "A CSV-file anonymizer")]
pub struct Args {
    /// If set, no additional output will be produced on stderr
    #[structopt(short = "q", long = "quiet")]
    pub quiet: bool,
    /// The delimiter of the input CSV file. The same delimiter will be used for output.
    #[structopt(short = "d", long = "delimiter", default_value = ",")]
    pub delimiter: char,
    /// The path to the CSV file to use as input
    #[structopt(parse(from_os_str))]
    pub csv_file: PathBuf,
    /// One or more rewrite specifications. They look like '<column>:<type>', where <column> is
    /// a zero-based column indexed, separated from the <type> being the type of data to fake.
    /// Valid types are Internet.safe_email, Name.name, Name.first_name, Name.last_name
    /// Address.zip
    #[structopt(parse(try_from_str = "parse_spec"))]
    pub specs: Vec<Spec>,
}