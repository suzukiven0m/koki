# Spec 1: Graceful Shutdown and Ctrl+C Handling

**Status: READY FOR IMPLEMENTATION**

## Problem Statement

Three issues with the current shutdown path:

1. **No Ctrl+C handler.** `input_loop` (line 98) runs in a detached `std::thread::spawn` (line 86) with an infinite `stdin.read_line()` loop. There is no `tokio::signal::ctrl_c()` anywhere. Ctrl+C kills the process immediately, skipping `router.shutdown()` (line 93).

2. **`input_loop` thread is abandoned.** The `std::thread::spawn` at line 86 is fire-and-forget. When `line_rx.recv()` returns `None` (channel dropped), `main` exits and the thread is left dangling. The thread has no way to be told to stop.

3. **`subscribe_loop` task is abandoned.** The `tokio::spawn` at line 83 runs until the gossip receiver ends or errors. There is no cancellation token or abort handle. If `main` returns early, this task may still be running when the runtime shuts down, causing incomplete cleanup of the gossip subscription.

4. **Ticket prints unconditionally for both commands.** Lines 56-61 construct a ticket and line 62 prints `"> ticket to join us: {ticket}"` regardless of whether the user ran `Command::Open` or `Command::Join`. The message "ticket to join us" only makes sense for `Open`. For `Join`, the user already has a ticket and printing a new one is misleading.

## Proposed Solution

**Approach:** Use `tokio_util::sync::CancellationToken` as the single shutdown coordination primitive. `ctrl_c()` sets the token. `stdin_loop` and `subscribe_loop` observe the token to exit cleanly. `main` awaits both before calling `router.shutdown()`.

**New dependency:** `tokio-util = "0.7"` (for `CancellationToken`). Already a transitive dep of iroh-gossip, so no net-new compilation cost.

**Changes to `main.rs`:**

- Add `tokio::select!` with `ctrl_c()` in the main message-sending loop (lines 88-92).
- Replace `input_loop` with an async `stdin_loop` that accepts a `CancellationToken` and returns when it fires.
- Update `subscribe_loop` to accept a `CancellationToken` and return when it fires.
- Store the `subscribe_loop` `JoinHandle` and abort it on shutdown.
- Gate the ticket print on `Command::Open` only.

**Blocking stdin problem:** `stdin.read_line()` is a blocking syscall that cannot be interrupted by a `CancellationToken` without platform-specific tricks. The cleanest approach: switch `input_loop` to use `tokio::io::stdin()` (async) running on a `tokio::spawn` task instead of a `std::thread`. This lets `tokio::select!` cancel it naturally.

## Implementation Plan

**Step 1: Add dependencies**

In `Cargo.toml`, add `tokio-util` and explicitly declare the tokio features this project requires:

```toml
tokio = { version = "1.52.3", features = ["macros", "rt-multi-thread", "signal", "io-std"] }
tokio-util = "0.7"
```

The explicit tokio features document the project's actual requirements. The current bare `tokio = "1.52.3"` only works because transitive deps happen to enable `signal` and `io-std`; this is fragile across upstream releases.

**Step 2: Add imports**

Add:
```rust
use tokio::io::AsyncBufReadExt;
use tokio_util::sync::CancellationToken;
```

**Step 3: Replace `input_loop` with async `stdin_loop`**

Delete the `input_loop` function (lines 98-106). Replace with:

```rust
async fn stdin_loop(
    line_tx: tokio::sync::mpsc::Sender<String>,
    cancel: CancellationToken,
) {
    let stdin = tokio::io::stdin();
    let reader = tokio::io::BufReader::new(stdin);
    let mut lines = reader.lines();
    loop {
        tokio::select! {
            _ = cancel.cancelled() => break,
            line = lines.next_line() => {
                match line {
                    Ok(Some(text)) => {
                        if line_tx.send(text).await.is_err() {
                            break; // main dropped the receiver
                        }
                    }
                    Ok(None) => break, // EOF (piped input)
                    Err(e) => {
                        eprintln!("> stdin error: {e}");
                        break;
                    }
                }
            }
        }
    }
    // line_tx is dropped here, closing the channel and signaling main
}
```

Note: `AsyncBufReadExt::lines()` assumes valid UTF-8 and returns an error on invalid byte sequences. The current `std::io::stdin().read_line()` also fails on non-UTF8 input, but the error messages will differ. For a chat tool this is a non-issue, but worth knowing during debugging.

**Step 4: Update `subscribe_loop` to accept cancellation**

