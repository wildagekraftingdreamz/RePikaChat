//! Optional bridge to a *local* language model.
//!
//! RePikaChat answers on its own from `bot.rs`. If the consumer opts in and a
//! local server is running, the same extracted facts are handed to that model so
//! the words improve while memory, mood and validation stay ours.
//!
//! Two wire formats are supported, both over plain HTTP on a loopback host:
//!
//!   * `ollama`  — POST {base}/api/chat, probe GET {base}/api/tags
//!   * `openai`  — POST {base}/v1/chat/completions, probe GET {base}/v1/models
//!                 (llama.cpp server, LM Studio, vLLM, LocalAI, text-generation-webui)
//!
//! Endpoints are validated as loopback before any request is made, so no chat
//! text can be sent to another machine by accident. Any failure — daemon down,
//! model missing, HTTP error, timeout, empty output — returns an error and the
//! caller keeps the deterministic reply.

use std::net::IpAddr;
use std::time::Duration;

use serde::Serialize;
use serde_json::{json, Value};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::model::ModelSettings;

#[derive(Serialize, Clone)]
pub struct BrainView {
    pub active: bool,
    pub provider: String,
    pub model: String,
    pub endpoint: String,
}

pub fn brain_view(s: &ModelSettings) -> BrainView {
    BrainView {
        active: s.provider != "off" && !s.model.trim().is_empty(),
        provider: s.provider.clone(),
        model: s.model.clone(),
        endpoint: s.endpoint.clone(),
    }
}

pub struct Target {
    pub host: String,
    pub port: u16,
    /// Base path without a trailing slash, e.g. "" or "/v1".
    pub path: String,
}

/// Parses and validates a local endpoint. Only loopback hosts and plain http
/// are accepted, because there is no TLS stack in this binary.
pub fn parse_endpoint(raw: &str) -> Result<Target, String> {
    let text = raw.trim();
    if text.is_empty() {
        return Err("Give an endpoint such as http://127.0.0.1:11434".to_string());
    }
    let rest = if let Some(r) = text.strip_prefix("http://") {
        r
    } else if text.starts_with("https://") {
        return Err(
            "Only plain http:// endpoints work here (no TLS client). Use http://127.0.0.1:PORT."
                .to_string(),
        );
    } else {
        text
    };

    let (authority, path) = match rest.find('/') {
        Some(i) => (&rest[..i], &rest[i..]),
        None => (rest, ""),
    };
    if authority.trim().is_empty() {
        return Err("The endpoint is missing a host.".to_string());
    }

    let (host, port) = if let Some(stripped) = authority.strip_prefix('[') {
        // [::1]:11434
        let end = stripped
            .find(']')
            .ok_or_else(|| "Malformed IPv6 endpoint.".to_string())?;
        let host = stripped[..end].to_string();
        let after = &stripped[end + 1..];
        let port = after
            .strip_prefix(':')
            .and_then(|p| p.parse::<u16>().ok())
            .unwrap_or(11434);
        (host, port)
    } else if let Some((h, p)) = authority.rsplit_once(':') {
        let port = p
            .parse::<u16>()
            .map_err(|_| format!("`{p}` is not a port."))?;
        (h.to_string(), port)
    } else {
        (authority.to_string(), 11434)
    };

    if host.trim().is_empty() {
        return Err("The endpoint is missing a host.".to_string());
    }
    let loopback = host.eq_ignore_ascii_case("localhost")
        || host
            .parse::<IpAddr>()
            .map(|ip| ip.is_loopback())
            .unwrap_or(false);
    if !loopback {
        return Err(format!(
            "`{host}` is not a loopback address. RePikaChat only talks to a model on this machine (127.0.0.1, ::1 or localhost)."
        ));
    }

    Ok(Target {
        host,
        port,
        path: path.trim_end_matches('/').to_string(),
    })
}

pub struct Client {
    pub provider: String,
    target: Target,
    model: String,
    timeout: Duration,
    temperature: f32,
}

