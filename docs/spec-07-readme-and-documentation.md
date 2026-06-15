# Spec 7: README and Usage Documentation

**Status: READY FOR IMPLEMENTATION**

## Problem Statement

The project has no README.md. Anyone encountering the repo -- whether cloning, browsing, or receiving a ticket link -- has zero context on what this tool does, how to build it, or how to use it. The only documentation is SPECS.md (a developer-facing spec catalog, not user documentation). Key information that is currently undiscoverable without reading `src/main.rs`:

- The tool is a P2P CLI chat built on iroh-gossip (not obvious from the name alone).
- Two subcommands exist: `open` (create a room) and `join` (connect via ticket).
- The `--name` flag is optional but defaults to a truncated EndpointId with no fallback name (Spec 4 territory).
- Tickets are base32-encoded JSON blobs containing a TopicId and endpoint list.
- No build prerequisites beyond a standard Rust toolchain are documented.
- The `--bind-port` flag exists (line 16: `default_value = "0"`) but is undiscoverable without `--help`.

No files reference any README. There is no CI, no LICENSE, no CHANGELOG.

## Proposed Solution

Create a single `README.md` at the project root. No new Rust code. No dependency changes. No Cargo.toml modifications.

The README covers six sections:

1. **Header and One-liner** -- project name, one-sentence description, badges (optional, no CI exists yet so omit CI badges).
2. **What This Is** -- brief explanation of iroh, gossip protocol, and what the tool does. No assumes prior iroh knowledge.
3. **Prerequisites** -- Rust toolchain version (edition 2024 requires nightly or 1.85+ stable as of this writing).
4. **Build** -- `cargo build --release`, where the binary lands.
5. **Usage** -- `open` and `join` subcommands, `--name` and `--bind-port` flags, full example session with two terminals.
6. **Dependencies** -- table of all direct crate dependencies from Cargo.toml with one-line purpose descriptions.
7. **How It Works** (optional architecture section) -- short paragraph on the message flow: `Msg` -> `serde_json` -> gossip broadcast -> `subscribe_loop` decode.

## Implementation Plan

**Step 1: Create `README.md`**

The file should contain approximately this content:

```markdown
# iroh-message

A peer-to-peer CLI chat tool built on [iroh-gossip](https://crates.io/crates/iroh-gossip).
No servers, no accounts -- share a ticket, start chatting.

## What Is This?

iroh-message lets two or more people chat over a peer-to-peer gossip network.
One user opens a chat room and gets a ticket (a shareable connection string).
Other users join by pasting that ticket. Messages are broadcast to all participants
via the iroh-gossip protocol, which uses iroh's QUIC-based P2P transport under the hood.

## Prerequisites

- Rust toolchain (stable 1.85+ or nightly -- required for edition 2024)

Install Rust via [rustup](https://rustup.rs/) if you don't have it.

## Build

```sh
cargo build --release
```

The binary is at `target/release/iroh-message`.

## Usage

### Open a chat room

```sh
iroh-message open
```

This prints a ticket string. Copy it and send it to the person you want to chat with.

Optional flags:
- `--name <NAME>` -- set your display name (default: truncated endpoint ID)
- `--bind-port <PORT>` -- bind to a specific UDP port (default: 0, meaning OS-assigned)

### Join a chat room

```sh
iroh-message join <TICKET>
```

Paste the ticket you received from the room opener.

### Example session

**Terminal 1 (Alice):**
```
$ iroh-message open --name alice
> opening chat room for topic <topic>
> our endpoint id: <id>
> ticket to join us: <ticket>
> waiting for endpoints to join us...
> connected!
> type a msg and hit enter to broadcast...
hello from alice
> sent: hello from alice
bob: hi alice!
```

**Terminal 2 (Bob):**
```
$ iroh-message join <ticket> --name bob
> joining chat room for topic <topic>
> our endpoint id: <id>
> trying to connect to 1 endpoints...
> connected!
> alice is now known as alice
> type a msg and hit enter to broadcast...
alice: hello from alice
hi alice!
> sent: hi alice!
```

## Architecture

iroh-message is a single-file Rust application (~190 lines in `src/main.rs`).

- **CLI layer**: `clap` parses `open`/`join` subcommands and flags.
- **Endpoint setup**: `iroh::Endpoint::bind` creates a QUIC endpoint using iroh's default relay network (N0).
- **Gossip**: `iroh_gossip::Gossip` manages pub/sub on a topic. Messages are broadcast via `sender.broadcast()`.
- **Message format**: `Msg` wraps a `MessageBody` enum (`AboutMe` or `Msg`) plus a random nonce, serialized as JSON.
- **Ticket**: a base32-encoded JSON blob containing the `TopicId` and the opener's `EndpointAddr` list.
- **Input loop**: a dedicated `std::thread` reads stdin and sends lines over an `mpsc` channel to the async main loop.
- **Subscribe loop**: a spawned tokio task decodes incoming gossip events and prints them.

## Dependencies

| Crate | Version | Purpose |
|---|---|---|
| `iroh` | 1.0.0 | P2P QUIC endpoint and router |
| `iroh-gossip` | 0.101.0 | Gossip-based pub/sub messaging |
| `tokio` | 1.52.3 | Async runtime |
| `clap` | 4.6.1 | CLI argument parsing (derive feature) |
| `serde` | 1.0.228 | Serialization (derive feature) |
| `serde_json` | 1.0.150 | JSON encoding for messages and tickets |
| `data-encoding` | 2.11.0 | Base32 encoding for ticket display |
| `futures-lite` | 2.6.1 | Stream utilities (`try_next`) |
| `anyhow` | 1.0.102 | Error handling |
| `rand` | 0.10.1 | Random nonces and topic IDs |

## License

No license specified yet.
```

