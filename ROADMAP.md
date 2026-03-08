# ROADMAP.md

## Phase 1

Workspace foundation

- create workspace manifest
- create tukubai-core
- create initial command crate layout
- implement record reader
- implement field iterator
- implement shared error type
- add unit tests

## Phase 2

First commands

- finish `lcnt`
- add stdin and file integration tests under `crates/tukubai-lcnt/tests/`

## Phase 3

Additional simple commands

- self
- add command-specific integration tests

## Phase 4

Join command

- join1
- add command-specific tests

## Phase 5

Expand command coverage gradually.

Commands should be added incrementally and tested individually.
Each command crate should own its integration tests and command-specific
fixtures.

## Deferred topics

- sorting commands
- temporary file handling
- external merge
- compatibility audit against historical behavior per command
