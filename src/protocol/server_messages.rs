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

use crate::protocol::GameType;

use super::types::{
    AdminGameInfo, ErrorCode, GameChange, GamePlayerInfo, GameSnapshot, Grid, LobbyChange,
    LobbyGameInfo, LobbyPlayerInfo, LobbyType, PlayerInfo, RematchCountdownState, ScoreInfo,
    SpectatorInfo, TimerVoteState,
};

/// Messages sent from server to client.
#[serde_with::serde_as]
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
        #[serde_as(as = "serde_with::DisplayFromStr")]
        player_id: i64,
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

    /// The lobby host has changed.
    HostChanged {
        #[serde_as(as = "serde_with::DisplayFromStr")]
        new_host_id: i64,
        lobby_id: String,
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
        #[serde_as(as = "serde_with::DisplayFromStr")]
        current_turn: i64,
        round: u8,
        max_rounds: u8,
        /// Turn time limit in seconds (if configured)
        #[serde(skip_serializing_if = "Option::is_none")]
        turn_time_limit: Option<u32>,
    },

    /// Full game state snapshot.
    ///
    /// Sent when joining as spectator or when delta sync fails.
    GameSnapshot { game_id: String, game: GameSnapshot },

    /// Incremental game state update.
    GameDelta {
        game_id: String,
        changes: Vec<GameChange>,
    },

    /// Game has ended normally.
    GameOver {
        game_id: String,
        /// Final scores, sorted by rank
        final_scores: Vec<ScoreInfo>,
        /// Winner's user ID
        #[serde_as(as = "serde_with::DisplayFromStr")]
        winner_id: i64,
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
        #[serde_as(as = "serde_with::DisplayFromStr")]
        player_id: i64,
        #[serde(skip_serializing_if = "Option::is_none")]
        reason: Option<String>,
    },

    /// A player reconnected after disconnection.
    PlayerReconnected {
        #[serde_as(as = "serde_with::DisplayFromStr")]
        player_id: i64,
    },

    /// A player disconnected (may reconnect).
    PlayerDisconnected {
        #[serde(skip_serializing_if = "Option::is_none")]
        game_id: Option<String>,
        #[serde_as(as = "serde_with::DisplayFromStr")]
        player_id: i64,
        /// Grace period in seconds before they're removed
        grace_period_seconds: u32,
    },

    /// A player's ready state changed.
    PlayerReadyChanged {
        #[serde_as(as = "serde_with::DisplayFromStr")]
        player_id: i64,
        is_ready: bool,
    },

    /// A word was successfully scored.
    WordScored {
        #[serde_as(as = "serde_with::DisplayFromStr")]
        player_id: i64,
        game_id: String,
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
        #[serde_as(as = "serde_with::DisplayFromStr")]
        player_id: i64,
        game_id: String,
        round: u8,
        /// Time remaining for this turn (if timer active)
        #[serde(skip_serializing_if = "Option::is_none")]
        time_remaining: Option<u32>,
    },

    /// A player passed their turn.
    TurnPassed {
        #[serde_as(as = "serde_with::DisplayFromStr")]
        player_id: i64,
        game_id: String,
    },

    /// Round number changed.
    RoundChanged {
        game_id: String,
        round: u8,
        max_rounds: u8,
    },

    /// Board was shuffled.
    BoardShuffled {
        #[serde_as(as = "serde_with::DisplayFromStr")]
        player_id: i64,
        game_id: String,
        new_grid: Grid,
        gems_spent: i32,
        /// Player's remaining gems after shuffle
        total_gems: i32,
    },

    /// A tile was swapped.
    TileSwapped {
        #[serde_as(as = "serde_with::DisplayFromStr")]
        player_id: i64,
        game_id: String,
        row: usize,
        col: usize,
        old_letter: char,
        new_letter: char,
        gems_spent: i32,
        /// Player's remaining gems after swap
        total_gems: i32,
    },

    /// Player entered swap mode (for animation).
    SwapModeEntered {
        #[serde_as(as = "serde_with::DisplayFromStr")]
        player_id: i64,
        game_id: String,
    },

    /// Player exited swap mode.
    SwapModeExited {
        #[serde_as(as = "serde_with::DisplayFromStr")]
        player_id: i64,
        game_id: String,
    },

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
    SpectatorAdded {
        spectator: SpectatorInfo,
        game_id: String,
    },

    /// A spectator left.
    SpectatorRemoved {
        #[serde_as(as = "serde_with::DisplayFromStr")]
        spectator_id: i64,
        game_id: String,
    },

    /// Spectator joined as player.
    SpectatorBecamePlayer {
        #[serde_as(as = "serde_with::DisplayFromStr")]
        player_id: i64,
        username: String,
        game_id: String,
    },

    /// Confirmation of leaving spectator mode.
    SpectatorLeft,

    // ========================================================================
    // Live Update Messages
    // ========================================================================
    /// Another player's tile selection (for live preview).
    SelectionUpdate {
        #[serde_as(as = "serde_with::DisplayFromStr")]
        player_id: i64,
        game_id: String,
        positions: Vec<super::types::Position>,
    },

    // ========================================================================
    // Timer Vote Messages
    // ========================================================================
    /// Timer vote state changed.
    TimerVoteUpdate {
        state: TimerVoteState,
        game_id: String,
    },

    /// Turn timer started (vote passed).
    TurnTimerStarted {
        #[serde_as(as = "serde_with::DisplayFromStr")]
        target_player_id: i64,
        game_id: String,
        seconds: u32,
    },

    /// Turn timer expired - player auto-passed.
    TurnTimerExpired {
        #[serde_as(as = "serde_with::DisplayFromStr")]
        player_id: i64,
        game_id: String,
    },

    // ========================================================================
    // Rematch Countdown Messages
    // ========================================================================
    /// Rematch countdown state update.
    ///
    /// Sent to all players on the results screen after a game ends.
    RematchCountdownUpdate {
        /// Current countdown state
        state: RematchCountdownState,
        /// The game that just ended
        previous_game_id: String,
    },

    /// A player opted out of rematch queue.
    ///
    /// Broadcast to remaining players so they can update the player list.
    PlayerLeftRematch {
        #[serde_as(as = "serde_with::DisplayFromStr")]
        player_id: i64,
        /// The game they left from
        previous_game_id: String,
    },

    /// Rematch is starting (sent right before GameStarted).
    ///
    /// Allows frontend to show "Starting..." before the new game begins.
    RematchStarting {
        /// Who triggered the early start (None if countdown expired naturally)
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde_as(as = "Option<serde_with::DisplayFromStr>")]
        triggered_by: Option<i64>,
        /// The previous game that ended
        previous_game_id: String,
    },

    // ========================================================================
    // Queue
    // ========================================================================
    //
    PlayerQueueChanged {
        #[serde_as(as = "serde_with::DisplayFromStr")]
        player_id: i64,
        old_queue: Option<GameType>,
        new_queue: Option<GameType>,
    },

    // ========================================================================
    // Queue Messages (Legacy)
    // ========================================================================
    /// Player joined the game queue.
    QueueJoined {
        position: i32,
        total_in_queue: i32,
        game_id: String,
    },

    /// Queue position updated.
    QueueUpdate {
        position: i32,
        total_in_queue: i32,
        game_id: String,
    },

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
        current_turn: i64,
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
    #[must_use]
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
    #[must_use]
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
            Self::HostChanged { .. } => "host_changed",
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
            Self::PlayerQueueChanged { .. } => "player_queue_changed",
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
            Self::SpectatorLeft => "spectator_left",
            Self::SpectatorBecamePlayer { .. } => "spectator_became_player",
            Self::SelectionUpdate { .. } => "selection_update",
            Self::TimerVoteUpdate { .. } => "timer_vote_update",
            Self::TurnTimerStarted { .. } => "turn_timer_started",
            Self::TurnTimerExpired { .. } => "turn_timer_expired",
            Self::RematchCountdownUpdate { .. } => "rematch_countdown_update",
            Self::PlayerLeftRematch { .. } => "player_left_rematch",
            Self::RematchStarting { .. } => "rematch_starting",
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
    #[must_use]
    pub fn is_error(&self) -> bool {
        matches!(self, Self::Error { .. })
    }

    /// Check if this message should be stored for reconnection replay.
    ///
    /// Some messages are transient and don't need to be replayed.
    #[must_use]
    pub fn should_store_for_replay(&self) -> bool {
        !matches!(
            self,
            Self::Hello { .. }
                | Self::HeartbeatAck { .. }
                | Self::SelectionUpdate { .. }
                | Self::TimerVoteUpdate {
                    state: TimerVoteState::Idle,
                    ..
                }
                | Self::PlayerQueueChanged { .. }
                | Self::TurnTimerStarted { .. }
                | Self::TurnTimerExpired { .. }
        )
    }
}

