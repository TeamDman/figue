//! Transparent domain values should keep their ordinary scalar CLI syntax.

use facet::Facet;
use std::collections::BTreeMap;
use std::fmt::Debug;

#[derive(Facet, Debug, PartialEq)]
#[facet(transparent)]
struct Text(String);

#[derive(Facet, Debug, PartialEq)]
#[facet(transparent)]
struct NestedText(Text);

#[derive(Facet, Debug, PartialEq)]
#[facet(transparent)]
struct Limit(u64);

#[derive(Facet, Debug, PartialEq)]
#[facet(transparent)]
struct Ratio(f64);

#[derive(Facet, Debug, PartialEq)]
#[facet(transparent)]
struct Separator(char);

#[derive(Facet, Debug, PartialEq)]
#[facet(transparent)]
struct Enabled(bool);

impl TryFrom<String> for Enabled {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "yes" => Ok(Self(true)),
            "no" => Ok(Self(false)),
            _ => Err(format!("expected yes or no, got {value}")),
        }
    }
}

impl From<&Enabled> for String {
    fn from(value: &Enabled) -> Self {
        if value.0 { "yes" } else { "no" }.into()
    }
}

#[derive(Facet, Debug, PartialEq)]
struct ProxiedBoolArgs {
    #[facet(figue::named, proxy = String)]
    enabled: Enabled,
}

#[test]
fn transparent_boolean_does_not_override_an_explicit_field_proxy() {
    let config = figue::builder::<ProxiedBoolArgs>().unwrap().build();
    let (_, arg) = config.schema.args().args().get("enabled").unwrap();
    assert!(arg.required(), "String proxy must retain its requiredness");
    assert!(
        figue::from_slice::<ProxiedBoolArgs>(&[])
            .into_result()
            .is_err()
    );
    let value: ProxiedBoolArgs = parse(&["--enabled", "yes"]);
    assert!(value.enabled.0);
    roundtrip(&value);
}

#[derive(Facet, Debug, PartialEq)]
#[facet(transparent)]
struct BoolText(String);

impl TryFrom<BoolText> for bool {
    type Error = String;

    fn try_from(value: BoolText) -> Result<Self, Self::Error> {
        Enabled::try_from(value.0).map(|value| value.0)
    }
}

impl From<&bool> for BoolText {
    fn from(value: &bool) -> Self {
        Self(if *value { "yes" } else { "no" }.into())
    }
}

#[derive(Facet, Debug, PartialEq)]
#[facet(transparent)]
struct FieldProxiedBool(#[facet(proxy = BoolText)] bool);

#[derive(Facet, Debug, PartialEq)]
struct OptionalFieldProxiedArgs {
    #[facet(figue::named)]
    enabled: Option<FieldProxiedBool>,
}

#[test]
fn transparent_chain_does_not_bypass_an_inner_field_proxy() {
    let value: OptionalFieldProxiedArgs = parse(&["--enabled", "yes"]);
    assert_eq!(value.enabled, Some(FieldProxiedBool(true)));
    let false_value: OptionalFieldProxiedArgs = parse(&["--enabled", "no"]);
    assert_eq!(false_value.enabled, Some(FieldProxiedBool(false)));
    assert_eq!(parse::<OptionalFieldProxiedArgs>(&[]).enabled, None);
}

#[derive(Facet, Debug, PartialEq)]
struct DefaultTrueArgs {
    #[facet(figue::named, default = Enabled(true))]
    enabled: Enabled,
}

#[test]
fn transparent_boolean_preserves_an_explicit_true_default() {
    assert!(parse::<DefaultTrueArgs>(&[]).enabled.0);
    let disabled: DefaultTrueArgs = parse(&["--no-enabled"]);
    assert!(!disabled.enabled.0);
    roundtrip(&disabled);
    roundtrip(&DefaultTrueArgs {
        enabled: Enabled(true),
    });
}

#[derive(Facet, Debug, PartialEq)]
#[facet(transparent)]
struct OutputPath(camino::Utf8PathBuf);

#[derive(Facet, Debug, PartialEq)]
struct PathArgs {
    #[facet(figue::named)]
    output: OutputPath,
}

#[derive(Facet, Debug, PartialEq)]
struct PlainPathArgs {
    #[facet(figue::named)]
    output: camino::Utf8PathBuf,
}

#[test]
fn transparent_opaque_scalar_keeps_the_inner_cli_representation() {
    let expected = camino::Utf8PathBuf::from("reports/result.json");
    let plain: PlainPathArgs = parse(&["--output", "reports/result.json"]);
    assert_eq!(plain.output, expected);
    roundtrip(&plain);
    let wrapped: PathArgs = parse(&["--output", "reports/result.json"]);
    assert_eq!(wrapped.output, OutputPath(expected));
    roundtrip(&wrapped);
}

fn parse<T: Facet<'static>>(args: &[&str]) -> T {
    figue::from_slice(args)
        .into_result()
        .unwrap_or_else(|error| panic!("parse {args:?}: {error}"))
        .get_silent()
}

