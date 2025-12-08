//! Server-to-client messages.
//!
//! All messages that can be sent from the backend server to frontend clients.
//! Messages are tagged with `type` field for JSON serialization.
//!
//! # Categories
//!
//! - **Connection**: Handshake responses, heartbeat acks
//! - **Lobby State**: Snapshots and deltas for lobby state
//! - **Game State**: Snapshots and deltas for game state
//! - **Events**: Discrete events (player joined, word scored, etc.)
//! - **Errors**: Error responses with codes and messages

use serde::{Deserialize, Serialize};

use super::types::{
    AdminGameInfo, ErrorCode, GameChange, GamePlayerInfo, GameState, Grid, LobbyChange,
    LobbyGameInfo, LobbyPlayerInfo, LobbyType, PlayerInfo, ScoreInfo, SpectatorInfo,
    TimerVoteState,
};

/// Messages sent from server to client.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerMessage {
    // ========================================================================
    // Connection Messages
    // ========================================================================
    /// Initial server greeting after WebSocket connect.
    ///
    /// Sent immediately upon connection, before Identify.
    Hello {
        /// Recommended heartbeat interval in milliseconds
        heartbeat_interval_ms: u32,
        /// Server version for compatibility checks
        #[serde(skip_serializing_if = "Option::is_none")]
        server_version: Option<String>,
    },

    /// Successful authentication response.
    ///
    /// Contains the full initial state snapshot.
    Ready {
        /// Unique session ID for this connection
        session_id: String,
        /// The authenticated player's user ID
        player_id: String,
        /// Full lobby state (if in a lobby)
        #[serde(skip_serializing_if = "Option::is_none")]
        lobby: Option<LobbySnapshot>,
        /// Full game state (if in a game)
        #[serde(skip_serializing_if = "Option::is_none")]
        game: Option<GameSnapshot>,
    },

    /// Session resumed successfully after reconnect.
    ///
    /// Contains any events missed during disconnection.
    Resumed {
        /// Events that occurred while disconnected
        missed_events: Vec<ServerMessage>,
    },

    /// Heartbeat response.
    ///
    /// Echoes back for latency calculation.
    HeartbeatAck {
        /// Server timestamp when heartbeat was received
        server_time: u64,
    },

    /// Session is invalid or expired.
    ///
    /// Client should re-authenticate.
    InvalidSession { reason: String },

    // ========================================================================
    // Lobby State Messages
    // ========================================================================
    /// Sent when successfully joining a lobby.
    LobbyJoined {
        lobby_id: String,
        /// 6-char code for custom lobbies
        #[serde(skip_serializing_if = "Option::is_none")]
        lobby_code: Option<String>,
        /// Full lobby state
        lobby: LobbySnapshot,
    },

    /// Full lobby state snapshot.
    ///
    /// Sent on initial join or when delta sync fails.
    LobbySnapshot { lobby: LobbySnapshot },

    /// Incremental lobby state update.
    ///
    /// More efficient than full snapshots for small changes.
    LobbyDelta { changes: Vec<LobbyChange> },

    /// Confirmation of leaving lobby.
    LobbyLeft,

    /// Custom lobby was created successfully.
    CustomLobbyCreated {
        lobby_id: String,
        lobby_code: String,
    },

    // ========================================================================
    // Game Lifecycle Messages
    // ========================================================================
    /// A new game has started.
    ///
    /// Contains initial game state for all participants.
    GameStarted {
        game_id: String,
        grid: Grid,
        players: Vec<GamePlayerInfo>,
        /// Your turn order (0-indexed)
        your_turn_order: u8,
        /// Who goes first
        current_turn: String,
        round: u8,
        max_rounds: u8,
        /// Turn time limit in seconds (if configured)
        #[serde(skip_serializing_if = "Option::is_none")]
        turn_time_limit: Option<u32>,
    },

    /// Full game state snapshot.
    ///
    /// Sent when joining as spectator or when delta sync fails.
    GameSnapshot { game: GameSnapshot },

    /// Incremental game state update.
    GameDelta { changes: Vec<GameChange> },

    /// Game has ended normally.
    GameOver {
        game_id: String,
        /// Final scores, sorted by rank
        final_scores: Vec<ScoreInfo>,
        /// Winner's user ID
        winner_id: String,
        /// Whether it was a draw
        #[serde(default)]
        is_draw: bool,
    },

    /// Game was cancelled (not enough players, host left, etc.).
    GameCancelled { game_id: String, reason: String },

    // ========================================================================
    // Game Event Messages
    // ========================================================================
    /// A player joined the lobby.
    PlayerJoined { player: LobbyPlayerInfo },

    /// A player left the lobby.
    PlayerLeft {
        player_id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        reason: Option<String>,
    },

    /// A player reconnected after disconnection.
    PlayerReconnected { player_id: String },

    /// A player disconnected (may reconnect).
    PlayerDisconnected {
        player_id: String,
        /// Grace period in seconds before they're removed
        grace_period_seconds: u32,
    },

    /// A player's ready state changed.
    PlayerReadyChanged { player_id: String, is_ready: bool },

    /// A word was successfully scored.
    WordScored {
        player_id: String,
        word: String,
        score: i32,
        /// Positions that formed the word
        path: Vec<super::types::Position>,
        /// New total score
        total_score: i32,
        /// Gems earned from this word
        gems_earned: i32,
        /// New gem total
        total_gems: i32,
        /// Updated grid (letters replaced)
        new_grid: Grid,
    },

    /// Turn changed to another player.
    TurnChanged {
        player_id: String,
        round: u8,
        /// Time remaining for this turn (if timer active)
        #[serde(skip_serializing_if = "Option::is_none")]
        time_remaining: Option<u32>,
    },

    /// A player passed their turn.
    TurnPassed { player_id: String },

    /// Round number changed.
    RoundChanged { round: u8, max_rounds: u8 },

    /// Board was shuffled.
    BoardShuffled {
        player_id: String,
        new_grid: Grid,
        gems_spent: i32,
    },

    /// A tile was swapped.
    TileSwapped {
        player_id: String,
        row: usize,
        col: usize,
        old_letter: char,
        new_letter: char,
        gems_spent: i32,
    },

    /// Player entered swap mode (for animation).
    SwapModeEntered { player_id: String },

    /// Player exited swap mode.
    SwapModeExited { player_id: String },

    // ========================================================================
    // Spectator Messages
    // ========================================================================
    /// Successfully joined as spectator.
    SpectatorJoined {
        game_id: String,
        /// Full game state
        game: GameSnapshot,
    },

    /// A new spectator joined (broadcast to others).
    SpectatorAdded { spectator: SpectatorInfo },

    /// A spectator left.
    SpectatorRemoved { spectator_id: String },

    /// Spectator joined as player.
    SpectatorBecamePlayer { player_id: String, username: String },

    // ========================================================================
    // Live Update Messages
    // ========================================================================
    /// Another player's tile selection (for live preview).
    SelectionUpdate {
        player_id: String,
        game_id: String,
        positions: Vec<super::types::Position>,
    },

    // ========================================================================
    // Timer Vote Messages
    // ========================================================================
    /// Timer vote state changed.
    TimerVoteUpdate { state: TimerVoteState },

    /// Turn timer started (vote passed).
    TurnTimerStarted {
        target_player_id: String,
        seconds: u32,
    },

    /// Turn timer expired - player auto-passed.
    TurnTimerExpired { player_id: String },

    // ========================================================================
    // Queue Messages (Legacy)
    // ========================================================================
    /// Player joined the game queue.
    QueueJoined { position: i32, total_in_queue: i32 },

    /// Queue position updated.
    QueueUpdate { position: i32, total_in_queue: i32 },

    /// Left the queue.
    QueueLeft,

    // ========================================================================
    // Admin Messages
    // ========================================================================
    /// Response to admin game list request.
    AdminGamesList { games: Vec<AdminGameInfo> },

    /// Game was deleted by admin.
    AdminGameDeleted { game_id: String },

    // ========================================================================
    // Legacy Compatibility Messages
    // ========================================================================
    /// Generic state update (legacy format).
    ///
    /// Used for backward compatibility with existing frontend.
    #[serde(rename = "game_state")]
    GameStateUpdate {
        game_id: String,
        state: String,
        grid: Grid,
        players: Vec<PlayerInfo>,
        current_turn: String,
        round: i32,
        max_rounds: i32,
        used_words: Vec<String>,
        spectators: Vec<SpectatorInfo>,
        timer_vote_state: TimerVoteState,
    },

    /// Lobby state update (legacy format).
    #[serde(rename = "lobby_state")]
    LobbyStateUpdate {
        lobby_id: String,
        players: Vec<LobbyPlayerInfo>,
        games: Vec<LobbyGameInfo>,
    },

    // ========================================================================
    // Error Messages
    // ========================================================================
    /// Error response.
    Error {
        code: ErrorCode,
        message: String,
        /// Additional context (e.g., which field was invalid)
        #[serde(skip_serializing_if = "Option::is_none")]
        details: Option<serde_json::Value>,
    },
}

