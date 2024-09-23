use crate::doc::Doc;
use crate::docs::{DocResults, Docs};
use crate::error::Error;
use crate::json;

impl Doc {
    fn markdown_doc(self, base_url: &str, data: &json::Value) -> Result<Doc, Error> {
        self.parse_and_uplift_frontmatter()
            .autotemplate("templates")
            .render_markdown()
            .absolutize_urls(base_url)
            .render_liquid(&data)
    }

    pub fn markdown_post(self, base_url: &str, data: &json::Value) -> Result<Doc, Error> {
        Ok(self.markdown_doc(base_url, data)?.set_blog_permalink())
    }

    pub fn markdown_page(self, base_url: &str, data: &json::Value) -> Result<Doc, Error> {
        Ok(self.markdown_doc(base_url, data)?.set_page_permalink())
    }
}

pub trait BlogDocs: Docs {
    fn markdown_post(self, base_url: &str, data: &json::Value) -> impl DocResults {
        self.map(|doc| doc.markdown_post(base_url, data))
    }

    fn markdown_page(self, base_url: &str, data: &json::Value) -> impl DocResults {
        self.map(|doc| doc.markdown_page(base_url, data))
    }
}

impl<I> BlogDocs for I where I: Docs {}
