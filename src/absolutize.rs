use crate::{doc::Doc, docs::Docs};
use regex::Regex;

/// Qualify a URL with a base URL if it's relative.
pub fn qualify_url(url: &str, base_url: &str) -> String {
    if url.starts_with('/') {
        format!("{}{}", base_url.trim_end_matches('/'), url)
    } else {
        url.to_string()
    }
}

/// Replace relative URLs in content with absolute URLs.
pub fn absolutize_urls_in_html(html: &str, base_url: &str) -> String {
    let re = Regex::new(r#"(src|href)=["'](.*?)["']"#)
        .expect("Failed to compile regex for absolutizing URLs");
    re.replace_all(html, |caps: &regex::Captures| {
        let attr = &caps[1];
        let value = &caps[2];
        let url = qualify_url(value, base_url);
        format!(r#"{}="{}""#, attr, url)
    })
    .to_string()
}

impl Doc {
    /// Absolutize URLs in the content of this document.
    pub fn absolutize_urls(self, base_url: &str) -> Self {
        let content = absolutize_urls_in_html(&self.content, base_url);
        self.set_content(&content)
    }
}

pub trait AbsolutizableDocs: Docs {
    /// Absolutize URLs in the content of a sequence of documents.
    fn absolutize_urls(self, base_url: &str) -> impl Docs {
        self.map(move |doc| doc.absolutize_urls(base_url))
    }
}

impl<I> AbsolutizableDocs for I where I: Docs {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::json::json;
    use chrono::Utc;
    use std::path::PathBuf;

    #[test]
    fn test_qualify_url() {
        assert_eq!(
            qualify_url("/path", "https://example.com"),
            "https://example.com/path"
        );
        assert_eq!(
            qualify_url("https://other.com", "https://example.com"),
            "https://other.com"
        );
        assert_eq!(
            qualify_url("/path", "https://example.com/"),
            "https://example.com/path"
        );
    }

    #[test]
    fn test_absolutize_urls_in_html() {
        let html = r#"<a href="/relative">Link</a><img src="https://absolute.com/image.jpg">"#;
        let base_url = "https://example.com";
        let expected = r#"<a href="https://example.com/relative">Link</a><img src="https://absolute.com/image.jpg">"#;
        assert_eq!(absolutize_urls_in_html(html, base_url), expected);
    }

    #[test]
    fn test_doc_absolutize_urls() {
        let doc = Doc {
            id_path: PathBuf::from("test.md"),
            input_path: None,
            output_path: PathBuf::from("test.html"),
            template_path: None::<PathBuf>,
            created: Utc::now(),
            modified: Utc::now(),
            title: "Test".to_string(),
            content: "<a href='/relative'>Link</a>".to_string(),
            meta: json!({}),
        };
        let base_url = "https://example.com";
        let expected_content = r#"<a href="https://example.com/relative">Link</a>"#;
        let absolutized = doc.absolutize_urls(base_url);
        assert_eq!(absolutized.content, expected_content);
    }
}
