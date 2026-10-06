//! Inherited help must describe the same per-spelling options that parsing accepts.

use facet::Facet;
use figue::{self as args, Driver, DriverError, FigueBuiltins, HelpConfig};

fn help_for<T: Facet<'static>>(arguments: &[&str]) -> String {
    let config = figue::builder::<T>()
        .expect("valid schema")
        .cli(|cli| cli.args(arguments.iter().copied()))
        .help(|help| help.program_name("cloud-terrastodon").width(120))
        .build();
    match Driver::new(config).run().into_result() {
        Err(DriverError::Help { text, .. }) => strip_ansi_escapes::strip_str(&text),
        Err(error) => panic!("expected help, got {error:?}"),
        Ok(_) => panic!("expected help, got a parsed value"),
    }
}

fn option_row<'a>(help: &'a str, description: &str) -> &'a str {
    let mut previous = None;
    for line in help.lines() {
        if line.trim() == description {
            return previous.expect("option description follows its names");
        }
        previous = Some(line);
    }
    panic!("missing option description {description:?}:\n{help}");
}

#[derive(Facet, Debug)]
struct GlobalArgs {
    /// Choose an authentication source.
    #[facet(args::named, args::label = "SOURCE", default = "auto")]
    auth_source: String,
    /// Enable diagnostic logging.
    #[facet(args::named)]
    debug: bool,
    /// Set the console log filter.
    #[facet(args::named, args::alias = "log-level", default = "info")]
    log_filter: String,
    /// Set the file log filter.
    #[facet(args::named)]
    log_file_filter: Option<String>,
    /// Choose the log destination.
    #[facet(args::named, args::label = "FILE|DIR")]
    log_file: Option<String>,
    #[facet(flatten)]
    builtins: FigueBuiltins,
}

/// A Cloud Terrastodon-shaped command tree with flattened global options.
#[derive(Facet, Debug)]
struct CloudCli {
    /// Root workspace positional.
    #[facet(args::positional, default = ".")]
    workspace: String,
    #[facet(flatten)]
    global: GlobalArgs,
    #[facet(args::subcommand)]
    command: CloudCommand,
}

#[derive(Facet, Debug)]
#[repr(u8)]
enum CloudCommand {
    #[facet(args::alias = "az")]
    Azure {
        /// Suppress Azure progress output.
        #[facet(args::named, args::short = 'q')]
        quiet: bool,
        /// Intermediate subscription positional.
        #[facet(args::positional, default)]
        subscription: Option<String>,
        #[facet(args::subcommand)]
        command: AzureCommand,
    },
    /// Root sibling command.
    Echo,
}

#[derive(Facet, Debug)]
#[repr(u8)]
enum AzureCommand {
    Tenant {
        #[facet(args::subcommand)]
        command: TenantCommand,
    },
    /// Intermediate sibling command.
    Accounts,
}

#[derive(Facet, Debug)]
#[repr(u8)]
enum TenantCommand {
    Login {
        /// Tracked tenant identifier or alias.
        #[facet(args::positional)]
        tenant: String,
    },
    /// Leaf sibling command.
    Logout,
}

fn assert_globals_once(help: &str) {
    for spelling in [
        "--auth-source ",
        "--[no-]debug",
        "--log-filter ",
        "--log-file-filter ",
        "--log-file ",
        "--[no-]help",
        "--[no-]html-help",
        "--[no-]version",
        "--completions ",
        "--export-jsonschemas ",
    ] {
        assert_eq!(
            help.matches(spelling).count(),
            1,
            "expected {spelling} once:\n{help}"
        );
    }
    assert!(help.contains("aliases: log-level"), "{help}");
    assert!(
        help.contains("--auth-source <SOURCE> [Default: `auto`]"),
        "{help}"
    );
    assert!(help.contains("--log-file <FILE|DIR>"), "{help}");
}

