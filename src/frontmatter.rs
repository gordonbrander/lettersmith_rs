use crate::{doc::Doc, docs::Docs};
use regex::Regex;
use std::sync::LazyLock;

static FRONTMATTER_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    // `(?ms)` means "multiline" and "dot matches newline"
    Regex::new("(?ms)^---\n(.*)---\n?").expect("Could not compile frontmatter Regex")
});

pub fn extract_front_matter_and_content(text: &str) -> (String, String) {
    match FRONTMATTER_REGEX.find(text) {
        Some(match_result) => {
            let front_matter = FRONTMATTER_REGEX
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
    /// Parses YAML frontmatter from the document's content and assigns it to the `meta` field.
    ///
    /// Extracts the frontmatter (if present) from the document's content,
    /// attempts to parse it as YAML, and assigns the resulting data to the `meta` field.
    /// If parsing succeeds, it updates the `meta` field and removes the frontmatter from the content.
    /// If parsing fails, the `meta` field remains unchanged.
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

pub trait FrontmatterDocs: Docs {
    fn parse_frontmatter(self) -> impl Docs {
        self.map(|doc| doc.parse_frontmatter())
    }

    fn parse_and_uplift_frontmatter(self) -> impl Docs {
        self.map(|doc| doc.parse_and_uplift_frontmatter())
    }
}

impl<I> FrontmatterDocs for I where I: Docs {}

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
