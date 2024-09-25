use std::sync::LazyLock;

use regex::Regex;

static HTML_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"<[^<]+?>").expect("Could not compile regular expression"));

/// A very simple tag stripper that removes anything between angle
/// brackets.
pub fn strip_html(html_str: &str) -> String {
    HTML_REGEX.replace_all(html_str, "").to_string()
}
