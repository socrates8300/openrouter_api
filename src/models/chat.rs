/*
   src/models/chat.rs

   This module defines the chat models used to construct and parse chat completion requests.
*/

use serde::{Deserialize, Serialize};

/// Defines the role of a chat message (user, assistant, or system).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChatRole {
    User,
    Assistant,
    System,
}

/// Represents a chat message with a role and content.
/// This is the model-side representation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: ChatRole,
    pub content: String,
}

/// Conversion from the model’s ChatMessage to the types::chat::Message used in API requests.
impl From<ChatMessage> for crate::types::chat::Message {
    fn from(chat_msg: ChatMessage) -> Self {
        // Convert the role enum to the expected lowercase string.
        let role_str = match chat_msg.role {
            ChatRole::User => "user".to_string(),
            ChatRole::Assistant => "assistant".to_string(),
            ChatRole::System => "system".to_string(),
        };
        Self {
            role: role_str,
            content: chat_msg.content,
            name: None,
            tool_calls: None,
            tool_call_id: None,
        }
    }
}
