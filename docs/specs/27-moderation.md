# 27. Moderation and Block Lists

**Tier:** 3 — Long-term / Vision
**Complexity:** M
**Dependencies:** Persistent identity (spec 21), protocol extension for moderation messages

## Problem

In open gossip rooms, any node can join and spam.

## Solution

Implement local block lists (by EndpointId), message rate limiting, and optionally a moderator role that can publish a shared ban list through the gossip topic itself.
