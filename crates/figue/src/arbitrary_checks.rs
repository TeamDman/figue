#![cfg(feature = "arbitrary")]

use std::collections::BTreeMap;
use std::ffi::OsString;

use arbitrary::Arbitrary;
use facet_core::Facet;
use heck::ToKebabCase;
use rand::TryRngCore;
use rand::rngs::OsRng;

use crate::schema::{ArgKind, ArgLevelSchema, Schema};
use crate::{Driver, ToArgs, builder};

/// Error returned by arbitrary-based CLI roundtrip checks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArbitraryCheckError {
    /// Number of successful arbitrary samples processed.
    pub successful_samples: usize,
    /// Number of total attempts made.
    pub attempts: usize,
    /// Human-readable failure description.
    pub message: String,
}

impl core::fmt::Display for ArbitraryCheckError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "{message} (successful_samples={successful}, attempts={attempts})",
            message = self.message,
            successful = self.successful_samples,
            attempts = self.attempts
        )
    }
}

impl std::error::Error for ArbitraryCheckError {}

/// Configuration for `to_args()` consistency checks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TestToArgsConsistencyConfig {
    /// Number of successful arbitrary samples required.
    pub success_count: usize,
    /// Maximum number of attempts before failing the test.
    pub max_attempts: usize,
    /// Size of the random byte buffer used to seed arbitrary generation.
    pub random_data_len: usize,
}

impl Default for TestToArgsConsistencyConfig {
    fn default() -> Self {
        Self {
            success_count: 500,
            max_attempts: 10_000,
            random_data_len: 1024,
        }
    }
}

/// Configuration for `to_args()` roundtrip checks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TestToArgsRoundTrip {
    /// Number of successful arbitrary samples required per command leaf.
    pub success_count_per_leaf: usize,
    /// Number of successful arbitrary samples required for CLIs without subcommands.
    pub success_count_global: usize,
    /// Maximum number of attempts allowed per command leaf.
    pub max_attempts_per_leaf: usize,
    /// Maximum number of attempts allowed for CLIs without subcommands.
    pub max_attempts_global: usize,
    /// Size of the random byte buffer used to seed arbitrary generation.
    pub random_data_len: usize,
}

impl Default for TestToArgsRoundTrip {
    fn default() -> Self {
        Self {
            success_count_per_leaf: 64,
            success_count_global: 64,
            max_attempts_per_leaf: 64 * 4_000,
            max_attempts_global: 64 * 80,
            random_data_len: 1024,
        }
    }
}

/// Assert that `to_args()` is deterministic for arbitrary-generated values.
///
/// This validates that repeated calls for the same value produce identical
/// argument vectors.
pub fn assert_to_args_consistency<T>(config: TestToArgsConsistencyConfig) -> Result<(), ArbitraryCheckError>
where
    T: Facet<'static> + for<'a> Arbitrary<'a> + core::fmt::Debug,
{
    let mut data = vec![0u8; config.random_data_len];
    let mut os_rng = OsRng;

    let mut successful_samples = 0usize;
    let mut attempts = 0usize;

    while successful_samples < config.success_count && attempts < config.max_attempts {
        attempts += 1;

        os_rng
            .try_fill_bytes(&mut data)
            .map_err(|error| ArbitraryCheckError {
                successful_samples,
                attempts,
                message: format!("failed to gather random bytes: {error}"),
            })?;

        let mut rng = arbitrary::Unstructured::new(&data);
        let Ok(instance) = T::arbitrary(&mut rng) else {
            continue;
        };

        let args1 = instance.to_args().map_err(|error| ArbitraryCheckError {
            successful_samples,
            attempts,
            message: format!("first to_args() call failed: {error}"),
        })?;

        let args2 = instance.to_args().map_err(|error| ArbitraryCheckError {
            successful_samples,
            attempts,
            message: format!("second to_args() call failed: {error}"),
        })?;

        if args1 != args2 {
            return Err(ArbitraryCheckError {
                successful_samples,
                attempts,
                message: format!(
                    "to_args() is non-deterministic for generated value: {instance:?}\nfirst={args1:?}\nsecond={args2:?}"
                ),
            });
        }

        successful_samples += 1;
    }

    if successful_samples < config.success_count {
        return Err(ArbitraryCheckError {
            successful_samples,
            attempts,
            message: "insufficient arbitrary coverage for consistency test".to_string(),
        });
    }

    Ok(())
}

