use std::collections::HashMap;
use std::sync::Mutex;

/// In-memory upload store for testing.
pub struct InMemoryUploadStore {
    sessions: Mutex<HashMap<String, serde_json::Value>>,
}

impl InMemoryUploadStore {
    pub fn new() -> Self {
        Self {
            sessions: Mutex::new(HashMap::new()),
        }
    }

    pub fn insert(&self, session: serde_json::Value) {
        let id = session["id"].as_str().unwrap().to_string();
        self.sessions.lock().unwrap().insert(id, session);
    }

    pub fn get(&self, id: &str) -> Option<serde_json::Value> {
        self.sessions.lock().unwrap().get(id).cloned()
    }

    pub fn list_by_space(&self, space_id: &str) -> Vec<serde_json::Value> {
        self.sessions
            .lock()
            .unwrap()
            .values()
            .filter(|s| s["space_id"].as_str() == Some(space_id))
            .cloned()
            .collect()
    }

    pub fn list_by_state(&self, state: &str) -> Vec<serde_json::Value> {
        self.sessions
            .lock()
            .unwrap()
            .values()
            .filter(|s| s["state"].as_str() == Some(state))
            .cloned()
            .collect()
    }

    pub fn delete(&self, id: &str) -> bool {
        self.sessions.lock().unwrap().remove(id).is_some()
    }

    pub fn count(&self) -> usize {
        self.sessions.lock().unwrap().len()
    }

    pub fn clear(&self) {
        self.sessions.lock().unwrap().clear();
    }
}

impl Default for InMemoryUploadStore {
    fn default() -> Self {
        Self::new()
    }
}
