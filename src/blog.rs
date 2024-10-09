use crate::absolutize::AbsolutizableDocs;
use crate::docs::{DocResults, Docs};
use crate::permalink::PermalinkDocs;
use crate::prelude::{FrontmatterDocs, MarkdownDocs};
use crate::tera::TeraDocs;

pub trait BlogDocs: Docs {
    fn markdown_blog_doc(
        self,
        permalink_template: &str,
        site_url: &str,
        renderer: &tera::Tera,
        context: &tera::Context,
    ) -> impl DocResults {
        self.parse_and_uplift_frontmatter()
            .set_permalink(permalink_template)
            .auto_template()
            .render_markdown()
            .absolutize_urls(&site_url)
            .render_tera_template(renderer, context)
    }
}

impl<I> BlogDocs for I where I: Docs {}
