use serde::Serialize;
use serde_json::Value;
use std::collections::BTreeMap;

const ASSET_BASE_URL: &str = "https://assets";

#[derive(Debug, Clone, Serialize)]
pub struct GeneratedImageAsset {
    pub id: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct GeneratedVideoAsset {
    pub id: String,
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolution_name: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ParsedMediaResponse<T> {
    pub assets: Vec<T>,
    pub errors: Vec<String>,
}

#[allow(dead_code)]
pub fn parse_image_generation_body(body: &str) -> ParsedMediaResponse<GeneratedImageAsset> {
    let mut assets = BTreeMap::new();
    let mut errors = Vec::new();

    for value in parse_lines(body, &mut errors) {
        errors.extend(extract_error_messages(&value));

        if let Some(stream) = value
            .get("result")
            .and_then(|v| v.get("response"))
            .and_then(|v| v.get("streamingImageGenerationResponse"))
        {
            let id = stream.get("imageId").and_then(Value::as_str);
            let url = stream.get("imageUrl").and_then(Value::as_str);
            if let (Some(id), Some(url)) = (id, url) {
                assets.insert(
                    id.to_string(),
                    GeneratedImageAsset {
                        id: id.to_string(),
                        url: absolutize_asset_url(url),
                    },
                );
            }
        }

        if let Some(urls) = value
            .get("result")
            .and_then(|v| v.get("response"))
            .and_then(|v| v.get("userResponse"))
            .and_then(|v| v.get("generatedImageUrls"))
            .and_then(Value::as_array)
        {
            for url in urls.iter().filter_map(Value::as_str) {
                let normalized_url = absolutize_asset_url(url);
                assets.insert(
                    normalized_url.clone(),
                    GeneratedImageAsset {
                        id: asset_id_from_url(url),
                        url: normalized_url,
                    },
                );
            }
        }
    }

    ParsedMediaResponse {
        assets: assets.into_values().collect(),
        errors: unique_messages(errors),
    }
}

pub fn parse_video_generation_body(body: &str) -> ParsedMediaResponse<GeneratedVideoAsset> {
    let mut assets = BTreeMap::new();
    let mut errors = Vec::new();

    for value in parse_lines(body, &mut errors) {
        errors.extend(extract_error_messages(&value));

        if let Some(stream) = value
            .get("result")
            .and_then(|v| v.get("response"))
            .and_then(|v| v.get("streamingVideoGenerationResponse"))
        {
            let id = stream.get("videoId").and_then(Value::as_str);
            let url = stream.get("videoUrl").and_then(Value::as_str);
            if let (Some(id), Some(url)) = (id, url) {
                assets.insert(
                    id.to_string(),
                    GeneratedVideoAsset {
                        id: id.to_string(),
                        url: absolutize_asset_url(url),
                        model_name: stream
                            .get("modelName")
                            .and_then(Value::as_str)
                            .map(str::to_string),
                        resolution_name: stream
                            .get("resolutionName")
                            .and_then(Value::as_str)
                            .map(str::to_string),
                    },
                );
            }
        }

        if let Some(urls) = value
            .get("result")
            .and_then(|v| v.get("response"))
            .and_then(|v| v.get("userResponse"))
            .and_then(|v| v.get("generatedVideoUrls"))
            .and_then(Value::as_array)
        {
            for url in urls.iter().filter_map(Value::as_str) {
                let normalized_url = absolutize_asset_url(url);
                assets.insert(
                    normalized_url.clone(),
                    GeneratedVideoAsset {
                        id: asset_id_from_url(url),
                        url: normalized_url,
                        model_name: None,
                        resolution_name: None,
                    },
                );
            }
        }
    }

    ParsedMediaResponse {
        assets: assets.into_values().collect(),
        errors: unique_messages(errors),
    }
}

fn parse_lines(body: &str, errors: &mut Vec<String>) -> Vec<Value> {
    body.lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                return None;
            }

            match serde_json::from_str::<Value>(trimmed) {
                Ok(value) => Some(value),
                Err(error) => {
                    errors.push(format!("Invalid JSON line: {error}"));
                    None
                }
            }
        })
        .collect()
}

fn extract_error_messages(value: &Value) -> Vec<String> {
    let mut messages = Vec::new();

    if let Some(message) = value
        .get("error")
        .and_then(|v| v.get("message"))
        .and_then(Value::as_str)
    {
        messages.push(message.to_string());
    }

    if let Some(message) = value
        .get("result")
        .and_then(|v| v.get("response"))
        .and_then(|v| v.get("error"))
        .and_then(|v| v.get("message"))
        .and_then(Value::as_str)
    {
        messages.push(message.to_string());
    }

    messages.extend(extract_messages_from_array(
        value
            .get("result")
            .and_then(|v| v.get("response"))
            .and_then(|v| v.get("modelResponse"))
            .and_then(|v| v.get("streamErrors")),
    ));
    messages.extend(extract_messages_from_array(
        value
            .get("result")
            .and_then(|v| v.get("response"))
            .and_then(|v| v.get("userResponse"))
            .and_then(|v| v.get("streamErrors")),
    ));
    messages.extend(extract_messages_from_array(
        value
            .get("result")
            .and_then(|v| v.get("response"))
            .and_then(|v| v.get("userResponse"))
            .and_then(|v| v.get("metadata"))
            .and_then(|v| v.get("stream_errors")),
    ));

    messages
}

fn extract_messages_from_array(value: Option<&Value>) -> Vec<String> {
    value
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.get("message").and_then(Value::as_str))
                .map(str::to_string)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

fn asset_id_from_url(url: &str) -> String {
    let trimmed = url.trim_end_matches('/');
    trimmed
        .rsplit('/')
        .next()
        .filter(|segment| !segment.is_empty())
        .unwrap_or(trimmed)
        .to_string()
}

fn unique_messages(messages: Vec<String>) -> Vec<String> {
    let mut seen = BTreeMap::new();
    for message in messages {
        seen.entry(message.clone()).or_insert(message);
    }
    seen.into_values().collect()
}

fn absolutize_asset_url(url: &str) -> String {
    if url.starts_with("http://") || url.starts_with("https://") {
        url.to_string()
    } else {
        format!("{ASSET_BASE_URL}/{url}")
    }
}
