use crate::doc::Doc;
use crate::docs::Docs;

impl Doc {
    pub fn blog_post(self) -> Doc {
        self.parse_frontmatter()
            .uplift_meta()
            .autotemplate()
            .render_markdown()
            .set_extension("html")
    }
}

pub trait BlogDocs: Docs {
    fn blog_post(self) -> impl Docs {
        self.map(|doc| doc.blog_post())
    }
}

impl<I> BlogDocs for I where I: Docs {}
