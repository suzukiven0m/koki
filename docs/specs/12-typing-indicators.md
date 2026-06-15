# 12. Typing Indicators

**Tier:** 2 — Mid-term / Feature Expansion
**Complexity:** M
**Dependencies:** Terminal manipulation (crossterm or similar)

## Problem

No way to know when other peers are composing a message.

## Solution

Add a `MessageBody::Typing { from }` variant. Broadcast it when the user is composing (debounced). Other peers see "alice is typing..." in their terminal. Requires ANSI line manipulation to render without breaking the input line.
