// Import necessary modules
use crate::{kvstore::KV_STORE, response::types::Response};

// Import types from the same module
use super::types::{ArgumentError, Execute};

// Define a struct for the Get command
pub struct Get {
    key: String,
}

// Implement the Execute trait for the Get command
impl Execute for Get {
    // Define the execute method for the Get command
    fn execute(self: Box<Self>) -> Response {
        // Use the KV_STORE to get the value associated with the key
        // If the key is not found, return a Null response
        // Otherwise, return a response with the value
        KV_STORE
            .get(&self.key)
            .map_or(Response::Null, |value| Response::ss(&value))
    }
}

// Define a struct for the Builder of the Get command
pub struct Builder {
    key: Option<String>,
}

impl Builder {
    // Define a new method to create a new Builder instance
    pub const fn new() -> Self {
        Self { key: None }
    }

    // Define a key method to set the key for the Builder
    pub fn key(mut self, key: &str) -> Self {
        self.key = Some(key.into());
        self
    }

    // Define a build method to build a Get instance from the Builder
    pub fn build(self) -> Result<Get, ArgumentError> {
        // If the key is present, create a Get instance with the key
        // Otherwise, return an error indicating a missing key
        Ok(Get {
            key: self.key.ok_or(ArgumentError::Missing)?,
        })
    }
}
