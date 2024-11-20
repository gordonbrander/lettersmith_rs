use crate::doc::Doc;
use crate::docs::Docs;
use crate::error::Error;
use crate::json::json;
use crate::tera::{Context, Tera};
use chrono::Utc;
use std::path::PathBuf;

const SITEMAP_TEMPLATE: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
  {% for doc in data.sitemap_items %}
  <url>
    <loc>{{ doc.output_path | to_url(base_url) }}</loc>
    <lastmod>{{ doc.modified | date }}</lastmod>
  </url>
  {% endfor %}
</urlset>"#;

pub trait SitemapDocs: Docs {
    /// Generate a sitemap doc given an iterator of docs
    fn sitemap(self, base_url: &str) -> Result<Doc, Error> {
        // The sitemap spec limits each sitemap to 50k entries.
        // https://www.sitemaps.org/protocol.html
        let stubs_50k: Vec<Doc> = self.take(50000).collect();
        let output_path = "sitemap.xml".to_string();
        let now = Utc::now();

        let sitemap = Doc {
            id_path: PathBuf::from(&output_path),
            output_path: PathBuf::from(&output_path),
            input_path: None,
            template_path: None,
            created: now,
            modified: now,
            title: "".to_string(),
            summary: "".to_string(),
            content: "".to_string(),
            meta: json!({}),
        };

        let mut renderer = Tera::default();
        let mut context = Context::new();
        context.insert("base_url", base_url);
        context.insert("sitemap_items", &stubs_50k);
        sitemap.render_tera_str(&mut renderer, SITEMAP_TEMPLATE, &context)
    }
}

impl<I> SitemapDocs for I where I: Docs {}
