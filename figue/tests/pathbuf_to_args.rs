//! Native path fields should not need a consumer-defined String proxy.

use facet::Facet;
use figue::{ToArgs, ToArgsError};
use std::ffi::OsString;
use std::fmt::Debug;
use std::path::PathBuf;

#[derive(Facet, Debug, PartialEq)]
struct OptionalPath {
    #[facet(figue::named)]
    log_file: Option<PathBuf>,
}

fn roundtrip<T: Facet<'static> + Debug + PartialEq>(value: &T) -> Vec<OsString> {
    let emitted = value.to_args().expect("typed arguments should serialize");
    let text = emitted
        .iter()
        .map(|arg| arg.to_str().expect("these test values are UTF-8"))
        .collect::<Vec<_>>();
    let parsed: T = figue::from_slice(&text)
        .into_result()
        .expect("emitted arguments should parse")
        .value;
    assert_eq!(&parsed, value);
    emitted
}

#[test]
fn optional_log_path_can_be_emitted_after_parsing() {
    let parsed: OptionalPath = figue::from_slice(&["--log-file", "logs/events.ndjson"])
        .into_result()
        .expect("PathBuf parsing should succeed")
        .value;
    assert_eq!(parsed.log_file, Some(PathBuf::from("logs/events.ndjson")));
    assert_eq!(
        roundtrip(&parsed),
        ["--log-file", "logs/events.ndjson"].map(OsString::from),
    );
}

#[test]
fn path_parsing_already_works_without_serialization() {
    let parsed: OptionalPath = figue::from_slice(&["--log-file", "logs/events.ndjson"])
        .into_result()
        .expect("PathBuf parsing should succeed")
        .value;
    assert_eq!(parsed.log_file, Some(PathBuf::from("logs/events.ndjson")));
}

#[test]
fn absent_optional_path_emits_no_arguments() {
    assert!(roundtrip(&OptionalPath { log_file: None }).is_empty());
}

#[test]
fn required_named_path_preserves_exact_utf8_tokens() {
    #[derive(Facet, Debug, PartialEq)]
    struct Named {
        #[facet(figue::named)]
        output: PathBuf,
    }

    for text in [
        "",
        "logs/events.ndjson",
        "logs with spaces/日本語-é.ndjson",
        "-log",
        "a/../b",
    ] {
        assert_eq!(
            roundtrip(&Named {
                output: text.into()
            }),
            ["--output", text].map(OsString::from),
        );
    }
}

#[test]
fn positional_path_preserves_token_boundaries() {
    #[derive(Facet, Debug, PartialEq)]
    struct Positional {
        #[facet(figue::positional)]
        path: PathBuf,
    }

    for text in ["", "directory with spaces/é.txt", "-file", "--"] {
        let expected = if text.starts_with('-') {
            vec![OsString::from("--"), OsString::from(text)]
        } else {
            vec![OsString::from(text)]
        };
        assert_eq!(roundtrip(&Positional { path: text.into() }), expected);
    }
}

#[test]
fn list_of_paths_roundtrips() {
    #[derive(Facet, Debug, PartialEq)]
    struct Paths {
        #[facet(figue::named)]
        input: Vec<PathBuf>,
    }

    let value = Paths {
        input: vec!["a.txt".into(), "dir with spaces/é.txt".into()],
    };
    assert_eq!(
        roundtrip(&value),
        ["--input", "a.txt", "--input", "dir with spaces/é.txt"].map(OsString::from),
    );
}

#[test]
fn flattened_global_and_nested_command_paths_roundtrip() {
    #[derive(Facet, Debug, PartialEq)]
    struct Cli {
        #[facet(flatten)]
        global: OptionalPath,
        #[facet(figue::subcommand)]
        command: Command,
    }
    #[derive(Facet, Debug, PartialEq)]
    #[repr(u8)]
    enum Command {
        Inspect {
            #[facet(figue::subcommand)]
            command: Action,
        },
    }
    #[derive(Facet, Debug, PartialEq)]
    #[repr(u8)]
    enum Action {
        File {
            #[facet(figue::positional)]
            path: PathBuf,
        },
    }

    roundtrip(&Cli {
        global: OptionalPath {
            log_file: Some("logs/events.ndjson".into()),
        },
        command: Command::Inspect {
            command: Action::File {
                path: "input with spaces.txt".into(),
            },
        },
    });
}

#[test]
fn string_control_keeps_its_existing_representation() {
    #[derive(Facet, Debug, PartialEq)]
    struct Text {
        #[facet(figue::named)]
        output: String,
    }
    assert_eq!(
        roundtrip(&Text {
            output: "logs with spaces/é.txt".into()
        }),
        ["--output", "logs with spaces/é.txt"].map(OsString::from),
    );
}

