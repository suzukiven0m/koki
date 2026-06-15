# Spec 6: Trim Trailing Newline from Input

**Status: READY FOR IMPLEMENTATION**

## Problem Statement

`input_loop` at line 154 calls `stdin.read_line(&mut buffer)`, which appends the trailing `\n` to `buffer`. Line 155 then sends this untrimmed string through the channel. The full data flow:

1. **Line 155** -- `line_tx.blocking_send(buffer.clone())` sends `"hello\n"` instead of `"hello"`.
2. **Line 89** -- `text` (carrying the newline) is embedded in `MessageBody::Message`.
3. **Line 91** -- The newline is serialized into the gossip broadcast.
4. **Line 92** -- Local echo prints `> sent: hello` (println strips its own newline, but the stored `text` still has `\n`).
5. **Line 142** -- Remote peer prints `println!("{name}: {text}")`, where `text` is `"hello\n"`. `println!` appends another `\n`, producing double-spaced output: `alice: hello` followed by a blank line.

The local echo on line 92 masks the bug because `println!` ignores trailing content after its own newline. Remote peers are the ones who see the extra blank line.

## Proposed Solution

Trim the trailing newline from `buffer` before sending it through the channel. One line change in `input_loop`, no new types, no new dependencies.

`str::trim_end()` removes all trailing ASCII whitespace (`\n`, `\r`, `\r\n`, spaces, tabs). This is broader than strictly necessary (`trim_end_matches('\n')` would suffice), but is the more defensive choice -- it handles Windows-style `\r\n` line endings and stray trailing spaces from paste operations without any downside for a chat input field.

## Implementation Plan

**File:** `src/main.rs`

**Change 1 -- line 155:** Replace the `blocking_send` call to trim the buffer before cloning and sending.

Before:
```rust
stdin.read_line(&mut buffer)?;
line_tx.blocking_send(buffer.clone())?;
buffer.clear();
```

After:
```rust
stdin.read_line(&mut buffer)?;
line_tx.blocking_send(buffer.trim_end().to_string())?;
buffer.clear();
```

That is the entire change. No other lines, structs, functions, or files are modified. `buffer.clear()` on line 156 still resets the buffer correctly -- `trim_end()` returns a `&str` slice and does not mutate `buffer`.

**Why `trim_end().to_string()` instead of trimming in-place on `buffer`:** In-place trimming via `buffer.truncate(buffer.trim_end().len())` would also work but saves no allocations (we need a `String` to send through the channel regardless) and is less readable. The `trim_end().to_string()` form is idiomatic Rust for "I need an owned `String` without trailing whitespace."

## Testing Strategy

**Manual verification (primary):**
1. Run `cargo run -- open --name alice` in terminal A.
2. Copy the ticket, run `cargo run -- join <ticket> --name bob` in terminal B.
3. Type `hello` and press Enter in terminal B.
4. Terminal A should display `bob: hello` with no blank line after it. Before this fix it would show `bob: hello` followed by a blank line.

**Automated test (optional, adds one test function):**
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn input_loop_trims_trailing_newline() {
        let input = b"hello world\n";
        let cursor = Cursor::new(input);
        // This test would require refactoring input_loop to accept
        // a generic Read. For a spec this small, manual testing is
        // sufficient. Left as a note for future refactoring.
    }
}
```

Note: `input_loop` is hard to unit-test as written because it reads from `std::io::stdin()` directly. A full refactor to accept a `dyn Read` generic is out of scope for this spec. The manual test above is the appropriate verification level for a one-line change.

## Risks and Edge Cases

- **Empty input:** If the user presses Enter with no text, `read_line` returns `"\n"`, `trim_end()` yields `""`, and an empty string is broadcast. This is harmless -- the remote peer prints `alice: ` (an empty message). No crash, no deserialization error. If empty-message filtering is desired, it is a separate spec.
- **Whitespace-only input:** `"   \n"` trims to `""`. Same behavior as empty input -- harmless, out of scope.
- **Embedded newlines:** Not possible via `read_line` (it reads until the first `\n`), so no risk.
- **Windows `\r\n`:** `trim_end()` strips both `\r` and `\n`, so this fix also handles Windows terminal input correctly without additional logic.
- **Unicode whitespace:** `trim_end()` strips Unicode whitespace (e.g., U+3000 ideographic space). For a chat tool this is fine -- no one intentionally sends trailing Unicode whitespace.
- **Performance:** `trim_end().to_string()` allocates a new `String` instead of cloning. The allocation is trivial for chat-length messages (tens of bytes). No measurable difference.

## Acceptance Criteria

1. Typing `hello` and pressing Enter in one peer displays `hello` (not `hello\n`) on remote peers -- no trailing blank line.
2. Local echo (`> sent: hello`) is unchanged.
3. `cargo build` succeeds with no new warnings.
4. `cargo clippy` reports no new lints on the changed code.
5. Empty input (pressing Enter alone) does not cause a panic or disconnect.
