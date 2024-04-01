use std::{error::Error, fmt};

#[derive(Debug, PartialEq)]
pub enum InlineCommand {
    Ping(Option<String>),
    Echo(String),
    Get(String),
}

#[derive(Debug, PartialEq)]
enum DeserializeError {
    UnexpectedFirstByte,
}

impl Error for DeserializeError {}

impl fmt::Display for DeserializeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            DeserializeError::UnexpectedFirstByte => write!(f, "unexpected first byte"),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Request {
    SimpleString(String),
    Error(String),
    Integer(i64),
    BulkString(String),
    Null,
    Array(Vec<String>),
    InlineCommand(InlineCommand),
}

impl fmt::Display for Request {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Request::SimpleString(ref s) => write!(f, "{}", s),
            Request::Error(ref s) => write!(f, "error: {}", s),
            Request::Integer(i) => write!(f, "{}", i),
            Request::BulkString(ref s) => write!(f, "{}", s),
            Request::Null => write!(f, "NULL"),
            Request::Array(ref v) => write!(
                f,
                "[{}]",
                v.iter()
                    .map(|s| format!("{}", s))
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            Request::InlineCommand(ref c) => match c {
                InlineCommand::Ping(ref s) => write!(f, "PING {}", s.clone().unwrap_or_default()),
                InlineCommand::Echo(ref s) => write!(f, "ECHO {}", s),
                InlineCommand::Get(ref s) => write!(f, "GET {}", s),
            },
        }
    }
}

pub fn deserialize(s: &str) -> Result<Request, Box<dyn Error>> {
    let mut lines = s.lines();
    let mut line = match lines.next() {
        Some(l) => l,
        None => return Err("no lines".into()),
    };
    match line.chars().next() {
        Some('+') => Ok(Request::SimpleString(line[1..].into())),
        Some('-') => Ok(Request::Error(line[1..].into())),
        Some(':') => Ok(Request::Integer(line[1..].parse()?)),
        Some('$') => {
            let num_elem = line[1..].parse()?;
            match num_elem {
                -1 => Ok(Request::Null),
                _ => {
                    let len_str = line[1..].parse::<usize>()?;
                    line = match lines.next() {
                        Some(l) => l,
                        None => return Err("no further lines in bulk string".into()),
                    };
                    Ok(Request::BulkString(line[..len_str].into()))
                }
            }
        }
        Some('*') => {
            let num_elem = line[1..].parse::<usize>()?;
            let mut buf = Vec::<String>::new();
            for _ in 0..num_elem {
                line = match lines.next() {
                    Some(l) => l,
                    None => return Err("no further lines in array for element length".into()),
                };
                let len_first_byte = &line[..1];
                if len_first_byte != "$" {
                    return Err(
                        format!("invalid first byte ({}) for array element length", {
                            len_first_byte
                        })
                        .into(),
                    );
                };
                let len = line[1..].parse::<usize>()?;
                line = match lines.next() {
                    Some(l) => l,
                    None => return Err("no further lines in array for element".into()),
                };
                buf.push(line[..len].into());
            }
            Ok(Request::Array(buf))
        }
        _ => {
            let mut parts = line.split_whitespace();
            let command = match parts.next() {
                Some(c) => c.to_uppercase(),
                None => return Err("no command".into()),
            };
            match command.as_str() {
                "PING" => Ok(Request::InlineCommand(InlineCommand::Ping(
                    match parts.next() {
                        Some(s) => Some(s.into()),
                        _ => None,
                    },
                ))),
                "ECHO" => Ok(Request::InlineCommand(InlineCommand::Echo(
                    parts.next().unwrap_or_default().into(),
                ))),
                "GET" => Ok(Request::InlineCommand(InlineCommand::Get(
                    parts.next().unwrap_or_default().into(),
                ))),
                _ => Err("invalid command".into()),
            }
        }
    }
}

const TERM: &str = "\r\n";

