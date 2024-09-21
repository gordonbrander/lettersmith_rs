use crate::docs::Docs;
use crate::json;
use crate::stub::Stub;
use std::collections::HashMap;

/// Given an index-shaped hashmap and a list of keys, return a combined
/// iterator of the items of those keys.
pub fn combine_index_keys<'a>(
    index: &'a HashMap<String, Vec<Stub>>,
    keys: &'a [String],
) -> impl Iterator<Item = &'a Stub> + 'a {
    keys.iter()
        .flat_map(move |key| index.get(key).into_iter().flatten())
}

pub trait TaggedDocs: Docs {
    /// Index docs by taxonomy.
    /// Looks for an array in the meta key specified.
    /// Returns a hashmap of stub lists, indexed by term.
    /// ```
    ///    {
    ///      "term_a": [stub, ...],
    ///      "term_b": [stub, ...]
    ///    }
    fn index_by_taxonomy(&mut self, key: &str) -> HashMap<String, Vec<Stub>> {
        let mut tax_index: HashMap<String, Vec<Stub>> = HashMap::new();
        for doc in self {
            if let Some(json::Value::Array(terms)) = doc.meta.get(key) {
                for term in terms {
                    if let Some(term) = term.as_str() {
                        tax_index
                            .entry(term.to_string())
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
    /// ```
    ///    {
    ///      "term_a": [stub, ...],
    ///      "term_b": [stub, ...]
    ///    }
    fn index_by_tag(&mut self) -> HashMap<String, Vec<Stub>> {
        self.index_by_taxonomy("tags")
    }
}

impl<I> TaggedDocs for I where I: Docs {}