/// Convert a `ServerMessage` to a `serde_json::Value`.
impl From<ServerMessage> for serde_json::Value {
    fn from(msg: ServerMessage) -> Self {
        serde_json::to_value(msg).unwrap()
    }
}

/// Convert a `serde_json::Value` to a `ServerMessage`.
impl TryFrom<serde_json::Value> for ServerMessage {
    type Error = serde_json::Error;

    fn try_from(
        value: serde_json::Value,
    ) -> Result<Self, <ServerMessage as TryFrom<serde_json::Value>>::Error> {
        serde_json::from_value(value)
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
    pub host_id: Option<i64>,
    /// Maximum players allowed
    #[serde(default = "default_max_players")]
    pub max_players: u8,
}

fn default_max_players() -> u8 {
    6
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::{types, GameType};

    #[test]
    fn test_server_message_into_json() {
        let msg = ServerMessage::BoardShuffled {
            player_id: 1_234_567_890,
            game_id: "1234567890".to_string(),
            new_grid: Grid::new(),
            gems_spent: 0,
            total_gems: 0,
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains(r#""type":"board_shuffled""#));
        assert!(json.contains(r#""player_id":"1234567890""#));
        assert!(json.contains(r#""game_id":"1234567890""#));
        assert!(json.contains(r#""new_grid":[]"#));
        assert!(json.contains(r#""gems_spent":0"#));
        assert!(json.contains(r#""total_gems":0"#));
    }

    #[test]
    fn test_heartbeat_ack_serialization() {
        let msg = ServerMessage::HeartbeatAck {
            server_time: 1_701_234_567_890,
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
                user_id: 123,
                username: "TestPlayer".to_string(),
                avatar_url: None,
                is_ready: false,
                current_queue: None,
            },
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains(r#""type":"player_joined""#));
        assert!(json.contains(r#""user_id":"123""#));
    }

    #[test]
    fn test_game_state_update_legacy() {
        // Verify legacy format still works
        let json = r#"{"type":"game_state","game_id":"abc","state":"in_progress","grid":[],"players":[],"current_turn":123,"round":1,"max_rounds":5,"used_words":[],"spectators":[],"timer_vote_state":{"status":"idle"}}"#;
        let msg: ServerMessage = serde_json::from_str(json).unwrap();
        assert!(matches!(msg, ServerMessage::GameStateUpdate { .. }));
    }

    #[test]
    fn test_should_store_for_replay() {
        assert!(!ServerMessage::HeartbeatAck { server_time: 0 }.should_store_for_replay());
        assert!(ServerMessage::PlayerJoined {
            player: LobbyPlayerInfo {
                user_id: 1,
                username: "x".into(),
                avatar_url: None,
                is_ready: false,
                current_queue: None,
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
            player_id: 11,
            game_id: "game1".to_string(),
            positions: vec![
                types::Position { row: 0, col: 0 },
                types::Position { row: 0, col: 1 },
            ],
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains(r#""type":"selection_update""#));
        assert!(json.contains(r#""player_id":"11""#));
        assert!(json.contains(r#""game_id":"game1""#));
    }

    #[test]
    fn test_player_queue_changed_serialization() {
        let msg = ServerMessage::PlayerQueueChanged {
            player_id: 123,
            old_queue: Some(GameType::Open),
            new_queue: Some(GameType::Adventure),
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains(r#""type":"player_queue_changed""#));
        assert!(json.contains(r#""player_id":"123""#));
        assert!(json.contains(r#""old_queue":"open""#));
        assert!(json.contains(r#""new_queue":"adventure""#));
    }
}
