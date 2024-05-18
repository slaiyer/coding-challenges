#![warn(clippy::all, clippy::pedantic, future_incompatible)]

use response::types::Response;
use std::error;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::spawn;
use tracing::{error, instrument};

mod kvstore;
use kvstore::KV_STORE;
mod request;
use request::{deserialize, types::Request};
mod command;
mod response;

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
    let mut buffer = [0; 1_024];

    loop {
        match stream.read(&mut buffer).await {
            Ok(buf_len) => {
                if buf_len == 0 {
                    break;
                }

                let response = process(&buffer);
                if let Err(e) = stream.write_all(response.as_bytes()).await {
                    error!("failed writing to stream: {e:?}");
                    break;
                }
            }
            Err(e) => {
                if e.raw_os_error() != Some(54) {
                    // ignore connection reset by peer
                    error!("failed reading from stream: {e:?}");
                }
                break;
            }
        }
    }
}

/// Processes a request and returns the corresponding response.
fn process(request_buf: &[u8]) -> String {
    Request::try_from(request_buf)
        .map_err(Response::err_from_error)
        .and_then(|request: Request| deserialize::parse_commands(&request))
        .map_or_else(
            |error| error.to_string(),
            |commands| {
                commands
                    .into_iter()
                    .map(command::types::Execute::execute)
                    .map(String::from)
                    .collect()
            },
        )
}
