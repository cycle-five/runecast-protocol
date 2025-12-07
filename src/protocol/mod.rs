//! Protocol module for RuneCast WebSocket communication.
//!
//! This module defines all message types exchanged between the frontend client
//! and backend server over WebSocket connections.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────────┐
//! │                           Protocol Layer                             │
//! ├─────────────────────────────────────────────────────────────────────┤
//! │  envelope.rs     - Message wrapper with seq/ack/timestamp           │
//! │  types.rs        - Shared data types (Grid, Position, etc.)         │
//! │  client_messages - Client → Server message definitions              │
//! │  server_messages - Server → Client message definitions              │
//! └─────────────────────────────────────────────────────────────────────┘
//! ```
//!
//! # Message Flow
//!
//! ```text
//! Client                                Server
//!   │                                      │
//!   │──── Identify ──────────────────────▶│
//!   │◀─── Ready (LobbySnapshot) ──────────│
//!   │                                      │
//!   │──── JoinChannelLobby ──────────────▶│
//!   │◀─── LobbyJoined ────────────────────│
//!   │                                      │
//!   │──── StartGame ─────────────────────▶│
//!   │◀─── GameStarted ────────────────────│
//!   │                                      │
//!   │──── SubmitWord ────────────────────▶│
//!   │◀─── WordScored ─────────────────────│
//!   │◀─── TurnChanged ────────────────────│
//! ```
//!
//! # Envelope Format (Optional)
//!
//! Messages can be sent raw (legacy) or wrapped in an envelope (new protocol):
//!
//! ```json
//! // Legacy (still supported)
//! {"type": "heartbeat"}
//!
//! // With envelope
//! {"seq": 42, "ack": 41, "ts": 1701234567890, "payload": {"type": "heartbeat"}}
//! ```
//!
//! # Migration Strategy
//!
//! This module is designed to coexist with the legacy `websocket/messages.rs`.
//! During migration:
//!
//! 1. New code imports from `protocol::`
//! 2. Compatibility functions convert between old and new formats
//! 3. Once migration is complete, remove legacy module

pub mod client_messages;
pub mod envelope;
pub mod server_messages;
pub mod types;

// Re-export main types for convenient access
pub use client_messages::ClientMessage;
pub use envelope::{Envelope, MaybeEnveloped};
pub use server_messages::{GameSnapshot, LobbySnapshot, ServerMessage};
pub use types::*;

// ============================================================================
// Protocol Constants
// ============================================================================

/// Recommended heartbeat interval (client should send heartbeat this often).
pub const HEARTBEAT_INTERVAL_MS: u32 = 30_000;

/// Heartbeat timeout (server closes connection if no heartbeat received).
pub const HEARTBEAT_TIMEOUT_MS: u32 = 45_000;

/// Grace period for reconnection before session expires.
pub const RECONNECT_GRACE_MS: u32 = 60_000;

/// Maximum message size in bytes.
pub const MAX_MESSAGE_SIZE: usize = 64 * 1024; // 64 KB

/// Protocol version for compatibility checks.
pub const PROTOCOL_VERSION: &str = "1.0.0";

// ============================================================================
// Compatibility Layer
// ============================================================================

/// Module for converting between legacy and new message formats.
///
/// This allows gradual migration without breaking the existing frontend.
pub mod compat {
    use super::*;
    use serde_json::Value;

    /// Parse a raw JSON message, handling both legacy and new formats.
    ///
    /// Returns the parsed message and whether it was enveloped.
    pub fn parse_client_message(
        json: &str,
    ) -> Result<(ClientMessage, Option<u64>, Option<u64>), serde_json::Error> {
        // First try to parse as enveloped
        if let Ok(enveloped) = serde_json::from_str::<MaybeEnveloped<ClientMessage>>(json) {
            match enveloped {
                MaybeEnveloped::Enveloped(env) => {
                    return Ok((env.payload, Some(env.seq), env.ack));
                }
                MaybeEnveloped::Raw(msg) => {
                    return Ok((msg, None, None));
                }
            }
        }

        // Fall back to legacy parsing
        let msg: ClientMessage = serde_json::from_str(json)?;
        Ok((msg, None, None))
    }

    /// Serialize a server message, optionally wrapping in an envelope.
    pub fn serialize_server_message(
        msg: &ServerMessage,
        seq: Option<u64>,
        ack: Option<u64>,
    ) -> Result<String, serde_json::Error> {
        match seq {
            Some(seq) => {
                let envelope = match ack {
                    Some(ack) => Envelope::with_ack(seq, ack, msg),
                    None => Envelope::new(seq, msg),
                };
                serde_json::to_string(&envelope)
            }
            None => serde_json::to_string(msg),
        }
    }

