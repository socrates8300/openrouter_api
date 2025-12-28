//! Tests for consolidated Message type with ChatRole enum
//!
//! This test suite ensures:
//! - Message uses ChatRole enum instead of raw String
//! - All role variants work correctly
//! - Multimodal content works
//! - Tool and assistant messages work
//! - Serialization/deserialization is correct

use openrouter_api::models::tool::ToolType;
use openrouter_api::types::chat::{
    AudioContent, AudioUrl, ChatRole, ContentPart, ContentType, FileContent, FileUrl, ImageContent,
    ImageUrl, Message, MessageContent, TextContent,
};

use openrouter_api::types::ids::ToolCallId;
use serde_json::{from_str, from_value, to_value};

/// Test that Message uses ChatRole enum, not String
#[test]
fn test_message_uses_chat_role_enum() {
    let msg = Message::text(ChatRole::User, "Hello, world!");

    // This should compile - role is ChatRole, not String
    match msg.role {
        ChatRole::User => {} // ✅ Should match
        ChatRole::Assistant => panic!("Wrong role"),
        ChatRole::System => panic!("Wrong role"),
        ChatRole::Tool => panic!("Wrong role"),
    }
}

/// Test all ChatRole variants work with Message
#[test]
fn test_all_chat_role_variants() {
    let user_msg = Message::text(ChatRole::User, "User message");
    assert_eq!(user_msg.role, ChatRole::User);

    let assistant_msg = Message::text(ChatRole::Assistant, "Assistant message");
    assert_eq!(assistant_msg.role, ChatRole::Assistant);

    let system_msg = Message::text(ChatRole::System, "System message");
    assert_eq!(system_msg.role, ChatRole::System);

    let tool_msg = Message::text(ChatRole::Tool, "Tool result");
    assert_eq!(tool_msg.role, ChatRole::Tool);
}

/// Test that ChatRole serializes correctly (lowercase)
#[test]
fn test_chat_role_serialization() {
    let msg = Message::text(ChatRole::User, "test");
    let json = to_value(&msg).unwrap();

    assert_eq!(json["role"], "user"); // lowercase, not "User"
}

/// Test that ChatRole deserializes from lowercase strings
#[test]
fn test_chat_role_deserialization() {
    let json = r#"{
        "role": "user",
        "content": "test"
    }"#;

    let msg: Message = from_str(json).unwrap();
    assert_eq!(msg.role, ChatRole::User);
}

/// Test assistant role deserializes correctly
#[test]
fn test_assistant_role_deserialization() {
    let json = r#"{
        "role": "assistant",
        "content": "response"
    }"#;

    let msg: Message = from_str(json).unwrap();
    assert_eq!(msg.role, ChatRole::Assistant);
}

/// Test system role deserializes correctly
#[test]
fn test_system_role_deserialization() {
    let json = r#"{
        "role": "system",
        "content": "instructions"
    }"#;

    let msg: Message = from_str(json).unwrap();
    assert_eq!(msg.role, ChatRole::System);
}

/// Test tool role deserializes correctly
#[test]
fn test_tool_role_deserialization() {
    let json = r#"{
        "role": "tool",
        "content": "tool output",
        "tool_call_id": "call_123"
    }"#;

    let msg: Message = from_str(json).unwrap();
    assert_eq!(msg.role, ChatRole::Tool);
    assert_eq!(msg.tool_call_id, Some(ToolCallId::new("call_123")));
}

/// Test Message with name field
#[test]
fn test_message_with_name() {
    let msg = Message::text_with_name(ChatRole::User, "message", "Alice");
    assert_eq!(msg.role, ChatRole::User);
    assert_eq!(msg.name, Some("Alice".to_string()));

    let json = r#"{
        "role": "user",
        "content": "message",
        "name": "Bob"
    }"#;

    let msg: Message = from_str(json).unwrap();
    assert_eq!(msg.role, ChatRole::User);
    assert_eq!(msg.name, Some("Bob".to_string()));
}

