use clap::{Parser, Subcommand};
use lettersmith::prelude::*;
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(version = "0.1.0")]
#[command(author = "Lettersmith")]
#[command(about = "Lettersmith is a command line tool for building static sites.")]
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
    Write {
        #[arg(help = "Directory to write files to")]
        output_dir: PathBuf,
    },

    #[command(about = "Transform docs into markdown blog posts or pages with Liquid templates")]
    Blog {
        #[arg(long = "site-url")]
        #[arg(default_value = "/")]
        #[arg(help = "URL for site. Used to absolutize URLS in the page.")]
        site_url: String,

        #[arg(long = "permalink-template")]
        #[arg(default_value = "{parents}/{slug}/index.html")]
        #[arg(help = "Template for rendering permalinks")]
        permalink_template: String,

        #[arg(long = "template-dir")]
        #[arg(default_value = "templates")]
        #[arg(
            help = "Directory containing templates. Used to qualify template paths when automatically assigning templates by path."
        )]
        template_dir: PathBuf,

        #[arg(long = "data")]
        #[arg(default_value = "data.json")]
        #[arg(help = "Path to JSON data file. Data will be provided to Liquid template.")]
        template_data_path: PathBuf,
    },

    #[command(about = "Set permalink via a template")]
    Permalink {
        #[arg(long = "template")]
        #[arg(default_value = "{parents}/{slug}/index.html")]
        #[arg(help = "Template for rendering permalinks")]
        permalink_template: String,
    },

    #[command(about = "Render doc with the liquid template set on doc's template_path")]
    Liquid {
        #[arg(long = "data")]
        #[arg(default_value = "data.json")]
        #[arg(help = "Path to JSON data file. Data will be provided to Liquid template.")]
        template_data_path: PathBuf,
    },

    #[command(
        about = "Parse and uplift frontmatter. Frontmatter is parsed as YAML and assigned to doc meta. Blessed fields, such as title are assigned to the corresponding field on the doc."
    )]
    Frontmatter {},
}

/// Read all file paths to docs and stream JSON to stdout.
fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Read { files } => read(files),
        Commands::Write { output_dir } => write(&output_dir),
        Commands::Blog {
            site_url,
            permalink_template,
            template_dir,
            template_data_path,
        } => blog(
            &site_url,
            &permalink_template,
            &template_dir,
            &template_data_path,
        ),
        Commands::Permalink { permalink_template } => permalink(&permalink_template),
        Commands::Liquid { template_data_path } => liquid(&template_data_path),
        Commands::Frontmatter {} => frontmatter(),
    }
}

/// Read docs from paths
fn read(files: Vec<PathBuf>) {
    docs::read(files.into_iter())
        .panic_at_first_error()
        .write_stdio();
}

fn write(output_dir: &Path) {
    docs::read_stdin().panic_at_first_error().write(output_dir);
}

fn blog(site_url: &str, permalink_template: &str, template_dir: &Path, template_data_path: &Path) {
    let template_data = json::read(template_data_path).unwrap_or(json::Value::Null);
    docs::read_stdin()
        .panic_at_first_error()
        .markdown_blog_doc(site_url, permalink_template, template_dir, &template_data)
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
fn liquid(template_data_path: &Path) {
    let template_data = json::read(template_data_path).unwrap_or(json::Value::Null);
    docs::read_stdin()
        .panic_at_first_error()
        .render_liquid(&template_data)
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
