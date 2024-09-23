use crate::doc::Doc;
use crate::docs::Docs;
use crate::text::to_slug;
use crate::token_template;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Config for permalinks in plugins section of lettersmith config file.
#[derive(Serialize, Deserialize)]
pub struct PermalinkConfig {
    #[serde(default = "default_post_permalink")]
    pub post: String,
    #[serde(default = "default_page_permalink")]
    pub page: String,
}

impl Default for PermalinkConfig {
    fn default() -> Self {
        PermalinkConfig {
            post: default_post_permalink(),
            page: default_page_permalink(),
        }
    }
}

fn default_post_permalink() -> String {
    "{yyyy}/{mm}/{dd}/{slug}/index.html".to_string()
}

fn default_page_permalink() -> String {
    "{parents}/{slug}/index.html".to_string()
}

impl Doc {
    /// Read permalink template parts from a document.
    /// Returns a hashmap of information useful for templating a permalink.
    pub fn get_permalink_template_parts(&self) -> Option<HashMap<&str, String>> {
        let name = self.id_path.file_name()?.to_string_lossy().into_owned();
        let stem = self.id_path.file_stem()?.to_string_lossy().into_owned();
        let slug = to_slug(&stem);
        let ext = self.id_path.extension()?.to_string_lossy().into_owned();
        let parents = self.id_path.parent()?.to_string_lossy().into_owned();
        let parent = self
            .id_path
            .parent()?
            .file_name()?
            .to_string_lossy()
            .into_owned();
        let yyyy = self.created.format("%Y").to_string();
        let yy = self.created.format("%y").to_string();
        let mm = self.created.format("%m").to_string();
        let dd = self.created.format("%d").to_string();
        let mut map = HashMap::new();
        // Name including extension
        map.insert("name", name);
        // Name excluding extension
        map.insert("stem", stem);
        map.insert("slug", slug);
        map.insert("ext", ext);
        // All parents
        map.insert("parents", parents);
        // Just the closest parent
        map.insert("parent", parent);
        map.insert("yyyy", yyyy);
        map.insert("yy", yy);
        map.insert("mm", mm);
        map.insert("dd", dd);
        Some(map)
    }

    /// Set doc permalink (`output_path`) using a template
    pub fn set_permalink(self, permalink_template: impl Into<String>) -> Self {
        let parts = self.get_permalink_template_parts().unwrap_or_default();
        let output_path = token_template::render(permalink_template, &parts);
        self.set_output_path(output_path)
    }

    /// Set blog-style permalink (`yyyy/mm/dd/slug/index.html`)
    pub fn set_blog_permalink(self) -> Self {
        self.set_permalink("{yyyy}/{mm}/{dd}/{slug}/index.html")
    }

    /// Set page-style permalink (`parent/directories/slug/index.html`)
    pub fn set_page_permalink(self) -> Self {
        self.set_permalink("{parents}/{slug}/index.html")
    }
}

pub trait PermalinkDocs: Docs {
    /// Set doc permalink (output path) using a template
    fn set_permalink(self, permalink_template: impl Into<String>) -> impl Docs {
        let permalink_template: String = permalink_template.into();
        self.map(move |doc| doc.set_permalink(&permalink_template))
    }

    /// Set blog-style permalink (`yyyy/mm/dd/slug/index.html`)
    fn set_blog_permalink(self) -> impl Docs {
        self.map(|doc| doc.set_blog_permalink())
    }

    /// Set page-style permalink (`parent/directories/slug/index.html`)
    fn set_page_permalink(self) -> impl Docs {
        self.map(|doc| doc.set_page_permalink())
    }
}

impl<I> PermalinkDocs for I where I: Docs {}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};
    use std::path::PathBuf;

    #[test]
    fn test_get_permalink_template_parts() {
        let doc = Doc {
            id_path: PathBuf::from("a/b/test file.md"),
            created: Utc.with_ymd_and_hms(2023, 5, 15, 0, 0, 0).unwrap(),
            ..Default::default()
        };

        let parts = doc.get_permalink_template_parts().unwrap();

        assert_eq!(parts.get("name"), Some(&"test file.md".to_string()));
        assert_eq!(parts.get("stem"), Some(&"test file".to_string()));
        assert_eq!(parts.get("slug"), Some(&"test-file".to_string()));
        assert_eq!(parts.get("ext"), Some(&"md".to_string()));
        assert_eq!(parts.get("parents"), Some(&"a/b".to_string()));
        assert_eq!(parts.get("parent"), Some(&"b".to_string()));
        assert_eq!(parts.get("yyyy"), Some(&"2023".to_string()));
        assert_eq!(parts.get("yy"), Some(&"23".to_string()));
        assert_eq!(parts.get("mm"), Some(&"05".to_string()));
        assert_eq!(parts.get("dd"), Some(&"15".to_string()));
    }

    #[test]
    fn test_permalink() {
        let doc = Doc {
            id_path: PathBuf::from("parent/test_file.md"),
            created: Utc.with_ymd_and_hms(2023, 5, 15, 0, 0, 0).unwrap(),
            ..Default::default()
        };

        let permalink_doc = doc.set_permalink("{yyyy}/{mm}/{dd}/{stem}/index.html");
        assert_eq!(
            permalink_doc.output_path,
            PathBuf::from("2023/05/15/test_file/index.html")
        );
    }
}
