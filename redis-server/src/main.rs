use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

mod serde;
use serde::{TERM, deserialize, serialize, Error, Request, Response};

mod command;

mod kvstore;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:6379").expect("failed to bind to port 6379");

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                handle_client(&mut stream);
            }
            Err(e) => {
                eprintln!("Error: {}", e);
            }
        }
    }
}

fn handle_client(stream: &mut TcpStream) {
    let mut buffer = [0; 1024];
    loop {
        match stream.read(&mut buffer) {
            Ok(_) => {
                // Process the request and send the response
                let response = process_request(&buffer);
                if let Err(e) = stream.write_all(response.as_bytes()) {
                    eprintln!("Failed to write to stream: {}", e);
                    break;
                }
            }
            Err(e) => {
                eprintln!("Failed to read from stream: {}", e);
                break;
            }
        }
    }
}

fn process_request(request: &[u8]) -> String {
    let request_str = match String::from_utf8(request.to_vec()) {
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
            responses.join(TERM)
        }
    }
}
