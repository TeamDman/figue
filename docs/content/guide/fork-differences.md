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
figue = { package = "teamy-figue", version = "5" }
```

## Versioning policy

- `teamy-figue` uses Teamy-controlled semver rather than mirroring upstream
  `figue` release numbers.
- The fork stays one major version ahead of the upstream `figue` major to make
  the distinction obvious at a glance.
- The exact upstream base for a release is recorded in the workspace metadata
  at `workspace.metadata.teamy.upstream_figue`.
- Breaking changes for `teamy-figue` users still require a `teamy-figue` major
  bump, regardless of the upstream base.

## Behavior differences

### Long-form CLI flag aliases for named args

The fork supports repeatable `args::alias` attributes on named arguments:

```rust,noexec
#[facet(
    args::named,
    args::alias = "drive-letter-pattern",
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

### Long-form subcommand aliases

The fork supports repeatable `args::alias` attributes on subcommand variants:

```rust,noexec
#[derive(Facet, Debug)]
#[repr(u8)]
enum Command {
    #[facet(args::alias = "profiles")]
    Profile,
}
```

This is a fork-specific extension intended for command migrations where one
variant should accept an old and new CLI spelling.

Behavior:

- `profile` and `profiles` both select the same enum variant.
- The canonical subcommand name remains the one used for generated args and
  help; aliases are shown after it.
- Completions include aliases.
- Unknown-subcommand suggestions consider aliases.
- Alias spellings work at any nesting level where the subcommand appears.
- Schema validation rejects duplicate aliases on one variant and collisions with
  other canonical subcommand names or aliases.

### Schema-driven `to_args` support

The fork includes schema-driven `to_args` support and roundtrip helpers beyond
what upstream currently provides.

### Optional-value named CLI args

The fork treats `Option<Option<T>>` on a named, non-bool, single-value CLI field
as an optional-value flag:

```rust,noexec
#[derive(Facet)]
struct Args {
    #[facet(args::named)]
    parallel: Option<Option<usize>>,
}
```

Behavior:

- an absent `--parallel` parses as `None`
- bare `--parallel` parses as `Some(None)`
- `--parallel 12` and `--parallel=12` parse as `Some(Some(12))`
- `--parallel --dry-run` parses the bare parallel flag and leaves `--dry-run`
  available as another flag
- dash-prefixed values use equals form, such as `--parallel=-3`
- `to_args` emits nothing for `None`, `--parallel` for `Some(None)`, and
  `--parallel=12` for `Some(Some(12))`

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