/// Assert that arbitrary-generated values roundtrip via `to_args()` and figue parsing.
///
/// This validates:
/// 1. value -> `to_args()`
/// 2. args -> parse with `Driver` (strict CLI mode)
/// 3. parsed value equals original value
pub fn assert_to_args_roundtrip<T>(config: TestToArgsRoundTrip) -> Result<(), ArbitraryCheckError>
where
    T: Facet<'static> + for<'a> Arbitrary<'a> + PartialEq + core::fmt::Debug,
{
    let schema = Schema::from_shape(T::SHAPE).map_err(|error| ArbitraryCheckError {
        successful_samples: 0,
        attempts: 0,
        message: format!("failed to build schema: {error}"),
    })?;

    let command_tree = command_node_from_arg_level(schema.args());
    let command_paths = collect_command_paths(&command_tree);

    if command_paths.is_empty() {
        return assert_to_args_roundtrip_global::<T>(config);
    }

    let mut data = vec![0u8; config.random_data_len];
    let mut os_rng = OsRng;

    let mut total_successful_samples = 0usize;
    let mut total_attempts = 0usize;

    for path in command_paths {
        let mut matched_samples = 0usize;
        let mut attempts_for_path = 0usize;

        while matched_samples < config.success_count_per_leaf
            && attempts_for_path < config.max_attempts_per_leaf
        {
            attempts_for_path += 1;
            total_attempts += 1;

            os_rng
                .try_fill_bytes(&mut data)
                .map_err(|error| ArbitraryCheckError {
                    successful_samples: total_successful_samples,
                    attempts: total_attempts,
                    message: format!("failed to gather random bytes: {error}"),
                })?;

            let mut rng = arbitrary::Unstructured::new(&data);
            let Ok(instance) = T::arbitrary(&mut rng) else {
                continue;
            };

            let args = instance.to_args().map_err(|error| ArbitraryCheckError {
                successful_samples: total_successful_samples,
                attempts: total_attempts,
                message: format!("to_args() failed: {error}"),
            })?;

            let extracted_path = extract_subcommand_path_from_args(&args, &command_tree);
            if extracted_path != path {
                continue;
            }

            let parsed = parse_from_os_args::<T>(&args).map_err(|message| ArbitraryCheckError {
                successful_samples: total_successful_samples,
                attempts: total_attempts,
                message: format!(
                    "failed to parse generated args for path {path:?}\nargs={args:?}\nvalue={instance:?}\nerror={message}"
                ),
            })?;

            if instance != parsed {
                return Err(ArbitraryCheckError {
                    successful_samples: total_successful_samples,
                    attempts: total_attempts,
                    message: format!(
                        "roundtrip mismatch for path {path:?}\noriginal={instance:?}\nparsed={parsed:?}\nargs={args:?}"
                    ),
                });
            }

            matched_samples += 1;
            total_successful_samples += 1;
        }

        if matched_samples < config.success_count_per_leaf {
            return Err(ArbitraryCheckError {
                successful_samples: total_successful_samples,
                attempts: total_attempts,
                message: format!(
                    "insufficient coverage for command path {path:?}: matched {matched_samples} samples after {attempts_for_path} attempts"
                ),
            });
        }
    }

    Ok(())
}

