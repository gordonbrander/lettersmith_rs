use lettersmith::blog::BlogDocs;
use lettersmith::cli::parse_args;
use lettersmith::docs::{self, DocResults, Docs};

fn main() {
    let config = parse_args().read_config().expect("Could not read config");
    docs::read_stdin()
        .panic_at_first_error()
        .markdown_post(&config)
        .panic_at_first_error()
        .write_stdio();
}
