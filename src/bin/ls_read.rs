use lettersmith::cli::parse_args;
use lettersmith::docs::{self, DocResults, Docs};

/// Read all file paths to docs and stream JSON to stdout.
fn main() {
    let args = parse_args();
    docs::read(args.files.into_iter())
        .panic_at_first_error()
        .write_stdio();
}
