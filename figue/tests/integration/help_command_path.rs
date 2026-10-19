//! Help must describe a valid command path, even when work arguments are invalid.

use facet::Facet;
use figue::{self as args, Driver, DriverError, DriverOutcome, FigueBuiltins};

// Narrow exception to TeamyStyle's one-type-per-file rule: these coupled
// integration-test fixtures exist only to describe the hierarchy exercised below.
#[derive(Facet, Debug)]
struct Cli {
    #[facet(args::subcommand)]
    command: Command,

    #[facet(flatten)]
    builtins: FigueBuiltins,
}

#[derive(Facet, Debug)]
#[repr(u8)]
enum Command {
    /// Manage jars.
    #[facet(args::alias = "jars")]
    Jar {
        #[facet(args::subcommand)]
        action: JarAction,
    },
}

#[derive(Facet, Debug)]
#[repr(u8)]
enum JarAction {
    /// Build a jar.
    #[facet(args::alias = "compile")]
    Build {
        /// Source input.
        #[facet(args::positional)]
        input: String,

        /// Build count.
        #[facet(args::named)]
        count: Option<u16>,

        /// Opaque label.
        #[facet(args::named)]
        label: Option<String>,
    },
}

fn parse(arguments: &[&str], strict: bool) -> DriverOutcome<Cli> {
    let config = figue::builder::<Cli>()
        .expect("valid fixture schema")
        .cli(|cli| {
            let cli = cli.args(arguments.iter().copied());
            if strict { cli.strict() } else { cli }
        })
        .help(|help| help.program_name("help-path").version("1.2.3"))
        .build();
    Driver::new(config).run()
}

fn assert_command_path_failure(arguments: &[&str], token: &str, strict: bool) {
    let error = parse(arguments, strict).unwrap_err();
    assert_ne!(
        error.exit_code(),
        0,
        "invalid command path must fail: {arguments:?}, strict={strict}: {error}"
    );
    let DriverError::Failed { report } = error else {
        panic!("expected parsing failure: {arguments:?}, strict={strict}: {error}");
    };
    assert!(
        report.diagnostics.iter().any(|diagnostic| {
            (diagnostic
                .message
                .starts_with(&format!("unexpected argument: {token}"))
                || diagnostic
                    .message
                    .starts_with(&format!("unexpected positional argument: {token}")))
                && diagnostic.span.is_some_and(|span| {
                    report
                        .cli_args_source
                        .get(span.start..span.start + span.len)
                        == Some(token)
                })
        }),
        "failure must identify the exact invalid token {token:?}: {}",
        report.render_pretty()
    );
}

#[test]
fn malformed_command_order_does_not_render_partial_help() {
    for strict in [false, true] {
        for request in ["--help", "-h", "help", "--html-help"] {
            for arguments in [
                [request, "build", "jar"],
                ["build", request, "jar"],
                ["build", "jar", request],
            ] {
                assert_command_path_failure(&arguments, "build", strict);
            }
        }
        assert_command_path_failure(&["--help", "--", "build", "jar"], "build", strict);
    }
}

#[test]
fn unknown_nested_command_does_not_render_parent_help() {
    for strict in [false, true] {
        for request in ["--help", "-h", "help", "--html-help"] {
            for arguments in [
                [request, "jar", "bogus"],
                ["jar", request, "bogus"],
                ["jar", "bogus", request],
            ] {
                assert_command_path_failure(&arguments, "bogus", strict);
            }
        }
        assert_command_path_failure(&["--help", "jar", "--", "bogus"], "bogus", strict);
    }
}

#[test]
fn help_lists_require_a_valid_command_path() {
    let invalid_cases: &[(&[&str], &str)] = &[
        (&["build", "jar", "help", "list"], "build"),
        (&["build", "jar", "help", "list", "--short"], "build"),
        (&["jar", "bogus", "help", "list"], "bogus"),
        (&["jar", "bogus", "help", "list", "--short"], "bogus"),
        (&["jar", "help", "list", "bogus"], "bogus"),
    ];
    for strict in [false, true] {
        let error = parse(&["jar", "help", "list", "--short"], strict).unwrap_err();
        assert_eq!(error.exit_code(), 0);
        let DriverError::Help { text, .. } = error else {
            panic!("expected help list: {error}");
        };
        assert_eq!(strip_ansi_escapes::strip_str(&text), "help-path jar build");

        for &(arguments, token) in invalid_cases {
            assert_command_path_failure(arguments, token, strict);
        }
    }
}

