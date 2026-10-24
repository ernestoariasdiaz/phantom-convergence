//! Tiny HTTP/1.1 client for a local llama-server. Std only, plain http.

use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

pub struct Endpoint {
    host: String,
    port: u16,
    base_path: String,
}

impl Endpoint {
    pub fn parse(url: &str) -> Result<Self, String> {
        let rest = url
            .strip_prefix("http://")
            .ok_or("server URL must start with http:// (llama-server is local, no TLS)")?;
        let (hostport, path) = match rest.find('/') {
            Some(i) => (&rest[..i], rest[i..].trim_end_matches('/').to_string()),
            None => (rest, String::new()),
        };
        let (host, port) = match hostport.rsplit_once(':') {
            Some((h, p)) => (h.to_string(), p.parse().map_err(|_| "bad port")?),
            None => (hostport.to_string(), 80),
        };
        Ok(Endpoint { host, port, base_path: path })
    }

    pub fn post_json(&self, path: &str, body: &str, timeout: Duration) -> Result<String, String> {
        let full = format!("{}{}", self.base_path, path);
        let mut stream = TcpStream::connect((self.host.as_str(), self.port))
            .map_err(|e| format!("cannot connect to {}:{} — is llama-server running? ({e})", self.host, self.port))?;
        stream.set_read_timeout(Some(timeout)).ok();
        stream.set_write_timeout(Some(timeout)).ok();
        let req = format!(
            "POST {full} HTTP/1.1\r\nHost: {}:{}\r\nContent-Type: application/json\r\nAccept: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            self.host,
            self.port,
            body.len()
        );
        stream.write_all(req.as_bytes()).map_err(|e| e.to_string())?;
        stream.write_all(body.as_bytes()).map_err(|e| e.to_string())?;
        let mut raw = Vec::new();
        stream.read_to_end(&mut raw).map_err(|e| format!("read failed: {e}"))?;

        let split = find(&raw, b"\r\n\r\n").ok_or("malformed HTTP response")?;
        let head = String::from_utf8_lossy(&raw[..split]).to_string();
        let mut body = raw[split + 4..].to_vec();
        let status: u16 = head
            .lines()
            .next()
            .and_then(|l| l.split_whitespace().nth(1))
            .and_then(|c| c.parse().ok())
            .unwrap_or(0);
        if head.to_ascii_lowercase().contains("transfer-encoding: chunked") {
            body = dechunk(&body);
        }
        let text = String::from_utf8_lossy(&body).to_string();
        if !(200..300).contains(&status) {
            return Err(format!("HTTP {status}: {}", text.chars().take(300).collect::<String>()));
        }
        Ok(text)
    }
}

fn find(h: &[u8], n: &[u8]) -> Option<usize> {
    h.windows(n.len()).position(|w| w == n)
}

fn dechunk(mut b: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    loop {
        let Some(eol) = find(b, b"\r\n") else { break };
        let size_str = String::from_utf8_lossy(&b[..eol]);
        let size = usize::from_str_radix(size_str.split(';').next().unwrap_or("0").trim(), 16).unwrap_or(0);
        b = &b[eol + 2..];
        if size == 0 || b.len() < size {
            break;
        }
        out.extend_from_slice(&b[..size]);
        b = &b[(size + 2).min(b.len())..];
    }
    out
}
