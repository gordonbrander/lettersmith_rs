use crate::error::{Error, ErrorKind};
use crate::io::write_file_deep;
use crate::json;
use serde::Serialize;
pub use serde_json::from_str;
pub use serde_json::from_value;
pub use serde_json::{json, to_string_pretty, Value};
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::path::PathBuf;

/// Read a JSON file, returning a result of the JSON value.
pub fn read(path: impl AsRef<Path>) -> Result<Value, Error> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let value = serde_json::from_reader(reader)?;
    Ok(value)
}

/// Write serializable value to JSON (pretty printed)
pub fn write_pretty<P, V>(path: P, json: V) -> Result<(), Error>
where
    P: AsRef<Path>,
    V: Serialize,
{
    let content = to_string_pretty(&json)?;
    write_file_deep(path, &content)
}

/// Read a series of paths to JSON files into hashmap of `data` for templates.
/// Returns a Result of `HashMap<String, json::Value>`, where string keys
/// are the file stems of the JSON files.
pub fn read_json_files_as_data_map(
    paths: &Vec<PathBuf>,
) -> Result<HashMap<String, json::Value>, Error> {
    let mut data: HashMap<String, json::Value> = HashMap::new();
    for path in paths {
        let stem = path
            .file_stem()
            .ok_or(Error::new(ErrorKind::Other, "Could not unwrap file stem"))?
            .to_string_lossy()
            .into_owned();
        let value = read(path)?;
        data.insert(stem, value);
    }
    Ok(data)
}

/// Merge two JSON values together
/// https://www.rfc-editor.org/rfc/rfc7396
pub fn merge(a: &mut Value, b: Value) {
    if let Value::Object(a) = a {
        if let Value::Object(b) = b {
            for (k, v) in b {
                if v.is_null() {
                    a.remove(&k);
                } else {
                    merge(a.entry(k).or_insert(Value::Null), v);
                }
            }
            return;
        }
    }

    *a = b;
}
