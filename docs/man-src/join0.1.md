% JOIN0(1) open-tukubai-in-rust | User Commands
% open-tukubai-in-rust
% March 23, 2026

# NAME

join0 - filter transaction records by key existence in a master input

# SYNOPSIS

**join0** [**+ng**\ *FD*] **key=***KEY* *MASTER* [*TRAN*]

# DESCRIPTION

`join0` performs a semi-join style existence filter on two byte-oriented
inputs.

For each record in *TRAN*, `join0` resolves the transaction key specified by
*KEY*, maps that resolved field-position pattern onto *MASTER*, and tests
whether the same key exists in the current master stream.

If the key exists in *MASTER*, `join0` writes the original *TRAN* record.

If the key does not exist, the record is discarded unless **+ng**\ *FD* is
specified.

Input is processed using the shared open-tukubai-in-rust parsing rules
described in **tukubai-parsing**(7).

`-` means standard input.

If *TRAN* is omitted, `join0` reads transaction records from standard input.

Using standard input for both *MASTER* and *TRAN* is an error.

`join0` is a streaming command intended for inputs that are already sorted by
the specified key. Output order always follows the original *TRAN* order.

# OPTIONS

**+ng**\ *FD*
: Write non-matching *TRAN* records to file descriptor *FD*.

This option is accepted only immediately after the command name and before
**key=***KEY*.

# KEY SYNTAX

`key=<KEY>` is one command-line token. Splitting `key=` and the key text into
separate arguments is not supported.

`join0` supports these field forms inside *KEY*:

- single fields such as `2`, `NF`, and `NF-1`
- inclusive ranges such as `2/4`, `4/2`, and `NF-3/NF`
- composite keys such as `2@NF`

Each field term may be followed by an optional comparison attribute:

- no suffix: byte comparison, ascending sort order
- `n`: numeric comparison, ascending sort order
- `r`: byte comparison, descending sort order
- `nr`: numeric comparison, descending sort order

Examples:

- `2@NF`
- `2@NFn`
- `2n@NFr`
- `2/4`
- `2n/4n`
- `2nr/NFnr`

For range syntax `a/b`, both endpoints must use the same comparison
attributes.

Examples:

- `2/NF` is valid
- `2n/NFn` is valid
- `2r/NFr` is valid
- `2nr/NFnr` is valid
- `2/NFn` is invalid
- `2/NFr` is invalid
- `2n/NFr` is invalid

`NF` and `NF-<n>` are resolved separately for each record.

# KEY MAPPING

`join0` resolves *KEY* against each *TRAN* record first.

The resolved 1-based field positions are then translated so that the minimum
resolved position becomes field 1 on the *MASTER* side.

Examples:

- `key=3@5` means the *MASTER* key is `1@3`
- `key=NF-2/NF` on a three-field transaction record resolves to `1/3` on the
  *MASTER* side

This mapping uses resolved field positions, not the original selector text.

# COMPARISON

Composite keys are compared lexicographically.

`join0` compares the first key element first. If they differ, that result
determines ordering and equality. If they are equal, comparison proceeds to
the next key element.

Without `n`, fields are compared as raw byte sequences.

With `n`, fields are parsed as signed decimal integers or decimal fractions.
Exponent notation is not supported. Leading zeroes are allowed.

If a field marked with `n` is not a valid number, `join0` exits with a fatal
error.

`r` means the corresponding key element is sorted in descending order.

# OPERANDS

**KEY**
: Key selector program supplied as part of the single token `key=<KEY>`.

**MASTER**
: Master input path. `-` means standard input.

**TRAN**
: Transaction input path. If omitted, standard input is used. `-` also means
standard input.

# OUTPUT

Matching records are written exactly as read from *TRAN*, followed by LF
(`0x0A`).

`join0` does not reconstruct output from parsed fields, so leading spaces,
trailing spaces, and repeated internal spaces are preserved.

If **+ng**\ *FD* is specified, non-matching records are written to that file
descriptor in *TRAN* order.

# ASSUMPTIONS

`join0` assumes:

- *MASTER* and *TRAN* are both sorted by the specified key
- *MASTER* keys are unique

These conditions are not validated by the command. Behavior is undefined if
they are not satisfied.

# EXIT STATUS

`0`
: Success.

`1`
: The final input record is not LF-terminated, a key selector is invalid, a
key selector resolves to a non-existent field, a numeric key field is not a
valid number, both inputs would read from standard input, the **+ng** file
descriptor is invalid or unavailable, an input file cannot be opened, or
another I/O error occurs.

# EXAMPLES

Filter transaction records whose key exists in the master:

```sh
join0 key=2 master tran
```

Read transactions from standard input:

```sh
printf 'a 10\nb 20\n' | join0 key=1 master
```

Use a composite key:

```sh
join0 key=2@4 master tran
```

Use numeric comparison on one key element:

```sh
join0 key=1@2n master tran
```

Send non-matching transaction records to file descriptor 3:

```sh
join0 +ng3 key=2 master tran 3>ng.out
```

# SEE ALSO

**tukubai-parsing**(7), **tukubai-selectors**(7)
