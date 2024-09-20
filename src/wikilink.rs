use std::collections::HashMap;

use crate::markdown::strip_markdown;
use crate::{html::strip_tags, text::first_sentence, token_template};
use lazy_static::lazy_static;
use regex::{self, Regex};
use tap::Pipe;

use crate::doc::Doc;

fn compile_non_slug_chars_regex() -> Regex {
    let pattern_str = format!("[{}]", regex::escape("[](){}<>:,;?!^&%$#@'\"|*~"));
    Regex::new(&pattern_str).expect("Could not parse regular expression")
}

fn compile_wikilink_regex() -> Regex {
    Regex::new(r"\[\[([^\]]+)\]\]").expect("Could not parse regular expression")
}

fn compile_transclude_regex() -> Regex {
    Regex::new(r"^\[\[([^\]]+)\]\]$").expect("Could not parse regular expression")
}

lazy_static! {
    static ref NON_SLUG_CHARS: Regex = compile_non_slug_chars_regex();
    static ref WIKILINK: Regex = compile_wikilink_regex();
    static ref TRANSCLUDE: Regex = compile_transclude_regex();
}

pub fn remove_non_slug_chars(s: &str) -> String {
    NON_SLUG_CHARS.replace_all(s, "").into_owned()
}

pub fn to_slug(s: &str) -> String {
    s.trim()
        .to_lowercase()
        .replace(' ', "-")
        .pipe(|s| remove_non_slug_chars(&s))
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

pub fn find_wikilinks(s: &str) -> impl Iterator<Item = Wikilink> + '_ {
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

pub fn render_wikilinks_in_text(text: &str, template: &str) -> String {
    WIKILINK
        .replace_all(&text, |caps: &regex::Captures| {
            let wikilink = parse_wikilink(&caps[0]);
            let parts = HashMap::from(wikilink);
            token_template::render(template, &parts)
        })
        .into_owned()
}

pub fn get_summary_wiki_html(text: &str) -> String {
    first_sentence(text)
        .pipe(|s| strip_wikilinks(&s))
        .pipe(|s| strip_tags(&s))
}

pub fn get_summary_wiki_markdown(text: &str) -> String {
    first_sentence(&text)
        .pipe(|s| strip_wikilinks(&s))
        .pipe(|s| strip_markdown(&s))
}

impl Doc {
    pub fn render_wikilinks(mut self, template: &str) -> Self {
        self.content = render_wikilinks_in_text(&self.content, template);
        self
    }
}

pub fn render_wikilinks<'a>(
    docs: impl Iterator<Item = Doc> + 'a,
    template: &'a str,
) -> impl Iterator<Item = Doc> + 'a {
    docs.map(move |doc| doc.render_wikilinks(template))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_slug() {
        assert_eq!(to_slug("Hello World!"), "hello-world");
        assert_eq!(to_slug("Test 123"), "test-123");
        assert_eq!(to_slug("  Spaced  "), "spaced");
        assert_eq!(to_slug("Symbols@#$%"), "symbols");
    }

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
        let rendered = render_wikilinks_in_text(text, "<a href=\":slug.html\">:text</a>");
        assert_eq!(rendered, "This is a <a href=\"wikilink.html\">wikilink</a> and a <a href=\"link.html\">Custom Text</a>.");
    }

    #[test]
    fn test_remove_non_slug_chars() {
        assert_eq!(remove_non_slug_chars("Hello, World!"), "Hello World");
        assert_eq!(remove_non_slug_chars("Test@#$%^&*()"), "Test");
        assert_eq!(remove_non_slug_chars("[Bracketed]"), "Bracketed");
    }

    #[test]
    fn test_find_wikilinks_empty() {
        let text = "This text has no wikilinks.";
        let wikilinks: Vec<Wikilink> = find_wikilinks(text).collect();
        assert_eq!(wikilinks.len(), 0);
    }
}
