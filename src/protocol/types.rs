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
    /// True when this cell is a "hole" — occupies its position in the
    /// layout but can't be selected as part of a word path. Used by
    /// Adventure Mode levels that want asymmetric playable regions.
    /// Hole cells have no letter contribution and are skipped by the
    /// refill pipeline.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub is_hole: bool,
    /// True when this cell has been **poisoned** by a Snake adventure
    /// event. Poisoned letters contribute negative value when used in
    /// a word (sign-flipped — `value` becomes `-value` for scoring),
    /// so players want to avoid them. Poison clears when the cell is
    /// rerolled (word consumption or another event hitting the cell).
    /// Adventure-only; multiplayer sessions never set this.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub is_poisoned: bool,
    /// True when this cell has been **abducted** by a UFO adventure
    /// event. Abducted cells are temporarily unselectable (mirrors
    /// `is_hole` for path validation) and reroll back to normal on a
    /// later round. Adventure-only; multiplayer sessions never set
    /// this.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub is_abducted: bool,
}

/// The 5x5 game grid.
pub type Grid = Vec<Vec<GridCell>>;

/// Adventure Mode random-event kinds. Carried on `ServerMessage::AdventureEvent`.
///
/// The `Unknown` variant catches any future kind an older client doesn't
/// recognize, so adding new kinds server-side is non-breaking. Clients
/// matching on this enum should always handle the `Unknown` arm (e.g.
/// by rendering a generic "Something happened!" toast).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AdventureEventKind {
    /// 3×3 blast centered on a random cell; those cells reroll.
    Bomb,
    /// Poisons a handful of cells for a couple of rounds.
    Snake,
    /// Steals all multipliers currently on the board.
    Ufo,
    /// Forward-compat fallback. Serde decodes any unknown kind string
    /// to this variant via `#[serde(other)]`.
    #[serde(other)]
    Unknown,
}

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

/// Game type for game pools within a lobby.
///
/// Each lobby can have multiple game pools, one per game type. Players wait in pools for matchmaking.
/// Players join a game pool to find matches for that game type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GameType {
    /// Free-for-all open game (default)
    Open,
    /// 2v2 team game
    TwoVTwo,
    /// Adventure/co-op mode
    Adventure,
    /// Configurable unranked custom ("sandbox") game.
    Sandbox,
}

impl std::fmt::Display for GameType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GameType::Open => write!(f, "open"),
            GameType::TwoVTwo => write!(f, "two_v_two"),
            GameType::Adventure => write!(f, "adventure"),
            GameType::Sandbox => write!(f, "sandbox"),
        }
    }
}

/// Player information in the lobby (pre-game).
#[serde_with::serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LobbyPlayerInfo {
    /// User ID
    #[serde_as(as = "serde_with::DisplayFromStr")]
    pub user_id: i64,
    pub username: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_url: Option<String>,
    /// Profile banner URL (Discord CDN)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub banner_url: Option<String>,
    /// Profile accent color (integer representation)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accent_color: Option<i32>,
    /// The player's status within a game pool, if they are in one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_game_pool: Option<GameType>,
    /// Game they are in, if any
    pub active_game_id: Option<String>,
    /// Game they are spectating, if any
    pub spectate_game_id: Option<String>,
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
#[serde_with::serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerInfo {
    /// User ID (string to preserve JS number precision)
    #[serde_as(as = "serde_with::DisplayFromStr")]
    pub user_id: i64,
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

/// Player info specifically for `GameStarted` message (includes turn order).
#[serde_with::serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GamePlayerInfo {
    #[serde_as(as = "serde_with::DisplayFromStr")]
    pub user_id: i64,
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
#[serde_with::serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpectatorInfo {
    #[serde_as(as = "serde_with::DisplayFromStr")]
    pub user_id: i64,
    pub username: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_url: Option<String>,
}

