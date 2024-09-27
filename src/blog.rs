use crate::doc::Doc;
use crate::docs::{DocResults, Docs};
use crate::error::Error;
use crate::json;
use std::path::Path;

impl Doc {
    /// Processes a markdown blog doc through several steps.
    ///
    /// Performs the following:
    /// 1. Parses and uplifts the frontmatter
    /// 2. Sets the permalink based on the provided template
    /// 3. Applies an auto-template from the specified directory
    /// 4. Renders the markdown content
    /// 5. Absolutizes URLs using the provided site URL
    /// 6. Renders the document using Liquid templating with the given template data
    ///
    /// # Arguments
    ///
    /// * `site_url` - The base URL of the site
    /// * `permalink_template` - The template string for generating permalinks
    /// * `template_dir` - The directory containing template files
    /// * `template_data` - JSON data to be used in Liquid templating
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the processed `Doc` if successful, or an
    /// `Error` if any step fails.
    pub fn markdown_blog_doc(
        self,
        site_url: &str,
        permalink_template: &str,
        template_dir: &Path,
        template_data: &json::Value,
    ) -> Result<Doc, Error> {
        self.parse_and_uplift_frontmatter()
            .set_permalink(permalink_template)
            .autotemplate(template_dir)
            .render_markdown()
            .absolutize_urls(site_url)
            .render_liquid(template_data)
    }
}

pub trait BlogDocs: Docs {
    fn markdown_blog_doc(
        self,
        site_url: &str,
        permalink_template: &str,
        template_dir: &Path,
        template_data: &json::Value,
    ) -> impl DocResults {
        self.map(|doc| {
            doc.markdown_blog_doc(site_url, permalink_template, template_dir, template_data)
        })
    }
}

impl<I> BlogDocs for I where I: Docs {}
