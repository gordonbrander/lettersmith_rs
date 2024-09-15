use crate::doc::Doc;
use pulldown_cmark::{html, Parser};

pub fn markdown_to_html(markdown: &str) -> String {
    let parser = Parser::new(markdown);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);
    html_output
}

impl Doc {
    /// Render content to markdown
    pub fn render_markdown(mut self) -> Self {
        self.content = markdown_to_html(&self.content);
        self
    }
}

/// Render markdown content for all docs
pub fn render_markdown(docs: impl Iterator<Item = Doc>) -> impl Iterator<Item = Doc> {
    docs.map(|doc| doc.render_markdown())
}