/// Score information for results/leaderboards.
#[serde_with::serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreInfo {
    #[serde_as(as = "serde_with::DisplayFromStr")]
    pub user_id: i64,
    pub username: String,
    pub score: i32,
}

/// Player info in lobby game list (simplified).
#[serde_with::serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LobbyGamePlayerInfo {
    #[serde_as(as = "serde_with::DisplayFromStr")]
    pub user_id: i64,
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

/// Complete snapshot of the game state.
#[serde_with::serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameSnapshot {
    pub game_id: String,
    pub state: GameState,
    pub grid: Grid,
    pub players: Vec<PlayerInfo>,
    pub spectators: Vec<SpectatorInfo>,
    #[serde_as(as = "serde_with::DisplayFromStr")]
    pub current_turn: i64,
    pub round: u8,
    pub max_rounds: u8,
    pub used_words: Vec<String>,
    pub timer_vote_state: TimerVoteState,
    /// Your player info (for the receiving client)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub your_player: Option<PlayerInfo>,
    /// When the turn timer expires (if active)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timer_expiration_time: Option<chrono::DateTime<chrono::Utc>>,
}

// ============================================================================
// Timer Vote Types
// ============================================================================

/// State of the timer vote system.
///
/// The timer vote allows players to collectively vote to start a turn timer
/// on the current player. This prevents indefinite stalling.
#[serde_with::serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum TimerVoteState {
    /// No vote in progress, button is idle
    #[default]
    Idle,

    /// Vote is in progress
    VoteInProgress {
        /// User ID of who initiated the vote
        #[serde_as(as = "serde_with::DisplayFromStr")]
        initiator_id: i64,
        /// User IDs of players who have voted yes
        #[serde_as(as = "Vec<serde_with::DisplayFromStr>")]
        voters: Vec<i64>,
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
        #[serde_as(as = "serde_with::DisplayFromStr")]
        target_player_id: i64,
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
// Rematch Countdown Types
// ============================================================================

/// State of the auto-rematch countdown after a game ends.
///
/// When a game ends, players are automatically re-queued and a countdown begins.
/// Any player can trigger an early start, or opt out to return to the lobby.
#[serde_with::serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum RematchCountdownState {
    /// No rematch countdown active
    #[default]
    Idle,

    /// Countdown is active - new game will start when it reaches 0
    Active {
        /// When the countdown expires (absolute time for clock sync)
        expires_at: chrono::DateTime<chrono::Utc>,
        /// Seconds remaining (for initial state display)
        seconds_remaining: u32,
        /// Player IDs still in the rematch pool
        #[serde_as(as = "Vec<serde_with::DisplayFromStr>")]
        player_ids: Vec<i64>,
        /// The game pool/game type for the rematch
        game_type: GameType,
    },

    /// A player triggered an early start
    Starting {
        /// Who triggered the early start (None if countdown expired naturally)
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde_as(as = "Option<serde_with::DisplayFromStr>")]
        triggered_by: Option<i64>,
    },
}

// ============================================================================
// Delta Types (for efficient state updates)
// ============================================================================

/// Changes to lobby state (for delta updates instead of full snapshots).
#[serde_with::serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "change_type", rename_all = "snake_case")]
pub enum LobbyChange {
    /// A player joined the lobby
    PlayerJoined { player: LobbyPlayerInfo },

    /// A player left the lobby
    PlayerLeft {
        #[serde_as(as = "serde_with::DisplayFromStr")]
        player_id: i64,
        #[serde(skip_serializing_if = "Option::is_none")]
        reason: Option<String>,
    },

    /// A player's connection state changed
    PlayerConnectionChanged {
        #[serde_as(as = "serde_with::DisplayFromStr")]
        player_id: i64,
        is_connected: bool,
    },

    /// A game's state changed
    GameStateChanged { game_id: String, state: GameState },

    /// Pool count updated for a game
    PoolUpdated { game_id: String, pool_count: u32 },

