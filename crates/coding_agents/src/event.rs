//! Normalized event types for AI agent output.
//!
//! All AI coding agents produce different output formats, but this module
//! normalizes them to a common event stream format.
//!
//! These types are defined in `rpc_protocol` and re-exported here for
//! convenience.

pub use rpc_protocol::{FileChangeType, NormalizedEvent, TimestampedEvent, TokenUsage};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalized_event_creation() {
        let text = NormalizedEvent::text("Hello, world!", false);
        assert!(matches!(
            text,
            NormalizedEvent::TextOutput {
                content,
                stream: false
            } if content == "Hello, world!"
        ));

        let error = NormalizedEvent::error("Something went wrong");
        assert!(matches!(
            error,
            NormalizedEvent::ErrorOutput { content } if content == "Something went wrong"
        ));
    }

    #[test]
    fn test_tty_input_detection() {
        let ask = NormalizedEvent::ask_user("Continue?", Some(vec!["Yes".into(), "No".into()]));
        assert!(ask.is_tty_input_required());

        let text = NormalizedEvent::text("Output", false);
        assert!(!text.is_tty_input_required());
    }

    #[test]
    fn test_file_change_types() {
        let create =
            NormalizedEvent::file_change("test.rs", FileChangeType::Create, Some("content".into()));
        assert!(matches!(
            create,
            NormalizedEvent::FileChange {
                path,
                change_type: FileChangeType::Create,
                ..
            } if path == "test.rs"
        ));

        let rename = NormalizedEvent::file_change(
            "new.rs",
            FileChangeType::Rename {
                from: "old.rs".into(),
            },
            None,
        );
        assert!(matches!(
            rename,
            NormalizedEvent::FileChange {
                change_type: FileChangeType::Rename { from },
                ..
            } if from == "old.rs"
        ));
    }

    #[test]
    fn test_serialization() {
        let event = NormalizedEvent::tool_use("read_file", serde_json::json!({"path": "test.rs"}));
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("tool_use"));
        assert!(json.contains("read_file"));
    }
}