#[test]
fn leaf_help_shows_all_flattened_globals_and_only_local_positionals() {
    let help = help_for::<CloudCli>(&["az", "tenant", "login", "--help"]);
    assert_globals_once(&help);
    assert_eq!(help.matches("--[no-]quiet").count(), 1, "{help}");
    assert!(
        help.contains("cloud-terrastodon azure tenant login [OPTIONS] <TENANT>"),
        "{help}"
    );
    assert!(
        help.contains("Tracked tenant identifier or alias."),
        "{help}"
    );
    for absent in [
        "WORKSPACE",
        "SUBSCRIPTION",
        "Root workspace positional.",
        "Intermediate subscription positional.",
        "sibling command.",
        "COMMANDS:",
    ] {
        assert!(!help.contains(absent), "unexpected {absent}:\n{help}");
    }
    assert_eq!(
        help,
        help_for::<CloudCli>(&["azure", "tenant", "login", "-h"])
    );
    assert_eq!(
        help,
        help_for::<CloudCli>(&["az", "tenant", "login", "help"])
    );
}

#[test]
fn intermediate_help_inherits_root_options_without_root_positionals_or_siblings() {
    let help = help_for::<CloudCli>(&["az", "tenant", "--help"]);
    assert_globals_once(&help);
    assert_eq!(help.matches("--[no-]quiet").count(), 1, "{help}");
    assert!(help.contains("login"), "{help}");
    assert!(help.contains("logout"), "{help}");
    assert!(!help.contains("WORKSPACE"), "{help}");
    assert!(!help.contains("SUBSCRIPTION"), "{help}");
    assert!(!help.contains("Root sibling command."), "{help}");
    assert!(!help.contains("Intermediate sibling command."), "{help}");
}

#[test]
fn documented_globals_parse_after_the_leaf_command() {
    let cli = figue::from_slice::<CloudCli>(&[
        "az",
        "tenant",
        "login",
        "example",
        "--auth-source",
        "browser",
        "--debug",
        "--log-level",
        "trace",
        "--log-file-filter",
        "debug",
        "--log-file",
        "logs",
        "-q",
    ])
    .unwrap();
    assert_eq!(cli.global.auth_source, "browser");
    assert!(cli.global.debug);
    assert_eq!(cli.global.log_filter, "trace");
    assert_eq!(cli.global.log_file_filter.as_deref(), Some("debug"));
    assert_eq!(cli.global.log_file.as_deref(), Some("logs"));
    let CloudCommand::Azure {
        quiet,
        subscription,
        command:
            AzureCommand::Tenant {
                command: TenantCommand::Login { tenant },
            },
        ..
    } = cli.command
    else {
        panic!("expected tenant login");
    };
    assert!(quiet);
    assert_eq!(subscription, None);
    assert_eq!(tenant, "example");
}

#[test]
fn root_help_still_matches_direct_generation() {
    let config = HelpConfig {
        program_name: Some("cloud-terrastodon".to_owned()),
        width: 120,
        ..Default::default()
    };
    let direct = strip_ansi_escapes::strip_str(figue::generate_help::<CloudCli>(&config));
    assert_eq!(help_for::<CloudCli>(&["--help"]), direct);
    assert_globals_once(&direct);
}

#[test]
fn full_help_list_includes_globals_for_each_leaf() {
    let help = help_for::<CloudCli>(&["az", "tenant", "help", "list"]);
    let sections = help.split("\n\ncloud-terrastodon ").collect::<Vec<_>>();
    assert_eq!(sections.len(), 2, "{help}");
    for section in sections {
        assert_globals_once(section);
        assert_eq!(section.matches("--[no-]quiet").count(), 1, "{section}");
        assert!(!section.contains("WORKSPACE"), "{section}");
        assert!(!section.contains("SUBSCRIPTION"), "{section}");
    }
    let short = help_for::<CloudCli>(&["az", "tenant", "help", "list", "--short"]);
    assert_eq!(
        short,
        "cloud-terrastodon azure tenant login\ncloud-terrastodon azure tenant logout"
    );
}

