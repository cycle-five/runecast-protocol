//! Player identity and context types.
//!
//! These types are the single source of truth for player identity information,
//! shared across the handlers crate and backend.

use crate::ServerMessage;

/// Core player identity - the immutable parts that identify a player.
///
/// This is embedded in all player-related structs to avoid field duplication.
/// Contains the fields that are constant for a player's session.
pub trait PlayerIdentity {
    /// Get player ID.
    fn player_id(&self) -> i64;

    /// Get username.
    fn username(&self) -> &str;

    /// Get avatar URL.
    fn avatar_url(&self) -> &str;
}

#[async_trait::async_trait]
pub trait PlayerContext: Send + Sync {
    async fn send_message(&self, msg: ServerMessage);

    /// Get the player's identity.
    fn identity(&self) -> &dyn PlayerIdentity;

    /// Check if the player has admin privileges.
    fn is_admin(&self) -> bool;

    /// Get the current lobby ID if in a lobby.
    fn lobby_id(&self) -> Option<&str>;

    /// Get the current game ID if in a game.
    fn game_id(&self) -> Option<&str>;

    /// Check if the player is spectating.
    fn is_spectating(&self) -> bool;

    /// Check if the player is connected.
    fn is_connected(&self) -> bool;

    // =========================================================================
    // Convenience methods with default implementations
    // =========================================================================

    /// Get player ID directly (convenience for `identity().player_id()`).
    fn player_id(&self) -> i64 {
        self.identity().player_id()
    }

    /// Check if the player is in a lobby.
    fn in_lobby(&self) -> bool {
        self.lobby_id().is_some()
    }

    /// Check if the player is in a game.
    fn in_game(&self) -> bool {
        self.game_id().is_some()
    }

    // /// setters
    // fn set_player_id(&mut self, player_id: i64);
    // fn set_username(&mut self, username: &str);
    // fn set_avatar_url(&mut self, avatar_url: Option<String>);
    // fn set_is_admin(&mut self, is_admin: bool);
    // fn set_lobby_id(&mut self, lobby_id: &str);
    // fn set_game_id(&mut self, game_id: &str);
    // fn set_is_spectating(&mut self, is_spectating: bool);
    // fn set_is_connected(&mut self, is_connected: bool);
}
