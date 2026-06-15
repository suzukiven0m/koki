# iroh-message

P2P encrypted chat over iroh-gossip. Single-file Rust CLI.

Peers connect directly using [iroh](https://github.com/n0-computer/iroh) networking and exchange messages through the [iroh-gossip](https://github.com/n0-computer/iroh/tree/main/iroh-gossip) protocol. No central server required.

## Build

```sh
cargo build --release
```

## Usage

Create a room:

```sh
cargo run -- open [--name YOUR_NAME]
```

This prints a **ticket** -- a base32-encoded string containing the topic ID and your endpoint address. Share it with whoever you want to chat with.

Join an existing room:

```sh
cargo run -- join <TICKET> [--name YOUR_NAME]
```

If `--name` is omitted, a random name is generated (e.g. `swift-fox-42`).

Press `Ctrl-C` to leave.

## Example session

**Terminal 1 -- open a room:**

```
$ cargo run -- open --name alice
[14:23:01] > opening chat room for topic 3a7f...
[14:23:01] > our endpoint id: d1b2c3...
[14:23:01] > ticket to join us: mjqqt4zycm...
[14:23:01] > waiting for endpoints to join us...
[14:23:01] > connected!
[14:23:01] > type a msg and hit enter to broadcast...
hello from alice
[14:23:05] alice: hello from alice
[14:23:12] swift-fox-42: hi alice!
```

**Terminal 2 -- join with the ticket:**

```
$ cargo run -- join mjqqt4zycm... --name bob
[14:23:10] > joining chat room for topic 3a7f...
[14:23:10] > our endpoint id: e4f5a6...
[14:23:10] > trying to connect to 1 endpoints...
[14:23:11] > connected!
[14:23:11] > type a msg and hit enter to broadcast...
[14:23:12] alice: hello from alice
hi alice!
[14:23:14] bob: hi alice!
```

## How it works

- **iroh endpoint** -- each peer binds an encrypted QUIC endpoint using iroh's default presets (N0 relays for NAT traversal).
- **iroh-gossip** -- peers join a shared topic and broadcast messages to all subscribers. Gossip handles peer discovery and message propagation.
- **Tickets** -- the `open` command generates a ticket containing the topic ID and the creator's endpoint address. The ticket is JSON-serialized, then base32-encoded for easy copy-paste. Pass it to `join` to connect.
- **Message format** -- each message is a JSON object with a `body` (either `AboutMe` for name announcements or `msg` for chat text) and a random 16-byte `nonce`. The nonce ensures every message has a unique serialization, which the gossip layer uses for deduplication.
- **Stdin** -- input runs on a dedicated OS thread (via `std::thread::spawn`) and sends lines to the async runtime over a `tokio::sync::mpsc` channel, keeping the event loop unblocked.

## Dependencies

| Crate | Purpose |
|---|---|
| [iroh](https://crates.io/crates/iroh) | P2P QUIC networking |
| [iroh-gossip](https://crates.io/crates/iroh-gossip) | Gossip protocol for message broadcast |
| [tokio](https://crates.io/crates/tokio) | Async runtime |
| [clap](https://crates.io/crates/clap) | CLI argument parsing (derive) |
| [serde](https://crates.io/crates/serde) / [serde_json](https://crates.io/crates/serde_json) | Message serialization |
| [chrono](https://crates.io/crates/chrono) | Timestamps in log output |
| [rand](https://crates.io/crates/rand) | Random nonces and name generation |
| [data-encoding](https://crates.io/crates/data-encoding) | Base32 ticket encoding |
| [anyhow](https://crates.io/crates/anyhow) | Error handling |
| [futures-lite](https://crates.io/crates/futures-lite) | Stream utilities |

## License

MIT
