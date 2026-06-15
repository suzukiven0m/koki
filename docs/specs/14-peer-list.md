# 14. Peer List Command

**Tier:** 2 — Mid-term / Feature Expansion
**Complexity:** S
**Dependencies:** Refactor to share state between subscribe_loop and main loop

## Problem

No way to see who is currently in the chat.

## Solution

Add a `/peers` or `/who` in-band command the user can type to list all known peers and their display names. The `names` HashMap in `subscribe_loop` already tracks this; expose it through a channel back to the main loop.
