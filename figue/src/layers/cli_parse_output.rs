use crate::driver::LayerOutput;

/// CLI layer values and metadata needed before the driver's special exits.
pub(crate) struct CliParseOutput {
    pub layer: LayerOutput,
    /// A token matched neither a subcommand nor a positional at a command level.
    /// Help must not describe a partial path recovered after that token.
    pub invalid_subcommand_path: bool,
}
