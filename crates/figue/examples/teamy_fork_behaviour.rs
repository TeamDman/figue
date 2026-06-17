use facet::Facet;
use figue::{self as args, Driver, DriverError, FigueBuiltins, ToArgs};
use std::fmt;
use std::str::FromStr;

macro_rules! with_demo_help {
    ($builder:expr) => {
        $builder.help(|help| {
            help.program_name("teamy-fork-behaviour")
                // Teamy binaries usually pass figue a version string assembled
                // by the consuming crate, not just CARGO_PKG_VERSION.
                .version(version_string())
                .include_implementation_source_file(true)
                .include_implementation_git_url("TeamDman/figue", implementation_revision())
        })
    };
}

fn version_string() -> String {
    format!(
        "{} (rev {}, built {})",
        env!("CARGO_PKG_VERSION"),
        option_env!("GIT_REVISION").unwrap_or("unknown"),
        build_time()
    )
}

fn build_time() -> String {
    option_env!("BUILD_TIMESTAMP_UNIX").map_or_else(
        || "unknown build time".to_string(),
        |timestamp| format!("unix {timestamp}"),
    )
}

fn implementation_revision() -> &'static str {
    // Downstream Teamy binaries normally provide `GIT_REVISION` from their
    // build script. This fallback keeps the standalone example runnable from
    // the figue repository without requiring that build-time environment.
    option_env!("GIT_REVISION").unwrap_or("main")
}

/// A compact Teamy-style CLI used to demonstrate fork-only behaviour.
///
/// This example intentionally models the shape used in Teamy projects:
///
/// - flattened global args and `FigueBuiltins`
/// - nested subcommands
/// - compatibility aliases for old command/flag spellings
/// - typed CLI values that can be converted back into argv with `ToArgs`
///
/// The alias attributes and the schema-driven `ToArgs` helpers are the key bits
/// that are specific to `teamy-figue` compared with the upstream `figue` base
/// recorded in this repository's workspace metadata.
#[derive(Facet, Debug)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
struct Cli {
    /// Global arguments that are accepted before or after subcommands.
    #[facet(flatten)]
    global: GlobalArgs,

    /// Standard `--help`, `--html-help`, `--version`, and completion flags.
    ///
    /// Teamy CLIs usually flatten this into the root type and then let
    /// `DriverOutcome::unwrap()` handle the early exits in production.
    #[facet(flatten)]
    #[cfg_attr(feature = "arbitrary", arbitrary(default))]
    builtins: FigueBuiltins,

    /// The command to run.
    #[facet(args::subcommand)]
    command: Command,
}

impl PartialEq for Cli {
    fn eq(&self, other: &Self) -> bool {
        // `FigueBuiltins` intentionally does not implement `PartialEq`; consumer
        // roundtrip tests usually ignore it because it is an early-exit surface.
        self.global == other.global && self.command == other.command
    }
}

/// Global arguments shared by all commands.
#[derive(Facet, Debug, PartialEq)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
struct GlobalArgs {
    /// Enable extra diagnostics.
    ///
    /// `args::long_alias` is fork-specific. Both `--debug` and the legacy
    /// `--trace` spelling parse into this field. The canonical spelling remains
    /// `--debug`, so generated help and `ToArgs` prefer that spelling.
    #[facet(args::named, default, args::long_alias = "trace")]
    debug: bool,

    /// Tracing filter directive.
    ///
    /// Aliases are repeatable, which is useful during migrations where a CLI has
    /// accumulated several names for the same concept.
    #[facet(
        args::named,
        default,
        args::long_alias = "log-level",
        args::long_alias = "tracing-filter"
    )]
    log_filter: Option<String>,

    /// Value parsed by domain logic instead of transparent inner-string mapping.
    #[facet(args::named, default)]
    #[cfg_attr(feature = "arbitrary", arbitrary(default))]
    convert: Option<ConvertedValue>,
}

