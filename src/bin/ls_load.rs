use lettersmith::cli::parse_args;
use lettersmith::docs;
use lettersmith::io::dump_errors_to_stderr;
use tap::Pipe;

/// Read all file paths to docs and stream JSON to stdout.
fn main() {
    let args = parse_args();
    let docs = docs::read(args.files.into_iter()).pipe(dump_errors_to_stderr);
    docs::write_stdio(docs);
}
