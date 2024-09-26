use crate::doc::Doc;
use crate::docs::{DocResults, Docs};
use crate::error::Error;
use crate::json;
use std::path::Path;

impl Doc {
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
