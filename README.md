ATTENTION: `teamy-figue` is a fork of `figue`; unless you specifically need the TeamDman fork behavior below, you probably want the upstream crate instead.

I am publishing this fork so I can publish `teamy-mft`, which depends on fork-specific behavior that is not available in upstream `figue`.

Notable differences in this fork include:

- first-class long-form CLI flag aliases for named args via `#[facet(args::long_alias = "...")]`
- long-form subcommand aliases via `#[facet(args::alias = "...")]`
- optional-value named args via `Option<Option<T>>`, so `--parallel`,
  `--parallel=12`, and an absent flag are distinct states
- schema-driven `to_args` support and roundtrip helpers
- arbitrary-based test helpers and default arbitrary test configs
- fixes for transparent CLI args and `to_args` roundtrips
- richer help UX, including `help` pseudo-commands, recursive help listing, and implementation/source-file hints in help output
- Windows-oriented fixes around program-name normalization and snapshot behavior

The canonical maintained list lives in [docs/content/guide/fork-differences.md](docs/content/guide/fork-differences.md).
For a compact runnable summary, see
[crates/figue/examples/teamy_fork_behaviour.rs](crates/figue/examples/teamy_fork_behaviour.rs).
It demonstrates aliases, `to_args` roundtrips, `help list`, implementation
source hints, optional-value named args, Teamy-style version metadata,
transparent newtypes, and custom fallible scalar parsing through a Facet proxy.

Versioning policy:

- `teamy-figue` uses its own semver.
- The fork stays one major version ahead of the upstream `figue` major.
- The exact upstream base is recorded in `Cargo.toml` under `workspace.metadata.teamy.upstream_figue`.

# figue

[![crates.io](https://img.shields.io/crates/v/teamy-figue.svg)](https://crates.io/crates/teamy-figue)
[![documentation](https://docs.rs/teamy-figue/badge.svg)](https://docs.rs/teamy-figue)
[![MIT/Apache-2.0 licensed](https://img.shields.io/crates/l/teamy-figue.svg)](LICENSE-MIT)

figue (pronounced 'fig', like the fruit) provides configuration parsing from
CLI arguments, environment variables, and config files, a bit like
[figment](https://docs.rs/figment/latest/figment/) but based on facet
reflection:

```rust
use facet_pretty::FacetPretty;
use facet::Facet;
use figue as args;

#[derive(Facet)]
struct Args {
    #[facet(args::positional)]
    path: String,

    #[facet(args::named, args::short = 'v')]
    verbose: bool,

    #[facet(args::named, args::short = 'j')]
    concurrency: usize,
}

# fn main() -> Result<(), Box<dyn std::error::Error>> {
let args: Args = figue::from_slice(&["--verbose", "-j", "14", "example.rs"])?;
eprintln!("args: {}", args.pretty());
Ok(())
# }
```

The entry point of figue is [`builder`] — let yourself be guided from there.

## Documentation

- **Upstream guide & reference**: <https://figue.bearcove.eu> — task-oriented
  guide, copy-paste recipes, and an exhaustive reference (attributes, grammar,
  merge rules, error catalog).
- **Fork API docs**: <https://docs.rs/teamy-figue>
- **Fork differences**: [docs/content/guide/fork-differences.md](docs/content/guide/fork-differences.md) — intentional Teamy fork behavior that differs from upstream
- **Runnable fork behavior summary**: [crates/figue/examples/teamy_fork_behaviour.rs](crates/figue/examples/teamy_fork_behaviour.rs) — a single annotated example covering the main Teamy fork extensions

The site sources live in `docs/` and are built with
[dodeca](https://github.com/bearcove/dodeca) (`ddc serve` locally, deployed to
GitHub Pages on push to `main`).

## Color

figue uses [facet-color](https://docs.rs/facet-color) for coloring output.

## Contributing

Run `hooks/install.sh` to install pre-commit and pre-push hooks.

## Sponsors

Thanks to all individual sponsors:

<p> <a href="https://github.com/sponsors/fasterthanlime">
<picture>
<source media="(prefers-color-scheme: dark)" srcset="https://github.com/bearcove/figue/raw/main/static/sponsors-v3/github-dark.svg">
<img src="https://github.com/bearcove/figue/raw/main/static/sponsors-v3/github-light.svg" height="40" alt="GitHub Sponsors">
</picture>
</a> <a href="https://patreon.com/fasterthanlime">
    <picture>
    <source media="(prefers-color-scheme: dark)" srcset="https://github.com/bearcove/figue/raw/main/static/sponsors-v3/patreon-dark.svg">
    <img src="https://github.com/bearcove/figue/raw/main/static/sponsors-v3/patreon-light.svg" height="40" alt="Patreon">
    </picture>
</a> </p>

...along with corporate sponsors:

<p> <a href="https://aws.amazon.com">
<picture>
<source media="(prefers-color-scheme: dark)" srcset="https://github.com/bearcove/figue/raw/main/static/sponsors-v3/aws-dark.svg">
<img src="https://github.com/bearcove/figue/raw/main/static/sponsors-v3/aws-light.svg" height="40" alt="AWS">
</picture>
</a> <a href="https://zed.dev">
<picture>
<source media="(prefers-color-scheme: dark)" srcset="https://github.com/bearcove/figue/raw/main/static/sponsors-v3/zed-dark.svg">
<img src="https://github.com/bearcove/figue/raw/main/static/sponsors-v3/zed-light.svg" height="40" alt="Zed">
</picture>
</a> <a href="https://depot.dev?utm_source=facet">
<picture>
<source media="(prefers-color-scheme: dark)" srcset="https://github.com/bearcove/figue/raw/main/static/sponsors-v3/depot-dark.svg">
<img src="https://github.com/bearcove/figue/raw/main/static/sponsors-v3/depot-light.svg" height="40" alt="Depot">
</picture>
</a> </p>

...without whom this work could not exist.

## Special thanks

The facet logo was drawn by [Misiasart](https://misiasart.com/).

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](https://github.com/bearcove/figue/blob/main/LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](https://github.com/bearcove/figue/blob/main/LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.
