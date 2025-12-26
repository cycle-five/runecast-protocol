//! Player identity and context types.
//!
//! These types are the single source of truth for player identity information,
//! shared across the handlers crate and backend.

/// Core player identity - the immutable parts that identify a player.
///
/// This is embedded in all player-related structs to avoid field duplication.
/// Contains the fields that are constant for a player's session.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct PlayerIdentity {
    /// Database user ID
    pub user_id: i64,
    /// Display name
    pub username: String,
    /// Avatar URL if available
    pub avatar_url: Option<String>,
}

impl PlayerIdentity {
    /// Create a new player identity.
    #[must_use]
    pub fn new(user_id: i64, username: impl Into<String>, avatar_url: Option<String>) -> Self {
        Self {
            user_id,
            username: username.into(),
            avatar_url,
        }
    }
}

/// Player context for a connected session.
///
/// This is the single source of truth for a player's identity and current state.
/// It's created on WebSocket connect and updated as the player moves between
/// lobbies and games.
///
/// Contains both identity (immutable for session) and session state (mutable).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PlayerContext {
    /// Core identity (immutable for session)
    pub identity: PlayerIdentity,
    /// Whether player has admin privileges
    pub is_admin: bool,

    // === Session state (mutable) ===
    /// Current lobby ID if in a lobby
    pub lobby_id: Option<String>,
    /// Current game ID if in a game
    pub game_id: Option<String>,
    /// Whether player is spectating (vs playing)
    pub is_spectating: bool,
}

impl PlayerContext {
    /// Create a new context for a freshly connected player.
    #[must_use]
    pub fn new(
        user_id: i64,
        username: impl Into<String>,
        avatar_url: Option<String>,
        is_admin: bool,
    ) -> Self {
        Self {
            identity: PlayerIdentity::new(user_id, username, avatar_url),
            is_admin,
            lobby_id: None,
            game_id: None,
            is_spectating: false,
        }
    }

    /// Create context from an existing identity.
    #[must_use]
    pub fn from_identity(identity: PlayerIdentity, is_admin: bool) -> Self {
        Self {
            identity,
            is_admin,
            lobby_id: None,
            game_id: None,
            is_spectating: false,
        }
    }

    // === Convenience accessors ===

    /// Get user ID.
    #[must_use]
    pub fn user_id(&self) -> i64 {
        self.identity.user_id
    }

    /// Get username.
    #[must_use]
    pub fn username(&self) -> &str {
        &self.identity.username
    }

    /// Get avatar URL.
    #[must_use]
    pub fn avatar_url(&self) -> Option<&str> {
        self.identity.avatar_url.as_deref()
    }

    // === State checks ===

    /// Check if player is in a lobby.
    #[must_use]
    pub fn in_lobby(&self) -> bool {
        self.lobby_id.is_some()
    }

    /// Check if player is in a game (playing or spectating).
    #[must_use]
    pub fn in_game(&self) -> bool {
        self.game_id.is_some()
    }

    /// Check if player is actively playing (not spectating).
    #[must_use]
    pub fn is_playing(&self) -> bool {
        self.game_id.is_some() && !self.is_spectating
    }

    // === State mutations ===

    /// Update lobby membership.
    pub fn set_lobby(&mut self, lobby_id: Option<String>) {
        self.lobby_id = lobby_id;
    }

    /// Update game membership.
    pub fn set_game(&mut self, game_id: Option<String>, is_spectating: bool) {
        self.game_id = game_id;
        self.is_spectating = is_spectating;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_player_identity_new() {
        let identity = PlayerIdentity::new(123, "TestUser", Some("http://avatar.url".to_string()));
        assert_eq!(identity.user_id, 123);
        assert_eq!(identity.username, "TestUser");
        assert_eq!(identity.avatar_url, Some("http://avatar.url".to_string()));
    }

    #[test]
    fn test_player_context_new() {
        let ctx = PlayerContext::new(123, "TestUser", None, false);
        assert_eq!(ctx.user_id(), 123);
        assert_eq!(ctx.username(), "TestUser");
        assert_eq!(ctx.avatar_url(), None);
        assert!(!ctx.is_admin);
        assert!(!ctx.in_lobby());
        assert!(!ctx.in_game());
    }

    #[test]
    fn test_player_context_state_mutations() {
        let mut ctx = PlayerContext::new(123, "TestUser", None, false);

        // Join lobby
        ctx.set_lobby(Some("lobby-1".to_string()));
        assert!(ctx.in_lobby());
        assert_eq!(ctx.lobby_id, Some("lobby-1".to_string()));

        // Join game as player
        ctx.set_game(Some("game-1".to_string()), false);
        assert!(ctx.in_game());
        assert!(ctx.is_playing());
        assert!(!ctx.is_spectating);

        // Switch to spectating
        ctx.set_game(Some("game-1".to_string()), true);
        assert!(ctx.in_game());
        assert!(!ctx.is_playing());
        assert!(ctx.is_spectating);

        // Leave game
        ctx.set_game(None, false);
        assert!(!ctx.in_game());

        // Leave lobby
        ctx.set_lobby(None);
        assert!(!ctx.in_lobby());
    }

    #[test]
    fn test_player_context_from_identity() {
        let identity =
            PlayerIdentity::new(456, "FromIdentity", Some("http://example.com".to_string()));
        let ctx = PlayerContext::from_identity(identity.clone(), true);

        assert_eq!(ctx.user_id(), 456);
        assert_eq!(ctx.username(), "FromIdentity");
        assert_eq!(ctx.avatar_url(), Some("http://example.com"));
        assert!(ctx.is_admin);
        assert!(!ctx.in_lobby());
        assert!(!ctx.in_game());
        assert!(!ctx.is_spectating);
    }
}