fn roundtrip<T: Facet<'static> + PartialEq + Debug>(value: &T) {
    let args =
        figue::to_os_args(value).unwrap_or_else(|error| panic!("serialize {value:?}: {error}"));
    let words = args
        .iter()
        .map(|word| word.to_str().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(&parse::<T>(&words), value, "emitted args: {words:?}");
}

#[derive(Facet, Debug, PartialEq)]
struct ScalarArgs {
    #[facet(figue::named)]
    tenant: NestedText,
    #[facet(figue::named)]
    limit: Limit,
    #[facet(figue::named)]
    ratio: Ratio,
    #[facet(figue::named)]
    separator: Separator,
}

#[test]
fn transparent_named_scalars_parse_and_roundtrip() {
    let value: ScalarArgs = parse(&[
        "--tenant",
        "development",
        "--limit",
        "18446744073709551615",
        "--ratio",
        "1.25",
        "--separator",
        "λ",
    ]);
    assert_eq!(
        value,
        ScalarArgs {
            tenant: NestedText(Text("development".into())),
            limit: Limit(u64::MAX),
            ratio: Ratio(1.25),
            separator: Separator('λ'),
        }
    );
    roundtrip(&value);
}

#[derive(Facet, Debug, PartialEq)]
struct PositionalArgs {
    #[facet(figue::positional)]
    tenant: Text,
}

#[test]
fn transparent_positional_parse_and_roundtrip() {
    let value: PositionalArgs = parse(&["development"]);
    assert_eq!(value.tenant, Text("development".into()));
    roundtrip(&value);
    roundtrip(&PositionalArgs {
        tenant: Text("--literal".into()),
    });
}

#[test]
fn transparent_emission_does_not_require_a_prior_parse() {
    roundtrip(&PositionalArgs {
        tenant: Text("development".into()),
    });
}

#[derive(Facet, Debug, PartialEq)]
struct BoolArgs {
    #[facet(figue::named)]
    enabled: Enabled,
}

#[test]
fn transparent_boolean_uses_boolean_flag_semantics() {
    assert_eq!(parse::<BoolArgs>(&[]).enabled, Enabled(false));
    assert_eq!(parse::<BoolArgs>(&["--enabled"]).enabled, Enabled(true));
    assert_eq!(parse::<BoolArgs>(&["--no-enabled"]).enabled, Enabled(false));
    roundtrip(&BoolArgs {
        enabled: Enabled(true),
    });
    roundtrip(&BoolArgs {
        enabled: Enabled(false),
    });
}

#[derive(Facet, Debug, PartialEq)]
struct Containers {
    #[facet(figue::named)]
    tenant: Option<Text>,
    #[facet(figue::named, default)]
    scopes: Vec<Text>,
}

