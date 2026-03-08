# tests

Repository-level shared test assets.

## Layout

- `fixtures/`
  repository-wide fixtures reused across multiple crates
- `tmp/`
  temporary outputs created during local or automated test runs

`tmp/` is intentionally ignored by git.

Keep fixture files byte-exact.
Do not normalize line endings or text encoding.

Integration tests do not live here.
Place them under each crate's `tests/` directory instead.
