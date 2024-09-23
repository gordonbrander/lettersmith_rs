use crate::config::Config;
use crate::error::Error;
pub use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Cli {
    /// Sets a custom config file
    #[clap(short, long, value_parser, default_value = "lettersmith.json")]
    pub config: PathBuf,

    /// Input files to process
    #[clap(value_parser)]
    pub files: Vec<PathBuf>,
}

impl Cli {
    pub fn read_config(&self) -> Result<Config, Error> {
        Config::read(&self.config)
    }
}

pub fn parse_args() -> Cli {
    Cli::parse()
}
