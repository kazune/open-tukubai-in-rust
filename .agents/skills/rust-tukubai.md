---
name: rust-tukubai
description: development guidance for open-tukubai-in-rust
---

# Rust tukubai skill

## Repository intent

Rust implementation of open usp tukubai.

## Core rule

Always follow SPEC.md for parsing.

## Byte semantics

Only two bytes have structure:

0x0A
0x20

All others are ordinary data.

## Validation

Run:

cargo test
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
