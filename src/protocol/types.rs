//! Shared types used in protocol messages.
//!
//! These types are used by both client and server messages and represent
//! the core data structures of the game.

use serde::{Deserialize, Serialize};

// Re-export model types that are part of the protocol
// These would come from crate::models in the actual backend
// For now, we define them here for the protocol module to be self-contained

/// Grid position (row, column).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Position {
    pub row: usize,
    pub col: usize,
}

/// Letter multiplier on a grid cell.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Multiplier {
    DoubleLetter,
    TripleLetter,
    DoubleWord,
}

/// A single cell in the game grid.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GridCell {
    pub letter: char,
    pub value: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub multiplier: Option<Multiplier>,
    #[serde(default)]
    pub has_gem: bool,
}

/// The 5x5 game grid.
pub type Grid = Vec<Vec<GridCell>>;

/// Game mode variants.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GameMode {
    Solo,
    #[default]
    Multiplayer,
    Adventure,
}

// ============================================================================
// Lobby Types
// ============================================================================

/// Type of lobby - determines how players join.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LobbyType {
    /// Lobby tied to a specific Discord channel
    Channel,
    /// Custom lobby with a shareable code
    Custom,
}

/// Player information in the lobby (pre-game).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LobbyPlayerInfo {
    /// User ID (string to preserve JS number precision)
    pub user_id: String,
    pub username: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_url: Option<String>,
    /// Whether the player has marked themselves ready
    #[serde(default)]
    pub is_ready: bool,
}

/// Summary of a game visible from the lobby.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameSummary {
    pub game_id: String,
    pub state: GameState,
    pub current_round: u8,
    pub max_rounds: u8,
    pub player_count: u8,
    pub spectator_count: u8,
}

/// High-level game state (not the full game data).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GameState {
    /// Waiting for players / not started
    Idle,
    /// Players are queueing
    Queueing,
    /// Game is starting (countdown)
    Starting,
    /// Game is in progress
    InProgress,
    /// Game has ended
    Finished,
    /// Game was cancelled
    Cancelled,
}

// ============================================================================
// In-Game Types
// ============================================================================

/// Player information during a game.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerInfo {
    /// User ID (string to preserve JS number precision)
    pub user_id: String,
    pub username: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_url: Option<String>,
    pub score: i32,
    /// Gems collected (0-10, used for powers)
    #[serde(default)]
    pub gems: i32,
    /// Team number for team modes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team: Option<i32>,
    /// Whether the player is currently connected
    #[serde(default = "default_true")]
    pub is_connected: bool,
}

fn default_true() -> bool {
    true
}

/// Player info specifically for GameStarted message (includes turn order).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GamePlayerInfo {
    pub user_id: String,
    pub username: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_url: Option<String>,
    pub turn_order: u8,
    pub score: i32,
    pub gems: i32,
    pub is_connected: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team: Option<i32>,
}

/// Spectator information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpectatorInfo {
    pub user_id: String,
    pub username: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_url: Option<String>,
}

/// Score information for results/leaderboards.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreInfo {
    pub user_id: String,
    pub username: String,
    pub score: i32,
}

/// Player info in lobby game list (simplified).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LobbyGamePlayerInfo {
    pub user_id: String,
    pub username: String,
    pub score: i32,
}

/// Game info as shown in lobby games list.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LobbyGameInfo {
    pub game_id: String,
    pub current_round: i32,
    pub max_rounds: i32,
    pub players: Vec<LobbyGamePlayerInfo>,
}

// ============================================================================
// Snapshot Types
// ============================================================================

/// Snapshot of a player's state within a game.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GamePlayerSnapshot {
    pub user_id: String,
    pub username: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_url: Option<String>,
    pub score: i32,
    pub gems: i32,
    pub is_connected: bool,
}

/// Snapshot of a spectator's state within a game.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameSpectatorSnapshot {
    pub user_id: String,
    pub username: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_url: Option<String>,
}

/// Complete snapshot of the game state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameSnapshot {
    pub game_id: String,
    pub state: GameState,
    pub grid: Grid,
    pub players: Vec<GamePlayerSnapshot>,
    pub spectators: Vec<GameSpectatorSnapshot>,
    pub current_turn: Option<String>,
    pub round: i32,
    pub max_rounds: i32,
    pub used_words: Vec<String>,
}

// ============================================================================
// Timer Vote Types
// ============================================================================

