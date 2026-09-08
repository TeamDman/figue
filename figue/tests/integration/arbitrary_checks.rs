use arbitrary::Arbitrary;
use facet::Facet;
use figue::{
    self as args, ArbitraryCheckError, TestToArgsConsistencyConfig, TestToArgsRoundTrip,
    assert_to_args_consistency, assert_to_args_roundtrip,
};

// Deliberately omit Debug from the entire command tree: these tests also
// verify the public helpers' trait bounds at compile time.
#[derive(Facet, Arbitrary, PartialEq)]
#[repr(u8)]
enum Command {
    Build {
        #[facet(args::named)]
        release: bool,

        #[facet(args::positional)]
        target: Option<String>,
    },
    Clean,
}

#[derive(Facet, Arbitrary, PartialEq)]
struct Cli {
    #[facet(args::named)]
    verbose: bool,

    #[facet(args::subcommand)]
    command: Command,
}

#[test]
fn exported_consistency_helper_smoke_test() {
    assert_to_args_consistency::<Cli>(TestToArgsConsistencyConfig {
        success_count: 8,
        max_attempts: 256,
        root_seed: Some(42),
        ..Default::default()
    })
    .expect("consistency helper should succeed");
}

#[test]
fn exported_roundtrip_helper_smoke_test() {
    assert_to_args_roundtrip::<Cli>(TestToArgsRoundTrip {
        success_count_per_leaf: 2,
        success_count_global: 2,
        max_attempts_per_leaf: 256,
        max_attempts_global: 256,
        root_seed: Some(42),
        ..Default::default()
    })
    .expect("roundtrip helper should succeed");
}

#[derive(Facet, Arbitrary, PartialEq)]
struct GlobalCli {
    #[facet(args::named)]
    count: u16,
}

#[test]
fn exported_global_helpers_work_without_debug() {
    assert_to_args_consistency::<GlobalCli>(TestToArgsConsistencyConfig {
        success_count: 8,
        max_attempts: 8,
        root_seed: Some(42),
        ..Default::default()
    })
    .expect("global consistency check should pass without Debug");
    assert_to_args_roundtrip::<GlobalCli>(failure_config())
        .expect("global roundtrip check should pass without Debug");
}

// Arbitrary can generate values rejected by parsing's invariant checks.
#[derive(Facet, Arbitrary, PartialEq)]
#[facet(invariants = is_even)]
struct InvalidValue {
    #[facet(args::named)]
    #[arbitrary(value = 37)]
    count: u32,
}

fn is_even(value: &InvalidValue) -> bool {
    value.count.is_multiple_of(2)
}

// NaN roundtrips through its CLI representation but is not equal to itself.
// This also verifies that the helper still uses PartialEq for comparisons.
#[derive(Facet, Arbitrary, PartialEq)]
struct NonReflexiveValue {
    #[facet(args::named)]
    #[arbitrary(value = f64::NAN)]
    amount: f64,
}

#[derive(Facet, Arbitrary, PartialEq)]
#[repr(u8)]
enum InvalidValueCommand {
    Run(InvalidValue),
}

#[derive(Facet, Arbitrary, PartialEq)]
struct InvalidValueCli {
    #[facet(args::subcommand)]
    command: InvalidValueCommand,
}

#[derive(Facet, Arbitrary, PartialEq)]
#[repr(u8)]
enum NonReflexiveValueCommand {
    Run(NonReflexiveValue),
}

#[derive(Facet, Arbitrary, PartialEq)]
struct NonReflexiveValueCli {
    #[facet(args::subcommand)]
    command: NonReflexiveValueCommand,
}

fn failure_config() -> TestToArgsRoundTrip {
    TestToArgsRoundTrip {
        success_count_per_leaf: 1,
        success_count_global: 1,
        max_attempts_per_leaf: 1,
        max_attempts_global: 1,
        random_data_len: 32,
        prefill_sample_count: 1,
        root_seed: Some(42),
    }
}

fn assert_failure_context(error: &ArbitraryCheckError, fragments: &[&str]) {
    assert_eq!(error.successful_samples, 0);
    assert_eq!(error.attempts, 1);
    assert!(
        error
            .message
            .contains("root_seed=42 random_data_len=32 prefill_sample_count=1"),
        "{error}"
    );
    assert!(!error.message.contains('\u{1b}'), "{error}");
    for fragment in fragments {
        assert!(
            error.message.contains(fragment),
            "missing {fragment:?}: {error}"
        );
    }
}

fn check_roundtrip_failure<T>(fragments: &[&str])
where
    T: Facet<'static> + for<'a> Arbitrary<'a> + PartialEq,
{
    let error = assert_to_args_roundtrip::<T>(failure_config())
        .expect_err("the fixture must fail roundtripping");
    assert_failure_context(&error, fragments);
    let repeated = assert_to_args_roundtrip::<T>(failure_config())
        .expect_err("the fixture must fail roundtripping again");
    assert_eq!(
        error, repeated,
        "the same seed must reproduce the diagnostics"
    );
}

#[test]
fn global_parse_failure_formats_value_without_debug() {
    check_roundtrip_failure::<InvalidValue>(&[
        "failed to parse generated args\n",
        "args=[\"--count\", \"37\"]",
        "value=InvalidValue {",
        "count: 37",
        "error=",
    ]);
}

#[test]
fn subcommand_parse_failure_formats_value_without_debug() {
    check_roundtrip_failure::<InvalidValueCli>(&[
        "failed to parse generated args for path [\"Run\"]",
        "args=[\"run\", \"--count\", \"37\"]",
        "value=InvalidValueCli",
        "InvalidValue {",
        "count: 37",
        "error=",
    ]);
}

#[test]
fn global_mismatch_formats_both_values_without_debug() {
    check_roundtrip_failure::<NonReflexiveValue>(&[
        "roundtrip mismatch\n",
        "original=NonReflexiveValue {\n  amount: NaN",
        "parsed=NonReflexiveValue {\n  amount: NaN",
        "args=[\"--amount\", \"NaN\"]",
    ]);
}

#[test]
fn subcommand_mismatch_formats_both_values_without_debug() {
    check_roundtrip_failure::<NonReflexiveValueCli>(&[
        "roundtrip mismatch for path [\"Run\"]",
        "original=NonReflexiveValueCli",
        "parsed=NonReflexiveValueCli",
        "NonReflexiveValue {",
        "amount: NaN",
        "args=[\"run\", \"--amount\", \"NaN\"]",
    ]);
}

thread_local! {
    static SKIP_NEXT: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

fn alternating_skip(_: &bool) -> bool {
    SKIP_NEXT.with(|skip| skip.replace(!skip.get()))
}

#[derive(Facet, Arbitrary)]
struct NonDeterministicCli {
    #[facet(args::named, skip_serializing_if = alternating_skip)]
    #[arbitrary(value = true)]
    verbose: bool,
}

#[test]
fn consistency_failure_formats_value_without_debug_or_partial_eq() {
    // An intentionally stateful serialization predicate makes repeated
    // serialization of one value produce different argument vectors.
    SKIP_NEXT.with(|skip| skip.set(false));
    let error = assert_to_args_consistency::<NonDeterministicCli>(TestToArgsConsistencyConfig {
        success_count: 1,
        max_attempts: 1,
        random_data_len: 32,
        prefill_sample_count: 1,
        root_seed: Some(42),
    })
    .expect_err("the stateful fixture must fail consistency checking");
    assert_failure_context(
        &error,
        &[
            "to_args() is non-deterministic for generated value: NonDeterministicCli {",
            "verbose: true",
            "first=[\"--verbose\"]",
            "second=[]",
        ],
    );
}