```rust
async fn subscribe_loop(
    mut receiver: iroh_gossip::api::GossipReceiver,
    cancel: CancellationToken,
) {
    let mut names = std::collections::HashMap::new();
    loop {
        tokio::select! {
            _ = cancel.cancelled() => break,
            event = receiver.try_next() => {
                match event {
                    Ok(Some(msg)) => { /* existing match body unchanged */ }
                    Ok(None) => break,
                    Err(e) => { eprintln!("> gossip error: {e}"); break; }
                }
            }
        }
    }
}
```

**Step 5: Update `main()` to coordinate shutdown**

```rust
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let cancel = CancellationToken::new();

    // ... endpoint setup, gossip setup (unchanged) ...

    // Gate ticket print on Open only
    if matches!(args.command, Command::Open) {
        println!("> ticket to join us: {ticket}");
    }

    let (sender, receiver) = gossip.subscribe_and_join(topic, endpoint_ids).await?.split();

    // Broadcast AboutMe (unchanged)
    if let Some(name) = args.name {
        let msg = Msg::new(MessageBody::AboutMe { from: endpoint.id(), name });
        sender.broadcast(msg.to_vec().into()).await?;
    }

    // Spawn subscribe_loop with cancellation
    let sub_cancel = cancel.clone();
    let sub_handle = tokio::spawn(subscribe_loop(receiver, sub_cancel));

    // Spawn stdin_loop with cancellation (replaces std::thread::spawn)
    let (line_tx, mut line_rx) = tokio::sync::mpsc::channel(1);
    let stdin_cancel = cancel.clone();
    tokio::spawn(stdin_loop(line_tx, stdin_cancel));

    // Main send loop with ctrl_c
    println!("> type a msg and hit enter to broadcast...");
    loop {
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                println!("> shutting down...");
                cancel.cancel();
                break;
            }
            text = line_rx.recv() => {
                match text {
                    Some(text) => {
                        let msg = Msg::new(MessageBody::Msg { from: endpoint.id(), text: text.clone() });
                        sender.broadcast(msg.to_vec().into()).await?;
                        println!("> sent: {text}");
                    }
                    None => break, // stdin_loop exited (EOF or cancel)
                }
            }
        }
    }

    // Clean shutdown
    sub_handle.abort();
    router.shutdown().await?;
    Ok(())
}
```

**Step 6: Remove the old `input_loop` function**

Delete lines 98-106 entirely.

### Testing Strategy

**Manual verification:**

1. `cargo run -- open --name alice` in Terminal A.
2. `cargo run -- join <ticket> --name bob` in Terminal B.
3. Type messages, verify delivery works.
4. Press Ctrl+C in Terminal B. Verify:
   - Terminal B prints `> shutting down...` and exits cleanly (exit code 0).
   - Terminal A does not crash (it may print gossip disconnect events, which is fine).
5. Press Ctrl+C in Terminal A. Verify clean shutdown.

**Edge cases to test:**

- Ctrl+C during `subscribe_and_join` (before "connected!") -- should cancel and exit.
- Ctrl+C while waiting for first message -- should exit immediately.
- Piped input: `echo "hello" | cargo run -- open` -- should send "hello" and exit on EOF.

### Risks and Edge Cases

1. **`tokio::io::stdin()` behavior on Windows.** Async stdin on Windows uses a background thread internally (same as tokio's `stdin()` implementation). This is transparent to the caller but worth noting.

2. **`subscribe_loop` cancellation vs graceful gossip unsubscribe.** `CancellationToken` breaks out of the `try_next()` loop but does not send a gossip leave message. Peers will see a connection drop, not a graceful departure. This is acceptable for a CLI tool.

3. **Double Ctrl+C.** If the user presses Ctrl+C twice rapidly, the second signal arrives while shutdown is in progress. `ctrl_c()` future resolves on the first signal; subsequent signals are handled by the OS (which may force-kill). This is standard behavior.

4. **Race between EOF and cancel.** If stdin reaches EOF and cancel fires simultaneously, `tokio::select!` picks one branch. Either path leads to clean exit.

### Acceptance Criteria

- [ ] `cargo build` succeeds with zero errors.
- [ ] `cargo test` passes (if any tests exist).
- [ ] Ctrl+C during a live session exits cleanly with `> shutting down...` message.
- [ ] `router.shutdown()` is called before process exit.
- [ ] The `subscribe_loop` task is aborted before process exit.
- [ ] Ticket is only printed for `Command::Open`, not `Command::Join`.
- [ ] `input_loop` is replaced by async `stdin_loop`; no `std::thread::spawn` remains.
- [ ] Piped input (`echo "hello" | cargo run -- open`) works and exits on EOF.