#[test]
fn explicit_proxy_still_controls_its_representation() {
    #[derive(Facet, Debug, PartialEq)]
    #[facet(proxy = String)]
    struct ProxyPath(PathBuf);

    impl From<&ProxyPath> for String {
        fn from(value: &ProxyPath) -> Self {
            format!("proxy:{}", value.0.to_str().expect("UTF-8 test path"))
        }
    }
    impl TryFrom<String> for ProxyPath {
        type Error = &'static str;
        fn try_from(value: String) -> Result<Self, Self::Error> {
            value
                .strip_prefix("proxy:")
                .map(|text| Self(text.into()))
                .ok_or("missing proxy prefix")
        }
    }
    #[derive(Facet, Debug, PartialEq)]
    struct Cli {
        #[facet(figue::named)]
        path: ProxyPath,
    }

    assert_eq!(
        roundtrip(&Cli {
            path: ProxyPath("file.txt".into())
        }),
        ["--path", "proxy:file.txt"].map(OsString::from),
    );
}

#[test]
fn path_default_inside_config_root_is_used_issue_105() {
    #[derive(Facet, Debug)]
    struct Settings {
        #[facet(default = "./default-path")]
        path: PathBuf,
    }
    #[derive(Facet, Debug)]
    struct Cli {
        #[facet(figue::config)]
        settings: Settings,
    }

    let config = figue::builder::<Cli>()
        .expect("valid schema")
        .cli(|cli| cli.args(Vec::<String>::new()))
        .build();
    let parsed = figue::Driver::new(config)
        .run()
        .into_result()
        .expect("path default should be used")
        .value;
    assert_eq!(parsed.settings.path, PathBuf::from("./default-path"));
}

#[test]
fn named_path_default_is_used_and_roundtrips() {
    #[derive(Facet, Debug, PartialEq)]
    struct Cli {
        #[facet(figue::named, default = "logs/default.ndjson")]
        log_file: PathBuf,
    }
    let parsed: Cli = figue::from_slice::<Cli>(&[])
        .into_result()
        .expect("named path default should be used")
        .value;
    assert_eq!(parsed.log_file, PathBuf::from("logs/default.ndjson"));
    roundtrip(&parsed);
}

#[cfg(any(unix, windows))]
fn assert_non_utf8_rejected(path: PathBuf) {
    assert!(path.to_str().is_none());
    let value = OptionalPath {
        log_file: Some(path),
    };
    for result in [
        value.to_args().map(|_| ()),
        value.to_args_string().map(|_| ()),
    ] {
        let ToArgsError::Serialize(message) = result.expect_err("non-UTF-8 must return an error")
        else {
            panic!("expected a serialization error");
        };
        assert!(message.contains("PathBuf"), "{message}");
        assert!(message.contains("UTF-8"), "{message}");
    }
}

#[cfg(windows)]
#[test]
fn windows_unpaired_surrogate_is_rejected_without_lossy_conversion() {
    use std::os::windows::ffi::OsStringExt;
    assert_non_utf8_rejected(OsString::from_wide(&[0xd800]).into());
}

#[cfg(unix)]
#[test]
fn unix_invalid_utf8_is_rejected_without_lossy_conversion() {
    use std::os::unix::ffi::OsStringExt;
    assert_non_utf8_rejected(OsString::from_vec(vec![0xff]).into());
}

#[cfg(feature = "arbitrary")]
mod generated {
    use super::*;
    use arbitrary::Arbitrary;

    impl<'a> Arbitrary<'a> for OptionalPath {
        fn arbitrary(data: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
            // Every generated sample exercises a path; None has its own control.
            Ok(Self {
                log_file: Some(String::arbitrary(data)?.into()),
            })
        }
    }

    #[test]
    fn generated_path_consistency() {
        figue::assert_to_args_consistency::<OptionalPath>(figue::TestToArgsConsistencyConfig {
            success_count: 128,
            max_attempts: 10_000,
            root_seed: Some(42),
            ..Default::default()
        })
        .expect("generated path serialization should be consistent");
    }

    #[test]
    fn generated_path_roundtrips() {
        figue::assert_to_args_roundtrip::<OptionalPath>(figue::TestToArgsRoundTrip {
            success_count_global: 128,
            max_attempts_global: 10_000,
            root_seed: Some(42),
            ..Default::default()
        })
        .expect("generated path arguments should roundtrip");
    }
}
