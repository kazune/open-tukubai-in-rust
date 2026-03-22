# man-policy.md

This document defines the repository policy for manual pages.

## Goals

Manual pages in this repository should:

- document user-facing command behavior
- stay consistent with the implemented CLI
- avoid duplicating shared parsing and selector rules across command pages
- be easy to review as ordinary text files

## Source format

Manual page sources are written in Markdown and converted with `pandoc`.

Do not write roff sources directly in this repository.

## Source layout

Manual page sources live under:

```text
docs/man-src/
```

Naming rules:

- command pages use `docs/man-src/<name>.<section>.md`
- shared reference pages use `docs/man-src/<topic>.<section>.md`

Examples:

```text
docs/man-src/lcnt.1.md
docs/man-src/self.1.md
docs/man-src/delf.1.md
docs/man-src/tukubai-parsing.7.md
docs/man-src/tukubai-selectors.7.md
```

## Generated output

Generated manual pages are written under:

```text
docs/man/
```

Generated files are not tracked by git.

The repository ignores generated outputs such as:

```text
docs/man/*.1
docs/man/*.7
```

## Build flow

Generate manual pages from the workspace root with:

```sh
make man
```

This uses `pandoc` to convert sources in `docs/man-src/` into man pages in
`docs/man/`.

Lint generated manual pages with:

```sh
make man-lint
```

Run generation and lint together with:

```sh
make man-check
```

`make man-lint` runs `mandoc -Tlint` against generated man pages under
`docs/man/`.

## Section usage

Use section `1` for user commands.

Use section `7` for shared reference material such as parsing and selector
rules.

## Writing rules

Command manual pages should focus on:

- command purpose
- supported syntax and operands
- command-specific behavior
- output and error behavior
- practical examples

Shared rules should be factored into common section `7` pages when they are
used by multiple commands.

Current shared pages:

- `tukubai-parsing(7)` for shared byte-oriented parsing rules
- `tukubai-selectors(7)` for the shared selector model

Command pages should summarize only the subset they support and refer readers
to the shared page for the full model.

## Minimum format rules

Manual page sources should follow these minimum formatting rules:

- start with a three-line pandoc title block
- use a sectioned file name such as `<name>.1.md` or `<topic>.7.md`
- keep the `NAME` section in the form `name - one line summary`
- use uppercase top-level section names such as `NAME`, `SYNOPSIS`, and
  `DESCRIPTION`
- keep `SYNOPSIS` compact and limited to supported syntax only
- write examples as executable shell snippets
- use `SEE ALSO` only for directly related commands or shared section `7` pages

Command pages should usually include:

- `NAME`
- `SYNOPSIS`
- `DESCRIPTION`
- command-specific sections such as `OPTIONS`, `OPERANDS`, `SELECTORS`, or
  `OUTPUT`
- `EXIT STATUS`
- `EXAMPLES`

Shared section `7` pages should usually include:

- `NAME`
- `DESCRIPTION`
- the shared rule sections needed for the topic
- `SEE ALSO`

Prefer short command-specific summaries over repeating large blocks of shared
material.

## Consistency rules

Manual pages must stay consistent with:

- `SPEC.md` for shared parsing behavior
- command implementations in each crate
- command integration tests

If a command behavior changes, update the corresponding man page source in the
same change.

If a shared parsing or selector rule changes, update the corresponding section
`7` manual page source in the same change.
