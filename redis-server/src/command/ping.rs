/// This module contains the implementation of the `Ping` command.
/// The `Ping` command is used to check if the server is alive.
/// It returns a response with the message "PONG" if the server is alive.
/// If a custom message is provided, it returns the custom message instead.
use super::types::Execute;

use crate::response::types::Response;

use std::fmt;

/// Represents the `Ping` command.
pub struct Ping {
    message: Option<String>,
}

impl Execute for Ping {
    /// Executes the `Ping` command and returns the response.
    fn execute(self) -> Response {
        self.message.as_ref().map_or_else(
            || Response::SimpleString("PONG".into()),
            |s| Response::ss(s),
        )
    }
}

impl fmt::Display for Ping {
    /// Formats the `Ping` command for display purposes.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.message {
            Some(s) => write!(f, "PING {s}"),
            None => write!(f, "PING"),
        }
    }
}

/// Builder for the `Ping` command.
pub struct Builder {
    msg: Option<String>,
}

impl Builder {
    /// Creates a new `Builder` instance.
    pub const fn new() -> Self {
        Self { msg: None }
    }

    /// Sets the custom message for the `Ping` command.
    pub fn message(mut self, message: &str) -> Self {
        self.msg = Some(message.into());
        self
    }

    /// Builds the `Ping` command with the provided message.
    pub fn build(self) -> Ping {
        Ping {
            message: match self.msg {
                Some(s) if !s.is_empty() => Some(s),
                _ => None,
            },
        }
    }
}
