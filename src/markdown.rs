use crate::{doc::Doc, html::strip_html};
use pulldown_cmark::{html, Parser};
use tap::Pipe;

pub fn render_markdown(markdown: &str) -> String {
    let parser = Parser::new(markdown);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);
    html_output
}

pub fn strip_markdown(markdown: &str) -> String {
    render_markdown(markdown).pipe(|html_str| strip_html(&html_str))
}

impl Doc {
    /// Render content to markdown
    pub fn render_markdown(mut self) -> Self {
        self.content = render_markdown(&self.content);
        self
    }

    /// Strip markdown from text
    pub fn strip_markdown(mut self) -> Self {
        // render markdown to html
        let html_str = render_markdown(&self.content);
        // then strip the html and assign
        self.content = strip_html(&html_str);
        self
    }
}

/// Render markdown content for all docs
pub fn render_docs_markdown(docs: impl Iterator<Item = Doc>) -> impl Iterator<Item = Doc> {
    docs.map(|doc| doc.render_markdown())
}
