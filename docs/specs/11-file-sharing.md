# 11. File and Image Sharing

**Tier:** 2 — Mid-term / Feature Expansion
**Complexity:** L
**Dependencies:** Chunking protocol design, iroh-blobs integration consideration

## Problem

No way to share files or images between peers.

## Solution

Extend `MessageBody` with a `File` variant carrying filename, MIME type, and base64-encoded (or chunked) payload. The receiver writes it to a designated directory. Gossip has message size limits, so this requires chunking logic and reassembly, but gossip's blob-like nature helps.
