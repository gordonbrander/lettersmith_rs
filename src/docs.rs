/// Functions for working with iterators of docs
use crate::doc::Doc;
use std::collections::HashSet;
use std::path::Path;

/// Filter out docs with a given id_path
pub fn remove_with_id_path(
    docs: impl Iterator<Item = Doc>,
    id_path: impl AsRef<Path>,
) -> impl Iterator<Item = Doc> {
    docs.filter(move |doc| doc.id_path != id_path.as_ref())
}

/// Filter docs who's id_path matches a glob pattern.
pub fn filter_matching(
    docs: impl Iterator<Item = Doc>,
    glob_pattern: &str,
) -> impl Iterator<Item = Doc> {
    let matcher = glob::Pattern::new(glob_pattern).expect("Invalid glob pattern");
    docs.filter(move |doc| {
        matcher.matches(
            doc.id_path
                .to_str()
                .expect("Could not convert path to string"),
        )
    })
}

/// Filter out docs who's file name in the id_path starts with an underscore.
pub fn remove_drafts(docs: impl Iterator<Item = Doc>) -> impl Iterator<Item = Doc> {
    docs.filter(|doc| {
        !Path::new(&doc.id_path)
            .file_name()
            .expect("Could not get file name")
            .to_str()
            .expect("Cound not convert file name to str")
            .starts_with('_')
    })
}

/// Filter out docs who's file name in the id_path is "index".
pub fn remove_index(docs: impl Iterator<Item = Doc>) -> impl Iterator<Item = Doc> {
    docs.filter(|doc| {
        Path::new(&doc.id_path)
            .file_stem()
            .expect("Could not get file stem from path")
            != "index"
    })
}

/// De-duplicate docs by id_path
pub fn dedupe(docs: impl Iterator<Item = Doc>) -> impl Iterator<Item = Doc> {
    let mut seen = HashSet::new();
    docs.filter(move |doc| seen.insert(doc.id_path.clone()))
}

/// Sort docs by created date
pub fn sort_by_created(docs: impl Iterator<Item = Doc>) -> impl Iterator<Item = Doc> {
    let mut docs_vec: Vec<Doc> = docs.into_iter().collect();
    docs_vec.sort_by(|a, b| b.created.cmp(&a.created));
    docs_vec.into_iter()
}

/// Sort docs by modified date
pub fn sort_by_modified(docs: impl Iterator<Item = Doc>) -> impl Iterator<Item = Doc> {
    let mut docs_vec: Vec<Doc> = docs.into_iter().collect();
    docs_vec.sort_by(|a, b| b.modified.cmp(&a.modified));
    docs_vec.into_iter()
}

/// Sort docs by title (A-Z)
pub fn sort_by_title(docs: impl Iterator<Item = Doc>) -> impl Iterator<Item = Doc> {
    let mut docs_vec: Vec<Doc> = docs.into_iter().collect();
    docs_vec.sort_by(|a, b| a.title.cmp(&b.title));
    docs_vec.into_iter()
}
