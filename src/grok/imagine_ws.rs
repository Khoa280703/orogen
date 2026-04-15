use std::collections::{BTreeMap, BTreeSet};
use std::time::Duration;

use futures::{SinkExt, StreamExt};
use serde::Deserialize;
use serde_json::json;
use tokio::net::TcpStream;
use tokio::time::timeout;
use tokio_socks::tcp::Socks5Stream;
use tokio_tungstenite::client_async_tls_with_config;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::{Error as WsError, Message};

use crate::account::types::GrokCookies;
use crate::grok::client::GrokRequestError;
use crate::grok::media_response_parser::GeneratedImageAsset;

const IMAGINE_WS_URL: &str = "wss://grok.com/ws/imagine/listen";
const IMAGINE_HOST: &str = "grok.com";
const IMAGINE_PORT: u16 = 443;
const CHROME_UA: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/136.0.0.0 Safari/537.36";
const REQUEST_TIMEOUT: Duration = Duration::from_secs(120);
const EXPECTED_IMAGE_COUNT: usize = 4;

#[derive(Debug, Deserialize)]
struct ImagineMessage {
    #[serde(default)]
    request_id: Option<String>,
    #[serde(default, rename = "type")]
    message_type: Option<String>,
    #[serde(default)]
    current_status: Option<String>,
    #[serde(default)]
    order: Option<u32>,
    #[serde(default)]
    id: Option<String>,
    #[serde(default)]
    url: Option<String>,
    #[serde(default)]
    err_code: Option<String>,
    #[serde(default)]
    err_message: Option<String>,
}

pub async fn generate_images(
    cookies: &GrokCookies,
    prompt: &str,
    enable_pro: bool,
    proxy_url: Option<&String>,
) -> Result<Vec<GeneratedImageAsset>, GrokRequestError> {
    timeout(
        REQUEST_TIMEOUT,
        generate_images_inner(cookies, prompt, enable_pro, proxy_url),
    )
    .await
    .map_err(|_| GrokRequestError::Network("Imagine websocket timed out".into()))?
}

async fn generate_images_inner(
    cookies: &GrokCookies,
    prompt: &str,
    enable_pro: bool,
    proxy_url: Option<&String>,
) -> Result<Vec<GeneratedImageAsset>, GrokRequestError> {
    let stream = connect_tcp_stream(proxy_url).await?;
    let mut request = IMAGINE_WS_URL
        .into_client_request()
        .map_err(|error| GrokRequestError::Network(format!("Build websocket request: {error}")))?;
    let headers = request.headers_mut();
    headers.insert("Host", IMAGINE_HOST.parse().unwrap());
    headers.insert("Origin", "https://grok.com".parse().unwrap());
    headers.insert("Referer", "https://grok.com/imagine".parse().unwrap());
    headers.insert("User-Agent", CHROME_UA.parse().unwrap());
    headers.insert("Cookie", cookies.to_header().parse().unwrap());
    let (mut ws, _) = client_async_tls_with_config(request, stream, None, None)
        .await
        .map_err(map_websocket_error)?;

    let request_id = uuid::Uuid::new_v4().to_string();
    if !enable_pro {
        ws.send(Message::Text(build_update_session_message(prompt).into()))
            .await
            .map_err(map_websocket_error)?;
    }
    ws.send(Message::Text(
        build_generation_message(prompt, &request_id, enable_pro).into(),
    ))
    .await
    .map_err(map_websocket_error)?;

    let mut assets_by_order = BTreeMap::new();
    let mut completed_orders = BTreeSet::new();

    while let Some(message) = ws.next().await {
        let message = message.map_err(map_websocket_error)?;
        if !message.is_text() {
            continue;
        }
        let payload = message.into_text().map_err(|error| {
            GrokRequestError::Network(format!("Decode websocket text: {error}"))
        })?;
        let event: ImagineMessage = serde_json::from_str(&payload)
            .map_err(|error| GrokRequestError::Network(format!("Invalid imagine JSON: {error}")))?;

        if event.request_id.as_deref() != Some(request_id.as_str()) {
            continue;
        }

        if matches!(event.message_type.as_deref(), Some("error"))
            || matches!(event.current_status.as_deref(), Some("error"))
        {
            return Err(classify_imagine_error(
                event.err_code.as_deref(),
                event.err_message.as_deref(),
            ));
        }

        if let (Some(order), Some(id), Some(url)) = (event.order, event.id, event.url) {
            assets_by_order.insert(order, GeneratedImageAsset { id, url });
        }

        if matches!(event.current_status.as_deref(), Some("completed")) {
            if let Some(order) = event.order {
                completed_orders.insert(order);
            }
            if completed_orders.len() >= EXPECTED_IMAGE_COUNT {
                break;
            }
        }
    }

    let mut assets = completed_orders
        .iter()
        .filter_map(|order| assets_by_order.get(order).cloned())
        .collect::<Vec<_>>();
    if assets.is_empty() {
        assets = assets_by_order.into_values().collect::<Vec<_>>();
    }
    if assets.is_empty() {
        return Err(GrokRequestError::Network(
            "Imagine websocket returned no images".into(),
        ));
    }
    Ok(assets)
}

