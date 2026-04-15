use futures::StreamExt;
use serde::Serialize;
use tokio::sync::mpsc;
use wreq_util::Emulation;

use crate::account::types::GrokCookies;
use crate::grok::headers::build_grok_headers;
use crate::grok::types::{GrokRequest, GrokStreamEvent};

const GROK_API_URL: &str = "https://grok.com/rest/app-chat/conversations/new";
const REQUEST_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(120);
const CONNECT_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(30);

/// wreq-based Grok API client with optional SOCKS5 proxy
#[derive(Clone)]
pub struct GrokClient;

impl GrokClient {
    pub async fn new() -> Result<Self, GrokRequestError> {
        Ok(Self)
    }

    async fn post_json<T: Serialize>(
        url: &str,
        cookies: &GrokCookies,
        payload: &T,
        proxy_url: Option<&String>,
    ) -> Result<String, GrokRequestError> {
        let client = Self::build_client(proxy_url)?;
        let cookie_header = cookies.to_header();
        let headers = build_grok_headers(&cookie_header, None);
        let body = serde_json::to_string(payload)
            .map_err(|e| GrokRequestError::Network(format!("Serialize: {e}")))?;

        let mut req = client.post(url).body(body);
        for (key, value) in headers {
            req = req.header(key, value);
        }

        let resp = req
            .send()
            .await
            .map_err(|e| {
                if proxy_url.is_some() {
                    GrokRequestError::ProxyFailed(format!("Proxy request failed: {e}"))
                } else {
                    GrokRequestError::Network(format!("Request failed: {e}"))
                }
            })?;

        let status = resp.status().as_u16();
        let text = resp
            .text()
            .await
            .map_err(|e| GrokRequestError::Network(format!("Read body: {e}")))?;

        classify_response(status, &text)?;
        Ok(text)
    }

    /// Build a fresh wreq client per request with optional proxy
    fn build_client(proxy_url: Option<&String>) -> Result<wreq::Client, GrokRequestError> {
        let mut builder = wreq::Client::builder()
            .emulation(Emulation::Chrome136)
            .timeout(REQUEST_TIMEOUT)
            .connect_timeout(CONNECT_TIMEOUT);

        if let Some(url) = proxy_url {
            tracing::debug!(
                "Using proxy: {}",
                url.split('@').last().unwrap_or(url.as_str())
            );
            let proxy = wreq::Proxy::all(url)
                .map_err(|e| GrokRequestError::ProxyFailed(format!("Invalid proxy: {e}")))?;
            builder = builder.proxy(proxy);
        }

        builder
            .build()
            .map_err(|e| GrokRequestError::Network(format!("Build client: {e}")))
    }

    /// Send a non-streaming request with optional proxy
    pub async fn send_request(
        &self,
        cookies: &GrokCookies,
        payload: &GrokRequest,
        proxy_url: Option<&String>,
    ) -> Result<String, GrokRequestError> {
        Self::post_json(GROK_API_URL, cookies, payload, proxy_url).await
    }

    pub async fn send_json_request<T: Serialize>(
        &self,
        url: &str,
        cookies: &GrokCookies,
        payload: &T,
        proxy_url: Option<&String>,
    ) -> Result<String, GrokRequestError> {
        Self::post_json(url, cookies, payload, proxy_url).await
    }

    /// Send a streaming request with optional proxy — returns receiver with parsed events
    pub async fn send_request_stream(
        &self,
        cookies: &GrokCookies,
        payload: &GrokRequest,
        proxy_url: Option<&String>,
    ) -> Result<mpsc::UnboundedReceiver<StreamEvent>, GrokRequestError> {
        let client = Self::build_client(proxy_url)?;
        let cookie_header = cookies.to_header();
        let headers = build_grok_headers(&cookie_header, None);

        let body = serde_json::to_string(payload)
            .map_err(|e| GrokRequestError::Network(format!("Serialize: {e}")))?;

        let mut req = client.post(GROK_API_URL).body(body);
        for (key, value) in headers {
            req = req.header(key, value);
        }

        let resp = req
            .send()
            .await
            .map_err(|e| {
                if proxy_url.is_some() {
                    GrokRequestError::ProxyFailed(format!("Proxy request failed: {e}"))
                } else {
                    GrokRequestError::Network(format!("Request failed: {e}"))
                }
            })?;

        let status = resp.status().as_u16();
        if status != 200 {
            let text = resp.text().await.unwrap_or_default();
            classify_response(status, &text)?;
            unreachable!();
        }

        let (tx, rx) = mpsc::unbounded_channel();

        tokio::spawn(async move {
            let mut stream = resp.bytes_stream();
            let mut pending = Vec::new();
            while let Some(chunk_result) = stream.next().await {
                match chunk_result {
                    Ok(bytes) => match extract_stream_events(&mut pending, &bytes) {
                        Ok(events) => {
                            for event in events {
                                if tx.send(StreamEvent::Event(event)).is_err() {
                                    return;
                                }
                            }
                        }
                        Err(error) => {
                            let _ = tx.send(StreamEvent::Error(error));
                            return;
                        }
                    },
                    Err(e) => {
                        let _ = tx.send(StreamEvent::Error(e.to_string()));
                        return;
                    }
                }
            }

            match flush_pending_event(&mut pending) {
                Ok(Some(event)) => {
                    let _ = tx.send(StreamEvent::Event(event));
                }
                Ok(None) => {}
                Err(error) => {
                    let _ = tx.send(StreamEvent::Error(error));
                    return;
                }
            }
            let _ = tx.send(StreamEvent::Done);
        });

        Ok(rx)
    }

