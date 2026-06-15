# Spec 8: Basic Integration Test with Two Nodes

**Status: READY FOR IMPLEMENTATION**

## Problem Statement

The project has zero tests. The 190-line `main.rs` is entirely untestable in its current form:

1. **No library crate boundary.** Everything lives in `main.rs` (lines 1-190). Integration tests in Rust can only import from library crates (`src/lib.rs`), not binaries.
2. **Business logic is inlined in `main()`** (lines 31-97). The gossip subscribe, broadcast, stdin loop, and ticket creation are all inside the async main function. There is no way to programmatically drive the "open room, join room, send message, receive message" flow without spawning external processes.
3. **No `#[cfg(test)]` blocks exist.** Zero unit tests, zero integration tests.
4. **`Message`, `Ticket`, `MessageBody`** (lines 100-190) are private to the binary crate. They cannot be imported by a `tests/` directory.

The core happy path -- node A opens a room, node B joins via ticket, B sends a message, A receives it -- is untested.

## Proposed Solution

Split the project into a library crate (`lib.rs`) and thin binary (`main.rs`), then write a `tokio::test` integration test that spawns two in-process iroh endpoints, runs the gossip protocol end-to-end, and asserts message delivery.

**Structural changes:**

| File | Action | Purpose |
|---|---|---|
| `src/lib.rs` | Create | Expose `Message`, `Ticket`, `MessageBody`, and a new `run_node()` async function |
| `src/main.rs` | Modify | Thin wrapper calling into `lib.rs`; keep CLI parsing and stdin loop here |
| `tests/integration.rs` | Create | Two-node integration test |
| `Cargo.toml` | Modify | Add `[dev-dependencies]` for `tokio` test features |

**Key design decisions:**

- Expose a `run_node(config) -> (NodeHandle, Ticket)` function in `lib.rs` that encapsulates endpoint creation, gossip subscription, and message sending/receiving without touching stdin/stdout.
- The test never spawns OS processes. Both nodes run as in-process async tasks, which is faster, more reliable, and gives direct access to message channels for assertion.
- Use `Endpoint::bind(presets::N0)` -- the actual iroh 1.0.0 API -- for endpoint creation. The `presets::N0` preset handles ALPN and relay configuration. This matches the existing `main.rs` (line 47).
- The `run_node()` function uses `tokio::time::timeout` around `subscribe_and_join` for the join case, mitigating the race where the join node attempts to connect before the open node is ready.

**Visibility changes from original code:**

The original `main.rs` keeps `Message`, `MessageBody`, `Message::from_bytes`, and `Ticket` fields private, with only `Message::new()`, `Message::to_vec()`, and `Ticket::to_bytes()` as `pub`. The extraction to `lib.rs` makes all types, fields, and constructors `pub` so integration tests can import and inspect them. This is an intentional widening of visibility for the library boundary.

## Implementation Plan

**Step 1: Create `src/lib.rs` and extract shared types**

Move these items out of `main.rs` into `lib.rs`. Type names match the existing codebase exactly -- `Message` (not `Msg`), `MessageBody::Message` (not `MessageBody::Msg`).

```rust
// src/lib.rs
pub mod node;

use std::{collections::HashMap, fmt, str::FromStr};

use anyhow::Result;
use iroh::{Endpoint, EndpointAddr, EndpointId, endpoint::presets, protocol::Router};
use iroh_gossip::{
    api::{Event, GossipReceiver},
    net::Gossip,
    proto::TopicId,
};
use serde::{Deserialize, Serialize};

// --- Public types (current main.rs lines 100-190) ---

#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    pub body: MessageBody,
    pub nonce: [u8; 16],
}

#[derive(Debug, Serialize, Deserialize)]
pub enum MessageBody {
    AboutMe { from: EndpointId, name: String },
    Message { from: EndpointId, text: String },
}

impl Message {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        serde_json::from_slice(bytes).map_err(Into::into)
    }
    pub fn new(body: MessageBody) -> Self {
        Message { body, nonce: rand::random() }
    }
    pub fn to_vec(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("serde_json::to_vec is infallible")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ticket {
    pub topic: TopicId,
    pub endpoints: Vec<EndpointAddr>,
}

impl Ticket {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        serde_json::from_slice(bytes).map_err(Into::into)
    }
    pub fn to_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("serde_json::to_vec is infallible")
    }
}

impl fmt::Display for Ticket {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut text = data_encoding::BASE32_NOPAD.encode(&self.to_bytes()[..]);
        text.make_ascii_lowercase();
        write!(f, "{text}")
    }
}

impl FromStr for Ticket {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes = data_encoding::BASE32_NOPAD.decode(s.to_ascii_uppercase().as_bytes())?;
        Self::from_bytes(&bytes)
    }
}
```

