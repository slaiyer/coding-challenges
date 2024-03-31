use std::{error::Error, fmt};

fn main() {}

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
enum Response {
    SimpleString(String),
    Error(String),
    Integer(i64),
    BulkString(String),
    Null,
    Array(Vec<String>),
}

fn deserialize(s: &str) -> Result<Response, Box<dyn Error>> {
    let mut lines = s.lines();
    let mut line = match lines.next() {
        Some(l) => l,
        None => return Err("no lines".into()),
    };
    match line.chars().next() {
        Some('+') => Ok(Response::SimpleString(line[1..].to_string())),
        Some('-') => Ok(Response::Error(line[1..].to_string())),
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
                    Ok(Response::BulkString(line[..len_str].to_string()))
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
                    return Err(format!("invalid first byte ({}) for array element length", {len_first_byte}).into());
                };
                let len = line[1..].parse::<usize>()?;
                line = match lines.next() {
                    Some(l) => l,
                    None => return Err("no further lines in array for element".into()),
                };
                buf.push(line[..len].to_string());
            }
            Ok(Response::Array(buf))
        }
        _ => Err(DeserializeError::UnexpectedFirstByte.into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_deserialize_null_bulk_string() {
        let s = "$-1\r\n";
        assert_eq!(deserialize(s).unwrap(), Response::Null,);
    }

    #[test]
    fn test_deserialize_array_ping() {
        let s = "*1\r\n$4\r\nping\r\n";
        assert_eq!(
            deserialize(s).unwrap(),
            Response::Array(["ping".to_string(),].to_vec()),
        );
    }

    #[test]
    fn test_deserialize_integer_666() {
        let s = ":666\r\n";
        assert_eq!(deserialize(s).unwrap(), Response::Integer(666),);
    }

    #[test]
    fn test_deserialize_integer_minus_1000() {
        let s = ":-1000\r\n";
        assert_eq!(deserialize(s).unwrap(), Response::Integer(-1000),);
    }

    #[test]
    fn test_deserialize_array_echo_hello_world() {
        let s = "*2\r\n$4\r\necho\r\n$11\r\nhello world\r\n";
        assert_eq!(
            deserialize(s).unwrap(),
            Response::Array(["echo".to_string(), "hello world".to_string(),].to_vec()),
        );
    }

    #[test]
    fn test_deserialize_array_get_key() {
        let s = "*2\r\n$3\r\nget\r\n$3\r\nkey\r\n";
        assert_eq!(
            deserialize(s).unwrap(),
            Response::Array(["get".to_string(), "key".to_string(),].to_vec()),
        );
    }

    #[test]
    fn test_deserialize_simple_string_ok() {
        let s = "+OK\r\n";
        assert_eq!(
            deserialize(s).unwrap(),
            Response::SimpleString("OK".to_string()),
        );
    }

    #[test]
    fn test_deserialize_error_message() {
        let s = "-Error message\r\n";
        assert_eq!(
            deserialize(s).unwrap(),
            Response::Error("Error message".to_string()),
        );
    }

    #[test]
    fn test_deserialize_empty_bulk_string() {
        let s = "$0\r\n\r\n";
        assert_eq!(
            deserialize(s).unwrap(),
            Response::BulkString("".to_string()),
        );
    }

    #[test]
    fn test_deserialize_simple_string_hello_world() {
        let s = "+hello world\r\n";
        assert_eq!(
            deserialize(s).unwrap(),
            Response::SimpleString("hello world".to_string()),
        );
    }
}
