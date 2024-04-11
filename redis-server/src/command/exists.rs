use super::types::{ArgumentError, Execute};

use crate::response::types::Response;

use crate::kvstore::KV_STORE;

struct Exists {
    key: String,
}

impl Execute for Exists {
    fn execute(&self) -> Response {
        Response::ss(&u64::from(KV_STORE.exists(&self.key)).to_string())
    }
}

pub struct Builder {
    key: Option<String>,
}

impl Builder {
    pub const fn new() -> Self {
        Self { key: None }
    }

    fn key(mut self, key: &str) -> Self {
        self.key = Some(key.into());
        self
    }

    fn build(self) -> Result<Exists, ArgumentError> {
        Ok(Exists {
            key: self.key.ok_or(ArgumentError::Missing)?,
        })
    }
}
