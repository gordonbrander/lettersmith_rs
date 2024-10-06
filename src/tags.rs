use crate::doc::Doc;
use crate::docs::Docs;
use crate::json::{self, json};
use crate::stub::Stub;
use crate::text::{remove_non_slug_chars, to_slug};
use crate::token_template;
use std::collections::HashMap;
use std::path::PathBuf;
use tap::Pipe;

/// Convert string to tag.
/// Similar to `to_slug()`, but it replaces spaces with underscores instead
/// of dashes. This is to make tag slugs compatible with hashtag syntax.
pub fn to_tag(term: &str) -> String {
    term.trim()
        .to_lowercase()
        .replace(' ', "_")
        .pipe(|s| remove_non_slug_chars(&s))
}

/// Given an index-shaped hashmap and a list of keys, return a combined and
/// deduplicated vector of the items for those keys.
/// We return a vector instead of a HashSet to allow for ordering/sorting.
pub fn get_union_for_index_keys(index: &HashMap<String, Vec<Stub>>, keys: &[String]) -> Vec<Stub> {
    let mut stubs: Vec<Stub> = keys
        .iter()
        .flat_map(move |key| {
            index
                .get(key)
                .into_iter()
                .flatten()
                .map(|item| item.to_owned())
        })
        .collect();
    stubs.dedup();
    stubs
}

impl Doc {
    /// Get get tag string values from meta.
    /// Sluggifies tags to normalize them for string-matching.
    pub fn get_meta_taxonomy(&self, key: &str) -> Option<Vec<String>> {
        match self.meta.get(key) {
            Some(json::Value::Array(tag_values)) => {
                let mut tag_strings: Vec<String> = tag_values
                    .iter()
                    .filter_map(|value| value.as_str())
                    .map(|value| to_tag(value))
                    .collect();
                tag_strings.dedup();
                Some(tag_strings)
            }
            _ => None,
        }
    }

    /// Get get tag string values from meta.
    /// Sluggifies tags to normalize them for string-matching.
    pub fn get_meta_tags(&self) -> Option<Vec<String>> {
        self.get_meta_taxonomy("tags")
    }

    /// Set "related" key on meta
    pub fn set_meta_related(self, related: Vec<Stub>) -> Self {
        self.merge_meta(json!({ "related": related }))
    }

    /// Given an index, check the taxonomy keys in meta, pluck the related stubs
    /// and set them on the "related" field of meta.
    pub fn set_meta_related_from_taxonomy_index(
        self,
        taxonomy_key: &str,
        taxonomy_index: HashMap<String, Vec<Stub>>,
    ) -> Self {
        let Some(tags) = self.get_meta_taxonomy(taxonomy_key) else {
            return self;
        };
        let related: Vec<Stub> = get_union_for_index_keys(&taxonomy_index, &tags);
        self.merge_meta(json!({
            "related": related
        }))
    }

    /// Given an index, check the tag keys in meta, pluck the related stubs
    /// and set them on the "related" field of meta.
    pub fn set_meta_related_from_tag_index(
        self,
        taxonomy_index: HashMap<String, Vec<Stub>>,
    ) -> Self {
        self.set_meta_related_from_taxonomy_index("tags", taxonomy_index)
    }
}

pub trait TaggedDocs: Docs {
    /// Index docs by taxonomy.
    /// Looks for an array in the meta key specified.
    /// Returns a hashmap of stub lists, indexed by term.
    /// Terms are sluggified to normalize them for lookup by key.
    fn index_by_taxonomy(self, key: &str) -> HashMap<String, Vec<Stub>> {
        let mut tax_index: HashMap<String, Vec<Stub>> = HashMap::new();
        for doc in self {
            if let Some(json::Value::Array(terms)) = doc.meta.get(key) {
                for term in terms {
                    if let Some(term) = term.as_str() {
                        tax_index
                            .entry(to_tag(term))
                            .or_insert_with(Vec::new)
                            .push(Stub::from(&doc));
                    }
                }
            }
        }
        tax_index
    }

    /// Index docs by tag
    /// Looks for an array in the meta key "tags".
    /// Returns a hashmap of stub lists, indexed by term.
    fn index_by_tag(&mut self) -> HashMap<String, Vec<Stub>> {
        self.index_by_taxonomy("tags")
    }

    /// Generate taxonomy archive docs for this docs iterator.
    /// Looks up tags by taxonomy and files stubs by tag under generated archive pages.
    /// Returns a new docs iterator made up of just the archives generated.
    fn generate_taxonomy_archives(
        self,
        key: &str,
        output_path_template: &str,
        template_path: Option<PathBuf>,
    ) -> impl Docs {
        let tax_index = self.index_by_taxonomy(key);
        tax_index.into_iter().map(move |(term, stubs)| {
            let mut parts = HashMap::new();
            parts.insert("taxonomy", to_slug(key));
            parts.insert("term", to_slug(&term));
            let output_path = token_template::render(output_path_template, &parts);
            let meta = json!({ "items": stubs });
            let now = chrono::Utc::now();
            Doc {
                id_path: PathBuf::from(&output_path),
                output_path: PathBuf::from(&output_path),
                input_path: None,
                template_path: template_path.clone(),
                created: now,
                modified: now,
                title: term.to_owned(),
                content: "".to_owned(),
                meta,
            }
        })
    }

    /// Generate tag archive docs for this docs iterator.
    fn generate_tag_archives(
        self,
        output_path_template: &str,
        template_path: Option<PathBuf>,
    ) -> impl Docs {
        self.generate_taxonomy_archives("tags", output_path_template, template_path)
    }
}

impl<I> TaggedDocs for I where I: Docs {}
