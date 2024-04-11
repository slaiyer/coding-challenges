use crate::response::types::Response;

use super::types::{ArgumentError, Execute};

pub struct Set {
    key: String,
    value: String,
}

impl Execute for Set {
    fn execute(self: Box<Self>) -> Response {
        crate::kvstore::KV_STORE.set(&self.key, &self.value);
        Response::ss("OK")
    }
}

pub struct Builder {
    key: Option<String>,
    value: Option<String>,
}

impl Builder {
    pub const fn new() -> Self {
        Self {
            key: None,
            value: None,
        }
    }

    pub fn key(mut self, key: &str) -> Self {
        self.key = Some(key.into());
        self
    }

    pub fn value(mut self, value: &str) -> Self {
        self.value = Some(value.into());
        self
    }

    pub fn build(self) -> Result<Set, ArgumentError> {
        Ok(Set {
            key: self.key.ok_or(ArgumentError::Missing)?,
            value: self.value.ok_or(ArgumentError::Missing)?,
        })
    }
}
