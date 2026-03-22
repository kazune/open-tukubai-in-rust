% TUKUBAI-SELECTORS(7) open-tukubai-in-rust | Miscellaneous Information Manual
% open-tukubai-in-rust
% March 22, 2026

# NAME

tukubai-selectors - shared selector model for open-tukubai-in-rust commands

# DESCRIPTION

`tukubai-selectors` describes the shared selector syntax used by selector-based
commands in open-tukubai-in-rust.

Commands may support only a subset of this model. In particular, support for
selector `0` and inclusive ranges is command-specific.

# SELECTOR FORMS

The shared selector model includes these forms:

`<digits>`
: Field number. Field numbers are 1-based.

`NF`
: Last field in the current record.

`NF-<digits>`
: Field counted from the end of the current record.

`<expr>/<expr>`
: Inclusive range. If the start is greater than the end, the range expands in
reverse order.

Some commands also support:

`0`
: Raw record bytes before field splitting.

# SYNTAX RULES

- Only uppercase `NF` is accepted.
- `NF+<n>` is invalid.
- If `NF` is not used, the selector must contain only ASCII digits.
- `01` is accepted and interpreted numerically.
- `1+2`, `4-3`, and `+5` are invalid.
- `/5` and `3/` are invalid.
- `0` must not appear inside a range such as `0/3` or `2/0`.

# SEMANTICS

Selectors are parsed once and resolved independently for each input record.

`NF-<n>` is resolved against the current record.

Ranges are inclusive.

Selectors are processed in the order given.

Duplicate selectors are preserved.

When a command supports selector `0`, it means the raw record bytes before
field splitting.

# ERRORS

A command may fail if:

- a selector is syntactically invalid
- a selector resolves to a non-existent field
- `NF-<n>` resolves to `0`
- the command does not support a selector form that appears in the input

Exact error handling and supported selector subsets are defined by each command.

# SEE ALSO

**self**(1), **delf**(1)
