use dashmap::DashMap;
use once_cell::sync::Lazy;

pub struct KvStore {
    store: DashMap<String, String>,
}

impl KvStore {
    fn new() -> Self {
        Self {
            store: DashMap::new(),
        }
    }

    pub fn exists(&self, key: String) -> bool {
        self.store.contains_key(&key)
    }

    pub fn set(&self, key: String, value: String) {
        self.store.insert(key, value);
    }

    pub fn get(&self, key: String) -> Option<String> {
        self.store.get(&key).map(|value_ref| value_ref.clone())
    }

    pub fn del(&self, key: String) -> Option<String> {
        self.store.remove(&key).map(|(_, v)| v)
    }
}

pub static KV_STORE: Lazy<KvStore> = Lazy::new(KvStore::new);
