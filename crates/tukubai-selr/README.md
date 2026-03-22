# selr

`selr` filters records by exact field equality.

## Usage

```bash
selr <fldnum> <str> [<file>]
```

If `<file>` is omitted or `-`, input is read from standard input.
Only one input source is accepted.

## Selector Syntax

`<fldnum>` uses the shared single-field selector syntax.

Supported forms:

* `<digits>`
* `NF`
* `NF-<digits>`

Not supported:

* selector `0`
* ranges such as `a/b`

Leading zeroes are accepted, so `01` means field 1.

## Match Semantics

The input record is split using the repository-wide byte-oriented field rules.

`selr` outputs only records whose selected field is exactly equal to `<str>`.

Comparison is byte-exact:

* no substring matching
* no regular expressions
* no Unicode normalization
* no locale-aware behavior

`<str>` is treated as an arbitrary byte sequence from the command line.

If `<str>` is the empty byte string, `selr` outputs every input record as a
special case.

## Output

Matching records are written using the original record bytes.

This means `selr` preserves:

* leading spaces
* trailing spaces
* repeated internal spaces

The command does not reconstruct output from parsed fields.

## Errors

The command exits with an error if:

* the final input record is not terminated by `0x0A`
* `<fldnum>` is syntactically invalid
* `<fldnum>` uses selector `0`
* `<fldnum>` uses a range such as `a/b`
* the selected field does not exist for any record, unless `<str>` is empty

Errors are fatal for the whole command.

## Examples

```bash
selr 2 beta data
printf 'a b\nc d\n' | selr 2 d
selr NF done data
printf '  a  b  \n' | selr 2 b
printf '\n a\n' | selr 1 ''
```
