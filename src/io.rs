use crate::doc::Doc;
use serde_json;
use std::error::Error;
use std::fs::{self, File};
use std::io::Write;
use std::io::{self, BufRead};
use std::path::Path;

/// Filter out errors and log them to stderr.
/// Returns a new iterator of only the successful values.
pub fn dump_errors_to_stderr<T, E>(
    iter: impl Iterator<Item = Result<T, E>>,
) -> impl Iterator<Item = T>
where
    E: Error,
{
    iter.filter_map(|result| match result {
        Ok(value) => Some(value),
        Err(err) => {
            eprintln!("Error: {}", err);
            None
        }
    })
}

/// Parse JSON documents from stdin as line-separated JSON.
/// Returns an iterator of doc results.
pub fn read_stdin() -> impl Iterator<Item = Result<Doc, impl Error>> {
    io::stdin()
        .lock()
        .lines()
        .filter_map(Result::ok)
        .map(|line| serde_json::from_str(&line))
}

/// Load documents from an iterator of paths.
/// Errors are output to stderr.
/// Returns an iterator of docs.
pub fn read_docs<P, I>(paths: I) -> impl Iterator<Item = Result<Doc, impl Error>>
where
    P: AsRef<Path>,
    I: IntoIterator<Item = P>,
{
    paths.into_iter().map(|path| Doc::load(path))
}

/// Write documents to stdout as line-separated JSON.
/// Any errors are output to stderr.
pub fn write_stdio(docs: impl IntoIterator<Item = Doc>) {
    for doc in docs {
        let serialized = serde_json::to_string(&doc);
        match serialized {
            Ok(json) => {
                println!("{}", json);
            }
            Err(err) => {
                eprintln!("Error serializing doc: {}", err);
            }
        }
    }
}

/// Write docs to the output directory.
/// Logs successful writes to stdout.
/// Logs errors to stderr.
pub fn write_docs(docs: impl IntoIterator<Item = Doc>, output_dir: &Path) {
    for doc in docs {
        match doc.write(output_dir) {
            Ok(_) => println!("Wrote {:?} to {:?}", doc.id_path, doc.output_path),
            Err(err) => eprintln!("Error writing doc: {}", err),
        }
    }
}

/// Write content to a file, creating directories if necessary.
///
/// # Arguments
///
/// * `path` - The path to the file to write.
/// * `content` - The content to write to the file.
///
/// # Errors
///
/// This function will return an error if the file cannot be created or written to,
/// or if the directories cannot be created.
pub fn write_file_deep<P: AsRef<Path>>(path: P, content: &str) -> std::io::Result<()> {
    // Create parent directories if they don't exist
    if let Some(parent) = path.as_ref().parent() {
        fs::create_dir_all(parent)?;
    }

    // Write the content to the file
    let mut file = File::create(path)?;
    file.write_all(content.as_bytes())?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_write_file_deep() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("deep/nested/file.txt");
        let content = "Hello, world!";

        write_file_deep(&file_path, content).unwrap();

        assert!(file_path.exists());
        assert_eq!(fs::read_to_string(file_path).unwrap(), content);
    }
}
