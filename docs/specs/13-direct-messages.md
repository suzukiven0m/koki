# 13. Private / Direct Messages

**Tier:** 2 — Mid-term / Feature Expansion
**Complexity:** M
**Dependencies:** None (protocol extension only)

## Problem

All messages are public to the entire topic. No way to send private messages.

## Solution

Add `MessageBody::Direct { from, to, text }` where `to` is an `EndpointId`. Only the intended recipient processes it; others ignore it. Enables private conversations within a group topic without needing a separate channel.
