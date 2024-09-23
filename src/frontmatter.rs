use crate::doc::Doc;
use regex::Regex;

pub fn extract_front_matter_and_content(text: &str) -> (String, String) {
    // `(?ms)` means "multiline" and "dot matches newline"
    let re = Regex::new("(?ms)^---\n(.*)---\n?").unwrap();

    match re.find(text) {
        Some(match_result) => {
            let front_matter = re
                .captures(text)
                .and_then(|cap| cap.get(1).map(|m| m.as_str().trim().to_string()))
                .unwrap_or_else(String::new);

            let content = text[match_result.end()..].trim().to_string();

            (front_matter, content)
        }
        None => (String::new(), text.to_string()),
    }
}

impl Doc {
    /// Parse YAML frontmatter and assign to `meta`
    pub fn parse_frontmatter(mut self) -> Self {
        let (frontmatter, content) = extract_front_matter_and_content(&self.content);
        if let Ok(meta) = serde_yml::from_str(&frontmatter) {
            self.meta = meta;
        }
        self.content = content;
        self
    }

    /// Parse YAML frontmatter and assign to `meta`, and set certain blessed
    /// meta fields to doc fields.
    pub fn parse_and_uplift_frontmatter(self) -> Self {
        self.parse_frontmatter().uplift_meta()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_front_matter_and_content() {
        let input = r#"---
title: Test Document
date: 2023-04-14
tags: [test, example]
---

This is the main content of the document.
It can span multiple lines."#;

        let (front_matter, content) = extract_front_matter_and_content(input);

        assert_eq!(
            front_matter,
            "title: Test Document\ndate: 2023-04-14\ntags: [test, example]"
        );
        assert_eq!(
            content,
            "This is the main content of the document.\nIt can span multiple lines."
        );
    }

    #[test]
    fn test_extract_front_matter_and_content_no_frontmatter() {
        let input = "This is a document without front matter.";

        let (front_matter, content) = extract_front_matter_and_content(input);

        assert_eq!(front_matter, "");
        assert_eq!(content, "This is a document without front matter.");
    }

    #[test]
    fn test_extract_front_matter_and_content_empty_frontmatter() {
        let input = "---\n\n---\nContent after empty front matter.";

        let (front_matter, content) = extract_front_matter_and_content(input);

        assert_eq!(front_matter, "");
        assert_eq!(content, "Content after empty front matter.");
    }
}
