# 1. Graceful Shutdown and Ctrl+C Handling

**Tier:** 1 — Near-term / MVP Polish
**Complexity:** S
**Dependencies:** None

## Problem

Currently `input_loop` runs in a detached `std::thread` with no shutdown signal. Ctrl+C kills the process without closing the endpoint or gossip subscription cleanly.

## Solution

Add `tokio::signal::ctrl_c()` handling, send a shutdown signal through the mpsc channel, and await `router.shutdown()` properly. Prevents resource leaks and abrupt disconnects.
