% DELR(1) open-tukubai-in-rust | User Commands
% open-tukubai-in-rust
% March 23, 2026

# NAME

delr - delete records by exact field equality

# SYNOPSIS

**delr** *FLDNUM* *STR* [*FILE*]

# DESCRIPTION

`delr` filters byte-oriented tukubai records by removing records whose
selected field is exactly equal to a target byte string.

Input is processed using the shared open-tukubai-in-rust parsing rules
described in **tukubai-parsing**(7).

If *FILE* is omitted or `-`, input is read from standard input.

Only one input file is accepted.

Records that remain are written using the original record bytes. `delr` does
not reconstruct output from parsed fields, so leading spaces, trailing spaces,
and repeated internal spaces are preserved.

Comparison is byte-exact. `delr` does not perform substring matching, regular
expression matching, Unicode normalization, or locale-aware comparison.

If *STR* is empty, `delr` writes every input record without evaluating
*FLDNUM*.

# SELECTORS

`delr` accepts one shared single-field selector.

Supported forms:

- field numbers such as `1` and `03`
- `NF`
- `NF-<n>`

Not supported:

- selector `0`
- ranges such as `2/5`

See **tukubai-selectors**(7) for the shared selector model.

# OPERANDS

**FLDNUM**
: Selector token to resolve against each record.

**STR**
: Byte string to compare with the selected field.

**FILE**
: Input file path. `-` means standard input.

# OUTPUT

Each non-matching input record is written exactly as read, followed by LF
(`0x0A`).

# EXIT STATUS

`0`
: Success.

`1`
: The final input record is not LF-terminated, the selector is invalid, the
selector resolves to a non-existent field, an input file cannot be opened, or
another I/O error occurs.

# EXAMPLES

Remove records whose second field is `beta`:

```sh
delr 2 beta data
```

Read from standard input:

```sh
printf 'a b\nc d\n' | delr 2 d
```

Match against the last field:

```sh
delr NF done data
```

Preserve original spacing in remaining records:

```sh
printf '  a  b  \n' | delr 2 x
```

Pass all records when the target string is empty:

```sh
printf '\n a\n' | delr 1 ''
```

# SEE ALSO

**selr**(1), **tukubai-parsing**(7), **tukubai-selectors**(7)
