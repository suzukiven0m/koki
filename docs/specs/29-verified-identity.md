# 29. End-to-End Verified Identity (Key Signing)

**Tier:** 3 — Long-term / Vision
**Complexity:** L
**Dependencies:** Persistent identity (spec 21), fingerprint display, verification protocol

## Problem

No way to verify peers are who they claim to be.

## Solution

Beyond persistent keys, allow users to verify each other's identity out-of-band (QR code, fingerprint comparison). Implement a trust-on-first-use (TOFU) model with warnings when a known peer's key changes. Prevents impersonation in open topics.
