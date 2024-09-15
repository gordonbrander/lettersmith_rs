use lettersmith::cli::parse_args;
use lettersmith::io::{dump_errors_to_stderr, read_docs, write_stdio};
use tap::Pipe;

/// Read all file paths to docs and stream JSON to stdout.
fn main() {
    let args = parse_args();
    let docs = read_docs(args.files).pipe(dump_errors_to_stderr);
    write_stdio(docs);
}
