# 4. Configurable Display Name with Fallback

**Tier:** 1 — Near-term / MVP Polish
**Complexity:** S
**Dependencies:** Add `rand` adjective/noun lists or a small crate like `names`

## Problem

The `--name` flag is optional and there is no default. If omitted, the user appears as a truncated endpoint ID.

## Solution

Generate a random human-friendly default name (e.g., `anonymous-fox-42`) so every user has a readable identity without needing to pass a flag.
