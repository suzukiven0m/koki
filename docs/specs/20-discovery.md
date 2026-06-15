# 20. Discovery via iroh-docs or DNS

**Tier:** 3 — Long-term / Vision
**Complexity:** L
**Dependencies:** iroh-docs or custom discovery protocol

## Problem

Joining requires copy-pasting a ticket string out-of-band.

## Solution

Implement topic discovery: publish topics to a well-known iroh-docs namespace, or use DNS-SD/mDNS for LAN discovery. Users could browse available rooms instead of needing tickets.
