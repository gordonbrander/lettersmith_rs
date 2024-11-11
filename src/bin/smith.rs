use clap::{Parser, Subcommand};
use lettersmith::prelude::*;
use std::env;
use std::path::{Path, PathBuf};

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
    #[command(
        about = "Read docs from text files. Creates docs from text files, assigning sensible defaults. File name becomes title, contents of file become content, etc."
    )]
    Read {
        #[arg(
            help = "File paths to read. Tip: you can use glob patterns to match specific lists of files. Example: smith read posts/*.md"
        )]
        #[arg(value_name = "FILE")]
        files: Vec<PathBuf>,
    },

    #[command(
        about = "Write docs to a directory. Typically used at the end of a chain of piped smith commands to take the stream of JSON docs and write it to disk."
    )]
    Write {
        #[arg(help = "Directory to write docs to")]
        #[arg(value_name = "DIRECTORY")]
        #[arg(default_value = "public")]
        output_dir: PathBuf,
    },

    #[command(
        about = "Read docs from JSON archive files. Deserializes the contents of the JSON and outputs docs to stdout."
    )]
    ReadArchive {
        #[arg(
            help = "File path of archive file to read. Example: smith read-archive build/posts.json"
        )]
        #[arg(value_name = "FILE")]
        file: PathBuf,
    },

    #[command(
        about = "Write docs to a JSON archive file. Typically used to save a transformed collections of documents so you can read them back out in various parts of your build pipeline."
    )]
    WriteArchive {
        #[arg(help = "File to write JSON archive to")]
        #[arg(value_name = "FILE")]
        file: PathBuf,
    },

    #[command(about = "Render templates for blog posts or pages")]
    Blog {
        #[arg(long = "permalink-template")]
        #[arg(default_value = "{parents}/{slug}/index.html")]
        #[arg(help = "Template for rendering permalinks")]
        permalink_template: String,

        #[arg(
            help = "JSON files to include in template context. Example: smith template --data data/*.json"
        )]
        #[arg(long = "data")]
        #[arg(value_name = "FILE")]
        data: Vec<PathBuf>,
    },

    #[command(about = "Render markdown")]
    Markdown {},

    #[command(about = "Set permalink via a template")]
    Permalink {
        #[arg(long = "template")]
        #[arg(default_value = "{parents}/{slug}/index.html")]
        #[arg(help = "Template for rendering permalinks")]
        permalink_template: String,
    },

    #[command(about = "Render doc with the Tera template set on doc's template_path")]
    Template {
        #[arg(
            help = "JSON files to include in template context. Example: smith template --data data/*.json"
        )]
        #[arg(long = "data")]
        #[arg(value_name = "FILE")]
        data: Vec<PathBuf>,
    },

    #[command(
        about = "Generate a tag index from docs. You can use this command to generate a JSON file containing a tag index which you can include in templates via the --data flag"
    )]
    Tagindex {
        #[arg(help = "Output path for data file")]
        #[arg(value_name = "FILE")]
        output_path: PathBuf,

        #[arg(long = "taxonomy")]
        #[arg(default_value = "tags")]
        taxonomy: String,
    },

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
        Commands::Read { files } => read_cmd(files),
        Commands::Write { output_dir } => write_cmd(output_dir.as_path()),
        Commands::ReadArchive { file } => read_archive_cmd(file),
        Commands::WriteArchive { file } => write_archive_cmd(file.as_path()),
        Commands::Markdown {} => markdown_cmd(),
        Commands::Blog {
            permalink_template,
            data,
        } => blog_cmd(&permalink_template, &data, &config),
        Commands::Permalink { permalink_template } => permalink_cmd(&permalink_template),
        Commands::Template { data } => template(&data, &config),
        Commands::Tagindex {
            output_path,
            taxonomy,
        } => tagindex_cmd(taxonomy, output_path),
        Commands::Frontmatter {} => frontmatter_cmd(),
    }
}

/// Read docs from paths
fn read_cmd(files: Vec<PathBuf>) {
    docs::read(files.into_iter())
        .panic_at_first_error()
        .write_stdio();
}

/// Write docs as text files
fn write_cmd(output_dir: &Path) {
    docs::read_stdin().panic_at_first_error().write(output_dir);
}

/// Read docs from JSON file paths
fn read_archive_cmd(file: PathBuf) {
    archive::read(file.as_path())
        .unwrap()
        .into_iter()
        .write_stdio();
}

/// Write docs as JSON files
fn write_archive_cmd(output_dir: &Path) {
    docs::read_stdin()
        .panic_at_first_error()
        .write_archive(output_dir)
        .unwrap();
}

fn markdown_cmd() {
    docs::read_stdin()
        .panic_at_first_error()
        .render_markdown()
        .write_stdio();
}

fn blog_cmd(permalink_template: &str, data_files: &Vec<PathBuf>, config: &Config) {
    let data = json::read_json_files_as_data_map(data_files).unwrap();

    // Set up Tera instance
    let renderer = tera::renderer(&config.templates).unwrap();
    let mut context = tera::context();
    context.insert("data", &data);
    context.insert("site", config);

    docs::read_stdin()
        .panic_at_first_error()
        .blog_doc(permalink_template, &config.site_url, &renderer, &context)
        .panic_at_first_error()
        .write_stdio();
}

fn permalink_cmd(template: &str) {
    docs::read_stdin()
        .panic_at_first_error()
        .set_permalink(template)
        .write_stdio();
}

/// Render Tera templates
fn template(data_files: &Vec<PathBuf>, config: &Config) {
    let data = json::read_json_files_as_data_map(data_files).unwrap();

    // Set up Tera instance
    let renderer = tera::renderer(&config.templates).unwrap();
    let mut context = tera::context();
    context.insert("data", &data);
    context.insert("site", config);

    docs::read_stdin()
        .panic_at_first_error()
        .auto_template()
        .render_tera_template(&renderer, &context)
        .panic_at_first_error()
        .write_stdio();
}

/// Index all docs by tag and create JSON doc
fn tagindex_cmd(taxonomy: String, output_path: PathBuf) {
    docs::read_stdin()
        .panic_at_first_error()
        .generate_tag_index_doc(&taxonomy, &output_path)
        .unwrap()
        .write_stdio();
}

/// Parse and uplift frontmatter
fn frontmatter_cmd() {
    docs::read_stdin()
        .panic_at_first_error()
        .parse_and_uplift_frontmatter()
        .write_stdio();
}
