use crate::doc::Doc;
use chrono::{DateTime, Utc};
use std::path::PathBuf;

/// A struct for representing a stub. A stub is just a container for
/// the summary details of a document. No content, no meta, no template.
///
/// Only properties that implement Hash and Eq, so stubs can be used in HashSets.
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Stub {
    pub id_path: PathBuf,
    pub output_path: PathBuf,
    pub created: DateTime<Utc>,
    pub modified: DateTime<Utc>,
    pub title: String,
    pub summary: String,
}

impl From<&Doc> for Stub {
    fn from(doc: &Doc) -> Self {
        Stub {
            id_path: doc.id_path.clone(),
            output_path: doc.output_path.clone(),
            created: doc.created,
            modified: doc.modified,
            title: doc.title.clone(),
            summary: doc
                .meta
                .get("summary")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
        }
    }
}

/// Create an iterator of Stubs from an iterator of Docs
pub fn stubs(docs: impl Iterator<Item = Doc>) -> impl Iterator<Item = Stub> {
    docs.into_iter().map(|doc| Stub::from(&doc))
}