/// A domain value with fallible parsing.
///
/// Unlike `BranchSelector`, this is not `#[facet(transparent)]`. `opaque`
/// hides the implementation fields from Facet, and the `String` proxy tells
/// figue to deserialize a single CLI token through `TryFrom<String>` and
/// serialize it back through `From<&ConvertedValue> for String`.
#[derive(Clone, Debug, Eq, Facet, PartialEq)]
#[facet(opaque, proxy = String)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
struct ConvertedValue(String);

impl FromStr for ConvertedValue {
    type Err = String;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input {
            // Pretend this is expensive domain logic: database lookup, checksum
            // validation, enum migration, unit conversion, etc.
            "123" => Ok(Self("ABC".to_string())),
            // Also accept the canonical display form so `ToArgs` can roundtrip
            // an already-parsed value.
            "ABC" => Ok(Self("ABC".to_string())),
            _ => Err(format!("`{input}` must be `123` or canonical `ABC`")),
        }
    }
}

impl fmt::Display for ConvertedValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl TryFrom<String> for ConvertedValue {
    type Error = String;

    fn try_from(input: String) -> Result<Self, Self::Error> {
        input.parse()
    }
}

impl From<&ConvertedValue> for String {
    fn from(value: &ConvertedValue) -> Self {
        value.0.clone()
    }
}

/// Top-level command surface.
#[derive(Facet, Debug, PartialEq)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[repr(u8)]
enum Command {
    /// Manage terminal windows and defaults.
    ///
    /// `args::alias` is also fork-specific. `terminal`, `term`, and `tty` all
    /// select this variant, while help and `ToArgs` keep `terminal` canonical.
    #[facet(args::alias = "term", args::alias = "tty")]
    Terminal(TerminalArgs),
}

/// Terminal command group.
#[derive(Facet, Debug, PartialEq)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
struct TerminalArgs {
    /// Terminal subcommand to run.
    #[facet(args::subcommand)]
    command: TerminalCommand,
}

/// Terminal subcommands.
#[derive(Facet, Debug, PartialEq)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[repr(u8)]
enum TerminalCommand {
    /// Show or change the configured default shell.
    ///
    /// Nested aliases work the same way as top-level aliases. This mirrors
    /// migration paths such as `terminal shell ...` becoming
    /// `terminal default-shell ...`.
    #[facet(args::alias = "shell")]
    DefaultShell(DefaultShellArgs),

    /// Open a terminal.
    #[facet(args::alias = "spawn")]
    Open(OpenTerminalArgs),
}

/// Default-shell subcommand group.
#[derive(Facet, Debug, PartialEq)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
struct DefaultShellArgs {
    /// Default-shell action.
    #[facet(args::subcommand)]
    action: DefaultShellAction,
}

/// Default-shell actions.
#[derive(Facet, Debug, PartialEq)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[repr(u8)]
enum DefaultShellAction {
    /// Set the shell for a branch/workspace selector.
    Set {
        /// Branch selector affected by the setting.
        ///
        /// `BranchSelector` is a transparent newtype, so the CLI accepts this as
        /// a normal string-like value while the application still receives a
        /// domain type. `--workspace` is a compatibility alias for `--branch`.
        #[facet(args::named, default, args::long_alias = "workspace")]
        branch: BranchSelector,

        /// Shell executable, such as `pwsh` or `cmd`.
        #[facet(args::positional)]
        shell: String,
    },

    /// Show the configured default shell.
    Show,
}

/// Arguments for `terminal open`.
#[derive(Facet, Debug, PartialEq)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
struct OpenTerminalArgs {
    /// Optional terminal title.
    #[facet(args::named, default, args::long_alias = "window-title")]
    title: Option<String>,

    /// Attach stdin to the new terminal.
    #[facet(args::named, default)]
    stdin: bool,
}

/// Domain-specific CLI value that still parses as one string token.
///
/// `sfm-propagate-changes` uses this pattern for branch selectors and several
/// JSON/API id wrappers. The transparent shape lets figue treat this field like
/// the inner `String` for CLI/schema purposes.
#[derive(Clone, Debug, Eq, Facet, PartialEq)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[facet(transparent)]
struct BranchSelector(String);

