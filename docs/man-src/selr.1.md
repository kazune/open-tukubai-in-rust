% SELR(1) open-tukubai-in-rust | User Commands
% open-tukubai-in-rust
% March 23, 2026

# NAME

selr - select records by exact field equality

# SYNOPSIS

**selr** *FLDNUM* *STR* [*FILE*]

# DESCRIPTION

`selr` filters byte-oriented tukubai records by comparing one selected field
with a target byte string.

Input is processed using the shared open-tukubai-in-rust parsing rules
described in **tukubai-parsing**(7).

This implementation does not support the `--through` option.

If *FILE* is omitted or `-`, input is read from standard input.

Only one input file is accepted.

Matching records are written using the original record bytes. `selr` does not
reconstruct output from parsed fields, so leading spaces, trailing spaces, and
repeated internal spaces are preserved.

Comparison is byte-exact. `selr` does not perform substring matching, regular
expression matching, Unicode normalization, or locale-aware comparison.

If *STR* is empty, `selr` writes every input record without evaluating
*FLDNUM*.

# SELECTORS

`selr` accepts one shared single-field selector.

Supported forms:

- field numbers such as `1` and `03`
- `NF`
- `NF-<n>`

Not supported:

- `--through`
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

Each matching input record is written exactly as read, followed by LF
(`0x0A`).

# EXIT STATUS

`0`
: Success.

`1`
: The final input record is not LF-terminated, the selector is invalid, the
selector resolves to a non-existent field, an input file cannot be opened, or
another I/O error occurs.

# EXAMPLES

Select records whose second field is `beta`:

```sh
selr 2 beta data
```

Read from standard input:

```sh
printf 'a b\nc d\n' | selr 2 d
```

Match the last field:

```sh
selr NF done data
```

Preserve original spacing in matching records:

```sh
printf '  a  b  \n' | selr 2 b
```

Pass all records when the target string is empty:

```sh
printf '\n a\n' | selr 1 ''
```

# SEE ALSO

**delr**(1), **tukubai-parsing**(7), **tukubai-selectors**(7)
