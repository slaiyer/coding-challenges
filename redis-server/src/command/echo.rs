use crate::response::types::Response;

use super::types::{ArgumentError, Execute};

/// Represents the Echo command, which echoes back a message.
pub struct Echo {
    message: String,
}

impl Execute for Echo {
    /// Executes the Echo command and returns the response.
    fn execute(self) -> Response {
        Response::ss(self.message.as_str())
    }
}

/// Builder for creating an Echo instance.
pub struct Builder {
    msg: Option<String>,
}

impl Builder {
    /// Creates a new Builder instance.
    pub const fn new() -> Self {
        Self { msg: None }
    }

    /// Sets the message for the Echo command.
    pub fn message(mut self, message: &str) -> Self {
        self.msg = Some(message.into());
        self
    }

    /// Builds the Echo instance.
    ///
    /// # Errors
    ///
    /// Returns an `ArgumentError::Missing` if the message is not provided.
    pub fn build(self) -> Result<Echo, ArgumentError> {
        Ok(Echo {
            message: self.msg.ok_or(ArgumentError::Missing)?,
        })
    }
}
