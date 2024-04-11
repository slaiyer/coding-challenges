#![warn(rust_2018_idioms, future_incompatible)]
#![warn(clippy::all, clippy::pedantic, clippy::nursery)]

use std::error;

use command::types::{Command, Execute};
use response::types::Response;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::spawn;

mod kvstore;
use kvstore::KV_STORE;

mod request;
use request::{deserialize, types::Request};

mod response;

mod command;

#[tokio::main]
async fn main() -> Result<(), Box<dyn error::Error>> {
    KV_STORE.len(); // initialize singleton

    let listener = TcpListener::bind("127.0.0.1:6379").await?;

    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                spawn(handle_client(stream));
            }
            Err(e) => {
                eprintln!("failed to accept connection: {e:?}");
            }
        }
    }
}

async fn handle_client(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    loop {
        match stream.read(&mut buffer).await {
            Ok(_) => {
                let first_byte = buffer[0];
                if first_byte == 0 || first_byte == b'\r' || first_byte == b'\n' {
                    continue;
                }

                let response = process(&buffer);
                if let Err(e) = stream.write_all(response.as_bytes()).await {
                    eprintln!("failed writing to stream: {e:?}");
                    break;
                }
            }
            Err(e) => {
                eprintln!("failed reading from stream: {e:?}");
                break;
            }
        }
    }
}

fn process(request_buf: &[u8]) -> String {
    let request_str = match deserialize::stringify(request_buf) {
        Ok(result) => result,
        Err(error) => return error.to_string(),
    };

    let request = match request_str.parse::<Request>() {
        Ok(result) => result,
        Err(error) => return error.to_string(),
    };

    let commands = match parse_commands(&request) {
        Ok(value) => value,
        Err(value) => return value,
    };

    commands
        .into_iter()
        .map(|command| command.execute().to_string())
        .collect()
}

fn parse_commands(request: &Request) -> Result<Vec<Box<dyn Execute>>, String> {
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
                        Response::err("", "unexpected number of arguments for SET").to_string(),
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
                        Response::err("", "unexpected number of arguments for GET").to_string(),
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
                        Response::err("", "unexpected number of arguments for DEL").to_string(),
                    )
                }
            },
        });
    }
    Ok(commands)
}
