use crate::doc::Doc;
use crate::docs::{DocResults, Docs};
use crate::json;
use std::io::Result;

impl Doc {
    pub fn blog_post(self, data: &json::Value) -> Result<Doc> {
        self.parse_frontmatter()
            .uplift_meta()
            .autotemplate("templates")
            .render_markdown()
            .set_extension("html")
            .render_liquid(&data)
    }
}

pub trait BlogDocs: Docs {
    fn blog_post(self, data: &json::Value) -> impl DocResults {
        self.map(|doc| doc.blog_post(data))
    }
}

impl<I> BlogDocs for I where I: Docs {}
