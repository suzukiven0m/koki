# 9. Encrypted Message Payloads

**Tier:** 2 — Mid-term / Feature Expansion
**Complexity:** M
**Dependencies:** Add `chacha20poly1305` or `aes-gcm`, key derivation logic

## Problem

Gossip messages are currently serialized JSON visible to any node that joins the topic.

## Solution

Encrypt `MessageBody` with a shared key derived from the topic or a passphrase, so only participants with the secret can read content. Use `chacha20poly1305` or similar AEAD cipher. The ticket would carry or derive the key.
