use dashmap::DashMap;
use once_cell::sync::Lazy;

pub struct KvStore {
    store: DashMap<String, String>,
}

impl KvStore {
    fn new() -> Self {
        // TODO: load from disk

        Self {
            store: DashMap::new(),
        }
    }

    fn save(&self) {
        unimplemented!()
    }

    fn load(&self) -> Self {
        unimplemented!()
    }

    pub fn len(&self) -> usize {
        self.store.len()
    }

    pub fn exists(&self, key: &str) -> bool {
        self.store.contains_key(key)
    }

    fn get(&self, key: &str) -> Option<String> {
        self.store.get(key).map(|v| v.value().clone())
    }

    fn set(&self, key: &str, value: &str) {
        self.store.insert(key.into(), value.into());
    }

    fn del(&self, key: &str) -> Option<String> {
        self.store.remove(key).map(|v| v.1)
    }
}

impl Drop for KvStore {
    fn drop(&mut self) {
        self.save();
    }
}

pub static KV_STORE: Lazy<KvStore> = Lazy::new(KvStore::new);
