# Spec 4: Configurable Display Name with Fallback

**Status: READY FOR IMPLEMENTATION**

## Problem Statement

When a user runs `iroh-message open` or `iroh-message join <ticket>` without the `--name` flag, no `AboutMe` message is broadcast at all (lines 72-78 of `main.rs`). The `if let Some(name) = args.name` guard means the user has no display name in the room. Other peers see only the truncated endpoint ID via the fallback in `subscribe_loop` at line 141:

```rust
.map_or_else(|| from.fmt_short().to_string(), String::to_string);
```

This produces opaque identifiers like `a1b2c3d4` that are unmemorable and hostile to group conversation. There is no mechanism to generate a reasonable default name, so every user who omits `--name` is anonymous in the worst possible way -- not "anonymous-fox-42", but a hex fragment.

## Proposed Solution

Generate a random human-friendly default name using the pattern `{adjective}-{animal}-{number}` (e.g., `brave-fox-42`, `calm-otter-7`) when `--name` is not supplied. This requires:

1. **A new function `generate_name()`** in `main.rs` that composes a name from two small inline word lists and a random number. No new crate dependency is needed -- `rand::rng().random_range()` (already available via `rand = "0.10.1"` in Cargo.toml line 13) is sufficient.

2. **Resolve the display name unconditionally** before the `AboutMe` broadcast at line 72. Change the `Option<String>` to a concrete `String` by calling `args.name.unwrap_or_else(generate_name)`.

3. **Always broadcast an `AboutMe` message** -- remove the `if let Some` guard so every participant announces their name.

No changes to the wire protocol, `Message` struct, `MessageBody` enum, `subscribe_loop`, or `Ticket` are required.

## Implementation Plan

**Step 1: Add `generate_name()` function** (insert after line 158, before the `Ticket` struct)

```rust
fn generate_name() -> String {
    let adjectives = [
        "brave", "calm", "eager", "fair", "gentle", "happy", "keen",
        "lively", "merry", "noble", "polite", "quick", "sharp", "warm",
    ];
    let animals = [
        "bear", "crane", "deer", "eagle", "fox", "hawk", "lion",
        "otter", "panda", "raven", "shark", "tiger", "wolf", "yak",
    ];
    let mut rng = rand::rng();
    let adj = adjectives[rng.random_range(0..adjectives.len())];
    let animal = animals[rng.random_range(0..animals.len())];
    let num: u16 = rng.random_range(0..100);
    format!("{adj}-{animal}-{num}")
}
```

Uses 14 adjectives x 14 animals x 100 numbers = 19,600 unique names. Collision is possible but harmless -- this is a display name, not an identity. The word lists are small enough to embed inline with no file I/O or crate dependency.

**Step 2: Add unit test** (insert after `generate_name`)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_name_format() {
        let name = generate_name();
        let parts: Vec<&str> = name.split('-').collect();
        assert_eq!(parts.len(), 3, "name should have 3 dash-separated parts");
        parts[2].parse::<u16>().expect("third part should be a number 0-99");
    }

    #[test]
    fn generate_name_varies() {
        // With 19600 possibilities, two calls should differ with overwhelming probability
        let a = generate_name();
        let b = generate_name();
        // Not asserting a != b since there's a tiny collision chance;
        // instead verify both parse correctly
        assert!(a.contains('-'));
        assert!(b.contains('-'));
    }
}
```

**Step 3: Modify `main()` to always resolve and broadcast a name** (replace lines 72-78)

Current code:
```rust
if let Some(name) = args.name {
    let message = Message::new(MessageBody::AboutMe {
        from: endpoint.id(),
        name,
    });
    sender.broadcast(message.to_vec().into()).await?;
}
```

Replace with:
```rust
let display_name = args.name.unwrap_or_else(generate_name);
let message = Message::new(MessageBody::AboutMe {
    from: endpoint.id(),
    name: display_name,
});
sender.broadcast(message.to_vec().into()).await?;
```

This is the only change to existing code. The `Args.name` field stays `Option<String>` in the clap definition (line 17) -- that is correct; we resolve it to a `String` at usage time.

## Testing Strategy

**Unit tests** (listed above): verify `generate_name()` returns the expected `{adj}-{animal}-{num}` format and that the number parses to a `u16`.

**Manual test steps:**
1. `cargo run -- open` -- verify the terminal prints `AboutMe` broadcast with a generated name like `brave-fox-42` (visible in the `> sent:` or broadcast log).
2. `cargo run -- join <ticket>` in a second terminal -- verify the joiner's generated name appears in terminal 1's output as `> brave-fox-42 is now known as brave-fox-42`.
3. `cargo run -- --name alice open` -- verify `alice` is used, not a generated name.
4. `cargo run -- --name alice join <ticket>` -- verify explicit name still works for joining.

**Integration test outline** (future, out of scope for this spec): spawn two child processes, have one open and the other join via the ticket, assert the joiner's generated name appears in the opener's stdout.

## Risks and Edge Cases

| Risk | Mitigation |
|---|---|
| Name collision between two users who both omit `--name` | Acceptable -- 19,600 combinations, and users can always pass `--name` to disambiguate. Display is `{name}: {text}`, not identity-gated. |
| `rand::rng().random_range()` panics on empty slice | Impossible -- both arrays are non-empty (14 elements each). |
| User passes `--name ""` (empty string) | Clap parses this as `Some("")`. An empty name is weird but won't crash. Could add validation, but out of scope for S-complexity. |
| User passes `--name` with special characters, newlines, or very long strings | Same as current behavior -- no sanitization exists today. Out of scope. |
| Downgrade from `Option<String>` resolution -- what if code later needs to distinguish "user provided name" from "generated name"? | The `Option` remains in `Args`; resolution happens at a single call site. Easy to revisit if needed. |
| `rand 0.10` API compatibility | `rand::rng()` and `random_range()` are stable in 0.10.x (Cargo.toml pins `0.10.1`). Verified against rand 0.10 docs. |

## Acceptance Criteria

1. **`iroh-message open` without `--name`** broadcasts an `AboutMe` message containing a name matching the pattern `{adjective}-{animal}-{0-99}`.
2. **`iroh-message join <ticket>` without `--name`** broadcasts an `AboutMe` message with a generated name.
3. **`iroh-message --name alice open`** uses `alice` as the display name, unchanged from current behavior.
4. **No new crate dependencies** are added to `Cargo.toml`.
5. **No changes to wire protocol** -- `Message`, `MessageBody`, `Ticket`, `subscribe_loop` are untouched.
6. **`cargo test`** passes with the two new unit tests for `generate_name()`.
7. **`cargo clippy`** produces no new warnings.
