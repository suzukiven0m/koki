# 22. Message Reactions and Threads

**Tier:** 3 — Long-term / Vision
**Complexity:** L
**Dependencies:** TUI (spec 19), message ID referencing scheme

## Problem

No way to react to messages or have threaded conversations.

## Solution

Extend `MessageBody` with `Reaction { from, target_nonce, emoji }` and `Thread { from, parent_nonce, text }`. Builds a richer conversation model on top of the gossip substrate. Requires rendering logic in the TUI.
