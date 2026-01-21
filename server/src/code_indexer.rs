use std::collections::HashMap;
use md5;

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
        // Use a unique identifier for the code snippet instead of length
        let key = format!("{:x}", md5::compute(code));
        self.index.insert(key, code.to_string());
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
