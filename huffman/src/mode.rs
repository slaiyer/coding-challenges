use std::error::Error;
use std::fmt;
use std::str::FromStr;

pub enum Mode {
    Compress,
    Decompress,
}

impl FromStr for Mode {
    type Err = ParseModeError;

    fn from_str(s: &str) -> Result<Mode, Self::Err>{
        match s {
            "c" => Ok(Mode::Compress),
            "d" => Ok(Mode::Decompress),
            _ => Err(ParseModeError),
        }
    }
}

#[derive(Debug)]
pub struct ParseModeError;

impl fmt::Display for ParseModeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "invalid mode")
    }
}

impl Error for ParseModeError{}
