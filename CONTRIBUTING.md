# CONTRIBUTING.md

## General rules

- keep changes small
- update tests when behavior changes
- update documentation when semantics change
- keep shared parsing logic in `crates/tukubai-core`
- do not reimplement parsing rules in command crates

## Parsing rules

Shared parsing must follow SPEC.md.

Do not introduce Unicode-aware whitespace behavior.

Avoid using:

```

trim()
split_whitespace()
String-based parsing

```

## Code style

Preferred style:

- explicit byte comparisons
- iterator-based parsing
- small functions

Operate on `&[u8]` in shared parsing code.

Treat only these bytes as structural:

- `0x0A` for record separation
- `0x20` for field separation

Treat TAB, CR, NUL, and non-UTF-8 bytes as ordinary data.

## Test layout

Use the repository test directories consistently:

- `tests/fixtures/shared/` for checked-in fixtures reused across crates
- `crates/<name>/tests/` for crate-local integration tests
- `crates/<name>/tests/fixtures/` for crate-specific input and golden files
- `tests/tmp/` only for repository-level scratch output when needed

Do not commit generated files under `tests/tmp/`.
Prefer crate-local temporary directories during tests when practical.

## Review checklist

Before submitting:

- does the change follow SPEC.md?
- does it preserve streaming behavior?
- does it keep the final-record LF requirement intact?
- are CR and NUL still treated as data?
- are only `0x0A` and `0x20` treated as structural bytes?
- were tests updated?
