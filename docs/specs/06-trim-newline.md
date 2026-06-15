# 6. Trim Trailing Newline from Input

**Tier:** 1 — Near-term / MVP Polish
**Complexity:** S
**Dependencies:** None

## Problem

`input_loop` reads lines including the `\n` and sends it. The `text.clone()` on line 89 carries the newline into the broadcast. Remote peers display it, causing double-spaced output.

## Solution

Add `.trim_end()` before sending.
