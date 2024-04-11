use super::types::{ArgumentError, Execute, SubcommandError};

use crate::response::types::Response;

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
    fn execute(self: Box<Self>) -> Response {
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

impl Builder {
    pub const fn new() -> Self {
        Self { args_raw: None }
    }

    pub fn args(mut self, args: Vec<String>) -> Self {
        self.args_raw = Some(args);
        self
    }

    pub fn build(self) -> Result<Config, Box<dyn Error>> {
        let Some(args_raw) = self.args_raw else {
            return Err(Box::new(SubcommandError::Missing));
        };

        let subcommand = match args_raw.first() {
            Some(sub) => sub.parse()?,
            None => return Err(Box::new(SubcommandError::Missing)),
        };

        let args: Vec<_> = args_raw.into_iter().skip(1).collect();
        if args.iter().any(String::is_empty) {
            Err(Box::new(ArgumentError::Missing))
        } else {
            Ok(Config { subcommand, args })
        }
    }
}
