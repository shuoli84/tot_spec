use std::path::PathBuf;

#[derive(Debug, Default)]
pub struct Entry {
    path: PathBuf,
    component: String,
    children: Vec<Entry>,
}

impl Entry {
    fn new(component: String, path: PathBuf) -> Self {
        Self {
            path,
            component,
            children: vec![],
        }
    }

    fn insert(&mut self, components: &[String]) {
        if components.is_empty() {
            return;
        }

        let component = &components[0];

        for child in self.children.iter_mut() {
            if child.component.eq(component) {
                child.insert(&components[1..]);
                return;
            }
        }

        let mut child = Entry::new(component.clone(), self.path.join(component));
        child.insert(&components[1..]);
        self.children.push(child);
    }

    pub fn component(&self) -> &str {
        &self.component
    }

    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    pub fn len(&self) -> usize {
        self.children.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn iter_child(&self) -> impl Iterator<Item = &Entry> {
        self.children.iter()
    }

    fn foreach_entry_recursive(&self, f: &mut impl FnMut(&Entry)) {
        for child in self.children.iter() {
            f(child);
            child.foreach_entry_recursive(f);
        }
    }
}

#[derive(Debug, Default)]
pub struct SpecFolder {
    root: Entry,
}

impl SpecFolder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, path: impl AsRef<std::path::Path>) {
        let components = path
            .as_ref()
            .components()
            .map(|c| c.as_os_str().to_owned().to_str().unwrap().to_string())
            .collect::<Vec<_>>();

        self.root.insert(&components);
    }

    pub fn foreach_entry_recursively(&self, mut f: impl FnMut(&Entry)) {
        (&mut f)(&self.root);
        self.root.foreach_entry_recursive(&mut f);
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn test_stack() {
        let mut stack = SpecFolder::new();

        stack.insert(&PathBuf::from("a/b/c.yaml"));
        stack.insert(&PathBuf::from("a/b/d.yaml"));
        stack.insert(&PathBuf::from("a/b/e/f.yaml"));
        stack.insert(&PathBuf::from("b/e/f.yaml"));

        let mut entries = Vec::new();
        stack.foreach_entry_recursively(|e| {
            entries.push(e.path().clone());
        });

        assert_eq!(
            entries,
            vec![
                "",
                "a",
                "a/b",
                "a/b/c.yaml",
                "a/b/d.yaml",
                "a/b/e",
                "a/b/e/f.yaml",
                "b",
                "b/e",
                "b/e/f.yaml",
            ]
            .iter()
            .map(PathBuf::from)
            .collect::<Vec<_>>()
        );
    }
}