/// State of the timer vote system.
///
/// The timer vote allows players to collectively vote to start a turn timer
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum TimerVoteState {
    /// No vote in progress, button is idle
    #[default]
    Idle,

    /// Vote is in progress
    VoteInProgress {
        /// User ID of who initiated the vote
        initiator_id: String,
        /// User IDs of players who have voted yes
        voters: Vec<String>,
        /// Total votes needed to pass
        votes_needed: u32,
        /// When the vote expires
        expires_at: chrono::DateTime<chrono::Utc>,
    },

    /// Timer is actively counting down
    TimerActive {
        /// When the timer expires
        expires_at: chrono::DateTime<chrono::Utc>,
        /// Target player ID (user ID)
        target_player_id: String,
    },

    /// Vote failed, in cooldown before another can start
    Cooldown {
        /// When the cooldown expires
        expires_at: chrono::DateTime<chrono::Utc>,
    },

    /// Feature disabled (not enough players)
    Disabled,
}

// ============================================================================
// Delta Types (for efficient state updates)
// ============================================================================

/// Changes to lobby state (for delta updates instead of full snapshots).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "change_type", rename_all = "snake_case")]
pub enum LobbyChange {
    /// A player joined the lobby
    PlayerJoined { player: LobbyPlayerInfo },

    /// A player left the lobby
    PlayerLeft {
        player_id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        reason: Option<String>,
    },

    /// A player's ready state changed
    PlayerReadyChanged { player_id: String, is_ready: bool },

    /// A player's connection state changed
    PlayerConnectionChanged {
        player_id: String,
        is_connected: bool,
    },

    /// A game's state changed
    GameStateChanged { game_id: String, state: GameState },

    /// Queue count updated for a game
    QueueUpdated { game_id: String, queue_count: u32 },

    /// Host changed
    HostChanged { new_host_id: String },
}

/// Changes to game state (for delta updates).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "change_type", rename_all = "snake_case")]
pub enum GameChange {
    /// Grid was updated (after word submission)
    GridUpdated {
        grid: Grid,
        /// Positions that were replaced
        #[serde(skip_serializing_if = "Option::is_none")]
        replaced_positions: Option<Vec<Position>>,
    },

    /// A player's score changed
    ScoreUpdated {
        player_id: String,
        score: i32,
        gems: i32,
    },

    /// Turn changed to another player
    TurnChanged { player_id: String },

    /// Round number changed
    RoundChanged { round: u8 },

    /// A word was added to used words
    WordUsed { word: String },

    /// A spectator joined the game
    SpectatorJoined { spectator: SpectatorInfo },

    /// A spectator left the game
    SpectatorLeft { spectator_id: String },

    /// A player's connection state changed
    PlayerConnectionChanged {
        player_id: String,
        is_connected: bool,
    },
}

// ============================================================================
// Admin Types
// ============================================================================

/// Admin game info (for admin panel).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminGameInfo {
    pub game_id: String,
    pub state: GameState,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub players: Vec<String>,
}

// ============================================================================
// Error Types
// ============================================================================

/// Standard error codes for protocol errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    // Connection errors
    NotAuthenticated,
    SessionExpired,
    InvalidSession,

    // Lobby errors
    LobbyNotFound,
    LobbyFull,
    NotInLobby,
    AlreadyInLobby,

    // Game errors
    GameNotFound,
    GameInProgress,
    GameNotActive,
    NotInGame,
    AlreadyInGame,

    // Turn errors
    NotYourTurn,
    InvalidAction,
    ActionTimeout,

    // Word submission errors
    InvalidPath,
    PathTooShort,
    WordNotInDictionary,
    WordAlreadyUsed,

    // Permission errors
    NotHost,
    NotEnoughPlayers,
    TooManyPlayers,

    // Resource errors
    InsufficientGems,

    // Rate limiting
    TooManyRequests,
    MessageTooLarge,

    // Generic
    InvalidRequest,
    InternalError,
}

