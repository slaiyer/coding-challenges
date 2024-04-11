use crate::{kvstore::KV_STORE, response::types::Response};

use super::types::{ArgumentError, Execute};

pub struct Del {
    key: String,
}

impl Execute for Del {
    fn execute(self: Box<Self>) -> Response {
        KV_STORE.del(&self.key);
        Response::ss("OK")
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

    pub fn build(self) -> Result<Del, ArgumentError> {
        Ok(Del {
            key: self.key.ok_or(ArgumentError::Missing)?,
        })
    }
}
