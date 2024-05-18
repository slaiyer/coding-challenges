use super::types::{ArgumentError, Execute, SubcommandError};

use crate::response::types::Response;

use core::fmt;
use std::{error::Error, str::FromStr, vec};

#[derive(Debug)]
pub struct Config {
    subcommand: ConfigSubcommand,
    args: Vec<String>,
}

#[derive(Debug)]
enum ConfigSubcommand {
    Get,
}

impl FromStr for ConfigSubcommand {
    type Err = SubcommandError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "GET" => Ok(Self::Get),
            _ => Err(Self::Err::Unknown),
        }
    }
}

impl Execute for Config {
    fn execute(self) -> Response {
        match &self.subcommand {
            ConfigSubcommand::Get => self.args.first().map_or_else(
                || Response::err("", "missing argument for CONFIG GET"),
                |arg| match arg.as_str() {
                    "save" => Response::arr(vec!["save".into(), String::new()]),
                    "appendonly" => Response::arr(vec!["appendonly".into(), "no".into()]),
                    _ => Response::err("", "unexpected CONFIG GET argument"),
                },
            ),
        }
    }
}

pub struct Builder {
    args_raw: Option<Vec<String>>,
}

#[derive(Debug)]
pub enum CommandBuildError {
    Subcommand(SubcommandError),
    Argument(ArgumentError),
}

impl Error for CommandBuildError {}

impl fmt::Display for CommandBuildError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Subcommand(e) => write!(f, "{e}"),
            Self::Argument(e) => write!(f, "{e}"),
        }
    }
}

impl From<SubcommandError> for CommandBuildError {
    fn from(e: SubcommandError) -> Self {
        Self::Subcommand(e)
    }
}

impl Builder {
    pub const fn new() -> Self {
        Self { args_raw: None }
    }

    pub fn args(mut self, args: Vec<String>) -> Self {
        self.args_raw = Some(args);
        self
    }

    pub fn build(self) -> Result<Config, CommandBuildError> {
        let Some(args) = self.args_raw else {
            return Err(CommandBuildError::Subcommand(SubcommandError::Missing));
        };

        let subcommand = match args.first() {
            Some(sub) => ConfigSubcommand::from_str(sub)?,
            None => return Err(CommandBuildError::Subcommand(SubcommandError::Missing)),
        };

        let mut args_iter = args.iter().skip(1);
        if args_iter.any(String::is_empty) {
            Err(CommandBuildError::Argument(ArgumentError::Missing))
        } else {
            Ok(Config {
                subcommand,
                args: args_iter.map(String::to_string).collect(),
            })
        }
    }
}