fn assert_to_args_roundtrip_global<T>(config: TestToArgsRoundTrip) -> Result<(), ArbitraryCheckError>
where
    T: Facet<'static> + for<'a> Arbitrary<'a> + PartialEq + core::fmt::Debug,
{
    let mut data = vec![0u8; config.random_data_len];
    let mut os_rng = OsRng;

    let mut successful_samples = 0usize;
    let mut attempts = 0usize;

    while successful_samples < config.success_count_global && attempts < config.max_attempts_global
    {
        attempts += 1;

        os_rng
            .try_fill_bytes(&mut data)
            .map_err(|error| ArbitraryCheckError {
                successful_samples,
                attempts,
                message: format!("failed to gather random bytes: {error}"),
            })?;

        let mut rng = arbitrary::Unstructured::new(&data);
        let Ok(instance) = T::arbitrary(&mut rng) else {
            continue;
        };

        let args = instance.to_args().map_err(|error| ArbitraryCheckError {
            successful_samples,
            attempts,
            message: format!("to_args() failed: {error}"),
        })?;

        let parsed = parse_from_os_args::<T>(&args).map_err(|message| ArbitraryCheckError {
            successful_samples,
            attempts,
            message: format!(
                "failed to parse generated args\nargs={args:?}\nvalue={instance:?}\nerror={message}"
            ),
        })?;

        if instance != parsed {
            return Err(ArbitraryCheckError {
                successful_samples,
                attempts,
                message: format!(
                    "roundtrip mismatch\noriginal={instance:?}\nparsed={parsed:?}\nargs={args:?}"
                ),
            });
        }

        successful_samples += 1;
    }

    if successful_samples < config.success_count_global {
        return Err(ArbitraryCheckError {
            successful_samples,
            attempts,
            message: "insufficient arbitrary coverage for roundtrip test".to_string(),
        });
    }

    Ok(())
}

#[derive(Clone, Debug)]
struct CommandBranch {
    cli_name: String,
    effective_name: String,
    node: CommandNode,
}

#[derive(Clone, Debug, Default)]
struct CommandNode {
    positional_count: usize,
    named_flag_consumes_value: BTreeMap<String, bool>,
    subcommands: Vec<CommandBranch>,
}

fn command_node_from_arg_level(level: &ArgLevelSchema) -> CommandNode {
    let mut node = CommandNode::default();

    for (name, schema) in level.args() {
        match schema.kind() {
            ArgKind::Positional => {
                node.positional_count += 1;
            }
            ArgKind::Named { counted, .. } => {
                let consumes_value = !counted && !schema.value().inner_if_option().is_bool();
                node.named_flag_consumes_value
                    .insert(name.to_kebab_case(), consumes_value);
            }
        }
    }

    for subcommand in level.subcommands().values() {
        node.subcommands.push(CommandBranch {
            cli_name: subcommand.cli_name().to_string(),
            effective_name: subcommand.effective_name().to_string(),
            node: command_node_from_arg_level(subcommand.args()),
        });
    }

    node
}

fn collect_command_paths(root: &CommandNode) -> Vec<Vec<String>> {
    fn visit(node: &CommandNode, current: &mut Vec<String>, output: &mut Vec<Vec<String>>) {
        if node.subcommands.is_empty() {
            if !current.is_empty() {
                output.push(current.clone());
            }
            return;
        }

        for branch in &node.subcommands {
            current.push(branch.effective_name.clone());
            visit(&branch.node, current, output);
            let _ = current.pop();
        }
    }

    let mut output = Vec::new();
    let mut current = Vec::new();
    visit(root, &mut current, &mut output);
    output
}

