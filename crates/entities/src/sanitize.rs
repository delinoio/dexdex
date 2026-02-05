//! Input sanitization utilities shared across the application.
//!
//! These functions provide consistent sanitization for user-provided text
//! across both the main-server API and the Tauri app commands.

/// Maximum allowed length for plan update feedback in characters.
///
/// This prevents resource exhaustion from very large feedback payloads.
pub const MAX_FEEDBACK_LENGTH: usize = 10_000;

/// Sanitizes user-provided text by removing control characters.
///
/// Removes all control characters except newlines (`\n`) and tabs (`\t`),
/// which are valid in user prompts and feedback.
///
/// # Examples
///
/// ```
/// use entities::sanitize_user_input;
///
/// assert_eq!(sanitize_user_input("Hello\x00World"), "HelloWorld");
/// assert_eq!(sanitize_user_input("Line1\nLine2\tTabbed"), "Line1\nLine2\tTabbed");
/// ```
pub fn sanitize_user_input(input: &str) -> String {
    input
        .chars()
        .filter(|c| !c.is_control() || *c == '\n' || *c == '\t')
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_preserves_normal_text() {
        assert_eq!(sanitize_user_input("Hello World"), "Hello World");
    }

    #[test]
    fn test_sanitize_preserves_newlines_and_tabs() {
        assert_eq!(
            sanitize_user_input("Line1\nLine2\tTabbed"),
            "Line1\nLine2\tTabbed"
        );
    }

    #[test]
    fn test_sanitize_removes_null_bytes() {
        assert_eq!(sanitize_user_input("Hello\x00World"), "HelloWorld");
    }

    #[test]
    fn test_sanitize_removes_control_characters() {
        assert_eq!(sanitize_user_input("Hello\x01\x02\x03World"), "HelloWorld");
    }

    #[test]
    fn test_sanitize_empty_string() {
        assert_eq!(sanitize_user_input(""), "");
    }

    #[test]
    fn test_max_feedback_length_constant() {
        assert_eq!(MAX_FEEDBACK_LENGTH, 10_000);
    }
}
