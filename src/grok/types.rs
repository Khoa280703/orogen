use serde::{Deserialize, Serialize};

/// Grok API request payload
#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GrokRequest {
    pub message: String,
    pub model_name: String,
    pub temporary: bool,
    pub file_attachments: Vec<()>,
    pub image_attachments: Vec<()>,
    pub disable_search: bool,
    pub enable_image_generation: bool,
    pub return_image_bytes: bool,
    pub return_raw_grok_in_xai_request: bool,
    pub enable_image_streaming: bool,
    pub image_generation_count: u32,
    pub force_concise: bool,
    pub tool_overrides: serde_json::Value,
    pub enable_side_by_side: bool,
    pub is_preset: bool,
    pub send_final_metadata: bool,
    pub custom_instructions: String,
    pub deepsearch_preset: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_metadata: Option<serde_json::Value>,
    pub is_reasoning: bool,
}

impl GrokRequest {
    pub fn new(message: String, model: String, reasoning: bool, system_prompt: String) -> Self {
        Self {
            message,
            model_name: model,
            temporary: true,
            file_attachments: vec![],
            image_attachments: vec![],
            disable_search: false,
            enable_image_generation: false,
            return_image_bytes: false,
            return_raw_grok_in_xai_request: false,
            enable_image_streaming: false,
            image_generation_count: 0,
            force_concise: false,
            tool_overrides: serde_json::json!({}),
            enable_side_by_side: false,
            is_preset: false,
            send_final_metadata: true,
            custom_instructions: system_prompt,
            deepsearch_preset: String::new(),
            response_metadata: None,
            is_reasoning: reasoning,
        }
    }

    pub fn new_image_generation(message: String, model: String) -> Self {
        let mut request = Self::new(message, model, false, String::new());
        request.tool_overrides = serde_json::json!({ "imageGen": true });
        request.response_metadata = Some(serde_json::json!({
            "modelConfigOverride": {
                "modelMap": {
                    "imageEditModel": "imagine"
                }
            }
        }));
        request.enable_side_by_side = true;
        request
    }

    pub fn new_video_generation(
        message: String,
        model: String,
        parent_post_id: String,
        aspect_ratio: String,
        duration_seconds: u32,
        resolution_name: String,
    ) -> Self {
        let mut request = Self::new(message, model, false, String::new());
        request.tool_overrides = serde_json::json!({ "videoGen": true });
        request.response_metadata = Some(serde_json::json!({
            "modelConfigOverride": {
                "modelMap": {
                    "videoGenModelConfig": {
                        "parentPostId": parent_post_id,
                        "aspectRatio": aspect_ratio,
                        "videoLength": duration_seconds,
                        "isVideoEdit": false,
                        "resolutionName": resolution_name,
                        "isReferenceToVideo": false
                    }
                }
            }
        }));
        request.enable_side_by_side = false;
        request
    }
}

/// Parsed streaming event from Grok response
#[derive(Debug, Clone)]
pub enum GrokStreamEvent {
    Token(String),
    Thinking(String),
    WebSearch,
    Done,
}

/// Grok response line structure
#[derive(Debug, Deserialize)]
pub struct GrokResponseLine {
    pub result: Option<GrokResult>,
}

#[derive(Debug, Deserialize)]
pub struct GrokResult {
    pub response: Option<GrokResponseData>,
    #[serde(rename = "webSearchResults")]
    pub web_search_results: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct GrokResponseData {
    pub token: Option<String>,
    pub thinking: Option<String>,
    pub search: Option<serde_json::Value>,
}
