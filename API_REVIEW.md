# Public API Review

This document reviews the publicly exposed API and types in the runecast-protocol crate.

## Overview

The crate provides a clean, well-documented API for WebSocket protocol communication between RuneCast clients and servers.

## Public API Surface

### Top-Level Re-exports (lib.rs)

The following types are re-exported at the crate root for convenience:

```rust
pub use protocol::{
    ClientMessage,       // Client → Server messages
    Envelope,            // Message wrapper with seq/ack
    ErrorCode,           // Standard error codes
    GameSnapshot,        // Complete game state
    GameState,           // High-level game state enum
    Grid,                // 5x5 game grid type
    GridCell,            // Individual cell in grid
    LobbySnapshot,       // Complete lobby state
    MaybeEnveloped,      // Wrapper for legacy/new format
    Multiplier,          // Letter/word multipliers
    PlayerInfo,          // Player information
    Position,            // Grid position (row, col)
    ServerMessage,       // Server → Client messages
    TimerVoteState,      // Timer vote system state
};
```

### Protocol Module

The `protocol` module is the main module containing all protocol definitions.

#### Sub-modules

1. **`protocol::client_messages`** - 26 client message variants
2. **`protocol::server_messages`** - 40+ server message variants  
3. **`protocol::types`** - Shared data types
4. **`protocol::envelope`** - Message envelope for reliable delivery
5. **`protocol::compat`** - Compatibility layer for migration

### Key Types

#### Message Enums

**ClientMessage** (26 variants)
- Connection: `Identify`, `Heartbeat`, `Ack`
- Lobby: `JoinChannelLobby`, `CreateCustomLobby`, `JoinCustomLobby`, `LeaveLobby`, `ToggleReady`
- Game Lifecycle: `CreateGame`, `StartGame`
- Game Actions: `SubmitWord`, `PassTurn`, `ShuffleBoard`, `SwapTile`
- Spectator: `SpectateGame`, `JoinGameAsPlayer`, `LeaveSpectator`
- Timer Vote: `InitiateTimerVote`, `VoteForTimer`
- Admin: `AdminGetGames`, `AdminDeleteGame`

**ServerMessage** (40+ variants)
- Connection: `Hello`, `Ready`, `Resumed`, `HeartbeatAck`, `InvalidSession`
- Lobby: `LobbyJoined`, `LobbySnapshot`, `LobbyDelta`, `LobbyLeft`
- Game Lifecycle: `GameStarted`, `GameSnapshot`, `GameDelta`, `GameOver`, `GameCancelled`
- Events: `PlayerJoined`, `PlayerLeft`, `WordScored`, `TurnChanged`, etc.
- Errors: `Error` with typed `ErrorCode`

**ErrorCode** (22 variants)
- Connection: `NotAuthenticated`, `SessionExpired`, `InvalidSession`
- Lobby: `LobbyNotFound`, `LobbyFull`, `NotInLobby`, `AlreadyInLobby`
- Game: `GameNotFound`, `GameInProgress`, `NotInGame`, `NotYourTurn`
- Word validation: `InvalidPath`, `WordNotInDictionary`, `WordAlreadyUsed`
- Resources: `InsufficientGems`
- Rate limiting: `TooManyRequests`, `MessageTooLarge`

#### Data Structures

**Envelope<T>**
- Wraps messages with sequence numbers and acknowledgments
- Fields: `seq`, `ack` (optional), `timestamp`, `payload`

**Position**
- Grid coordinates: `row`, `col`

**GridCell**
- Cell data: `letter`, `value`, `multiplier` (optional), `has_gem`

**PlayerInfo**
- Player data: `user_id`, `username`, `avatar_url`, `score`, `gems`, `team`, `is_connected`

**LobbySnapshot**
- Complete lobby state with players, games, host, max_players

**GameSnapshot**
- Complete game state with grid, players, spectators, turn info, used words, timer state

**TimerVoteState** (5 variants)
- `Idle`, `VoteInProgress`, `TimerActive`, `Cooldown`, `Disabled`

### Protocol Constants

```rust
pub const HEARTBEAT_INTERVAL_MS: u32 = 30_000;
pub const HEARTBEAT_TIMEOUT_MS: u32 = 45_000;
pub const RECONNECT_GRACE_MS: u32 = 60_000;
pub const MAX_MESSAGE_SIZE: usize = 64 * 1024;
pub const PROTOCOL_VERSION: &str = "1.0.0";
```

### Compatibility Layer

The `protocol::compat` module provides migration helpers:

```rust
pub fn parse_client_message(json: &str) 
    -> Result<(ClientMessage, Option<u64>, Option<u64>), serde_json::Error>

pub fn serialize_server_message(msg: &ServerMessage, seq: Option<u64>, ack: Option<u64>) 
    -> Result<String, serde_json::Error>

pub fn legacy_game_state_to_snapshot(value: &Value) -> Option<GameSnapshot>

pub fn snapshot_to_legacy_game_state(snapshot: &GameSnapshot) -> ServerMessage
```

## Documentation Quality

### Strengths

✅ **All public items are documented** - No missing doc comments  
✅ **Rich module-level documentation** - Explains architecture and message flow  
✅ **Field-level documentation** - Important fields have clear descriptions  
✅ **Examples included** - lib.rs includes usage examples  
✅ **No doc warnings** - `cargo doc` produces clean output  
✅ **Structured organization** - Clear module hierarchy  
✅ **Migration guide** - Compatibility layer well documented  

### Documentation Highlights

1. **lib.rs** - High-level overview with usage example
2. **protocol/mod.rs** - Architecture diagram and message flow
3. **protocol/envelope.rs** - Wire format explanation with examples
4. **protocol/types.rs** - Data structure documentation
5. **protocol/client_messages.rs** - Per-message documentation
6. **protocol/server_messages.rs** - Per-message documentation

### API Design Observations

**Excellent:**
- Type-safe enums instead of string types
- Clear separation of concerns (client/server messages)
- Backward compatibility via `MaybeEnveloped`
- Serde-based serialization with tagged enums
- Helper methods on enums (e.g., `ErrorCode::message()`)

**Consider:**
- All types derive `Debug`, `Clone`, `Serialize`, `Deserialize`
- Good use of `#[serde(skip_serializing_if = "Option::is_none")]`
- Consistent naming conventions (snake_case for JSON, PascalCase for Rust)

## Testing

The crate includes 31 passing tests covering:
- Message serialization/deserialization
- Envelope parsing (legacy and new formats)
- Compatibility layer functions
- Error code messages
- Type-specific serialization

## Recommendations

### Already Excellent ✅
- Documentation is comprehensive and clear
- API surface is well-designed and type-safe
- Tests cover critical functionality
- Backward compatibility is handled properly

### Nice-to-Have (Optional)
- Consider adding more usage examples in doc comments
- Could add a CHANGELOG.md for version tracking
- Might benefit from integration tests if used with real server

## Conclusion

The runecast-protocol crate has an **excellent public API** with:
- Clean type-safe design
- Comprehensive documentation
- Good test coverage
- Backward compatibility support
- Clear module organization

The API is ready for production use and the documentation generated by `cargo doc` will serve as excellent reference material for developers.
