# 3. Display Sender Name in "sent" Confirmation

**Tier:** 1 — Near-term / MVP Polish
**Complexity:** S
**Dependencies:** None

## Problem

Line 89 prints `> sent: {text}` but does not show the user's own name.

## Solution

After `AboutMe` is broadcast, the local name should be cached and shown: `> alice: hello` on the sender side too, matching what remote peers see.