    /// Host changed
    HostChanged { new_host_id: String },
}

/// Changes to game state (for delta updates).
#[serde_with::serde_as]
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
        #[serde_as(as = "serde_with::DisplayFromStr")]
        player_id: i64,
        score: i32,
        gems: i32,
    },

    /// Turn changed to another player
    TurnChanged {
        #[serde_as(as = "serde_with::DisplayFromStr")]
        player_id: i64,
    },

    /// Round number changed
    RoundChanged { round: u8 },

    /// A word was added to used words
    WordUsed { word: String },

    /// A spectator joined the game
    SpectatorJoined { spectator: SpectatorInfo },

    /// A spectator left the game
    SpectatorLeft {
        #[serde_as(as = "serde_with::DisplayFromStr")]
        spectator_id: i64,
    },

    /// A player's connection state changed
    PlayerConnectionChanged {
        #[serde_as(as = "serde_with::DisplayFromStr")]
        player_id: i64,
        is_connected: bool,
    },
}

// ============================================================================
// Admin Types
// ============================================================================

/// Admin game info (for admin panel).
#[serde_with::serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminGameInfo {
    pub game_id: String,
    pub state: GameState,
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[serde_as(as = "Vec<serde_with::DisplayFromStr>")]
    pub players: Vec<i64>,
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
    /// Caller lacks the capability for an entitlement-gated action
    /// (e.g. starting or configuring a custom sandbox game when
    /// `sandbox_enabled` is false for them). Server-authoritative.
    NotAuthorized,

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
    #[must_use]
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
            Self::NotAuthorized => "You are not authorized to do this",
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

// ============================================================================
// Bot Types
// ============================================================================

/// Bot difficulty on the wire. Lowercase to match the backend's
/// `game_players.bot_difficulty` CHECK constraint (`easy|medium|hard`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BotDifficulty {
    Easy,
    Medium,
    Hard,
}

/// One bot seat requested for a custom game.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BotSpec {
    pub difficulty: BotDifficulty,
}

// ============================================================================
// Game Configuration Types
// ============================================================================

/// Random-event config for a custom (sandbox) game. Absent for FFA/Adventure.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct EventConfig {
    /// Kinds eligible to fire (ambient or fire-now). Empty = none.
    #[serde(default)]
    pub enabled_kinds: Vec<AdventureEventKind>,
    /// Ambient per-turn probability one enabled kind fires (0.0–1.0).
    #[serde(default)]
    pub frequency: f64,
}

/// Configuration options for starting a new game.
///
/// These options customize game behavior for a single game session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameConfig {
    /// If true, regenerate the entire board at the start of each round.
    /// Default is false (board persists across rounds).
    #[serde(default)]
    pub regenerate_board_each_round: bool,

    /// Grid size (4 or 5). Determines which dice set is used.
    /// Default is 5 (standard 5x5 Big Boggle-style).
    #[serde(default = "default_grid_size")]
    pub grid_size: u8,

    /// Adventure level ID (1..=50). `Some` signals this session is an
    /// Adventure Mode run; the server pins bot difficulty, grid size, and
    /// target thresholds from the level manifest. `None` = normal session.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub adventure_level: Option<u32>,

    /// Score thresholds for 1/2/3-star completion of the current Adventure
    /// level. Sent from server → client at session start so the UI can
    /// render target chips. Only present when `adventure_level.is_some()`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub level_targets: Option<LevelTargets>,

    /// Number of rounds for a custom game. `None` = server default (5).
    /// Honored only for custom (sandbox) games; ignored for FFA.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub num_rounds: Option<u8>,

    /// Bot seats to add (custom games only). Empty for FFA.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub bots: Vec<BotSpec>,

    /// `Some` marks this session as a custom (unranked) sandbox game.
    /// Mirrors `adventure_level` — both gate the same ranked-skip paths.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom: Option<CustomMeta>,

    /// Random-event config for custom games. `None` for FFA/Adventure.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub events: Option<EventConfig>,
}