impl Default for BranchSelector {
    fn default() -> Self {
        Self("core".to_string())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let explicit_args = std::env::args_os().skip(1).collect::<Vec<_>>();
    if !explicit_args.is_empty() {
        let cli: Cli = Driver::new(
            with_demo_help!(
                figue::builder::<Cli>()?.cli(|cli| cli.args_os(explicit_args).strict())
            )
            .build(),
        )
        .run()
        .unwrap();

        println!("{cli:#?}");
        return Ok(());
    }

    demonstrate_alias_parsing_and_to_args()?;
    demonstrate_bool_alias_negation()?;
    demonstrate_custom_fallible_parse()?;
    demonstrate_version_metadata()?;
    demonstrate_help_extensions()?;
    demonstrate_optional_arbitrary_helpers()?;

    Ok(())
}

fn parse_cli(args: &[&str]) -> Result<Cli, Box<dyn std::error::Error>> {
    let config = with_demo_help!(
        figue::builder::<Cli>()?.cli(|cli| cli.args(args.iter().copied()).strict())
    )
    .build();

    Ok(Driver::new(config).run().into_result()?.get_silent())
}

fn demonstrate_alias_parsing_and_to_args() -> Result<(), Box<dyn std::error::Error>> {
    let legacy_spelling = parse_cli(&[
        "--trace",
        "--log-level",
        "debug,teamy=trace",
        "term",
        "shell",
        "set",
        "--workspace",
        "core>=1.21.0",
        "pwsh",
    ])?;

    let canonical_spelling = parse_cli(&[
        "--debug",
        "--log-filter",
        "debug,teamy=trace",
        "terminal",
        "default-shell",
        "set",
        "--branch",
        "core>=1.21.0",
        "pwsh",
    ])?;

    assert_eq!(legacy_spelling, canonical_spelling);

    let generated = legacy_spelling.to_args_string()?;
    assert_eq!(
        generated,
        "--debug --log-filter debug,teamy=trace terminal default-shell set --branch core>=1.21.0 pwsh"
    );

    let generated_refs = generated.split_whitespace().collect::<Vec<_>>();
    let reparsed = parse_cli(&generated_refs)?;
    assert_eq!(legacy_spelling, reparsed);

    println!("alias parse: legacy spellings and canonical spellings produce the same value");
    println!("to_args:     {generated}");
    println!();

    Ok(())
}

/// A tiny isolated parser used to show alias-aware boolean negation.
///
/// Keeping this separate avoids making `Cli` contain a default-true bool, which
/// would be a poor fit for the arbitrary roundtrip helper below: `ToArgs` emits
/// present flags, not `--no-*` overrides for default-true booleans.
#[derive(Facet, Debug)]
struct ColourCli {
    /// Use colour output.
    #[facet(args::named, default = true, args::long_alias = "colour")]
    color: bool,
}

fn demonstrate_bool_alias_negation() -> Result<(), Box<dyn std::error::Error>> {
    let parsed: ColourCli = figue::from_slice(&["--no-colour"])
        .into_result()?
        .get_silent();

    assert!(!parsed.color);

    println!("bool alias:  --no-colour negates the canonical --color flag");
    println!();

    Ok(())
}

fn demonstrate_custom_fallible_parse() -> Result<(), Box<dyn std::error::Error>> {
    let parsed = parse_cli(&["--convert", "123", "terminal", "open"])?;

    assert_eq!(
        parsed.global.convert,
        Some(ConvertedValue("ABC".to_string()))
    );

    let canonical = parsed.to_args_string()?;
    assert_eq!(canonical, "--convert ABC terminal open");

    let canonical_refs = canonical.split_whitespace().collect::<Vec<_>>();
    let reparsed = parse_cli(&canonical_refs)?;
    assert_eq!(parsed, reparsed);

    assert!(
        parse_cli(&["--convert", "nope", "terminal", "open"]).is_err(),
        "invalid converted values should be rejected by FromStr"
    );

    println!("fromstr:     --convert 123 parses through custom fallible domain logic");
    println!("             canonical ToArgs form: {canonical}");
    println!();

    Ok(())
}

fn demonstrate_version_metadata() -> Result<(), Box<dyn std::error::Error>> {
    let text = version_text_for(&["--version"])?;

    assert!(text.contains(env!("CARGO_PKG_VERSION")));
    assert!(text.contains("rev "));
    assert!(text.contains("built "));

    println!("version:     --version includes package, git revision, and build time metadata");
    println!("             {text}");
    println!();

    Ok(())
}

fn demonstrate_help_extensions() -> Result<(), Box<dyn std::error::Error>> {
    let root_help = help_text_for(&["--help"])?;
    assert!(root_help.contains("aliases: term, tty"));
    assert!(root_help.contains("Implementation:"));
    assert!(root_help.contains("teamy_fork_behaviour.rs"));
    assert!(root_help.contains("https://github.com/TeamDman/figue/blob/"));

    let terminal_help = help_text_for(&["terminal", "--help"])?;
    assert!(terminal_help.contains("aliases: shell"));
    assert!(terminal_help.contains("aliases: spawn"));

    let nested_full = help_text_for(&["terminal", "help", "list"])?;
    assert!(nested_full.contains("default-shell set"));
    assert!(nested_full.contains("--branch"));
    assert!(nested_full.contains("--workspace"));
    assert!(nested_full.contains("--title"));

    let nested_list = help_text_for(&["terminal", "help", "list", "--short"])?;
    assert!(nested_list.contains("default-shell"));
    assert!(nested_list.contains("open"));
    assert!(!nested_list.contains("--stdin"));

    println!("help:        root help shows aliases and implementation/source hints");
    println!("help list:   `terminal help list` shows full help for reachable child commands");
    println!("help short:  `terminal help list --short` lists only child command paths");
    println!();

    Ok(())
}

fn version_text_for(args: &[&str]) -> Result<String, Box<dyn std::error::Error>> {
    let config = with_demo_help!(
        figue::builder::<Cli>()?.cli(|cli| cli.args(args.iter().copied()).strict())
    )
    .build();

    match Driver::new(config).run().into_result() {
        Err(DriverError::Version { text }) => Ok(text),
        Ok(_) => Err(format!("expected version for args {args:?}, got successful parse").into()),
        Err(error) => Err(format!("expected version for args {args:?}, got {error}").into()),
    }
}

fn help_text_for(args: &[&str]) -> Result<String, Box<dyn std::error::Error>> {
    let config = with_demo_help!(
        figue::builder::<Cli>()?.cli(|cli| cli.args(args.iter().copied()).strict())
    )
    .build();

    match Driver::new(config).run().into_result() {
        Err(DriverError::Help { text, .. }) => Ok(text),
        Ok(_) => Err(format!("expected help for args {args:?}, got successful parse").into()),
        Err(error) => Err(format!("expected help for args {args:?}, got {error}").into()),
    }
}

#[cfg(feature = "arbitrary")]
fn demonstrate_optional_arbitrary_helpers() -> Result<(), Box<dyn std::error::Error>> {
    figue::assert_to_args_consistency::<Cli>(figue::TestToArgsConsistencyConfig {
        success_count: 16,
        max_attempts: 512,
        root_seed: Some(0x5EED_5EED),
        ..Default::default()
    })?;

    figue::assert_to_args_roundtrip::<Cli>(figue::TestToArgsRoundTrip {
        success_count_per_leaf: 1,
        max_attempts_per_leaf: 1024,
        root_seed: Some(0xF16E_F16E),
        ..Default::default()
    })?;

    println!("arbitrary:   generated values are deterministic and roundtrip through ToArgs");
    println!();

    Ok(())
}

#[cfg(not(feature = "arbitrary"))]
fn demonstrate_optional_arbitrary_helpers() -> Result<(), Box<dyn std::error::Error>> {
    let _ = std::env::current_dir()?;
    println!(
        "arbitrary:   enable the `arbitrary` feature to run the fork's generated roundtrip checks"
    );
    println!(
        "             cargo run -p teamy-figue --example teamy_fork_behaviour --features arbitrary"
    );
    println!();

    Ok(())
}
