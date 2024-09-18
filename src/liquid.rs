use crate::doc::Doc;
use crate::json;
pub use liquid::model;
use std::io::{Error, Result};

/// Implement From for Doc -> liquid::Object.
impl From<&Doc> for model::Object {
    fn from(doc: &Doc) -> Self {
        liquid::object!({
            "id_path": doc.id_path,
            "created": doc.created,
            "modified": doc.modified,
            "title": doc.title,
            "content": doc.content,
            "meta": doc.meta,
        })
    }
}

pub fn json_to_liquid(value: &json::Value) -> liquid::model::Value {
    match value {
        json::Value::Null => model::Value::Nil,
        json::Value::Bool(b) => model::Value::scalar(*b),
        json::Value::Number(n) => {
            if n.is_i64() {
                model::Value::scalar(n.as_i64().unwrap())
            } else {
                model::Value::scalar(n.as_f64().unwrap())
            }
        }
        json::Value::String(s) => model::Value::scalar(s.clone()),
        json::Value::Array(a) => model::Value::Array(a.iter().map(|v| json_to_liquid(v)).collect()),
        json::Value::Object(o) => {
            model::Value::Object(o.iter().fold(model::Object::new(), |mut acc, (k, v)| {
                acc.insert(k.into(), json_to_liquid(v));
                acc
            }))
        }
    }
}

impl Doc {
    /// Render the liquid template with the given data object.
    pub fn render_liquid(self, data: &json::Value) -> Result<Doc> {
        // Construct the parser
        let parser = match liquid::ParserBuilder::with_stdlib().build() {
            Ok(parser) => parser,
            Err(err) => return Err(Error::new(std::io::ErrorKind::Other, err)),
        };

        // Parse the template
        let template = match parser.parse(&self.template) {
            Ok(template) => template,
            Err(err) => return Err(Error::new(std::io::ErrorKind::Other, err)),
        };

        // Set up the template data
        let mut globals = model::Object::new();
        globals.insert("data".into(), json_to_liquid(&data));
        globals.insert(
            "doc".into(),
            model::Value::Object(model::Object::from(&self)),
        );

        // Render the template
        let content = match template.render(&globals) {
            Ok(rendered) => rendered,
            Err(err) => return Err(Error::new(std::io::ErrorKind::Other, err)),
        };

        // Set content and return
        Ok(self.set_content(content).set_extension_html())
    }
}

pub fn render_liquid(
    docs: impl Iterator<Item = Doc>,
    data: json::Value,
) -> impl Iterator<Item = Result<Doc>> {
    docs.map(move |doc| doc.render_liquid(&data))
}