impl ServerMessage {
    /// Create an error message from an error code.
    pub fn error(code: ErrorCode) -> Self {
        Self::Error {
            message: code.message().to_string(),
            code,
            details: None,
        }
    }

    /// Create an error message with custom message.
    pub fn error_with_message(code: ErrorCode, message: impl Into<String>) -> Self {
        Self::Error {
            code,
            message: message.into(),
            details: None,
        }
    }

    /// Create an error message with details.
    pub fn error_with_details(
        code: ErrorCode,
        message: impl Into<String>,
        details: serde_json::Value,
    ) -> Self {
        Self::Error {
            code,
            message: message.into(),
            details: Some(details),
        }
    }

    /// Get the message type as a string (for logging/debugging).
    pub fn message_type(&self) -> &'static str {
        match self {
            Self::Hello { .. } => "hello",
            Self::Ready { .. } => "ready",
            Self::Resumed { .. } => "resumed",
            Self::HeartbeatAck { .. } => "heartbeat_ack",
            Self::InvalidSession { .. } => "invalid_session",
            Self::LobbyJoined { .. } => "lobby_joined",
            Self::LobbySnapshot { .. } => "lobby_snapshot",
            Self::LobbyDelta { .. } => "lobby_delta",
            Self::LobbyLeft => "lobby_left",
            Self::CustomLobbyCreated { .. } => "custom_lobby_created",
            Self::GameStarted { .. } => "game_started",
            Self::GameSnapshot { .. } => "game_snapshot",
            Self::GameDelta { .. } => "game_delta",
            Self::GameOver { .. } => "game_over",
            Self::GameCancelled { .. } => "game_cancelled",
            Self::PlayerJoined { .. } => "player_joined",
            Self::PlayerLeft { .. } => "player_left",
            Self::PlayerReconnected { .. } => "player_reconnected",
            Self::PlayerDisconnected { .. } => "player_disconnected",
            Self::PlayerReadyChanged { .. } => "player_ready_changed",
            Self::WordScored { .. } => "word_scored",
            Self::TurnChanged { .. } => "turn_changed",
            Self::TurnPassed { .. } => "turn_passed",
            Self::RoundChanged { .. } => "round_changed",
            Self::BoardShuffled { .. } => "board_shuffled",
            Self::TileSwapped { .. } => "tile_swapped",
            Self::SwapModeEntered { .. } => "swap_mode_entered",
            Self::SwapModeExited { .. } => "swap_mode_exited",
            Self::SpectatorJoined { .. } => "spectator_joined",
            Self::SpectatorAdded { .. } => "spectator_added",
            Self::SpectatorRemoved { .. } => "spectator_removed",
            Self::SpectatorBecamePlayer { .. } => "spectator_became_player",
            Self::SelectionUpdate { .. } => "selection_update",
            Self::TimerVoteUpdate { .. } => "timer_vote_update",
            Self::TurnTimerStarted { .. } => "turn_timer_started",
            Self::TurnTimerExpired { .. } => "turn_timer_expired",
            Self::QueueJoined { .. } => "queue_joined",
            Self::QueueUpdate { .. } => "queue_update",
            Self::QueueLeft => "queue_left",
            Self::AdminGamesList { .. } => "admin_games_list",
            Self::AdminGameDeleted { .. } => "admin_game_deleted",
            Self::GameStateUpdate { .. } => "game_state",
            Self::LobbyStateUpdate { .. } => "lobby_state",
            Self::Error { .. } => "error",
        }
    }

    /// Check if this is an error message.
    pub fn is_error(&self) -> bool {
        matches!(self, Self::Error { .. })
    }

    /// Check if this message should be stored for reconnection replay.
    ///
    /// Some messages are transient and don't need to be replayed.
    pub fn should_store_for_replay(&self) -> bool {
        !matches!(
            self,
            Self::Hello { .. }
                | Self::HeartbeatAck { .. }
                | Self::SelectionUpdate { .. }
                | Self::TimerVoteUpdate {
                    state: TimerVoteState::Idle
                }
        )
    }
}