#[test]
fn valid_command_paths_and_aliases_keep_their_help_target() {
    let cases: &[(&[&str], &str, bool)] = &[
        (&[], "help-path [OPTIONS] <COMMAND>", false),
        (&["jar"], "help-path jar [OPTIONS] <COMMAND>", false),
        (&["jars"], "help-path jar [OPTIONS] <COMMAND>", false),
        (
            &["jar", "build"],
            "help-path jar build [OPTIONS] <INPUT>",
            true,
        ),
        (
            &["jars", "compile"],
            "help-path jar build [OPTIONS] <INPUT>",
            true,
        ),
    ];
    for strict in [false, true] {
        for &(path, usage, leaf) in cases {
            for request in ["--help", "-h", "help"] {
                let mut arguments = path.to_vec();
                arguments.push(request);
                let error = parse(&arguments, strict).unwrap_err();
                assert_eq!(error.exit_code(), 0, "{arguments:?}: {error}");
                let DriverError::Help { text, .. } = error else {
                    panic!("expected help: {arguments:?}: {error}");
                };
                let text = strip_ansi_escapes::strip_str(&text);
                assert!(
                    text.lines().any(|line| line.trim() == usage),
                    "{arguments:?}: {text}"
                );
                assert!(text.contains("--[no-]help"), "{arguments:?}: {text}");
                assert_eq!(text.contains("--count"), leaf, "{arguments:?}: {text}");
            }
        }
    }
}

#[test]
fn help_still_ignores_missing_and_invalid_work_arguments() {
    let cases: &[&[&str]] = &[
        &["jar", "build", "--help"],
        &["jar", "build", "--count", "invalid", "--help"],
        &["jar", "build", "--help", "--count"],
        &["jar", "build", "--help", "--label"],
        &["jar", "build", "--unknown", "--help"],
    ];
    for strict in [false, true] {
        for &arguments in cases {
            let error = parse(arguments, strict).unwrap_err();
            assert_eq!(error.exit_code(), 0, "{arguments:?}: {error}");
            assert!(error.is_help(), "{arguments:?}: {error}");
            assert!(error.help_text().unwrap().contains("help-path jar build"));
        }
    }
}

#[test]
fn version_remains_independent_of_command_path_and_work_arguments() {
    let cases: &[&[&str]] = &[
        &["--version"],
        &["-V"],
        &["jar", "--version"],
        &["jar", "build", "--version"],
        &["jar", "build", "--count", "invalid", "--version"],
        &["jar", "build", "--version", "--count"],
        &["build", "jar", "--version"],
    ];
    for strict in [false, true] {
        for &arguments in cases {
            let error = parse(arguments, strict).unwrap_err();
            assert_eq!(error.exit_code(), 0, "{arguments:?}: {error}");
            let DriverError::Version { text } = error else {
                panic!("expected version: {arguments:?}: {error}");
            };
            assert_eq!(text, "help-path 1.2.3");
        }
    }
}

#[test]
fn separator_and_opaque_values_do_not_request_help() {
    let cases: &[(&[&str], &str, Option<&str>)] = &[
        (&["jar", "build", "--", "--help"], "--help", None),
        (
            &["jar", "build", "fixture.jar", "--label=--help"],
            "fixture.jar",
            Some("--help"),
        ),
        (
            &["jar", "build", "fixture.jar", "--label", "--help"],
            "fixture.jar",
            Some("--help"),
        ),
    ];
    for strict in [false, true] {
        for &(arguments, expected_input, expected_label) in cases {
            let output = parse(arguments, strict).into_result().unwrap();
            assert!(!output.value.builtins.help, "{arguments:?}");
            let Command::Jar {
                action:
                    JarAction::Build {
                        input,
                        count,
                        label,
                    },
            } = output.value.command;
            assert_eq!(input, expected_input, "{arguments:?}");
            assert_eq!(count, None, "{arguments:?}");
            assert_eq!(label.as_deref(), expected_label, "{arguments:?}");
        }
    }
}
