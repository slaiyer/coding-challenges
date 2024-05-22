#![warn(clippy::all, clippy::pedantic, future_incompatible)]

use response::types::Response;
use std::io;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::spawn;
use tracing::{error, instrument};

mod kvstore;
use kvstore::KV_STORE;

mod request;
use request::types::Request;

mod command;
use command::types::Command;
mod response;

/// The main entry point of the Redis server.
#[tokio::main]
async fn main() -> Result<(), io::Error> {
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
        .map_err(Response::from)
        .and_then(Vec::<Command>::try_from)
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
