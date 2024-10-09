use crate::absolutize::AbsolutizableDocs;
use crate::docs::{DocResults, Docs};
use crate::permalink::PermalinkDocs;
use crate::tera::TeraDocs;

pub trait BlogDocs: Docs {
    fn blog_doc(
        self,
        permalink_template: &str,
        site_url: &str,
        renderer: &tera::Tera,
        context: &tera::Context,
    ) -> impl DocResults {
        self.set_permalink(permalink_template)
            .auto_template()
            .absolutize_urls(&site_url)
            .render_tera_template(renderer, context)
    }
}

impl<I> BlogDocs for I where I: Docs {}
