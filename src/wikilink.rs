use crate::doc::Doc;
use crate::docs::Docs;
use crate::html::strip_html;
use crate::markdown::strip_markdown;
use crate::text::{first_sentence, to_slug};
use crate::token_template;
use regex::{self, Regex};
use std::collections::HashMap;
use std::sync::LazyLock;
use tap::Pipe;

static WIKILINK: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\[\[([^\]]+)\]\]").expect("Could not parse regular expression"));

static TRANSCLUDE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^\[\[([^\]]+)\]\]$").expect("Could not parse regular expression")
});

/// Represents a parsed wikilink
pub struct Wikilink {
    pub text: String,
    pub slug: String,
}

impl From<Wikilink> for HashMap<&str, String> {
    fn from(wikilink: Wikilink) -> Self {
        HashMap::from([("text", wikilink.text), ("slug", wikilink.slug)])
    }
}

fn parse_wikilink(wikilink_str: &str) -> Wikilink {
    let inner = wikilink_str.trim_matches(|c| c == '[' || c == ']' || c == ' ');
    if let Some((slug, text)) = inner.split_once('|') {
        Wikilink {
            text: text.trim().to_string(),
            slug: to_slug(slug.trim()),
        }
    } else {
        let text = inner.trim();
        Wikilink {
            text: text.to_string(),
            slug: to_slug(text),
        }
    }
}

pub fn find_wikilinks<'a>(s: &'a str) -> impl Iterator<Item = Wikilink> + 'a {
    WIKILINK
        .captures_iter(s)
        .map(|caps| parse_wikilink(&caps[0]))
}

pub fn strip_wikilinks(text: &str) -> String {
    let text = TRANSCLUDE.replace_all(text, "");
    WIKILINK
        .replace_all(&text, |caps: &regex::Captures| {
            let wikilink = parse_wikilink(&caps[0]);
            wikilink.text
        })
        .into_owned()
}

/// Render wikilinks in text using a template and an index of slugs to stubs.
/// Wikilink will be sluggified, then the slug used to look up a corresponding
/// stub who's values are used to render the wikilink.
/// If no stub is found, the wikilink is replaced with plain text.
/// Otherwise, the wikilink will be replaced by the rendered template.
///
/// Template variables available are:
/// - `text`: the text of the wikilink (display text if using pipe)
/// - `slug`: the sluggified text of the wikilink
/// - `output_path`: the output_path of the stub
/// - `title`: the title of the stub
/// - `summary`: the summary of the stub
pub fn render_wikilinks_with_template(
    text: &str,
    wikilink_template: &str,
    nolink_template: &str,
    slug_to_doc_index: &HashMap<String, Doc>,
) -> String {
    WIKILINK
        .replace_all(&text, |caps: &regex::Captures| {
            let wikilink = parse_wikilink(&caps[0]);
            match slug_to_doc_index.get(&wikilink.slug) {
                Some(doc) => {
                    let mut context: HashMap<&str, String> = HashMap::new();
                    context.insert(
                        "output_path",
                        doc.output_path.to_string_lossy().into_owned(),
                    );
                    context.insert("title", doc.title.clone());
                    context.insert("summary", doc.summary.clone());
                    context.insert("text", wikilink.text);
                    context.insert("slug", wikilink.slug);
                    token_template::render(wikilink_template, &context)
                }
                None => {
                    let mut context: HashMap<&str, String> = HashMap::new();
                    context.insert("text", wikilink.text);
                    context.insert("slug", wikilink.slug);
                    token_template::render(nolink_template, &context)
                }
            }
        })
        .into_owned()
}

pub fn get_summary_wiki_html(text: &str) -> String {
    first_sentence(text)
        .pipe(|s| strip_wikilinks(&s))
        .pipe(|s| strip_html(&s))
}

pub fn get_summary_wiki_markdown(text: &str) -> String {
    first_sentence(&text)
        .pipe(|s| strip_wikilinks(&s))
        .pipe(|s| strip_markdown(&s))
}

impl Doc {
    pub fn find_wikilinks<'a>(&'a self) -> impl Iterator<Item = Wikilink> + 'a {
        let content: &'a str = &self.content;
        find_wikilinks(content)
    }

    pub fn get_summary_wiki_html(&self) -> String {
        first_sentence(&self.content)
            .pipe(|s| strip_wikilinks(&s))
            .pipe(|s| strip_html(&s))
    }

    pub fn get_summary_wiki_markdown(&self) -> String {
        first_sentence(&self.content)
            .pipe(|s| strip_wikilinks(&s))
            .pipe(|s| strip_markdown(&s))
    }

    /// Render wikilinks using a custom template.
    pub fn render_wikilinks_with_template(
        mut self,
        wikilink_template: &str,
        nolink_template: &str,
        slug_to_doc_index: &HashMap<String, Doc>,
    ) -> Self {
        self.content = render_wikilinks_with_template(
            &self.content,
            wikilink_template,
            nolink_template,
            slug_to_doc_index,
        );
        self
    }

    /// Render wikilinks using a default template
    pub fn render_wikilinks<'a>(self, slug_to_doc_index: &'a HashMap<String, Doc>) -> Self {
        self.render_wikilinks_with_template(
            r#"<a class="wikilink" href="{output_path}">{text}</a>"#,
            r#"<span class="nolink">{text}</span>"#,
            slug_to_doc_index,
        )
    }
}

