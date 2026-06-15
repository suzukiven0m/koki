# Spec 3: Display Sender Name in Sent Confirmation

**Status: READY FOR IMPLEMENTATION**

## Problem Statement

Line 89 in `main.rs` prints the send confirmation as:

```rust
println!("> sent: {text}");
```

This omits the sender's own name. When a user provides `--name alice` and sends "hello", the output is `> sent: hello` instead of the desired `alice: hello`. Meanwhile, other peers display `alice: hello` correctly because their `subscribe_loop` resolves the name from the `AboutMe` broadcast. The local sender has no equivalent name resolution for their own messages.

The root cause is that `args.name` (an `Option<String>`) is consumed on line 75 and never retained for later use at the send confirmation site. There is no local name cache in the main loop.

## Proposed Solution

Store the local user's name in a variable before it is consumed, then use it in the send confirmation `println!` on line 89.

**What changes:**

- `main()` in `src/main.rs` -- one variable binding added, one `println!` format string modified.

**What does NOT change:**

- No struct or function signature changes.
- No new files.
- No new crate dependencies.
- The `Msg`, `MessageBody`, `subscribe_loop`, `input_loop`, `Ticket` structs and functions are untouched.

## Implementation Plan

**Step 1.** Extract the local name before it is moved into the `AboutMe` broadcast.

After line 71 (`let (sender, receiver) = gossip.subscribe_and_join(...).split();`), and before the existing `if let Some(name) = args.name` block (line 73), add:

```rust
let local_name = args.name.clone();
```

This must come *before* the `if let Some(name) = args.name` block at line 73, because `args.name` is an `Option<String>` that gets moved by the `if let`. Cloning it beforehand preserves the value.

**Step 2.** Update the send confirmation on line 89.

Current code:

```rust
println!("> sent: {text}");
```

Replace with:

```rust
match &local_name {
    Some(name) => println!("> {name}: {text}"),
    None => println!("> {}: {text}", endpoint.id().fmt_short()),
}
```

When `--name` was provided, the confirmation mirrors the format other peers see: `> alice: hello`. When no name was given, it falls back to the short endpoint ID (matching the `subscribe_loop` fallback on line 152).

**Full diff context** (lines 71-92 after change):

```rust
    let (sender, receiver) = gossip.subscribe_and_join(topic, endpoint_ids).await?.split();
    println!("> connected!");
    let local_name = args.name.clone();                          // NEW
    if let Some(name) = args.name {
        let msg = Msg::new(MessageBody::AboutMe {
            from: endpoint.id(),
            name,
        });
        sender.broadcast(msg.to_vec().into()).await?;
    }
    tokio::spawn(subscribe_loop(receiver));
    let (line_tx, mut line_rx) = tokio::sync::mpsc::channel(1);
    std::thread::spawn(move || input_loop(line_tx));
    println!("> type a msg and hit enter to broadcast...");
    while let Some(text) = line_rx.recv().await {
        let msg = Msg::new(MessageBody::Msg {
            from: endpoint.id(),
            text: text.clone(),
        });
        sender.broadcast(msg.to_vec().into()).await?;
        match &local_name {                                      // CHANGED
            Some(name) => println!("> {name}: {text}"),
            None => println!("> {}: {text}", endpoint.id().fmt_short()),
        }
    }
```

No other lines are affected.

## Testing Strategy

**Manual test -- named sender:**

1. Terminal A: `cargo run -- open --name alice`
2. Terminal B: `cargo run -- join <ticket> --name bob`
3. In Terminal A, type `hello` and press Enter.
4. Expected Terminal A output: `> alice: hello`
5. Expected Terminal B output: `alice: hello`

**Manual test -- unnamed sender:**

1. Terminal A: `cargo run -- open` (no `--name`)
2. Terminal B: `cargo run -- join <ticket>`
3. In Terminal A, type `hello` and press Enter.
4. Expected Terminal A output: `> <short-id>: hello` (where `<short-id>` is the local endpoint's short ID)

**Manual test -- round-trip consistency:**

1. Confirm Terminal B's send confirmation also shows `bob: hello` (if `--name bob` was used).
2. Confirm that the name shown locally matches the name remote peers display.

No unit tests are feasible here because the change lives entirely inside the `async main` function with side effects (println, network I/O). The logic is a single `match` on an `Option`, which is trivially verifiable by inspection and manual testing.

## Risks and Edge Cases

| Risk | Mitigation |
|---|---|
| `args.name` is `None` (user omits `--name`) | Fallback to `endpoint.id().fmt_short()`, matching the existing behavior in `subscribe_loop` line 152. No panic. |
| `args.name` clone adds one heap allocation | Negligible. A single `String` clone per process lifetime. |
| `local_name` captured before `AboutMe` broadcast but `AboutMe` fails | If the broadcast fails, the function returns `Err` via `?` and exits. `local_name` is never printed. No inconsistency. |
| Future refactor moves `args.name` earlier | The `clone()` is explicit and will not compile-err silently; the borrow checker enforces correctness. |
| Name contains special characters or is very long | This is a pre-existing concern unrelated to this spec. No regression introduced. |

## Acceptance Criteria

1. When `--name alice` is provided, typing a message prints `> alice: hello` on the sender's terminal (not `> sent: hello`).
2. When `--name` is omitted, typing a message prints `> <endpoint-short-id>: hello` on the sender's terminal.
3. The sender-side format matches the remote-peer display format produced by `subscribe_loop` (line 152: `{name}: {text}` or `{short_id}: {text}`).
4. No new crate dependencies are added.
5. The existing `AboutMe` broadcast behavior is unchanged.
6. `cargo check` and `cargo clippy` pass with no new warnings.
