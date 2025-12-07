//! Message envelope for reliable delivery and synchronization.
//!
//! Every message between client and server is wrapped in an envelope that provides:
//! - Sequence numbers for ordering and acknowledgment
//! - Timestamps for clock synchronization
//! - Piggyback acknowledgments to reduce round trips
//!
//! # Wire Format
//!
//! The envelope is optional for backward compatibility. Messages can be sent
//! either as raw payloads (legacy) or wrapped in envelopes (new protocol).
//!
//! ```json
//! // Legacy format (still supported)
//! { "type": "heartbeat" }
//!
//! // Envelope format
//! {
//!   "seq": 42,
//!   "ack": 41,
//!   "ts": 1701234567890,
//!   "payload": { "type": "heartbeat" }
//! }
//! ```

use serde::{Deserialize, Serialize};

/// Message envelope wrapping any payload with delivery metadata.
///
/// Used for reliable message delivery with:
/// - Sequence numbers for ordering
/// - Acknowledgments for delivery confirmation
/// - Timestamps for latency measurement and clock sync
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Envelope<T> {
    /// Monotonically increasing sequence number (per connection).
    /// Server and client maintain separate sequences.
    pub seq: u64,

    /// Piggyback acknowledgment of the last received sequence number.
    /// Allows confirming receipt without a separate ack message.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ack: Option<u64>,

    /// Server timestamp in milliseconds since Unix epoch.
    /// Clients can use this for latency calculation and clock sync.
    #[serde(rename = "ts")]
    pub timestamp: u64,

    /// The actual message payload.
    pub payload: T,
}

impl<T> Envelope<T> {
    /// Create a new envelope with the given sequence number and payload.
    pub fn new(seq: u64, payload: T) -> Self {
        Self {
            seq,
            ack: None,
            timestamp: Self::now_millis(),
            payload,
        }
    }

    /// Create an envelope with a piggyback acknowledgment.
    pub fn with_ack(seq: u64, ack: u64, payload: T) -> Self {
        Self {
            seq,
            ack: Some(ack),
            timestamp: Self::now_millis(),
            payload,
        }
    }

    /// Get current time in milliseconds since Unix epoch.
    fn now_millis() -> u64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0)
    }
}

impl<T> Envelope<T>
where
    T: Clone,
{
    /// Transform the payload while preserving envelope metadata.
    pub fn map<U, F>(self, f: F) -> Envelope<U>
    where
        F: FnOnce(T) -> U,
    {
        Envelope {
            seq: self.seq,
            ack: self.ack,
            timestamp: self.timestamp,
            payload: f(self.payload),
        }
    }
}

/// Either an enveloped message or a raw payload (for backward compatibility).
///
/// During the migration period, clients may send either format.
/// The server should accept both and respond in the same format.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MaybeEnveloped<T> {
    /// New format with envelope
    Enveloped(Envelope<T>),
    /// Legacy format without envelope
    Raw(T),
}

impl<T> MaybeEnveloped<T> {
    /// Extract the payload regardless of format.
    pub fn into_payload(self) -> T {
        match self {
            MaybeEnveloped::Enveloped(env) => env.payload,
            MaybeEnveloped::Raw(payload) => payload,
        }
    }

    /// Get sequence number if enveloped, None otherwise.
    pub fn seq(&self) -> Option<u64> {
        match self {
            MaybeEnveloped::Enveloped(env) => Some(env.seq),
            MaybeEnveloped::Raw(_) => None,
        }
    }

    /// Get acknowledgment if enveloped, None otherwise.
    pub fn ack(&self) -> Option<u64> {
        match self {
            MaybeEnveloped::Enveloped(env) => env.ack,
            MaybeEnveloped::Raw(_) => None,
        }
    }

    /// Check if this is an enveloped message.
    pub fn is_enveloped(&self) -> bool {
        matches!(self, MaybeEnveloped::Enveloped(_))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_envelope_serialization() {
        let envelope = Envelope::new(42, json!({"type": "heartbeat"}));
        let json = serde_json::to_string(&envelope).unwrap();

        assert!(json.contains("\"seq\":42"));
        assert!(json.contains("\"ts\":"));
        assert!(json.contains("\"payload\""));
    }

    #[test]
    fn test_envelope_with_ack() {
        let envelope = Envelope::with_ack(42, 41, "test");
        assert_eq!(envelope.seq, 42);
        assert_eq!(envelope.ack, Some(41));
    }

    #[test]
    fn test_maybe_enveloped_raw() {
        let raw: MaybeEnveloped<String> = MaybeEnveloped::Raw("hello".to_string());
        assert!(!raw.is_enveloped());
        assert_eq!(raw.seq(), None);
        assert_eq!(raw.into_payload(), "hello");
    }

    #[test]
    fn test_maybe_enveloped_envelope() {
        let env = MaybeEnveloped::Enveloped(Envelope::new(1, "hello".to_string()));
        assert!(env.is_enveloped());
        assert_eq!(env.seq(), Some(1));
    }

    #[test]
    fn test_deserialize_raw_message() {
        let json = r#"{"type": "heartbeat"}"#;
        let result: MaybeEnveloped<serde_json::Value> = serde_json::from_str(json).unwrap();
        assert!(!result.is_enveloped());
    }

    #[test]
    fn test_deserialize_enveloped_message() {
        let json = r#"{"seq": 1, "ts": 12345, "payload": {"type": "heartbeat"}}"#;
        let result: MaybeEnveloped<serde_json::Value> = serde_json::from_str(json).unwrap();
        assert!(result.is_enveloped());
        assert_eq!(result.seq(), Some(1));
    }
}
