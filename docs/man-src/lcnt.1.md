% LCNT(1) open-tukubai-in-rust | User Commands
% open-tukubai-in-rust
% March 22, 2026

# NAME

lcnt - count LF-terminated records

# SYNOPSIS

**lcnt** [**-f**] [*FILE*...]

# DESCRIPTION

`lcnt` counts the number of records in each input.

Input is processed using the shared open-tukubai-in-rust parsing rules
described in **tukubai-parsing**(7).

If no file is given, input is read from standard input.

If `-` is given as a file name, it means standard input.

Without `-f`, only the count is printed.

With `-f`, the file name is printed before the count. When the input source is
standard input, the file name is printed as `-`.

# OPTIONS

**-f**
: Print the file name before the count.

# OPERANDS

**FILE**
: Input file path. `-` means standard input.

# EXIT STATUS

`0`
: Success.

`1`
: The input is not LF-terminated, an input file cannot be opened, or another
I/O error occurs.

# EXAMPLES

Count records from standard input:

```sh
printf 'a\nb\n' | lcnt
```

Count records from files:

```sh
lcnt data1 data2 data3
```

Show file names with counts:

```sh
lcnt -f data1 data2 data3
```

Read standard input explicitly and print its source name:

```sh
printf 'a\nb\n' | lcnt -f -
```

# SEE ALSO

**tukubai-parsing**(7)