/// Test multimodal message with text content
#[test]
fn test_multimodal_text_content() {
    let text_part = ContentPart::Text(TextContent {
        content_type: ContentType::Text,
        text: "Hello".to_string(),
    });

    let msg = Message::multimodal(ChatRole::User, vec![text_part]);

    match msg.content {
        MessageContent::Parts(parts) => {
            assert_eq!(parts.len(), 1);
        }
        MessageContent::Text(_) => panic!("Should be Parts, not Text"),
    }
}

/// Test multimodal message with image content
#[test]
fn test_multimodal_image_content() {
    let image_part = ContentPart::Image(ImageContent {
        content_type: ContentType::ImageUrl,
        image_url: ImageUrl {
            url: "https://example.com/image.jpg".to_string(),
            detail: None,
        },
    });

    let msg = Message::multimodal(ChatRole::User, vec![image_part]);

    match msg.content {
        MessageContent::Parts(parts) => {
            assert_eq!(parts.len(), 1);
        }
        MessageContent::Text(_) => panic!("Should be Parts, not Text"),
    }
}

/// Test multimodal message with audio content
#[test]
fn test_multimodal_audio_content() {
    let audio_part = ContentPart::Audio(AudioContent {
        content_type: ContentType::AudioUrl,
        audio_url: AudioUrl {
            url: "https://example.com/audio.mp3".to_string(),
        },
    });

    let msg = Message::multimodal(ChatRole::User, vec![audio_part]);

    match msg.content {
        MessageContent::Parts(parts) => {
            assert_eq!(parts.len(), 1);
        }
        MessageContent::Text(_) => panic!("Should be Parts, not Text"),
    }
}

/// Test multimodal message with file content
#[test]
fn test_multimodal_file_content() {
    let file_part = ContentPart::File(FileContent {
        content_type: ContentType::FileUrl,
        file_url: FileUrl {
            url: "https://example.com/document.pdf".to_string(),
        },
    });

    let msg = Message::multimodal(ChatRole::User, vec![file_part]);

    match msg.content {
        MessageContent::Parts(parts) => {
            assert_eq!(parts.len(), 1);
        }
        MessageContent::Text(_) => panic!("Should be Parts, not Text"),
    }
}

/// Test multimodal message with mixed content
#[test]
fn test_multimodal_mixed_content() {
    let text_part = ContentPart::Text(TextContent {
        content_type: ContentType::Text,
        text: "Look at this image:".to_string(),
    });

    let image_part = ContentPart::Image(ImageContent {
        content_type: ContentType::ImageUrl,
        image_url: ImageUrl {
            url: "https://example.com/image.jpg".to_string(),
            detail: None,
        },
    });

    let msg = Message::multimodal(ChatRole::User, vec![text_part, image_part]);

    match msg.content {
        MessageContent::Parts(parts) => {
            assert_eq!(parts.len(), 2);
        }
        MessageContent::Text(_) => panic!("Should be Parts, not Text"),
    }
}

/// Test tool message construction
#[test]
fn test_tool_message() {
    let msg = Message::tool("tool output", "call_abc123");

    assert_eq!(msg.role, ChatRole::Tool);
    assert_eq!(msg.tool_call_id, Some(ToolCallId::new("call_abc123")));

    match msg.content {
        MessageContent::Text(s) => assert_eq!(s, "tool output"),
        MessageContent::Parts(_) => panic!("Tool message should have Text content"),
    }
}

/// Test assistant message with tools
#[test]
fn test_assistant_with_tools() {
    use openrouter_api::models::tool::{FunctionCall, ToolCall};

    let tool_calls = vec![ToolCall {
        id: ToolCallId::new("call_123"),
        kind: ToolType::Function,
        function_call: FunctionCall {
            name: "get_weather".to_string(),
            arguments: r#"{"city": "NYC"}"#.to_string(),
        },
    }];

    let msg = Message::assistant_with_tools(Some("I'll check the weather"), tool_calls.clone());

    assert_eq!(msg.role, ChatRole::Assistant);
    assert_eq!(msg.tool_calls, Some(tool_calls));

    match msg.content {
        MessageContent::Text(s) => assert_eq!(s, "I'll check the weather"),
        MessageContent::Parts(_) => panic!("Assistant message should have Text content"),
    }
}

