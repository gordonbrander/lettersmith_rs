use crate::doc::Doc;
use liquid::model::Value as LiquidValue;
use std::io::{Error, Result};

impl Doc {
    pub fn to_liquid(&self) -> liquid::Object {
        let obj = liquid::object!({
            "id_path": self.id_path,
            "created": self.created,
            "modified": self.modified,
            "title": self.title,
            "content": self.content,
            "meta": self.meta.clone(),
        });
        obj
    }

    pub fn render_liquid(&self) -> Result<String> {
        let parser = match liquid::ParserBuilder::with_stdlib().build() {
            Ok(parser) => parser,
            Err(err) => return Err(Error::new(std::io::ErrorKind::Other, err)),
        };

        let template = match parser.parse(&self.template) {
            Ok(template) => template,
            Err(err) => return Err(Error::new(std::io::ErrorKind::Other, err)),
        };

        let mut globals = liquid::Object::new();
        globals.insert("doc".into(), LiquidValue::Object(self.to_liquid()));

        match template.render(&globals) {
            Ok(rendered) => Ok(rendered),
            Err(err) => Err(Error::new(std::io::ErrorKind::Other, err)),
        }
    }
}
