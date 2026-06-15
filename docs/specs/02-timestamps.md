# 2. Timestamp on Messages

**Tier:** 1 — Near-term / MVP Polish
**Complexity:** S
**Dependencies:** None (add `chrono` or use `std::time`)

## Problem

All printed messages lack any time context.

## Solution

Add `chrono` or `std::time` timestamps to every `println!` output so users can follow conversation flow. Format like `[14:23:05] alice: hello`. Trivial change with outsized UX value.
