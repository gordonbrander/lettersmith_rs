// Utilities for reading/writing a collection of docs to a JSON file
use crate::doc::Doc;
use crate::docs::Docs;
use crate::error::Error;
use crate::io::write_file_deep;
use crate::stub::{Stub, Stubs};
use std::fs::read_to_string;
use std::path::Path;

/// Read JSON Doc archive at path to a vec of Docs
pub fn read(path: impl AsRef<Path>) -> Result<Vec<Doc>, Error> {
    let json_string = read_to_string(path)?;
    let docs: Vec<Doc> = serde_json::from_str(&json_string)?;
    return Ok(docs);
}

pub trait ArchiveDocs: Docs {
    fn write_archive(self, path: &Path) -> Result<(), Error> {
        let docs: Vec<Doc> = self.collect();
        let json = serde_json::to_string(&docs)?;
        write_file_deep(path, &json)?;
        Ok(())
    }
}

impl<I> ArchiveDocs for I where I: Iterator<Item = Doc> {}

pub trait ArchiveStubs: Stubs {
    fn write_archive(self, path: &Path) -> Result<(), Error> {
        let stubs: Vec<Stub> = self.collect();
        let json = serde_json::to_string(&stubs)?;
        write_file_deep(path, &json)?;
        Ok(())
    }
}

impl<I> ArchiveStubs for I where I: Iterator<Item = Stub> {}
