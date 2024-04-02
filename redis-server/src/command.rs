use std::fmt;
use std::str::FromStr;

use crate::kvstore::KV_STORE;
use crate::serde::{serialize, Error, Response};

#[derive(Debug, PartialEq)]
pub enum CommandType {
    Ping,
    Echo,
    Exists,
    Set,
    Get,
    Del,
    Config,
}

impl FromStr for CommandType {
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

#[derive(Debug, PartialEq)]
pub struct Command {
    command: CommandType,
    args: Vec<String>,
}

#[derive(Debug, PartialEq)]
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
            InvalidCommandError::InvalidCommand => write!(f, "invalid command"),
            InvalidCommandError::NoCommands => write!(f, "no commands"),
            InvalidCommandError::InvalidBulkStringLength => write!(f, "invalid bulk string length"),
            InvalidCommandError::InvalidCommandLength => write!(f, "invalid command length"),
            InvalidCommandError::MissingCommand => write!(f, "missing command"),
        }
    }
}

impl Command {
    pub fn new(command: CommandType, args: Vec<String>) -> Self {
        Self { command, args }
    }

    pub fn new_from_str(s: &str) -> Result<Self, InvalidCommandError> {
        let mut parts = s.split_whitespace();
        let command = CommandType::from_str(parts.next().unwrap_or_default())?;

        Ok(Self {
            command,
            args: parts.map(|s| s.into()).collect(),
        })
    }

    pub fn args(&self) -> Vec<String> {
        self.args.clone()
    }

    pub fn execute(&self) -> Result<String, String> {
        let args = self.args();

        match self.command {
            CommandType::Ping => match args.len() {
                0 => Ok(serialize(Response::SimpleString("PONG".into()))),
                _ => Ok(serialize(Response::SimpleString(args.join(" ")))),
            },
            CommandType::Echo => Ok(serialize(Response::SimpleString(args.join(" ")))),
            CommandType::Exists => {
                if args.len() != 1 {
                    return Err(serialize(Response::Error(Error::new_generic(
                        "EXISTS requires one argument",
                    ))));
                }

                let exists = KV_STORE.exists(args[0].clone());
                Ok(serialize(Response::Integer(exists as i64)))
            }
            CommandType::Set => {
                if args.len() != 2 {
                    return Err(serialize(Response::Error(Error::new_generic(
                        "SET requires two arguments",
                    ))));
                }

                KV_STORE.set(args[0].clone(), args[1].clone());
                Ok(serialize(Response::SimpleString("OK".into())))
            }
            CommandType::Get => {
                if args.len() != 1 {
                    return Err(serialize(Response::Error(Error::new_generic(
                        "GET requires one argument",
                    ))));
                }

                match KV_STORE.get(args[0].clone()) {
                    Some(value) => Ok(serialize(Response::BulkString(value))),
                    None => Ok(serialize(Response::Null)),
                }
            }
            CommandType::Del => {
                if args.len() != 1 {
                    return Err(serialize(Response::Error(Error::new_generic(
                        "DEL requires one argument",
                    ))));
                }

                match KV_STORE.del(args[0].clone()) {
                    Some(_) => Ok(serialize(Response::Integer(1))),
                    None => Ok(serialize(Response::Integer(0))),
                }
            }
            CommandType::Config => {
                if args.len() != 2 {
                    return Err(serialize(Response::Error(Error::new_generic(
                        "CONFIG requires two arguments",
                    ))));
                }

                match args[0].as_str() {
                    "GET" => {
                        match args[1].as_str() {
                            "save" => Ok(serialize(Response::SimpleString("".into()))),
                            "appendonly" => Ok(serialize(Response::SimpleString("no".into()))),
                            _ => Ok(serialize(Response::Error(Error::new_generic(
                                "CONFIG GET only supports save, appendonly",
                            )))),
                        }
                    }
                    _ => Ok(serialize(Response::Error(Error::new_generic(
                        "CONFIG only supports GET",
                    )))),
                }
            },
        }
    }
}
