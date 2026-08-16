use std::collections::HashMap;
use std::sync::Mutex;

/// In-memory space store for testing.
pub struct InMemorySpaceStore {
    spaces: Mutex<HashMap<String, serde_json::Value>>,
}

impl InMemorySpaceStore {
    pub fn new() -> Self {
        Self {
            spaces: Mutex::new(HashMap::new()),
        }
    }

    pub fn insert(&self, space: serde_json::Value) {
        let id = space["id"].as_str().unwrap().to_string();
        self.spaces.lock().unwrap().insert(id, space);
    }

    pub fn get(&self, id: &str) -> Option<serde_json::Value> {
        self.spaces.lock().unwrap().get(id).cloned()
    }

    pub fn list_by_tenant(&self, tenant_id: &str) -> Vec<serde_json::Value> {
        self.spaces
            .lock()
            .unwrap()
            .values()
            .filter(|s| s["tenant_id"].as_str() == Some(tenant_id))
            .cloned()
            .collect()
    }

    pub fn delete(&self, id: &str) -> bool {
        self.spaces.lock().unwrap().remove(id).is_some()
    }

    pub fn count(&self) -> usize {
        self.spaces.lock().unwrap().len()
    }

    pub fn clear(&self) {
        self.spaces.lock().unwrap().clear();
    }
}

impl Default for InMemorySpaceStore {
    fn default() -> Self {
        Self::new()
    }
}
