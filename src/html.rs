use regex::Regex;

/// A very simple tag stripper that removes anything between angle
/// brackets.
pub fn strip_tags(html_str: &str) -> String {
    let re = Regex::new(r"<[^<]+?>").expect("Could not compile regular expression");
    re.replace_all(html_str, "").to_string()
}
