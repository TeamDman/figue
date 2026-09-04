use facet::Facet;
use figue::{self as args, ToArgs};

#[derive(Debug, Facet, PartialEq, Eq)]
struct Options {
    enabled: bool,
}

impl TryFrom<String> for Options {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "enabled" => Ok(Self { enabled: true }),
            "disabled" => Ok(Self { enabled: false }),
            _ => Err(format!("unexpected options value: {value}")),
        }
    }
}

impl TryFrom<&Options> for String {
    type Error = String;

    fn try_from(value: &Options) -> Result<Self, Self::Error> {
        Ok(if value.enabled { "enabled" } else { "disabled" }.to_string())
    }
}

impl TryFrom<bool> for Options {
    type Error = String;

    fn try_from(enabled: bool) -> Result<Self, Self::Error> {
        Ok(Self { enabled })
    }
}

impl TryFrom<&Options> for bool {
    type Error = String;

    fn try_from(value: &Options) -> Result<Self, Self::Error> {
        Ok(value.enabled)
    }
}

#[derive(Debug, Facet, PartialEq, Eq)]
struct FigueSpecificProxyArgs {
    #[facet(args::named, proxy = String, figue::proxy = bool)]
    options: Options,
}

#[derive(Debug, Facet, PartialEq, Eq)]
#[facet(transparent)]
struct ListProxy(String);

impl TryFrom<ListProxy> for Vec<String> {
    type Error = String;

    fn try_from(value: ListProxy) -> Result<Self, Self::Error> {
        Ok(vec![value.0])
    }
}

impl TryFrom<&Vec<String>> for ListProxy {
    type Error = String;

    fn try_from(value: &Vec<String>) -> Result<Self, Self::Error> {
        match value.as_slice() {
            [value] => Ok(Self(value.clone())),
            _ => Err("test proxy only supports one item".to_string()),
        }
    }
}

#[derive(Debug, Facet, PartialEq, Eq)]
struct ListProxyArgs {
    #[facet(args::named, proxy = ListProxy)]
    values: Vec<String>,
}

#[derive(Debug, Facet, PartialEq, Eq)]
#[facet(transparent)]
struct OptionalOptionsProxy(String);

impl TryFrom<OptionalOptionsProxy> for Option<Options> {
    type Error = String;

    fn try_from(value: OptionalOptionsProxy) -> Result<Self, Self::Error> {
        if value.0.is_empty() {
            Ok(None)
        } else {
            Options::try_from(value.0).map(Some)
        }
    }
}

impl TryFrom<&Option<Options>> for OptionalOptionsProxy {
    type Error = String;

    fn try_from(value: &Option<Options>) -> Result<Self, Self::Error> {
        value
            .as_ref()
            .map(String::try_from)
            .transpose()
            .map(|value| Self(value.unwrap_or_default()))
    }
}

#[derive(Debug, Facet, PartialEq, Eq)]
struct OptionalProxyArgs {
    #[facet(args::named, proxy = OptionalOptionsProxy)]
    options: Option<Options>,
}

#[test]
fn field_proxy_prefers_figue_specific_representation_at_runtime() {
    let parsed: FigueSpecificProxyArgs = args::from_slice(&["--options"]).unwrap();
    assert_eq!(parsed.options, Options { enabled: true });

    let omitted: FigueSpecificProxyArgs = args::from_slice(&[]).unwrap();
    assert_eq!(omitted.options, Options { enabled: false });

    let args = parsed
        .to_args()
        .expect("the Figue-specific bool proxy should serialize as a flag");
    let args = args
        .iter()
        .map(|arg| arg.to_string_lossy().to_string())
        .collect::<Vec<_>>();
    assert_eq!(args, vec!["--options".to_string()]);
}

#[test]
fn field_proxy_representation_is_preserved_during_runtime_coercion() {
    let parsed: ListProxyArgs = args::from_slice(&["--values", "alpha"]).unwrap();

    assert_eq!(parsed.values, ["alpha"]);
}

#[test]
fn field_proxy_preserves_optional_presence_at_runtime() {
    let omitted: OptionalProxyArgs = args::from_slice(&[]).unwrap();
    assert_eq!(omitted.options, None);

    let supplied: OptionalProxyArgs = args::from_slice(&["--options", "enabled"]).unwrap();
    assert_eq!(supplied.options, Some(Options { enabled: true }));
}
