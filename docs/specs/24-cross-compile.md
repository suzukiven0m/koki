# 24. Cross-Compilation and Binary Distribution

**Tier:** 3 — Long-term / Vision
**Complexity:** M
**Dependencies:** GitHub Actions, `cross` toolchain, release process

## Problem

Only buildable from source by Rust developers.

## Solution

Build release binaries for Linux (x86_64, aarch64), macOS (universal), and Windows. Set up GitHub Actions CI with `cross` or native runners. Publish to crates.io. Provide a Homebrew formula and AUR package. Makes the tool installable by non-Rust-developers.
