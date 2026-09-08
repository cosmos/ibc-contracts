// SPDX-License-Identifier: Apache-2.0

const CLIENT_ID_PREFIX: &str = "client-";
const CHANNEL_ID_PREFIX: &str = "channel-";

/// Validates a custom IBC identifier per ICS-24 rules.
///
/// - Length must be between `min_len` and `max_len` characters
/// - Must NOT start with "client-" or "channel-" (reserved prefixes)
/// - Can only contain:
///   - Alphanumeric characters (a-z, A-Z, 0-9)
///   - Special characters: `.`, `_`, `+`, `-`, `#`, `[`, `]`, `<`, `>`
pub fn validate_custom_ibc_identifier(id: &str, min_len: usize, max_len: usize) -> bool {
    let len = id.len();

    if len < min_len || len > max_len {
        return false;
    }

    if id.starts_with(CLIENT_ID_PREFIX) || id.starts_with(CHANNEL_ID_PREFIX) {
        return false;
    }

    id.bytes().all(|c| {
        matches!(c,
            b'a'..=b'z' |
            b'0'..=b'9' |
            b'A'..=b'Z' |
            b'.' | b'_' | b'+' | b'-' |
            b'#' | b'[' | b']' | b'<' | b'>'
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_identifiers() {
        assert!(validate_custom_ibc_identifier("ab", 2, 128));
        assert!(validate_custom_ibc_identifier("abcd", 4, 128));
        assert!(validate_custom_ibc_identifier("test-port", 2, 128));
        assert!(validate_custom_ibc_identifier("my.port_v2", 2, 128));
        assert!(validate_custom_ibc_identifier("port+extra#1", 2, 128));
        assert!(validate_custom_ibc_identifier("a[0]", 2, 128));
        assert!(validate_custom_ibc_identifier("<tag>", 2, 128));
    }

    #[test]
    fn test_empty_and_whitespace() {
        assert!(!validate_custom_ibc_identifier("", 2, 128));
        assert!(!validate_custom_ibc_identifier("   ", 2, 128));
    }

    #[test]
    fn test_too_short() {
        assert!(!validate_custom_ibc_identifier("a", 2, 128));
        assert!(!validate_custom_ibc_identifier("abc", 4, 128));
    }

    #[test]
    fn test_too_long() {
        let long = "a".repeat(129);
        assert!(!validate_custom_ibc_identifier(&long, 2, 128));
    }

    #[test]
    fn test_reserved_prefixes() {
        assert!(!validate_custom_ibc_identifier("client-foo", 2, 128));
        assert!(!validate_custom_ibc_identifier("channel-bar", 2, 128));
    }

    #[test]
    fn test_invalid_characters() {
        assert!(!validate_custom_ibc_identifier("test@port", 2, 128));
        assert!(!validate_custom_ibc_identifier("test port", 2, 128));
        assert!(!validate_custom_ibc_identifier("test/port", 2, 128));
        assert!(!validate_custom_ibc_identifier("test\x00port", 2, 128));
    }

    #[test]
    fn test_max_boundary_length() {
        let max = "a".repeat(128);
        assert!(validate_custom_ibc_identifier(&max, 2, 128));
        let over = "a".repeat(129);
        assert!(!validate_custom_ibc_identifier(&over, 2, 128));
    }
}
