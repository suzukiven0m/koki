# Spec 2: Timestamp on Messages

**Status: READY FOR IMPLEMENTATION**

## Problem Statement

All user-facing output in `main.rs` prints messages with no time context. There are 11 `println!` calls across two locations:

**`main()` function (lines 48-97) -- system/status messages:**
- Line 48: `println!("> opening chat room for topic {topic}");`
- Line 53: `println!("> joining chat room for topic {topic}");`
- Line 58: `println!("> our endpoint id: {}", endpoint.id());`
- Line 72: `println!("> ticket to join us: {ticket}");`
- Line 78: `println!("> waiting for endpoints to join us...");`
- Line 80: `println!("> trying to connect to {} endpoints...", endpoints.len());`
- Line 83: `println!("> connected!");`
- Line 91: `println!("> type a msg and hit enter to broadcast...");`
- Line 97: `println!("> sent: {text}");`

**`subscribe_loop()` function (lines 111, 117) -- incoming gossip messages:**
- Line 111: `println!("> {} is now known as {}", from.fmt_short(), name);`
- Line 117: `println!("{name}: {text}");`

No timestamps, no wall-clock awareness. Users in a long-running chat session have zero temporal reference for when messages arrived or actions occurred.

## Proposed Solution

**Crate choice: `chrono 0.4`**

Rationale:
- `std::time` provides `SystemTime` and `Instant` but has no formatting, no local-time support, and no `HH:MM:SS` pattern. You would need 15+ lines of manual conversion logic against `libc` or `time` anyway.
- The `time` crate can do this but requires `localtime_r` via the `local-offset` feature, which brings in the same safety/complexity surface as chrono. It is not meaningfully lighter for this use case.
- `chrono` is the de facto Rust datetime crate (500M+ downloads, maintained by chronotope org). It provides `Local::now().format("[%H:%M:%S]")` in one expression.

**Design: a helper function `timestamp()` that returns the formatted prefix string.**

Instead of scattering `chrono::Local::now().format(...)` into every `println!`, introduce a single function:

```rust
fn timestamp() -> String {
    chrono::Local::now().format("[%H:%M:%S]").to_string()
}
```

Every `println!` is then prepended with `{timestamp()}`.

**Why a function, not a macro or custom `println!` wrapper:** A function is the simplest approach for 11 call sites. A macro would save zero effort and add indirection. A custom wrapper like `tprintln!` would be warranted at 50+ call sites, not 11.

## Implementation Plan

**Step 1: Add chrono dependency to `Cargo.toml`**

```toml
chrono = "0.4"
```

Add this line to `[dependencies]`. No feature flags needed -- `Local` and `format` are in the default feature set.

**Step 2: Add import to `src/main.rs`**

Add to the existing use block:

```rust
use chrono::Local;
```

**Step 3: Add the `timestamp()` helper function**

Place after the existing `input_loop` function or at the bottom of the file:

```rust
fn timestamp() -> String {
    Local::now().format("[%H:%M:%S]").to_string()
}
```

**Step 4: Update all 11 `println!` calls**

Each `println!` gets the timestamp prepended as a prefix. The format pattern is `{timestamp()} original_text`.

Specific changes in `main()`:

| Line | Before | After |
|------|--------|-------|
| 48 | `println!("> opening chat room for topic {topic}");` | `println!("{} > opening chat room for topic {topic}", timestamp());` |
| 53 | `println!("> joining chat room for topic {topic}");` | `println!("{} > joining chat room for topic {topic}", timestamp());` |
| 58 | `println!("> our endpoint id: {}", endpoint.id());` | `println!("{} > our endpoint id: {}", timestamp(), endpoint.id());` |
| 72 | `println!("> ticket to join us: {ticket}");` | `println!("{} > ticket to join us: {ticket}", timestamp());` |
| 78 | `println!("> waiting for endpoints to join us...");` | `println!("{} > waiting for endpoints to join us...", timestamp());` |
| 80 | `println!("> trying to connect to {} endpoints...", endpoints.len());` | `println!("{} > trying to connect to {} endpoints...", timestamp(), endpoints.len());` |
| 83 | `println!("> connected!");` | `println!("{} > connected!", timestamp());` |
| 91 | `println!("> type a msg and hit enter to broadcast...");` | `println!("{} > type a msg and hit enter to broadcast...", timestamp());` |
| 97 | `println!("> sent: {text}");` | `println!("{} > sent: {text}", timestamp());` |

In `subscribe_loop()`:

| Line | Before | After |
|------|--------|-------|
| 111 | `println!("> {} is now known as {}", from.fmt_short(), name);` | `println!("{} > {} is now known as {}", timestamp(), from.fmt_short(), name);` |
| 117 | `println!("{name}: {text}");` | `println!("{} {name}: {text}", timestamp());` |