/// Test assistant message with tools but no content
#[test]
fn test_assistant_with_tools_no_content() {
    use openrouter_api::models::tool::{FunctionCall, ToolCall};

    let tool_calls = vec![ToolCall {
        id: ToolCallId::new("call_456"),
        kind: ToolType::Function,
        function_call: FunctionCall {
            name: "get_weather".to_string(),
            arguments: r#"{"city": "LA"}"#.to_string(),
        },
    }];

    let msg = Message::assistant_with_tools(None::<String>, tool_calls);

    assert_eq!(msg.role, ChatRole::Assistant);
    assert!(msg.tool_calls.is_some());
}

/// Test Message default implementation
#[test]
fn test_message_default() {
    let msg = Message::default();
    assert_eq!(msg.role, ChatRole::User); // Should default to user
    assert_eq!(msg.name, None);
    assert_eq!(msg.tool_call_id, None);
    assert_eq!(msg.tool_calls, None);
}

/// Test full round-trip serialization/deserialization
#[test]
fn test_message_roundtrip_serialization() {
    let original = Message::text_with_name(ChatRole::System, "You are helpful", "SystemPrompt");

    let json = to_value(&original).unwrap();
    let deserialized: Message = from_value(json).unwrap();

    assert_eq!(original.role, deserialized.role);
    assert_eq!(original.name, deserialized.name);
    assert_eq!(original.tool_call_id, deserialized.tool_call_id);
}

/// Test that invalid role strings are rejected during deserialization
#[test]
fn test_invalid_role_deserialization_fails() {
    let json = r#"{
        "role": "invalid_role",
        "content": "test"
    }"#;

    let result: Result<Message, _> = from_str(json);
    assert!(result.is_err(), "Should reject invalid role strings");
}

/// Test case-insensitive role deserialization
#[test]
fn test_case_insensitive_role_deserialization() {
    let json = r#"{
        "role": "USER",
        "content": "test"
    }"#;

    // This should fail - we want strict lowercase only
    let result: Result<Message, _> = from_str(json);
    assert!(result.is_err(), "Should reject uppercase role");
}

/// Test that MessageContent::Text serializes correctly
#[test]
fn test_message_content_text_serialization() {
    let msg = Message::text(ChatRole::User, "Hello");
    let json = to_value(&msg).unwrap();

    assert_eq!(json["content"], "Hello");
}

/// Test that MessageContent::Parts serializes correctly
#[test]
fn test_message_content_parts_serialization() {
    let parts = vec![
        ContentPart::Text(TextContent {
            content_type: ContentType::Text,
            text: "Check this:".to_string(),
        }),
        ContentPart::Image(ImageContent {
            content_type: ContentType::ImageUrl,
            image_url: ImageUrl {
                url: "https://example.com/img.jpg".to_string(),
                detail: None,
            },
        }),
    ];

    let msg = Message::multimodal(ChatRole::User, parts);
    let json = to_value(&msg).unwrap();

    assert!(json["content"].is_array());
    assert_eq!(json["content"].as_array().unwrap().len(), 2);
}

/// Test that ChatRole implements all required traits
#[test]
fn test_chat_role_traits() {
    let role = ChatRole::User;

    // Test Clone
    let role_clone = role.clone();
    assert_eq!(role, role_clone);

    // Test PartialEq
    assert_eq!(role, ChatRole::User);
    assert_ne!(role, ChatRole::Assistant);

    // Test Debug
    let debug_str = format!("{:?}", role);
    assert!(debug_str.contains("User"));
}

/// Test that Message implements all required traits
#[test]
fn test_message_traits() {
    let msg = Message::text(ChatRole::User, "test");

    // Test Clone
    let msg_clone = msg.clone();
    assert_eq!(msg.role, msg_clone.role);

    // Test Debug
    let debug_str = format!("{:?}", msg);
    assert!(debug_str.contains("Message"));

    // Test PartialEq
    assert_eq!(msg, msg_clone);
}
