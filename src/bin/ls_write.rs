use lettersmith::cli::parse_args;
use lettersmith::docs::{DocResults, Docs};
use lettersmith::{docs, json};
use std::path::PathBuf;

/// Read docs from stdin and write to output dir
fn main() {
    let args = parse_args();
    let config = args.read_config().expect("Could not read config");
    let output_dir = match config.get("output_dir") {
        Some(json::Value::String(value)) => PathBuf::from(value),
        _ => PathBuf::from("public"),
    };
    docs::read_stdin()
        .dump_errors_to_stderr()
        .write(&output_dir);
}