pub fn serialize(r: Request) -> String {
    match r {
        Request::SimpleString(s) => format!("+{}{T}", s, T = TERM),
        Request::Error(s) => format!("-{}{T}", s, T = TERM),
        Request::Integer(i) => format!(":{}{T}", i, T = TERM),
        Request::BulkString(s) => format!("${}{T}{}{T}", s.len(), s, T = TERM),
        Request::Null => format!("$-1{T}", T = TERM),
        Request::Array(v) => format!(
            "*{}{T}{}",
            v.len(),
            v.iter()
                .map(|s| format!("${}{T}{}{T}", s.len(), s, T = TERM))
                .collect::<Vec<String>>()
                .concat(),
            T = TERM,
        ),
        Request::InlineCommand(c) => match c {
            InlineCommand::Ping(s) => format!(
                "PING{}{T}",
                match s {
                    Some(s) => format!(" {}", s),
                    None => "".into(),
                },
                T = TERM,
            ),
            InlineCommand::Echo(s) => format!("ECHO {}{T}", s, T = TERM),
            InlineCommand::Get(s) => format!("GET {}{T}", s, T = TERM),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_deserialize_null_bulk_string() {
        let a = "$-1\r\n";
        let b = Request::Null;
        assert_eq!(deserialize(a).unwrap(), b);
        assert_eq!(serialize(b), a);
    }

    #[test]
    fn test_deserialize_array_ping() {
        let a = "*1\r\n$4\r\nping\r\n";
        let b = Request::Array(["ping".into()].into());
        assert_eq!(deserialize(a).unwrap(), b);
        assert_eq!(serialize(b), a);
    }

    #[test]
    fn test_deserialize_integer_666() {
        let a = ":666\r\n";
        let b = Request::Integer(666);
        assert_eq!(deserialize(a).unwrap(), b);
        assert_eq!(serialize(b), a);
    }

    #[test]
    fn test_deserialize_integer_minus_1000() {
        let a = ":-1000\r\n";
        let b = Request::Integer(-1000);
        assert_eq!(deserialize(a).unwrap(), b);
        assert_eq!(serialize(b), a);
    }

    #[test]
    fn test_deserialize_array_echo_hello_world() {
        let a = "*2\r\n$4\r\necho\r\n$11\r\nhello world\r\n";
        let b = Request::Array(["echo".into(), "hello world".into()].into());
        assert_eq!(deserialize(a).unwrap(), b);
        assert_eq!(serialize(b), a);
    }

    #[test]
    fn test_deserialize_array_get_key() {
        let a = "*2\r\n$3\r\nget\r\n$3\r\nkey\r\n";
        let b = Request::Array(["get".into(), "key".into()].into());
        assert_eq!(deserialize(a).unwrap(), b);
        assert_eq!(serialize(b), a);
    }

    #[test]
    fn test_deserialize_simple_string_ok() {
        let a = "+OK\r\n";
        let b = Request::SimpleString("OK".into());
        assert_eq!(deserialize(a).unwrap(), b);
        assert_eq!(serialize(b), a);
    }

    #[test]
    fn test_deserialize_error_message() {
        let a = "-Error message\r\n";
        let b = Request::Error("Error message".into());
        assert_eq!(deserialize(a).unwrap(), b);
        assert_eq!(serialize(b), a);
    }

    #[test]
    fn test_deserialize_empty_bulk_string() {
        let a = "$0\r\n\r\n";
        let b = Request::BulkString("".into());
        assert_eq!(deserialize(a).unwrap(), b);
        assert_eq!(serialize(b), a);
    }

    #[test]
    fn test_deserialize_simple_string_hello_world() {
        let a = "+hello world\r\n";
        let b = Request::SimpleString("hello world".into());
        assert_eq!(deserialize(a).unwrap(), b);
        assert_eq!(serialize(b), a);
    }

    #[test]
    fn test_deserialize_inline_ping() {
        let a = "ping\r\n";
        let b = Request::InlineCommand(InlineCommand::Ping(None));
        assert_eq!(deserialize(a).unwrap(), b);
    }

    #[test]
    fn test_deserialize_inline_ping_message() {
        let a = "ping ling\r\n";
        let b = Request::InlineCommand(InlineCommand::Ping(Some("ling".into())));
        assert_eq!(deserialize(a).unwrap(), b);
    }

    #[test]
    fn test_deserialize_inline_echo() {
        let a = "echo\r\n";
        let b = Request::InlineCommand(InlineCommand::Echo("".into()));
        assert_eq!(deserialize(a).unwrap(), b);
    }

    #[test]
    fn test_deserialize_inline_echo_message() {
        let a = "echo checo\r\n";
        let b = Request::InlineCommand(InlineCommand::Echo("checo".into()));
        assert_eq!(deserialize(a).unwrap(), b);
    }

    #[test]
    fn test_deserialize_inline_get() {
        let a = "get foo\r\n";
        let b = Request::InlineCommand(InlineCommand::Get("foo".into()));
        assert_eq!(deserialize(a).unwrap(), b);
    }
}
