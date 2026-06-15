# 25. Plugin / Extension System

**Tier:** 3 — Long-term / Vision
**Complexity:** L
**Dependencies:** IPC design (stdin/stdout JSON, Unix socket, or WASM)

## Problem

No way to extend message handling without modifying core.

## Solution

Define a trait or JSON-RPC interface that external processes can implement to extend message handling: logging to a database, forwarding to a webhook, AI summarization, translation. The core stays small; power users compose behavior.
