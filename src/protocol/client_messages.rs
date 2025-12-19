//! Client-to-server messages.
//!
//! All messages that can be sent from the frontend client to the backend server.
//! Messages are tagged with `type` field for JSON serialization.
//!
//! # Categories
//!
//! - **Connection**: Handshake, heartbeat, acknowledgments
//! - **Lobby**: Join, leave, create lobbies
//! - **Game Lifecycle**: Start, end games
//! - **Game Actions**: Submit words, pass turn, use powers
//! - **Spectator**: Watch games, join as player
//! - **Timer Vote**: Vote to start turn timer
//! - **Admin**: Administrative commands

use serde::{Deserialize, Serialize};

use super::types::{GameMode, Position};

/// Messages sent from client to server.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientMessage {
    // ========================================================================
    // Connection Messages
    // ========================================================================
    /// Initial identification after WebSocket connect.
    ///
    /// If `resume_seq` is provided, attempt to resume a previous session
    /// and receive missed messages since that sequence number.
    Identify {
        /// Last seen sequence number (for session resumption)
        #[serde(skip_serializing_if = "Option::is_none")]
        resume_seq: Option<u64>,
    },

    /// Keep-alive ping. Server responds with `HeartbeatAck`.
    ///
    /// Should be sent every 20-30 seconds to survive proxy timeouts.
    Heartbeat,

    /// Explicit acknowledgment of received messages.
    ///
    /// Can be used when no other message is being sent to confirm receipt.
    /// Usually, acks are piggybacked on other messages via the envelope.
    Ack {
        /// Sequence number being acknowledged
        seq: u64,
    },

    // ========================================================================
    // Lobby Messages
    // ========================================================================
    /// Join a channel-based lobby (default Discord Activity behavior).
    ///
    /// The lobby is automatically created if it doesn't exist.
    JoinChannelLobby {
        /// Discord channel ID
        channel_id: String,
        /// Discord guild ID (optional for DM activities)
        #[serde(skip_serializing_if = "Option::is_none")]
        guild_id: Option<String>,
    },

    /// Create a new custom lobby with a shareable code.
    ///
    /// Returns a 6-character code that others can use to join.
    CreateCustomLobby,

    /// Join an existing custom lobby by its code.
    JoinCustomLobby {
        /// 6-character lobby code (case-insensitive)
        lobby_code: String,
    },

    /// Leave the current lobby.
    ///
    /// If in a game, this also leaves the game.
    LeaveLobby,

    /// Toggle ready state in lobby.
    ///
    /// Ready state indicates willingness to start a game.
    ToggleReady,

    // ========================================================================
    // Game Lifecycle Messages
    // ========================================================================
    /// Request to create a new game (legacy - prefer StartGame).
    #[serde(rename = "create_game")]
    CreateGame { mode: GameMode },

    /// Start a new game in the current lobby.
    ///
    /// By default, any player can start. Can be restricted to host only
    /// via server configuration.
    ///
    /// Requirements:
    /// - Must be in a lobby
    /// - 1-6 connected players
    /// - No game already in progress
    StartGame,

    // ========================================================================
    // Game Action Messages (only valid when Playing)
    // ========================================================================
    /// Submit a word during your turn.
    ///
    /// The word is derived from the positions on the grid.
    /// Positions must form a valid path (adjacent cells, no repeats).
    SubmitWord {
        game_id: String,
        /// The word being submitted (for validation)
        word: String,
        /// Grid positions forming the word path
        positions: Vec<Position>,
    },

    /// Pass your turn without submitting a word.
    ///
    /// Awards 0 points and advances to the next player.
    PassTurn { game_id: String },

    /// Shuffle the board (costs 1 gem).
    ///
    /// Randomizes tile positions while keeping their properties
    /// (letters, multipliers, gems stay on tiles, just positions change).
    ShuffleBoard { game_id: String },

    /// Enter swap mode (for UI feedback).
    ///
    /// Broadcasts to other players that you're considering a swap.
    /// Triggers wobble animation on their screens.
    EnterSwapMode { game_id: String },

    /// Exit swap mode without swapping.
    ExitSwapMode { game_id: String },

    /// Swap a tile's letter (costs 3 gems).
    ///
    /// Changes the letter on a specific tile. The multiplier and gem
    /// status of the tile are preserved.
    SwapTile {
        game_id: String,
        row: usize,
        col: usize,
        /// New letter (A-Z)
        new_letter: char,
    },

    // ========================================================================
    // Spectator Messages
    // ========================================================================
    /// Join a game as a spectator.
    ///
    /// Spectators can view the game but cannot interact with it.
    #[serde(rename = "join_game")]
    SpectateGame { game_id: String },

    /// Join an active game as a player (from spectator mode).
    ///
    /// The player is added at the end of the turn queue.
    /// Previous rounds count as 0 points.
    JoinGameAsPlayer { game_id: String },

    /// Leave spectator mode and return to lobby view.
    LeaveSpectator { game_id: String },

    /// Legacy leave game message.
    LeaveGame { game_id: String },

    // ========================================================================
    // Live Update Messages
    // ========================================================================
    /// Broadcast current tile selection to other players.
    ///
    /// Sent as the player selects tiles, allowing spectators and
    /// other players to see the selection in real-time.
    SelectionUpdate {
        game_id: String,
        positions: Vec<Position>,
    },

    // ========================================================================
    // Timer Vote Messages
    // ========================================================================
    /// Initiate a vote to start a turn timer on the current player.
    ///
    /// Requirements:
    /// - At least 3 players in game
    /// - Not your turn
    /// - No vote already in progress
    /// - Not in cooldown
    InitiateTimerVote { game_id: String },

    /// Vote yes on an active timer vote.
    ///
    /// Requirements:
    /// - Vote must be in progress
    /// - You haven't already voted
    /// - You didn't initiate the vote
    /// - Not your turn
    VoteForTimer { game_id: String },

    // ========================================================================
    // Admin Messages
    // ========================================================================
    /// Request list of games (admin only).
    AdminGetGames,

    /// Delete a specific game (admin only).
    AdminDeleteGame { game_id: String },

    // ========================================================================
    // System Messages (server-generated, not sent by clients)
    // ========================================================================
    /// Player disconnected from WebSocket.
    ///
    /// This message is generated by the backend when a WebSocket connection
    /// closes unexpectedly. It is dispatched to handlers to trigger grace
    /// period logic and schedule cleanup timers.
    ///
    /// **Not sent by clients** - synthesized by the server.
    PlayerDisconnected {
        /// The lobby the player was in (if any)
        #[serde(skip_serializing_if = "Option::is_none")]
        lobby_id: Option<String>,
        /// The game the player was in (if any)
        #[serde(skip_serializing_if = "Option::is_none")]
        game_id: Option<String>,
    },
}

