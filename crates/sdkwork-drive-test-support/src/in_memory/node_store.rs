use std::collections::HashMap;
use std::sync::Mutex;

/// In-memory node store for testing.
pub struct InMemoryNodeStore {
    nodes: Mutex<HashMap<String, serde_json::Value>>,
}

impl InMemoryNodeStore {
    pub fn new() -> Self {
        Self {
            nodes: Mutex::new(HashMap::new()),
        }
    }

    pub fn insert(&self, node: serde_json::Value) {
        let id = node["id"].as_str().unwrap().to_string();
        self.nodes.lock().unwrap().insert(id, node);
    }

    pub fn get(&self, id: &str) -> Option<serde_json::Value> {
        self.nodes.lock().unwrap().get(id).cloned()
    }

    pub fn list_by_space(&self, space_id: &str) -> Vec<serde_json::Value> {
        self.nodes
            .lock()
            .unwrap()
            .values()
            .filter(|n| n["space_id"].as_str() == Some(space_id))
            .cloned()
            .collect()
    }

    pub fn list_by_parent(&self, parent_id: &str) -> Vec<serde_json::Value> {
        self.nodes
            .lock()
            .unwrap()
            .values()
            .filter(|n| n["parent_id"].as_str() == Some(parent_id))
            .cloned()
            .collect()
    }

    pub fn delete(&self, id: &str) -> bool {
        self.nodes.lock().unwrap().remove(id).is_some()
    }

    pub fn count(&self) -> usize {
        self.nodes.lock().unwrap().len()
    }

    pub fn clear(&self) {
        self.nodes.lock().unwrap().clear();
    }
}

impl Default for InMemoryNodeStore {
    fn default() -> Self {
        Self::new()
    }
}
