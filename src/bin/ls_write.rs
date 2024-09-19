use lettersmith::cli::parse_args;
use lettersmith::io::dump_errors_to_stderr;
use lettersmith::{docs, json};
use std::path::PathBuf;
use tap::Pipe;

/// Read docs from stdin and write to output dir
fn main() {
    let args = parse_args();
    let config = args.read_config().expect("Could not read config");
    let output_dir = match config.get("output_dir") {
        Some(json::Value::String(value)) => PathBuf::from(value),
        _ => PathBuf::from("public"),
    };
    let docs = docs::read_stdin().pipe(dump_errors_to_stderr);
    docs::write(docs, &output_dir);
}
