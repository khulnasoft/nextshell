use std::collections::HashMap;

pub struct CodeIndexer {
    index: HashMap<String, String>,
}

impl CodeIndexer {
    pub fn new() -> Self {
        CodeIndexer {
            index: HashMap::new(),
        }
    }

    pub fn index_code(&mut self, code: &str) {
        // Simple indexing logic: store the code with its length as the key
        self.index.insert(code.len().to_string(), code.to_string());
    }

    pub fn suggest(&self, query: &str) -> Vec<String> {
        // Simple suggestion logic: return all code snippets that contain the query
        self.index
            .values()
            .filter(|&code| code.contains(query))
            .cloned()
            .collect()
    }
}
