use crate::doc::Doc;
use crate::docs::{DocResults, Docs};
use crate::error::Error;
use crate::json;

impl Doc {
    pub fn blog_post(self, base_url: &str, data: &json::Value) -> Result<Doc, Error> {
        self.parse_frontmatter()
            .uplift_meta()
            .autotemplate("templates")
            .render_markdown()
            .absolutize_urls(base_url)
            .render_liquid(&data)
    }
}

pub trait BlogDocs: Docs {
    fn blog_post(self, base_url: &str, data: &json::Value) -> impl DocResults {
        self.map(|doc| doc.blog_post(base_url, data))
    }
}

impl<I> BlogDocs for I where I: Docs {}