impl ErrorCode {
    /// Get a human-readable message for this error code.
    pub fn message(&self) -> &'static str {
        match self {
            Self::NotAuthenticated => "Not authenticated",
            Self::SessionExpired => "Session expired",
            Self::InvalidSession => "Invalid session",
            Self::LobbyNotFound => "Lobby not found",
            Self::LobbyFull => "Lobby is full",
            Self::NotInLobby => "You must be in a lobby",
            Self::AlreadyInLobby => "Already in a lobby",
            Self::GameNotFound => "Game not found",
            Self::GameInProgress => "A game is already in progress",
            Self::GameNotActive => "Game is not active",
            Self::NotInGame => "You are not in this game",
            Self::AlreadyInGame => "You are already in this game",
            Self::NotYourTurn => "It's not your turn",
            Self::InvalidAction => "Invalid action",
            Self::ActionTimeout => "Action timed out",
            Self::InvalidPath => "Invalid path - letters must be adjacent",
            Self::PathTooShort => "Word must be at least 3 letters",
            Self::WordNotInDictionary => "Word not found in dictionary",
            Self::WordAlreadyUsed => "Word has already been used",
            Self::NotHost => "Only the host can do this",
            Self::NotEnoughPlayers => "Not enough players",
            Self::TooManyPlayers => "Too many players",
            Self::InsufficientGems => "Not enough gems",
            Self::TooManyRequests => "Too many requests",
            Self::MessageTooLarge => "Message too large",
            Self::InvalidRequest => "Invalid request",
            Self::InternalError => "Internal server error",
        }
    }
}

impl std::fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_serialization() {
        let pos = Position { row: 2, col: 3 };
        let json = serde_json::to_string(&pos).unwrap();
        assert_eq!(json, r#"{"row":2,"col":3}"#);
    }

    #[test]
    fn test_multiplier_serialization() {
        assert_eq!(
            serde_json::to_string(&Multiplier::DoubleLetter).unwrap(),
            r#""double_letter""#
        );
        assert_eq!(
            serde_json::to_string(&Multiplier::DoubleWord).unwrap(),
            r#""double_word""#
        );
    }

    #[test]
    fn test_timer_vote_state_serialization() {
        let idle = TimerVoteState::Idle;
        let json = serde_json::to_string(&idle).unwrap();
        assert!(json.contains(r#""status":"idle""#));

        let now = chrono::Utc::now();
        let vote = TimerVoteState::VoteInProgress {
            initiator_id: "123".to_string(),
            voters: vec!["456".to_string()],
            votes_needed: 2,
            expires_at: now,
        };
        let json = serde_json::to_string(&vote).unwrap();
        assert!(json.contains(r#""status":"vote_in_progress""#));
        assert!(json.contains(r#""initiator_id":"123""#));
        assert!(json.contains(r#""expires_at""#));

        let active = TimerVoteState::TimerActive {
            expires_at: now,
            target_player_id: "789".to_string(),
        };
        let json = serde_json::to_string(&active).unwrap();
        assert!(json.contains(r#""status":"timer_active""#));
        assert!(json.contains(r#""target_player_id":"789""#));

        let cooldown = TimerVoteState::Cooldown { expires_at: now };
        let json = serde_json::to_string(&cooldown).unwrap();
        assert!(json.contains(r#""status":"cooldown""#));
        assert!(json.contains(r#""expires_at""#));
    }

    #[test]
    fn test_lobby_change_serialization() {
        let change = LobbyChange::PlayerJoined {
            player: LobbyPlayerInfo {
                user_id: "123".to_string(),
                username: "TestUser".to_string(),
                avatar_url: None,
                is_ready: false,
            },
        };
        let json = serde_json::to_string(&change).unwrap();
        assert!(json.contains(r#""change_type":"player_joined""#));
    }

    #[test]
    fn test_error_code_message() {
        assert_eq!(ErrorCode::NotYourTurn.message(), "It's not your turn");
        assert_eq!(
            ErrorCode::WordNotInDictionary.message(),
            "Word not found in dictionary"
        );
    }

    #[test]
    fn test_game_snapshot_serialization() {
        let player = GamePlayerSnapshot {
            user_id: "p1".to_string(),
            username: "Player1".to_string(),
            avatar_url: None,
            score: 10,
            gems: 5,
            is_connected: true,
        };

        let spectator = GameSpectatorSnapshot {
            user_id: "s1".to_string(),
            username: "Spec1".to_string(),
            avatar_url: Some("http://avatar.url".to_string()),
        };

        let snapshot = GameSnapshot {
            game_id: "game1".to_string(),
            state: GameState::InProgress,
            grid: vec![vec![GridCell {
                letter: 'A',
                value: 1,
                multiplier: None,
                has_gem: false,
            }]],
            players: vec![player],
            spectators: vec![spectator],
            current_turn: Some("p1".to_string()),
            round: 1,
            max_rounds: 3,
            used_words: vec!["WORD".to_string()],
        };

        let json = serde_json::to_string(&snapshot).unwrap();
        assert!(json.contains(r#""game_id":"game1""#));
        assert!(json.contains(r#""players":[{"user_id":"p1""#));
        assert!(json.contains(r#""spectators":[{"user_id":"s1""#));
        assert!(json.contains(r#""current_turn":"p1""#));
    }
}
