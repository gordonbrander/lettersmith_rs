use crate::text::to_slug;
use crate::{doc::Doc, docs::Docs};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// A struct for representing a stub. A stub is just a container for
/// the summary details of a document. No content, no meta, no template.
///
/// Only properties that implement Hash and Eq, so stubs can be used in HashSets.
#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq, Hash, Default)]
pub struct Stub {
    pub id_path: PathBuf,
    pub output_path: PathBuf,
    pub created: DateTime<Utc>,
    pub modified: DateTime<Utc>,
    pub title: String,
    pub summary: String,
}

impl Stub {
    pub fn get_title_slug(&self) -> String {
        to_slug(&self.title)
    }
}

impl From<&Doc> for Stub {
    fn from(doc: &Doc) -> Self {
        Stub {
            id_path: doc.id_path.clone(),
            output_path: doc.output_path.clone(),
            created: doc.created,
            modified: doc.modified,
            title: doc.title.clone(),
            summary: doc.summary_280(),
        }
    }
}

pub trait StubDocs: Docs {
    /// Create an iterator of Stubs from an iterator of Docs
    fn stubs(self) -> impl Iterator<Item = Stub> {
        self.map(|doc| Stub::from(&doc))
    }
}

impl<I> StubDocs for I where I: Docs {}

pub trait Stubs: Iterator<Item = Stub> {
    fn index_by_slug(stubs: impl Stubs) -> std::collections::HashMap<String, Stub> {
        stubs.map(|stub| (stub.get_title_slug(), stub)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::doc::Doc;
    use crate::json::json;

    #[test]
    fn test_stub_from_doc() {
        let doc = Doc {
            id_path: PathBuf::from("test/path"),
            input_path: None,
            output_path: PathBuf::from("output/path"),
            template_path: None::<PathBuf>,
            created: Utc::now(),
            modified: Utc::now(),
            title: "Test Title".to_string(),
            content: "Test content".to_string(),
            meta: json!({
                "summary": "Test summary".to_string()
            }),
        };

        let stub = Stub::from(&doc);

        assert_eq!(stub.id_path, doc.id_path);
        assert_eq!(stub.output_path, doc.output_path);
        assert_eq!(stub.created, doc.created);
        assert_eq!(stub.modified, doc.modified);
        assert_eq!(stub.title, doc.title);
        assert_eq!(stub.summary, "Test summary");
    }

    #[test]
    fn test_stub_from_doc_without_summary() {
        let doc = Doc {
            id_path: PathBuf::from("test/path"),
            input_path: None,
            output_path: PathBuf::from("output/path"),
            template_path: None::<PathBuf>,
            created: Utc::now(),
            modified: Utc::now(),
            title: "Test Title".to_string(),
            content:
                "This is a test content that should be truncated to 280 characters for the summary."
                    .to_string(),
            meta: json!({}),
        };

        let stub = Stub::from(&doc);

        assert_eq!(stub.summary, doc.summary_280());
    }

    #[test]
    fn test_stubs_iterator() {
        let docs = vec![
            Doc {
                id_path: PathBuf::from("test/path1"),
                input_path: None,
                output_path: PathBuf::from("output/path1"),
                template_path: None::<PathBuf>,
                created: Utc::now(),
                modified: Utc::now(),
                title: "Test Title 1".to_string(),
                content: "Test content 1".to_string(),
                meta: json!({}),
            },
            Doc {
                id_path: PathBuf::from("test/path2"),
                input_path: None,
                output_path: PathBuf::from("output/path2"),
                template_path: None::<PathBuf>,
                created: Utc::now(),
                modified: Utc::now(),
                title: "Test Title 2".to_string(),
                content: "Test content 2".to_string(),
                meta: json!({}),
            },
        ];

        let stubs: Vec<Stub> = docs.into_iter().stubs().collect();

        assert_eq!(stubs.len(), 2);
        assert_eq!(stubs[0].title, "Test Title 1");
        assert_eq!(stubs[1].title, "Test Title 2");
    }
}
