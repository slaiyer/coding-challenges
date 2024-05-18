use crate::response::types::Response;

use super::types::{ArgumentError, Execute};

/// Represents a command to set a key-value pair in the key-value store.
pub struct Set {
    key: String,
    value: String,
}

impl Execute for Set {
    /// Executes the set command by storing the key-value pair in the key-value store.
    /// Returns a response indicating the success of the operation.
    fn execute(self) -> Response {
        crate::kvstore::KV_STORE.set(&self.key, &self.value);
        Response::ss("OK")
    }
}

/// Builder for constructing a `Set` command.
pub struct Builder {
    key: Option<String>,
    value: Option<String>,
}

impl Builder {
    /// Creates a new `Builder` instance.
    pub const fn new() -> Self {
        Self {
            key: None,
            value: None,
        }
    }

    /// Sets the key for the `Set` command being built.
    pub fn key(mut self, key: &str) -> Self {
        self.key = Some(key.into());
        self
    }

    /// Sets the value for the `Set` command being built.
    pub fn value(mut self, value: &str) -> Self {
        self.value = Some(value.into());
        self
    }

    /// Builds a `Set` command using the provided key and value.
    /// Returns a `Result` indicating whether the command was successfully built or not.
    pub fn build(self) -> Result<Set, ArgumentError> {
        Ok(Set {
            key: self.key.ok_or(ArgumentError::Missing)?,
            value: self.value.ok_or(ArgumentError::Missing)?,
        })
    }
}
