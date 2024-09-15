use crate::io::write_file_deep;
use crate::json::{self, merge};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use toml;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Doc {
    pub id_path: PathBuf,
    pub output_path: PathBuf,
    pub input_path: Option<PathBuf>,
    pub created: DateTime<Utc>,
    pub modified: DateTime<Utc>,
    pub title: String,
    pub content: String,
    pub meta: json::Value,
    pub template: String,
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
        meta: json::Value,
        template: impl Into<String>,
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

    /// Load a document from a file path.
    pub fn load(path: impl AsRef<Path>) -> std::io::Result<Self> {
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
            serde_json::Value::Null,
            String::new(),
        ))
    }

    /// Write the document to its output path.
    pub fn write(&self, output_dir: impl AsRef<Path>) -> std::io::Result<()> {
        let write_path = output_dir.as_ref().join(&self.output_path);
        write_file_deep(write_path, &self.content)
    }

    /// Merge new meta into existing meta
    pub fn merge_meta(mut self, patch: json::Value) -> Self {
        merge(&mut self.meta, patch);
        self
    }

    /// Set output path extension.
    pub fn set_extension<S: AsRef<OsStr>>(mut self, extension: S) -> Self {
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

    pub fn autotemplate(mut self) -> Self {
        if self.template.is_empty() {
            let template = PathBuf::from(&self.id_path)
                .parent()
                .and_then(|p| p.file_name())
                .and_then(|name| name.to_str())
                .map(|name| format!("{}.html", name))
                .unwrap_or_else(|| "default.html".to_string());
            self.template = template;
        }
        self
    }

    pub fn with_template(mut self, template: impl Into<String>) -> Self {
        if self.template.is_empty() {
            self.template = template.into();
        }
        self
    }

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

    pub fn parse_frontmatter(mut self) -> Self {
        if let Some((frontmatter, content)) = self.content.split_once("---\n") {
            if let Ok(meta) = toml::from_str(frontmatter) {
                self.meta = meta;
                self.content = content.to_string();
            }
        }
        self
    }
}
