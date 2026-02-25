//! Minimal health check binary for Docker HEALTHCHECK.
//!
//! Makes an HTTP GET to localhost:8000/api/health using only std::net.
//! Exits 0 if response contains "healthy", 1 otherwise.
//! Designed for distroless containers (no shell, no curl/wget).

use std::io::{Read, Write};
use std::net::TcpStream;
use std::process::ExitCode;
use std::time::Duration;

fn main() -> ExitCode {
    let port = std::env::var("PORT").unwrap_or_else(|_| "8000".to_string());
    let addr = format!("127.0.0.1:{}", port);

    let stream = match TcpStream::connect_timeout(
        &addr.parse().unwrap_or_else(|_| "127.0.0.1:8000".parse().expect("valid fallback address")),
        Duration::from_secs(3),
    ) {
        Ok(s) => s,
        Err(_) => return ExitCode::FAILURE,
    };

    if stream.set_read_timeout(Some(Duration::from_secs(3))).is_err() {
        return ExitCode::FAILURE;
    }

    let mut stream = stream;
    let request = format!(
        "GET /api/health HTTP/1.0\r\nHost: localhost:{}\r\nConnection: close\r\n\r\n",
        port
    );

    if stream.write_all(request.as_bytes()).is_err() {
        return ExitCode::FAILURE;
    }

    let mut response = String::new();
    if stream.read_to_string(&mut response).is_err() {
        return ExitCode::FAILURE;
    }

    if response.contains("200") && response.contains("healthy") {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}
