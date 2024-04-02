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

    pub fn len(&self) -> usize {
        self.store.len()
    }

    pub fn exists(&self, key: &str) -> bool {
        self.store.contains_key(key)
    }

    pub fn set(&self, key: String, value: String) {
        self.store.insert(key, value);
    }

    pub fn get(&self, key: &str) -> Option<String> {
        self.store.get(key).map(|value_ref| value_ref.clone())
    }

    pub fn del(&self, key: &str) -> Option<String> {
        self.store.remove(key).map(|(_, v)| v)
    }
}

pub static KV_STORE: Lazy<KvStore> = Lazy::new(KvStore::new);
