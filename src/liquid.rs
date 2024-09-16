use crate::doc::Doc;
use liquid::model;
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
            "meta": doc.meta.clone(),
        })
    }
}

impl Doc {
    /// Render the liquid template with the given data object.
    pub fn render_liquid(self, data: model::Object) -> Result<Doc> {
        let parser = match liquid::ParserBuilder::with_stdlib().build() {
            Ok(parser) => parser,
            Err(err) => return Err(Error::new(std::io::ErrorKind::Other, err)),
        };

        let template = match parser.parse(&self.template) {
            Ok(template) => template,
            Err(err) => return Err(Error::new(std::io::ErrorKind::Other, err)),
        };

        let mut globals = model::Object::new();
        globals.insert("data".into(), model::Value::Object(data));
        globals.insert(
            "doc".into(),
            model::Value::Object(model::Object::from(&self)),
        );

        let content = match template.render(&globals) {
            Ok(rendered) => rendered,
            Err(err) => return Err(Error::new(std::io::ErrorKind::Other, err)),
        };

        Ok(self.set_content(content).set_extension_html())
    }
}
