use lettersmith::blog::BlogDocs;
use lettersmith::docs::{self, DocResults, Docs};

fn main() {
    docs::read_stdin()
        .dump_errors_to_stderr()
        .blog_post()
        .write_stdio();
}
