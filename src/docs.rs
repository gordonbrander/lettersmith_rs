use crate::doc::Doc;
use crate::error::Error;
use crate::io::{dump_errors_to_stderr, panic_at_first_error};
use serde_json;
use std::collections::HashSet;
use std::io::{self, BufRead};
use std::path::{Path, PathBuf};

/// Docs trait is any iterator of Docs
pub trait Docs: Iterator<Item = Doc> + Sized {
    /// Write docs to file system under output_dir
    /// Outputs a series of JSON result objects
    fn write(self, output_dir: &Path) {
        for doc in self.into_iter() {
            match doc.write(output_dir) {
                Ok(_) => {
                    println!(
                        "Wrote {} → {}",
                        doc.id_path.to_string_lossy(),
                        doc.output_path.to_string_lossy()
                    )
                }
                Err(err) => eprintln!("{}", err),
            }
        }
    }

    /// Write docs to stdio
    /// - JSON serialized docs are printed to stdout
    /// - Serialization failures are printed to stderr
    fn write_stdio(self) {
        for doc in self {
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
    fn remove_with_id_path(self, id_path: impl AsRef<Path>) -> impl Docs {
        self.filter(move |doc| doc.id_path != id_path.as_ref())
    }

    /// Filter docs who's id_path matches a glob pattern.
    fn filter_matching(self, glob_pattern: &str) -> impl Docs {
        let matcher = glob::Pattern::new(glob_pattern).expect("Invalid glob pattern");
        self.filter(move |doc| {
            matcher.matches(
                doc.id_path
                    .to_str()
                    .expect("Could not convert path to string"),
            )
        })
    }

    /// Filter out docs who's file name in the id_path starts with an underscore.
    fn remove_drafts(self) -> impl Docs {
        self.filter(|doc| {
            !Path::new(&doc.id_path)
                .file_name()
                .expect("Could not get file name")
                .to_str()
                .expect("Cound not convert file name to str")
                .starts_with('_')
        })
    }

    /// Filter out docs who's file name in the id_path is "index".
    fn remove_index(self) -> impl Docs {
        self.filter(|doc| {
            Path::new(&doc.id_path)
                .file_stem()
                .expect("Could not get file stem from path")
                != "index"
        })
    }

    /// De-duplicate docs by id_path
    fn dedupe(self) -> impl Docs {
        let mut seen = HashSet::new();
        self.filter(move |doc| seen.insert(doc.id_path.clone()))
    }

    /// Sort docs by created date
    fn sort_by_created(self) -> impl Docs {
        let mut docs_vec: Vec<Doc> = self.into_iter().collect();
        docs_vec.sort_by(|a, b| b.created.cmp(&a.created));
        docs_vec.into_iter()
    }

    /// Sort docs by modified date
    fn sort_by_modified(self) -> impl Docs {
        let mut docs_vec: Vec<Doc> = self.collect();
        docs_vec.sort_by(|a, b| b.modified.cmp(&a.modified));
        docs_vec.into_iter()
    }

    /// Sort docs by title (A-Z)
    fn sort_by_title(self) -> impl Docs {
        let mut docs_vec: Vec<Doc> = self.collect();
        docs_vec.sort_by(|a, b| a.title.cmp(&b.title));
        docs_vec.into_iter()
    }

    /// Get most recent n docs
    fn most_recent(self, n: usize) -> impl Docs {
        self.sort_by_created().take(n)
    }

    /// Set output path extension.
    fn set_extension(self, extension: &str) -> impl Docs {
        self.map(|doc| doc.set_extension(extension))
    }

    /// Set output path extension to ".html".
    fn set_extension_html(self) -> impl Docs {
        self.map(|doc| doc.set_extension_html())
    }

    /// Set template
    fn set_template(self, template_path: impl Into<PathBuf>) -> impl Docs {
        let template_path: PathBuf = template_path.into();
        self.map(move |doc| doc.set_template(&template_path))
    }

    /// Set template based on parent directory name.
    /// Falls back to default.html if no parent.
    fn autotemplate(self) -> impl Docs {
        self.map(move |doc| doc.autotemplate())
    }
}

/// Blanket-implement DocIterator for any iterator of docs
impl<I> Docs for I where I: Iterator<Item = Doc> {}

pub trait DocResults: Iterator<Item = Result<Doc, Error>> + Sized {
    /// Dump errors in result to stderr and unwrap values values.
    /// Returns an Iterator of Doc.
    fn dump_errors_to_stderr(self) -> impl Docs {
        dump_errors_to_stderr(self)
    }

    /// Panic at the first error spotted.
    /// Panic prints a debug error to stderr.
    fn panic_at_first_error(self) -> impl Docs {
        panic_at_first_error(self)
    }
}

impl<I> DocResults for I where I: Iterator<Item = Result<Doc, Error>> {}

/// Load documents from an iterator of paths.
/// Returns an iterator of doc results.
pub fn read(paths: impl Iterator<Item = PathBuf>) -> impl DocResults {
    paths.map(|path| Doc::read(path))
}

/// Parse JSON documents from stdin as line-separated JSON.
/// Returns an iterator of doc results.
pub fn read_stdin() -> impl DocResults {
    io::stdin()
        .lock()
        .lines()
        .filter_map(Result::ok)
        .map(|line| match serde_json::from_str(&line) {
            Ok(doc) => Ok(doc),
            Err(err) => Err(Error::from(err)),
        })
}
