use super::*;
use std::collections::HashMap;

#[test]
fn test_code_indexer_new() {
    let indexer = CodeIndexer::new();
fn test_code_indexer_new() {
    let indexer = CodeIndexer::new();
    assert_eq!(indexer.suggest("any_query").len(), 0);
}
}

#[test]
fn test_code_indexer_index_code() {
    let mut indexer = CodeIndexer::new();
    indexer.index_code("fn main() {}");
fn test_code_indexer_index_code() {
    let mut indexer = CodeIndexer::new();
    indexer.index_code("fn main() {}");
    let suggestions = indexer.suggest("main");
    assert_eq!(suggestions.len(), 1);
    assert_eq!(suggestions[0], "fn main() {}");
}
}

#[test]
fn test_code_indexer_suggest() {
    let mut indexer = CodeIndexer::new();
    indexer.index_code("fn main() {}");
    indexer.index_code("fn test() {}");
    let suggestions = indexer.suggest("main");
    assert_eq!(suggestions.len(), 1);
    assert_eq!(suggestions[0], "fn main() {}");
    let suggestions = indexer.suggest("fn");
    assert_eq!(suggestions.len(), 2);
}

#[test]
fn test_code_indexer_collision() {
    let mut indexer = CodeIndexer::new();
    // Both snippets have the same length (12 characters)
    indexer.index_code("fn main() {}");
    indexer.index_code("fn test() {}");
    
    // With the current implementation, only the second one would be stored
    let suggestions = indexer.suggest("main");
    
    // This will fail with the current implementation, proving there's a collision issue
    assert_eq!(suggestions.len(), 1);
    assert_eq!(suggestions[0], "fn main() {}");
}
