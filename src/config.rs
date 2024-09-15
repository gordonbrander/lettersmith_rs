use crate::io::write_file_deep;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use toml;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Config {
    site_url: String,
    site_title: String,
    site_description: String,
    site_author: String,
    /// A map of plugin names to their configuration (provided as JSON-typed values).
    plugins: HashMap<String, toml::Value>,
}

impl Config {
    pub fn new(
        site_url: impl Into<String>,
        site_title: impl Into<String>,
        site_description: impl Into<String>,
        site_author: impl Into<String>,
        plugins: HashMap<String, toml::Value>,
    ) -> Self {
        Config {
            site_url: site_url.into(),
            site_title: site_title.into(),
            site_description: site_description.into(),
            site_author: site_author.into(),
            plugins,
        }
    }

    /// Load a configuration from a file path.
    pub fn load(path: impl AsRef<Path>) -> std::io::Result<Self> {
        let path = path.as_ref();
        let content = std::fs::read_to_string(path)?;
        match toml::from_str(&content) {
            Ok(config) => Ok(config),
            Err(e) => Err(std::io::Error::new(std::io::ErrorKind::Other, e)),
        }
    }

    /// Load a configuration from the default path.
    pub fn load_default() -> std::io::Result<Self> {
        Self::load(Path::new("lettersmith.toml"))
    }

    /// Save a configuration to a file path.
    pub fn write(&self, path: impl AsRef<Path>) -> std::io::Result<()> {
        match toml::to_string(self) {
            Ok(content) => write_file_deep(path, &content),
            Err(e) => Err(std::io::Error::new(std::io::ErrorKind::Other, e)),
        }
    }
}