impl ClientMessage {
    /// Get the message type as a string (for logging/debugging).
    pub fn message_type(&self) -> &'static str {
        match self {
            Self::Identify { .. } => "identify",
            Self::Heartbeat => "heartbeat",
            Self::Ack { .. } => "ack",
            Self::JoinChannelLobby { .. } => "join_channel_lobby",
            Self::CreateCustomLobby => "create_custom_lobby",
            Self::JoinCustomLobby { .. } => "join_custom_lobby",
            Self::LeaveLobby => "leave_lobby",
            Self::ToggleReady => "toggle_ready",
            Self::CreateGame { .. } => "create_game",
            Self::StartGame => "start_game",
            Self::SubmitWord { .. } => "submit_word",
            Self::PassTurn { .. } => "pass_turn",
            Self::ShuffleBoard { .. } => "shuffle_board",
            Self::EnterSwapMode { .. } => "enter_swap_mode",
            Self::ExitSwapMode { .. } => "exit_swap_mode",
            Self::SwapTile { .. } => "swap_tile",
            Self::SpectateGame { .. } => "join_game",
            Self::JoinGameAsPlayer { .. } => "join_game_as_player",
            Self::LeaveSpectator { .. } => "leave_spectator",
            Self::LeaveGame { .. } => "leave_game",
            Self::SelectionUpdate { .. } => "selection_update",
            Self::InitiateTimerVote { .. } => "initiate_timer_vote",
            Self::VoteForTimer { .. } => "vote_for_timer",
            Self::AdminGetGames => "admin_get_games",
            Self::AdminDeleteGame { .. } => "admin_delete_game",
            Self::PlayerDisconnected { .. } => "player_disconnected",
        }
    }

    /// Check if this message requires the sender to be in a lobby.
    pub fn requires_lobby(&self) -> bool {
        matches!(
            self,
            Self::LeaveLobby
                | Self::ToggleReady
                | Self::StartGame
                | Self::SubmitWord { .. }
                | Self::PassTurn { .. }
                | Self::ShuffleBoard { .. }
                | Self::EnterSwapMode { .. }
                | Self::ExitSwapMode { .. }
                | Self::SwapTile { .. }
                | Self::SpectateGame { .. }
                | Self::JoinGameAsPlayer { .. }
                | Self::LeaveSpectator { .. }
                | Self::SelectionUpdate { .. }
                | Self::InitiateTimerVote { .. }
                | Self::VoteForTimer { .. }
                | Self::AdminGetGames
                | Self::AdminDeleteGame { .. }
        )
    }

    /// Check if this message requires an active game.
    pub fn requires_active_game(&self) -> bool {
        matches!(
            self,
            Self::SubmitWord { .. }
                | Self::PassTurn { .. }
                | Self::ShuffleBoard { .. }
                | Self::EnterSwapMode { .. }
                | Self::ExitSwapMode { .. }
                | Self::SwapTile { .. }
                | Self::SelectionUpdate { .. }
                | Self::InitiateTimerVote { .. }
                | Self::VoteForTimer { .. }
        )
    }

    /// Check if this message requires it to be the sender's turn.
    pub fn requires_turn(&self) -> bool {
        matches!(
            self,
            Self::SubmitWord { .. }
                | Self::PassTurn { .. }
                | Self::ShuffleBoard { .. }
                | Self::SwapTile { .. }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_heartbeat_serialization() {
        let msg = ClientMessage::Heartbeat;
        let json = serde_json::to_string(&msg).unwrap();
        assert_eq!(json, r#"{"type":"heartbeat"}"#);
    }

    #[test]
    fn test_join_channel_lobby_serialization() {
        let msg = ClientMessage::JoinChannelLobby {
            channel_id: "123456".to_string(),
            guild_id: Some("789".to_string()),
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains(r#""type":"join_channel_lobby""#));
        assert!(json.contains(r#""channel_id":"123456""#));
        assert!(json.contains(r#""guild_id":"789""#));
    }

    #[test]
    fn test_submit_word_serialization() {
        let msg = ClientMessage::SubmitWord {
            game_id: "game_1".to_string(),
            word: "HELLO".to_string(),
            positions: vec![
                Position { row: 0, col: 0 },
                Position { row: 0, col: 1 },
                Position { row: 1, col: 1 },
            ],
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains(r#""type":"submit_word""#));
        assert!(json.contains(r#""word":"HELLO""#));
    }

    #[test]
    fn test_deserialize_heartbeat() {
        let json = r#"{"type":"heartbeat"}"#;
        let msg: ClientMessage = serde_json::from_str(json).unwrap();
        assert!(matches!(msg, ClientMessage::Heartbeat));
    }

    #[test]
    fn test_deserialize_join_channel_lobby() {
        let json = r#"{"type":"join_channel_lobby","channel_id":"123","guild_id":null}"#;
        let msg: ClientMessage = serde_json::from_str(json).unwrap();
        match msg {
            ClientMessage::JoinChannelLobby {
                channel_id,
                guild_id,
            } => {
                assert_eq!(channel_id, "123");
                assert!(guild_id.is_none());
            }
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_message_type() {
        assert_eq!(ClientMessage::Heartbeat.message_type(), "heartbeat");
        assert_eq!(ClientMessage::StartGame.message_type(), "start_game");
    }

    #[test]
    fn test_requires_lobby() {
        assert!(!ClientMessage::Heartbeat.requires_lobby());
        assert!(!ClientMessage::CreateCustomLobby.requires_lobby());
        assert!(ClientMessage::StartGame.requires_lobby());
        assert!(ClientMessage::ToggleReady.requires_lobby());
    }

    #[test]
    fn test_requires_turn() {
        assert!(!ClientMessage::Heartbeat.requires_turn());
        assert!(!ClientMessage::InitiateTimerVote {
            game_id: "game_1".to_string()
        }
        .requires_turn());
        assert!(ClientMessage::PassTurn {
            game_id: "game_1".to_string()
        }
        .requires_turn());
        assert!(ClientMessage::SubmitWord {
            game_id: "game_1".to_string(),
            word: "TEST".to_string(),
            positions: vec![]
        }
        .requires_turn());
    }
}
