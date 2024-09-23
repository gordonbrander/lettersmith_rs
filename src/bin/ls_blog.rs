use lettersmith::blog::BlogDocs;
use lettersmith::cli::parse_args;
use lettersmith::docs::{self, DocResults, Docs};

fn main() {
    let config = parse_args().read_config().expect("Could not read config");
    docs::read_stdin()
        .dump_errors_to_stderr()
        .markdown_post(&config)
        .dump_errors_to_stderr()
        .write_stdio();
}
