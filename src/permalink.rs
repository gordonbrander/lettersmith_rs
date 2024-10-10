use crate::doc::Doc;
use crate::docs::Docs;
use crate::text::to_slug;
use crate::token_template;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Sluggify all the normal components of a path.
pub fn sluggify_path(path: &Path) -> PathBuf {
    path.components()
        .map(|component| match component {
            std::path::Component::Normal(os_str) => os_str
                .to_str()
                .map(to_slug)
                .unwrap_or_else(|| os_str.to_string_lossy().into_owned()),
            _ => component.as_os_str().to_string_lossy().into_owned(),
        })
        .collect()
}

/// Makes an ugly path into a "nice path". Nice paths are paths that end with
/// an index file, so you can reference them `like/this/` instead of
/// `like/This.html`.
///
/// ugly_path:
///     some/File.md
///
/// nice_path:
///     some/file/index.html
///
/// ugly_path:
///     some/index.md
///
/// nice_path:
///     some/index.html
pub fn to_nice_path(path: &Path) -> Option<PathBuf> {
    let sluggified_path = sluggify_path(path);
    let stem = sluggified_path.file_stem()?.to_str()?;
    if stem == "index" {
        // If it is an index file, canonicalize it as index.html, but leave
        // it flat and don't add an additional subdir
        // (do not do `index/index.html`).
        Some(sluggified_path.with_file_name("index.html"))
    } else {
        let parent = sluggified_path.parent()?;
        let nice = parent.join(format!("{}/{}", stem, "index.html"));
        Some(nice)
    }
}

impl Doc {
    /// Extracts permalink template parts from a document.
    ///
    /// Returns `Some(HashMap<&str, String>)` containing the following key-value pairs:
    /// - "name": File name including extension
    /// - "stem": File name excluding extension
    /// - "slug": URL-friendly version of the stem
    /// - "ext": File extension
    /// - "parents": All parent directories
    /// - "parent": Closest parent directory
    /// - "yyyy": Full year (4 digits)
    /// - "yy": Year (2 digits)
    /// - "mm": Month (2 digits)
    /// - "dd": Day (2 digits)
    ///
    /// Returns `None` if any required path component is missing.
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

    /// Sets the document's permalink (output path) using a provided template.
    ///
    /// The template can include placeholders that will be replaced with
    /// corresponding values from the document's metadata. Available placeholders are:
    /// - {name}: File name including extension
    /// - {stem}: File name excluding extension
    /// - {slug}: URL-friendly version of the stem
    /// - {ext}: File extension
    /// - {parents}: All parent directories
    /// - {parent}: Closest parent directory
    /// - {yyyy}: Full year (4 digits)
    /// - {yy}: Year (2 digits)
    /// - {mm}: Month (2 digits)
    /// - {dd}: Day (2 digits)
    ///
    /// # Arguments
    ///
    /// * `permalink_template` - A string or string-like object representing the template
    ///
    /// # Returns
    ///
    /// Returns `Self` with the updated output path.
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

    #[test]
    fn test_nice_path() {
        let path = Path::new("foo bar/Baz/Some.md");
        let nice = to_nice_path(path).unwrap();
        assert_eq!(nice, PathBuf::from("foo-bar/baz/some/index.html"));
    }

    #[test]
    fn test_nice_path_index() {
        let path = Path::new("foo bar/Baz/index.md");
        let nice = to_nice_path(path).unwrap();
        assert_eq!(nice, PathBuf::from("foo-bar/baz/index.html"));
    }

    #[test]
    fn test_sluggify_path() {
        let path = Path::new("Foo bar/baZ/INDEX.md");
        let sluggified_path = sluggify_path(path);
        assert_eq!(sluggified_path, PathBuf::from("foo-bar/baz/index.md"));
    }
}
