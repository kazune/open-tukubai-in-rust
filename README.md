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

## Current workspace

The repository currently contains these crates:

```
crates/
  tukubai-core
  tukubai-delf
  tukubai-lcnt
  tukubai-self
```

- **tukubai-core**
  shared parsing rules and record/field model
- **tukubai-delf**
  minimal field-removal command using the shared selector model
- **tukubai-lcnt**
  first command crate; counts LF-terminated records
- **tukubai-self**
  minimal field-selection command using the shared selector model

Additional command crates will be added incrementally after the shared parsing
layer is in place.

Command crates use a split naming convention:

- Cargo package names are `tukubai-<command>`
- installed or built binary names are `<command>`

Example:

- package `tukubai-lcnt` builds the `lcnt` binary

Shared input handling rules:

- `-` means standard input in command crates
- if a command needs to print the standard input source name, it uses `-`
- final-record termination policy is selected by each command via `tukubai-core`

## Planned commands

Initial command coverage is expected to include:

| command | status | description |
|-------|--------|-------------|
| delf | in workspace | delete selected fields from records |
| lcnt | in workspace | count records |
| self | in workspace | select fields from records |
| join1 | planned | join two inputs |

## Documents

- SPEC.md — shared parsing specification
- DESIGN.md — architecture
- TESTING.md — testing strategy
- ROADMAP.md — implementation phases
- CONTRIBUTING.md — contribution and review rules

## Development

Repository-level checks are run from the workspace root:

```
cargo test
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
```

Test assets are split by scope:

- `tests/fixtures/shared/` — checked-in fixtures reused across crates
- `crates/<name>/tests/` — integration tests for each crate
- `crates/<name>/tests/fixtures/` — crate-specific fixtures
- `tests/tmp/` — optional repository-level scratch area ignored by git