impl Client {
    pub fn from_settings(s: &ModelSettings) -> Result<Client, String> {
        if !matches!(s.provider.as_str(), "ollama" | "openai") {
            return Err("No local model is enabled.".to_string());
        }
        if s.model.trim().is_empty() {
            return Err("Set a model name first.".to_string());
        }
        Ok(Client {
            provider: s.provider.clone(),
            target: parse_endpoint(&s.endpoint)?,
            model: s.model.trim().to_string(),
            timeout: Duration::from_millis(s.timeout_ms.clamp(1_000, 120_000)),
            temperature: if s.temperature.is_finite() {
                s.temperature.clamp(0.0, 1.5)
            } else {
                0.85
            },
        })
    }

    fn chat_path(&self) -> String {
        if self.provider == "openai" {
            if self.target.path.ends_with("/v1") {
                format!("{}/chat/completions", self.target.path)
            } else {
                format!("{}/v1/chat/completions", self.target.path)
            }
        } else {
            format!("{}/api/chat", self.target.path)
        }
    }

    fn models_path(&self) -> String {
        if self.provider == "openai" {
            if self.target.path.ends_with("/v1") {
                format!("{}/models", self.target.path)
            } else {
                format!("{}/v1/models", self.target.path)
            }
        } else {
            format!("{}/api/tags", self.target.path)
        }
    }

    /// Ask for one in-character reply. `turns` must end with the consumer's
    /// newest message.
    pub async fn reply(&self, system: &str, turns: &[(String, String)]) -> Result<String, String> {
        let mut messages: Vec<Value> = Vec::with_capacity(turns.len() + 1);
        messages.push(json!({ "role": "system", "content": system }));
        for (role, text) in turns {
            let role = if role == "user" { "user" } else { "assistant" };
            messages.push(json!({ "role": role, "content": text }));
        }

        let payload = if self.provider == "openai" {
            json!({
                "model": self.model,
                "messages": messages,
                "stream": false,
                "temperature": self.temperature,
                "max_tokens": 200
            })
        } else {
            json!({
                "model": self.model,
                "messages": messages,
                "stream": false,
                "options": {
                    "temperature": self.temperature,
                    "top_p": 0.9,
                    "repeat_penalty": 1.12,
                    "num_predict": 160
                }
            })
        };

        let body = payload.to_string();
        let raw = tokio::time::timeout(
            self.timeout,
            self.request("POST", &self.chat_path(), Some(&body)),
        )
        .await
        .map_err(|_| format!("no answer within {} ms", self.timeout.as_millis()))??;

        parse_chat(&self.provider, &raw)
    }

    /// Lists the models the local server reports, so the UI can offer a choice.
    pub async fn probe(&self) -> Result<Vec<String>, String> {
        let raw = tokio::time::timeout(
            self.timeout,
            self.request("GET", &self.models_path(), None),
        )
        .await
        .map_err(|_| format!("no answer within {} ms", self.timeout.as_millis()))??;

        let value: Value =
            serde_json::from_str(&raw).map_err(|e| format!("unexpected response ({e})"))?;
        if let Some(err) = value.get("error").and_then(|e| e.as_str()) {
            return Err(err.to_string());
        }
        let list = value
            .get("models")
            .or_else(|| value.get("data"))
            .and_then(|v| v.as_array())
            .ok_or_else(|| "the server did not list any models".to_string())?;

        let mut names: Vec<String> = Vec::new();
        for item in list {
            let name = item
                .get("name")
                .or_else(|| item.get("id"))
                .or_else(|| item.get("model"))
                .and_then(|v| v.as_str())
                .or_else(|| item.as_str());
            if let Some(n) = name {
                if !n.trim().is_empty() {
                    names.push(n.to_string());
                }
            }
        }
        names.sort();
        names.dedup();
        Ok(names)
    }

    /// Model names the local runtime already has on disk, lowercased. `None`
    /// when the runtime could not be asked (not running, wrong endpoint, no
    /// answer in time).
    pub async fn installed_models(endpoint: &str, provider: &str) -> Option<Vec<String>> {
        let target = parse_endpoint(endpoint).ok()?;
        let client = Client {
            provider: provider.to_string(),
            target,
            model: String::new(),
            timeout: Duration::from_millis(800),
            temperature: 0.85,
        };
        let names = client.probe().await.ok()?;
        Some(names.iter().map(|n| n.trim().to_lowercase()).collect())
    }

