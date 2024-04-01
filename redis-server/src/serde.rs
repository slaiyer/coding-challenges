use std::fmt;

use crate::command::{Command, InvalidCommandError};

pub const TERM: &str = "\r\n";

#[derive(Debug, PartialEq)]
pub enum Request {
    Array(Vec<Command>),
    InlineCommand(Command),
}

impl fmt::Display for Response {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Response::SimpleString(ref s) => write!(f, "+{}{T}", s, T = TERM),
            Response::Error(ref s) => s.fmt(f),
            Response::Integer(i) => write!(f, ":{}{T}", i, T = TERM),
            Response::BulkString(ref s) => write!(f, "${}{T}{}{T}", s.len(), s, T = TERM),
            Response::Null => write!(f, "$-1{T}", T = TERM),
            Response::Array(ref v) => write!(
                f,
                "*{}{T}{}",
                v.len(),
                v.iter()
                    .map(|s| format!("${}{T}{}{T}", s.len(), s, T = TERM))
                    .collect::<Vec<String>>()
                    .concat(),
                T = TERM,
            ),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Error {
    kind: String,
    message: String,
}

impl Error {
    pub fn new_generic(message: &str) -> Self {
        Self {
            kind: "ERR".into(),
            message: message.into(),
        }
    }

    pub fn new_specific(kind: &str, message: &str) -> Self {
        Self {
            kind: kind.into(),
            message: message.into(),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let kind = match self.kind {
            ref s if s.is_empty() => "ERR ".to_string(),
            ref s => format!("{} ", s),
        };
        write!(f, "-{}{}{T}", kind, self.message, T = TERM)
    }
}

#[derive(PartialEq)]
pub enum Response {
    SimpleString(String),
    Error(Error),
    Integer(i64),
    BulkString(String),
    Null,
    Array(Vec<String>),
}

pub fn deserialize(s: &str) -> Result<Request, InvalidCommandError> {
    let mut lines = s.lines();
    let line = match lines.next() {
        Some(l) => l,
        None => return Err(InvalidCommandError::NoCommands),
    };
    match line.chars().next() {
        Some('*') => {
            let num_lines = match line[1..].parse::<usize>() {
                Ok(n) => n,
                Err(_) => return Err(InvalidCommandError::InvalidBulkStringLength),
            };
            parse_bulk_string(num_lines, lines)
        }
        _ => Command::new_from_str(line).map(Request::InlineCommand),
    }
}

fn parse_bulk_string(
    num_lines: usize,
    mut lines: std::str::Lines<'_>,
) -> Result<Request, InvalidCommandError> {
    let mut commands = Vec::<Command>::new();

    let mut line: &str;
    for _ in 0..num_lines {
        line = match lines.next() {
            Some(l) => l,
            None => return Err(InvalidCommandError::NoCommands),
        };
        let len_first_byte = &line[..1];
        if len_first_byte != "$" {
            return Err(InvalidCommandError::InvalidCommandLength);
        };
        let len = match line[1..].parse::<usize>() {
            Ok(n) => n,
            Err(_) => return Err(InvalidCommandError::InvalidCommandLength),
        };
        line = match lines.next() {
            Some(l) => l,
            None => return Err(InvalidCommandError::MissingCommand),
        };
        let command = match Command::new_from_str(&line[..len]) {
            Ok(c) => c,
            Err(_) => return Err(InvalidCommandError::InvalidCommand),
        };
        commands.push(command);
    }
    Ok(Request::Array(commands))
}

pub fn serialize(r: Response) -> String {
    r.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command::CommandType;

    #[test]
    fn test_deserialize_null_bulk_string() {
        let a = Response::Null;
        let b = "$-1\r\n";
        assert_eq!(serialize(a), b);
    }

    #[test]
    fn test_deserialize_array_ping() {
        let a = Response::Array(["ping".into()].into());
        let b = "*1\r\n$4\r\nping\r\n";
        assert_eq!(serialize(a), b);
    }

    #[test]
    fn test_deserialize_integer_666() {
        let a = Response::Integer(666);
        let b = ":666\r\n";
        assert_eq!(serialize(a), b);
    }

    #[test]
    fn test_deserialize_integer_minus_1000() {
        let a = Response::Integer(-1000);
        let b = ":-1000\r\n";
        assert_eq!(serialize(a), b);
    }

    #[test]
    fn test_deserialize_array_echo_hello_world() {
        let a = Response::Array(["echo".into(), "hello world".into()].into());
        let b = "*2\r\n$4\r\necho\r\n$11\r\nhello world\r\n";
        assert_eq!(serialize(a), b);
    }

    #[test]
    fn test_deserialize_array_get_key() {
        let a = Response::Array(["get".into(), "key".into()].into());
        let b = "*2\r\n$3\r\nget\r\n$3\r\nkey\r\n";
        assert_eq!(serialize(a), b);
    }

    #[test]
    fn test_deserialize_simple_string_ok() {
        let a = Response::SimpleString("OK".into());
        let b = "+OK\r\n";
        assert_eq!(serialize(a), b);
    }

    #[test]
    fn test_deserialize_error_message() {
        let a = Response::Error(Error::new_specific("", "Error message"));
        let b = "-ERR Error message\r\n";
        assert_eq!(serialize(a), b);
    }

    #[test]
    fn test_deserialize_empty_bulk_string() {
        let a = Response::BulkString("".into());
        let b = "$0\r\n\r\n";
        assert_eq!(serialize(a), b);
    }

    #[test]
    fn test_deserialize_simple_string_hello_world() {
        let a = Response::SimpleString("hello world".into());
        let b = "+hello world\r\n";
        assert_eq!(serialize(a), b);
    }

    #[test]
    fn test_deserialize_inline_ping() {
        let a = "ping\r\n";
        let b = Request::InlineCommand(Command::new(CommandType::Ping, vec![]));
        assert_eq!(deserialize(a).unwrap(), b);
    }

    #[test]
    fn test_deserialize_inline_ping_message() {
        let a = "ping ling\r\n";
        let b = Request::InlineCommand(Command::new(CommandType::Ping, vec!["ling".into()]));
        assert_eq!(deserialize(a).unwrap(), b);
    }

    #[test]
    fn test_deserialize_inline_echo() {
        let a = "echo\r\n";
        let b = Request::InlineCommand(Command::new(CommandType::Echo, vec![]));
        assert_eq!(deserialize(a).unwrap(), b);
    }

    #[test]
    fn test_deserialize_inline_echo_message() {
        let a = "echo checo\r\n";
        let b = Request::InlineCommand(Command::new(CommandType::Echo, vec!["checo".into()]));
        assert_eq!(deserialize(a).unwrap(), b);
    }

    #[test]
    fn test_deserialize_inline_get() {
        let a = "get foo\r\n";
        let b = Request::InlineCommand(Command::new(CommandType::Get, vec!["foo".into()]));
        assert_eq!(deserialize(a).unwrap(), b);
    }
}
