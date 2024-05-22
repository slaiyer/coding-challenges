/// This module provides functions for deserializing Redis requests.
use crate::{
    command::types::{Command, CommandBuilder},
    response::types::Response,
};

use super::types::Request;

/// Parses a Redis request into a vector of executable commands.
///
/// # Arguments
/// * `request` - The Redis request to parse.
///
/// # Returns
/// * `Result<Vec<Command>, String>` - A vector of executable commands, or an error message.
pub fn parse_commands(request: &Request) -> Result<Vec<Command>, Response> {
    let mut commands: Vec<Command> = Vec::new();
    for cmd in request.commands() {
        let cmd_type = match cmd.first() {
            Some(cmd) => match cmd.parse::<CommandBuilder>() {
                Ok(result) => result,
                Err(error) => return Err(Response::from(error)),
            },
            None => return Err(Response::err("", "empty command")),
        };

        commands.push(match cmd_type {
            CommandBuilder::Ping(builder) => match cmd.len() {
                1 => Command::Ping(builder.build()),
                2 => Command::Ping(builder.message(cmd[1].as_str()).build()),
                _ => return Err(Response::err("", "unexpected number of arguments for PING")),
            },
            CommandBuilder::Echo(builder) => match cmd.len() {
                2 => match builder.message(cmd[1].as_str()).build() {
                    Ok(result) => Command::Echo(result),
                    Err(error) => return Err(Response::from(error)),
                },
                _ => return Err(Response::err("", "unexpected number of arguments for ECHO")),
            },
            CommandBuilder::Exists(builder) => match cmd.len() {
                2 => match builder.key(cmd[1].as_str()).build() {
                    Ok(result) => Command::Exists(result),

                    Err(error) => return Err(Response::from(error)),
                },
                _ => {
                    return Err(Response::err(
                        "",
                        "unexpected number of arguments for EXISTS",
                    ))
                }
            },
            CommandBuilder::Config(builder) => match cmd.len() {
                3 => match builder.args(cmd[1..].to_vec()).build() {
                    Ok(result) => Command::Config(result),
                    Err(error) => return Err(Response::from(error)),
                },
                _ => {
                    return Err(Response::err(
                        "",
                        "unexpected number of arguments for CONFIG",
                    ))
                }
            },
            CommandBuilder::Set(builder) => match cmd.len() {
                3 => match builder.key(cmd[1].as_str()).value(cmd[2].as_str()).build() {
                    Ok(result) => Command::Set(result),

                    Err(error) => return Err(Response::from(error)),
                },
                _ => return Err(Response::err("", "unexpected number of arguments for SET")),
            },
            CommandBuilder::Get(builder) => match cmd.len() {
                2 => match builder.key(cmd[1].as_str()).build() {
                    Ok(result) => Command::Get(result),
                    Err(error) => return Err(Response::from(error)),
                },
                _ => return Err(Response::err("", "unexpected number of arguments for GET")),
            },
            CommandBuilder::Del(builder) => match cmd.len() {
                2 => match builder.key(cmd[1].as_str()).build() {
                    Ok(result) => Command::Del(result),
                    Err(error) => return Err(Response::from(error)),
                },
                _ => return Err(Response::err("", "unexpected number of arguments for DEL")),
            },
        });
    }
    Ok(commands)
}

#[cfg(test)]
/// Module containing unit tests for the `stringify` and `parse_commands` functions.
mod tests {
    use super::*;
    use crate::command;

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
                .map(command::types::Execute::execute)
                .map(String::from)
                .collect::<String>(),
            "+ling\r\n"
        );
    }
}
