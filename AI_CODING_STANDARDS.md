# AI Coding Standards & Instructions

**ATTENTION ALL AI ASSISTANTS (Claude, ChatGPT, Copilot, Antigravity, etc.)**

This document serves as a strict guideline for anyone (human or AI) writing code in this repository. You are expected to follow these rules without exception.

## 1. Strict Compile-Time Type Safety for Serialization

**Do NOT write JSON out by hand.**

*   **Prohibited**:
    *   Using the `json!({})` macro to construct ad-hoc objects for protocol messages or complex internal data.
    *   Manually building `serde_json::Value` maps.
    *   String concatenation to build JSON.

*   **Required**:
    *   **Define Serializable Types**: Create dedicated Rust structs or enums with `#[derive(Serialize, Deserialize)]`.
    *   **Location**: Place these type definitions in `src/protocol/types.rs` (or `runecast-protocol` crate) if they are shared, or appropriate local modules if they are internal.
    *   **Use Strong Types**: Pass these structs around instead of `serde_json::Value`.

### Reasoning
Manual JSON construction bypasses the Rust type system, leading to runtime errors, typos, and maintenance headaches. We want the compiler to guarantee that our data matches our schema.

### Example

#### ❌ WRONG (Ad-hoc JSON)

```rust
// DO NOT DO THIS
let response = json!({
    "status": "game_over",
    "final_score": 150,
    "winner": "player1"
});
send(response);
```

#### ✅ CORRECT (Strongly Typed)

Define the type (e.g., in `src/protocol/types.rs`):

```rust
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GameStatus {
    GameOver {
        final_score: i32,
        winner: String,
    },
    // ...
}
```

Use the type:

```rust
let response = GameStatus::GameOver {
    final_score: 150,
    winner: "player1".to_string(),
};
// Serialization happens automatically and safely via serde
send(response);
```

## 2. Protocol Consistency
*   When modifying the protocol, always update `runecast-protocol` (specifically `src/protocol/types.rs` and `src/protocol/server_messages.rs`) first.
*   Ensure the frontend and backend share these definitions where possible (or keep them effectively synced via these types).

## Summary
If you find yourself writing `json!(...)`, **STOP**. Define a struct instead.
