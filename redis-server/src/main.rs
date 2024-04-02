#![warn(unused_extern_crates)]

use std::error;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::spawn;

mod serde;
use serde::{deserialize, serialize, Error, Request, Response};

mod command;

mod kvstore;
use kvstore::KV_STORE;

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
                eprintln!("failed to accept connection: {:?}", e);
            }
        }
    }
}

async fn handle_client(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    loop {
        match stream.read(&mut buffer).await {
            Ok(_) => {
                let response = process_request(&buffer);
                if let Err(e) = stream.write_all(response.as_bytes()).await {
                    eprintln!("failed reading from stream: {:?}", e);
                    break;
                }
            }
            Err(e) => {
                eprintln!("failed writing to stream: {:?}", e);
                break;
            }
        }
    }
}

fn process_request(request_buf: &[u8]) -> String {
    let request_str = match String::from_utf8(request_buf.to_vec()) {
        Ok(r) => r,
        Err(_) => return serialize(Response::Error(Error::new_generic("invalid request"))),
    };

    let request = match deserialize(&request_str) {
        Ok(r) => r,
        Err(e) => return serialize(Response::Error(Error::new_generic(e.to_string().as_str()))),
    };

    handle_command(request)
}

fn handle_command(request: Request) -> String {
    match request {
        Request::InlineCommand(c) => match c.execute() {
            Ok(response) => response,
            Err(error) => error,
        },
        Request::Array(commands) => {
            let mut responses = Vec::<String>::new();
            for c in commands {
                match c.execute() {
                    Ok(response) => responses.push(response),
                    Err(error) => responses.push(error),
                }
            }
            responses.concat()
        }
    }
}
