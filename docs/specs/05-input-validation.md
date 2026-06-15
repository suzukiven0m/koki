# 5. Input Validation and Error Display

**Tier:** 1 — Near-term / MVP Polish
**Complexity:** S
**Dependencies:** None

## Problem

`Ticket::from_str` can fail on malformed input but the error is propagated up and printed as a raw anyhow chain.

## Solution

Validate ticket input early, print a user-friendly message like `"Invalid ticket format. Expected a base32-encoded string."`, and exit cleanly.