// ============================================================================
// Snapshot Types
// ============================================================================

/// Complete lobby state snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LobbySnapshot {
    pub lobby_id: String,
    pub lobby_type: LobbyType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lobby_code: Option<String>,
    pub players: Vec<LobbyPlayerInfo>,
    pub games: Vec<LobbyGameInfo>,
    /// User ID of the lobby host
    #[serde(skip_serializing_if = "Option::is_none")]
    pub host_id: Option<String>,
    /// Maximum players allowed
    #[serde(default = "default_max_players")]
    pub max_players: u8,
}

fn default_max_players() -> u8 {
    6
}

/// Complete game state snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameSnapshot {
    pub game_id: String,
    pub state: GameState,
    pub grid: Grid,
    pub players: Vec<PlayerInfo>,
    pub spectators: Vec<SpectatorInfo>,
    pub current_turn: String,
    pub round: u8,
    pub max_rounds: u8,
    pub used_words: Vec<String>,
    pub timer_vote_state: TimerVoteState,
    /// Your player info (for the receiving client)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub your_player: Option<PlayerInfo>,
    /// Time remaining in current turn (if timer active)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub turn_time_remaining: Option<u32>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::types;

    #[test]
    fn test_heartbeat_ack_serialization() {
        let msg = ServerMessage::HeartbeatAck {
            server_time: 1701234567890,
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains(r#""type":"heartbeat_ack""#));
        assert!(json.contains(r#""server_time":1701234567890"#));
    }

    #[test]
    fn test_error_message_creation() {
        let msg = ServerMessage::error(ErrorCode::NotYourTurn);
        match msg {
            ServerMessage::Error { code, message, .. } => {
                assert_eq!(code, ErrorCode::NotYourTurn);
                assert_eq!(message, "It's not your turn");
            }
            _ => panic!("Expected error message"),
        }
    }

    #[test]
    fn test_error_with_details() {
        let msg = ServerMessage::error_with_details(
            ErrorCode::InvalidPath,
            "Tiles must be adjacent",
            serde_json::json!({"positions": [[0,0], [2,2]]}),
        );
        match msg {
            ServerMessage::Error { details, .. } => {
                assert!(details.is_some());
            }
            _ => panic!("Expected error message"),
        }
    }

    #[test]
    fn test_player_joined_serialization() {
        let msg = ServerMessage::PlayerJoined {
            player: LobbyPlayerInfo {
                user_id: "123".to_string(),
                username: "TestPlayer".to_string(),
                avatar_url: None,
                is_ready: false,
            },
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains(r#""type":"player_joined""#));
        assert!(json.contains(r#""user_id":"123""#));
    }

    #[test]
    fn test_game_state_update_legacy() {
        // Verify legacy format still works
        let json = r#"{"type":"game_state","game_id":"abc","state":"in_progress","grid":[],"players":[],"current_turn":"123","round":1,"max_rounds":5,"used_words":[],"spectators":[],"timer_vote_state":{"status":"idle"}}"#;
        let msg: ServerMessage = serde_json::from_str(json).unwrap();
        assert!(matches!(msg, ServerMessage::GameStateUpdate { .. }));
    }

    #[test]
    fn test_should_store_for_replay() {
        assert!(!ServerMessage::HeartbeatAck { server_time: 0 }.should_store_for_replay());
        assert!(ServerMessage::PlayerJoined {
            player: LobbyPlayerInfo {
                user_id: "1".into(),
                username: "x".into(),
                avatar_url: None,
                is_ready: false,
            }
        }
        .should_store_for_replay());
    }

    #[test]
    fn test_message_type() {
        assert_eq!(
            ServerMessage::HeartbeatAck { server_time: 0 }.message_type(),
            "heartbeat_ack"
        );
        assert_eq!(
            ServerMessage::error(ErrorCode::NotYourTurn).message_type(),
            "error"
        );
    }

    #[test]
    fn test_selection_update_serialization() {
        let msg = ServerMessage::SelectionUpdate {
            player_id: "player1".to_string(),
            game_id: "game1".to_string(),
            positions: vec![
                types::Position { row: 0, col: 0 },
                types::Position { row: 0, col: 1 },
            ],
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains(r#""type":"selection_update""#));
        assert!(json.contains(r#""player_id":"player1""#));
        assert!(json.contains(r#""game_id":"game1""#));
    }
}