**Step 2: No other files change.** Cargo.toml, main.rs, and SPECS.md are untouched.

## Testing Strategy

No automated tests. This is a static documentation file. Verification is manual:

1. Run `cargo build --release` from a clean clone following only the README instructions. It must succeed.
2. Run `iroh-message --help` and confirm the flags match what the README documents.
3. Open two terminals. Follow the "Example session" section exactly. Confirm the output matches (modulo IDs and nonces which are random).
4. Verify no broken markdown: render with `grip` or push to a fork and view on GitHub.
5. Verify all crate versions in the dependency table match `Cargo.toml` exactly.

## Risks and Edge Cases

- **Staleness**: If Cargo.toml dependencies change, the table drifts. Mitigation: this is a known documentation maintenance burden. The table is a snapshot; no auto-generation is proposed.
- **Rust edition 2024**: This edition requires Rust 1.85+ or nightly. If the user has an older toolchain, `cargo build` will fail with a confusing edition error. The README explicitly states the minimum version.
- **Relay network availability**: iroh's N0 preset uses a relay network. If iroh's relay infrastructure is down, `Endpoint::bind` may fail or peers may not discover each other. The README does not attempt to cover this operational detail (it's an iroh concern, not this tool's).
- **Ticket overflow**: Long tickets may wrap awkwardly in terminal copy-paste. The README does not address this (out of scope for a README).
- **No LICENSE file**: The README notes this. Spec 7 does not include adding a license -- that is a separate decision.
- **Example output format**: The `> ` prefix on system messages and `name: text` format on chat messages is current behavior (lines 72, 73, 79, 81, 89 of main.rs). If Spec 2 (timestamps) or Spec 16 (colors) land first, the example session will need updating. Flag this as a dependency ordering note.

## Acceptance Criteria

1. `README.md` exists at the project root.
2. The file contains all six required sections: What Is This, Prerequisites, Build, Usage, Dependencies, Architecture (or How It Works).
3. The Usage section documents both `open` and `join` subcommands with their flags (`--name`, `--bind-port`).
4. The Usage section includes a two-terminal example session showing `open` on one side and `join` on the other, with actual output lines matching what the code produces (system messages prefixed with `> `, chat messages in `name: text` format).
5. The Dependencies table lists all 10 crates from Cargo.toml with correct version numbers.
6. A fresh user can follow the README from clone to working chat without reading any source code.
7. No Rust source files or Cargo.toml are modified.
