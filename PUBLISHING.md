# Publishing the Teamy Figue fork

The current publication target is `teamy-figue 6.0.0-rc.5`. It reuses the already
published `teamy-figue-attrs 6.0.0-rc.3`; do not publish a new attributes version
for this dependency-only release. These are TeamDman fork packages, not upstream
releases.

First publish the compatible Teamy dependency closure: Facet core
`0.50.0-rc.7` and Format/JSON `0.50.0-rc.9`, from the sibling `facet` and
`facet-format` forks. Each Figue manifest uses exact Teamy package versions
alongside local paths. Cargo replaces local path dependencies with the declared
registry dependencies when packaging; missing registry packages must be published
first.

Once those packages are available, inspect the archive and publish:

```powershell
cargo package -p teamy-figue --list
cargo publish -p teamy-figue --dry-run
cargo publish -p teamy-figue
```

Wait for crates.io to make each dependency available before packaging its
consumers. Review the changes, versions, fork README files, and package
contents before executing the publishing commands. Do not bypass package
verification to conceal a dependency mismatch. When the attribute grammar itself
changes, publish its new version before the Figue version that depends on it.

Local development uses sibling fork paths; downstream consumers use the
package aliases shown in the README and need no root-level patch. `target/`
is build output and must remain ignored rather than entering a release.

## Validation scope

Run the repository tests against the sibling source trees before publishing:

```powershell
cargo test --workspace --features teamy-figue/arbitrary --lib --tests -- --test-threads=1
```

Tests are serialized because the environment-substitution suite changes process
variables. Internal Teamy development dependencies use paths without registry
versions, so Cargo omits them from normalized archives. This breaks publication
cycles through test helpers while preserving the source-tree tests. Published
normal dependencies still have exact versions.

Isolated archive verification checks the library against the declared registry
graph; it does not rerun the repository's sibling-dependent test suite. Retain
both source-tree test results and archive verification evidence for a release.
