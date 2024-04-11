use super::types::{ArgumentError, Execute};

use crate::response::types::Response;

pub struct Echo {
    message: String,
}

impl Execute for Echo {
    fn execute(self: Box<Self>) -> Response {
        Response::ss(self.message.as_str())
    }
}

pub struct Builder {
    msg: Option<String>,
}

impl Builder {
    pub const fn new() -> Self {
        Self { msg: None }
    }

    pub fn message(mut self, message: &str) -> Self {
        self.msg = Some(message.into());
        self
    }

    pub fn build(self) -> Result<Echo, ArgumentError> {
        Ok(Echo {
            message: self.msg.ok_or(ArgumentError::Missing)?,
        })
    }
}