**Step 2: Create `NodeHandle` and `run_node()` in `src/lib.rs` (or `src/node.rs`)**

This is the new core abstraction. It wraps an iroh endpoint + gossip sender/receiver into a handle that tests (and main) can use without stdin.

```rust
// src/node.rs (or inline in lib.rs)
use anyhow::Result;
use iroh::{Endpoint, EndpointAddr, EndpointId, endpoint::presets, protocol::Router};
use iroh_gossip::{
    api::{Event, GossipReceiver},
    net::Gossip,
    proto::TopicId,
};
use std::collections::HashMap;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

use crate::{Message, MessageBody, Ticket};

pub struct NodeHandle {
    pub endpoint_id: EndpointId,
    pub endpoint_addr: EndpointAddr,
    name_tx: mpsc::Sender<String>,
    received: mpsc::Receiver<ReceivedMsg>,
    router: Router,
    recv_task: JoinHandle<Result<()>>,
    send_task: JoinHandle<()>,
}

pub enum ReceivedMsg {
    NameAnnounce { from: EndpointId, name: String },
    Chat { from: EndpointId, name: String, text: String },
}

pub struct NodeConfig {
    pub name: Option<String>,
    pub topic: TopicId,
    pub connect_to: Vec<EndpointAddr>, // empty for "open", populated for "join"
}

pub async fn run_node(config: NodeConfig) -> Result<(NodeHandle, Ticket)> {
    let endpoint = Endpoint::bind(presets::N0).await?;

    let gossip = Gossip::builder().spawn(endpoint.clone());
    let router = Router::builder(endpoint.clone())
        .accept(iroh_gossip::ALPN, gossip.clone())
        .spawn();

    let topic = config.topic;
    let endpoint_ids: Vec<EndpointId> = config.connect_to.iter().map(|a| a.id).collect();

    // For the "join" case, wrap subscribe_and_join in a timeout to handle
    // the race where the open node is not yet fully ready. The test's
    // ordering (start node A before node B) mitigates this in practice,
    // but the timeout prevents a hang if the open node is slow.
    let join_timeout = std::time::Duration::from_secs(30);
    let (sender, receiver) = if config.connect_to.is_empty() {
        gossip.subscribe_and_join(topic, endpoint_ids).await?.split()
    } else {
        tokio::time::timeout(join_timeout, gossip.subscribe_and_join(topic, endpoint_ids))
            .await
            .map_err(|_| anyhow::anyhow!("timed out joining gossip mesh"))??
            .split()
    };

    // Build ticket
    let ticket = Ticket {
        topic,
        endpoints: vec![endpoint.addr()],
    };

    // Announce name if provided
    if let Some(name) = &config.name {
        let message = Message::new(MessageBody::AboutMe {
            from: endpoint.id(),
            name: name.clone(),
        });
        sender.broadcast(message.to_vec().into()).await?;
    }

    // Give the gossip mesh a moment to propagate after subscribe_and_join.
    // This is cheaper than a sleep and often sufficient for the first
    // message to be delivered correctly.
    tokio::task::yield_now().await;

    // Spawn receive loop -> mpsc
    let (recv_tx, recv_rx) = mpsc::channel(64);
    let recv_task = tokio::spawn(receive_loop(receiver, recv_tx));

    // Spawn send loop <- mpsc
    let (send_tx, mut send_rx) = mpsc::channel::<String>(64);
    let my_id = endpoint.id();
    let send_task = tokio::spawn(async move {
        while let Some(text) = send_rx.recv().await {
            let message = Message::new(MessageBody::Message { from: my_id, text });
            // Propagate broadcast errors via logging. The send channel is
            // fire-and-forget from the caller's perspective; a failed
            // broadcast means the peer disconnected, not a bug in our code.
            if let Err(e) = sender.broadcast(message.to_vec().into()).await {
                eprintln!("> broadcast error: {e}");
            }
        }
    });

    let handle = NodeHandle {
        endpoint_id: endpoint.id(),
        endpoint_addr: endpoint.addr(),
        name_tx: send_tx,
        received: recv_rx,
        router,
        recv_task,
        send_task,
    };

    Ok((handle, ticket))
}

async fn receive_loop(
    mut receiver: GossipReceiver,
    tx: mpsc::Sender<ReceivedMsg>,
) -> Result<()> {
    let mut names: HashMap<EndpointId, String> = HashMap::new();
    while let Some(event) = receiver.try_next().await? {
        match event {
            Event::Received(msg) => {
                match Message::from_bytes(&msg.content)?.body {
                    MessageBody::AboutMe { from, name } => {
                        names.insert(from, name.clone());
                        let _ = tx.send(ReceivedMsg::NameAnnounce { from, name }).await;
                    }
                    MessageBody::Message { from, text } => {
                        let name = names
                            .get(&from)
                            .cloned()
                            .unwrap_or_else(|| from.fmt_short().to_string());
                        let _ = tx.send(ReceivedMsg::Chat { from, name, text }).await;
                    }
                }
            }
            // Other gossip events (NeighborUp, NeighborDown, Lagged) are
            // not relevant to message delivery. Log at trace level for
            // debuggability if needed.
            _ => {}
        }
    }
    Ok(())
}

impl NodeHandle {
    pub async fn send(&self, text: &str) -> Result<()> {
        self.name_tx.send(text.to_string()).await.map_err(|_| {
            anyhow::anyhow!("node closed")
        })
    }

    pub async fn recv(&mut self) -> Option<ReceivedMsg> {
        self.received.recv().await
    }

    /// Non-blocking receive for use in test assertions.
    pub fn try_recv(&mut self) -> Option<ReceivedMsg> {
        self.received.try_recv().ok()
    }

    pub async fn shutdown(self) -> Result<()> {
        // Drop the send channel so the send task exits its loop.
        drop(self.name_tx);
        // Abort spawned tasks to prevent resource leaks if run_node()
        // is used outside of test contexts where the runtime drop would
        // clean them up.
        self.recv_task.abort();
        self.send_task.abort();
        self.router.shutdown().await
    }
}
```