#[derive(Facet, Debug)]
struct ShadowCli {
    /// Parent verbose option.
    #[facet(args::named, args::short = 'v')]
    verbose: bool,
    /// Parent log filter option.
    #[facet(args::named, args::alias = "log-level")]
    log_filter: Option<String>,
    /// Parent output option.
    #[facet(args::named, args::alias = "shared")]
    output: Option<String>,
    /// Parent retries option.
    #[facet(args::named, args::short = 'r', args::alias = "attempts")]
    retries: Option<u8>,
    /// Parent color option.
    #[facet(args::named, default = true)]
    color: bool,
    /// Parent enabled option.
    #[facet(args::named, default = true)]
    enabled: bool,
    /// Parent explicit negative option.
    #[facet(args::named)]
    no_feature: Option<String>,
    /// Parent short-only option.
    #[facet(args::named, args::short = 'p', args::label = "VALUE")]
    short_only: Option<String>,
    /// Parent counted option.
    #[facet(args::named, args::short = 'c', args::counted)]
    count: u8,
    #[facet(args::subcommand)]
    command: ShadowCommand,
    #[facet(flatten)]
    builtins: FigueBuiltins,
}

#[derive(Facet, Debug)]
#[repr(u8)]
enum ShadowCommand {
    Leaf {
        /// Local verbose option.
        #[facet(args::named, args::short = 'v')]
        local_verbose: bool,
        /// Local log filter option.
        #[facet(args::named)]
        log_filter: Option<String>,
        /// Local output option.
        #[facet(args::named, args::alias = "shared")]
        local_output: Option<String>,
        /// Local retries option.
        #[facet(args::named, args::short = 'r', args::alias = "attempts")]
        retries: Option<u8>,
        /// Local color value.
        #[facet(args::named)]
        color: Option<String>,
        /// Local explicit negative option.
        #[facet(args::named)]
        no_enabled: Option<String>,
        /// Local feature option.
        #[facet(args::named)]
        feature: bool,
        /// Local short-only value.
        #[facet(args::named)]
        short_only: Option<String>,
    },
}

#[test]
fn short_collision_keeps_the_parent_long_option() {
    let help = help_for::<ShadowCli>(&["leaf", "--help"]);
    let parent = option_row(&help, "Parent verbose option.");
    assert!(parent.contains("--[no-]verbose"), "{parent}");
    assert!(!parent.contains("-v,"), "{parent}");
    assert!(
        option_row(&help, "Local verbose option.").contains("-v,"),
        "{help}"
    );
    let cli = figue::from_slice::<ShadowCli>(&["leaf", "--verbose", "-v"]).unwrap();
    assert!(cli.verbose);
    let ShadowCommand::Leaf { local_verbose, .. } = cli.command;
    assert!(local_verbose);
}

#[test]
fn canonical_and_alias_collisions_preserve_other_spellings() {
    let help = help_for::<ShadowCli>(&["leaf", "--help"]);
    let parent = option_row(&help, "Parent log filter option.");
    assert!(parent.contains("--log-level "), "{parent}");
    assert!(!parent.contains("--log-filter "), "{parent}");
    assert!(
        option_row(&help, "Local log filter option.").contains("--log-filter "),
        "{help}"
    );
    assert!(
        option_row(&help, "Parent output option.").contains("--output "),
        "{help}"
    );
    assert_eq!(help.matches("aliases: shared").count(), 1, "{help}");
    let cli = figue::from_slice::<ShadowCli>(&[
        "leaf",
        "--log-level",
        "parent",
        "--log-filter",
        "local",
        "--output",
        "parent-out",
        "--shared",
        "local-out",
    ])
    .unwrap();
    assert_eq!(cli.log_filter.as_deref(), Some("parent"));
    assert_eq!(cli.output.as_deref(), Some("parent-out"));
    let ShadowCommand::Leaf {
        log_filter,
        local_output,
        ..
    } = cli.command;
    assert_eq!(log_filter.as_deref(), Some("local"));
    assert_eq!(local_output.as_deref(), Some("local-out"));
}

#[test]
fn fully_shadowed_parent_is_omitted() {
    let help = help_for::<ShadowCli>(&["leaf", "--help"]);
    assert!(!help.contains("Parent retries option."), "{help}");
    assert_eq!(help.matches("--retries ").count(), 1, "{help}");
    assert_eq!(help.matches("aliases: attempts").count(), 1, "{help}");
    for spelling in ["--retries", "--attempts", "-r"] {
        let cli = figue::from_slice::<ShadowCli>(&["leaf", spelling, "3"]).unwrap();
        assert_eq!(cli.retries, None);
        let ShadowCommand::Leaf { retries, .. } = cli.command;
        assert_eq!(retries, Some(3));
    }
}

