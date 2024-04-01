use std::collections::HashMap;
use std::sync::Mutex;
use lazy_static::lazy_static;

pub struct KvStore {
    store: Mutex<HashMap<String, String>>,
}

impl KvStore {
    fn new() -> Self {
        Self {
            store: Mutex::new(HashMap::new()),
        }
    }

    pub fn exists(&self, key: String) -> bool {
        let store = self.store.lock().unwrap();
        store.contains_key(&key)
    }

    pub fn set(&self, key: String, value: String) {
        let mut store = self.store.lock().unwrap();
        store.insert(key, value);
    }

    pub fn get(&self, key: String) -> Option<String> {
        let store = self.store.lock().unwrap();
        store.get(&key).cloned()
    }

    pub fn del(&self, key: String) -> Option<String> {
        let mut store = self.store.lock().unwrap();
        store.remove(&key)
    }
}

lazy_static! {
    pub static ref KV_STORE: KvStore = KvStore::new();
}