fn default_grid_size() -> u8 {
    5
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            regenerate_board_each_round: false,
            grid_size: default_grid_size(),
            adventure_level: None,
            level_targets: None,
            num_rounds: None,
            bots: Vec::new(),
            custom: None,
            events: None,
        }
    }
}

/// Marker payload for a custom (sandbox) game. Empty for now; a struct
/// (not a bare bool) so we can carry custom metadata later without a
/// wire break.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct CustomMeta {}

/// Per-level score thresholds for Adventure Mode star awards.
///
/// Hitting `one_star` completes the level; `two_star` and `three_star`
/// are progressive bonuses. Stars are awarded server-side at session end.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct LevelTargets {
    pub one_star: i32,
    pub two_star: i32,
    pub three_star: i32,
}

// ============================================================================
// Debug State Types (for diagnostics)
// ============================================================================

/// Player info in debug state response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugPlayerInfo {
    pub user_id: i64,
    pub username: String,
}

/// WebSocket connection context in debug state response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugWebsocketContext {
    pub lobby_id: Option<String>,
    pub game_id: Option<String>,
    pub is_spectating: bool,
}

/// Lobby state in debug state response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DebugLobbyState {
    Found {
        lobby_id: String,
        player_in_lobby: bool,
        lobby_player_ids: Vec<i64>,
        active_game_id: Option<String>,
    },
    Error {
        error: String,
    },
}

/// Backend game state in debug state response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DebugBackendGameState {
    Found {
        game_id: String,
        player_in_session_players: bool,
        spectator_in_session: bool,
        session_player_ids: Vec<i64>,
        session_spectator_ids: Vec<i64>,
        lobby_id: String,
    },
    Error {
        error: String,
    },
}

/// Handler game state in debug state response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DebugHandlerGameState {
    Found {
        game_id: String,
        player_in_handler_game: bool,
        handler_player_ids: Vec<i64>,
        current_turn_index: usize,
        round: u8,
        state: String,
    },
    Error {
        error: String,
    },
}

/// Type of news notification. Mirrors the canonical values the backend
/// stores in `news.notification_type` (validated against the
/// `ALLOWED_NOTIFICATION_TYPES` list at the API boundary).
///
/// `#[serde(other)]` keeps old clients functional when the server
/// introduces a new variant — unknown values deserialize to
/// `Unknown` instead of erroring out. Mirrors the
/// `AdventureEventKind::Unknown` pattern already in this crate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NewsNotificationType {
    Maintenance,
    Announcement,
    Update,
    /// Fallback for variants the server adds after this client was built.
    #[serde(other)]
    Unknown,
}

impl NewsNotificationType {
    /// Parse a wire-string into the typed enum. Unknown / unrecognized
    /// strings map to `Unknown` so callers don't have to handle parse
    /// errors — backend's `validate_news_payload` already rejects
    /// unknown values at the API boundary, so this only kicks in
    /// defensively for forward-compat.
    #[must_use]
    pub fn from_wire_str(s: &str) -> Self {
        match s {
            "maintenance" => Self::Maintenance,
            "announcement" => Self::Announcement,
            "update" => Self::Update,
            _ => Self::Unknown,
        }
    }
}

impl From<&str> for NewsNotificationType {
    fn from(s: &str) -> Self {
        Self::from_wire_str(s)
    }
}

impl From<&String> for NewsNotificationType {
    fn from(s: &String) -> Self {
        Self::from_wire_str(s.as_str())
    }
}

