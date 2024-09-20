use crate::doc::Doc;
use crate::docs::Docs;
use crate::html::strip_html;
use pulldown_cmark::{html, Parser};

pub fn render_markdown(markdown: &str) -> String {
    let parser = Parser::new(markdown);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);
    html_output
}

pub fn strip_markdown(markdown: &str) -> String {
    strip_html(&render_markdown(markdown))
}

impl Doc {
    /// Render content to markdown
    pub fn render_markdown(mut self) -> Self {
        self.content = render_markdown(&self.content);
        self
    }

    /// Strip markdown from text
    pub fn strip_markdown(mut self) -> Self {
        self.content = strip_markdown(&&self.content);
        self
    }
}

pub trait MarkdownDocs: Docs {
    fn render_markdown(self) -> impl Docs {
        self.map(|doc| doc.render_markdown())
    }

    fn strip_markdown(self) -> impl Docs {
        self.map(|doc| doc.strip_markdown())
    }
}

/// Blanket-implement DocIterator for any iterator of docs
impl<I> MarkdownDocs for I where I: Docs {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_markdown() {
        let input = "# Hello\n\nThis is a **test**";
        let expected = "<h1>Hello</h1>\n<p>This is a <strong>test</strong></p>\n";
        assert_eq!(render_markdown(input), expected);
    }

    #[test]
    fn test_strip_markdown() {
        let input = "# Hello\n\nThis is a **test**";
        let expected = "Hello\nThis is a test\n";
        assert_eq!(strip_markdown(input), expected);
    }

    #[test]
    fn test_doc_render_markdown() {
        let doc = Doc::draft("test.md").set_content("# Test");
        let rendered = doc.render_markdown();
        assert_eq!(rendered.content, "<h1>Test</h1>\n");
    }

    #[test]
    fn test_doc_strip_markdown() {
        let doc = Doc::draft("test.md").set_content("# Test");
        let stripped = doc.strip_markdown();
        assert_eq!(stripped.content, "Test\n");
    }

    #[test]
    fn test_markdown_docs_render() {
        let docs = vec![
            Doc::draft("foo.md").set_content("# One"),
            Doc::draft("bar.md").set_content("## Two"),
        ];
        let rendered: Vec<Doc> = docs.into_iter().render_markdown().collect();
        assert_eq!(rendered[0].content, "<h1>One</h1>\n");
        assert_eq!(rendered[1].content, "<h2>Two</h2>\n");
    }

    #[test]
    fn test_markdown_docs_strip() {
        let docs = vec![
            Doc::draft("test.md").set_content("# One"),
            Doc::draft("test.md").set_content("## Two"),
        ];
        let stripped: Vec<Doc> = docs.into_iter().strip_markdown().collect();
        assert_eq!(stripped[0].content, "One\n");
        assert_eq!(stripped[1].content, "Two\n");
    }
}
