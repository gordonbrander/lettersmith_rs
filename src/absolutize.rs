use crate::doc::Doc;
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
        self.set_content(content)
    }
}

/// Absolutize URLs in the content of a sequence of documents.
pub fn absolutize_urls<'a>(
    docs: impl Iterator<Item = Doc> + 'a,
    base_url: &'a str,
) -> impl Iterator<Item = Doc> + 'a {
    docs.map(move |doc| doc.absolutize_urls(base_url))
}
