# 23. Voice Notes / Audio Streaming

**Tier:** 3 — Long-term / Vision
**Complexity:** XL
**Dependencies:** Add `cpal` (audio I/O), `opus` crate, streaming protocol over gossip, TUI for push-to-talk UX

## Problem

Text-only communication.

## Solution

Capture audio from the microphone, opus-encode it, chunk it through gossip, and play it back on the receiver. Turns this into a walkie-talkie / voice channel. Extremely ambitious but uniquely compelling for P2P.
