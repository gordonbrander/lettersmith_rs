use crate::doc::Doc;
use crate::docs::{DocResults, Docs};
use crate::error::Error;
use crate::json::get_deep;
use crate::markdown::render_markdown;
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
    let markdown_str = value.as_str().ok_or(tera::Error::msg(
        "Markdown filter can only be called on strings",
    ))?;
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
/// {{ doc | related(index=data.tags) }}
/// ```
fn filter_related(
    value: &tera::Value,
    args: &HashMap<String, tera::Value>,
) -> tera::Result<tera::Value> {
    let doc: Doc = tera::from_value(value.to_owned())?;
    let taxonomy_key = args
        .get("key")
        .map(|value| value.as_str())
        .flatten()
        .unwrap_or("tags");
    let Some(index_value) = args.get("index") else {
        return Ok(tera::Value::Array(Vec::new()));
    };
    let index: HashMap<String, Vec<Doc>> = tera::from_value(index_value.to_owned())?;
    let related = doc.get_related_from_tag_index(taxonomy_key, index);
    let value = tera::to_value(related)?;
    return Ok(value);
}

fn filter_path(
    value: &tera::Value,
    args: &HashMap<String, tera::Value>,
) -> tera::Result<tera::Value> {
    let path = match args.get("attribute") {
        Some(path_value) => try_get_value!("path", "attribute", String, path_value),
        None => return Err(tera::Error::msg("requires path argument")),
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
    let str = value
        .as_str()
        .ok_or(tera::Error::msg("must be called on a string"))?;
    let slug = text::to_slug(str);
    Ok(tera::Value::String(slug))
}

/// Deterministically choose an element in an array using the hash of a value
/// to pick.
fn filter_choose_by_hash(
    value: &tera::Value,
    args: &HashMap<String, tera::Value>,
) -> tera::Result<tera::Value> {
    let vec = value
        .as_array()
        .ok_or(tera::Error::msg("must be called on an array"))?;
    let hashable = args
        .get("value")
        .ok_or(tera::Error::msg("value argument needed"))?;
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
    let object = value
        .as_object()
        .ok_or(tera::Error::msg("must be called on an object"))?;
    let values: Vec<tera::Value> = object.values().cloned().collect();
    Ok(tera::Value::Array(values))
}

/// Given an object, return an array of keys for that object
pub fn filter_keys(
    value: &tera::Value,
    _args: &HashMap<String, tera::Value>,
) -> tera::Result<tera::Value> {
    let object = value
        .as_object()
        .ok_or(tera::Error::msg("must be called on an object"))?;
    let values: Vec<tera::Value> = object
        .keys()
        .cloned()
        .map(|str| tera::Value::String(str))
        .collect();
    Ok(tera::Value::Array(values))
}

/// Filter docs by id_path, using a glob pattern.
pub fn filter_filter_by_id_path(
    value: &tera::Value,
    args: &HashMap<String, tera::Value>,
) -> tera::Result<tera::Value> {
    let glob = args
        .get("glob")
        .ok_or(tera::Error::msg("glob argument needed"))?
        .as_str()
        .ok_or(tera::Error::msg("glob argument must be a string"))?;
    let matching_docs: Vec<tera::Value> = tera::from_value(value.to_owned())
        .unwrap_or(Vec::new())
        .into_iter()
        .filter_matching(glob)
        .map(|doc| tera::to_value(doc))
        .filter_map(|doc| doc.ok())
        .collect();
    Ok(tera::Value::Array(matching_docs))
}

/// Decorate Tera instance with Lettersmith-specific configuration
pub fn decorate_renderer(renderer: Tera) -> Tera {
    let mut renderer = renderer;
    renderer.register_filter("related", filter_related);
    renderer.register_filter("markdown", filter_markdown);
    renderer.register_filter("path", filter_path);
    renderer.register_filter("choose_by_hash", filter_choose_by_hash);
    renderer.register_filter("to_slug", filter_to_slug);
    renderer.register_filter("slugify", filter_to_slug);
    renderer.register_filter("keys", filter_keys);
    renderer.register_filter("values", filter_values);
    renderer.register_filter("filter_by_id_path", filter_filter_by_id_path);
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
