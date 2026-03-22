% DELF(1) open-tukubai-in-rust | User Commands
% open-tukubai-in-rust
% March 22, 2026

# NAME

delf - remove selected fields from byte-oriented records

# SYNOPSIS

**delf** *SELECTOR*... [*FILE*]

# DESCRIPTION

`delf` removes selected fields from byte-oriented tukubai records.

This minimal implementation supports field removal only. Selector `0`,
substring selectors such as `1.4` or `2.1.4`, `-d`, `-f`, and `-n<string>` are
not supported.

Input is processed using the shared open-tukubai-in-rust parsing rules
described in **tukubai-parsing**(7).

If *FILE* is omitted or `-`, input is read from standard input.

Only one input file is accepted. If the last argument is not a valid selector
token, it is interpreted as *FILE*.

# SELECTORS

`delf` supports these selector forms:

- field numbers such as `1` and `3`
- `NF` and `NF-<n>`
- inclusive ranges such as `2/5` and `NF-1/NF`

`delf` does not support selector `0`.

Selectors identify fields to remove. The output keeps the remaining fields in
their original left-to-right order.

If the same field is selected more than once, it is still removed only once.

See **tukubai-selectors**(7) for the shared selector model.

# OPERANDS

**SELECTOR**
: Selector token to apply to each record.

**FILE**
: Input file path. `-` means standard input.

# OUTPUT

Each input record produces one output line.

Remaining fields are joined with a single SPACE byte (`0x20`).

If all fields are removed, the output for that record is an empty line
containing only LF (`0x0A`).

# EXIT STATUS

`0`
: Success.

`1`
: The final input record is not LF-terminated, a selector is invalid, a
selector resolves to a non-existent field, an input file cannot be opened, or
another I/O error occurs.

# EXAMPLES

Remove one field:

```sh
delf 2 data
```

Read from standard input:

```sh
printf 'a b c\n' | delf 2
```

Remove a range:

```sh
delf 2/5 data
```

Use `NF` expressions:

```sh
delf 1 NF-1 data
```

Remove all fields:

```sh
printf 'a b\n' | delf 1 2
```

Read standard input explicitly with `-`:

```sh
printf 'a b\n' | delf 1 -
```

# SEE ALSO

**self**(1), **tukubai-parsing**(7), **tukubai-selectors**(7)
