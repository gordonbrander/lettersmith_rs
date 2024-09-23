use lettersmith::blog::BlogDocs;
use lettersmith::cli::parse_args;
use lettersmith::docs::{self, DocResults, Docs};

fn main() {
    let config = parse_args().read_config_or_default();
    let base_url = config["base_url"].as_str().unwrap_or("/");
    docs::read_stdin()
        .dump_errors_to_stderr()
        .blog_post(base_url, &config)
        .dump_errors_to_stderr()
        .write_stdio();
}
