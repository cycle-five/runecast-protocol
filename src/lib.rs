//! `RuneCast` Protocol Library
//!
//! This crate defines the WebSocket protocol for `RuneCast`, including:
//!
//! - Message envelopes with sequence numbers for reliable delivery
//! - Client-to-server message types
//! - Server-to-client message types
//! - Shared data types (`Grid`, `Position`, `PlayerInfo`, etc.)
//! - Player identity and context types
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

pub mod player;
pub mod protocol;

// Re-export commonly used items at crate root for convenience
pub use player::{PlayerContext, PlayerIdentity};
pub use protocol::compat::{
    legacy_game_state_to_snapshot, parse_client_message, serialize_server_message,
    snapshot_to_legacy_game_state,
};
pub use protocol::{
    client_messages::ClientMessage,
    envelope::{Envelope, MaybeEnveloped},
    server_messages::ServerMessage,
    types::{
        AdminGameInfo, DebugBackendGameState, DebugHandlerGameState, DebugLobbyState,
        DebugPlayerInfo, DebugWebsocketContext, ErrorCode, GameChange, GameConfig, GamePlayerInfo,
        GameSnapshot, GameState, GameSummary, Grid, GridCell, LobbyChange, LobbyGameInfo,
        LobbyGamePlayerInfo, LobbyPlayerInfo, LobbyType, Multiplier, PlayerInfo, Position,
        ScoreInfo, SpectatorInfo, TimerVoteState,
    },
    LobbySnapshot,
};