**Important API note:** `receiver.try_next().await?` (line 131 of current `main.rs`) comes from `futures_lite::StreamExt`. The `futures-lite` dependency (currently `"2.6.1"` in Cargo.toml) must remain in `[dependencies]`.

**Step 3: Refactor `src/main.rs` to use `lib.rs`**

`main.rs` becomes a thin CLI shell. The `--bind-port` argument is explicitly preserved even though it is currently unused -- removing it would break any scripts that reference it. A comment marks it as reserved for future use.

```rust
use clap::Parser;
use iroh_message::{Ticket, node::{run_node, NodeConfig, ReceivedMsg}};
use iroh_gossip::proto::TopicId;
use std::str::FromStr;

#[derive(Parser, Debug)]
struct Args {
    #[clap(short, long)]
    name: Option<String>,
    /// Reserved for future use. Currently unused.
    #[clap(short, long, default_value = "0")]
    bind_port: u16,
    #[clap(subcommand)]
    command: Command,
}

#[derive(Parser, Debug)]
enum Command {
    Open,
    Join { ticket: String },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args.parse();

    let (topic, connect_to) = match &args.command {
        Command::Open => {
            let topic = TopicId::from_bytes(rand::random());
            println!("> opening chat room for topic {topic}");
            (topic, vec![])
        }
        Command::Join { ticket } => {
            let t = Ticket::from_str(ticket)?;
            println!("> joining chat room for topic {}", t.topic);
            (t.topic, t.endpoints)
        }
    };

    let config = NodeConfig {
        name: args.name,
        topic,
        connect_to,
    };

    let (mut handle, ticket) = run_node(config).await?;
    println!("> ticket to join us: {ticket}");

    let (line_tx, mut line_rx) = tokio::sync::mpsc::channel(1);
    std::thread::spawn(move || input_loop(line_tx));

    println!("> type a message and hit enter to broadcast...");
    while let Some(text) = line_rx.recv().await {
        handle.send(&text).await?;
        println!("> sent: {text}");
    }

    handle.shutdown().await?;
    Ok(())
}

fn input_loop(line_tx: tokio::sync::mpsc::Sender<String>) -> anyhow::Result<()> {
    let mut buffer = String::new();
    let stdin = std::io::stdin();
    loop {
        stdin.read_line(&mut buffer)?;
        line_tx.blocking_send(buffer.clone())?;
        buffer.clear();
    }
}
```

**Step 4: Create `tests/integration.rs`**

