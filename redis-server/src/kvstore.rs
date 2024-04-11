use dashmap::DashMap;
use once_cell::sync::Lazy;

/// Represents a key-value store.
pub struct KvStore {
    store: DashMap<String, String>,
}

impl KvStore {
    /// Creates a new instance of `KvStore`.
    ///
    /// # Returns
    ///
    /// A new `KvStore` instance.
    fn new() -> Self {
        // TODO: load from disk

        Self {
            store: DashMap::new(),
        }
    }

    /// Saves the contents of the key-value store to disk.
    pub fn save(&self) {
        unimplemented!()
    }

    /// Loads the contents of the key-value store from disk.
    ///
    /// # Returns
    ///
    /// A new `KvStore` instance with the loaded contents.
    fn load(&self) -> Self {
        unimplemented!()
    }

    /// Returns the number of key-value pairs in the store.
    ///
    /// # Returns
    ///
    /// The number of key-value pairs in the store.
    pub fn len(&self) -> usize {
        self.store.len()
    }

    /// Checks if a key exists in the store.
    ///
    /// # Parameters
    ///
    /// - `key`: The key to check.
    ///
    /// # Returns
    ///
    /// `true` if the key exists, `false` otherwise.
    pub fn exists(&self, key: &str) -> bool {
        self.store.contains_key(key)
    }

    /// Retrieves the value associated with a key from the store.
    ///
    /// # Parameters
    ///
    /// - `key`: The key to retrieve the value for.
    ///
    /// # Returns
    ///
    /// The value associated with the key, or `None` if the key does not exist.
    pub fn get(&self, key: &str) -> Option<String> {
        self.store.get(key).map(|v| v.value().clone())
    }

    /// Sets a key-value pair in the store.
    ///
    /// # Parameters
    ///
    /// - `key`: The key to set.
    /// - `value`: The value to associate with the key.
    pub fn set(&self, key: &str, value: &str) {
        self.store.insert(key.into(), value.into());
    }

    /// Removes a key-value pair from the store.
    ///
    /// # Parameters
    ///
    /// - `key`: The key to remove.
    ///
    /// # Returns
    ///
    /// The value associated with the removed key, or `None` if the key does not exist.
    pub fn del(&self, key: &str) -> Option<String> {
        self.store.remove(key).map(|v| v.1)
    }
}

/// Drop is not called for static variables.
impl Drop for KvStore {
    fn drop(&mut self) {
        self.save();
    }
}

/// The global instance of `KvStore`.
pub static KV_STORE: Lazy<KvStore> = Lazy::new(KvStore::new);
