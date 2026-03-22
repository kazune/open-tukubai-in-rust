# delr

`delr` filters records by exact field inequality.

## Usage

```bash
delr <fldnum> <str> [<file>]
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

`delr` outputs only records whose selected field is not exactly equal to `<str>`.

Comparison is byte-exact:

* no substring matching
* no regular expressions
* no Unicode normalization
* no locale-aware behavior

`<str>` is treated as an arbitrary byte sequence from the command line.

If `<str>` is the empty byte string, `delr` outputs every input record as a
special case.

## Output

Matching records are written using the original record bytes.

This means `delr` preserves:

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
delr 2 beta data
printf 'a b\nc d\n' | delr 2 d
delr NF done data
printf '  a  b  \n' | delr 2 x
printf '\n a\n' | delr 1 ''
```
