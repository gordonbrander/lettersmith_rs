use crate::io::write_file_deep;
use crate::json::{self, merge};
use crate::text::{first_sentence, to_slug, truncate, truncate_280};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::io::Result;
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq, Hash, Default)]
pub struct Doc {
    pub id_path: PathBuf,
    pub output_path: PathBuf,
    pub input_path: Option<PathBuf>,
    pub created: DateTime<Utc>,
    pub modified: DateTime<Utc>,
    pub title: String,
    pub content: String,
    pub template: String,
    pub meta: json::Value,
}

impl Doc {
    pub fn new(
        id_path: impl AsRef<Path>,
        output_path: impl AsRef<Path>,
        input_path: Option<impl AsRef<Path>>,
        created: DateTime<Utc>,
        modified: DateTime<Utc>,
        title: impl Into<String>,
        content: impl Into<String>,
        template: impl Into<String>,
        meta: json::Value,
    ) -> Self {
        Doc {
            id_path: id_path.as_ref().to_path_buf(),
            output_path: output_path.as_ref().to_path_buf(),
            input_path: input_path.map(|p| p.as_ref().to_path_buf()),
            created,
            modified,
            title: title.into(),
            content: content.into(),
            meta,
            template: template.into(),
        }
    }

    /// Create a draft doc from just an id_path
    pub fn draft(id_path: impl AsRef<Path>) -> Self {
        let path_ref = id_path.as_ref();
        Doc {
            id_path: path_ref.to_path_buf(),
            ..Default::default()
        }
    }

    /// Load a document from a file path.
    pub fn read(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let metadata = std::fs::metadata(path)?;
        let content = std::fs::read_to_string(path)?;
        let title = path.file_stem().unwrap().to_string_lossy().into_owned();

        Ok(Doc::new(
            path,
            path,
            Some(path),
            metadata.created()?.into(),
            metadata.modified()?.into(),
            title,
            content,
            "",
            serde_json::Value::Null,
        ))
    }

    /// Write the document to its output path.
    pub fn write(&self, output_dir: impl AsRef<Path>) -> Result<()> {
        let write_path = output_dir.as_ref().join(&self.output_path);
        write_file_deep(write_path, &self.content)
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

    /// Summarize doc using either meta summary field, or truncating to
    /// max 280 chars.
    pub fn summary_280(&self) -> String {
        if let Some(str) = self.meta.get("summary").and_then(|v| v.as_str()) {
            str.to_string()
        } else {
            truncate_280(&self.content)
        }
    }

    /// Summarize doc using either meta summary field, or truncating to
    /// `max_chars`.
    pub fn summary(&self, max_chars: usize, suffix: &str) -> String {
        if let Some(str) = self.meta.get("summary").and_then(|v| v.as_str()) {
            str.to_string()
        } else {
            truncate(&self.content, max_chars, suffix)
        }
    }

    /// Get first sentence of content
    pub fn first_sentence(&self) -> String {
        first_sentence(&self.content)
    }

    /// Set template, overwriting whatever was there previously
    pub fn set_template(mut self, template: impl Into<String>) -> Self {
        self.template = template.into();
        self
    }

    // Read template file at path and set the contents as the template of this
    // doc.
    pub fn read_and_set_template(self, template_path: impl AsRef<Path>) -> Result<Self> {
        let template_path = template_path.as_ref();
        let template_doc = Doc::read(template_path)?;
        Ok(self.set_template(template_doc.content))
    }

    /// Set template based on parent directory name.
    /// Falls back to `default.html` if no parent.
    pub fn autotemplate(self) -> Result<Self> {
        if self.template.is_empty() {
            let template_path = PathBuf::from(&self.id_path)
                .parent()
                .and_then(|p| p.file_name())
                .and_then(|name| name.to_str())
                .map(|name| format!("{}.html", name))
                .unwrap_or_else(|| "default.html".to_string());
            self.read_and_set_template(template_path)
        } else {
            Ok(self)
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
    /// - created
    /// - modified
    /// - permalink
    /// - template
    pub fn uplift_meta(mut self) -> Self {
        if let Some(json::Value::String(title)) = self.meta.get("title") {
            self.title = title.clone();
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
        if let Some(json::Value::String(template)) = self.meta.get("template") {
            self.template = template.clone();
        }
        self
    }

    /// Parse JSON frontmatter
    pub fn parse_frontmatter(mut self) -> Self {
        if let Some((frontmatter, content)) = self.content.split_once("---\n") {
            if let Ok(meta) = serde_json::from_str(frontmatter) {
                self.meta = meta;
                self.content = content.to_string();
            }
        }
        self
    }
}
