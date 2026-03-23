# delf (minimal implementation)

`delf` removes selected fields from byte-oriented tukubai records.

This minimal implementation covers only field removal.
Substring extraction and width-based processing are intentionally out of scope.

## Scope

The minimal implementation supports:

* removing fields by number
* removing fields selected by inclusive ranges with `a/b`
* `NF` and `NF-<n>`
* one input source: `[<file>]` or standard input

The minimal implementation does not support:

* selector `0`
* option `-d`

## Usage

```bash
delf <selector>... [<file>]
```

If `<file>` is omitted or `-`, input is read from standard input.
Only one input file is accepted.
If the last argument is not a valid selector token, it is interpreted as `<file>`.

## Parsing Model

`delf` follows the repository-wide parsing rules from `SPEC.md`.

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
* `NF`: last field
* `NF-<digits>`: field counted from the end
* `<expr>/<expr>`: inclusive range

Selector syntax rules:

* only uppercase `NF` is accepted
* selector `0` is not accepted
* `NF+<n>` is not accepted
* if `NF` is not used, the selector must contain only ASCII digits
* `01` is accepted and interpreted numerically
* `1+2`, `4-3`, and `+5` are not accepted
* `/5` and `3/` are not accepted

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

Ranges are inclusive.
If the start is greater than the end, the range expands in reverse order.

Selectors identify fields to remove.
The output keeps the remaining fields in their original left-to-right order.

Selectors are processed as a removal set.
If the same field is selected more than once, it is still removed only once.

## Output

Each input record produces one output line.

Remaining fields are joined with a single `0x20` byte.

If all fields are removed, the output for that record is an empty line containing
only `0x0A`.

## Errors

The command exits with an error if:

* the final input record is not terminated by `0x0A`
* any selector is syntactically invalid
* any selector resolves to a non-existent field
* `NF-<n>` resolves to `0`
* a record is empty

Errors are fatal for the whole command.
If one input record fails selector resolution, processing stops.

## Examples

Remove one field:

```bash
delf 2 data
```

Read from standard input:

```bash
printf 'a b c\n' | delf 2
```

Remove a range:

```bash
delf 2/5 data
```

Use `NF` expressions:

```bash
delf 1 NF-1 data
```

Remove all fields:

```bash
printf 'a b\n' | delf 1 2
```

Use `-` to read standard input explicitly:

```bash
printf 'a b\n' | delf 1 -
```

## Shared Selection Model

This field-selection grammar is intended to be shared across multiple command
crates.

The shared layer can include:

* selector parsing
* `NF` resolution against the current record
* missing-field validation

Selector `0` is command-specific and is not supported by `delf`.
