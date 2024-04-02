use std::fmt;

use crate::command::{Command, InvalidCommandError};

pub const TERM: &str = "\r\n";

#[derive(Debug, PartialEq, Eq)]
// #[cfg_attr(test, derive(Debug))]
pub enum Request {
    Array(Vec<Command>),
    InlineCommand(Command),
}

impl fmt::Display for Response {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Self::SimpleString(ref s) => write!(f, "+{s}{TERM}"),
            Self::Error(ref s) => s.fmt(f),
            Self::Integer(i) => write!(f, ":{i}{TERM}"),
            Self::BulkString(ref s) => write!(f, "${len}{TERM}{s}{TERM}", len = s.len()),
            Self::Null => write!(f, "$-1{TERM}"),
            Self::Array(ref v) => write!(
                f,
                "*{}{TERM}{}",
                v.len(),
                v.iter()
                    .map(|s| format!("${len}{TERM}{s}{TERM}", len = s.len()))
                    .collect::<Vec<String>>()
                    .concat(),
            ),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Error {
    kind: String,
    message: String,
}

impl Error {
    pub fn new(kind: &str, message: &str) -> Self {
        Self {
            kind: match kind {
                "" => "ERR",
                _ => kind,
            }
            .into(),
            message: message.into(),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let kind = match self.kind {
            ref s if s.is_empty() => "ERR ".to_string(),
            ref s => format!("{s} "),
        };
        write!(f, "-{}{}{T}", kind, self.message, T = TERM)
    }
}

#[derive(PartialEq, Eq)]
pub enum Response {
    SimpleString(String),
    Error(Error),
    Integer(i64),
    BulkString(String),
    Null,
    Array(Vec<String>),
}

pub fn deserialize(s: &str) -> Result<Request, InvalidCommandError> {
    if s.is_empty() {
        return Err(InvalidCommandError::Command);
    }

    if s.starts_with('*') {
        parse_bulk_string(s)
    } else {
        Command::new_from_str(s).map(Request::InlineCommand)
    }
}

// TODO: reimplement this function referring https://redis.io/docs/reference/protocol-spec/#high-performance-parser-for-the-redis-protocol
fn parse_bulk_string(s: &str) -> Result<Request, InvalidCommandError> {
    let mut commands = Vec::<Command>::new();

    let command_strs = s.split('*').skip(1).collect::<Vec<&str>>();
    for cmd_str in command_strs {
        let cmd_frags = cmd_str.split('$').collect::<Vec<&str>>();

        let num_cmd_frags = match cmd_frags[0].strip_suffix(TERM) {
            Some(s) => match s.parse::<usize>() {
                Ok(n) if n > 0 => n,
                _ => return Err(InvalidCommandError::BulkStringLength),
            },
            None => return Err(InvalidCommandError::Command),
        };

        let mut cmd_frag_vec = Vec::with_capacity(num_cmd_frags);
        for cmd_frag in cmd_frags.iter().skip(1).take(num_cmd_frags) {
            let Some(cmd_frag) = cmd_frag.strip_suffix(TERM) else {
                return Err(InvalidCommandError::Command);
            };

            let cmd_frag_split = cmd_frag.split(TERM).collect::<Vec<&str>>();
            if cmd_frag_split.len() != 2 {
                return Err(InvalidCommandError::Command);
            }

            let cmd_frag_len = match cmd_frag_split[0].parse::<usize>() {
                Ok(n) if n > 0 => n,
                _ => return Err(InvalidCommandError::CommandLength),
            };

            let Some(cmd) = cmd_frag_split[1].get(..cmd_frag_len) else {
                return Err(InvalidCommandError::Command);
            };

            cmd_frag_vec.push(cmd.to_string());
        }

        let command = Command::new_from_str(&cmd_frag_vec.join(" "))?;
        commands.push(command);
    }

    Ok(Request::Array(commands))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command::CmdType;

    #[test]
    fn test_deserialize_null_bulk_string() {
        let a = Response::Null;
        let b = "$-1\r\n";
        assert_eq!(a.to_string(), b);
    }

    #[test]
    fn test_deserialize_array_ping() {
        let a = Response::Array(["ping".into()].into());
        let b = "*1\r\n$4\r\nping\r\n";
        assert_eq!(a.to_string(), b);
    }

    #[test]
    fn test_deserialize_integer_666() {
        let a = Response::Integer(666);
        let b = ":666\r\n";
        assert_eq!(a.to_string(), b);
    }

    #[test]
    fn test_deserialize_integer_minus_1000() {
        let a = Response::Integer(-1000);
        let b = ":-1000\r\n";
        assert_eq!(a.to_string(), b);
    }

    #[test]
    fn test_deserialize_array_echo_hello_world() {
        let a = Response::Array(["echo".into(), "hello world".into()].into());
        let b = "*2\r\n$4\r\necho\r\n$11\r\nhello world\r\n";
        assert_eq!(a.to_string(), b);
    }

    #[test]
    fn test_deserialize_array_get_key() {
        let a = Response::Array(["get".into(), "key".into()].into());
        let b = "*2\r\n$3\r\nget\r\n$3\r\nkey\r\n";
        assert_eq!(a.to_string(), b);
    }

    #[test]
    fn test_deserialize_simple_string_ok() {
        let a = Response::SimpleString("OK".into());
        let b = "+OK\r\n";
        assert_eq!(a.to_string(), b);
    }

    #[test]
    fn test_deserialize_error_message() {
        let a = Response::Error(Error::new("", "Error message"));
        let b = "-ERR Error message\r\n";
        assert_eq!(a.to_string(), b);
    }

    #[test]
    fn test_deserialize_empty_bulk_string() {
        let a = Response::BulkString(String::new());
        let b = "$0\r\n\r\n";
        assert_eq!(a.to_string(), b);
    }

    #[test]
    fn test_deserialize_simple_string_hello_world() {
        let a = Response::SimpleString("hello world".into());
        let b = "+hello world\r\n";
        assert_eq!(a.to_string(), b);
    }

    #[test]
    fn test_deserialize_inline_ping() {
        let a = "ping\r\n";
        let b = Request::InlineCommand(Command::new(CmdType::Ping, vec![]));
        assert_eq!(deserialize(a).unwrap(), b);
    }

    #[test]
    fn test_deserialize_inline_ping_message() {
        let a = "ping ling\r\n";
        let b = Request::InlineCommand(Command::new(CmdType::Ping, vec!["ling".into()]));
        assert_eq!(deserialize(a).unwrap(), b);
    }

    #[test]
    fn test_deserialize_inline_echo() {
        let a = "echo\r\n";
        let b = Request::InlineCommand(Command::new(CmdType::Echo, vec![]));
        assert_eq!(deserialize(a).unwrap(), b);
    }

    #[test]
    fn test_deserialize_inline_echo_message() {
        let a = "echo checo\r\n";
        let b = Request::InlineCommand(Command::new(CmdType::Echo, vec!["checo".into()]));
        assert_eq!(deserialize(a).unwrap(), b);
    }

    #[test]
    fn test_deserialize_inline_get() {
        let a = "get foo\r\n";
        let b = Request::InlineCommand(Command::new(CmdType::Get, vec!["foo".into()]));
        assert_eq!(deserialize(a).unwrap(), b);
    }
}
