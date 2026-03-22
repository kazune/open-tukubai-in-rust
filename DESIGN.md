# DESIGN.md

## Overview

The repository is implemented as a Rust workspace.

Shared parsing behavior is centralized in the crate:

```

tukubai-core

```

Command crates depend on tukubai-core.

## Design goals

- explicit byte semantics
- deterministic behavior
- streaming processing
- small command implementations

## Workspace structure

```

crates/
tukubai-core
tukubai-self
tukubai-selr
tukubai-join1
tukubai-count

```

Command crate naming convention:

- Cargo package name: `tukubai-<command>`
- binary name: `<command>`

This keeps workspace crate names unique while exposing user-facing command
names that match tukubai commands.

Shared command-input conventions:

- `-` is interpreted as standard input
- when a command prints the standard input source name, it prints `-`
- command crates choose the final-record termination policy through
  `tukubai-core::ReaderOptions`

## tukubai-core responsibilities

- shared error types
- shared command error formatting
- record reader
- field splitting
- shared field-selector parsing and resolution
- helper utilities

## Command crate responsibilities

Each command crate handles:

- CLI arguments
- opening input
- calling tukubai-core
- formatting output
- exit codes

## Data types

Preferred internal types:

```

&[u8]

```

Avoid:

```

String
&str
Unicode whitespace detection
locale-aware logic

```

## Record processing

Typical processing pipeline:

```

reader -> record -> field iterator -> command logic -> output

```

For commands that support shared field selectors, the pipeline becomes:

```

CLI selector args -> selector parser -> record -> field iterator
-> selector resolver -> command logic -> output

```

## Shared selector model

Field-selector syntax that is reused across commands belongs in `tukubai-core`.

The shared selector layer should cover:

- selector parsing
- `NF` resolution against the current record
- missing-field validation
- command-configurable support for selector `0`
- command-configurable support for inclusive ranges

The shared selector layer should not cover command-specific substring syntax such as:

```

1.4
2.1.4

```

Those forms remain in the corresponding command crate, such as `tukubai-self`.

### Selector syntax scope

The shared selector syntax should support, with command-configurable subsets:

- decimal field numbers
- `NF`
- `NF-<n>`
- inclusive ranges `a/b`

The syntax rules are byte-oriented and command-line-token based.
Only uppercase `NF` is valid.
Selectors that do not contain `NF` must be ASCII decimal digits.

Examples of accepted tokens:

- `1`
- `01`
- `NF`
- `NF-0`
- `NF-1/NF`
- `5/2`

Examples of rejected tokens:

- `nf`
- `NF+1`
- `1+2`
- `4-3`
- `+5`
- `/5`
- `3/`

### Selector `0`

Some commands, including `self`, support selector `0` to mean the raw record bytes
before field splitting.

Many other commands will not support `0`.

To keep selector behavior reusable, `tukubai-core` should make `0` support explicit
through configuration, for example:

- an option field
- an enum that selects the zero-selector policy

Selector `0` must not appear inside a range.

Examples that must be rejected:

- `0/3`
- `2/0`

### Range support

Some commands support only a single field selector token and must reject
inclusive ranges such as `a/b`.

To keep selector behavior reusable, `tukubai-core` should make range support
explicit through configuration, alongside selector `0` support.

### Parsing and resolution split

Selector handling should have two phases.

Phase 1: parse command-line selector tokens into a command-independent representation.

Phase 2: resolve that representation against one record.

This split allows commands to:

- parse selectors once at startup
- reuse the parsed representation for every record
- share syntax validation across commands
- keep record-dependent logic in one place

### Suggested API shape

One possible public API shape is:

```rust
pub struct SelectorOptions {
    pub allow_zero: bool,
}

pub struct SelectorProgram { /* parsed selector terms */ }

pub struct FieldPosition(/* 1-based resolved field position */);

pub enum ResolvedItem<'a> {
    Field(&'a [u8]),
    RawRecord(&'a [u8]),
}

pub fn parse_selectors<I, S>(
    inputs: I,
    options: SelectorOptions,
) -> Result<SelectorProgram, SelectorParseError>
where
    I: IntoIterator<Item = S>,
    S: AsRef<[u8]>;

pub fn resolve_selectors<'a>(
    program: &'a SelectorProgram,
    record: &'a [u8],
) -> Result<Vec<ResolvedItem<'a>>, SelectorResolveError>;

pub fn resolve_selector_positions(
    program: &SelectorProgram,
    record: &[u8],
) -> Result<Vec<FieldPosition>, SelectorResolveError>;
```

The exact type names may change, but the split between parsing and per-record
resolution should remain.

### Error model

Selector errors should be distinct from record-reading errors.

Recommended separation:

- record-reading failures stay in `ParseError`
- selector syntax failures use a selector-parse error type
- selector resolution failures use a selector-resolution error type

This keeps command error reporting clear while still allowing command crates to
format all failures through the shared command-error layout.

### Field access model

Selector resolution is record-local.

This means:

- `NF-<n>` is resolved separately for each record
- if `NF-<n>` resolves to `0`, resolution fails
- if any selector refers to a missing field, resolution fails
- if a record is empty, selectors other than `0` fail
- duplicate selectors are preserved
- reverse ranges expand in reverse order

### Command responsibilities after extraction

After selector handling is moved into `tukubai-core`, command crates remain
responsible for:

- CLI argument structure
- file and stdin handling
- command-specific output formatting
- command-specific selector extensions
- exit status decisions

## Compatibility

The project aims to implement open usp tukubai commands.

However compatibility must be defined per command.

Assumptions about historical behavior should not be introduced
without tests.
