//! Protocol module for `RuneCast` WebSocket communication.
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
//!   │◀─── Hello ──────────────────────────│
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

pub mod client_messages;
pub mod server_messages;
pub mod types;

// Re-export main types for convenient access
pub use client_messages::ClientMessage;
pub use server_messages::{LobbySnapshot, ServerMessage};
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants() {
        assert_eq!(HEARTBEAT_INTERVAL_MS, 30_000);
        const {
            assert!(HEARTBEAT_TIMEOUT_MS > HEARTBEAT_INTERVAL_MS);
        }
        const {
            assert!(RECONNECT_GRACE_MS > HEARTBEAT_TIMEOUT_MS);
        }
    }
}
