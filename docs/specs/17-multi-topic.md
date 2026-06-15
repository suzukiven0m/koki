# 17. Multi-Topic Support

**Tier:** 2 — Mid-term / Feature Expansion
**Complexity:** M
**Dependencies:** Significant refactor of main loop into per-topic task management

## Problem

Can only participate in one gossip topic at a time.

## Solution

Allow a single process to open or join multiple gossip topics simultaneously. Each topic gets its own sender/receiver pair, and messages are prefixed with the topic name. Useful for monitoring multiple channels.
