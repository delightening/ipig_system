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
        &addr
            .parse()
            .unwrap_or_else(|_| "127.0.0.1:8000".parse().expect("valid fallback address")),
        Duration::from_secs(3),
    ) {
        Ok(s) => s,
        Err(_) => return ExitCode::FAILURE,
    };

    if stream
        .set_read_timeout(Some(Duration::from_secs(3)))
        .is_err()
    {
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

    // PR6 (方案 I, #238)：容器健檢只認 liveness——HTTP 200（DB up）即視為存活，不再要求
    // body 含 "healthy"。如此 metrics / disk degraded（health 回 200 + "degraded"）不會把
    // 容器標 unhealthy；只有 DB down（503）才 FAILURE。
    // bot review #631：精確解析狀態行的狀態碼欄位（"HTTP/1.1 200 OK" 的第 2 欄），
    // 避免 reason phrase / 協定版本意外含 "200" 造成誤判。
    let status_line_ok = response
        .lines()
        .next()
        .map(|line| line.split_whitespace().nth(1) == Some("200"))
        .unwrap_or(false);
    if status_line_ok {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}
