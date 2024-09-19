use lazy_static::lazy_static;
use regex::{self, Regex};
use tap::Pipe;

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

pub fn render_wikilinks<F>(text: &str, render_wikilink: F) -> String
where
    F: Fn(&str, &str, &str) -> String,
{
    let text = TRANSCLUDE.replace_all(text, |caps: &regex::Captures| {
        let wikilink = parse_wikilink(&caps[0]);
        render_wikilink(&wikilink.slug, &wikilink.text, "transclude")
    });

    WIKILINK
        .replace_all(&text, |caps: &regex::Captures| {
            let wikilink = parse_wikilink(&caps[0]);
            render_wikilink(&wikilink.slug, &wikilink.text, "inline")
        })
        .into_owned()
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
        let rendered = render_wikilinks(text, |slug, text, _| {
            format!("<a href=\"{}.html\">{}</a>", slug, text)
        });
        assert_eq!(rendered, "This is a <a href=\"wikilink.html\">wikilink</a> and a <a href=\"link.html\">Custom Text</a>.");
    }

    #[test]
    fn test_transclude() {
        let text = "Text\n[[transclude]]\nMore text.";
        let rendered = render_wikilinks(text, |slug, text, kind| {
            if kind == "transclude" {
                format!("<div class=\"transclude\">{}</div>", slug)
            } else {
                format!("<a href=\"{}.html\">{}</a>", slug, text)
            }
        });
        assert_eq!(
            rendered,
            "Text\n<div class=\"transclude\">transclude</div>\nMore text."
        );
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

    #[test]
    fn test_strip_wikilinks_with_transclude() {
        let text = "[[Transclude]]\nThis is a [[wikilink]].";
        let stripped = strip_wikilinks(text);
        assert_eq!(stripped, "\nThis is a wikilink.");
    }
}
