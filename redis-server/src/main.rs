#![warn(
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
    rust_2024_compatibility,
    future_incompatible
)]

use std::{error, str::FromStr};

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::spawn;

use tracing::{error, instrument};

mod kvstore;
use kvstore::KV_STORE;

mod request;
use request::{deserialize, types::Request};

mod response;

mod command;

/// The main entry point of the Redis server.
#[tokio::main]
async fn main() -> Result<(), Box<dyn error::Error>> {
    tracing_subscriber::fmt::init();

    KV_STORE.len(); // initialize singleton

    let listener = TcpListener::bind("127.0.0.1:6379").await?;

    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                spawn(handle_client(stream));
            }
            Err(e) => {
                error!("failed to accept connection: {e:?}");
            }
        }
    }
}

/// Handles a client connection by reading requests and sending responses.
#[instrument]
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
                    error!("failed writing to stream: {e:?}");
                    break;
                }
            }
            Err(e) => {
                error!("failed reading from stream: {e:?}");
                break;
            }
        }
    }
}

/// Processes a request and returns the corresponding response.
fn process(request_buf: &[u8]) -> String {
    let request_str = match deserialize::stringify(request_buf) {
        Ok(result) => result,
        Err(error) => return error.to_string(),
    };

    let request = match Request::from_str(request_str) {
        Ok(result) => result,
        Err(error) => return error.to_string(),
    };

    let commands = match deserialize::parse_commands(&request) {
        Ok(value) => value,
        Err(value) => return value,
    };

    commands
        .into_iter()
        .map(command::types::Execute::execute)
        .map(String::from)
        .collect()
}
