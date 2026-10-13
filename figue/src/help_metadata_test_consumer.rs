//! Consumer-owned CLI declarations live outside the parser envelope's source file.

use crate as args;
use facet::Facet;

/// Consumer root documentation.
///
/// Consumer root details.
#[derive(Facet)]
pub(super) struct ConsumerCli {
    #[facet(args::named, default)]
    pub(super) root_flag: bool,
    #[facet(args::subcommand)]
    pub(super) command: ConsumerCommand,
}

#[derive(Facet)]
#[repr(u8)]
#[allow(
    dead_code,
    reason = "This reflected help-only fixture retains a typed variant payload for source metadata"
)]
pub(super) enum ConsumerCommand {
    /// Consumer leaf command documentation.
    Inspect(ConsumerLeaf),
}

#[derive(Facet)]
pub(super) struct ConsumerLeaf {
    #[facet(args::named, default)]
    pub(super) leaf_flag: bool,
}
