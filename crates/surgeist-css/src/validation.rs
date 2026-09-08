use crate::{CssGlobalKeyword, CssKnownProperty, CssLengthUnit};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum LengthUnitStatus {
    Supported(CssLengthUnit),
    Unknown,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PropertyNameStatus {
    Supported,
    KnownUnsupported,
    Unknown,
}

const KNOWN_UNSUPPORTED_PROPERTY_NAMES: &[&str] = &[];

pub(crate) fn property_for_supported_name(name: &str) -> Option<CssKnownProperty> {
    CssKnownProperty::from_name(name)
}

pub(crate) fn classify_property_name(name: &str) -> PropertyNameStatus {
    if property_for_supported_name(name).is_some() {
        PropertyNameStatus::Supported
    } else if contains_ascii_case(KNOWN_UNSUPPORTED_PROPERTY_NAMES, name) {
        PropertyNameStatus::KnownUnsupported
    } else {
        PropertyNameStatus::Unknown
    }
}

pub(crate) fn classify_length_unit(unit: &str) -> LengthUnitStatus {
    CssLengthUnit::from_css_unit(unit)
        .map_or(LengthUnitStatus::Unknown, LengthUnitStatus::Supported)
}

pub(crate) fn parse_global_keyword(name: &str) -> Option<CssGlobalKeyword> {
    if name.eq_ignore_ascii_case("inherit") {
        Some(CssGlobalKeyword::Inherit)
    } else if name.eq_ignore_ascii_case("initial") {
        Some(CssGlobalKeyword::Initial)
    } else if name.eq_ignore_ascii_case("unset") {
        Some(CssGlobalKeyword::Unset)
    } else if name.eq_ignore_ascii_case("revert") {
        Some(CssGlobalKeyword::Revert)
    } else if name.eq_ignore_ascii_case("revert-layer") {
        Some(CssGlobalKeyword::RevertLayer)
    } else {
        None
    }
}

pub(crate) fn unsupported_keyword_reason(domain: &str, keyword: impl AsRef<str>) -> String {
    format!("unsupported {domain} keyword `{}`", keyword.as_ref())
}

fn contains_ascii_case(haystack: &[&str], needle: &str) -> bool {
    haystack
        .iter()
        .any(|candidate| candidate.eq_ignore_ascii_case(needle))
}
