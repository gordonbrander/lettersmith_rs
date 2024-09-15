pub use serde_json::Value;
use std::fs::File;
use std::io::{BufReader, Result};
use std::path::Path;

/// Read a JSON file, returning a result of the JSON value.
pub fn read(path: impl AsRef<Path>) -> Result<Value> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let value = serde_json::from_reader(reader)?;
    Ok(value)
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
