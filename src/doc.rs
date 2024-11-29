use crate::error::Error;
use crate::html::strip_html;
use crate::io::write_file_deep;
use crate::json::{self, get_deep, merge};
use crate::text::{to_slug, truncate_280};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq, Hash, Default)]
pub struct Doc {
    pub id_path: PathBuf,
    pub output_path: PathBuf,
    pub input_path: Option<PathBuf>,
    pub template_path: Option<PathBuf>,
    pub created: DateTime<Utc>,
    pub modified: DateTime<Utc>,
    pub title: String,
    pub summary: String,
    pub content: String,
    pub meta: json::Value,
}

impl Doc {
    pub fn new(
        id_path: PathBuf,
        output_path: PathBuf,
        input_path: Option<PathBuf>,
        template_path: Option<PathBuf>,
        created: DateTime<Utc>,
        modified: DateTime<Utc>,
        title: String,
        summary: String,
        content: String,
        meta: json::Value,
    ) -> Self {
        Doc {
            id_path: id_path.into(),
            output_path: output_path.into(),
            input_path: input_path.map(|p| p.into()),
            template_path: template_path.map(|p| p.into()),
            created: created.into(),
            modified: modified.into(),
            title: title.into(),
            summary: summary.into(),
            content: content.into(),
            meta,
        }
    }

    /// Create a draft doc from just an id_path
    pub fn draft(id_path: impl AsRef<Path>) -> Self {
        let path_ref = id_path.as_ref();
        Doc {
            id_path: path_ref.to_path_buf(),
            output_path: path_ref.to_path_buf(),
            ..Default::default()
        }
    }

    /// Load a document from a file path.
    pub fn read(path: impl AsRef<Path>) -> Result<Self, Error> {
        let path = path.as_ref();
        let metadata = std::fs::metadata(path)?;
        let content = std::fs::read_to_string(path)?;
        let title = path.file_stem().unwrap().to_string_lossy().into_owned();

        Ok(Doc::new(
            path.into(),
            path.into(),
            Some(path.to_owned()),
            None,
            metadata.created()?.into(),
            metadata.modified()?.into(),
            title,
            "".to_string(),
            content,
            serde_json::Value::Null,
        ))
    }

    /// Write the doc to its output path.
    /// Returns a result containing the write path of the file on success.
    pub fn write(&self, output_dir: impl AsRef<Path>) -> Result<PathBuf, Error> {
        let write_path = output_dir.as_ref().join(&self.output_path);
        write_file_deep(&write_path, &self.content)?;
        Ok(write_path)
    }

    /// Write doc to stdio
    /// - JSON serialized docs are printed to stdout
    /// - Serialization failures are printed to stderr
    pub fn write_stdio(&self) {
        let serialized = serde_json::to_string(self);
        match serialized {
            Ok(json) => {
                println!("{}", json);
            }
            Err(err) => {
                eprintln!("Error serializing doc: {:?}", err);
            }
        }
    }

    pub fn set_output_path(mut self, output_path: impl AsRef<Path>) -> Self {
        self.output_path = output_path.as_ref().to_path_buf();
        self
    }

    pub fn set_created(mut self, created: DateTime<Utc>) -> Self {
        self.created = created;
        self
    }

    pub fn set_modified(mut self, modified: DateTime<Utc>) -> Self {
        self.modified = modified;
        self
    }

    pub fn set_title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    /// Get title field as a slug.
    /// "My Title" is returned as "my-title".
    pub fn get_title_slug(&self) -> String {
        to_slug(&self.title)
    }

    pub fn set_content(mut self, content: impl Into<String>) -> Self {
        self.content = content.into();
        self
    }

    pub fn set_summary(mut self, summary: impl Into<String>) -> Self {
        self.summary = summary.into();
        self
    }

    pub fn set_summary_if_empty(self, summary: impl Into<String>) -> Self {
        if self.summary.is_empty() {
            self.set_summary(summary)
        } else {
            self
        }
    }

    /// Generate a summary from content if no summary has already been assigned.
    pub fn auto_summary(self) -> Self {
        let summary = truncate_280(&strip_html(&self.content));
        self.set_summary_if_empty(summary)
    }

    /// Set template, overwriting whatever was there previously
    pub fn set_template(mut self, template_path: impl Into<PathBuf>) -> Self {
        self.template_path = Some(template_path.into());
        self
    }

    /// Set template based on parent directory name.
    /// - Uses parent path to assign template
    /// - Or falls back to `default.html` if no parent
    /// - If a template is already assigned to doc, skips and does nothing
    ///
    /// For example:
    /// - A doc with id_path `posts/a.md` gets assigned
    ///   `posts.html`.
    /// - A doc with id_path `pages/company/about.md` gets assigned
    ///   `pages/company.html`
    pub fn auto_template(self) -> Self {
        if self.template_path.is_none() {
            let file_name: String = self
                .id_path
                .parent()
                .and_then(|p| p.file_name())
                .and_then(|name| name.to_str())
                .map(|name| format!("{}.html", name).to_string())
                .unwrap_or_else(|| "default.html".to_string());
            self.set_template(file_name)
        } else {
            self
        }
    }

