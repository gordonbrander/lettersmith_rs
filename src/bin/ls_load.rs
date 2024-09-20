use lettersmith::cli::parse_args;
use lettersmith::docs::{self, DocResults, Docs};

/// Read all file paths to docs and stream JSON to stdout.
fn main() {
    let args = parse_args();
    docs::read(args.files.into_iter())
        .dump_errors_to_stderr()
        .write_stdio();
}
