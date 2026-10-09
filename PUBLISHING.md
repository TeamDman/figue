# Publishing the Teamy Figue fork

The publication targets are `teamy-figue-attrs` and `teamy-figue`, both at
`6.0.0-rc.1`. These are TeamDman fork packages, not upstream releases.

First publish the complete compatible Teamy Facet `0.50.0-rc.7` dependency
closure from the sibling `facet` and `facet-format` forks. Each Figue manifest
uses exact Teamy package versions alongside local paths. Cargo replaces local
path dependencies with the declared registry dependencies when packaging;
missing registry packages must be published first.

Once those packages are available, inspect each archive and publish in order:

```powershell
cargo package -p teamy-figue-attrs --list
cargo publish -p teamy-figue-attrs --dry-run
cargo publish -p teamy-figue-attrs
cargo package -p teamy-figue --list
cargo publish -p teamy-figue --dry-run
cargo publish -p teamy-figue
```

Wait for crates.io to make each dependency available before packaging its
consumers. Review the changes, versions, fork README files, and package
contents before executing the publishing commands. Do not bypass package
verification to conceal a dependency mismatch.

Local development uses sibling fork paths; downstream consumers use the
package aliases shown in the README and need no root-level patch. `target/`
is build output and must remain ignored rather than entering a release.

## Validation scope

Run the repository tests against the sibling source trees before publishing:

```powershell
cargo test -p teamy-figue --features arbitrary --lib --test main --test pathbuf_to_args -- --test-threads=1
```

Tests are serialized because the environment-substitution suite changes process
variables. Internal Teamy development dependencies use paths without registry
versions, so Cargo omits them from normalized archives. This breaks publication
cycles through test helpers while preserving the source-tree tests. Published
normal dependencies still have exact versions.

Isolated archive verification checks the library against the declared registry
graph; it does not rerun the repository's sibling-dependent test suite. Retain
both source-tree test results and archive verification evidence for a release.
