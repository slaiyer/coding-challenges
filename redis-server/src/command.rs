use std::fmt;
use std::str::FromStr;

use crate::kvstore::KV_STORE;
use crate::serde::{Error, Response};

#[derive(Debug, PartialEq, Eq)]
pub enum CmdType {
    Ping,
    Echo,
    Exists,
    Set,
    Get,
    Del,
    Config,
}

impl FromStr for CmdType {
    type Err = InvalidCommandError;

    fn from_str(s: &str) -> Result<Self, InvalidCommandError> {
        match s.to_uppercase().as_str() {
            "PING" => Ok(Self::Ping),
            "ECHO" => Ok(Self::Echo),
            "EXISTS" => Ok(Self::Exists),
            "GET" => Ok(Self::Get),
            "SET" => Ok(Self::Set),
            "DEL" => Ok(Self::Del),
            "CONFIG" => Ok(Self::Config),
            _ => Err(InvalidCommandError::InvalidCommand),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Command {
    command: CmdType,
    args: Vec<String>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum InvalidCommandError {
    InvalidCommand,
    NoCommands,
    InvalidBulkStringLength,
    InvalidCommandLength,
    MissingCommand,
}

impl fmt::Display for InvalidCommandError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Self::InvalidCommand => write!(f, "invalid command"),
            Self::NoCommands => write!(f, "no commands"),
            Self::InvalidBulkStringLength => write!(f, "invalid bulk string length"),
            Self::InvalidCommandLength => write!(f, "invalid command length"),
            Self::MissingCommand => write!(f, "missing command"),
        }
    }
}

impl Command {
    pub fn new(command: CmdType, args: Vec<String>) -> Self {
        Self { command, args }
    }

    pub fn new_from_str(s: &str) -> Result<Self, InvalidCommandError> {
        let mut parts = s.split_whitespace();
        let command = CmdType::from_str(parts.next().unwrap_or_default())?;

        Ok(Self {
            command,
            args: parts.map(std::convert::Into::into).collect(),
        })
    }

    pub fn args(&self) -> Vec<String> {
        self.args.clone()
    }

    pub fn execute(&self) -> Result<String, String> {
        let args = self.args();

        match self.command {
            CmdType::Ping => match args.len() {
                0 => Ok(Response::SimpleString("PONG".into()).to_string()),
                _ => Ok(Response::SimpleString(args.join(" ")).to_string()),
            },
            CmdType::Echo => Ok(Response::SimpleString(args.join(" ")).to_string()),
            CmdType::Exists => {
                if args.len() != 1 {
                    return Err(Response::Error(Error::new_generic(
                        "EXISTS requires one argument",
                    )).to_string());
                }

                let exists = KV_STORE.exists(&args[0]);
                Ok(Response::Integer(i64::from(exists)).to_string())
            }
            CmdType::Set => {
                if args.len() != 2 {
                    return Err(Response::Error(Error::new_generic(
                        "SET requires two arguments",
                    )).to_string());
                }

                KV_STORE.set(args[0].clone(), args[1].clone());
                Ok(Response::SimpleString("OK".into()).to_string())
            }
            CmdType::Get => {
                if args.len() != 1 {
                    return Err(Response::Error(Error::new_generic(
                        "GET requires one argument",
                    )).to_string());
                }

                KV_STORE.get(&args[0]).map_or_else(
                    || Ok(Response::Null.to_string()),
                    |value| Ok(Response::BulkString(value).to_string()),
                )
            }
            CmdType::Del => {
                if args.len() != 1 {
                    return Err(Response::Error(Error::new_generic(
                        "DEL requires one argument",
                    )).to_string());
                }

                match KV_STORE.del(&args[0]) {
                    Some(_) => Ok(Response::Integer(1).to_string()),
                    None => Ok(Response::Integer(0).to_string()),
                }
            }
            CmdType::Config => {
                if args.len() != 2 {
                    return Err(Response::Error(Error::new_generic(
                        "CONFIG requires two arguments",
                    )).to_string());
                }

                match args[0].as_str().to_uppercase().as_str() {
                    // dummy response for redis-benchmark
                    "GET" => Ok(Response::Array(vec!["save".into(), String::new()]).to_string()
                        + Response::Array(vec!["appendonly".into(), "no".into()]).to_string().as_str()),
                    _ => Err(Response::Error(Error::new_generic(
                        "CONFIG only supports GET",
                    )).to_string()),
                }
            }
        }
    }
}
