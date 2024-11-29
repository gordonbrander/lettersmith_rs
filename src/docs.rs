use crate::doc::Doc;
use crate::error::Error;
use crate::io::{dump_errors_to_stderr, panic_at_first_error};
use serde::{Deserialize, Serialize};
use serde_json;
use std::cmp::Ordering;
use std::collections::HashSet;
use std::io::{self, BufRead};
use std::path::{Path, PathBuf};

/// Docs trait is any iterator of Docs
pub trait Docs: Iterator<Item = Doc> + Sized {
    /// Write docs to file system under output_dir
    /// Prints a series of confirmation messages
    fn write(self, output_dir: &Path) {
        for doc in self {
            match doc.write(output_dir) {
                Ok(write_path) => {
                    println!(
                        "Wrote {} → {}",
                        doc.id_path.to_string_lossy(),
                        write_path.to_string_lossy()
                    )
                }
                Err(err) => eprintln!("{:?}", err),
            }
        }
    }

    /// Write docs to stdio
    /// - JSON serialized docs are printed to stdout
    /// - Serialization failures are printed to stderr
    fn write_stdio(self) {
        for doc in self {
            doc.write_stdio();
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
                .expect("Could not convert file name to str")
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

    fn sorted_by(self, key: SortKey, asc: bool) -> impl Docs {
        let docs: Vec<Doc> = self.collect();
        let sorted = match key {
            SortKey::IdPath => sorted_by(docs, |a, b| a.id_path.cmp(&b.id_path), asc),
            SortKey::OutputPath => sorted_by(docs, |a, b| a.output_path.cmp(&b.output_path), asc),
            SortKey::Created => sorted_by(docs, |a, b| a.created.cmp(&b.created), asc),
            SortKey::Modified => sorted_by(docs, |a, b| a.modified.cmp(&b.modified), asc),
            SortKey::Title => sorted_by(docs, |a, b| a.title.cmp(&b.title), asc),
        };
        sorted.into_iter()
    }

    /// Get most recent n docs
    fn most_recent(self, n: usize) -> impl Docs {
        self.sorted_by(SortKey::Created, false).take(n)
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
    fn auto_template(self) -> impl Docs {
        self.map(move |doc| doc.auto_template())
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

#[derive(clap::ValueEnum, Copy, Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SortKey {
    IdPath,
    OutputPath,
    Created,
    Modified,
    Title,
}

impl From<SortKey> for &str {
    fn from(value: SortKey) -> Self {
        match value {
            SortKey::IdPath => "id_path",
            SortKey::Created => "created",
            SortKey::Modified => "modified",
            SortKey::OutputPath => "output_path",
            SortKey::Title => "title",
        }
    }
}

impl TryFrom<&str> for SortKey {
    type Error = Error;
    fn try_from(value: &str) -> Result<Self, Error> {
        match value {
            "id_path" => Ok(SortKey::IdPath),
            "output_path" => Ok(SortKey::OutputPath),
            "created" => Ok(SortKey::Created),
            "modified" => Ok(SortKey::Modified),
            "title" => Ok(SortKey::Title),
            _ => Err(Error::value(format!(
                "String {} does not correspond to any SortKey",
                value
            ))),
        }
    }
}

fn sorted_by<T, F>(mut vec: Vec<T>, mut compare: F, asc: bool) -> Vec<T>
where
    F: FnMut(&T, &T) -> Ordering,
{
    vec.sort_by(|a, b| {
        let ord = compare(a, b);
        if asc {
            ord
        } else {
            ord.reverse()
        }
    });
    vec
}
#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_doc(id: &str, title: &str) -> Doc {
        Doc::draft(id).set_title(title)
    }

    #[test]
    fn test_remove_with_id_path() {
        let docs = vec![
            make_test_doc("doc1.md", "Doc 1"),
            make_test_doc("doc2.md", "Doc 2"),
        ];

        let filtered: Vec<_> = docs.into_iter().remove_with_id_path("doc1.md").collect();

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id_path, PathBuf::from("doc2.md"));
    }

    #[test]
    fn test_filter_matching() {
        let docs = vec![
            make_test_doc("posts/doc1.md", "Doc 1"),
            make_test_doc("pages/doc2.md", "Doc 2"),
        ];

        let filtered: Vec<_> = docs.into_iter().filter_matching("posts/*").collect();

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id_path, PathBuf::from("posts/doc1.md"));
    }

    #[test]
    fn test_remove_drafts() {
        let docs = vec![
            make_test_doc("doc1.md", "Doc 1"),
            make_test_doc("_draft.md", "Draft"),
        ];

        let filtered: Vec<_> = docs.into_iter().remove_drafts().collect();

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id_path, PathBuf::from("doc1.md"));
    }

    #[test]
    fn test_remove_index() {
        let docs = vec![
            make_test_doc("doc1.md", "Doc 1"),
            make_test_doc("index.md", "Index"),
        ];

        let filtered: Vec<_> = docs.into_iter().remove_index().collect();

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id_path, PathBuf::from("doc1.md"));
    }

    #[test]
    fn test_dedupe() {
        let docs = vec![
            make_test_doc("doc1.md", "Doc 1"),
            make_test_doc("doc1.md", "Doc 1 Dupe"),
        ];

        let deduped: Vec<_> = docs.into_iter().dedupe().collect();

        assert_eq!(deduped.len(), 1);
        assert_eq!(deduped[0].id_path, PathBuf::from("doc1.md"));
    }

    #[test]
    fn test_sorted_by() {
        let docs = vec![
            make_test_doc("doc2.md", "B Doc"),
            make_test_doc("doc1.md", "A Doc"),
        ];

        let sorted: Vec<_> = docs.into_iter().sorted_by(SortKey::Title, true).collect();

        assert_eq!(sorted[0].title, "A Doc");
        assert_eq!(sorted[1].title, "B Doc");
    }

    #[test]
    fn test_sort_key_from_string() {
        assert_eq!(SortKey::try_from("title").unwrap(), SortKey::Title);
        assert_eq!(SortKey::try_from("id_path").unwrap(), SortKey::IdPath);
        assert_eq!(
            SortKey::try_from("output_path").unwrap(),
            SortKey::OutputPath
        );
        assert_eq!(SortKey::try_from("created").unwrap(), SortKey::Created);
        assert_eq!(SortKey::try_from("modified").unwrap(), SortKey::Modified);
        assert!(SortKey::try_from("invalid").is_err());
    }

    #[test]
    fn test_sort_key_into_string() {
        assert_eq!(<&str>::from(SortKey::Title), "title");
        assert_eq!(<&str>::from(SortKey::IdPath), "id_path");
        assert_eq!(<&str>::from(SortKey::OutputPath), "output_path");
        assert_eq!(<&str>::from(SortKey::Created), "created");
        assert_eq!(<&str>::from(SortKey::Modified), "modified");
    }

    #[test]
    fn test_set_extension() {
        let docs = vec![make_test_doc("doc1.md", "Doc 1")];

        let modified: Vec<_> = docs.into_iter().set_extension("html").collect();

        eprint!("!!! {:?}", modified[0].output_path);
        assert_eq!(modified[0].output_path.extension().unwrap(), "html");
    }
}
