# iroh-message — Spec Catalog

> P2P CLI messaging tool built on iroh-gossip. This document tracks all developable specs across three tiers.

## Tier 1 — Near-term / MVP Polish

Low effort, high value. Fixes bugs and polishes the existing experience.

| # | Spec | Complexity |
|---|------|------------|
| 1 | [Graceful Shutdown and Ctrl+C Handling](01-graceful-shutdown.md) | S |
| 2 | [Timestamp on Messages](02-timestamps.md) | S |
| 3 | [Display Sender Name in "sent" Confirmation](03-sender-name.md) | S |
| 4 | [Configurable Display Name with Fallback](04-default-name.md) | S |
| 5 | [Input Validation and Error Display](05-input-validation.md) | S |
| 6 | [Trim Trailing Newline from Input](06-trim-newline.md) | S |
| 7 | [README and Usage Documentation](07-readme.md) | S |
| 8 | [Basic Integration Test with Two Nodes](08-integration-test.md) | M |

## Tier 2 — Mid-term / Feature Expansion

Meaningful new capabilities that build on the existing architecture.

| # | Spec | Complexity |
|---|------|------------|
| 9 | [Encrypted Message Payloads](09-encryption.md) | M |
| 10 | [Persistent Message History](10-message-history.md) | M |
| 11 | [File and Image Sharing](11-file-sharing.md) | L |
| 12 | [Typing Indicators](12-typing-indicators.md) | M |
| 13 | [Private / Direct Messages](13-direct-messages.md) | M |
| 14 | [Peer List Command](14-peer-list.md) | S |
| 15 | [Reconnection and Resilience](15-reconnection.md) | M |
| 16 | [ANSI Color and Terminal UI](16-ansi-colors.md) | S |
| 17 | [Multi-Topic Support](17-multi-topic.md) | M |
| 18 | [Nonce Deduplication](18-nonce-dedup.md) | S |

## Tier 3 — Long-term / Vision

Ambitious directions that transform this from a demo into a real product.

| # | Spec | Complexity |
|---|------|------------|
| 19 | [TUI (Terminal User Interface) with Ratatui](19-tui-ratatui.md) | XL |
| 20 | [Discovery via iroh-docs or DNS](20-discovery.md) | L |
| 21 | [Persistent Identity (Key Management)](21-persistent-identity.md) | M |
| 22 | [Message Reactions and Threads](22-reactions-threads.md) | L |
| 23 | [Voice Notes / Audio Streaming](23-voice-notes.md) | XL |
| 24 | [Cross-Compilation and Binary Distribution](24-cross-compile.md) | M |
| 25 | [Plugin / Extension System](25-plugin-system.md) | L |
| 26 | [Web UI via WASM or HTTP Bridge](26-web-ui.md) | XL |
| 27 | [Moderation and Block Lists](27-moderation.md) | M |
| 28 | [Offline Message Queue (Store-and-Forward)](28-offline-queue.md) | XL |
| 29 | [End-to-End Verified Identity (Key Signing)](29-verified-identity.md) | L |
| 30 | [Bridge to Other Protocols (Matrix, IRC, XMPP)](30-protocol-bridge.md) | XL |

## Summary

| Complexity | Count | Examples |
|---|---|---|
| S | 9 | Shutdown, timestamps, trim input, README, colors, nonce dedup, name fallback, input validation, sent-name |
| M | 10 | Integration test, encryption, persistence, DMs, reconnection, multi-topic, key management, binary distro, moderation, peer list |
| L | 6 | File sharing, TUI (partial), discovery, reactions/threads, plugin system, verified identity |
| XL | 5 | Full TUI, voice audio, WASM/web UI, store-and-forward, protocol bridges |

**Recommended path:** Specs 1–6 + 16 + 18 as a single cleanup pass → then Spec 19 (TUI) as the transformative feature.
