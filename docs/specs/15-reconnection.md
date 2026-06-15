# 15. Reconnection and Resilience

**Tier:** 2 — Mid-term / Feature Expansion
**Complexity:** M
**Dependencies:** Error handling refactor, potentially iroh reconnection APIs

## Problem

If the gossip connection drops (network blip), the current code has no retry logic.

## Solution

Add automatic re-join with exponential backoff. Cache the ticket so the user does not need to re-enter it. Show connection status in the terminal.
