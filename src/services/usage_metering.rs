use std::collections::HashMap;

use serde::Deserialize;
use serde_json::Value;

use crate::providers::ChatMessage;

#[derive(Debug, Clone, Copy, Default)]
pub struct UsageSnapshot {
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub cached_input_tokens: i64,
    pub estimated: bool,
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct CreditRates {
    #[serde(default = "default_input_rate")]
    pub input_per_token: f64,
    #[serde(default = "default_output_rate")]
    pub output_per_token: f64,
    #[serde(default = "default_cached_rate")]
    pub cached_input_per_token: f64,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct PricingPolicy {
    #[serde(default)]
    pub default: Option<CreditRates>,
    #[serde(default)]
    pub models: HashMap<String, CreditRates>,
}

fn default_input_rate() -> f64 {
    1.0
}

fn default_output_rate() -> f64 {
    4.0
}

fn default_cached_rate() -> f64 {
    0.2
}

impl Default for CreditRates {
    fn default() -> Self {
        Self {
            input_per_token: default_input_rate(),
            output_per_token: default_output_rate(),
            cached_input_per_token: default_cached_rate(),
        }
    }
}

impl PricingPolicy {
    pub fn rates_for_model(&self, model_slug: &str) -> CreditRates {
        self.models
            .get(model_slug)
            .copied()
            .or(self.default)
            .unwrap_or_else(|| builtin_rates(model_slug))
    }
}

pub fn parse_pricing_policy(features: Option<&Value>) -> PricingPolicy {
    features
        .and_then(|value| value.get("quota"))
        .and_then(|value| value.get("pricing"))
        .cloned()
        .and_then(|value| serde_json::from_value::<PricingPolicy>(value).ok())
        .unwrap_or_default()
}

pub fn estimate_text_tokens(text: &str) -> i64 {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return 0;
    }
    ((trimmed.chars().count() as f64) / 4.0).ceil() as i64
}

pub fn estimate_chat_input_tokens(system_prompt: &str, messages: &[ChatMessage]) -> i64 {
    let mut total = estimate_text_tokens(system_prompt);
    for message in messages {
        total += 4;
        total += estimate_text_tokens(&message.role);
        total += estimate_text_tokens(&message.content);
    }
    total
}

pub fn estimate_output_tokens(output_text: &str, reasoning_text: &str) -> i64 {
    estimate_text_tokens(output_text) + estimate_text_tokens(reasoning_text)
}

pub fn build_estimated_usage(
    input_tokens: i64,
    output_text: &str,
    reasoning_text: &str,
) -> UsageSnapshot {
    UsageSnapshot {
        input_tokens,
        output_tokens: estimate_output_tokens(output_text, reasoning_text),
        cached_input_tokens: 0,
        estimated: true,
    }
}

pub fn calculate_credits(snapshot: UsageSnapshot, rates: CreditRates) -> i64 {
    let billable_input = (snapshot.input_tokens - snapshot.cached_input_tokens).max(0) as f64;
    let cached_input = snapshot.cached_input_tokens.max(0) as f64;
    let output = snapshot.output_tokens.max(0) as f64;
    let total = (billable_input * rates.input_per_token)
        + (cached_input * rates.cached_input_per_token)
        + (output * rates.output_per_token);
    total.ceil().max(0.0) as i64
}

fn builtin_rates(model_slug: &str) -> CreditRates {
    match model_slug {
        "gpt-5.4" => CreditRates {
            input_per_token: 2.0,
            output_per_token: 8.0,
            cached_input_per_token: 0.5,
        },
        "gpt-5.4-mini" => CreditRates {
            input_per_token: 1.0,
            output_per_token: 4.0,
            cached_input_per_token: 0.25,
        },
        "gpt-5.3-codex" => CreditRates {
            input_per_token: 2.0,
            output_per_token: 6.0,
            cached_input_per_token: 0.4,
        },
        "gpt-5.2" => CreditRates {
            input_per_token: 1.5,
            output_per_token: 5.0,
            cached_input_per_token: 0.3,
        },
        _ if model_slug.starts_with("grok-4") => CreditRates {
            input_per_token: 1.5,
            output_per_token: 5.0,
            cached_input_per_token: 0.3,
        },
        _ => CreditRates::default(),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        CreditRates, PricingPolicy, UsageSnapshot, build_estimated_usage, calculate_credits,
        estimate_chat_input_tokens, estimate_text_tokens,
    };
    use crate::providers::ChatMessage;

    #[test]
    fn estimates_tokens_from_chars() {
        assert_eq!(estimate_text_tokens(""), 0);
        assert_eq!(estimate_text_tokens("1234"), 1);
        assert_eq!(estimate_text_tokens("12345"), 2);
    }

    #[test]
    fn estimates_chat_with_message_overhead() {
        let messages = vec![ChatMessage {
            role: "user".into(),
            content: "hello world".into(),
        }];
        assert!(estimate_chat_input_tokens("system", &messages) > estimate_text_tokens("hello world"));
    }

    #[test]
    fn calculates_credits_with_custom_rates() {
        let snapshot = UsageSnapshot {
            input_tokens: 10,
            output_tokens: 5,
            cached_input_tokens: 2,
            estimated: true,
        };
        let credits = calculate_credits(
            snapshot,
            CreditRates {
                input_per_token: 1.0,
                output_per_token: 2.0,
                cached_input_per_token: 0.5,
            },
        );
        assert_eq!(credits, 19);
    }

    #[test]
    fn prefers_model_specific_rates() {
        let policy: PricingPolicy = serde_json::from_value(serde_json::json!({
            "default": { "input_per_token": 1, "output_per_token": 4, "cached_input_per_token": 0.2 },
            "models": {
                "gpt-5.4": { "input_per_token": 3, "output_per_token": 9, "cached_input_per_token": 1 }
            }
        }))
        .unwrap();
        assert_eq!(policy.rates_for_model("gpt-5.4").input_per_token, 3.0);
        assert_eq!(policy.rates_for_model("unknown").input_per_token, 1.0);
    }

    #[test]
    fn builds_output_usage_estimate() {
        let usage = build_estimated_usage(100, "answer", "thinking");
        assert_eq!(usage.input_tokens, 100);
        assert!(usage.output_tokens > 0);
        assert!(usage.estimated);
    }
}
