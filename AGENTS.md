# AGENTS.md

## Repository identity

This repository is a Rust implementation of open usp tukubai.

The implementation uses a strict byte-oriented parsing model.

The repository does not interpret input as Unicode text.

## Read order

Before modifying code, read documents in this order:

1. SPEC.md
2. DESIGN.md
3. TESTING.md
4. crate-level README

SPEC.md defines the shared parsing behavior.

## Core parsing rules

The following bytes have structural meaning:

- `0x0A` — record separator
- `0x20` — field separator

All other bytes are treated as ordinary data.

Important implications:

- CR (`0x0D`) is data
- TAB (`0x09`) is data
- NUL (`0x00`) is data
- non-UTF-8 bytes are allowed

## Record rules

- records are separated by `0x0A`
- every record must end with `0x0A`
- unterminated final input is an error

## Field rules

Within each record:

- leading `0x20` bytes are ignored
- trailing `0x20` bytes are ignored
- `0x20+` separates fields

Empty records have zero fields.

## Coding rules

Shared parsing code must:

- operate on `&[u8]`
- avoid Unicode utilities
- avoid locale-sensitive logic

Do not use:

```

trim()
split_whitespace()
String-based parsing

```

## Architecture rules

Shared parsing belongs in:

```

crates/tukubai-core

```

Command crates must not reimplement parsing logic.

## Validation

Run before finishing:

```

cargo test
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings

```

If parsing behavior changes:

- update SPEC.md
- update tests
