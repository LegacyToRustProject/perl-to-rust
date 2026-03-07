pub struct MyModule {
    items: Vec<String>,
}

impl MyModule {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    pub fn add_item(&mut self, item: String) {
        self.items.push(item);
    }

    pub fn count(&self) -> usize {
        self.items.len()
    }

    pub fn all_items(&self) -> &[String] {
        &self.items
    }
}

impl std::fmt::Display for MyModule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.items.join(", "))
    }
}
