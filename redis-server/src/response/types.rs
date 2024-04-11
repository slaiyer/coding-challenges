use std::{error::Error, fmt};

const TERM: &str = "\r\n";

#[derive(Debug, PartialEq, Eq)]
pub enum Response {
    SimpleString(String),
    Error(RedisError),
    Integer(i64),
    Null,
    Array(Vec<String>),
}

impl Response {
    pub fn ss(s: &str) -> Self {
        Self::SimpleString(s.into())
    }

    pub fn err(kind: &str, message: &str) -> Self {
        Self::Error(RedisError::new(kind, message))
    }

    pub fn err_from_error(e: impl Error) -> Self {
        Self::Error(RedisError::new("ERR", e.to_string().as_str()))
    }

    pub const fn i(i: i64) -> Self {
        Self::Integer(i)
    }
    pub const fn null() -> Self {
        Self::Null
    }

    pub fn arr(arr: Vec<String>) -> Self {
        Self::Array(arr)
    }
}

impl fmt::Display for Response {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SimpleString(s) => write!(f, "+{s}{TERM}"),
            Self::Error(e) => write!(f, "{e}"),
            Self::Integer(i) => write!(f, ":{i}{TERM}"),
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

#[derive(Debug, PartialEq, Eq)]
pub struct RedisError {
    kind: String,
    message: String,
}

impl RedisError {
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
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let kind = match self.kind {
            ref s if s.is_empty() => "ERR ".to_string(),
            ref s => format!("{s} "),
        };
        write!(f, "-{}{}{T}", kind, self.message, T = TERM)
    }
}
