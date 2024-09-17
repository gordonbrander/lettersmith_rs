use std::collections::HashMap;

/// Render a simple string template, where variables are prefixed by `:`.
/// Substitutions not present in hashmap will be left untouched.
pub fn render(template: impl Into<String>, parts: &HashMap<&str, String>) -> String {
    let mut result: String = template.into();
    for (key, value) in parts {
        result = result.replace(&format!(":{}", key), value);
    }
    result
}
