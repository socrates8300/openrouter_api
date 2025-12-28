use openrouter_api::client::{
    OpenRouterClient, Unconfigured, ROUTING_FLOOR, ROUTING_NITRO, ROUTING_ONLINE,
};
use openrouter_api::types::chat::{
    AudioContent, AudioUrl, ChatCompletionRequest, ChatRole, ContentPart, ContentType, FileContent,
    FileUrl, Message,
};
use openrouter_api::types::provider::ProviderPreferences;

#[test]
fn test_multimodal_serialization() {
    let audio_part = ContentPart::Audio(AudioContent {
        content_type: ContentType::AudioUrl,
        audio_url: AudioUrl {
            url: "https://example.com/audio.mp3".to_string(),
        },
    });

    let file_part = ContentPart::File(FileContent {
        content_type: ContentType::FileUrl,
        file_url: FileUrl {
            url: "https://example.com/document.pdf".to_string(),
        },
    });

    let json_audio = serde_json::to_value(&audio_part).unwrap();
    assert_eq!(json_audio["type"], "audio_url");
    assert_eq!(
        json_audio["audio_url"]["url"],
        "https://example.com/audio.mp3"
    );

    let json_file = serde_json::to_value(&file_part).unwrap();
    assert_eq!(json_file["type"], "file_url");
    assert_eq!(
        json_file["file_url"]["url"],
        "https://example.com/document.pdf"
    );
}

#[test]
fn test_routing_shortcuts() {
    assert_eq!(ROUTING_NITRO, ":nitro");
    assert_eq!(ROUTING_FLOOR, ":floor");
    assert_eq!(ROUTING_ONLINE, ":online");

    let model = "openai/gpt-4o";
    let model_with_nitro =
        OpenRouterClient::<Unconfigured>::model_with_shortcut(model, ROUTING_NITRO);
    assert_eq!(model_with_nitro, "openai/gpt-4o:nitro");
}

#[test]
fn test_policy_controls() {
    let prefs = ProviderPreferences {
        allow: Some(vec!["OpenAI".to_string(), "Anthropic".to_string()]),
        data_collection: Some("deny".to_string()), // ZDR
        ..ProviderPreferences::default()
    };

    let json_prefs = serde_json::to_value(&prefs).unwrap();
    assert_eq!(json_prefs["allow"][0], "OpenAI");
    assert_eq!(json_prefs["data_collection"], "deny");
}

#[test]
fn test_web_search_plugin() {
    let request = ChatCompletionRequest {
        model: "openai/gpt-4o".to_string(),
        messages: vec![Message::text(ChatRole::User, "Search for Rust news")],
        plugins: Some(vec!["web".to_string()]),
        ..Default::default()
    };

    let json_req = serde_json::to_value(&request).unwrap();
    assert_eq!(json_req["plugins"][0], "web");
}

#[test]
fn test_zdr_helper() {
    let _client = OpenRouterClient::<Unconfigured>::new()
        .skip_url_configuration()
        .with_zdr();

    // We can't easily inspect the private router_config, but we can verify it compiles and runs.
    // In a real scenario we might want to expose a way to inspect config or use a mock.
    // For now, this ensures the method exists and doesn't panic.
}
