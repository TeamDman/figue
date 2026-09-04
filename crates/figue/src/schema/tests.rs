use super::*;
use crate as args;
use facet::Facet;
use facet_testhelpers::test;

macro_rules! assert_schema_snapshot {
    ($result:expr) => {{
        match $result {
            Ok(value) => insta::assert_snapshot!(facet_json::to_string_pretty(&value).unwrap()),
            Err(err) => {
                let rendered = err.to_string();
                let stripped = strip_ansi_escapes::strip(rendered.as_bytes());
                let stripped = String::from_utf8_lossy(&stripped);
                insta::assert_snapshot!(stripped);
            }
        }
    }};
}

#[derive(Facet)]
struct BasicArgs {
    /// Verbose output
    #[facet(args::named, args::short = 'v')]
    verbose: bool,
    /// Input file
    #[facet(args::positional)]
    input: String,
    /// Include list
    #[facet(args::named)]
    include: Vec<String>,
    /// Quiet count
    #[facet(args::named, args::short = 'q', args::counted)]
    quiet: u32,
    /// Subcommand
    #[facet(args::subcommand)]
    command: Option<Command>,
    /// Config
    #[facet(args::config, args::env_prefix = "APP")]
    config: Option<AppConfig>,
}

#[derive(Facet)]
#[repr(u8)]
enum Command {
    /// Build stuff
    Build(BuildArgs),
    /// Clean
    #[facet(rename = "clean-all")]
    Clean,
}

#[derive(Facet)]
struct BuildArgs {
    /// Release build
    #[facet(args::named, args::short = 'r')]
    release: bool,
}

#[derive(Facet)]
struct AppConfig {
    host: String,
    port: u16,
}

#[derive(Facet)]
struct MissingArgsAnnotation {
    foo: String,
}

#[derive(Facet)]
#[repr(u8)]
enum SubA {
    A,
}

#[derive(Facet)]
#[repr(u8)]
enum SubB {
    B,
}

#[derive(Facet)]
struct MultipleSubcommands {
    #[facet(args::subcommand)]
    a: SubA,
    #[facet(args::subcommand)]
    b: SubB,
}

#[derive(Facet)]
struct SubcommandOnNonEnum {
    #[facet(args::subcommand)]
    value: String,
}

#[derive(Facet)]
struct CountedOnNonInteger {
    #[facet(args::named, args::counted)]
    value: bool,
}

#[derive(Facet)]
struct ShortOnPositional {
    #[facet(args::positional, args::short = 'p')]
    value: String,
}

#[derive(Facet)]
struct EnvPrefixWithoutConfig {
    #[facet(args::env_prefix = "APP")]
    value: String,
}

#[derive(Facet)]
struct ConflictingLongFlags {
    #[facet(args::named, rename = "dup")]
    a: bool,
    #[facet(args::named, rename = "dup")]
    b: bool,
}

#[derive(Facet)]
struct ConflictingShortFlags {
    #[facet(args::named, args::short = 'v')]
    a: bool,
    #[facet(args::named, args::short = 'v')]
    b: bool,
}

#[derive(Facet)]
struct ArgsWithAlias {
    #[facet(
        args::named,
        rename = "drive",
        args::alias = "drive-letter-pattern"
    )]
    drive_letter_pattern: bool,
}

#[derive(Facet)]
struct ConflictingAliasAndCanonical {
    #[facet(args::named, args::alias = "port")]
    drive: bool,
    #[facet(args::named)]
    port: bool,
}

#[derive(Facet)]
struct DuplicateAliasOnField {
    #[facet(
        args::named,
        args::alias = "drive-letter-pattern",
        args::alias = "drive-letter-pattern"
    )]
    drive: bool,
}

#[derive(Facet)]
struct ConflictingAliases {
    #[facet(args::named, args::alias = "drive-letter-pattern")]
    drive: bool,
    #[facet(args::named, args::alias = "drive-letter-pattern")]
    letter: bool,
}

