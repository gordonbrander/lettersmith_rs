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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_with_substitutions() {
        let mut parts = HashMap::new();
        parts.insert("name", "Alice".to_string());
        parts.insert("age", "30".to_string());

        let template = "Hello, :name! You are :age years old.";
        let result = render(template, &parts);

        assert_eq!(result, "Hello, Alice! You are 30 years old.");
    }

    #[test]
    fn test_render_with_missing_substitutions() {
        let mut parts = HashMap::new();
        parts.insert("name", "Bob".to_string());

        let template = "Hello, :name! Your email is :email.";
        let result = render(template, &parts);

        assert_eq!(result, "Hello, Bob! Your email is :email.");
    }

    #[test]
    fn test_render_with_empty_template() {
        let parts = HashMap::new();
        let template = "";
        let result = render(template, &parts);

        assert_eq!(result, "");
    }

    #[test]
    fn test_render_with_empty_hashmap() {
        let parts = HashMap::new();
        let template = "This is a :test template.";
        let result = render(template, &parts);

        assert_eq!(result, "This is a :test template.");
    }
}
