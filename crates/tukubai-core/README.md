# tukubai-core

Shared parsing utilities for open-tukubai-in-rust.

## Responsibilities

- record reading from any `BufRead` input
- configurable final-record termination policy
- field splitting for `&[u8]` records
- shared parse error types
- shared command error formatting
- shared standard-input source naming and `-` path handling

## Non responsibilities

- CLI argument parsing
- command behavior
- command output formatting
