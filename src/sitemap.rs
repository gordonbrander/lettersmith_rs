use crate::doc::Doc;
use crate::docs::Docs;
use crate::error::Error;
use crate::json::json;
use crate::stub::{Stub, StubDocs};
use chrono::Utc;
use std::path::PathBuf;

const SITEMAP_TEMPLATE: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
  {% for stub in data.sitemap_items %}
  <url>
    <loc>{{ stub.output_path | to_url(base_url) }}</loc>
    <lastmod>{{ stub.modified.isoformat() }}</lastmod>
  </url>
  {% endfor %}
</urlset>"#;

pub trait SitemapDocs: Docs {
    /// Generate a sitemap doc given an iterator of docs
    fn sitemap(self, base_url: &str) -> Result<Doc, Error> {
        // The sitemap spec limits each sitemap to 50k entries.
        // https://www.sitemaps.org/protocol.html
        let stubs_50k: Vec<Stub> = self.take(50000).stubs().collect();
        let output_path = "sitemap.xml".to_string();
        let now = Utc::now();

        let sitemap = Doc {
            id_path: PathBuf::from(&output_path),
            output_path: PathBuf::from(&output_path),
            input_path: None,
            template_path: None::<PathBuf>,
            created: now,
            modified: now,
            title: "".to_owned(),
            content: "".to_owned(),
            meta: json!({}),
        };

        let data = json!({
            "base_url": base_url,
            "sitemap_items": stubs_50k
        });

        sitemap.render_liquid_using_template_string(SITEMAP_TEMPLATE, &data)
    }
}

impl<I> SitemapDocs for I where I: Docs {}
