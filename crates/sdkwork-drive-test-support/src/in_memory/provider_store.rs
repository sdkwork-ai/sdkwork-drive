use std::collections::HashMap;
use std::sync::Mutex;

/// In-memory provider store for testing.
pub struct InMemoryProviderStore {
    providers: Mutex<HashMap<String, serde_json::Value>>,
}

impl InMemoryProviderStore {
    pub fn new() -> Self {
        Self {
            providers: Mutex::new(HashMap::new()),
        }
    }

    pub fn insert(&self, provider: serde_json::Value) {
        let id = provider["id"].as_str().unwrap().to_string();
        self.providers.lock().unwrap().insert(id, provider);
    }

    pub fn get(&self, id: &str) -> Option<serde_json::Value> {
        self.providers.lock().unwrap().get(id).cloned()
    }

    pub fn list_all(&self) -> Vec<serde_json::Value> {
        self.providers.lock().unwrap().values().cloned().collect()
    }

    pub fn list_by_status(&self, status: &str) -> Vec<serde_json::Value> {
        self.providers
            .lock()
            .unwrap()
            .values()
            .filter(|p| p["status"].as_str() == Some(status))
            .cloned()
            .collect()
    }

    pub fn delete(&self, id: &str) -> bool {
        self.providers.lock().unwrap().remove(id).is_some()
    }

    pub fn count(&self) -> usize {
        self.providers.lock().unwrap().len()
    }

    pub fn clear(&self) {
        self.providers.lock().unwrap().clear();
    }
}

impl Default for InMemoryProviderStore {
    fn default() -> Self {
        Self::new()
    }
}
