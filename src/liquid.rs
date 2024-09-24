use crate::doc::Doc;
use crate::docs::{DocResults, Docs};
use crate::error::{Error, ErrorKind};
use crate::json;
pub use liquid::{model, object};

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

/// Render liquid template using pre-defined features
pub fn render(template: &str, context: &model::Object) -> Result<String, Error> {
    // Construct the parser
    let parser = match liquid::ParserBuilder::with_stdlib().build() {
        Ok(parser) => parser,
        Err(err) => {
            return Err(Error::new(
                ErrorKind::Liquid(err),
                "Unable to build Liquid parser",
            ))
        }
    };

    // Parse the template
    let parsed_template = match parser.parse(template) {
        Ok(template) => template,
        Err(err) => {
            return Err(Error::new(
                ErrorKind::Liquid(err),
                format!("Unable to parse Liquid template"),
            ))
        }
    };

    match parsed_template.render(context) {
        Ok(content) => Ok(content),
        Err(err) => Err(Error::new(
            ErrorKind::Liquid(err),
            format!("Unable to render Liquid template"),
        )),
    }
}

impl Doc {
    /// Render doc using a given template string,
    /// Ignores the template at doc template path and uses string instead.
    ///
    /// The template is provided with `doc` and the additional `data` object
    /// you pass in.
    pub fn render_liquid_using_template_string(
        self,
        template: &str,
        config: &json::Value,
    ) -> Result<Doc, Error> {
        // Set up the template data
        let context = model::object!({
            "config": json_to_liquid(&config),
            "doc": &self
        });
        let content = render(template, &context)?;
        // Set content and return
        Ok(self.set_content(content).set_extension_html())
    }

    /// Render the doc using the template at `template_path` and Liquid
    /// template system.
    ///
    /// The template is provided with `doc` and the additional `data` object
    /// you pass in.
    pub fn render_liquid(self, config: &json::Value) -> Result<Doc, Error> {
        let Some(template_path) = &self.template_path else {
            return Ok(self);
        };
        // Read template and output a helpful error if we can't find it.
        let template_doc = Doc::read(template_path).map_err(|err| {
            Error::new(
                err.kind,
                format!(
                    "render_liquid: could not find template at {}",
                    template_path.to_string_lossy()
                ),
            )
        })?;
        self.render_liquid_using_template_string(&template_doc.content, config)
    }
}

pub trait LiquidDocs: Docs {
    fn render_liquid(self, data: json::Value) -> impl DocResults {
        self.map(move |doc| doc.render_liquid(&data))
    }
}

impl<I> LiquidDocs for I where I: Docs {}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};
    use serde_json::json;
    use std::path::PathBuf;

    #[test]
    fn test_json_to_liquid() {
        let json_value = json!({
            "null": null,
            "bool": true,
            "integer": 42,
            "float": 3.14,
            "string": "hello",
            "array": [1, 2, 3],
            "object": {"key": "value"}
        });

        let liquid_value = json_to_liquid(&json_value);

        if let model::Value::Object(obj) = liquid_value {
            assert!(matches!(obj.get("null"), Some(model::Value::Nil)));
            assert_eq!(obj.get("bool"), Some(&model::Value::scalar(true)));
            assert_eq!(obj.get("integer"), Some(&model::Value::scalar(42i64)));
            assert_eq!(obj.get("float"), Some(&model::Value::scalar(3.14f64)));
            assert_eq!(obj.get("string"), Some(&model::Value::scalar("hello")));

            if let Some(model::Value::Array(arr)) = obj.get("array") {
                assert_eq!(arr.len(), 3);
                assert_eq!(arr[0], model::Value::scalar(1i64));
                assert_eq!(arr[1], model::Value::scalar(2i64));
                assert_eq!(arr[2], model::Value::scalar(3i64));
            } else {
                panic!("Expected array");
            }

            if let Some(model::Value::Object(nested_obj)) = obj.get("object") {
                assert_eq!(nested_obj.get("key"), Some(&model::Value::scalar("value")));
            } else {
                panic!("Expected object");
            }
        } else {
            panic!("Expected object");
        }
    }

    #[test]
    fn test_doc_to_liquid_object() {
        let doc = Doc {
            id_path: "test/doc".into(),
            output_path: "test/doc".into(),
            input_path: None,
            created: Utc.with_ymd_and_hms(2020, 11, 10, 0, 1, 32).unwrap(),
            modified: Utc.with_ymd_and_hms(2020, 11, 10, 0, 1, 32).unwrap(),
            title: "Test Document".to_string(),
            content: "Test content".to_string(),
            template_path: None,
            meta: json!({"key": "value"}),
        };

        let liquid_obj: model::Object = (&doc).into();

        assert_eq!(
            liquid_obj.get("id_path"),
            Some(&model::Value::scalar("test/doc"))
        );
        assert_eq!(
            liquid_obj.get("created"),
            Some(&model::Value::scalar("2020-11-10T00:01:32Z"))
        );
        assert_eq!(
            liquid_obj.get("modified"),
            Some(&model::Value::scalar("2020-11-10T00:01:32Z"))
        );
        assert_eq!(
            liquid_obj.get("title"),
            Some(&model::Value::scalar("Test Document"))
        );
        assert_eq!(
            liquid_obj.get("content"),
            Some(&model::Value::scalar("Test content"))
        );

        if let Some(model::Value::Object(meta_obj)) = liquid_obj.get("meta") {
            assert_eq!(meta_obj.get("key"), Some(&model::Value::scalar("value")));
        } else {
            panic!("Expected meta to be an object");
        }
    }

    #[test]
    fn test_doc_render_liquid() {
        let doc = Doc {
            id_path: "test/doc".into(),
            output_path: "test/doc".into(),
            input_path: None,
            template_path: None::<PathBuf>,
            created: Utc.with_ymd_and_hms(2020, 11, 10, 0, 1, 32).unwrap(),
            modified: Utc.with_ymd_and_hms(2020, 11, 10, 0, 1, 32).unwrap(),
            title: "Test Document".to_string(),
            content: "Original content".to_string(),
            meta: json!({"key": "value"}),
        };

        let config = json!({"message": "Hello, World!"});

        let rendered_doc = doc
            .render_liquid_using_template_string("{{ config.message }} - {{ doc.title }}", &config)
            .unwrap();

        assert_eq!(rendered_doc.content, "Hello, World! - Test Document");
    }
}
