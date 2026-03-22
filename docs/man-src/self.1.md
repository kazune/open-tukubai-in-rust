% SELF(1) open-tukubai-in-rust | User Commands
% open-tukubai-in-rust
%

# NAME

self - extract selected fields from byte-oriented records

# SYNOPSIS

**self** *SELECTOR*... [*FILE*]

# DESCRIPTION

`self` extracts selected fields from byte-oriented tukubai records.

This minimal implementation supports field selection only. Substring selectors
such as `1.4` or `2.1.4`, `-d`, `-f`, and `-n<string>` are not supported.

Input is processed using the shared open-tukubai-in-rust parsing rules
described in **tukubai-parsing**(7).

If *FILE* is omitted or `-`, input is read from standard input.

Only one input file is accepted. If the last argument is not a valid selector
token, it is interpreted as *FILE*.

# SELECTORS

`self` supports these selector forms:

- field numbers such as `1` and `3`
- `0` for the raw record
- `NF` and `NF-<n>`
- inclusive ranges such as `2/5` and `NF-1/NF`

`self` processes selectors in the order given and preserves duplicates.

`self` does not support substring selectors such as `1.4` or `2.1.4`.

See **tukubai-selectors**(7) for the shared selector model.

# OPERANDS

**SELECTOR**
: Selector token to apply to each record.

**FILE**
: Input file path. `-` means standard input.

# OUTPUT

Each input record produces one output line.

Ordinary fields are joined with a single SPACE byte (`0x20`).

If `0` appears together with ordinary field selectors, the raw record is
written at that position in the selector order and selector terms are separated
by a single SPACE byte in the final output.

# EXIT STATUS

`0`
: Success.

`1`
: The final input record is not LF-terminated, a selector is invalid, a
selector resolves to a non-existent field, an input file cannot be opened, or
another I/O error occurs.

# EXAMPLES

Select specific fields:

```sh
self 4 2 data
```

Read from standard input:

```sh
printf 'a b c\n' | self 2 1
```

Output the raw record:

```sh
self 0 data
```

Combine the raw record with selected fields:

```sh
self 2 0 data
```

Select a range:

```sh
self 2/5 data
```

Use `NF` expressions:

```sh
self 1 NF-1 NF data
```

Read standard input explicitly with `-`:

```sh
printf 'a b\n' | self 1 -
```

# SEE ALSO

**delf**(1), **tukubai-parsing**(7), **tukubai-selectors**(7)
