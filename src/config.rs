use crate::error::{Error, ErrorKind};
use crate::json;
use serde::{Deserialize, Serialize};
use std::fs::read_to_string;
use std::path::{Path, PathBuf};

/// Reads well-known config properties from lettersmith config file
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Config {
    /// Directory to write built files to
    #[serde(default = "output_dir_default")]
    pub output_dir: PathBuf,

    /// Directory where templates are stored
    #[serde(default = "template_dir_default")]
    pub template_dir: PathBuf,

    /// The site's URL
    #[serde(default = "site_url_default")]
    pub site_url: String,

    /// The site's title
    #[serde(default)]
    pub site_title: String,

    /// The site's description
    #[serde(default)]
    pub site_description: String,

    /// The site's author
    #[serde(default)]
    pub site_author: String,

    /// Data to be passed into template
    #[serde(default = "data_default")]
    pub data: json::Value,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            output_dir: output_dir_default(),
            template_dir: template_dir_default(),
            site_url: site_url_default(),
            site_title: String::default(),
            site_description: String::default(),
            site_author: String::default(),
            data: data_default(),
        }
    }
}

fn output_dir_default() -> PathBuf {
    PathBuf::from("public")
}

fn template_dir_default() -> PathBuf {
    PathBuf::from("templates")
}

fn site_url_default() -> String {
    "/".to_string()
}

fn data_default() -> json::Value {
    json::json!({})
}

impl Config {
    /// Read config from file at path
    pub fn read(path: impl AsRef<Path>) -> Result<Self, Error> {
        let json_string = read_to_string(path)?;
        let config: Self = serde_json::from_str(&json_string)?;
        Ok(config)
    }

    /// Convert this config object into a `json::Value`
    pub fn to_json(&self) -> Result<json::Value, Error> {
        serde_json::to_value(self)
            .map_err(|err| Error::new(ErrorKind::Json(err), "Could not serialize Config to JSON"))
    }
}
