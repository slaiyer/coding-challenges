use std::{error::Error, fmt};

use crate::command::{
    config::CommandBuildError,
    types::{ArgumentError, CommandError},
};
use crate::request::types::ParseError;

const TERM: &str = "\r\n";

/// Represents the possible types of responses from a Redis server.
#[derive(Debug, PartialEq, Eq)]
pub enum Response {
    /// Represents a simple string response.
    SimpleString(String),
    /// Represents an error response.
    Error(RedisError),
    /// Represents a null response.
    Null,
    /// Represents an array response.
    Array(Vec<String>),
}

impl Response {
    /// Creates a new `Response` object with a simple string response.
    pub fn ss(s: &str) -> Self {
        Self::SimpleString(s.into())
    }

    /// Creates a new `Response` object with an error response.
    pub fn err(kind: &str, message: &str) -> Self {
        Self::Error(RedisError::new(kind, message))
    }

    /// Creates a new `Response` object from a Rust `Error` trait object.
    pub fn err_from_error(e: impl Error) -> Self {
        Self::Error(RedisError::new("ERR", e.to_string().as_str()))
    }

    /// Creates a new `Response` object with an array response.
    pub fn arr(arr: Vec<String>) -> Self {
        Self::Array(arr)
    }
}

impl fmt::Display for Response {
    /// Formats the `Response` object as a string.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::SimpleString(s) => write!(f, "+{s}{TERM}"),
            Self::Error(e) => write!(f, "{e}"),
            Self::Null => write!(f, "$-1{TERM}"),
            Self::Array(arr) => {
                let mut res = format!("*{}{TERM}", arr.len());
                for s in arr {
                    res.push_str(&format!("${}{TERM}{s}{TERM}", s.len()));
                }
                write!(f, "{res}")
            }
        }
    }
}

impl From<Response> for String {
    /// Converts a `Response` object to a string.
    fn from(r: Response) -> Self {
        format!("{r}")
    }
}

impl From<ParseError> for Response {
    fn from(e: ParseError) -> Self {
        Self::err_from_error(e)
    }
}

impl From<CommandError> for Response {
    fn from(e: CommandError) -> Self {
        Self::err_from_error(e)
    }
}

impl From<ArgumentError> for Response {
    fn from(e: ArgumentError) -> Self {
        Self::err_from_error(e)
    }
}

impl From<CommandBuildError> for Response {
    fn from(e: CommandBuildError) -> Self {
        Self::err_from_error(e)
    }
}

/// Represents an error returned by a Redis server.
#[derive(Debug, PartialEq, Eq)]
pub struct RedisError {
    kind: String,
    message: String,
}

impl RedisError {
    /// Creates a new `RedisError` object with the specified kind and message.
    pub fn new(kind: &str, message: &str) -> Self {
        Self {
            kind: match kind {
                "" => "ERR",
                _ => kind,
            }
            .into(),
            message: message.into(),
        }
    }
}

impl fmt::Display for RedisError {
    /// Formats the `RedisError` object as a string.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let kind = match self.kind {
            ref s if s.is_empty() => "ERR ".to_string(),
            ref s => format!("{s} "),
        };
        write!(f, "-{}{}{T}", kind, self.message, T = TERM)
    }
}
