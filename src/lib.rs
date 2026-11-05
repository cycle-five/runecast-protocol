//! `RuneCast` Protocol Library
//!
//! This crate defines the WebSocket protocol for `RuneCast`, including:
//!
//! - Client-to-server message types
//! - Server-to-client message types
//! - Shared data types (`Grid`, `Position`, `PlayerInfo`, etc.)
//! - Player identity and context types
//!
//! # Usage
//!
//! ```rust
//! use runecast_protocol::protocol::{ClientMessage, ServerMessage};
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
//! // Serialize it for the wire
//! let wire = serde_json::to_string(&response).unwrap();
//! assert!(wire.contains(r#""type":"heartbeat_ack""#));
//! ```

pub mod player;
pub mod protocol;

// Re-export commonly used items at crate root for convenience
pub use player::{PlayerContext, PlayerIdentity};
pub use protocol::{
    client_messages::ClientMessage,
    server_messages::ServerMessage,
    types::{
        default_dictionary, AdminGameInfo, AdventureEventKind, BotDifficulty, BotSpec, BuildInfo,
        ConnectKind, ConnectParams, CustomMeta, DebugBackendGameState, DebugHandlerGameState,
        DebugLobbyState, DebugPlayerInfo, DebugWebsocketContext, ErrorCode, EventConfig,
        GameConfig, GamePlayerInfo, GameSnapshot, GameState, GameSummary, Grid, GridCell,
        LobbyChange, LobbyGameInfo, LobbyGamePlayerInfo, LobbyPlayerInfo, LobbyType, ModeCap,
        Multiplier, PlayerInfo, PlayerName, Position, RuneBoxStanding, ScoreInfo, SpectatorInfo,
        TimerVoteState, TurnPassReason, MAIN_DICTIONARY,
    },
    LobbySnapshot,
};