fn extract_subcommand_path_from_args(args: &[OsString], root: &CommandNode) -> Vec<String> {
    let tokens = args
        .iter()
        .map(|arg| arg.to_string_lossy().to_string())
        .collect::<Vec<_>>();

    fn walk(node: &CommandNode, tokens: &[String], index: &mut usize, output: &mut Vec<String>) {
        let mut positionals_seen = 0usize;

        while *index < tokens.len() {
            let token = &tokens[*index];

            if token.starts_with("--") {
                let flag_name = token.trim_start_matches("--");
                if let Some(consumes_value) = node.named_flag_consumes_value.get(flag_name) {
                    if *consumes_value {
                        *index = (*index + 2).min(tokens.len());
                    } else {
                        *index += 1;
                    }
                } else {
                    *index += 1;
                    if *index < tokens.len() && !tokens[*index].starts_with('-') {
                        *index += 1;
                    }
                }
                continue;
            }

            if token.starts_with('-') {
                *index += 1;
                continue;
            }

            if positionals_seen < node.positional_count {
                positionals_seen += 1;
                *index += 1;
                continue;
            }

            if let Some(branch) = node
                .subcommands
                .iter()
                .find(|branch| branch.cli_name == normalize_command_token(token))
            {
                output.push(branch.effective_name.clone());
                *index += 1;
                walk(&branch.node, tokens, index, output);
            }

            return;
        }
    }

    let mut index = 0usize;
    let mut output = Vec::new();
    walk(root, &tokens, &mut index, &mut output);
    output
}

fn normalize_command_token(token: &str) -> String {
    token.replace('_', "-").to_ascii_lowercase()
}

fn parse_from_os_args<T>(args: &[OsString]) -> Result<T, String>
where
    T: Facet<'static>,
{
    let config = builder::<T>()
        .map_err(|error| format!("builder failed: {error}"))?
        .cli(|cli| cli.args_os(args).strict())
        .build();

    Driver::new(config)
        .run()
        .into_result()
        .map(|output| output.get_silent())
        .map_err(|error| format!("driver failed: {error:?}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate as args;
    use std::time::Duration;
    use std::time::Instant;
    use facet::Facet;

    #[derive(Facet, arbitrary::Arbitrary, Debug, PartialEq)]
    #[repr(u8)]
    enum Command {
        Build {
            #[facet(args::named)]
            release: bool,
        },
        Clean,
    }

    #[derive(Facet, arbitrary::Arbitrary, Debug, PartialEq)]
    #[repr(u8)]
    enum NestedAction {
        Set {
            #[facet(args::positional)]
            file: String,
        },
        Get,
    }

    #[derive(Facet, arbitrary::Arbitrary, Debug, PartialEq)]
    #[repr(u8)]
    enum NestedCommand {
        Output {
            #[facet(args::subcommand)]
            path: NestedAction,
        },
    }

    #[derive(Facet, arbitrary::Arbitrary, Debug, PartialEq)]
    struct NestedCli {
        #[facet(args::subcommand)]
        command: NestedCommand,
    }

    #[derive(Facet, arbitrary::Arbitrary, Debug, PartialEq)]
    struct Cli {
        #[facet(args::named)]
        verbose: bool,

        #[facet(args::subcommand)]
        command: Command,
    }

    #[test]
    fn arbitrary_consistency_smoke_test() {
        assert_to_args_consistency::<Cli>(TestToArgsConsistencyConfig::default())
            .expect("consistency check should pass");
    }

    #[test]
    fn arbitrary_roundtrip_smoke_test() {
        assert_to_args_roundtrip::<Cli>(TestToArgsRoundTrip::default())
            .expect("roundtrip check should pass");
    }

    #[test]
    fn configurable_roundtrip_stress_test_completes_quickly() {
        let start = Instant::now();
        assert_to_args_roundtrip::<Cli>(TestToArgsRoundTrip::default())
            .expect("roundtrip check should pass");
        assert!(
            start.elapsed() < Duration::from_secs(3),
            "roundtrip stress test took {:?}",
            start.elapsed()
        );
    }

    #[test]
    fn collects_nested_command_paths() {
        let schema = Schema::from_shape(NestedCli::SHAPE).expect("schema should be valid");
        let tree = command_node_from_arg_level(schema.args());
        let paths = collect_command_paths(&tree);

        assert!(
            paths.contains(&vec!["Output".to_string(), "Set".to_string()]),
            "expected Output -> Set path"
        );
        assert!(
            paths.contains(&vec!["Output".to_string(), "Get".to_string()]),
            "expected Output -> Get path"
        );
    }
}