/// A news/notification item as sent over the wire (server → client).
///
/// Mirrors the backend's `NewsItem` model but stripped of fields that
/// shouldn't leave the server (`created_by` user id).
///
/// Used in [`ServerMessage::NewsAnnounced`] for live push and is shape-
/// compatible with what `/api/news` returns, so the same UI code path
/// can render either source.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, PartialEq, Eq)]
pub struct NewsItemPayload {
    pub id: String,
    pub title: String,
    pub message: String,
    pub notification_type: NewsNotificationType,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    /// 0 = popup stays until dismissed; > 0 = auto-hide after N seconds.
    pub auto_hide_seconds: u32,
    /// When true the frontend re-shows the popup every page load even if
    /// the user has dismissed it (dismissal becomes session-scoped only).
    pub refresh_on_every_login: bool,
    /// Higher priority wins when multiple items are active.
    pub priority: i32,
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
            initiator_id: 123,
            voters: vec![456],
            votes_needed: 2,
            expires_at: now,
        };
        let json = serde_json::to_string(&vote).unwrap();
        assert!(json.contains(r#""status":"vote_in_progress""#));
        assert!(json.contains(r#""initiator_id":"123""#));
        assert!(json.contains(r#""expires_at""#));

        let active = TimerVoteState::TimerActive {
            expires_at: now,
            target_player_id: 789,
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
                user_id: 123,
                username: "TestUser".to_string(),
                avatar_url: None,
                banner_url: None,
                accent_color: None,
                current_game_pool: None,
                active_game_id: None,
                spectate_game_id: None,
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
        let player = PlayerInfo {
            user_id: 1,
            username: "Player1".to_string(),
            avatar_url: None,
            score: 10,
            gems: 5,
            team: None,
            is_connected: true,
        };

        let spectator = SpectatorInfo {
            user_id: 2,
            username: "Spec1".to_string(),
            avatar_url: Some("http://avatar.url".to_string()),
        };

        let snapshot = GameSnapshot {
            game_id: "game1".to_string(),
            state: GameState::InProgress,
            grid: vec![vec![GridCell {
                is_hole: false,
                letter: 'A',
                value: 1,
                multiplier: None,
                has_gem: false,
                is_poisoned: false,
                is_abducted: false,
            }]],
            players: vec![player],
            spectators: vec![spectator],
            current_turn: 1,
            round: 1,
            max_rounds: 3,
            used_words: vec!["WORD".to_string()],
            timer_vote_state: TimerVoteState::default(),
            your_player: None,
            timer_expiration_time: None,
        };

        let json = serde_json::to_string(&snapshot).unwrap();
        assert!(json.contains(r#""game_id":"game1""#));
        assert!(json.contains(r#""players":[{"user_id":"1""#));
        assert!(json.contains(r#""spectators":[{"user_id":"2""#));
        assert!(json.contains(r#""current_turn":"1""#));
    }

    #[test]
    fn test_game_config_default() {
        let config = GameConfig::default();
        assert!(!config.regenerate_board_each_round);
    }

    #[test]
    fn test_game_config_serialization() {
        // Test with default value (false)
        let config = GameConfig {
            regenerate_board_each_round: false,
            grid_size: 5,
            adventure_level: None,
            level_targets: None,
            num_rounds: None,
            bots: Vec::new(),
            custom: None,
            events: None,
        };
        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains(r#""regenerate_board_each_round":false"#));
        assert!(json.contains(r#""grid_size":5"#));
        // None-valued adventure fields are skipped on the wire.
        assert!(
            !json.contains("adventure_level"),
            "adventure_level should be omitted when None"
        );
        assert!(
            !json.contains("level_targets"),
            "level_targets should be omitted when None"
        );

        // Test with true value and 4x4 grid
        let config = GameConfig {
            regenerate_board_each_round: true,
            grid_size: 4,
            adventure_level: None,
            level_targets: None,
            num_rounds: None,
            bots: Vec::new(),
            custom: None,
            events: None,
        };
        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains(r#""regenerate_board_each_round":true"#));
        assert!(json.contains(r#""grid_size":4"#));

        // Adventure fields present should serialize with their expected names.
        let config = GameConfig {
            regenerate_board_each_round: false,
            grid_size: 4,
            adventure_level: Some(7),
            level_targets: Some(LevelTargets {
                one_star: 30,
                two_star: 60,
                three_star: 90,
            }),
            num_rounds: None,
            bots: Vec::new(),
            custom: None,
            events: None,
        };
        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains(r#""adventure_level":7"#));
        assert!(json.contains(r#""level_targets""#));
        assert!(json.contains(r#""one_star":30"#));
        assert!(json.contains(r#""two_star":60"#));
        assert!(json.contains(r#""three_star":90"#));
    }

    #[test]
    fn test_game_config_deserialization() {
        // Test deserializing with explicit false
        let json = r#"{"regenerate_board_each_round":false}"#;
        let config: GameConfig = serde_json::from_str(json).unwrap();
        assert!(!config.regenerate_board_each_round);
        assert_eq!(config.grid_size, 5, "grid_size should default to 5");
        assert!(
            config.adventure_level.is_none(),
            "adventure_level defaults to None"
        );
        assert!(
            config.level_targets.is_none(),
            "level_targets defaults to None"
        );

        // Test deserializing with explicit true
        let json = r#"{"regenerate_board_each_round":true}"#;
        let config: GameConfig = serde_json::from_str(json).unwrap();
        assert!(config.regenerate_board_each_round);

        // Test deserializing with missing field (should use default)
        let json = r"{}";
        let config: GameConfig = serde_json::from_str(json).unwrap();
        assert!(!config.regenerate_board_each_round);
        assert_eq!(config.grid_size, 5);
        assert!(config.adventure_level.is_none());
        assert!(config.level_targets.is_none());

        // Test deserializing with grid_size: 4
        let json = r#"{"grid_size":4}"#;
        let config: GameConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.grid_size, 4);
        assert!(!config.regenerate_board_each_round);

        // Adventure fields present should deserialize through.
        let json = r#"{
            "grid_size": 4,
            "adventure_level": 12,
            "level_targets": { "one_star": 50, "two_star": 75, "three_star": 100 }
        }"#;
        let config: GameConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.adventure_level, Some(12));
        assert_eq!(
            config.level_targets,
            Some(LevelTargets {
                one_star: 50,
                two_star: 75,
                three_star: 100
            })
        );
    }

    #[test]
    fn test_level_targets_serde_round_trip() {
        // Lock in the exact wire-level field names — renaming any of
        // them would silently break every client that had already
        // cached adventure progress.
        let targets = LevelTargets {
            one_star: 100,
            two_star: 200,
            three_star: 300,
        };
        let json = serde_json::to_value(targets).expect("LevelTargets should serialize");
        let expected = serde_json::json!({
            "one_star": 100,
            "two_star": 200,
            "three_star": 300,
        });
        assert_eq!(json, expected);

        let round_tripped: LevelTargets =
            serde_json::from_value(json).expect("LevelTargets should deserialize");
        assert_eq!(round_tripped, targets);
    }

    #[test]
    fn test_grid_cell_is_hole_wire_compat() {
        // Default (non-hole) cells should omit `is_hole` on the wire —
        // otherwise every existing cell in every broadcast pays the
        // bytes and older clients see an unexpected field. Hole cells
        // must carry the flag.
        let plain = GridCell {
            letter: 'A',
            value: 1,
            multiplier: None,
            has_gem: false,
            is_hole: false,
            is_poisoned: false,
            is_abducted: false,
        };
        let json = serde_json::to_string(&plain).unwrap();
        assert!(
            !json.contains("is_hole"),
            "is_hole should be omitted when false (got: {json})"
        );

        let hole = GridCell {
            letter: ' ',
            value: 0,
            multiplier: None,
            has_gem: false,
            is_hole: true,
            is_poisoned: false,
            is_abducted: false,
        };
        let json = serde_json::to_string(&hole).unwrap();
        assert!(json.contains(r#""is_hole":true"#));

        // Missing is_hole on the wire should deserialize to false.
        let incoming = r#"{"letter":"B","value":3,"has_gem":false}"#;
        let cell: GridCell = serde_json::from_str(incoming).unwrap();
        assert!(!cell.is_hole, "is_hole should default to false when absent");
    }

    #[test]
    fn test_grid_cell_is_poisoned_wire_compat() {
        // Default (non-poisoned) cells must omit `is_poisoned` on the
        // wire so every existing cell in non-adventure (or non-snaked)
        // broadcasts doesn't carry the field.
        let plain = GridCell {
            letter: 'A',
            value: 1,
            multiplier: None,
            has_gem: false,
            is_hole: false,
            is_poisoned: false,
            is_abducted: false,
        };
        let json = serde_json::to_string(&plain).unwrap();
        assert!(
            !json.contains("is_poisoned"),
            "is_poisoned should be omitted when false (got: {json})"
        );

        let poisoned = GridCell {
            letter: 'E',
            value: 1,
            multiplier: None,
            has_gem: false,
            is_hole: false,
            is_poisoned: true,
            is_abducted: false,
        };
        let json = serde_json::to_string(&poisoned).unwrap();
        assert!(json.contains(r#""is_poisoned":true"#));

        // Older client/server payloads without the field deserialize
        // to is_poisoned=false — wire-compat with pre-v0.9.0.
        let incoming = r#"{"letter":"B","value":3,"has_gem":false}"#;
        let cell: GridCell = serde_json::from_str(incoming).unwrap();
        assert!(
            !cell.is_poisoned,
            "is_poisoned should default to false when absent"
        );
    }

    #[test]
    fn test_grid_cell_is_abducted_wire_compat() {
        // Mirrors the is_hole / is_poisoned wire-compat contract:
        // default-false cells omit the field; abducted cells carry it.
        let plain = GridCell {
            letter: 'A',
            value: 1,
            multiplier: None,
            has_gem: false,
            is_hole: false,
            is_poisoned: false,
            is_abducted: false,
        };
        let json = serde_json::to_string(&plain).unwrap();
        assert!(
            !json.contains("is_abducted"),
            "is_abducted should be omitted when false (got: {json})"
        );

        let abducted = GridCell {
            letter: 'T',
            value: 1,
            multiplier: None,
            has_gem: false,
            is_hole: false,
            is_poisoned: false,
            is_abducted: true,
        };
        let json = serde_json::to_string(&abducted).unwrap();
        assert!(json.contains(r#""is_abducted":true"#));

        // Pre-v0.9.0 payloads deserialize cleanly with is_abducted=false.
        let incoming = r#"{"letter":"B","value":3,"has_gem":false}"#;
        let cell: GridCell = serde_json::from_str(incoming).unwrap();
        assert!(
            !cell.is_abducted,
            "is_abducted should default to false when absent"
        );
    }

    #[test]
    fn test_news_notification_type_serde() {
        // Known variants round-trip through snake_case.
        let json = serde_json::to_string(&NewsNotificationType::Maintenance).unwrap();
        assert_eq!(json, r#""maintenance""#);
        assert_eq!(
            serde_json::from_str::<NewsNotificationType>(r#""announcement""#).unwrap(),
            NewsNotificationType::Announcement
        );

        // #[serde(other)] catches future server-side variants — load-
        // bearing for forward compatibility with older clients.
        assert_eq!(
            serde_json::from_str::<NewsNotificationType>(r#""patch_notes""#).unwrap(),
            NewsNotificationType::Unknown
        );

        // from_wire_str / From<&str> agree with serde.
        assert_eq!(
            NewsNotificationType::from_wire_str("maintenance"),
            NewsNotificationType::Maintenance
        );
        assert_eq!(
            NewsNotificationType::from("update"),
            NewsNotificationType::Update
        );
        assert_eq!(
            NewsNotificationType::from_wire_str("unknown_future_type"),
            NewsNotificationType::Unknown
        );
    }

    #[test]
    fn bot_spec_round_trips_lowercase() {
        let spec = BotSpec {
            difficulty: BotDifficulty::Hard,
        };
        let json = serde_json::to_string(&spec).unwrap();
        assert_eq!(json, r#"{"difficulty":"hard"}"#);
        let back: BotSpec = serde_json::from_str(&json).unwrap();
        assert_eq!(back.difficulty, BotDifficulty::Hard);
    }

    #[test]
    fn test_adventure_event_kind_serde() {
        // Known kinds round-trip through snake_case names.
        let json = serde_json::to_string(&AdventureEventKind::Bomb).unwrap();
        assert_eq!(json, r#""bomb""#);
        assert_eq!(
            serde_json::from_str::<AdventureEventKind>(r#""snake""#).unwrap(),
            AdventureEventKind::Snake
        );
        assert_eq!(
            serde_json::from_str::<AdventureEventKind>(r#""ufo""#).unwrap(),
            AdventureEventKind::Ufo
        );

        // #[serde(other)] catches any future kind an older client
        // doesn't recognize — load-bearing for forward compatibility.
        assert_eq!(
            serde_json::from_str::<AdventureEventKind>(r#""earthquake""#).unwrap(),
            AdventureEventKind::Unknown
        );
    }

    #[test]
    fn default_gameconfig_wire_unchanged() {
        let json = serde_json::to_string(&GameConfig::default()).unwrap();
        assert!(json.contains(r#""grid_size":5"#));
        assert!(!json.contains("num_rounds")); // skipped when None
        assert!(!json.contains(r#""bots""#) || json.contains(r#""bots":[]"#));
        assert!(!json.contains("custom")); // skipped when None
    }

    #[test]
    fn sandbox_gameconfig_round_trips() {
        let cfg = GameConfig {
            num_rounds: Some(7),
            bots: vec![BotSpec {
                difficulty: BotDifficulty::Easy,
            }],
            custom: Some(CustomMeta {}),
            grid_size: 6,
            ..GameConfig::default()
        };
        let json = serde_json::to_string(&cfg).unwrap();
        let back: GameConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(back.num_rounds, Some(7));
        assert_eq!(back.bots.len(), 1);
        assert!(back.custom.is_some());
        assert_eq!(back.grid_size, 6);
    }

    #[test]
    fn legacy_gameconfig_json_still_parses() {
        let cfg: GameConfig =
            serde_json::from_str(r#"{"regenerate_board_each_round":true,"grid_size":4}"#).unwrap();
        assert!(cfg.regenerate_board_each_round);
        assert!(cfg.bots.is_empty());
        assert!(cfg.num_rounds.is_none());
        assert!(cfg.custom.is_none());
    }

    #[test]
    fn event_config_round_trips() {
        let cfg = EventConfig {
            enabled_kinds: vec![AdventureEventKind::Bomb, AdventureEventKind::Ufo],
            frequency: 0.5,
        };
        let json = serde_json::to_string(&cfg).unwrap();
        let back: EventConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(back.enabled_kinds.len(), 2);
        assert!((back.frequency - 0.5).abs() < 1e-9);
    }

    #[test]
    fn gameconfig_without_events_is_back_compat() {
        let json = serde_json::to_string(&GameConfig::default()).unwrap();
        assert!(!json.contains("events"));
        let back: GameConfig = serde_json::from_str(&json).unwrap();
        assert!(back.events.is_none());
    }

    #[test]
    fn game_type_sandbox_serializes_snake_case() {
        let gt = GameType::Sandbox;
        let json = serde_json::to_string(&gt).unwrap();
        assert_eq!(json, "\"sandbox\"");
        let back: GameType = serde_json::from_str("\"sandbox\"").unwrap();
        assert_eq!(back, GameType::Sandbox);
        assert_eq!(gt.to_string(), "sandbox");
    }
}