    /// Set output path extension.
    pub fn set_extension(mut self, extension: &str) -> Self {
        self.output_path.set_extension(extension);
        self
    }

    /// Set output path extension to ".md".
    pub fn set_extension_md(self) -> Self {
        self.set_extension("md")
    }

    /// Set output path extension to ".html".
    pub fn set_extension_html(self) -> Self {
        self.set_extension("html")
    }

    /// Path into meta, getting value if it exists
    /// Example: `doc.meta('music.artist.name')`
    pub fn meta(self, path: &str) -> Option<json::Value> {
        get_deep(&self.meta, path)
    }

    pub fn set_meta(mut self, meta: json::Value) -> Self {
        self.meta = meta;
        self
    }

    /// Merge new meta into existing meta
    pub fn merge_meta(mut self, patch: json::Value) -> Self {
        merge(&mut self.meta, patch);
        self
    }

    /// Uplift metadata, looking for blessed fields and assigning values to doc:
    /// - title
    /// - summary
    /// - created
    /// - modified
    /// - permalink
    /// - template
    pub fn uplift_meta(mut self) -> Self {
        if let Some(json::Value::String(title)) = self.meta.get("title") {
            self.title = title.to_string();
        }
        if let Some(json::Value::String(summary)) = self.meta.get("summary") {
            self.summary = summary.to_string();
        }
        if let Some(json::Value::String(created)) = self.meta.get("created") {
            if let Ok(created) = created.parse() {
                self.created = created;
            }
        }
        if let Some(json::Value::String(modified)) = self.meta.get("modified") {
            if let Ok(modified) = modified.parse() {
                self.modified = modified;
            }
        }
        if let Some(json::Value::String(permalink)) = self.meta.get("permalink") {
            self.output_path = PathBuf::from(permalink);
        }
        if let Some(json::Value::String(template_path)) = self.meta.get("template") {
            self.template_path = Some(PathBuf::from(template_path));
        }
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_new() {
        let doc = Doc::new(
            "test.md".into(),
            "test.html".into(),
            None,
            None,
            Utc::now(),
            Utc::now(),
            "Test Title".into(),
            "Test Summary".into(),
            "Test Content".into(),
            json!(null),
        );

        assert_eq!(doc.title, "Test Title");
        assert_eq!(doc.summary, "Test Summary");
        assert_eq!(doc.content, "Test Content");
    }

    #[test]
    fn test_draft() {
        let doc = Doc::draft("test.md");
        assert_eq!(doc.id_path, PathBuf::from("test.md"));
        assert_eq!(doc.output_path, PathBuf::from("test.md"));
        assert!(doc.input_path.is_none());
        assert!(doc.template_path.is_none());
    }

    #[test]
    fn test_write() -> Result<(), Error> {
        let dir = tempdir()?;
        let doc = Doc::new(
            "test.md".into(),
            "test.html".into(),
            None,
            None,
            Utc::now(),
            Utc::now(),
            "Test".into(),
            "Summary".into(),
            "Content".into(),
            json!(null),
        );

        let path = doc.write(&dir)?;
        assert!(path.exists());
        assert_eq!(fs::read_to_string(path)?, "Content");
        Ok(())
    }

    #[test]
    fn test_title_slug() {
        let doc = Doc::draft("test.md").set_title("My Test Title");
        assert_eq!(doc.get_title_slug(), "my-test-title");
    }

    #[test]
    fn test_auto_template() {
        let doc = Doc::draft("posts/test.md").auto_template();
        assert_eq!(doc.template_path, Some(PathBuf::from("posts.html")));

        let doc = Doc::draft("test.md").auto_template();
        assert_eq!(doc.template_path, Some(PathBuf::from("default.html")));
    }

    #[test]
    fn test_uplift_meta() {
        let meta = json!({
            "title": "Meta Title",
            "summary": "Meta Summary",
            "permalink": "meta.html",
            "template": "meta.html"
        });

        let doc = Doc::draft("test.md").set_meta(meta).uplift_meta();

        assert_eq!(doc.title, "Meta Title");
        assert_eq!(doc.summary, "Meta Summary");
        assert_eq!(doc.output_path, PathBuf::from("meta.html"));
        assert_eq!(doc.template_path, Some(PathBuf::from("meta.html")));
    }

    #[test]
    fn test_merge_meta() {
        let initial = json!({"a": 1, "b": {"c": 2}});
        let patch = json!({"b": {"d": 3}});

        let doc = Doc::draft("test.md").set_meta(initial).merge_meta(patch);

        assert_eq!(doc.meta.get("a").unwrap(), 1);
        assert_eq!(doc.meta.get("b").unwrap().get("c").unwrap(), 2);
        assert_eq!(doc.meta.get("b").unwrap().get("d").unwrap(), 3);
    }
}
