//! RuneCast Protocol Library
//!
//! This crate defines the WebSocket protocol for RuneCast, including:
//!
//! - Message envelopes with sequence numbers for reliable delivery
//! - Client-to-server message types
//! - Server-to-client message types
//! - Shared data types (Grid, Position, PlayerInfo, etc.)
//! - Compatibility layer for gradual migration
//!
//! # Usage
//!
//! ```rust
//! use runecast_protocol::protocol::{ClientMessage, ServerMessage, Envelope};
//!
//! // Parse an incoming client message
//! let json = r#"{"type":"heartbeat"}"#;
//! let msg: ClientMessage = serde_json::from_str(json).unwrap();
//!
//! // Create a server response
//! let response = ServerMessage::HeartbeatAck {
//!     server_time: 1701234567890,
//! };
//!
//! // Optionally wrap in envelope for reliable delivery
//! let envelope = Envelope::new(42, response);
//! ```

pub mod protocol;

// Re-export commonly used items at crate root for convenience
pub use protocol::{
    ClientMessage, Envelope, ErrorCode, GameSnapshot, GameState, Grid, GridCell, LobbySnapshot,
    MaybeEnveloped, Multiplier, PlayerInfo, Position, ServerMessage, TimerVoteState, AdminGameInfo,
};
