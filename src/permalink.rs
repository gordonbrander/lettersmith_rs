use crate::doc::Doc;
use crate::docs::Docs;
use crate::text::to_slug;
use crate::token_template;
use std::collections::HashMap;

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

    /// Set doc permalink (output path) using a template
    pub fn permalink(self, permalink_template: impl Into<String>) -> Self {
        let parts = self.get_permalink_template_parts().unwrap_or_default();
        let output_path = token_template::render(permalink_template, &parts);
        self.set_output_path(output_path)
    }

    /// Set blog-style permalink (`yyyy/mm/dd/slug/index.html`)
    pub fn blog_permalink(self) -> Self {
        self.permalink("{yyyy}/{mm}/{dd}/{slug}/index.html")
    }

    /// Set page-style permalink (`parent/directories/slug/index.html`)
    pub fn page_permalink(self) -> Self {
        self.permalink("{parents}/{slug}/index.html")
    }
}

pub trait PermalinkDocs: Docs {
    /// Set doc permalink (output path) using a template
    fn permalink(self, permalink_template: impl Into<String>) -> impl Docs {
        let permalink_template: String = permalink_template.into();
        self.map(move |doc| doc.permalink(&permalink_template))
    }

    /// Set blog-style permalink (`yyyy/mm/dd/slug/index.html`)
    fn blog_permalink(self) -> impl Docs {
        self.map(|doc| doc.blog_permalink())
    }

    /// Set page-style permalink (`parent/directories/slug/index.html`)
    fn page_permalink(self) -> impl Docs {
        self.map(|doc| doc.page_permalink())
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

        let permalink_doc = doc.permalink("{yyyy}/{mm}/{dd}/{stem}/index.html");
        assert_eq!(
            permalink_doc.output_path,
            PathBuf::from("2023/05/15/test_file/index.html")
        );
    }
}
