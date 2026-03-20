# self (minimal implementation)

`self` extracts selected fields from byte-oriented tukubai records.

This minimal implementation covers only field selection.
Substring extraction and width-based processing are intentionally out of scope.

## Scope

The minimal implementation supports:

* selecting fields by number
* reordering fields
* inclusive ranges with `a/b`
* `NF` and `NF-<n>`
* one input source: `[<file>]` or standard input

The minimal implementation does not support:

* substring selectors such as `1.4` or `2.1.4`
* `-d`
* `-f`
* `-n<string>`

## Usage

```bash
self <selector>... [<file>]
```

If `<file>` is omitted or `-`, input is read from standard input.
Only one input file is accepted.
If the last argument is not a valid selector token, it is interpreted as `<file>`.

## Parsing Model

`self` follows the repository-wide parsing rules from `SPEC.md`.

* records are separated by `0x0A` (LF)
* every input record must end with `0x0A`
* `0x20` (SPACE) separates fields
* leading and trailing `0x20` bytes are ignored
* one or more `0x20` bytes act as a single separator
* TAB (`0x09`), CR (`0x0D`), NUL (`0x00`), and non-UTF-8 bytes are ordinary data

Field parsing is equivalent to:

```rust
tukubai_core::split_fields(record)
```

## Selector Syntax

Supported selector terms:

* `<digits>`: field number
* `0`: raw record bytes
* `NF`: last field
* `NF-<digits>`: field counted from the end
* `<expr>/<expr>`: inclusive range

Selector syntax rules:

* only uppercase `NF` is accepted
* `NF+<n>` is not accepted
* if `NF` is not used, the selector must contain only ASCII digits
* `01` is accepted and interpreted numerically
* `1+2`, `4-3`, and `+5` are not accepted
* `/5` and `3/` are not accepted
* `0` must not appear inside a range such as `0/3` or `2/0`

Examples:

* `1` -> first field
* `NF` -> last field
* `NF-0` -> last field
* `NF-1` -> second-to-last field
* `2/5` -> fields 2, 3, 4, 5
* `5/2` -> fields 5, 4, 3, 2
* `NF-1/NF` -> last two fields

## Selector Semantics

Field numbers are 1-based.

`NF-<n>` is resolved independently for each record.

`0` means the raw record bytes before `split_fields`.
Many other commands will not support `0`.

Ranges are inclusive.
If the start is greater than the end, the range expands in reverse order.

Selectors are processed in the order given.
Duplicates are preserved, so `2 2 2` and `2/4 3` emit repeated fields.

## Output

Each input record produces one output line.

Ordinary fields are joined with a single `0x20` byte.

If `0` appears together with ordinary field selectors, the raw record is written
at that position in the selector order, and selector terms are separated by a
single `0x20` byte in the final output.

## Errors

The command exits with an error if:

* the final input record is not terminated by `0x0A`
* any selector is syntactically invalid
* any selector resolves to a non-existent field
* `NF-<n>` resolves to `0`
* a record is empty and any selector other than `0` is used

Errors are fatal for the whole command.
If one input record fails selector resolution, processing stops.

## Examples

Select specific fields:

```bash
self 4 2 data
```

Read from standard input:

```bash
printf 'a b c\n' | self 2 1
```

Output the raw record:

```bash
self 0 data
```

Combine the raw record with selected fields:

```bash
self 4 0 data
```

Select a range:

```bash
self 2/5 data
```

Use `NF` expressions:

```bash
self 1 NF-3 NF data
```

Use `-` to read standard input explicitly:

```bash
printf 'a b\n' | self 1 -
```

## Shared Selection Model

This field-selection grammar is intended to be shared across multiple command
crates.

The shared layer can include:

* selector parsing
* `NF` resolution against the current record
* missing-field validation

Support for selector `0` should be configurable, for example through an enum
or option in shared code.

Substring selectors such as `1.4` are specific to `self` and are not part of
the shared field-selection grammar.
