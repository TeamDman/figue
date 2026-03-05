use facet::Facet;
use facet_testhelpers::test;
use figue::{self as args, ToArgs};

#[derive(Facet, Debug, PartialEq)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[repr(u8)]
enum Command {
    Build {
        #[facet(args::named, args::short = 'r')]
        release: bool,

        #[facet(args::positional)]
        target: Option<String>,
    },
    Clean,
}

#[derive(Facet, Debug, PartialEq)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
struct Cli {
    #[facet(args::named, args::short = 'v')]
    verbose: bool,

    #[facet(args::subcommand)]
    command: Command,
}

#[test]
fn test_to_args_roundtrip() {
    let original = Cli {
        verbose: true,
        command: Command::Build {
            release: true,
            target: Some("app".to_string()),
        },
    };

    let args = original
        .to_args()
        .expect("to_args should serialize CLI value");
    let arg_strings = args
        .iter()
        .map(|arg| arg.to_string_lossy().to_string())
        .collect::<Vec<_>>();
    let arg_refs = arg_strings.iter().map(String::as_str).collect::<Vec<_>>();

    let parsed: Cli = figue::from_slice(&arg_refs)
        .into_result()
        .expect("roundtrip parse should succeed")
        .get_silent();

    assert_eq!(original, parsed);
}

#[test]
fn test_to_args_deterministic() {
    let cli = Cli {
        verbose: true,
        command: Command::Build {
            release: false,
            target: Some("worker".to_string()),
        },
    };

    let args1 = cli.to_args().expect("first conversion should succeed");
    let args2 = cli.to_args().expect("second conversion should succeed");

    assert_eq!(args1, args2);
}

#[cfg(feature = "arbitrary")]
#[test]
fn test_consumer_helper_assert_to_args_consistency() {
    figue::assert_to_args_consistency::<Cli>(8)
        .expect("consumer helper consistency check should pass");
}

#[cfg(feature = "arbitrary")]
#[test]
fn test_consumer_helper_assert_to_args_roundtrip() {
    figue::assert_to_args_roundtrip::<Cli>(4)
        .expect("consumer helper roundtrip check should pass");
}
