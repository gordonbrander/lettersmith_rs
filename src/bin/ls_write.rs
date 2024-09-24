use lettersmith::cli::parse_args;
use lettersmith::docs;
use lettersmith::docs::{DocResults, Docs};

/// Read docs from stdin and write to output dir
fn main() {
    let args = parse_args();
    let config = args.read_config().expect("Could not read config");
    docs::read_stdin()
        .panic_at_first_error()
        .write(&config.output_dir);
}
