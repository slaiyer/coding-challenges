/// This module contains the implementation of the `Exists` command.
/// The `Exists` command checks if a key exists in the key-value store.
/// It implements the `Execute` trait and returns a `Response` indicating whether the key exists or not.
use super::types::{ArgumentError, Execute};
use crate::kvstore::KV_STORE;
use crate::response::types::Response;

/// Represents the `Exists` command.
pub struct Exists {
    key: String,
}

impl Execute for Exists {
    /// Executes the `Exists` command and returns a `Response`.
    fn execute(self: Box<Self>) -> Response {
        Response::ss(&u64::from(KV_STORE.exists(&self.key)).to_string())
    }
}

/// Builder for the `Exists` command.
pub struct Builder {
    key: Option<String>,
}

impl Builder {
    /// Creates a new `Builder` instance.
    pub const fn new() -> Self {
        Self { key: None }
    }

    /// Sets the key for the `Exists` command.
    pub fn key(mut self, key: &str) -> Self {
        self.key = Some(key.into());
        self
    }

    /// Builds the `Exists` command.
    /// Returns a `Result` with the built `Exists` instance or an `ArgumentError` if the key is missing.
    pub fn build(self) -> Result<Exists, ArgumentError> {
        Ok(Exists {
            key: self.key.ok_or(ArgumentError::Missing)?,
        })
    }
}
