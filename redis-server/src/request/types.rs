use std::{error::Error, fmt, str::FromStr};

#[derive(Debug, PartialEq, Eq)]
pub struct Request {
    commands: Vec<Vec<String>>,
}

impl Request {
    pub fn new(commands: Vec<Vec<String>>) -> Self {
        Self { commands }
    }
}

#[derive(Debug)]
pub enum ParseError {
    InvalidRequest,
    InvalidBulkLength,
    InvalidTokenLength,
    EmptyCommand,
    EmptyRequest,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidRequest => write!(f, "invalid request"),
            Self::InvalidBulkLength => write!(f, "invalid bulk length"),
            Self::InvalidTokenLength => write!(f, "invalid token length"),
            Self::EmptyCommand => write!(f, "empty command"),
            Self::EmptyRequest => write!(f, "empty request"),
        }
    }
}

impl Error for ParseError {}

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
    let mut num_tokens: usize;
    let mut i = 0;
    while i < chars.len() {
        match chars.get(i) {
            Some('*') => {
                // parse bulk request length
                let mut j = i + 1;
                while chars[j] != '\r' {
                    j += 1;
                }
                num_tokens = s[i + 1..j]
                    .parse::<usize>()
                    .map_err(|_| ParseError::InvalidBulkLength)?;
                i = j + 2;

                // parse bulk request arguments
                let mut cmd: Vec<String> = Vec::new();
                for _ in 0..num_tokens {
                    // parse token length
                    if chars[i] != '$' {
                        return Err(ParseError::InvalidTokenLength);
                    }
                    i += 1;

                    let mut j = i;
                    while chars[j] != '\r' {
                        j += 1;
                    }
                    let token_len = s[i..j]
                        .parse::<usize>()
                        .map_err(|_| ParseError::InvalidTokenLength)?;
                    i = j + 2;

                    // parse token
                    let token = s[i..i + token_len].into();
                    cmd.push(token);
                    i += token_len + 2;
                }

                commands.push(cmd);
            }
            _ => return Err(ParseError::InvalidRequest),
        }
    }

    Ok(commands)
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
            Request::new(vec![vec!["ping".into(), "ling".into(),],],)
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
}
