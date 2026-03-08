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
tukubai-join1
tukubai-count

```

Command crate naming convention:

- Cargo package name: `tukubai-<command>`
- binary name: `<command>`

This keeps workspace crate names unique while exposing user-facing command
names that match tukubai commands.

## tukubai-core responsibilities

- shared error types
- record reader
- field splitting
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

## Compatibility

The project aims to implement open usp tukubai commands.

However compatibility must be defined per command.

Assumptions about historical behavior should not be introduced
without tests.
