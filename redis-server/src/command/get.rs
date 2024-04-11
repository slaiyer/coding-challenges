use crate::{kvstore::KV_STORE, response::types::Response};

use super::types::{ArgumentError, Execute};

pub struct Get {
    key: String,
}

impl Execute for Get {
    fn execute(self: Box<Self>) -> Response {
        KV_STORE
            .get(&self.key)
            .map_or(Response::Null, |value| Response::ss(&value))
    }
}

pub struct Builder {
    key: Option<String>,
}

impl Builder {
    pub const fn new() -> Self {
        Self { key: None }
    }

    pub fn key(mut self, key: &str) -> Self {
        self.key = Some(key.into());
        self
    }

    pub fn build(self) -> Result<Get, ArgumentError> {
        Ok(Get {
            key: self.key.ok_or(ArgumentError::Missing)?,
        })
    }
}
