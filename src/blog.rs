use crate::config::Config;
use crate::doc::Doc;
use crate::docs::{DocResults, Docs};
use crate::error::Error;
use crate::permalink::PermalinkConfig;

impl Doc {
    fn markdown_doc(self, config: &Config) -> Result<Doc, Error> {
        let base_url: &str = &config.site_url.as_str();
        let config_json = serde_json::to_value(config)?;
        self.parse_and_uplift_frontmatter()
            .autotemplate("templates")
            .render_markdown()
            .absolutize_urls(base_url)
            .render_liquid(&config_json)
    }

    pub fn markdown_post(self, config: &Config) -> Result<Doc, Error> {
        let doc = self.markdown_doc(config)?;
        let plugin: PermalinkConfig = config
            .get_plugin_config("permalink")
            .unwrap_or(PermalinkConfig::default());
        Ok(doc.set_permalink(plugin.post))
    }

    pub fn markdown_page(self, config: &Config) -> Result<Doc, Error> {
        let doc = self.markdown_doc(config)?;
        let plugin: PermalinkConfig = config
            .get_plugin_config("permalink")
            .unwrap_or(PermalinkConfig::default());
        Ok(doc.set_permalink(plugin.page))
    }
}

pub trait BlogDocs: Docs {
    fn markdown_post(self, config: &Config) -> impl DocResults {
        self.map(|doc| doc.markdown_post(config))
    }

    fn markdown_page(self, config: &Config) -> impl DocResults {
        self.map(|doc| doc.markdown_page(config))
    }
}

impl<I> BlogDocs for I where I: Docs {}
