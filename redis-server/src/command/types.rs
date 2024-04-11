use std::{error::Error, fmt, str::FromStr};

use crate::response::types::Response;

use super::{config, echo, exists, ping, set, get, del};

// TODO: make this trait required for all commands via a derive macro
pub trait Execute {
    fn execute(self: Box<Self>) -> Response;
}

pub enum Command {
    Ping(ping::Builder),
    Echo(echo::Builder),
    Config(config::Builder),
    Exists(exists::Builder),
    Set(set::Builder),
    Get(get::Builder),
    Del(del::Builder),
    // LPush,
    // RPush,
    // Save,
}

#[derive(Debug)]
pub enum CommandError {
    Unknown,
}

impl Error for CommandError {}

impl fmt::Display for CommandError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::Unknown => write!(f, "unknown command"),
        }
    }
}

impl FromStr for Command {
    type Err = CommandError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "PING" => Ok(Self::Ping(ping::Builder::new())),
            "ECHO" => Ok(Self::Echo(echo::Builder::new())),
            "CONFIG" => Ok(Self::Config(config::Builder::new())),
            "EXISTS" => Ok(Self::Exists(exists::Builder::new())),
            "SET" => Ok(Self::Set(set::Builder::new())),
            "GET" => Ok(Self::Get(get::Builder::new())),
            "DEL" => Ok(Self::Del(del::Builder::new())),
            // "LPUSH" => Ok(Self::LPush),
            // "RPUSH" => Ok(Self::RPush),
            // "SAVE" => Ok(Self::Save),
            _ => Err(Self::Err::Unknown),
        }
    }
}

#[derive(Debug)]
pub enum SubcommandError {
    Missing,
    Unknown,
}

impl Error for SubcommandError {}

impl fmt::Display for SubcommandError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::Missing => write!(f, "missing subcommand"),
            Self::Unknown => write!(f, "unknown subcommand"),
        }
    }
}

#[derive(Debug)]
pub enum ArgumentError {
    Missing,
}

impl Error for ArgumentError {}

impl fmt::Display for ArgumentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::Missing => write!(f, "missing argument"),
        }
    }
}
