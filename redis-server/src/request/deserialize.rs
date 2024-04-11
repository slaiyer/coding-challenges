use crate::response::types::Response;

pub fn stringify(request_buf: &[u8]) -> Result<String, Response> {
    let request_str: String = match std::str::from_utf8(request_buf) {
        Ok(s) => s.into(),
        Err(e) => return Err(Response::err_from_error(e)),
    };

    Ok(request_str.trim_matches('\0').into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stringify_inline_cmd_arg() {
        let request_buf = b"\0\0ping ling\r\n\0\0";
        let result = stringify(request_buf);
        assert_eq!(result, Ok("ping ling\r\n".into()));
    }

    #[test]
    fn test_stringify_bulk_cmd_arg() {
        let request_buf = b"\0\0*2\r\n$4ping\r\n$4ling\r\n\0\0";
        let result = stringify(request_buf);
        assert_eq!(result, Ok("*2\r\n$4ping\r\n$4ling\r\n".into()));
    }

    #[test]
    fn test_stringify_invalid_utf8() {
        let request_buf = b"\0\0ping\xFF ling\r\n\0\0";
        let result = stringify(request_buf);
        assert_eq!(result, Err(Response::err("", "invalid utf-8 sequence of 1 bytes from index 6")));
    }

    #[test]
    fn test_stringify_empty_request() {
        let request_buf = b"\0\0\0\0";
        let result = stringify(request_buf);
        assert_eq!(result, Ok(String::new()));
    }

    #[test]
    fn test_stringify_empty_command() {
        let request_buf = b"\0\0\r\n\0\0";
        let result = stringify(request_buf);
        assert_eq!(result, Ok("\r\n".into()));
    }

    #[test]
    fn test_stringify_empty_arg() {
        let request_buf = b"\0\0ping \r\n\0\0";
        let result = stringify(request_buf);
        assert_eq!(result, Ok("ping \r\n".into()));
    }
}
