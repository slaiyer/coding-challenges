use super::types::Execute;

use crate::response::types::Response;

use std::fmt;

pub struct Ping {
    message: Option<String>,
}

impl Execute for Ping {
    fn execute(self: Box<Self>) -> Response {
        self.message.as_ref().map_or_else(
            || Response::SimpleString("PONG".into()),
            |s| Response::ss(s),
        )
    }
}

impl fmt::Display for Ping {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.message {
            Some(s) => write!(f, "PING {s}"),
            None => write!(f, "PING"),
        }
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

    pub fn build(self) -> Ping {
        Ping {
            message: match self.msg {
                Some(s) if !s.is_empty() => Some(s),
                _ => None,
            }
        }
    }
}
