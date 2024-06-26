/// This module defines the types related to handling Redis requests.
///
/// The `Request` struct represents a Redis request, which consists of a list of commands.
/// Each command is a list of strings.
///
/// The `ParseError` enum represents the possible errors  during parsing of a Redis request.
/// These errors include invalid request format, invalid bulk length, invalid token length,
/// empty command, and empty request.
///
/// The `Request` struct and `ParseError` enum are implemented with various methods and traits
/// to enable parsing of Redis requests from strings and conversion to and from other types.
///
/// The module also includes unit tests to verify the correctness of the parsing logic.
use std::{
    error::Error,
    fmt,
    str::{FromStr, Utf8Error},
};

#[derive(Debug, PartialEq, Eq)]
pub struct Request {
    commands: Vec<Vec<String>>,
}

impl Request {
    pub fn new(commands: Vec<Vec<String>>) -> Self {
        Self { commands }
    }

    pub const fn commands(&self) -> &Vec<Vec<String>> {
        &self.commands
    }
}

impl TryFrom<&[u8]> for Request {
    type Error = ParseError;

    fn try_from(request_buf: &[u8]) -> Result<Self, Self::Error> {
        let request_str = match std::str::from_utf8(request_buf) {
            Ok(result) => result.trim_matches('\0'),
            Err(error) => return Err(Self::Error::Utf8(error)),
        };

        Ok(Self::new(if request_str.starts_with('*') {
            parse_bulk_requests(request_str)?
        } else {
            parse_inline_request(request_str)?
        }))
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ParseError {
    Utf8(Utf8Error),
    InvalidRequest,
    InvalidBulkLength,
    InvalidTokenLength,
    EmptyCommand,
    EmptyRequest,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Utf8(e) => write!(f, "invalid UTF-8: {e}"),
            Self::InvalidRequest => write!(f, "invalid request"),
            Self::InvalidBulkLength => write!(f, "invalid bulk length"),
            Self::InvalidTokenLength => write!(f, "invalid token length"),
            Self::EmptyCommand => write!(f, "empty command"),
            Self::EmptyRequest => write!(f, "empty request"),
        }
    }
}

impl Error for ParseError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Utf8(e) => Some(e),
            _ => None,
        }
    }
}

// Implement the conversion from `Utf8Error` to `ParseError`.
// This will be automatically called by `?` if a `Utf8Error`
// needs to be converted into a `ParseError`.
impl From<Utf8Error> for ParseError {
    fn from(err: Utf8Error) -> Self {
        Self::Utf8(err)
    }
}

impl FromStr for Request {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::new(if s.starts_with('*') {
            parse_bulk_requests(s)?
        } else {
            parse_inline_request(s)?
        }))
    }
}

fn parse_bulk_requests(s: &str) -> Result<Vec<Vec<String>>, ParseError> {
    let mut commands: Vec<Vec<String>> = Vec::new();

    let chars: Vec<_> = s.chars().collect();
    let mut num_tokens = usize::default();
    let mut i = 0;
    while i < chars.len() {
        match chars.get(i) {
            Some('*') => {
                parse_bulk_request_length(s, &chars, &mut i, &mut num_tokens)?;
                let cmd = parse_bulk_request_args(s, &chars, &mut i, num_tokens)?;

                commands.push(cmd);
            }
            _ => return Err(ParseError::InvalidRequest),
        }
    }

    Ok(commands)
}

fn parse_bulk_request_length(
    s: &str,
    chars: &[char],
    i: &mut usize,
    num_tokens: &mut usize,
) -> Result<(), ParseError> {
    let mut j = *i + 1;
    while j < s.len() && chars[j] != '\r' {
        j += 1;
    }

    *num_tokens = s[*i + 1..j]
        .parse::<usize>()
        .map_err(|_| ParseError::InvalidBulkLength)?;
    *i = j + 2;

    if *num_tokens < 1 {
        return Err(ParseError::EmptyCommand);
    }

    Ok(())
}

