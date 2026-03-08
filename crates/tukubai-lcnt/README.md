# lcnt

`lcnt` counts the number of records (lines) in the input.

A record is defined according to the repository-wide parsing rules
described in `SPEC.md`.

In particular:

- records are separated by `0x0A` (LF)
- every record must be LF-terminated
- if the final record is not terminated by LF, the command exits with an error

This command is equivalent to counting LF-terminated records.

---

## Usage

```

lcnt [-f] [file ...]

```

If no file is given, input is read from standard input.

If `-` is given as a file name, it means standard input.

---

## Options

### `-f`

Print the file name before the count.

When the input source is standard input, the file name is printed as `-`.

Without `-f`, only the count is printed.

---

## Output format

### default

```

<count>
```

### with `-f`

```
<filename> <count>
```

---

## Examples

Input files:

```
data1
data2
data3
```

### count lines

```
$ lcnt data1 data2 data3
3
2
4
```

### show file names

```
$ lcnt -f data1 data2 data3
data1 3
data2 2
data3 4
```

### read standard input explicitly

```
$ printf 'a\nb\n' | lcnt -
2
```

### show standard input name

```
$ printf 'a\nb\n' | lcnt -f -
- 2
```

---

## Input rules

`lcnt` follows the shared parsing model defined in `SPEC.md`.

Important implications:

* records are separated only by `0x0A`
* `0x0D` (CR) is treated as ordinary data
* `0x00` (NUL) is treated as ordinary data
* empty records are valid records

Example:

```
a

b
```

This input contains **3 records**.

---

## Error conditions

`lcnt` exits with an error if:

* the input does not end with `0x0A`
* an I/O error occurs while reading input

---

## Implementation notes

`lcnt` should count records while streaming input.

The implementation should:

* read input incrementally
* detect LF (`0x0A`)
* avoid loading entire files into memory
* follow the shared record model in `tukubai-core`
