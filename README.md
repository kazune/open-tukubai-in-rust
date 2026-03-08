# open-tukubai-in-rust

Rust implementation of open usp tukubai.

This repository implements tukubai-style command-line tools in Rust using a
strict byte-oriented parsing model.

The project focuses on:

- small and composable commands
- deterministic byte-level behavior
- streaming processing for large inputs
- explicit and testable semantics

## Parsing model

Shared parsing behavior is intentionally strict:

- `0x0A` (LF) is the only record separator
- `0x20` (SPACE) is the only field separator
- leading and trailing `0x20` in each record are ignored
- all other bytes are treated as ordinary data
- the final record must be LF-terminated

This repository does not interpret text encoding and does not depend on locale.

## Workspace structure

```

crates/
tukubai-core
tukubai-self
tukubai-join1
tukubai-count

```

- **tukubai-core**
  shared parsing rules and record/field model

- **command crates**
  Rust implementations of tukubai commands

## Example commands

| command | description |
|-------|-------------|
| self | select records |
| join1 | join two inputs |
| count | count records |

## Documents

- SPEC.md — shared parsing specification
- DESIGN.md — architecture
- TESTING.md — testing strategy
- ROADMAP.md — implementation phases
