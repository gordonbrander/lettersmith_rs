use crate::doc::Doc;
use crate::docs::{DocResults, Docs};
use crate::error::Error;
use crate::markdown::render_markdown;
use chrono::Utc;
use std::collections::HashMap;
pub use tera::{self, Context, Tera};

impl Doc {
    /// Render the content as a Tera template
    pub fn render_tera_in_content(
        self,
        renderer: &mut Tera,
        context: &tera::Context,
    ) -> Result<Self, Error> {
        let content = renderer.render_str(&self.content, context)?;
        Ok(self.set_content(content))
    }

    /// Render a str as a Tera template, assinging the result to content.
    pub fn render_tera_str(
        self,
        renderer: &mut Tera,
        template: &str,
        context: &tera::Context,
    ) -> Result<Self, Error> {
        let content = renderer.render_str(template, context)?;
        Ok(self.set_content(content))
    }

    /// Render the Tera template found at `template_path` and assign result to content
    pub fn render_tera_template(
        self,
        renderer: &Tera,
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

/// Filter to render text as Markdown
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

/// Decorate Tera instance with Lettersmith-specific configuration
pub fn decorate_renderer(renderer: Tera) -> Tera {
    let mut renderer = renderer;
    renderer.register_filter("markdown", filter_markdown);
    renderer
}

/// Create a Tera renderer with Lettersmith-specific configuration.
pub fn renderer(templates: &str) -> Result<Tera, Error> {
    let tera = Tera::new(templates)?;
    Ok(decorate_renderer(tera))
}

/// Decorate Tera context with default Lettersmith variables
pub fn decorate_context(context: Context) -> Context {
    let mut context = context;
    let now = Utc::now();
    context.insert("now", &now);
    context
}

/// Create a Tera context with default Lettersmith variables
pub fn context() -> Context {
    decorate_context(Context::new())
}

pub trait TeraDocs: Docs {
    fn render_tera_template(self, renderer: &Tera, context: &tera::Context) -> impl DocResults {
        self.map(|doc| doc.render_tera_template(renderer, context))
    }
}

impl<T> TeraDocs for T where T: Docs {}
