use crate::grok::types::{GrokResponseLine, GrokStreamEvent};

/// Parse a single JSON line from Grok streaming response
pub fn parse_line(line: &str) -> Option<GrokStreamEvent> {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return None;
    }

    let data: GrokResponseLine = serde_json::from_str(trimmed).ok()?;
    let result = data.result?;

    // Check for token
    if let Some(response) = &result.response {
        if let Some(token) = &response.token {
            if !token.is_empty() {
                return Some(GrokStreamEvent::Token(token.clone()));
            }
        }
        if let Some(thinking) = &response.thinking {
            if !thinking.is_empty() {
                return Some(GrokStreamEvent::Thinking(thinking.clone()));
            }
        }
        if response.search.is_some() {
            return Some(GrokStreamEvent::WebSearch);
        }
    }

    if result.web_search_results.is_some() {
        return Some(GrokStreamEvent::WebSearch);
    }

    None
}

/// Parse all lines from a complete Grok response, return concatenated tokens
#[allow(dead_code)]
pub fn parse_full_response(body: &str) -> Vec<GrokStreamEvent> {
    body.lines().filter_map(parse_line).collect()
}
