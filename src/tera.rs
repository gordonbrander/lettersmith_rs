use crate::doc::Doc;
use crate::docs::{DocResults, Docs};
use crate::error::Error;
use crate::json::get_deep;
use crate::markdown::render_markdown;
use crate::tags::get_union_for_index_keys;
use crate::text;
use chrono::Utc;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
pub use tera::{self, try_get_value, Context, Tera};

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

/// Tera filter to render text as Markdown
/// You can use this filter in block position to render a block of
/// text as Markdown within a Tera template.
///
/// Example:
/// ```tera
/// {% filter markdown %}
/// # Hello Markdown
/// This is _Markdown_.
/// {% endfilter %}
/// ```
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

/// Retrieves related stubs from an index.
///
/// This function is used as a Tera filter to find related stubs based on provided tags.
///
/// # Arguments
///
/// * `value` - The filter value: An index object that maps tag terms to arrays of stub values.
/// * `args` - A HashMap containing the `tags` argument, which is an array of tags to find related stubs for.
///
/// # Returns
///
/// Returns a `tera::Result<tera::Value>` containing the related stubs.
///
/// # Example
///
/// ```tera
/// {{ data.tags | related(tags=doc.meta.tags) }}
/// ```
fn filter_related(
    value: &tera::Value,
    args: &HashMap<String, tera::Value>,
) -> tera::Result<tera::Value> {
    let tags: Vec<String> = tera::from_value(value.to_owned()).unwrap_or(Vec::new());
    let Some(index_value) = args.get("index") else {
        return Ok(tera::Value::Array(Vec::new()));
    };
    let index = try_get_value!("related", "value", HashMap<String, Vec<Doc>>, index_value);
    let union = get_union_for_index_keys(&index, &tags);
    let value = tera::to_value(union)?;
    return Ok(value);
}

fn filter_get_deep(
    value: &tera::Value,
    args: &HashMap<String, tera::Value>,
) -> tera::Result<tera::Value> {
    let path = match args.get("path") {
        Some(path_value) => try_get_value!("get_deep", "path", String, path_value),
        None => return Err(tera::Error::msg("get_deep requires path argument")),
    };
    let default = args.get("default").unwrap_or(&tera::Value::Null).to_owned();
    return Ok(get_deep(value, &path).unwrap_or(default));
}

/// Tera filter to render text as slug
/// Example:
/// ```tera
/// {{ "Foo bar" | to_slug }}
/// ```
fn filter_to_slug(
    value: &tera::Value,
    _: &HashMap<String, tera::Value>,
) -> tera::Result<tera::Value> {
    let Some(string) = value.as_str() else {
        return Err(tera::Error::msg(
            "to_slug filter can only be called on strings",
        ));
    };
    let slug = text::to_slug(string);
    Ok(tera::Value::String(slug))
}

/// Deterministically choose an element in an array using the hash of a value
/// to pick.
fn filter_choose_by_hash(
    value: &tera::Value,
    args: &HashMap<String, tera::Value>,
) -> tera::Result<tera::Value> {
    let vec = match value.as_array() {
        Some(vec) => vec,
        None => return Err(tera::Error::msg("must be called on an array")),
    };
    let hashable = match args.get("value") {
        Some(hashable) => hashable,
        None => return Err(tera::Error::msg("value argument needed")),
    };
    let index = {
        let mut hasher = DefaultHasher::new();
        hashable.hash(&mut hasher);
        let hash = hasher.finish();
        (hash % vec.len() as u64) as usize
    };
    let item = vec[index].clone();
    return Ok(item);
}

/// Given an object, return an array of values for that object
pub fn filter_values(
    value: &tera::Value,
    _args: &HashMap<String, tera::Value>,
) -> tera::Result<tera::Value> {
    let object = match value.as_object() {
        Some(object) => object,
        None => return Err(tera::Error::msg("must be called on an object")),
    };
    let values: Vec<tera::Value> = object.values().cloned().collect();
    Ok(tera::Value::Array(values))
}

/// Given an object, return an array of keys for that object
pub fn filter_keys(
    value: &tera::Value,
    _args: &HashMap<String, tera::Value>,
) -> tera::Result<tera::Value> {
    let object = match value.as_object() {
        Some(object) => object,
        None => return Err(tera::Error::msg("must be called on an object")),
    };
    let values: Vec<tera::Value> = object
        .keys()
        .cloned()
        .map(|str| tera::Value::String(str))
        .collect();
    Ok(tera::Value::Array(values))
}

// pub fn filter_values()

/// Decorate Tera instance with Lettersmith-specific configuration
pub fn decorate_renderer(renderer: Tera) -> Tera {
    let mut renderer = renderer;
    renderer.register_filter("related", filter_related);
    renderer.register_filter("markdown", filter_markdown);
    renderer.register_filter("get_deep", filter_get_deep);
    renderer.register_filter("choose_by_hash", filter_choose_by_hash);
    renderer.register_filter("to_slug", filter_to_slug);
    renderer.register_filter("keys", filter_keys);
    renderer.register_filter("values", filter_values);
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
