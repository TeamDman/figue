# Teamy Figue

`teamy-figue` and `teamy-figue-attrs` are maintained by TeamDman as a fork of
[upstream Figue](https://github.com/bearcove/figue), originally created by
Amos Wenger. These packages are separate from the upstream `figue` and
`figue-attrs` packages. Use upstream Figue when you do not need this fork.

The maintained fork branch is [`teamy-main`](https://github.com/TeamDman/figue/tree/teamy-main).
The old published branch is retained as `legacy/published-main-2026-10-09`.
The current upstream base is Figue **5.0.0-rc.6**, at commit
[`4780161`](https://github.com/bearcove/figue/commit/47801613b720a7d5a05a9c8222d90104331823e2).
The fork release being prepared is **6.0.0-rc.1**; its version is independent
of the upstream release. These versions are publication targets until they
appear on crates.io.

## Dependencies

This release uses the separately named **Teamy Facet 0.50.0-rc.7** package
family. Keep one compatible reflection implementation throughout your
application. Alias the packages to their canonical Rust dependency names:

```toml
[dependencies]
facet = { package = "teamy-facet", version = "=0.50.0-rc.7" }
figue = { package = "teamy-figue", version = "=6.0.0-rc.1" }
```

Rust imports remain `facet` and `figue`, including `use figue as args` for
attributes. Other reflection dependencies must use matching Teamy packages,
for example `facet-json = { package = "teamy-facet-json", version = "=0.50.0-rc.7" }`.
Upstream and Teamy Facet traits and shapes belong to different package
identities; a type derived against upstream Facet cannot be passed directly
to this fork's Figue APIs. Consumers need no `[patch.crates-io]` section.
`teamy-figue` re-exports its attribute grammar; applications usually do not
need to depend on `teamy-figue-attrs` directly.

For local development, clone `TeamDman/facet`, `TeamDman/facet-format`, and
`TeamDman/figue` into sibling directories named `facet`, `facet-format`, and
`figue`, using each fork's `teamy-main` branch. Manifest dependencies have both
local paths and exact registry versions so published packages use the same
Teamy graph.

## Behavior and documentation

The current fork adds detailed Markdown help summaries, inherited option
help, transparent scalar and PathBuf argument handling, and corrections for
nested subcommands named `command`. [Fork differences](FORK_DIFFERENCES.md)
records those changes and separates behavior now shared with upstream.

- [Fork API documentation](https://docs.rs/teamy-figue)
- [Crate README and example](figue/README.md)
- [Upstream guide](https://figue.bearcove.eu), for behavior shared with upstream
- [Publication instructions](PUBLISHING.md)

Figue combines command-line arguments, environment variables, configuration
files, and defaults using Facet reflection. The upstream project and its
contributors retain their attribution and original MIT/Apache-2.0 licenses.
