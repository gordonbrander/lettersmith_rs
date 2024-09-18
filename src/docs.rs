use crate::doc::Doc;
use serde_json;
use std::collections::HashSet;
use std::error::Error;
use std::io::{self, BufRead};
use std::path::Path;

/// Load documents from an iterator of paths.
/// Errors are output to stderr.
/// Returns an iterator of docs.
pub fn read<P, I>(paths: I) -> impl Iterator<Item = Result<Doc, impl Error>>
where
    P: AsRef<Path>,
    I: Iterator<Item = P>,
{
    paths.map(|path| Doc::read(path))
}

/// Parse JSON documents from stdin as line-separated JSON.
/// Returns an iterator of doc results.
pub fn read_stdin() -> impl Iterator<Item = Result<Doc, impl Error>> {
    io::stdin()
        .lock()
        .lines()
        .filter_map(Result::ok)
        .map(|line| serde_json::from_str(&line))
}

/// Write docs to the output directory.
/// Logs successful writes to stdout.
/// Logs errors to stderr.
pub fn write(docs: impl Iterator<Item = Doc>, output_dir: &Path) {
    for doc in docs {
        match doc.write(output_dir) {
            Ok(_) => println!("Wrote {:?} to {:?}", doc.id_path, doc.output_path),
            Err(err) => eprintln!("Error writing doc: {}", err),
        }
    }
}

/// Write documents to stdout as line-separated JSON.
/// Any errors are output to stderr.
pub fn write_stdio(docs: impl Iterator<Item = Doc>) {
    for doc in docs {
        let serialized = serde_json::to_string(&doc);
        match serialized {
            Ok(json) => {
                println!("{}", json);
            }
            Err(err) => {
                eprintln!("Error serializing doc: {}", err);
            }
        }
    }
}

/// Filter out docs with a given id_path
pub fn remove_with_id_path(
    docs: impl Iterator<Item = Doc>,
    id_path: impl AsRef<Path>,
) -> impl Iterator<Item = Doc> {
    docs.filter(move |doc| doc.id_path != id_path.as_ref())
}

/// Filter docs who's id_path matches a glob pattern.
pub fn filter_matching(
    docs: impl Iterator<Item = Doc>,
    glob_pattern: &str,
) -> impl Iterator<Item = Doc> {
    let matcher = glob::Pattern::new(glob_pattern).expect("Invalid glob pattern");
    docs.filter(move |doc| {
        matcher.matches(
            doc.id_path
                .to_str()
                .expect("Could not convert path to string"),
        )
    })
}

/// Filter out docs who's file name in the id_path starts with an underscore.
pub fn remove_drafts(docs: impl Iterator<Item = Doc>) -> impl Iterator<Item = Doc> {
    docs.filter(|doc| {
        !Path::new(&doc.id_path)
            .file_name()
            .expect("Could not get file name")
            .to_str()
            .expect("Cound not convert file name to str")
            .starts_with('_')
    })
}

/// Filter out docs who's file name in the id_path is "index".
pub fn remove_index(docs: impl Iterator<Item = Doc>) -> impl Iterator<Item = Doc> {
    docs.filter(|doc| {
        Path::new(&doc.id_path)
            .file_stem()
            .expect("Could not get file stem from path")
            != "index"
    })
}

/// De-duplicate docs by id_path
pub fn dedupe(docs: impl Iterator<Item = Doc>) -> impl Iterator<Item = Doc> {
    let mut seen = HashSet::new();
    docs.filter(move |doc| seen.insert(doc.id_path.clone()))
}

/// Sort docs by created date
pub fn sort_by_created(docs: impl Iterator<Item = Doc>) -> impl Iterator<Item = Doc> {
    let mut docs_vec: Vec<Doc> = docs.into_iter().collect();
    docs_vec.sort_by(|a, b| b.created.cmp(&a.created));
    docs_vec.into_iter()
}

/// Sort docs by modified date
pub fn sort_by_modified(docs: impl Iterator<Item = Doc>) -> impl Iterator<Item = Doc> {
    let mut docs_vec: Vec<Doc> = docs.collect();
    docs_vec.sort_by(|a, b| b.modified.cmp(&a.modified));
    docs_vec.into_iter()
}

/// Sort docs by title (A-Z)
pub fn sort_by_title(docs: impl Iterator<Item = Doc>) -> impl Iterator<Item = Doc> {
    let mut docs_vec: Vec<Doc> = docs.collect();
    docs_vec.sort_by(|a, b| a.title.cmp(&b.title));
    docs_vec.into_iter()
}

/// Get most recent n docs
pub fn most_recent(docs: impl Iterator<Item = Doc>, n: usize) -> impl Iterator<Item = Doc> {
    sort_by_created(docs).take(n)
}

pub fn autotemplate(docs: impl Iterator<Item = Doc>) -> impl Iterator<Item = Doc> {
    docs.map(|doc| doc.autotemplate())
}

pub fn set_extension<'a>(
    docs: impl Iterator<Item = Doc> + 'a,
    extension: &'a str,
) -> impl Iterator<Item = Doc> + 'a {
    docs.map(move |doc| doc.set_extension(&extension))
}

pub fn set_extension_html(docs: impl Iterator<Item = Doc>) -> impl Iterator<Item = Doc> {
    docs.map(|doc| doc.set_extension_html())
}

pub fn set_template<'a>(
    docs: impl Iterator<Item = Doc> + 'a,
    template: &'a str,
) -> impl Iterator<Item = Doc> + 'a {
    docs.map(move |doc| doc.set_template(&template))
}

pub fn set_template_if_needed<'a>(
    docs: impl Iterator<Item = Doc> + 'a,
    template: &'a str,
) -> impl Iterator<Item = Doc> + 'a {
    docs.map(move |doc| doc.set_template_if_needed(template))
}
