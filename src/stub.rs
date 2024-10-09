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
    /// Create a draft stub
    pub fn draft(id_path: impl Into<PathBuf>) -> Self {
        let id_path: PathBuf = id_path.into();
        Stub {
            id_path: id_path.clone(),
            output_path: id_path,
            ..Default::default()
        }
    }

    pub fn get_title_slug(&self) -> String {
        to_slug(&self.title)
    }
}

impl Doc {
    /// Convert Doc into a Stub
    pub fn to_stub(&self) -> Stub {
        Stub {
            id_path: self.id_path.clone(),
            output_path: self.output_path.clone(),
            created: self.created,
            modified: self.modified,
            title: self.title.clone(),
            summary: self.summary.clone(),
        }
    }
}

impl From<&Doc> for Stub {
    fn from(doc: &Doc) -> Self {
        doc.to_stub()
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
            summary: "".to_string(),
            meta: json!({
                "summary": "Test summary".to_string()
            }),
        };

        let doc = doc.uplift_meta();

        let stub = Stub::from(&doc);

        assert_eq!(stub.id_path, doc.id_path);
        assert_eq!(stub.output_path, doc.output_path);
        assert_eq!(stub.created, doc.created);
        assert_eq!(stub.modified, doc.modified);
        assert_eq!(stub.title, doc.title);
        assert_eq!(stub.summary, "Test summary");
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
                summary: "".to_string(),
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
                summary: "".to_string(),
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
