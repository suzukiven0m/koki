# 16. ANSI Color and Terminal UI

**Tier:** 2 — Mid-term / Feature Expansion
**Complexity:** S
**Dependencies:** Add `colored` crate

## Problem

All output is plain `println!`. Hard to distinguish message types.

## Solution

Add color codes: own messages in green, others in blue, system messages in yellow, errors in red. Use the `colored` crate or raw ANSI escapes. Small change, large readability improvement.
