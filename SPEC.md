# SPEC.md

## 1. Purpose

This document defines the shared parsing model used by commands in
open-tukubai-in-rust.

## 2. Input model

Input is treated as an arbitrary byte stream.

Character encoding is not interpreted.

## 3. Record model

Records are separated by the byte:

```

0x0A

```

The byte `0x0A` is not part of the record payload.

Every record must end with `0x0A`.

If the final record is not terminated by `0x0A`, the command must exit with an error.

## 4. Field model

Within each record:

- leading `0x20` bytes are ignored
- trailing `0x20` bytes are ignored
- one or more `0x20` bytes separate fields

Examples:

```

"a b c" -> ["a","b","c"]

" a b " -> ["a","b"]

"   " -> []

"" -> []

```

## 5. Special bytes

Only two bytes have structural meaning:

```

0x0A
0x20

```

All other bytes are treated as ordinary data.

Examples of ordinary data bytes:

```

0x00 (NUL)
0x09 (TAB)
0x0D (CR)

```

These bytes must not receive special treatment.

## 6. Streaming requirement

Commands must support large inputs.

Implementations should process input incrementally rather than loading
entire files into memory.

No fixed limit on record length or input size is defined by this specification.

## 7. Scope

This document defines shared parsing behavior only.

Command-specific semantics must be documented separately.