#[derive(Facet)]
#[repr(u8)]
enum SubcommandWithShort {
    #[facet(args::short = 'd')]
    Daemon,
    Doctor,
}

#[derive(Facet)]
struct ArgsWithSubcommandShort {
    #[facet(args::subcommand)]
    command: SubcommandWithShort,
}

#[derive(Facet)]
#[repr(u8)]
enum SubcommandShortConflictsWithFlagCommand {
    #[facet(args::short = 'd')]
    Daemon,
}

#[derive(Facet)]
struct SubcommandShortConflictsWithFlag {
    #[facet(args::named, args::short = 'd')]
    debug: bool,
    #[facet(args::subcommand)]
    command: SubcommandShortConflictsWithFlagCommand,
}

#[derive(Facet)]
#[repr(u8)]
enum SubcommandShortConflictsCommand {
    #[facet(args::short = 'd')]
    Daemon,
    #[facet(args::short = 'd')]
    Doctor,
}

#[derive(Facet)]
struct SubcommandShortConflicts {
    #[facet(args::subcommand)]
    command: SubcommandShortConflictsCommand,
}

#[derive(Facet)]
struct BadConfigField {
    #[facet(args::config)]
    config: String,
}

#[derive(Facet)]
#[repr(u8)]
enum TopLevelEnum {
    Foo,
}

#[test]
fn snapshot_schema_basic() {
    assert_schema_snapshot!(Schema::from_shape(BasicArgs::SHAPE));
}

#[test]
fn snapshot_schema_top_level_enum() {
    assert_schema_snapshot!(Schema::from_shape(TopLevelEnum::SHAPE));
}

#[test]
fn snapshot_schema_missing_args_annotation() {
    assert_schema_snapshot!(Schema::from_shape(MissingArgsAnnotation::SHAPE));
}

#[test]
fn snapshot_schema_multiple_subcommands() {
    assert_schema_snapshot!(Schema::from_shape(MultipleSubcommands::SHAPE));
}

#[test]
fn snapshot_schema_subcommand_on_non_enum() {
    assert_schema_snapshot!(Schema::from_shape(SubcommandOnNonEnum::SHAPE));
}

#[test]
fn snapshot_schema_counted_on_non_integer() {
    assert_schema_snapshot!(Schema::from_shape(CountedOnNonInteger::SHAPE));
}

#[test]
fn snapshot_schema_short_on_positional() {
    assert_schema_snapshot!(Schema::from_shape(ShortOnPositional::SHAPE));
}

#[test]
fn snapshot_schema_env_prefix_without_config() {
    assert_schema_snapshot!(Schema::from_shape(EnvPrefixWithoutConfig::SHAPE));
}

#[test]
fn snapshot_schema_conflicting_long_flags() {
    assert_schema_snapshot!(Schema::from_shape(ConflictingLongFlags::SHAPE));
}

#[test]
fn snapshot_schema_conflicting_short_flags() {
    assert_schema_snapshot!(Schema::from_shape(ConflictingShortFlags::SHAPE));
}

#[test]
fn test_schema_aliases_are_stored() {
    let schema = Schema::from_shape(ArgsWithAlias::SHAPE).unwrap();
    let arg = schema
        .args()
        .args()
        .get("drive")
        .expect("flag should be found")
        .1;
    assert_eq!(arg.name(), "drive");
    assert_eq!(arg.aliases(), &["drive-letter-pattern".to_string()]);
    assert!(arg.matches_long_flag("drive"));
    assert!(arg.matches_long_flag("drive-letter-pattern"));
}

#[test]
fn test_schema_alias_conflicts_with_canonical_flag() {
    let result = Schema::from_shape(ConflictingAliasAndCanonical::SHAPE);
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("duplicate flag `--port`"),
        "unexpected error: {err}"
    );
}

