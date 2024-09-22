use crate::{doc::Doc, docs::Docs, json::json};
use chrono::{DateTime, Utc};
use std::io::Result;
use std::path::Path;

const RSS_TEMPLATE: &str = r#"
<rss version="2.0">
<channel>
  <title>{{title | escape}}</title>
  <link>{{base_url}}</link>
  <description>{{description | escape}}</description>
  <generator>{{generator}}</generator>
  <lastBuildDate>
    {{last_build_date.strftime("%a, %d %b %Y %H:%M:%S %Z")}}
  </lastBuildDate>
  {% for doc in data.recent %}
  <item>
    <title>{{doc.title}}</title>
    <link>{{doc.output_path | to_url(base_url)}}</link>
    <guid>{{doc.output_path | to_url(base_url)}}</guid>
    <description>{{doc | get_summary | escape}}</description>
    <content:encoded><![CDATA[
      {{doc.content}}
    ]]></content:encoded>
    <pubDate>{{doc.created.strftime("%a, %d %b %Y %H:%M:%S %Z")}}</pubDate>
    {% if doc.meta.author %}
      <author>{{doc.meta.author | escape}}</author>
    {% elif author %}
      <author>{{author | escape}}</author>
    {% endif %}
  </item>
  {% endfor %}
</channel>
</rss>
"#;

pub trait RssDocs: Docs {
    fn rss(
        docs: impl Docs,
        base_url: &str,
        title: &str,
        description: &str,
        author: &str,
        output_path: &str,
        last_build_date: Option<DateTime<Utc>>,
    ) -> Result<Doc> {
        let last_build_date = last_build_date.unwrap_or_else(|| Utc::now());
        let recent: Vec<Doc> = docs.most_recent(24).collect();

        let data = json!({
            "base_url": base_url,
        });

        let rss_doc = Doc::new(
            output_path,
            output_path,
            None::<&Path>,
            None::<&Path>,
            last_build_date,
            last_build_date,
            title,
            "".to_owned(),
            json!({
                "description": description,
                "author": author,
                "last_build_date": last_build_date,
                "recent": recent
            }),
        );

        rss_doc.render_liquid_using_template_string(RSS_TEMPLATE, &data)
    }
}

impl<I> RssDocs for I where I: Docs {}