#[test]
fn short_only_parent_keeps_its_value_placeholder() {
    let help = help_for::<ShadowCli>(&["leaf", "--help"]);
    let parent = option_row(&help, "Parent short-only option.");
    assert!(parent.contains("-p"), "{parent}");
    assert!(parent.contains("<VALUE>"), "{parent}");
    assert!(!parent.contains("--short-only"), "{parent}");
    let cli =
        figue::from_slice::<ShadowCli>(&["leaf", "-p", "7", "--short-only", "local"]).unwrap();
    assert_eq!(cli.short_only.as_deref(), Some("7"));
    let ShadowCommand::Leaf { short_only, .. } = cli.command;
    assert_eq!(short_only.as_deref(), Some("local"));
}

#[test]
fn inherited_counted_option_retains_repeatability() {
    let help = help_for::<ShadowCli>(&["leaf", "--help"]);
    assert!(
        option_row(&help, "Parent counted option.").contains("-c, --count"),
        "{help}"
    );
    assert_eq!(help.matches("[can be repeated]").count(), 1, "{help}");
    let cli = figue::from_slice::<ShadowCli>(&["leaf", "-cc", "--count"]).unwrap();
    assert_eq!(cli.count, 3);
}

#[test]
fn boolean_negation_can_remain_when_positive_name_is_shadowed() {
    let help = help_for::<ShadowCli>(&["leaf", "--help"]);
    let parent = option_row(&help, "Parent color option.");
    assert!(parent.contains("--no-color"), "{parent}");
    assert!(!parent.contains("--[no-]color"), "{parent}");
    assert!(
        option_row(&help, "Local color value.").contains("--color "),
        "{help}"
    );
    let cli = figue::from_slice::<ShadowCli>(&["leaf", "--color", "blue", "--no-color"]).unwrap();
    assert!(!cli.color);
    let ShadowCommand::Leaf { color, .. } = cli.command;
    assert_eq!(color.as_deref(), Some("blue"));
}

#[test]
fn explicit_negative_names_win_at_any_depth() {
    let help = help_for::<ShadowCli>(&["leaf", "--help"]);
    let parent = option_row(&help, "Parent enabled option.");
    assert!(parent.contains("--enabled"), "{parent}");
    assert!(!parent.contains("--[no-]enabled"), "{parent}");
    let local = option_row(&help, "Local feature option.");
    assert!(local.contains("--feature"), "{local}");
    assert!(!local.contains("--[no-]feature"), "{local}");
    let cli = figue::from_slice::<ShadowCli>(&[
        "leaf",
        "--no-enabled",
        "local",
        "--feature",
        "--no-feature",
        "parent",
    ])
    .unwrap();
    assert!(cli.enabled);
    assert_eq!(cli.no_feature.as_deref(), Some("parent"));
    let ShadowCommand::Leaf {
        no_enabled,
        feature,
        ..
    } = cli.command;
    assert_eq!(no_enabled.as_deref(), Some("local"));
    assert!(feature);
}

#[test]
fn root_help_does_not_advertise_a_shadowed_generated_negation() {
    #[derive(Facet)]
    struct Root {
        /// Root enabled option.
        #[facet(args::named)]
        enabled: bool,
        /// Root explicit negative option.
        #[facet(args::named)]
        no_enabled: Option<String>,
        #[facet(flatten)]
        builtins: FigueBuiltins,
    }

    let help = help_for::<Root>(&["--help"]);
    let enabled = option_row(&help, "Root enabled option.");
    assert!(enabled.contains("--enabled"), "{enabled}");
    assert!(!enabled.contains("--[no-]enabled"), "{enabled}");
    assert!(
        option_row(&help, "Root explicit negative option.").contains("--no-enabled "),
        "{help}"
    );
    let parsed = figue::from_slice::<Root>(&["--enabled", "--no-enabled", "value"])
        .into_result()
        .unwrap_or_else(|error| panic!("{error}"))
        .value;
    assert!(parsed.enabled);
    assert_eq!(parsed.no_enabled.as_deref(), Some("value"));
}
