# 10. Persistent Message History

**Tier:** 2 — Mid-term / Feature Expansion
**Complexity:** M
**Dependencies:** None (JSONL) or add `rusqlite` (SQLite)

## Problem

Messages exist only in terminal scrollback.

## Solution

Add an option to log all messages to a local file (JSONL or SQLite). Users can review past conversations. Could be toggled with `--log <path>` flag. Foundation for any future history/search feature.
