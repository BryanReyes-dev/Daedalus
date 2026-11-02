//! Newline-delimited JSON transport for the standalone runtime process.
//!
//! Each input line is one request and each output line is one response. Do not
//! write logs to stdout: stdout is the machine-readable protocol channel.

use std::io::{self, BufRead, BufReader, BufWriter, Read, Write};

use daedalus_runtime_engine::RuntimeEngine;
use daedalus_runtime_protocol::{RuntimeRequest, RuntimeResponse};

pub fn serve_stdio() -> io::Result<()> {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut engine = RuntimeEngine::new();

    serve(stdin.lock(), stdout.lock(), &mut engine)
}

fn serve<R, W>(reader: R, writer: W, engine: &mut RuntimeEngine) -> io::Result<()>
where
    R: Read,
    W: Write,
{
    let reader = BufReader::new(reader);
    let mut writer = BufWriter::new(writer);

    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }

        let response = match serde_json::from_str::<RuntimeRequest>(&line) {
            Ok(request) => engine.dispatch(request),
            Err(error) => RuntimeResponse::failure(
                "invalid-request".to_owned(),
                "INVALID_REQUEST",
                format!("Request could not be parsed: {error}"),
            ),
        };

        serde_json::to_writer(&mut writer, &response).map_err(io::Error::other)?;
        writer.write_all(b"\n")?;
        writer.flush()?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stdio_transport_handles_multiple_requests() {
        let input = concat!(
            "{\"protocolVersion\":1,\"requestId\":\"one\",\"method\":\"runtime.ping\"}\n",
            "{\"protocolVersion\":1,\"requestId\":\"two\",\"method\":\"runtime.getInfo\"}\n"
        );

        let mut output = Vec::new();
        let mut engine = RuntimeEngine::new();
        serve(input.as_bytes(), &mut output, &mut engine)
            .expect("stdio request stream should be served");

        let lines: Vec<&str> = std::str::from_utf8(&output)
            .expect("output should be UTF-8")
            .lines()
            .collect();

        assert_eq!(lines.len(), 2);

        let first: serde_json::Value =
            serde_json::from_str(lines[0]).expect("first response should be JSON");
        let second: serde_json::Value =
            serde_json::from_str(lines[1]).expect("second response should be JSON");

        assert_eq!(first["requestId"], "one");
        assert_eq!(first["result"]["alive"], true);
        assert_eq!(second["requestId"], "two");
        assert_eq!(second["result"]["name"], "Daedalus Runtime");
    }

    #[test]
    fn malformed_input_returns_a_protocol_error_without_stopping_the_stream() {
        let input = concat!(
            "{invalid json}\n",
            "{\"protocolVersion\":1,\"requestId\":\"after-error\",\"method\":\"runtime.ping\"}\n"
        );

        let mut output = Vec::new();
        let mut engine = RuntimeEngine::new();
        serve(input.as_bytes(), &mut output, &mut engine)
            .expect("malformed requests should not terminate the stream");

        let lines: Vec<&str> = std::str::from_utf8(&output)
            .expect("output should be UTF-8")
            .lines()
            .collect();

        assert_eq!(lines.len(), 2);

        let first: serde_json::Value =
            serde_json::from_str(lines[0]).expect("error response should be JSON");
        let second: serde_json::Value =
            serde_json::from_str(lines[1]).expect("valid response should be JSON");

        assert_eq!(first["ok"], false);
        assert_eq!(first["error"]["code"], "INVALID_REQUEST");
        assert_eq!(second["requestId"], "after-error");
        assert_eq!(second["ok"], true);
    }
}
