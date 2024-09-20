use crate::doc::Doc;
use crate::docs::Docs;
use crate::token_template;
use std::collections::HashMap;

impl Doc {
    /// Read permalink template parts from a document.
    pub fn get_permalink_template_parts(&self) -> Option<HashMap<&str, String>> {
        let name = self.id_path.file_name()?.to_string_lossy().into_owned();
        let stem = self.id_path.file_stem()?.to_string_lossy().into_owned();
        let ext = self.id_path.extension()?.to_string_lossy().into_owned();
        let parents = self.id_path.parent()?.to_string_lossy().into_owned();
        let parent = self
            .id_path
            .parent()?
            .file_name()?
            .to_string_lossy()
            .into_owned();
        let yyyy = self.created.format("%Y").to_string();
        let yy = self.created.format("%y").to_string();
        let mm = self.created.format("%m").to_string();
        let dd = self.created.format("%d").to_string();
        let mut map = HashMap::new();
        map.insert("name", name);
        map.insert("stem", stem);
        map.insert("ext", ext);
        map.insert("parents", parents);
        map.insert("parent", parent);
        map.insert("yyyy", yyyy);
        map.insert("yy", yy);
        map.insert("mm", mm);
        map.insert("dd", dd);
        Some(map)
    }

    /// Render doc permalink (output path) using a template
    pub fn permalink(self, permalink_template: impl Into<String>) -> Self {
        let parts = self.get_permalink_template_parts().unwrap_or_default();
        let output_path = token_template::render(permalink_template, &parts);
        self.set_output_path(output_path)
    }
}

pub trait PermalinkDocs: Docs {
    /// Render markdown content for all docs
    fn permalink(self, permalink_template: impl Into<String>) -> impl Docs {
        let permalink_template: String = permalink_template.into();
        self.map(move |doc| doc.permalink(&permalink_template))
    }

    fn blog_permalink(self) -> impl Docs {
        self.permalink(":yyyy/:mm/:dd/:stem/index.html")
    }

    fn page_permalink(self) -> impl Docs {
        self.permalink(":parents/:stem/index.html")
    }
}

impl<I> PermalinkDocs for I where I: Docs {}
