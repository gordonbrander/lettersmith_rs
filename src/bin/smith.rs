use clap::{Parser, Subcommand};
use lettersmith::prelude::*;
use std::env;
use std::path::PathBuf;

#[derive(Parser)]
#[command(version = "0.1.0")]
#[command(author = "Lettersmith")]
#[command(
    about = "Lettersmith is a static site generator built around a simple idea: piping JSON documents through stdio. Features are implemented as simple, single-purpose tools. To customize your own static site generator, you string together the features you want using Unix pipes and save those pipelines to a bash file."
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Read docs from file paths")]
    Read {
        #[arg(
            help = "File paths to read. Tip: you can use glob patterns to match specific lists of files. Example: smith read posts/*.md"
        )]
        #[arg(value_name = "FILE")]
        files: Vec<PathBuf>,
    },

    #[command(
        about = "Write docs to directory defined in config file. Typically used at the end of a chain of piped smith commands to take the stream of JSON docs and write it to disk."
    )]
    Write {},

    #[command(about = "Render markdown and templates for blog posts or pages")]
    Blog {
        #[arg(long = "permalink-template")]
        #[arg(default_value = "{parents}/{slug}/index.html")]
        #[arg(help = "Template for rendering permalinks")]
        permalink_template: String,
    },

    #[command(about = "Set permalink via a template")]
    Permalink {
        #[arg(long = "template")]
        #[arg(default_value = "{parents}/{slug}/index.html")]
        #[arg(help = "Template for rendering permalinks")]
        permalink_template: String,
    },

    #[command(about = "Render doc with the Tera template set on doc's template_path")]
    Template {},

    #[command(
        about = "Parse and uplift frontmatter. Frontmatter is parsed as YAML and assigned to doc meta. Blessed fields, such as title are assigned to the corresponding field on the doc."
    )]
    Frontmatter {},
}

/// Read all file paths to docs and stream JSON to stdout.
fn main() {
    let config_path = env::var("CONFIG").unwrap_or("lettersmith.json".to_string());
    let config = Config::read(config_path).unwrap_or(Config::default());
    let cli = Cli::parse();

    match cli.command {
        Commands::Read { files } => read(files),
        Commands::Write {} => write(&config),
        Commands::Blog { permalink_template } => blog(&permalink_template, &config),
        Commands::Permalink { permalink_template } => permalink(&permalink_template),
        Commands::Template {} => template(&config),
        Commands::Frontmatter {} => frontmatter(),
    }
}

/// Read docs from paths
fn read(files: Vec<PathBuf>) {
    docs::read(files.into_iter())
        .panic_at_first_error()
        .write_stdio();
}

fn write(config: &Config) {
    docs::read_stdin()
        .panic_at_first_error()
        .write(&config.output_dir);
}

fn blog(permalink_template: &str, config: &Config) {
    docs::read_stdin()
        .panic_at_first_error()
        .markdown_blog_doc_with_config(permalink_template, config)
        .panic_at_first_error()
        .write_stdio();
}

fn permalink(template: &str) {
    docs::read_stdin()
        .panic_at_first_error()
        .set_permalink(template)
        .write_stdio();
}

/// Render liquid templates
fn template(config: &Config) {
    docs::read_stdin()
        .panic_at_first_error()
        .render_tera_template_with_config(config)
        .panic_at_first_error()
        .write_stdio();
}

/// Parse and uplift frontmatter
fn frontmatter() {
    docs::read_stdin()
        .panic_at_first_error()
        .parse_and_uplift_frontmatter()
        .write_stdio();
}
