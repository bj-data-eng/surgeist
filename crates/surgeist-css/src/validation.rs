use crate::declaration::CssGlobalKeyword;
use crate::property::CssKnownProperty;

pub(crate) fn property_for_supported_name(name: &str) -> Option<CssKnownProperty> {
    CssKnownProperty::from_name(name)
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
