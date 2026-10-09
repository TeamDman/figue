# Teamy Figue fork differences

This document describes `teamy-figue` **6.0.0-rc.1** relative to upstream
Figue **5.0.0-rc.6** at
[`4780161`](https://github.com/bearcove/figue/commit/47801613b720a7d5a05a9c8222d90104331823e2).
The exact upstream base is also recorded in `workspace.metadata.teamy`.

## Package identity and versions

The published packages are `teamy-figue` and `teamy-figue-attrs`, with canonical
Rust library names `figue` and `figue_attrs`. Both use matching Teamy-controlled
versions, currently `6.0.0-rc.1`, against the exact Teamy Facet `0.50.0-rc.7`
family. Teamy version numbers do not promise interchangeability with upstream.
Breaking changes for fork consumers still require an appropriate fork release.

## Changes carried on teamy-main

- Help retains detailed Markdown documentation while presenting wrapped
  paragraph summaries in terminal output.
- Nested command help includes inherited options and handles subcommands
  literally named `command` without selecting the wrong help scope.
- Transparent scalar arguments retain their scalar parsing and configuration
  schema contracts.
- `to_args` serializes UTF-8 PathBuf values and preserves conversion errors.
- Arbitrary checking helpers do not impose an unrelated `Debug` requirement.
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
