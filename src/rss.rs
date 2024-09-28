use crate::{doc::Doc, docs::Docs, error::Error, json::json};
use chrono::{DateTime, Utc};
use std::path::Path;

const RSS_TEMPLATE: &str = r#"
<rss version="2.0">
<channel>
  <title>{{ doc.title | escape }}</title>
  <link>{{ doc.meta.site_url }}</link>
  <description>{{ doc.meta.description | escape }}</description>
  <generator>{{ doc.meta.generator }}</generator>
  <lastBuildDate>
    {{ doc.modified | date: "%a, %d %b %Y %H:%M:%S %Z" }}
  </lastBuildDate>
  {% for rdoc in doc.meta.recent %}
  <item>
    <title>{{ rdoc.title }}</title>
    <link>{{ rdoc.output_path | prepend: doc.meta.site_url }}</link>
    <guid>{{ rdoc.output_path | prepend: doc.meta.site_url }}</guid>
    <description>{{ rdoc.content | escape }}</description>
    <content:encoded><![CDATA[
      {{ rdoc.content }}
    ]]></content:encoded>
    <pubDate>{{ doc.created | date: "%a, %d %b %Y %H:%M:%S %Z" }}</pubDate>
    {% if rdoc.meta.author %}
      <author>{{ rdoc.meta.author | escape }}</author>
    {% elsif author %}
      <author>{{ doc.meta.author | escape }}</author>
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
        output_path: &str,
        last_build_date: Option<DateTime<Utc>>,
    ) -> Result<Doc, Error> {
        let last_build_date = last_build_date.unwrap_or_else(|| Utc::now());
        let recent: Vec<Doc> = self.most_recent(24).collect();

        let data = json!({});

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
                "site_url": site_url,
                "author": author,
                "description": description,
                "recent": recent,
                "generator": "Lettersmith"
            }),
        );

        rss_doc.render_liquid_using_template_string(RSS_TEMPLATE, &data)
    }
}

impl<I> RssDocs for I where I: Docs {}
