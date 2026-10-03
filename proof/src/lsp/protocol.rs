use std::io::{BufRead, Write, Result};

/// Read an LSP frame from the stream (Header: Content-Length: <n>\r\n\r\n followed by <n> bytes).
pub fn read_message<R: BufRead>(reader: &mut R) -> Result<Option<String>> {
    let mut content_length: Option<usize> = None;
    let mut line = String::new();

    loop {
        line.clear();
        let bytes_read = reader.read_line(&mut line)?;
        if bytes_read == 0 {
            return Ok(None); // EOF
        }

        let trimmed = line.trim();
        if trimmed.is_empty() {
            // End of headers
            break;
        }

        if trimmed.to_lowercase().starts_with("content-length:") {
            let parts: Vec<&str> = trimmed.split(':').collect();
            if parts.len() >= 2 {
                if let Ok(len) = parts[1].trim().parse::<usize>() {
                    content_length = Some(len);
                }
            }
        }
    }

    if let Some(len) = content_length {
        let mut buffer = vec![0u8; len];
        reader.read_exact(&mut buffer)?;
        let s = String::from_utf8_lossy(&buffer).to_string();
        Ok(Some(s))
    } else {
        Ok(None)
    }
}

/// Write an LSP frame to the stream with Content-Length header.
pub fn write_message<W: Write>(writer: &mut W, body: &str) -> Result<()> {
    write!(writer, "Content-Length: {}\r\n\r\n{}", body.len(), body)?;
    writer.flush()
}

/// Send a JSON-RPC response with given ID.
pub fn send_response<W: Write>(writer: &mut W, id: &str, result_json: &str) -> Result<()> {
    let body = format!("{{\"jsonrpc\": \"2.0\", \"id\": {}, \"result\": {}}}", id, result_json);
    write_message(writer, &body)
}

/// Send a JSON-RPC notification (no ID).
pub fn send_notification<W: Write>(writer: &mut W, method: &str, params_json: &str) -> Result<()> {
    let body = format!("{{\"jsonrpc\": \"2.0\", \"method\": \"{}\", \"params\": {}}}", method, params_json);
    write_message(writer, &body)
}