#[test]
fn test_schema_duplicate_alias_on_same_field_is_rejected() {
    let result = Schema::from_shape(DuplicateAliasOnField::SHAPE);
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("duplicate alias `--drive-letter-pattern`"),
        "unexpected error: {err}"
    );
}

#[test]
fn test_schema_duplicate_alias_across_fields_is_rejected() {
    let result = Schema::from_shape(ConflictingAliases::SHAPE);
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("duplicate flag `--drive-letter-pattern`"),
        "unexpected error: {err}"
    );
}

#[test]
fn test_schema_subcommand_short_stored() {
    let schema = Schema::from_shape(ArgsWithSubcommandShort::SHAPE).unwrap();
    let daemon = schema
        .args()
        .subcommands()
        .values()
        .find(|sub| sub.cli_name() == "daemon")
        .unwrap();
    assert_eq!(daemon.short(), Some('d'));
}

#[test]
fn test_schema_subcommand_short_conflicts_with_flag() {
    Schema::from_shape(SubcommandShortConflictsWithFlag::SHAPE)
        .expect("subcommand short alias 'd' should not conflict with flag short '-d'");
}

#[test]
fn test_schema_subcommand_short_conflicts_with_subcommand_short() {
    let result = Schema::from_shape(SubcommandShortConflicts::SHAPE);
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("duplicate subcommand short alias `d`"),
        "unexpected error: {err}"
    );
}

#[test]
fn snapshot_schema_bad_config_field() {
    assert_schema_snapshot!(Schema::from_shape(BadConfigField::SHAPE));
}

// ============================================================================
// Flatten tests
// ============================================================================

/// Common args that can be flattened into other structs.
#[derive(Facet)]
struct CommonArgs {
    #[facet(args::named, args::short = 'v')]
    verbose: bool,
    #[facet(args::named, args::short = 'q')]
    quiet: bool,
}

/// Args struct that flattens CommonArgs.
#[derive(Facet)]
struct ArgsWithFlatten {
    #[facet(args::positional)]
    input: String,
    #[facet(flatten)]
    common: CommonArgs,
}

#[test]
fn test_flatten_schema_builds() {
    let schema = Schema::from_shape(ArgsWithFlatten::SHAPE).expect("schema should build");

    // The flattened args should appear at top level
    let args = schema.args();
    assert!(
        args.args.contains_key("verbose"),
        "verbose should be in args"
    );
    assert!(args.args.contains_key("quiet"), "quiet should be in args");
    assert!(args.args.contains_key("input"), "input should be in args");
}

/// Nested flattening test structs
#[derive(Facet)]
struct OutputArgs {
    #[facet(args::named, args::short = 'f')]
    format: Option<String>,
}

#[derive(Facet)]
struct ExtendedCommonArgs {
    #[facet(flatten)]
    common: CommonArgs,
    #[facet(flatten)]
    output: OutputArgs,
}

#[derive(Facet)]
struct ArgsWithNestedFlatten {
    #[facet(args::positional)]
    input: String,
    #[facet(flatten)]
    extended: ExtendedCommonArgs,
}

/// Test conflicting flags from flatten
#[derive(Facet)]
struct ConflictingFlattenArgs {
    #[facet(args::named, args::short = 'v')]
    version: bool,
    #[facet(flatten)]
    common: CommonArgs, // CommonArgs also has -v for verbose
}

#[test]
fn test_flatten_conflict_detected() {
    let result = Schema::from_shape(ConflictingFlattenArgs::SHAPE);
    assert!(result.is_err(), "should detect duplicate -v flag");
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("duplicate") || err.contains("-v"),
        "error should mention duplicate: {}",
        err
    );
}

// ============================================================================
// Config-level flatten tests
// ============================================================================

/// Common config fields that can be flattened
#[derive(Facet)]
struct CommonConfig {
    /// Log level
    log_level: Option<String>,
    /// Debug mode
    debug: bool,
}

