// Takes a glob and streams JSON docs
use lettersmith::arg_parse::parse_args_to_paths;
use lettersmith::io::{dump_errors_to_stderr, read_docs, write_stdio};
use pipe_trait::Pipe;

fn main() {
    let paths = parse_args_to_paths();
    let docs = read_docs(paths).pipe(dump_errors_to_stderr);
    write_stdio(docs);
}
