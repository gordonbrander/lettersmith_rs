use regex::Regex;
use std::sync::LazyLock;
use tap::Pipe;

static FIRST_SENTENCE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[^.]+").expect("Could not compile Regex"));

static NON_SLUG_CHARS: LazyLock<Regex> = LazyLock::new(|| {
    let pattern_str = format!("[{}]", regex::escape("[](){}<>:,;?!^&%$#@'\"|*~"));
    Regex::new(&pattern_str).expect("Could not parse regular expression")
});

/// Extract the first sentence from a string.
pub fn first_sentence(plain_text: &str) -> String {
    match FIRST_SENTENCE.find(plain_text) {
        Some(mat) => mat.as_str().to_string(),
        None => String::new(),
    }
}

/// Truncate a string to a maximum length, adding a suffix if truncated.
pub fn truncate(text: &str, max_chars: usize, suffix: &str) -> String {
    let stripped = text.trim();
    if stripped.len() <= max_chars {
        return stripped.to_string();
    }
    let substring: String = text.chars().take(max_chars - 1).collect();
    let words: Vec<&str> = substring.split_whitespace().collect();
    let truncated = words[..words.len() - 1].join(" ");
    truncated + suffix
}

/// Truncate a string to 280 characters, adding an ellipsis if truncated.
pub fn truncate_280(text: &str) -> String {
    truncate(text, 280, "…")
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_first_sentence() {
        assert_eq!(
            first_sentence("Hello world. This is a test."),
            "Hello world"
        );
        assert_eq!(first_sentence("Single sentence."), "Single sentence");
        assert_eq!(first_sentence(""), "");
    }

    #[test]
    fn test_truncate() {
        assert_eq!(truncate("Short text", 20, "..."), "Short text");
        assert_eq!(truncate("This is a longer text", 10, "..."), "This is...");
        assert_eq!(
            truncate("Another long text for testing", 15, "..."),
            "Another long..."
        );
    }

    #[test]
    fn test_truncate_280() {
        assert_eq!(
            truncate_280(
                "
                I pray thee, mark me.
                I, thus neglecting worldly ends, all dedicated
                To closeness and the bettering of my mind
                With that which, but by being so retired,
                O'er-prized all popular rate, in my false brother
                Awaked an evil nature; and my trust,
                Like a good parent, did beget of him
                A falsehood in its contrary as great
                As my trust was; which had indeed no limit,
                A confidence sans bound. He being thus lorded,
                Not only with what my revenue yielded,
                But what my power might else exact, like one
                Who having into truth, by telling of it,
                Made such a sinner of his memory,
                To credit his own lie, he did believe
                He was indeed the duke; out o' the substitution
                And executing the outward face of royalty,
                With all prerogative: hence his ambition growing--
                Dost thou hear?
                "
            ),
            "I pray thee, mark me. I, thus neglecting worldly ends, all dedicated To closeness and the bettering of my mind With that which, but by being so retired, O'er-prized all popular rate, in my false…"
        );
    }

    #[test]
    fn test_remove_non_slug_chars() {
        assert_eq!(remove_non_slug_chars("Hello, World!"), "Hello World");
        assert_eq!(remove_non_slug_chars("Test@#$%^&*()"), "Test");
        assert_eq!(remove_non_slug_chars("[Bracketed]"), "Bracketed");
    }

    #[test]
    fn test_to_slug() {
        assert_eq!(to_slug("Hello World!"), "hello-world");
        assert_eq!(to_slug("Test 123"), "test-123");
        assert_eq!(to_slug("  Spaced  "), "spaced");
        assert_eq!(to_slug("Symbols@#$%"), "symbols");
    }
}
