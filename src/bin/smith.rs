use clap::{Parser, Subcommand};
use lettersmith::config::Config;
use lettersmith::prelude::*;
use std::env;
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(version = "0.1.0")]
#[command(author = "Lettersmith")]
#[command(about = "Lettersmith is a command line tool for building static sites.")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Read docs from file paths")]
    Read {
        #[arg(
            help = "File paths to read. Tip: you can use glob patterns to match specific lists of files. Example: smith read posts/*.md"
        )]
        #[arg(value_name = "FILE")]
        files: Vec<PathBuf>,
    },

    #[command(
        about = "Write docs to directory defined in config file. Typically used at the end of a chain of piped smith commands to take the stream of JSON docs and write it to disk."
    )]
    Write {},

    #[command(about = "Transforms for markdown blog posts with liquid templates")]
    Post {},

    #[command(about = "Transforms for markdown pages with liquid templates")]
    Page {},
}

/// Read docs from paths
fn read(files: Vec<PathBuf>) {
    docs::read(files.into_iter())
        .panic_at_first_error()
        .write_stdio();
}

fn write(output_dir: &Path) {
    docs::read_stdin().panic_at_first_error().write(output_dir);
}

fn post(config: &Config) {
    docs::read_stdin()
        .panic_at_first_error()
        .markdown_post(config)
        .panic_at_first_error()
        .write_stdio();
}

fn page(config: &Config) {
    docs::read_stdin()
        .panic_at_first_error()
        .markdown_page(config)
        .panic_at_first_error()
        .write_stdio();
}

/// Read all file paths to docs and stream JSON to stdout.
fn main() {
    let cli = Cli::parse();
    let config_path = env::var("CONFIG").unwrap_or("lettersmith.json".to_string());
    let config =
        Config::read(&config_path).expect(&format!("Could not read config at {}", config_path));

    match cli.command {
        Commands::Read { files } => read(files),
        Commands::Write {} => write(&config.output_dir),
        Commands::Post {} => post(&config),
        Commands::Page {} => page(&config),
    }
}