    /// Convert legacy game state JSON to new GameSnapshot format.
    ///
    /// This handles the transition from the flat `game_state` message
    /// to the structured `GameSnapshot` type.
    pub fn legacy_game_state_to_snapshot(value: &Value) -> Option<GameSnapshot> {
        // Extract fields from legacy format
        let game_id = value.get("game_id")?.as_str()?.to_string();
        let state_str = value.get("state")?.as_str()?;

        let state = match state_str {
            "idle" => GameState::Idle,
            "queueing" => GameState::Queueing,
            "starting" => GameState::Starting,
            "in_progress" => GameState::InProgress,
            "finished" => GameState::Finished,
            "cancelled" => GameState::Cancelled,
            _ => GameState::Idle,
        };

        Some(GameSnapshot {
            game_id,
            state,
            grid: serde_json::from_value(value.get("grid")?.clone()).ok()?,
            players: serde_json::from_value(value.get("players")?.clone()).ok()?,
            spectators: serde_json::from_value(value.get("spectators")?.clone())
                .unwrap_or_default(),
            current_turn: value.get("current_turn")?.as_str()?.to_string(),
            round: value.get("round")?.as_i64()? as u8,
            max_rounds: value.get("max_rounds")?.as_i64()? as u8,
            used_words: serde_json::from_value(value.get("used_words")?.clone())
                .unwrap_or_default(),
            timer_vote_state: serde_json::from_value(
                value
                    .get("timer_vote_state")
                    .cloned()
                    .unwrap_or(Value::Null),
            )
            .unwrap_or_default(),
            your_player: None,
            turn_time_remaining: None,
        })
    }

    /// Convert new GameSnapshot to legacy game_state message format.
    ///
    /// Used when sending to clients that haven't upgraded yet.
    pub fn snapshot_to_legacy_game_state(snapshot: &GameSnapshot) -> ServerMessage {
        let state_str = match snapshot.state {
            GameState::Idle => "idle",
            GameState::Queueing => "queueing",
            GameState::Starting => "starting",
            GameState::InProgress => "in_progress",
            GameState::Finished => "finished",
            GameState::Cancelled => "cancelled",
        };

        ServerMessage::GameStateUpdate {
            game_id: snapshot.game_id.clone(),
            state: state_str.to_string(),
            grid: snapshot.grid.clone(),
            players: snapshot.players.clone(),
            current_turn: snapshot.current_turn.clone(),
            round: snapshot.round as i32,
            max_rounds: snapshot.max_rounds as i32,
            used_words: snapshot.used_words.clone(),
            spectators: snapshot.spectators.clone(),
            timer_vote_state: snapshot.timer_vote_state.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_legacy_heartbeat() {
        let json = r#"{"type":"heartbeat"}"#;
        let (msg, seq, ack) = compat::parse_client_message(json).unwrap();
        assert!(matches!(msg, ClientMessage::Heartbeat));
        assert!(seq.is_none());
        assert!(ack.is_none());
    }

    #[test]
    fn test_parse_enveloped_heartbeat() {
        let json = r#"{"seq":42,"ack":41,"ts":12345,"payload":{"type":"heartbeat"}}"#;
        let (msg, seq, ack) = compat::parse_client_message(json).unwrap();
        assert!(matches!(msg, ClientMessage::Heartbeat));
        assert_eq!(seq, Some(42));
        assert_eq!(ack, Some(41));
    }

    #[test]
    fn test_serialize_without_envelope() {
        let msg = ServerMessage::HeartbeatAck { server_time: 12345 };
        let json = compat::serialize_server_message(&msg, None, None).unwrap();
        assert!(!json.contains("seq"));
        assert!(json.contains("heartbeat_ack"));
    }

    #[test]
    fn test_serialize_with_envelope() {
        let msg = ServerMessage::HeartbeatAck { server_time: 12345 };
        let json = compat::serialize_server_message(&msg, Some(1), Some(0)).unwrap();
        assert!(json.contains(r#""seq":1"#));
        assert!(json.contains(r#""ack":0"#));
        assert!(json.contains("payload"));
    }

    #[test]
    fn test_constants() {
        assert_eq!(HEARTBEAT_INTERVAL_MS, 30_000);
        assert!(HEARTBEAT_TIMEOUT_MS > HEARTBEAT_INTERVAL_MS);
        assert!(RECONNECT_GRACE_MS > HEARTBEAT_TIMEOUT_MS);
    }
}
