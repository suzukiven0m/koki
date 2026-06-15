# 21. Persistent Identity (Key Management)

**Tier:** 3 — Long-term / Vision
**Complexity:** M
**Dependencies:** Key serialization, config directory management

## Problem

Every `Endpoint::bind` generates a fresh identity. Users have no persistent persona across sessions.

## Solution

Implement key storage in `~/.config/iroh-message/` so the same EndpointId and display name persist. Enables reputation, blocklists, and trusted peer lists.
