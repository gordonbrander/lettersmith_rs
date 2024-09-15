use regex::Regex;

const _FIRST_SENTENCE: &str = r"^[^.]+";

/// Extract the first sentence from a string.
pub fn first_sentence(plain_text: &str) -> String {
    let re = Regex::new(_FIRST_SENTENCE).unwrap();
    match re.find(plain_text) {
        Some(mat) => mat.as_str().to_string(),
        None => String::new(),
    }
}

/// Truncate a string to a maximum length, adding a suffix if truncated.
pub fn truncate(text: &str, max_len: usize, suffix: &str) -> String {
    let stripped = text.trim();
    if stripped.len() <= max_len {
        return stripped.to_string();
    }
    let substr = &stripped[..max_len.min(stripped.len())];
    let words: Vec<&str> = substr.split_whitespace().collect();
    let truncated = words[..words.len() - 1].join(" ");
    truncated + suffix
}
