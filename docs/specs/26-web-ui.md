# 26. Web UI via WASM or HTTP Bridge

**Tier:** 3 — Long-term / Vision
**Complexity:** XL
**Dependencies:** Add `axum` or `warp` for HTTP, frontend framework, significant architecture split

## Problem

Terminal-only access limits who can use the tool.

## Solution

Expose a local HTTP server (or compile to WASM) so users can access the chat from a browser tab. The Rust core handles gossip; a lightweight web frontend renders messages. Enables sharing a chat room link that opens in a browser.