/// Database config
#[derive(Facet)]
struct DatabaseConfig {
    /// Database host
    host: String,
    /// Database port
    port: u16,
}

/// Config with flattened common fields
#[derive(Facet)]
struct ConfigWithFlatten {
    /// Application name
    name: String,
    /// Common settings
    #[facet(flatten)]
    common: CommonConfig,
}

/// Args with config that has flatten
#[derive(Facet)]
struct ArgsWithFlattenedConfig {
    #[facet(args::positional)]
    input: String,
    #[facet(args::config)]
    config: ConfigWithFlatten,
}

#[test]
fn test_config_flatten_schema_builds() {
    let schema = Schema::from_shape(ArgsWithFlattenedConfig::SHAPE).expect("schema should build");
    let config = schema.configs().first().expect("should have config");
    let fields = config.fields();

    // Should have 3 fields: name, log_level, debug (flattened from common)
    assert_eq!(fields.len(), 3, "should have 3 fields after flatten");
    assert!(fields.contains_key("name"), "should have name field");
    assert!(
        fields.contains_key("log_level"),
        "should have log_level from flattened common"
    );
    assert!(
        fields.contains_key("debug"),
        "should have debug from flattened common"
    );
}

/// Deeply nested config flatten: common inside extended
#[derive(Facet)]
struct ExtendedConfig {
    #[facet(flatten)]
    common: CommonConfig,
    #[facet(flatten)]
    database: DatabaseConfig,
}

#[derive(Facet)]
struct ConfigWithNestedFlatten {
    app_name: String,
    #[facet(flatten)]
    extended: ExtendedConfig,
}

#[derive(Facet)]
struct ArgsWithNestedFlattenConfig {
    #[facet(args::positional)]
    input: String,
    #[facet(args::config)]
    config: ConfigWithNestedFlatten,
}

#[test]
fn test_config_nested_flatten_schema_builds() {
    let schema =
        Schema::from_shape(ArgsWithNestedFlattenConfig::SHAPE).expect("schema should build");
    let config = schema.configs().first().expect("should have config");
    let fields = config.fields();

    // Should have 5 fields: app_name + log_level, debug (from common) + host, port (from database)
    assert_eq!(fields.len(), 5, "should have 5 fields after nested flatten");
    assert!(fields.contains_key("app_name"), "should have app_name");
    assert!(fields.contains_key("log_level"), "should have log_level");
    assert!(fields.contains_key("debug"), "should have debug");
    assert!(fields.contains_key("host"), "should have host");
    assert!(fields.contains_key("port"), "should have port");
}

/// Test conflict detection in config flatten
#[derive(Facet)]
struct ConflictingConfigA {
    name: String,
}

#[derive(Facet)]
struct ConflictingConfigB {
    name: String, // Same field name as ConflictingConfigA
}

#[derive(Facet)]
struct ConfigWithConflictingFlatten {
    #[facet(flatten)]
    a: ConflictingConfigA,
    #[facet(flatten)]
    b: ConflictingConfigB,
}

#[derive(Facet)]
struct ArgsWithConflictingConfigFlatten {
    #[facet(args::positional)]
    input: String,
    #[facet(args::config)]
    config: ConfigWithConflictingFlatten,
}

#[test]
fn test_config_flatten_conflict_detected() {
    let result = Schema::from_shape(ArgsWithConflictingConfigFlatten::SHAPE);
    assert!(result.is_err(), "should detect duplicate config field");
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("duplicate") || err.contains("name"),
        "error should mention duplicate: {}",
        err
    );
}

// ============================================================================
// Struct fields in args must be flattened
// ============================================================================

#[derive(Facet)]
struct NestedOptions {
    #[facet(args::named)]
    verbose: bool,
}

#[derive(Facet)]
struct ArgsWithUnflattenedStruct {
    #[facet(args::named)]
    options: NestedOptions, // ERROR: struct fields must use flatten
}

