# teamy-figue-attrs

This is **TeamDman's fork of the upstream Figue attribute grammar**, originally
created by Amos Wenger. `teamy-figue-attrs` is a separate package from upstream
`figue-attrs`. It uses Teamy Facet 0.50.0-rc.7 and matches `teamy-figue`
6.0.0-rc.1; applications should use `teamy-figue`, which re-exports the grammar.
See the [fork overview](https://github.com/TeamDman/figue/blob/teamy-main/README.md).

[![crates.io](https://img.shields.io/crates/v/teamy-figue-attrs.svg)](https://crates.io/crates/teamy-figue-attrs)
[![documentation](https://docs.rs/teamy-figue-attrs/badge.svg)](https://docs.rs/teamy-figue-attrs)
[![MIT/Apache-2.0 licensed](https://img.shields.io/crates/l/teamy-figue-attrs.svg)](LICENSE-MIT)

Attribute macros for [teamy-figue](https://crates.io/crates/teamy-figue) CLI argument parsing.

**Note:** This is an internal crate. Users should depend on `teamy-figue` directly, which
re-exports everything from this crate.

## Why a separate crate?

This crate exists to work around Rust's restriction on accessing macro-expanded
`#[macro_export]` macros by absolute paths within the same crate
([rust-lang/rust#52234](https://github.com/rust-lang/rust/issues/52234)).

By defining the attribute grammar macros in a separate crate, the main `teamy-figue` crate
can reference them via an external crate path.

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](https://github.com/bearcove/figue/blob/main/LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](https://github.com/bearcove/figue/blob/main/LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.