pub trait WikilinkDocs: Docs {
    /// Create a hashmap of docs keyed by sluggified-title.
    /// This hashmap can be passed to `render_wikilinks` to render wikilinks
    /// in the content.
    fn index_by_title_slug(self) -> HashMap<String, Doc> {
        self.map(|doc| (doc.get_title_slug(), doc))
            .into_iter()
            .pipe(|iter| HashMap::from_iter(iter))
    }

    /// Render wikilinks using a custom template
    fn render_wikilinks_with_template(
        self,
        wikilink_template: &str,
        nolink_template: &str,
        slug_to_doc_index: &HashMap<String, Doc>,
    ) -> impl Docs {
        self.map(|doc| {
            doc.render_wikilinks_with_template(
                wikilink_template,
                nolink_template,
                slug_to_doc_index,
            )
        })
    }

    /// Render wikilinks using default template
    fn render_wikilinks(self, slug_to_doc_index: &HashMap<String, Doc>) -> impl Docs {
        self.map(|doc| doc.render_wikilinks(slug_to_doc_index))
    }

    /// Render wikilinks using a default template between the docs in this iterator.
    /// E.g. a wikilink will match if there is a doc in this iterator that has a title who's slug
    /// matches the sluggified wikilink.
    fn render_wikilinks_between(self) -> impl Docs {
        let docs: Vec<Doc> = self.collect();
        let index = docs.clone().into_iter().index_by_title_slug();
        let docs: Vec<Doc> = docs.into_iter().render_wikilinks(&index).collect();
        docs.into_iter()
    }
}

impl<I> WikilinkDocs for I where I: Docs {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_wikilink() {
        let wikilink = parse_wikilink("[[Page Name]]");
        assert_eq!(wikilink.text, "Page Name");
        assert_eq!(wikilink.slug, "page-name");

        let wikilink = parse_wikilink("[[slug|Display Text]]");
        assert_eq!(wikilink.text, "Display Text");
        assert_eq!(wikilink.slug, "slug");
    }

    #[test]
    fn test_find_wikilinks() {
        let text = "This is a [[wikilink]] and another [[link|Custom Text]].";
        let wikilinks: Vec<Wikilink> = find_wikilinks(text).collect();

        assert_eq!(wikilinks.len(), 2);
        assert_eq!(wikilinks[0].text, "wikilink");
        assert_eq!(wikilinks[0].slug, "wikilink");
        assert_eq!(wikilinks[1].text, "Custom Text");
        assert_eq!(wikilinks[1].slug, "link");
    }

    #[test]
    fn test_strip_wikilinks() {
        let text = "This is a [[wikilink]] and a [[link|Custom Text]].";
        let stripped = strip_wikilinks(text);
        assert_eq!(stripped, "This is a wikilink and a Custom Text.");
    }

    #[test]
    fn test_render_wikilinks_link() {
        let text = "This is a [[wikilink]] and a [[link|Custom Text]].";

        let mut slug_to_stub_index: HashMap<String, Doc> = HashMap::new();
        slug_to_stub_index.insert("wikilink".into(), Doc::draft("wikilink.html"));
        slug_to_stub_index.insert("link".into(), Doc::draft("custom-text.html"));

        let rendered = render_wikilinks_with_template(
            text,
            r#"<a class="wikilink" href="{slug}.html">{text}</a>"#,
            r#"<span class="nolink">{text}</span>"#,
            &slug_to_stub_index,
        );
        assert_eq!(
            rendered,
            r#"This is a <a class="wikilink" href="wikilink.html">wikilink</a> and a <a class="wikilink" href="link.html">Custom Text</a>."#
        );
    }

    #[test]
    fn test_render_wikilinks_nolink() {
        let text = "This is a [[wikilink]] and a [[link|Custom Text]].";
        let rendered = render_wikilinks_with_template(
            text,
            r#"<a class="wikilink" href=\"{slug}.html\">{text}</a>"#,
            r#"<span class="nolink">{text}</span>"#,
            &HashMap::new(),
        );
        assert_eq!(
            rendered,
            r#"This is a <span class="nolink">wikilink</span> and a <span class="nolink">Custom Text</span>."#
        );
    }

    #[test]
    fn test_find_wikilinks_empty() {
        let text = "This text has no wikilinks.";
        let wikilinks: Vec<Wikilink> = find_wikilinks(text).collect();
        assert_eq!(wikilinks.len(), 0);
    }
}