impl TryFrom<String> for NestedOptions {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Ok(Self {
            verbose: value == "verbose",
        })
    }
}

impl TryFrom<&NestedOptions> for String {
    type Error = String;

    fn try_from(value: &NestedOptions) -> Result<Self, Self::Error> {
        Ok(value
            .verbose
            .then_some("verbose")
            .unwrap_or_default()
            .to_string())
    }
}

impl TryFrom<bool> for NestedOptions {
    type Error = String;

    fn try_from(verbose: bool) -> Result<Self, Self::Error> {
        Ok(Self { verbose })
    }
}

impl TryFrom<&NestedOptions> for bool {
    type Error = String;

    fn try_from(value: &NestedOptions) -> Result<Self, Self::Error> {
        Ok(value.verbose)
    }
}

#[derive(Facet)]
struct ArgsWithGenericFieldProxy {
    #[facet(args::positional, proxy = String)]
    options: NestedOptions,
}

#[derive(Facet)]
struct ArgsWithFigueFieldProxy {
    #[facet(args::named, proxy = String, figue::proxy = bool)]
    options: NestedOptions,
}

#[derive(Facet)]
struct ConfigWithGenericFieldProxy {
    #[facet(proxy = String)]
    options: NestedOptions,
}

#[derive(Facet)]
struct ArgsWithGenericConfigFieldProxy {
    #[facet(args::config)]
    config: ConfigWithGenericFieldProxy,
}

#[derive(Facet)]
#[repr(u8)]
enum ConfigEnumWithGenericFieldProxy {
    Selected {
        #[facet(proxy = String)]
        options: NestedOptions,
    },
}

#[derive(Facet)]
struct ConfigWithEnumFieldProxy {
    selection: ConfigEnumWithGenericFieldProxy,
}

#[derive(Facet)]
struct ArgsWithEnumConfigFieldProxy {
    #[facet(args::config)]
    config: ConfigWithEnumFieldProxy,
}

#[derive(Facet)]
#[facet(transparent)]
struct OptionalOptionsProxy(String);

impl TryFrom<OptionalOptionsProxy> for Option<NestedOptions> {
    type Error = String;

    fn try_from(value: OptionalOptionsProxy) -> Result<Self, Self::Error> {
        Ok(Some(NestedOptions::try_from(value.0)?))
    }
}

impl TryFrom<&Option<NestedOptions>> for OptionalOptionsProxy {
    type Error = String;

    fn try_from(value: &Option<NestedOptions>) -> Result<Self, Self::Error> {
        let value = value
            .as_ref()
            .map(String::try_from)
            .transpose()?
            .unwrap_or_default();
        Ok(Self(value))
    }
}

#[derive(Facet)]
struct ArgsWithOptionalFieldProxy {
    #[facet(args::named, proxy = OptionalOptionsProxy)]
    options: Option<NestedOptions>,
}

#[derive(Facet)]
struct ConfigWithOptionalFieldProxy {
    #[facet(proxy = OptionalOptionsProxy)]
    options: Option<NestedOptions>,
}

#[derive(Facet)]
struct ArgsWithOptionalConfigFieldProxy {
    #[facet(args::config)]
    config: ConfigWithOptionalFieldProxy,
}

#[derive(Facet)]
struct ArgsWithFlattenedFieldProxy {
    #[facet(flatten, proxy = String)]
    options: NestedOptions,
}

#[derive(Facet)]
struct ConfigWithFlattenedFieldProxy {
    #[facet(flatten, proxy = String)]
    options: NestedOptions,
}

#[derive(Facet)]
struct ArgsWithFlattenedConfigFieldProxy {
    #[facet(args::config)]
    config: ConfigWithFlattenedFieldProxy,
}

#[derive(Facet)]
#[facet(transparent)]
struct TransparentPattern(String);

