use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::config::load_config;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Conversation {
    pub id: String,
    pub title: String,
    pub model: String,
    pub messages: Vec<ChatMessage>,
    pub created_at: String,
    pub updated_at: String,
}

pub struct ConversationStore {
    data_dir: PathBuf,
}

impl ConversationStore {
    pub fn new() -> Self {
        let config = load_config();
        let data_dir = PathBuf::from(&config.data_dir);
        fs::create_dir_all(&data_dir).ok();
        Self { data_dir }
    }

    fn file_path(&self, id: &str) -> PathBuf {
        self.data_dir.join(format!("{id}.json"))
    }

    /// Create a new conversation
    pub fn create(&self, model: &str) -> Conversation {
        let now = chrono::Utc::now().to_rfc3339();
        let id = uuid::Uuid::new_v4().to_string()[..10].to_string();
        let conv = Conversation {
            id,
            title: "New conversation".into(),
            model: model.into(),
            messages: vec![],
            created_at: now.clone(),
            updated_at: now,
        };
        self.save(&conv);
        conv
    }

    /// Save conversation to disk
    pub fn save(&self, conv: &Conversation) {
        let mut conv = conv.clone();
        conv.updated_at = chrono::Utc::now().to_rfc3339();
        // Auto-title from first user message
        if conv.title == "New conversation" {
            if let Some(first_user) = conv.messages.iter().find(|m| m.role == "user") {
                conv.title = first_user
                    .content
                    .chars()
                    .take(60)
                    .collect::<String>()
                    .replace('\n', " ");
            }
        }
        let json = serde_json::to_string_pretty(&conv).unwrap();
        fs::write(self.file_path(&conv.id), json).ok();
    }

    /// Load a conversation by ID
    pub fn load(&self, id: &str) -> Option<Conversation> {
        let fp = self.file_path(id);
        let raw = fs::read_to_string(fp).ok()?;
        serde_json::from_str(&raw).ok()
    }

    /// List all conversations, sorted by updatedAt desc
    pub fn list(&self) -> Vec<Conversation> {
        let mut convs = Vec::new();
        let entries = match fs::read_dir(&self.data_dir) {
            Ok(e) => e,
            Err(_) => return convs,
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|e| e == "json") {
                if let Ok(raw) = fs::read_to_string(&path) {
                    if let Ok(conv) = serde_json::from_str::<Conversation>(&raw) {
                        convs.push(conv);
                    }
                }
            }
        }
        convs.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        convs
    }

    /// Delete a conversation
    pub fn delete(&self, id: &str) -> bool {
        let fp = self.file_path(id);
        if fp.exists() {
            fs::remove_file(fp).is_ok()
        } else {
            false
        }
    }

    /// Add a message to conversation and save
    pub fn add_message(&self, conv: &mut Conversation, role: &str, content: &str) {
        conv.messages.push(ChatMessage {
            role: role.into(),
            content: content.into(),
        });
        self.save(conv);
    }
}
