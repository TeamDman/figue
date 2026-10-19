# Teamy Figue fork differences

This document describes `teamy-figue` **6.0.0-rc.4** relative to upstream
Figue **5.0.0-rc.6** at
[`4780161`](https://github.com/bearcove/figue/commit/47801613b720a7d5a05a9c8222d90104331823e2).
The exact upstream base is also recorded in `workspace.metadata.teamy`.

## Package identity and versions

The published packages are `teamy-figue` and `teamy-figue-attrs`, with canonical
Rust library names `figue` and `figue_attrs`. Figue is `6.0.0-rc.4`; its
unchanged attributes companion remains `6.0.0-rc.3`. Core Facet uses
`0.50.0-rc.7`, while Format and JSON use `0.50.0-rc.8` for cooperative
serialization cancellation and caller-owned writer support. Teamy version numbers do not promise interchangeability with upstream.
Breaking changes for fork consumers still require an appropriate fork release.

## Changes carried on teamy-main

- Terminal and HTML help reject invalid command paths instead of describing a
  partially parsed command. Valid help requests still bypass invalid work
  arguments and configuration, and version requests remain path-independent.
- Help retains detailed Markdown documentation while presenting wrapped
  paragraph summaries in terminal output.
- Nested command help includes inherited options and handles subcommands
  literally named `command` without selecting the wrong help scope.
- Transparent scalar arguments retain their scalar parsing and configuration
  schema contracts.
- `to_args` serializes UTF-8 PathBuf values and preserves conversion errors.
- Arbitrary checking helpers compare reflected structure without requiring CLI
  schemas to implement `PartialEq` or `Debug`; comparison failures identify the
  differing or unsupported field.
- Driver outcomes and outputs can map the parsed value while retaining reports
  and configuration extraction metadata.
- Help can retain a consumer root type's documentation and command source hints
  when an internal parsing wrapper adds builtin flags.
- `FigueBuiltins` implements `PartialEq` and `Eq`.
- The dependency graph uses TeamDman's Facet fork, including reflected Cow
  construction and native enum variant alias handling.

## Behavior now shared with upstream

Earlier Teamy releases introduced named flag aliases, subcommand aliases,
optional-value named flags represented by `Option<Option<T>>`, schema-driven
`to_args`, arbitrary checking helpers, recursive `help list`, implementation
source hints, and Windows program-name handling. Much of this behavior is now
present in the newer upstream base restored from the Facet monorepo. These
features remain useful, but are not evidence that the current fork differs
from upstream in each case.

Keep this document current when fork behavior changes or an upstream release
incorporates an equivalent fix. The full source history is retained under the
legacy archive branch; the active workspace uses the upstream standalone
layout rather than the older bundled `crates/facet-format` copy.