fn parse_bulk_request_args(
    s: &str,
    chars: &[char],
    i: &mut usize,
    num_tokens: usize,
) -> Result<Vec<String>, ParseError> {
    let mut cmd: Vec<String> = Vec::new();
    for _ in 0..num_tokens {
        // parse token length
        if chars.get(*i) != Some(&'$') {
            return Err(ParseError::InvalidTokenLength);
        }
        *i += 1;

        let mut j = *i;
        while j < s.len() && chars[j] != '\r' {
            j += 1;
        }

        let token_len = s[*i..j]
            .parse::<usize>()
            .map_err(|_| ParseError::InvalidTokenLength)?;

        if *i + token_len >= s.len() {
            return Err(ParseError::InvalidTokenLength);
        }
        *i = j + 2;

        // parse token
        let token = s[*i..*i + token_len].into();
        *i += token_len + 2;

        cmd.push(token);
    }

    Ok(cmd)
}

fn parse_inline_request(s: &str) -> Result<Vec<Vec<String>>, ParseError> {
    let mut commands = Vec::new();

    for line in s.lines() {
        let mut args = Vec::new();
        for arg in line.split_whitespace() {
            args.push(arg.into());
        }

        if args.is_empty() {
            return Err(ParseError::EmptyCommand);
        }

        commands.push(args);
    }

    if commands.is_empty() {
        return Err(ParseError::EmptyRequest);
    }

    Ok(commands)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_inline_cmd_arg() {
        let request_str = "ping\r\necho ling\r\n";
        let result = request_str.parse::<Request>().unwrap();
        assert_eq!(
            result,
            Request::new(vec![
                vec!["ping".into()],
                vec!["echo".into(), "ling".into()]
            ])
        );
    }

    #[test]
    fn test_parse_bulk_single_cmd() {
        let request_str = "*2\r\n$4\r\nping\r\n$4\r\nling\r\n";
        let result = request_str.parse::<Request>().unwrap();
        assert_eq!(
            result,
            Request::new(vec![vec!["ping".into(), "ling".into()]])
        );
    }

    #[test]
    fn test_parse_bulk_multiple_cmd() {
        let request_str =
            "*2\r\n$4\r\nping\r\n$4\r\nling\r\n*3\r\n$6\r\nconfig\r\n$3\r\nget\r\n$4\r\nsave\r\n";
        let result = request_str.parse::<Request>().unwrap();
        assert_eq!(
            result,
            Request::new(vec![
                vec!["ping".into(), "ling".into(),],
                vec!["config".into(), "get".into(), "save".into(),],
            ],),
        );
    }

    #[test]
    fn test_parse_invalid_bulk_length() {
        let request_str = "*2\r\n$4\r\nping\r\n$4\r\nling\r\n*3\r\n$6\r\nconfig\r\n$3\r\nget\r\n";
        let result = request_str.parse::<Request>();
        assert_eq!(result, Err(ParseError::InvalidTokenLength));
    }

    #[test]
    fn test_parse_invalid_token_length() {
        let request_str =
            "*2\r\n$4\r\nping\r\n$4\r\nling\r\n*3\r\n$6\r\nconfig\r\n$3\r\nget\r\n$4\r\n";
        let result = request_str.parse::<Request>();
        assert_eq!(result, Err(ParseError::InvalidTokenLength));
    }

    #[test]
    fn test_parse_empty_command() {
        let request_str = "ping\r\n\r\n";
        let result = request_str.parse::<Request>();
        assert_eq!(result, Err(ParseError::EmptyCommand));
    }

    #[test]
    fn test_parse_empty_request() {
        let request_str = "";
        let result = request_str.parse::<Request>();
        assert_eq!(result, Err(ParseError::EmptyRequest));
    }

    #[test]
    fn test_parse_invalid_request2() {
        let request_str = "*2\r\n$4\r\nping\r\n$4\r\nling\r\n$3\r\nget\r\n";
        let result = request_str.parse::<Request>();
        assert_eq!(result, Err(ParseError::InvalidRequest));
    }

    #[test]
    fn test_parse_invalid_request3() {
        let request_str = "*2\r\n$4\r\nping\r\n$4\r\nling\r\n$3\r\nget\r\n$4\r\nsave\r\n";
        let result = request_str.parse::<Request>();
        assert_eq!(result, Err(ParseError::InvalidRequest));
    }

    #[test]
    fn test_parse_invalid_request4() {
        let request_str = "*2\r\n$4\r\nping\r\n$4\r\nling\r\n$3\r\nget\r\n$4\r\nsave\r\n$4\r\n";
        let result = request_str.parse::<Request>();
        assert_eq!(result, Err(ParseError::InvalidRequest));
    }
}