#[derive(Facet)]
struct ArgsWithTransparentNewtype {
    #[facet(args::named)]
    pattern: TransparentPattern,
}

#[test]
fn test_struct_field_without_flatten_is_error() {
    let result = Schema::from_shape(ArgsWithUnflattenedStruct::SHAPE);
    assert!(result.is_err(), "struct field without flatten should error");
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("flatten"),
        "error should mention flatten: {}",
        err
    );
}

#[test]
fn test_transparent_newtype_arg_is_allowed() {
    let schema = Schema::from_shape(ArgsWithTransparentNewtype::SHAPE)
        .expect("transparent newtype args should be treated like their inner leaf type");

    assert!(
        schema.args().args.contains_key("pattern"),
        "transparent newtype field should appear as a regular named arg"
    );
}

#[test]
fn field_proxy_makes_a_struct_positional_argument_a_scalar() {
    let schema = Schema::from_shape(ArgsWithGenericFieldProxy::SHAPE)
        .expect("a field proxy should make a struct usable as a positional argument");
    let arg = schema
        .args()
        .args()
        .iter()
        .find(|(name, _)| name.as_str() == "options")
        .expect("positional argument should be present")
        .1;

    assert_eq!(arg.value().type_identifier(), String::SHAPE.type_identifier);
    assert!(!matches!(arg.value(), ValueSchema::Struct { .. }));
}

#[test]
fn field_proxy_prefers_figue_specific_representation() {
    let schema = Schema::from_shape(ArgsWithFigueFieldProxy::SHAPE)
        .expect("the Figue-specific field proxy should build a schema");
    let arg = schema
        .args()
        .args()
        .get("options")
        .expect("named argument should be present")
        .1;

    assert!(arg.value().is_bool());
}

#[test]
fn field_proxy_is_used_for_nested_config_values() {
    let schema = Schema::from_shape(ArgsWithGenericConfigFieldProxy::SHAPE)
        .expect("a config field proxy should build a schema");
    let value = schema
        .configs()
        .first()
        .expect("config root should be present")
        .fields()
        .get("options")
        .expect("config field should be present")
        .value();

    assert_eq!(value.type_identifier(), String::SHAPE.type_identifier);
    assert!(!matches!(value, ConfigValueSchema::Struct(_)));
}

#[test]
fn field_proxy_is_used_for_config_enum_variant_values() {
    let schema = Schema::from_shape(ArgsWithEnumConfigFieldProxy::SHAPE)
        .expect("a config enum field proxy should build a schema");
    let ConfigValueSchema::Enum(selection) = schema
        .configs()
        .first()
        .expect("config root should be present")
        .fields()
        .get("selection")
        .expect("enum config field should be present")
        .value()
    else {
        panic!("selection should use an enum config schema");
    };
    let value = selection
        .variants()
        .values()
        .next()
        .expect("enum variant should be present")
        .fields()
        .get("options")
        .expect("variant field should be present")
        .value();

    assert_eq!(value.type_identifier(), String::SHAPE.type_identifier);
}

#[test]
fn field_proxy_preserves_optional_argument_presence() {
    let schema = Schema::from_shape(ArgsWithOptionalFieldProxy::SHAPE)
        .expect("an optional field proxy should build a schema");
    let arg = schema
        .args()
        .args()
        .get("options")
        .expect("named argument should be present")
        .1;

    assert!(arg.value().is_option());
    assert!(!arg.required());
    assert_eq!(
        arg.value().inner_if_option().type_identifier(),
        String::SHAPE.type_identifier
    );
}

#[test]
fn field_proxy_preserves_optional_config_presence() {
    let schema = Schema::from_shape(ArgsWithOptionalConfigFieldProxy::SHAPE)
        .expect("an optional config field proxy should build a schema");
    let value = schema
        .configs()
        .first()
        .expect("config root should be present")
        .fields()
        .get("options")
        .expect("config field should be present")
        .value();

    assert!(value.is_option());
    assert_eq!(
        value.inner_if_option().type_identifier(),
        String::SHAPE.type_identifier
    );
}

