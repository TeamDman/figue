#![cfg(feature = "arbitrary")]

use std::ffi::OsString;

use arbitrary::Arbitrary;
use facet_core::Facet;
use rand::TryRngCore;
use rand::rngs::OsRng;

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

/// Assert that `to_args()` is deterministic for arbitrary-generated values.
///
/// This validates that repeated calls for the same value produce identical
/// argument vectors.
pub fn assert_to_args_consistency<T>(samples: usize) -> Result<(), ArbitraryCheckError>
where
    T: Facet<'static> + for<'a> Arbitrary<'a> + core::fmt::Debug,
{
    let mut data = vec![0u8; 1024];
    let mut os_rng = OsRng;

    let mut successful_samples = 0usize;
    let mut attempts = 0usize;
    let max_attempts = samples.saturating_mul(20).max(samples);

    while successful_samples < samples && attempts < max_attempts {
        attempts += 1;

        os_rng.try_fill_bytes(&mut data).map_err(|error| ArbitraryCheckError {
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

    if successful_samples < samples {
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
pub fn assert_to_args_roundtrip<T>(samples: usize) -> Result<(), ArbitraryCheckError>
where
    T: Facet<'static> + for<'a> Arbitrary<'a> + PartialEq + core::fmt::Debug,
{
    let mut data = vec![0u8; 1024];
    let mut os_rng = OsRng;

    let mut successful_samples = 0usize;
    let mut attempts = 0usize;
    let max_attempts = samples.saturating_mul(40).max(samples);

    while successful_samples < samples && attempts < max_attempts {
        attempts += 1;

        os_rng.try_fill_bytes(&mut data).map_err(|error| ArbitraryCheckError {
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

    if successful_samples < samples {
        return Err(ArbitraryCheckError {
            successful_samples,
            attempts,
            message: "insufficient arbitrary coverage for roundtrip test".to_string(),
        });
    }

    Ok(())
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
    struct Cli {
        #[facet(args::named)]
        verbose: bool,

        #[facet(args::subcommand)]
        command: Command,
    }

    #[test]
    fn arbitrary_consistency_smoke_test() {
        assert_to_args_consistency::<Cli>(16).expect("consistency check should pass");
    }

    #[test]
    fn arbitrary_roundtrip_smoke_test() {
        assert_to_args_roundtrip::<Cli>(16).expect("roundtrip check should pass");
    }
}
