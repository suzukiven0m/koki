# 18. Nonce Deduplication

**Tier:** 2 — Mid-term / Feature Expansion
**Complexity:** S
**Dependencies:** None

## Problem

The `Message` struct carries a `nonce` field (line 103) but it is never checked for duplicates. Gossip protocols can deliver the same message twice.

## Solution

Track seen nonces in a `HashSet` (with TTL or bounded size) and skip already-processed messages.
