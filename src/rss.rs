use crate::tera::{Context, Tera};
use crate::{doc::Doc, docs::Docs, error::Error, json::json};
use chrono::{DateTime, Utc};
use std::path::Path;

const RSS_TEMPLATE: &str = r#"
<rss version="2.0">
<channel>
  <title>{{ doc.title | escape }}</title>
  <link>{{ site_url }}</link>
  <description>{{ description | escape }}</description>
  <generator>{{ generator }}</generator>
  <lastBuildDate>
    {{ doc.modified | date(format="%a, %d %b %Y %H:%M:%S %Z") }}
  </lastBuildDate>
  {% for rdoc in recent %}
  <item>
    <title>{{ rdoc.title }}</title>
    <link>{{site_url}}/{{ rdoc.output_path }}</link>
    <guid>{{site_url}}/{{ rdoc.output_path }}</guid>
    <description>{{ rdoc.content | escape }}</description>
    <content:encoded><![CDATA[
      {{ rdoc.content }}
    ]]></content:encoded>
    <pubDate>{{ doc.created | date(format="%a, %d %b %Y %H:%M:%S %Z") }}</pubDate>
    {% if rdoc.meta.author %}
      <author>{{ rdoc.meta.author | escape }}</author>
    {% elsif author %}
      <author>{{ author | escape }}</author>
    {% endif %}
  </item>
  {% endfor %}
</channel>
</rss>
"#;

pub trait RssDocs: Docs {
    fn rss(
        self,
        site_url: &str,
        title: &str,
        description: &str,
        author: &str,
        output_path: &Path,
        last_build_date: Option<DateTime<Utc>>,
    ) -> Result<Doc, Error> {
        let last_build_date = last_build_date.unwrap_or_else(|| Utc::now());
        let recent: Vec<Doc> = self.most_recent(24).collect();

        let mut renderer = Tera::default();
        let mut context = Context::new();
        context.insert("site_url", site_url);
        context.insert("recent", &recent);
        context.insert("description", description);
        context.insert("author", author);
        context.insert("generator", "Lettersmith");

        let rss_doc = Doc::new(
            output_path.into(),
            output_path.into(),
            None,
            None,
            last_build_date,
            last_build_date,
            title.to_string(),
            "".to_string(),
            "".to_string(),
            json!({}),
        );

        rss_doc.render_tera_str(&mut renderer, RSS_TEMPLATE, &context)
    }
}

impl<I> RssDocs for I where I: Docs {}
