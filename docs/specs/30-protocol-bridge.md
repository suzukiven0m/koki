# 30. Bridge to Other Protocols (Matrix, IRC, XMPP)

**Tier:** 3 — Long-term / Vision
**Complexity:** XL
**Dependencies:** Matrix/IRC/XMPP client libraries, bridge architecture, message format translation

## Problem

iroh-message is isolated from existing chat ecosystems.

## Solution

Build a bridge process that connects an iroh-gossip topic to a Matrix room, IRC channel, or XMPP MUC. Messages flow bidirectionally. This positions iroh-message as a P2P transport layer, not just an end-user app.
