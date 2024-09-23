use crate::error::Error;
use crate::io::read_to_string;
use crate::json;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Reads well-known config properties from lettersmith config file
#[derive(Serialize, Deserialize, Debug)]
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

    /// Open-ended plugin data
    #[serde(default = "plugins_default")]
    pub plugins: HashMap<String, json::Value>,
}

impl Config {
    pub fn read(path: impl AsRef<Path>) -> Result<Self, std::io::Error> {
        let json_string = read_to_string(path)?;
        match serde_json::from_str(&json_string) {
            Ok(value) => Ok(value),
            Err(err) => Err(std::io::Error::new(std::io::ErrorKind::Other, err)),
        }
    }

    pub fn get_plugin_config<T: DeserializeOwned>(&self, key: &str) -> Result<T, Error> {
        let plugin = self
            .plugins
            .get(key)
            .ok_or(Error::new(format!("No plugin for key {}", key)))?
            .to_owned();
        json::from_value(plugin).map_err(|err| Error::new(err.to_string()))
    }
}

fn output_dir_default() -> PathBuf {
    PathBuf::from("public")
}

fn template_dir_default() -> PathBuf {
    PathBuf::from("templates")
}

fn site_url_default() -> String {
    "".to_string()
}

fn data_default() -> json::Value {
    json::json!({})
}

fn plugins_default() -> HashMap<String, json::Value> {
    HashMap::new()
}
