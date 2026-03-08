# TESTING.md

## Principles

The repository uses byte-exact semantics.

Tests must verify exact byte behavior.

## Unit tests

Shared parsing tests must include:

- empty record
- spaces-only record
- multiple spaces
- TAB as data
- CR as data
- NUL as data
- non-UTF-8 data

## Record reader tests

Required tests:

- empty input
- multiple empty records
- normal LF-terminated input
- unterminated final record

## Command tests

Each command must have integration tests verifying:

- stdin input
- file input
- stdout
- stderr
- exit status

Place command integration tests under the corresponding crate:

```
crates/<command>/tests/
```

Place command-specific fixtures under:

```
crates/<command>/tests/fixtures/
```

Only cross-command reusable fixtures should live under:

```
tests/fixtures/shared/
```

## Golden tests

Golden tests verify exact output behavior.

## Test commands

Run:

```

cargo test

```

Also check:

```

cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings

```