async fn connect_tcp_stream(proxy_url: Option<&String>) -> Result<TcpStream, GrokRequestError> {
    let tcp_stream: TcpStream = if let Some(proxy_url) = proxy_url {
        let proxy = reqwest::Url::parse(proxy_url)
            .map_err(|error| GrokRequestError::Network(format!("Invalid proxy URL: {error}")))?;
        let proxy_host = proxy
            .host_str()
            .ok_or_else(|| GrokRequestError::Network("Proxy host is missing".into()))?;
        let proxy_port = proxy
            .port_or_known_default()
            .ok_or_else(|| GrokRequestError::Network("Proxy port is missing".into()))?;
        if proxy.username().is_empty() {
            Socks5Stream::connect((proxy_host, proxy_port), (IMAGINE_HOST, IMAGINE_PORT))
                .await
                .map(|stream: Socks5Stream<TcpStream>| stream.into_inner())
                .map_err(|error| {
                    GrokRequestError::Network(format!("SOCKS proxy connect failed: {error}"))
                })?
        } else {
            Socks5Stream::connect_with_password(
                (proxy_host, proxy_port),
                (IMAGINE_HOST, IMAGINE_PORT),
                proxy.username(),
                proxy.password().unwrap_or_default(),
            )
            .await
            .map(|stream: Socks5Stream<TcpStream>| stream.into_inner())
            .map_err(|error| {
                GrokRequestError::Network(format!("SOCKS proxy connect failed: {error}"))
            })?
        }
    } else {
        TcpStream::connect((IMAGINE_HOST, IMAGINE_PORT))
            .await
            .map_err(|error| GrokRequestError::Network(format!("TCP connect failed: {error}")))?
    };
    Ok(tcp_stream)
}

fn build_update_session_message(prompt: &str) -> String {
    json!({
        "type": "conversation.item.create",
        "timestamp": chrono::Utc::now().timestamp_millis(),
        "item": {
            "type": "message",
            "content": [{
                "type": "update_session",
                "properties": {
                    "section_count": 0,
                    "is_kids_mode": false,
                    "enable_nsfw": true,
                    "skip_upsampler": false,
                    "enable_side_by_side": true,
                    "is_initial": false,
                    "aspect_ratio": "2:3",
                    "enable_pro": false,
                    "last_prompt": prompt,
                }
            }]
        }
    })
    .to_string()
}

fn build_generation_message(prompt: &str, request_id: &str, enable_pro: bool) -> String {
    let content_type = if enable_pro {
        "input_text"
    } else {
        "input_scroll"
    };
    json!({
        "type": "conversation.item.create",
        "timestamp": chrono::Utc::now().timestamp_millis(),
        "item": {
            "type": "message",
            "content": [{
                "requestId": request_id,
                "text": prompt,
                "type": content_type,
                "properties": {
                    "section_count": 0,
                    "is_kids_mode": false,
                    "enable_nsfw": true,
                    "skip_upsampler": enable_pro,
                    "enable_side_by_side": true,
                    "is_initial": !enable_pro,
                    "aspect_ratio": "2:3",
                    "enable_pro": enable_pro,
                }
            }]
        }
    })
    .to_string()
}

fn classify_imagine_error(code: Option<&str>, message: Option<&str>) -> GrokRequestError {
    let error_message = message.unwrap_or("unknown imagine websocket error");
    match code.unwrap_or_default() {
        "image_query_rejected" | "rate_limit_exceeded" => GrokRequestError::RateLimited,
        "unauthorized" => GrokRequestError::Unauthorized,
        other if !other.is_empty() => GrokRequestError::Network(format!(
            "Imagine websocket error [{other}]: {error_message}"
        )),
        _ => GrokRequestError::Network(format!("Imagine websocket error: {error_message}")),
    }
}

fn map_websocket_error(error: WsError) -> GrokRequestError {
    match error {
        WsError::Http(response) => match response.status().as_u16() {
            401 => GrokRequestError::Unauthorized,
            403 => GrokRequestError::CfBlocked,
            429 => GrokRequestError::RateLimited,
            status => GrokRequestError::HttpError(status, "Websocket handshake failed".into()),
        },
        other => GrokRequestError::Network(format!("Websocket error: {other}")),
    }
}