```rust
use iroh_message::node::{run_node, NodeConfig, ReceivedMsg};
use iroh_gossip::proto::TopicId;
use std::time::Duration;

#[tokio::test]
async fn two_nodes_send_and_receive() {
    let topic = TopicId::from_bytes(rand::random());

    // Node A: opens the room
    let (mut node_a, ticket) = run_node(NodeConfig {
        name: Some("Alice".into()),
        topic,
        connect_to: vec![],
    })
    .await
    .expect("node A failed to start");

    // Node B: joins via ticket
    let mut node_b = run_node(NodeConfig {
        name: Some("Bob".into()),
        topic,
        connect_to: ticket.endpoints.clone(),
    })
    .await
    .expect("node B failed to start")
    .0; // discard duplicate ticket

    // Allow gossip mesh to stabilize. The test ordering (A before B)
    // and the timeout in run_node() handle the join race, but a brief
    // yield gives the mesh time to propagate.
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Node B sends a message
    node_b.send("hello from Bob").await.expect("send failed");

    // Node A should receive it. Drain up to 5 messages to handle
    // the case where NameAnnounce arrives before Chat.
    let mut found = false;
    for _ in 0..5 {
        let msg = tokio::time::timeout(Duration::from_secs(10), node_a.recv())
            .await
            .expect("timeout waiting for message")
            .expect("channel closed");

        match msg {
            ReceivedMsg::Chat { name, text, .. } => {
                assert_eq!(name, "Bob");
                assert_eq!(text, "hello from Bob");
                found = true;
                break;
            }
            ReceivedMsg::NameAnnounce { .. } => {
                // Bob's AboutMe arrived first; keep draining.
                continue;
            }
        }
    }
    assert!(found, "never received Chat message from Bob");

    node_a.shutdown().await.ok();
    node_b.shutdown().await.ok();
}

#[tokio::test]
async fn name_announcement_received() {
    let topic = TopicId::from_bytes(rand::random());

    let (mut node_a, ticket) = run_node(NodeConfig {
        name: Some("Alice".into()),
        topic,
        connect_to: vec![],
    })
    .await
    .expect("node A start");

    let _node_b = run_node(NodeConfig {
        name: Some("Bob".into()),
        topic,
        connect_to: ticket.endpoints.clone(),
    })
    .await
    .expect("node B start")
    .0;

    // Alice should see Bob's name announcement. Drain up to 5 messages
    // to find it, since ordering is not guaranteed.
    let mut found_name = false;
    for _ in 0..5 {
        let msg = tokio::time::timeout(Duration::from_secs(10), node_a.recv())
            .await
            .expect("timeout")
            .expect("channel closed");

        if let ReceivedMsg::NameAnnounce { name, .. } = msg {
            assert_eq!(name, "Bob");
            found_name = true;
            break;
        }
    }
    assert!(found_name, "never received NameAnnounce from Bob");
}
```

The `rt-multi-thread` feature is required because `run_node()` uses `tokio::spawn` for the receive and send loops. With `current_thread`, those spawned tasks would not poll unless the test explicitly yields. `rt-multi-thread` ensures spawned tasks make progress concurrently with the test body.

**Step 5: Update `Cargo.toml`**

```toml
[dev-dependencies]
tokio = { version = "1.52.3", features = ["test-util", "macros", "rt-multi-thread"] }
```