    async fn request(&self, method: &str, path: &str, body: Option<&str>) -> Result<String, String> {
        let mut stream = tokio::net::TcpStream::connect((self.target.host.as_str(), self.target.port))
            .await
            .map_err(|e| {
                format!(
                    "cannot reach {}:{} ({e}). Is the local model server running?",
                    self.target.host, self.target.port
                )
            })?;

        let len = body.map(|b| b.len()).unwrap_or(0);
        let head = format!(
            "{method} {path} HTTP/1.1\r\nHost: {host}:{port}\r\nContent-Type: application/json\r\n\
             Accept: application/json\r\nContent-Length: {len}\r\nConnection: close\r\n\r\n",
            host = self.target.host,
            port = self.target.port,
        );
        stream
            .write_all(head.as_bytes())
            .await
            .map_err(|e| format!("write failed ({e})"))?;
        if let Some(b) = body {
            stream
                .write_all(b.as_bytes())
                .await
                .map_err(|e| format!("write failed ({e})"))?;
        }
        stream
            .flush()
            .await
            .map_err(|e| format!("write failed ({e})"))?;

        let mut buf: Vec<u8> = Vec::new();
        let mut chunk = [0u8; 8192];
        loop {
            let n = stream
                .read(&mut chunk)
                .await
                .map_err(|e| format!("read failed ({e})"))?;
            if n == 0 {
                break;
            }
            buf.extend_from_slice(&chunk[..n]);
            if response_complete(&buf) {
                break;
            }
            if buf.len() > 4 * 1024 * 1024 {
                return Err("response too large".to_string());
            }
        }

        split_response(&buf)
    }
}

fn header_end(buf: &[u8]) -> Option<usize> {
    buf.windows(4)
        .position(|w| w == b"\r\n\r\n")
        .map(|i| i + 4)
}

fn response_complete(buf: &[u8]) -> bool {
    let end = match header_end(buf) {
        Some(e) => e,
        None => return false,
    };
    let head = String::from_utf8_lossy(&buf[..end]).to_ascii_lowercase();
    let body = &buf[end..];
    if head.contains("transfer-encoding: chunked") {
        body.windows(5).any(|w| w == b"0\r\n\r\n")
    } else if let Some(len) = head
        .split("\r\n")
        .find_map(|line| line.strip_prefix("content-length:"))
        .and_then(|v| v.trim().parse::<usize>().ok())
    {
        body.len() >= len
    } else {
        false
    }
}

fn dechunk(mut body: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    loop {
        let line_end = match body.windows(2).position(|w| w == b"\r\n") {
            Some(i) => i,
            None => break,
        };
        let size_text = String::from_utf8_lossy(&body[..line_end]);
        let size = usize::from_str_radix(
            size_text.trim().split(';').next().unwrap_or("0").trim(),
            16,
        )
        .unwrap_or(0);
        body = &body[line_end + 2..];
        if size == 0 || body.len() < size {
            break;
        }
        out.extend_from_slice(&body[..size]);
        body = if body.len() >= size + 2 {
            &body[size + 2..]
        } else {
            &[]
        };
    }
    out
}

fn split_response(buf: &[u8]) -> Result<String, String> {
    let end = header_end(buf).ok_or("incomplete HTTP response from the model")?;
    let head = String::from_utf8_lossy(&buf[..end]).to_string();
    let status: u16 = head
        .split_whitespace()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    let mut body = buf[end..].to_vec();
    if head.to_ascii_lowercase().contains("transfer-encoding: chunked") {
        body = dechunk(&body);
    }
    let text = String::from_utf8_lossy(&body).to_string();

    if !(200..300).contains(&status) {
        let detail = serde_json::from_str::<Value>(&text)
            .ok()
            .and_then(|v| {
                v.get("error")
                    .and_then(|e| e.as_str())
                    .map(|s| s.to_string())
            })
            .unwrap_or_else(|| text.trim().chars().take(160).collect());
        return Err(format!("model HTTP {status}: {detail}"));
    }
    Ok(text)
}

