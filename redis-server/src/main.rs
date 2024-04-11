#![warn(rust_2018_idioms, future_incompatible)]
#![warn(clippy::all, clippy::pedantic, clippy::nursery)]

use std::error;

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



    String::new()
}
