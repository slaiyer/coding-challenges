/// This module contains the implementation of the `Del` command.
/// The `Del` command is used to delete a key from the key-value store.
use crate::{kvstore::KV_STORE, response::types::Response};

use super::types::{ArgumentError, Execute};

/// Represents the `Del` command.
pub struct Del {
    key: String,
}

impl Execute for Del {
    /// Executes the `Del` command by deleting the specified key from the key-value store.
    fn execute(self: Box<Self>) -> Response {
        KV_STORE.del(&self.key);
        Response::ss("OK")
    }
}

/// Builder for the `Del` command.
pub struct Builder {
    key: Option<String>,
}

impl Builder {
    /// Creates a new `Builder` instance.
    pub const fn new() -> Self {
        Self { key: None }
    }

    /// Sets the key for the `Del` command.
    pub fn key(mut self, key: &str) -> Self {
        self.key = Some(key.into());
        self
    }

    /// Builds a `Del` instance from the builder.
    ///
    /// # Errors
    ///
    /// Returns an `ArgumentError::Missing` if the key is not provided.
    pub fn build(self) -> Result<Del, ArgumentError> {
        Ok(Del {
            key: self.key.ok_or(ArgumentError::Missing)?,
        })
    }
}