**Complete diff summary (2 files changed):**

1. `Cargo.toml` -- 1 line added
2. `src/main.rs` -- 1 use statement added, 1 function added (3 lines), 11 println lines modified

Total: approximately 15 lines changed. No new files.

## Testing Strategy

**Manual verification (primary, since this is a UI formatting change):**

1. `cargo build` -- confirms chrono resolves and compiles with zero errors/warnings.
2. Run `cargo run -- open --name alice` and verify the first output line matches:
   ```
   [14:23:05] > opening chat room for topic <id>
   [14:23:05] > our endpoint id: <id>
   [14:23:05] > ticket to join us: <base32>
   [14:23:05] > waiting for endpoints to join us...
   [14:23:05] > type a msg and hit enter to broadcast...
   ```
3. In a second terminal, `cargo run -- join <ticket> --name bob` and verify:
   ```
   [14:23:10] > joining chat room for topic <id>
   [14:23:10] > our endpoint id: <id>
   [14:23:10] > trying to connect to 1 endpoints...
   [14:23:10] > connected!
   ```
4. On the open side, verify the name announcement prints:
   ```
   [14:23:10] > bob is now known as bob
   ```
5. Type a message on the join side (`hello`), verify both sides print:
   ```
   [14:23:15] > sent: hello          (sender)
   [14:23:15] alice: hello           (receiver, with timestamp)
   ```

**Unit test (optional, for the `timestamp()` function):**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timestamp_format_matches_hh_mm_ss() {
        let ts = timestamp();
        // Must be exactly "[HH:MM:SS]" -- 10 chars, brackets, colons at positions 3 and 6
        assert_eq!(ts.len(), 10);
        assert!(ts.starts_with('['));
        assert!(ts.ends_with(']'));
        assert_eq!(ts.as_bytes()[3], b':');
        assert_eq!(ts.as_bytes()[6], b':');
    }
}
```

This is lightweight and deterministic -- it does not assert the actual clock value, only the format shape.

**No integration test needed.** The change is purely cosmetic (formatting of stdout). The gossip protocol, message serialization, and ticket encoding are untouched.

## Risks and Edge Cases

1. **`chrono` local time panic on exotic systems.** `Local::now()` calls `localtime_r` internally. On systems with a broken or missing TZ database (some minimal Docker images, embedded), this can panic. Mitigation: this is a CLI desktop chat tool, not a server. If the local timezone is broken, the user has bigger problems. No fallback needed.

2. **Thread safety.** `timestamp()` calls `Local::now()` which is a stateless syscall wrapper. It is safe to call from any thread. The `subscribe_loop` runs on a tokio task, and `main` runs on the main runtime thread -- both are fine.

3. **Performance.** `Local::now()` is a `clock_gettime(CLOCK_REALTIME)` call plus a `localtime_r` conversion. Sub-microsecond cost. Called 11 times total during a session. No measurable impact.

4. **Trailing newline consistency.** `println!` appends `\n`. The timestamp is prepended before the existing text. No existing trailing newlines are affected. The output shape is `[HH:MM:SS] original text\n`.

5. **Timezone display.** The format `[%H:%M:%S]` shows local time with no timezone indicator. This is intentional for a local-facing CLI. Adding `%:z` (e.g., `[14:23:05+09:00]`) would clutter the output. If timezone awareness is later desired, it is a one-line format string change.

6. **Message ordering vs clock skew.** Gossip messages may arrive out of order across peers. The timestamp reflects local receipt time, not send time. This is the correct behavior -- the user sees when they personally saw the message, not a remote clock. No action needed.

7. **Clock changes mid-session.** If the system clock is adjusted (NTP step, manual change), timestamps may jump forward or backward. This is acceptable for a chat tool. Not worth guarding against.

## Acceptance Criteria

- [ ] `cargo build` succeeds with zero errors and zero new warnings.
- [ ] `cargo test` passes (if the unit test for `timestamp()` is added).
- [ ] Every `println!` output line in `main()` and `subscribe_loop()` begins with a `[HH:MM:SS]` timestamp prefix.
- [ ] The timestamp reflects local wall-clock time (not UTC, not monotonic).
- [ ] The format is exactly `[HH:MM:SS]` -- two-digit zero-padded hour, minute, second, separated by colons, enclosed in brackets.
- [ ] A sample session (`open` then `join`) produces timestamps on all system messages (`> ...`), name announcements, sent messages, and received chat messages.
- [ ] No changes to `Msg`, `MessageBody`, `Ticket`, `Args`, `Cmd`, or any serialization logic. The wire protocol is untouched.
- [ ] The only new dependency is `chrono`.
