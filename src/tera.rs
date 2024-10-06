use crate::config::Config;
use crate::doc::Doc;
use crate::docs::{DocResults, Docs};
use crate::error::Error;
use crate::markdown::render_markdown;
use std::collections::HashMap;
pub use tera;

impl Doc {
    /// Render the content as a Tera template
    pub fn render_tera_in_content(
        self,
        renderer: &mut tera::Tera,
        context: &tera::Context,
    ) -> Result<Self, Error> {
        let content = renderer.render_str(&self.content, context)?;
        Ok(self.set_content(content))
    }

    /// Render a str as a Tera template, assinging the result to content.
    pub fn render_tera_str(
        self,
        renderer: &mut tera::Tera,
        template: &str,
        context: &tera::Context,
    ) -> Result<Self, Error> {
        let content = renderer.render_str(template, context)?;
        Ok(self.set_content(content))
    }

    /// Render the Tera template found at `template_path` and assign result to content
    pub fn render_tera_template(
        self,
        renderer: &tera::Tera,
        context: &tera::Context,
    ) -> Result<Self, Error> {
        let Some(template_path) = &self.template_path else {
            return Ok(self);
        };
        let template_name = template_path.to_string_lossy().into_owned();
        let mut context_ext = context.clone();
        context_ext.insert("doc", &self);
        let content = renderer.render(&template_name, &context_ext)?;
        Ok(self.set_content(content))
    }
}

/// Liquid filter to render text as Markdown
fn filter_markdown(
    value: &tera::Value,
    _: &HashMap<String, tera::Value>,
) -> tera::Result<tera::Value> {
    let Some(markdown_str) = value.as_str() else {
        return Err(tera::Error::msg(
            "Markdown filter can only be called on strings",
        ));
    };
    let rendered = render_markdown(markdown_str);
    Ok(tera::Value::String(rendered))
}

pub fn create_renderer(config: &Config) -> Result<tera::Tera, Error> {
    let mut tera = tera::Tera::new(&config.templates)?;
    tera.register_filter("markdown", filter_markdown);
    Ok(tera)
}

pub trait TeraDocs: Docs {
    fn render_tera_template(
        self,
        renderer: &tera::Tera,
        context: &tera::Context,
    ) -> impl DocResults {
        self.map(|doc| doc.render_tera_template(renderer, context))
    }

    /// Creates a shared Tera instance using the settings in configs
    /// and renders docs with it.
    fn render_tera_template_using_config(self, config: &Config) -> impl DocResults {
        let renderer = create_renderer(config).unwrap();
        let mut context = tera::Context::new();
        context.insert("site", config);
        self.map(move |doc| doc.render_tera_template(&renderer, &context))
    }
}
