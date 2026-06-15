# 8. Basic Integration Test with Two Nodes

**Tier:** 1 — Near-term / MVP Polish
**Complexity:** M
**Dependencies:** None (but needs careful async test setup)

## Problem

No automated validation of the core happy path.

## Solution

Spawn two `iroh-message` processes in a test, have one `open` and the other `join` using the ticket, send a message, assert receipt. Can use `tokio::process::Command` and parse stdout. Validates the core happy path does not regress.
