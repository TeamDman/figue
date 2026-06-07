+++
title = "Fork Differences"
weight = 1
insert_anchor_links = "heading"
+++

This page is the canonical list of intentional `teamy-figue` behavior that
differs from upstream `figue`.

If we add fork-only behavior, we should update this page in the same change. If
upstream later gains equivalent behavior, we should either remove the item or
note that the fork no longer diverges there.

## Package identity

- The published crate is `teamy-figue`, typically consumed as:

```toml
[dependencies]
figue = { package = "teamy-figue", version = "4" }
```

## Behavior differences

### Long-form CLI flag aliases for named args

The fork supports repeatable `args::long_alias` attributes on named arguments:

```rust,noexec
#[facet(
    args::named,
    rename = "drive",
    args::long_alias = "drive-letter-pattern",
)]
drive: Option<String>,
```

This is a fork-specific extension intended for compatibility migrations where
one field should accept multiple long flag spellings.

Behavior:

- `--drive` and `--drive-letter-pattern` both parse into the same field.
- The canonical name remains the main one used for generated args and help;
  aliases are shown after it.
- Completions include aliases.
- Unknown-flag suggestions consider aliases.
- Parent/subcommand flag lookup honors aliases.
- Boolean `--no-...` negation works for aliases too.
- Schema validation rejects duplicate aliases and alias collisions with other
  canonical long names or aliases.

### Schema-driven `to_args` support

The fork includes schema-driven `to_args` support and roundtrip helpers beyond
what upstream currently provides.

### Arbitrary-based test helpers

The fork includes arbitrary-based test helpers and default arbitrary test
configs used by Teamy projects.

### Transparent CLI arg and roundtrip fixes

The fork carries fixes for transparent CLI args and `to_args` roundtrips that
Teamy consumers rely on.

### Richer help UX

The fork includes additional help behavior, including:

- `help` pseudo-commands
- recursive help listing
- implementation/source-file hints in help output

### Windows-oriented fixes

The fork carries Windows-focused fixes around program-name normalization and
snapshot behavior.