#[test]
fn field_proxy_is_rejected_when_flattening_args() {
    let err = Schema::from_shape(ArgsWithFlattenedFieldProxy::SHAPE)
        .expect_err("a direct field proxy cannot be flattened");

    assert!(err.to_string().contains("cannot use a field-level proxy"));
}

#[test]
fn field_proxy_is_rejected_when_flattening_config() {
    let err = Schema::from_shape(ArgsWithFlattenedConfigFieldProxy::SHAPE)
        .expect_err("a direct field proxy cannot be flattened");

    assert!(err.to_string().contains("cannot use a field-level proxy"));
}

// ============================================================================
// Env alias conflict detection
// ============================================================================

#[derive(Facet)]
struct ConfigWithConflictingAliases {
    #[facet(args::env_alias = "DATABASE_URL")]
    db_url: String,
    #[facet(args::env_alias = "DATABASE_URL")]
    connection_string: String,
}

#[derive(Facet)]
struct ArgsWithConflictingAliases {
    #[facet(args::config)]
    config: ConfigWithConflictingAliases,
}

#[derive(Facet)]
#[repr(u8)]
enum CommandWithDuplicateAliasOnVariant {
    #[facet(args::alias = "profiles", args::alias = "profiles")]
    Profile,
}

#[derive(Facet)]
struct ArgsWithDuplicateAliasOnVariant {
    #[facet(args::subcommand)]
    command: CommandWithDuplicateAliasOnVariant,
}

#[derive(Facet)]
#[repr(u8)]
enum CommandWithAliasCanonicalConflict {
    #[facet(args::alias = "profiles")]
    Profile,
    Profiles,
}

#[derive(Facet)]
struct ArgsWithAliasCanonicalConflict {
    #[facet(args::subcommand)]
    command: CommandWithAliasCanonicalConflict,
}

#[derive(Facet)]
#[repr(u8)]
enum CommandWithAliasAliasConflict {
    #[facet(args::alias = "profiles")]
    Profile,
    #[facet(args::alias = "profiles")]
    Group,
}

#[derive(Facet)]
struct ArgsWithAliasAliasConflict {
    #[facet(args::subcommand)]
    command: CommandWithAliasAliasConflict,
}

#[test]
fn test_env_alias_conflict_detected() {
    let result = Schema::from_shape(ArgsWithConflictingAliases::SHAPE);
    assert!(result.is_err(), "should detect duplicate env alias");
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("DATABASE_URL") && err.contains("db_url") && err.contains("connection_string"),
        "error should mention the alias and both fields: {}",
        err
    );
}

#[test]
fn test_subcommand_duplicate_alias_on_same_variant_detected() {
    let result = Schema::from_shape(ArgsWithDuplicateAliasOnVariant::SHAPE);
    assert!(
        result.is_err(),
        "should detect duplicate alias on one variant"
    );
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("duplicate subcommand alias") && err.contains("profiles"),
        "error should mention the duplicate alias: {}",
        err
    );
}

#[test]
fn test_subcommand_alias_conflict_with_canonical_name_detected() {
    let result = Schema::from_shape(ArgsWithAliasCanonicalConflict::SHAPE);
    assert!(result.is_err(), "should detect alias/canonical conflict");
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("duplicate subcommand name") && err.contains("profiles"),
        "error should mention the conflicting subcommand name: {}",
        err
    );
}

#[test]
fn test_subcommand_alias_conflict_with_other_alias_detected() {
    let result = Schema::from_shape(ArgsWithAliasAliasConflict::SHAPE);
    assert!(result.is_err(), "should detect alias/alias conflict");
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("duplicate subcommand name") && err.contains("profiles"),
        "error should mention the conflicting alias: {}",
        err
    );
}


