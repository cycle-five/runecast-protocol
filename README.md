# runecast-protocol

Wire protocol types for RuneCast WebSocket communication.

## Overview

This crate defines the message types that flow between client and server. It's the **contract** - both sides agree on these shapes.

```
┌────────────┐                           ┌────────────┐
│   Client   │ ◄─── ServerMessage ────── │   Server   │
│  (Browser) │ ──── ClientMessage ─────► │   (Rust)   │
└────────────┘                           └────────────┘
```

## Design Goals

1. **Single source of truth** - Message types defined once, used everywhere
2. **Backward compatible** - Envelope is opt-in, legacy messages still work
3. **Type-safe** - Strongly typed error codes, game states, positions
4. **Serialization-ready** - All types derive Serialize/Deserialize

## Module Structure

```
src/protocol/
├── mod.rs              # Re-exports, constants, compat module
├── envelope.rs         # Envelope<T>, MaybeEnveloped<T>
├── types.rs            # Shared types (Position, Grid, PlayerInfo, etc.)
├── client_messages.rs  # ClientMessage enum (26 variants)
└── server_messages.rs  # ServerMessage enum (40+ variants)
```

## Key Types

### Envelope (Optional Wrapper)

```rust
// New format with envelope
{
  "seq": 42,
  "ack": 41,
  "timestamp": 1699900000000,
  "payload": { "type": "heartbeat" }
}

// Legacy format (still supported)
{ "type": "heartbeat" }
```

The `MaybeEnveloped<T>` type accepts both formats, enabling gradual migration.

### Client Messages

```rust
pub enum ClientMessage {
    // Connection
    Identify { token: String },
    Heartbeat,
    Ack { seq: u64 },
    
    // Lobby
    JoinChannelLobby { channel_id: String, guild_id: Option<String> },
    CreateCustomLobby,
    JoinCustomLobby { lobby_code: String },
    LeaveLobby,
    ToggleReady,
    
    // Game
    CreateGame,
    StartGame,
    SubmitWord { word: String, positions: Vec<Position> },
    PassTurn,
    ShuffleBoard,
    SwapTile { row: u8, col: u8, new_letter: char },
    
    // ... and more
}
```

### Server Messages

```rust
pub enum ServerMessage {
    // Connection
    Hello { session_id: String, server_time: u64 },
    Ready { player: PlayerInfo },
    HeartbeatAck { server_time: u64 },
    
    // Game Events
    GameStarted { game_id: String, grid: Grid, players: Vec<GamePlayerInfo> },
    WordScored { player_id: String, word: String, score: i32, path: Vec<Position> },
    TurnChanged { player_id: String, round: u8 },
    GameOver { final_scores: Vec<ScoreInfo>, winner_id: Option<String> },
    
    // Errors
    Error { code: ErrorCode, message: String, details: Option<Value> },
    
    // ... and more
}
```

### Error Codes

```rust
pub enum ErrorCode {
    NotAuthenticated,
    LobbyNotFound,
    LobbyFull,
    NotYourTurn,
    InvalidPath,
    WordNotInDictionary,
    WordAlreadyUsed,
    InsufficientGems,
    // ... 20+ typed errors
}
```

## Compatibility Layer

The `compat` module helps with migration:

```rust
use runecast_protocol::compat;

// Parse incoming (handles both legacy and enveloped)
let (msg, seq, ack) = compat::parse_client_message(&json_text)?;

// Serialize outgoing (envelope optional)
let json = compat::serialize_server_message(&response, Some(seq), Some(ack))?;

// Convert between legacy and new formats
let snapshot = compat::legacy_game_state_to_snapshot(old_value)?;
```

## Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
runecast-protocol = { path = "../runecast-protocol" }
```

```rust
use runecast_protocol::{
    ClientMessage, ServerMessage, ErrorCode,
    Position, Grid, PlayerInfo,
};

// Deserialize client message
let msg: ClientMessage = serde_json::from_str(&text)?;

// Create server response
let response = ServerMessage::Error {
    code: ErrorCode::NotYourTurn,
    message: "It's not your turn".to_string(),
    details: None,
};
```

## Constants

```rust
pub const HEARTBEAT_INTERVAL_MS: u64 = 30_000;
pub const HEARTBEAT_TIMEOUT_MS: u64 = 45_000;
pub const RECONNECT_GRACE_MS: u64 = 60_000;
pub const MAX_MESSAGE_SIZE: usize = 65_536;
pub const PROTOCOL_VERSION: &str = "1.0.0";
```

## Why a Separate Crate?

1. **Shared between server and potential Rust client** - If you ever build a Rust client or CLI tool, it can use these same types
2. **Forces clean boundaries** - Protocol types can't accidentally depend on server internals
3. **Version independently** - Protocol changes are explicit and trackable
4. **Documentation** - Acts as API documentation for frontend developers
