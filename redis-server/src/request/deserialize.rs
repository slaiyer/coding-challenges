/// This module provides functions for deserializing Redis requests.
use crate::{
    command::types::{Command, Execute},
    response::types::Response,
};

use super::types::Request;

/// Parses a Redis request into a vector of executable commands.
///
/// # Arguments
/// * `request` - The Redis request to parse.
///
/// # Returns
/// * `Result<Vec<Box<dyn Execute>>, String>` - The parsed commands as a vector of executable commands, or an error message.
pub fn parse_commands(request: &Request) -> Result<Vec<Box<dyn Execute>>, String> {
    let mut commands: Vec<Box<dyn Execute>> = Vec::new();
    for cmd in request.commands() {
        let cmd_type = match cmd.first() {
            Some(cmd) => match cmd.parse::<Command>() {
                Ok(result) => result,
                Err(error) => return Err(error.to_string()),
            },
            None => return Err(Response::err("", "empty command").to_string()),
        };

        commands.push(match cmd_type {
            Command::Ping(builder) => match cmd.len() {
                1 => Box::new(builder.build()),
                2 => Box::new(builder.message(cmd[1].as_str()).build()),
                _ => {
                    return Err(
                        Response::err("", "unexpected number of arguments for PING").to_string()
                    )
                }
            },
            Command::Echo(builder) => match cmd.len() {
                2 => match builder.message(cmd[1].as_str()).build() {
                    Ok(result) => Box::new(result),
                    Err(error) => return Err(error.to_string()),
                },
                _ => {
                    return Err(
                        Response::err("", "unexpected number of arguments for ECHO").to_string()
                    )
                }
            },
            Command::Exists(builder) => match cmd.len() {
                2 => match builder.key(cmd[1].as_str()).build() {
                    Ok(result) => Box::new(result),
                    Err(error) => return Err(error.to_string()),
                },
                _ => {
                    return Err(
                        Response::err("", "unexpected number of arguments for EXISTS").to_string(),
                    )
                }
            },
            Command::Config(builder) => match cmd.len() {
                3 => match builder.args(cmd[1..].to_vec()).build() {
                    Ok(result) => Box::new(result),
                    Err(error) => return Err(error.to_string()),
                },
                _ => {
                    return Err(
                        Response::err("", "unexpected number of arguments for CONFIG").to_string(),
                    )
                }
            },
            Command::Set(builder) => match cmd.len() {
                3 => match builder.key(cmd[1].as_str()).value(cmd[2].as_str()).build() {
                    Ok(result) => Box::new(result),
                    Err(error) => return Err(error.to_string()),
                },
                _ => {
                    return Err(
                        Response::err("", "unexpected number of arguments for SET").to_string()
                    )
                }
            },
            Command::Get(builder) => match cmd.len() {
                2 => match builder.key(cmd[1].as_str()).build() {
                    Ok(result) => Box::new(result),
                    Err(error) => return Err(error.to_string()),
                },
                _ => {
                    return Err(
                        Response::err("", "unexpected number of arguments for GET").to_string()
                    )
                }
            },
            Command::Del(builder) => match cmd.len() {
                2 => match builder.key(cmd[1].as_str()).build() {
                    Ok(result) => Box::new(result),
                    Err(error) => return Err(error.to_string()),
                },
                _ => {
                    return Err(
                        Response::err("", "unexpected number of arguments for DEL").to_string()
                    )
                }
            },
        });
    }
    Ok(commands)
}

#[cfg(test)]
/// Module containing unit tests for the `stringify` and `parse_commands` functions.
mod tests {
    use super::*;

    /// Test case for `parse_commands` function with an "echo" command.
    #[test]
    fn test_parse_commands_echo() {
        let request = "echo ling\r\n".parse::<Request>().unwrap();
        let result = parse_commands(&request);
        assert!(result.is_ok());
        let commands = result.unwrap();
        assert_eq!(commands.len(), 1);
        assert_eq!(
            commands
                .into_iter()
                .map(|cmd| cmd.execute().to_string())
                .collect::<String>(),
            "+ling\r\n"
        );
    }
}