fn parse_chat(provider: &str, raw: &str) -> Result<String, String> {
    let value: Value =
        serde_json::from_str(raw).map_err(|e| format!("bad JSON from the model ({e})"))?;
    if let Some(err) = value.get("error").and_then(|e| e.as_str()) {
        return Err(err.to_string());
    }

    let text = if provider == "openai" {
        value
            .pointer("/choices/0/message/content")
            .and_then(|c| c.as_str())
            .unwrap_or("")
    } else {
        value
            .pointer("/message/content")
            .and_then(|c| c.as_str())
            .or_else(|| value.get("response").and_then(|c| c.as_str()))
            .unwrap_or("")
    };

    let cleaned = tidy(text);
    if cleaned.is_empty() {
        return Err("the model returned an empty reply".to_string());
    }
    Ok(cleaned)
}

/// Models drift towards markdown, speaker labels and essays. Keep the reply
/// usable in a chat bubble: plain text, no fake label, sane length.
fn tidy(raw: &str) -> String {
    let mut out = raw.trim().to_string();

    let lower = out.to_ascii_lowercase();
    for label in ["assistant:", "ai:", "companion:", "aria:"] {
        if lower.starts_with(label) {
            out = out[label.len()..].trim_start().to_string();
            break;
        }
    }

    if out.len() >= 2 {
        let first = out.chars().next().unwrap_or(' ');
        let last = out.chars().last().unwrap_or(' ');
        if (first == '"' && last == '"') || (first == '\u{201c}' && last == '\u{201d}') {
            out = out
                .trim_matches(|c| c == '"' || c == '\u{201c}' || c == '\u{201d}')
                .trim()
                .to_string();
        }
    }

    out = out.replace("**", "").replace("__", "");
    while out.contains("\n\n\n") {
        out = out.replace("\n\n\n", "\n\n");
    }

    if out.chars().count() > 900 {
        out = out.chars().take(900).collect::<String>();
        if let Some(idx) = out.rfind(['.', '!', '?']) {
            out.truncate(idx + 1);
        }
    }
    out.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_loopback_endpoints() {
        let t = parse_endpoint("http://127.0.0.1:11434").unwrap();
        assert_eq!(t.host, "127.0.0.1");
        assert_eq!(t.port, 11434);
        assert_eq!(t.path, "");

        let t = parse_endpoint("localhost:1234/v1").unwrap();
        assert_eq!(t.host, "localhost");
        assert_eq!(t.port, 1234);
        assert_eq!(t.path, "/v1");

        assert!(parse_endpoint("http://[::1]:8080").is_ok());
    }

    #[test]
    fn refuses_remote_and_tls_endpoints() {
        assert!(parse_endpoint("http://example.com:11434").is_err());
        assert!(parse_endpoint("https://127.0.0.1:11434").is_err());
        assert!(parse_endpoint("http://0.0.0.0:11434").is_err());
        assert!(parse_endpoint("").is_err());
    }

    #[test]
    fn extracts_ollama_and_openai_replies() {
        let ollama = r#"{"message":{"role":"assistant","content":"Hello there."},"done":true}"#;
        assert_eq!(parse_chat("ollama", ollama).unwrap(), "Hello there.");

        let openai = r#"{"choices":[{"message":{"role":"assistant","content":"Hi."}}]}"#;
        assert_eq!(parse_chat("openai", openai).unwrap(), "Hi.");
    }

    #[test]
    fn surfaces_model_errors() {
        assert!(parse_chat("ollama", r#"{"error":"model not found"}"#).is_err());
        assert!(parse_chat("ollama", r#"{"message":{"content":"   "}}"#).is_err());
    }

    #[test]
    fn tidies_labels_quotes_and_length() {
        assert_eq!(tidy("Assistant: \"Hey.\""), "Hey.");
        assert_eq!(tidy("**bold** claim"), "bold claim");
        let long = "word ".repeat(400);
        assert!(tidy(&long).chars().count() <= 900);
    }

    #[test]
    fn decodes_chunked_bodies() {
        let body = b"5\r\nHello\r\n6\r\n there\r\n0\r\n\r\n";
        assert_eq!(String::from_utf8_lossy(&dechunk(body)), "Hello there");
    }
}
