/*
   src/models/chat.rs

   This module defines chat models used to construct and parse chat completion requests.
*/

use serde::{Deserialize, Serialize};

/// Represents a chat message with a role and content.
/// This is the model-side representation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: crate::types::chat::ChatRole,
    pub content: String,
}

/// Conversion from model's ChatMessage to types::chat::Message used in API requests.
impl From<ChatMessage> for crate::types::chat::Message {
    fn from(chat_msg: ChatMessage) -> Self {
        Self {
            role: chat_msg.role,
            content: crate::types::chat::MessageContent::Text(chat_msg.content),
            name: None,
            tool_calls: None,
            tool_call_id: None,
        }
    }
}
