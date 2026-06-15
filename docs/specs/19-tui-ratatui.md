# 19. TUI (Terminal User Interface) with Ratatui

**Tier:** 3 — Long-term / Vision
**Complexity:** XL
**Dependencies:** Add `ratatui`, `crossterm`; major architectural refactor to event-driven model

## Problem

Raw `println!` output is hard to use for active conversations.

## Solution

Replace raw `println!` with a proper terminal UI using `ratatui`: split pane with message history above, input below, peer list sidebar, status bar with connection info. This is the single biggest UX leap possible. Transforms the tool from a demo into something people would actually use daily.
