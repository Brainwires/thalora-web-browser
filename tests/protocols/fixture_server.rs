// Minimal local HTTP server for end-to-end MCP tests.
//
// Serves files from tests/fixtures/site plus a few dynamic routes, using only
// std so it works without extra dependencies:
//   GET  /<file>            static file from tests/fixtures/site
//   GET  /api/data?delay=MS JSON payload after an optional delay
//   *    /echo              HTML page echoing the method, query and body
//
// The MCP server must be started with THALORA_ALLOW_LOOPBACK=1 to reach it.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::time::Duration;

pub struct FixtureServer {
    port: u16,
}

impl FixtureServer {
    /// Start the server on an ephemeral 127.0.0.1 port. The listener thread
    /// lives until the test process exits.
    pub fn start() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind fixture server");
        let port = listener.local_addr().expect("local addr").port();
        std::thread::spawn(move || {
            for stream in listener.incoming().flatten() {
                std::thread::spawn(move || {
                    let _ = handle(stream);
                });
            }
        });
        Self { port }
    }

    /// Absolute URL for `path` (which should start with '/').
    pub fn url(&self, path: &str) -> String {
        format!("http://127.0.0.1:{}{}", self.port, path)
    }
}

fn site_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/site")
}

fn handle(mut stream: TcpStream) -> std::io::Result<()> {
    stream.set_read_timeout(Some(Duration::from_secs(10)))?;
    let mut reader = BufReader::new(stream.try_clone()?);

    let mut request_line = String::new();
    reader.read_line(&mut request_line)?;
    let mut parts = request_line.split_whitespace();
    let method = parts.next().unwrap_or("GET").to_string();
    let target = parts.next().unwrap_or("/").to_string();

    let mut content_length = 0usize;
    loop {
        let mut line = String::new();
        if reader.read_line(&mut line)? == 0 || line == "\r\n" || line == "\n" {
            break;
        }
        if let Some((name, value)) = line.split_once(':')
            && name.trim().eq_ignore_ascii_case("content-length")
        {
            content_length = value.trim().parse().unwrap_or(0);
        }
    }
    let mut body = vec![0u8; content_length];
    reader.read_exact(&mut body)?;
    let body = String::from_utf8_lossy(&body).to_string();

    let (path, query) = target.split_once('?').unwrap_or((target.as_str(), ""));

    let (status, content_type, payload) = match path {
        "/api/data" => {
            let delay = query
                .split('&')
                .find_map(|kv| kv.strip_prefix("delay="))
                .and_then(|v| v.parse::<u64>().ok())
                .unwrap_or(0);
            std::thread::sleep(Duration::from_millis(delay));
            (
                "200 OK",
                "application/json",
                r#"{"message":"fixture data loaded","items":[1,2,3]}"#.to_string(),
            )
        }
        "/echo" => (
            "200 OK",
            "text/html; charset=utf-8",
            format!(
                "<!DOCTYPE html><html><head><title>Echo</title></head><body>\
                 <h1>Echo</h1><p id=\"method\">{}</p><p id=\"query\">{}</p>\
                 <p id=\"body\">{}</p></body></html>",
                escape(&method),
                escape(query),
                escape(&body)
            ),
        ),
        _ => {
            let relative = path.trim_start_matches('/');
            let relative = if relative.is_empty() {
                "index.html"
            } else {
                relative
            };
            if relative.contains("..") {
                ("403 Forbidden", "text/plain", "forbidden".to_string())
            } else {
                match std::fs::read_to_string(site_root().join(relative)) {
                    Ok(content) => ("200 OK", content_type_for(relative), content),
                    Err(_) => ("404 Not Found", "text/plain", "not found".to_string()),
                }
            }
        }
    };

    write!(
        stream,
        "HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\n\
         Connection: close\r\n\r\n{payload}",
        payload.len()
    )?;
    stream.flush()
}

fn content_type_for(path: &str) -> &'static str {
    match path.rsplit('.').next() {
        Some("html") => "text/html; charset=utf-8",
        Some("js") => "application/javascript",
        Some("css") => "text/css",
        Some("json") => "application/json",
        _ => "application/octet-stream",
    }
}

fn escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
