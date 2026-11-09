//! Shared JSON logging for the proof API and SP1 diagnostics.
use std::time::{SystemTime, UNIX_EPOCH};

/// UTC Unix timestamp in milliseconds, independent of the host timezone.
pub fn timestamp_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}

pub fn log(mut fields: serde_json::Value) {
    fields["timestamp_ms"] = serde_json::json!(timestamp_ms());
    eprintln!("{fields}");
}
