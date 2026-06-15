# 28. Offline Message Queue (Store-and-Forward)

**Tier:** 3 — Long-term / Vision
**Complexity:** XL
**Dependencies:** iroh-blobs integration, peer presence detection, queue persistence

## Problem

Messages sent to offline peers are lost.

## Solution

When a peer is offline, messages addressed to them are queued locally. When the peer reconnects and re-joins, queued messages are delivered. Requires iroh-blobs or a custom store-and-forward layer on top of gossip.