No new runtime dependencies are added. `data-encoding` (used by `Ticket`'s `Display`/`FromStr` impls) and `futures-lite` (used by `GossipReceiver::try_next()`) remain in `[dependencies]` unchanged.

## Testing Strategy

**Automated (CI):**
- `cargo test` runs both `#[tokio::test]` functions in `tests/integration.rs`.
- `two_nodes_send_and_receive`: validates the full happy path -- open, join, send, receive, correct name resolution. Drains up to 5 messages to handle `NameAnnounce`-before-`Chat` ordering.
- `name_announcement_received`: validates `AboutMe` propagation. Drains up to 5 messages and asserts at least one `NameAnnounce` with `name == "Bob"` is found.

**Manual smoke test (preserves existing workflow):**
```sh
# Terminal 1
cargo run -- open --name alice
# Copy the ticket

# Terminal 2
cargo run -- join <TICKET> --name bob
# Type messages in both terminals, verify bidirectional delivery
```

**What is NOT tested (out of scope for Spec 8):**
- Reconnection / node failure
- More than 2 nodes
- Malformed tickets
- Long messages / binary payloads
- Concurrent name changes
- `--bind-port` behavior (currently unused)

## Risks and Edge Cases

| Risk | Mitigation |
|---|---|
| **Gossip mesh not ready before send.** Node B joins and immediately sends; the gossip mesh may not have fully formed. | 500ms sleep after join + `tokio::task::yield_now()` in `run_node()` after `subscribe_and_join`. The test's ordering (start node A before node B) and the 30s timeout on join further mitigate this. |
| **`AboutMe` arrives before `Message`.** Node A's recv channel may deliver Bob's name announcement before the chat message. | The test drains up to 5 messages in a loop, checking each one. This handles arbitrary ordering of `NameAnnounce` and `Chat` messages. |
| **Port conflicts on CI.** `Endpoint::bind(presets::N0)` uses the iroh relay network and picks random local ports. No hardcoded ports. | No action needed. |
| **iroh 1.0.0 API surface.** The `Endpoint::bind(presets::N0)` API and `Router::builder` are stable in iroh 1.0.0 but may evolve. | Pin to `iroh = "1.0.0"` in Cargo.toml (already done). No nightly features used. |
| **Test flakiness from timing.** Network-free loopback gossip is deterministic, but tokio scheduling can introduce variance. | 10-second timeout on recv is generous. If flaky, increase to 30s or add retry logic. |
| **Spawned task leakage.** `run_node()` spawns two tasks that outlive the `NodeHandle` if not properly shut down. | `NodeHandle::shutdown()` aborts both `JoinHandle`s and drops the send channel. In tests, the tokio runtime drop also cleans up, but explicit abort is correct for non-test usage. |
| **`lib.rs` exposing too much.** Making `Message`, `Ticket`, `NodeHandle` public creates API surface. | This is intentional -- the library boundary is the design goal. Keep internal helpers (`receive_loop`) private via `pub(crate)`. |
| **`subscribe_and_join` hang on join.** If the open node is not ready, the join node's `subscribe_and_join` could block indefinitely. | `run_node()` wraps the join case in `tokio::time::timeout(30s, ...)`. The open case is not timeout-wrapped since it has no remote dependency. |
| **Rust edition 2024.** The project uses `edition = "2024"`. New code must compile under edition 2024 rules (e.g., `impl Trait` lifetime capture, `gen` keyword reservation). | Unlikely to cause issues for this design, but all new code should be verified with `cargo check` under edition 2024. |
| **Clippy warnings on new public API.** Clippy may flag missing `#[must_use]` on `run_node()` return or missing docs on public items. | Add `#[allow(missing_docs)]` on the `node` module, or write doc comments on all public items. Addressed in acceptance criteria. |

## Acceptance Criteria

1. `src/lib.rs` exists and exposes `Message`, `Ticket`, `MessageBody` as public types. A `node` submodule (either `src/node.rs` or inline) exposes `NodeHandle`, `NodeConfig`, `ReceivedMsg`, `run_node` as public items.
2. `src/main.rs` imports from `iroh_message` (the library crate) and delegates all gossip logic to `run_node()`.
3. `src/main.rs` preserves the `--bind-port` argument (currently unused, marked as reserved).
4. `tests/integration.rs` contains at least 2 `#[tokio::test]` functions.
5. `cargo test` passes with zero failures on a fresh clone.
6. `cargo run -- open` and `cargo run -- join <ticket>` still work identically to the current behavior (manual smoke test).
7. The test `two_nodes_send_and_receive` asserts that a message sent by node B arrives at node A with the correct `name` and `text` fields, handling `NameAnnounce`-before-`Chat` ordering by draining up to 5 messages.
8. The test `name_announcement_received` asserts that a `NameAnnounce` with `name == "Bob"` is received, handling ordering by draining up to 5 messages.
9. No new runtime dependencies are added. `tokio` test features are in `[dev-dependencies]` only. `data-encoding` and `futures-lite` remain in `[dependencies]`.
10. `cargo clippy` passes with no new warnings. Public items in `lib.rs` have doc comments or the module uses `#[allow(missing_docs)]`.
11. `NodeHandle::shutdown()` aborts both spawned `JoinHandle`s and calls `router.shutdown()`.
12. The `subscribe_and_join` call for the join case is wrapped in `tokio::time::timeout` to prevent indefinite hangs.
13. Broadcast errors in the send loop are logged to stderr (not silently swallowed).
14. `receive_loop` has a comment acknowledging that non-`Event::Received` events are intentionally ignored.
