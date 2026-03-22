% TUKUBAI-PARSING(7) open-tukubai-in-rust | Miscellaneous Information Manual
% open-tukubai-in-rust
% March 22, 2026

# NAME

tukubai-parsing - shared byte-oriented parsing model for open-tukubai-in-rust

# DESCRIPTION

`tukubai-parsing` describes the shared input parsing rules used by
open-tukubai-in-rust commands.

Input is treated as an arbitrary byte stream. Character encoding is not
interpreted.

# RECORDS

Records are separated only by LF (`0x0A`).

The LF byte is not part of the record payload.

Every record must end with LF (`0x0A`).

If the final record is not terminated by LF, the command fails.

# FIELDS

Within each record:

- leading SPACE bytes (`0x20`) are ignored
- trailing SPACE bytes (`0x20`) are ignored
- one or more SPACE bytes (`0x20`) separate fields

Examples:

```text
"a b c" -> ["a", "b", "c"]
" a b " -> ["a", "b"]
"   " -> []
"" -> []
```

# SPECIAL BYTES

Only two bytes have structural meaning:

- LF (`0x0A`) as the record separator
- SPACE (`0x20`) as the field separator

All other bytes are treated as ordinary data.

This includes:

- CR (`0x0D`)
- TAB (`0x09`)
- NUL (`0x00`)
- non-UTF-8 bytes

# STREAMING

Commands are expected to process input incrementally and must not require the
entire input to be loaded into memory.

# SEE ALSO

**lcnt**(1), **self**(1), **delf**(1), **tukubai-selectors**(7)
