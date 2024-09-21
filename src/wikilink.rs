use std::collections::HashMap;

use crate::doc::Doc;
use crate::docs::Docs;
use crate::html::strip_html;
use crate::markdown::strip_markdown;
use crate::text::{first_sentence, to_slug};
use crate::token_template;
use lazy_static::lazy_static;
use regex::{self, Regex};
use tap::Pipe;

lazy_static! {
    static ref WIKILINK: Regex =
        Regex::new(r"\[\[([^\]]+)\]\]").expect("Could not parse regular expression");
    static ref TRANSCLUDE: Regex =
        Regex::new(r"^\[\[([^\]]+)\]\]$").expect("Could not parse regular expression");
}

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

pub fn render_wikilinks_in_text(
    text: &str,
    template: &str,
    context: &HashMap<&str, String>,
) -> String {
    WIKILINK
        .replace_all(&text, |caps: &regex::Captures| {
            let wikilink = parse_wikilink(&caps[0]);
            let mut context = context.clone();
            context.insert("text", wikilink.text);
            context.insert("slug", wikilink.slug);
            token_template::render(template, &context)
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

    pub fn render_wikilinks(mut self, template: &str, context: &HashMap<&str, String>) -> Self {
        self.content = render_wikilinks_in_text(&self.content, template, context);
        self
    }
}

pub trait WikilinkDocs: Docs {
    fn render_wikilinks(self, template: &str, context: &HashMap<&str, String>) -> impl Docs {
        self.map(move |doc| doc.render_wikilinks(template, context))
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
    fn test_render_wikilinks() {
        let text = "This is a [[wikilink]] and a [[link|Custom Text]].";
        let rendered =
            render_wikilinks_in_text(text, "<a href=\"{slug}.html\">{text}</a>", &HashMap::new());
        assert_eq!(rendered, "This is a <a href=\"wikilink.html\">wikilink</a> and a <a href=\"link.html\">Custom Text</a>.");
    }

    #[test]
    fn test_find_wikilinks_empty() {
        let text = "This text has no wikilinks.";
        let wikilinks: Vec<Wikilink> = find_wikilinks(text).collect();
        assert_eq!(wikilinks.len(), 0);
    }
}