#[test]
fn transparent_optional_and_repeated_values_keep_working() {
    let empty: Containers = parse(&[]);
    assert_eq!(
        empty,
        Containers {
            tenant: None,
            scopes: vec![]
        }
    );
    roundtrip(&empty);
    let value: Containers = parse(&[
        "--tenant",
        "development",
        "--scopes",
        "read",
        "--scopes",
        "write",
    ]);
    assert_eq!(
        value,
        Containers {
            tenant: Some(Text("development".into())),
            scopes: vec![Text("read".into()), Text("write".into())],
        }
    );
    roundtrip(&value);
}

#[derive(Facet)]
struct CommandArgs {
    #[facet(flatten)]
    builtins: figue::FigueBuiltins,
    #[facet(figue::subcommand)]
    command: Command,
}

#[derive(Facet)]
#[repr(u8)]
enum Command {
    Login(PositionalArgs),
}

#[test]
fn transparent_help_exposes_scalar_arguments() {
    let help = figue::generate_help::<ScalarArgs>(&figue::HelpConfig::default());
    assert!(!help.contains("Schema could not be built"), "{help}");
    for flag in ["--tenant", "--limit", "--ratio", "--separator"] {
        assert!(help.contains(flag), "{help}");
    }
    let config = figue::builder::<BoolArgs>().unwrap().build();
    let (_, enabled) = config.schema.args().args().get("enabled").unwrap();
    assert!(
        !enabled.required(),
        "transparent booleans should not be required"
    );

    let result = figue::from_slice::<CommandArgs>(&["login", "--help"]).into_result();
    let Err(figue::DriverError::Help { text, .. }) = result else {
        panic!("expected nested help");
    };
    assert!(text.contains("<TENANT>"), "{text}");
    let command: CommandArgs = parse(&["login", "development"]);
    let Command::Login(args) = command.command;
    assert_eq!(args.tenant, Text("development".into()));
}

#[derive(Facet, Debug, PartialEq)]
struct Settings {
    tenant: Text,
}

#[derive(Facet, Debug, PartialEq)]
struct SettingsArgs {
    #[facet(figue::config, figue::env_prefix = "TRANSPARENT")]
    settings: Settings,
}

#[test]
fn transparent_config_values_parse_from_each_source() {
    let file_config = figue::builder::<SettingsArgs>()
        .unwrap()
        .file(|file| file.content(r#"{"settings":{"tenant":"development"}}"#, "settings.json"))
        .build();
    let file_value = figue::Driver::new(file_config)
        .run()
        .into_result()
        .unwrap()
        .value;
    let env_config = figue::builder::<SettingsArgs>()
        .unwrap()
        .env(|env| {
            env.source(figue::MockEnv::from_pairs([(
                "TRANSPARENT__TENANT",
                "development",
            )]))
        })
        .build();
    let env_value = figue::Driver::new(env_config)
        .run()
        .into_result()
        .unwrap()
        .value;
    let cli_value: SettingsArgs = parse(&["--settings.tenant", "development"]);
    let expected = SettingsArgs {
        settings: Settings {
            tenant: Text("development".into()),
        },
    };
    assert_eq!(file_value, expected);
    assert_eq!(env_value, expected);
    assert_eq!(cli_value, expected);
}

#[derive(Facet)]
struct ExportedSchema {
    properties: BTreeMap<String, ExportedProperty>,
}

#[derive(Facet)]
struct ExportedProperty {
    #[facet(rename = "type")]
    property_type: String,
}

#[test]
fn transparent_config_schema_matches_the_string_wire_value() {
    let schemas = figue::generate_json_schemas::<SettingsArgs>().unwrap();
    let schema: ExportedSchema = facet_json::from_str(&schemas[0].contents).unwrap();
    assert_eq!(schema.properties["tenant"].property_type, "string");
}

#[derive(Facet)]
struct StructuredValue {
    value: String,
}

#[derive(Facet)]
#[facet(transparent)]
struct TransparentStruct(StructuredValue);

#[derive(Facet)]
struct StructuredArgs {
    #[facet(figue::named)]
    value: TransparentStruct,
}

#[test]
fn transparent_struct_is_not_implicitly_a_scalar_argument() {
    assert!(figue::builder::<StructuredArgs>().is_err());
}
