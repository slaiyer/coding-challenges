use std::{error::Error, fmt};

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
pub enum Response {
    SimpleString(String),
    Error(String),
    Integer(i64),
    BulkString(String),
    Null,
    Array(Vec<String>),
}

impl fmt::Display for Response {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Response::SimpleString(ref s) => write!(f, "{}", s),
            Response::Error(ref s) => write!(f, "error: {}", s),
            Response::Integer(i) => write!(f, "{}", i),
            Response::BulkString(ref s) => write!(f, "{}", s),
            Response::Null => write!(f, "NULL"),
            Response::Array(ref v) => write!(
                f,
                "[{}]",
                v.iter()
                    .map(|s| format!("{}", s))
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
        }
    }
}

pub fn deserialize(s: &str) -> Result<Response, Box<dyn Error>> {
    let mut lines = s.lines();
    let mut line = match lines.next() {
        Some(l) => l,
        None => return Err("no lines".into()),
    };
    match line.chars().next() {
        Some('+') => Ok(Response::SimpleString(line[1..].into())),
        Some('-') => Ok(Response::Error(line[1..].into())),
        Some(':') => Ok(Response::Integer(line[1..].parse()?)),
        Some('$') => {
            let num_elem = line[1..].parse()?;
            match num_elem {
                -1 => Ok(Response::Null),
                _ => {
                    let len_str = line[1..].parse::<usize>()?;
                    line = match lines.next() {
                        Some(l) => l,
                        None => return Err("no further lines in bulk string".into()),
                    };
                    Ok(Response::BulkString(line[..len_str].into()))
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
            Ok(Response::Array(buf))
        }
        _ => Err(DeserializeError::UnexpectedFirstByte.into()),
    }
}

const TERM: &str = "\r\n";

pub fn serialize(r: Response) -> String {
    match r {
        Response::SimpleString(s) => format!("+{}{T}", s, T = TERM),
        Response::Error(s) => format!("-{}{T}", s, T = TERM),
        Response::Integer(i) => format!(":{}{T}", i, T = TERM),
        Response::BulkString(s) => format!("${}{T}{}{T}", s.len(), s, T = TERM),
        Response::Null => format!("$-1{T}", T = TERM),
        Response::Array(v) => format!(
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_deserialize_null_bulk_string() {
        let a = "$-1\r\n";
        let b = Response::Null;
        assert_eq!(deserialize(a).unwrap(), b);
        assert_eq!(serialize(b), a);
    }

    #[test]
    fn test_deserialize_array_ping() {
        let a = "*1\r\n$4\r\nping\r\n";
        let b = Response::Array(["ping".into()].into());
        assert_eq!(deserialize(a).unwrap(), b);
        assert_eq!(serialize(b), a);
    }

    #[test]
    fn test_deserialize_integer_666() {
        let a = ":666\r\n";
        let b = Response::Integer(666);
        assert_eq!(deserialize(a).unwrap(), b);
        assert_eq!(serialize(b), a);
    }

    #[test]
    fn test_deserialize_integer_minus_1000() {
        let a = ":-1000\r\n";
        let b = Response::Integer(-1000);
        assert_eq!(deserialize(a).unwrap(), b);
        assert_eq!(serialize(b), a);
    }

    #[test]
    fn test_deserialize_array_echo_hello_world() {
        let a = "*2\r\n$4\r\necho\r\n$11\r\nhello world\r\n";
        let b = Response::Array(["echo".into(), "hello world".into()].into());
        assert_eq!(deserialize(a).unwrap(), b);
        assert_eq!(serialize(b), a);
    }

    #[test]
    fn test_deserialize_array_get_key() {
        let a = "*2\r\n$3\r\nget\r\n$3\r\nkey\r\n";
        let b = Response::Array(["get".into(), "key".into()].into());
        assert_eq!(deserialize(a).unwrap(), b);
        assert_eq!(serialize(b), a);
    }

    #[test]
    fn test_deserialize_simple_string_ok() {
        let a = "+OK\r\n";
        let b = Response::SimpleString("OK".into());
        assert_eq!(deserialize(a).unwrap(), b);
        assert_eq!(serialize(b), a);
    }

    #[test]
    fn test_deserialize_error_message() {
        let a = "-Error message\r\n";
        let b = Response::Error("Error message".into());
        assert_eq!(deserialize(a).unwrap(), b);
        assert_eq!(serialize(b), a);
    }

    #[test]
    fn test_deserialize_empty_bulk_string() {
        let a = "$0\r\n\r\n";
        let b = Response::BulkString("".into());
        assert_eq!(deserialize(a).unwrap(), b);
        assert_eq!(serialize(b), a);
    }

    #[test]
    fn test_deserialize_simple_string_hello_world() {
        let a = "+hello world\r\n";
        let b = Response::SimpleString("hello world".into());
        assert_eq!(deserialize(a).unwrap(), b);
        assert_eq!(serialize(b), a);
    }
}