    pub async fn shutdown(&self) {}
}

pub enum StreamEvent {
    Event(crate::grok::types::GrokStreamEvent),
    Error(String),
    Done,
}

fn extract_stream_events(
    pending: &mut Vec<u8>,
    chunk: &[u8],
) -> Result<Vec<GrokStreamEvent>, String> {
    pending.extend_from_slice(chunk);
    let mut events = Vec::new();

    while let Some(pos) = pending.iter().position(|byte| *byte == b'\n') {
        let line_bytes: Vec<u8> = pending.drain(..=pos).collect();
        let line = decode_stream_line(&line_bytes)?;
        if let Some(event) = parse_stream_line(&line) {
            events.push(event);
        }
    }

    Ok(events)
}

fn flush_pending_event(pending: &mut Vec<u8>) -> Result<Option<GrokStreamEvent>, String> {
    let line = decode_stream_line(pending)?;
    pending.clear();
    if line.is_empty() {
        return Ok(None);
    }

    Ok(parse_stream_line(&line))
}

fn decode_stream_line(bytes: &[u8]) -> Result<String, String> {
    let line =
        std::str::from_utf8(bytes).map_err(|error| format!("Invalid UTF-8 in stream: {error}"))?;
    Ok(line.trim().to_string())
}

fn parse_stream_line(line: &str) -> Option<GrokStreamEvent> {
    if line.is_empty() {
        None
    } else {
        crate::grok::stream_parser::parse_line(line)
    }
}

fn classify_response(status: u16, body: &str) -> Result<(), GrokRequestError> {
    let body_preview = &body.chars().take(300).collect::<String>();
    let body_lower = body.to_ascii_lowercase();

    match status {
        200 => {
            if body.contains("\"code\":7") || body.contains("anti-bot") {
                tracing::warn!("Anti-bot detected in response: {}", body_preview);
                return Err(GrokRequestError::AntiBot);
            }
            Ok(())
        }
        403 => {
            tracing::warn!("403 response from xAI: {}", body_preview);
            Err(GrokRequestError::CfBlocked)
        }
        429 => {
            tracing::warn!("429 rate limited: {}", body_preview);
            Err(GrokRequestError::RateLimited)
        }
        401 => {
            tracing::warn!("401 unauthorized: {}", body_preview);
            Err(GrokRequestError::Unauthorized)
        }
        400
            if body_lower.contains("failed to look up session id")
                || body_lower.contains("invalid-credentials")
                || body_lower.contains("unauthenticated") =>
        {
            tracing::warn!("400 invalid session mapped to unauthorized: {}", body_preview);
            Err(GrokRequestError::Unauthorized)
        }
        _ => {
            tracing::warn!("Unexpected status {}: {}", status, body_preview);
            Err(GrokRequestError::HttpError(
                status,
                body_preview.to_string(),
            ))
        }
    }
}

#[derive(Debug)]
pub enum GrokRequestError {
    Network(String),
    ProxyFailed(String),
    CfBlocked,
    RateLimited,
    Unauthorized,
    HttpError(u16, String),
    AntiBot,
}

impl std::fmt::Display for GrokRequestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Network(msg) => write!(f, "Network error: {msg}"),
            Self::ProxyFailed(msg) => write!(f, "Proxy failed: {msg}"),
            Self::CfBlocked => write!(f, "Cloudflare blocked (403)"),
            Self::RateLimited => write!(f, "Rate limited (429)"),
            Self::Unauthorized => write!(f, "Unauthorized (401)"),
            Self::HttpError(code, body) => write!(f, "HTTP error: {code}: {body}"),
            Self::AntiBot => write!(f, "Anti-bot rejection (code 7)"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{extract_stream_events, flush_pending_event};
    use crate::grok::types::GrokStreamEvent;

    #[test]
    fn extracts_events_when_line_is_split_across_chunks() {
        let mut pending = Vec::new();
        let chunk_a = br#"{"result":{"response":{"token":"Xin "}}}"#;
        let chunk_b = b"\n";

        let first = extract_stream_events(&mut pending, chunk_a).unwrap();
        let second = extract_stream_events(&mut pending, chunk_b).unwrap();

        assert!(first.is_empty());
        assert!(matches!(second.as_slice(), [GrokStreamEvent::Token(token)] if token == "Xin "));
    }

    #[test]
    fn preserves_utf8_when_multibyte_character_is_split_across_chunks() {
        let mut pending = Vec::new();
        let line = "{\"result\":{\"response\":{\"token\":\"Hôm\"}}}\n";
        let bytes = line.as_bytes();
        let split = bytes.iter().position(|byte| *byte == 0xC3).unwrap() + 1;

        let first = extract_stream_events(&mut pending, &bytes[..split]).unwrap();
        let second = extract_stream_events(&mut pending, &bytes[split..]).unwrap();

        assert!(first.is_empty());
        assert!(matches!(
            second.as_slice(),
            [GrokStreamEvent::Token(token)] if token == "Hôm"
        ));
    }

    #[test]
    fn flushes_last_line_without_trailing_newline() {
        let mut pending = r#"{"result":{"response":{"token":"chao"}}}"#.as_bytes().to_vec();

        assert!(matches!(
            flush_pending_event(&mut pending).unwrap(),
            Some(GrokStreamEvent::Token(token)) if token == "chao"
        ));
        assert!(pending.is_empty());
    }
}
