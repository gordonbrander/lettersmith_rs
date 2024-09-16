use std::error::Error;
use std::fs::{self, File};
use std::io::Write;
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
