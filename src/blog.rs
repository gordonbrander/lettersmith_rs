use crate::absolutize::AbsolutizableDocs;
use crate::config::Config;
use crate::docs::{DocResults, Docs};
use crate::permalink::PermalinkDocs;
use crate::prelude::{FrontmatterDocs, MarkdownDocs};
use crate::tera::TeraDocs;

pub trait BlogDocs: Docs {
    fn markdown_blog_doc_with_config(
        self,
        permalink_template: &str,
        config: &Config,
    ) -> impl DocResults {
        self.parse_and_uplift_frontmatter()
            .set_permalink(permalink_template)
            .autotemplate()
            .render_markdown()
            .absolutize_urls(&config.site_url)
            .render_tera_template_with_config(config)
    }
}

impl<I> BlogDocs for I where I: Docs {}
