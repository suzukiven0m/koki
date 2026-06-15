# iroh-message

P2P encrypted chat over iroh-gossip. Single-file Rust CLI.

## Stack
- **iroh** + **iroh-gossip** — peer-to-peer networking & gossip protocol
- **tokio** — async runtime
- **clap** — CLI argument parsing (derive)
- **serde** / **serde_json** — message serialization
- **anyhow** — error handling

## Commands
```bash
cargo build
cargo run -- open          # create a room, prints ticket
cargo run -- join <ticket> # join an existing room
cargo test
cargo check
```

## Conventions
- Edition 2024
- Single source file: `src/main.rs` — keep it here until it outgrows ~500 lines
- Specs live in `docs/specs/` (numbered `01-` through `30-`); completed tier-1 specs in `docs/`
- Read the relevant spec before implementing a feature
- `anyhow::Result` everywhere, no `unwrap()` in production paths
- Async: `#[tokio::main]` entry, `tokio::spawn` for concurrent tasks

## Key Patterns
- Wire format: JSON (`serde_json`) over gossip, nonce included in every message
- Tickets: `Ticket` struct → JSON bytes → base32 (nopad, lowercase for display, uppercase for parse)
- Blocking stdin: `input_loop` runs on `std::thread::spawn` (not tokio), sends lines via `mpsc::channel`
- Graceful shutdown: `router.shutdown().await`

## Git
- **Commit after every modification** — spec changes, code edits, new files, anything. Don't batch.
- **Never add Claude/Anthropic attribution** to commits or commit messages. No `Co-Authored-By`, no "Generated with", nothing. Commits must look human-authored.

## Do NOT
- Auto-read all docs/specs at session start — read only the spec relevant to the current task
- Add dependencies without asking
