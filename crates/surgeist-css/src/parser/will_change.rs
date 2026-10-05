//! CSS Will Change 1 §2 authored grammar.

use crate::error::{Error, basic, unsupported_value_at};
use crate::{
    CssWillChange, CssWillChangeFeature, CssWillChangeFeatures, CssWillChangePropertyName,
};
use cssparser::{ParseError, Parser};

pub(super) fn parse_will_change<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> Result<CssWillChange, ParseError<'i, Error>> {
    if input
        .try_parse(|input| input.expect_ident_matching("auto"))
        .is_ok()
    {
        return Ok(CssWillChange::Auto);
    }
    let features = input.parse_comma_separated(|input| {
        let location = input.current_source_location();
        let ident = input.expect_ident_cloned().map_err(basic)?;
        if ident.eq_ignore_ascii_case("scroll-position") {
            return Ok(CssWillChangeFeature::ScrollPosition);
        }
        if ident.eq_ignore_ascii_case("contents") {
            return Ok(CssWillChangeFeature::Contents);
        }
        CssWillChangePropertyName::try_new(ident.to_string())
            .map(CssWillChangeFeature::Property)
            .ok_or_else(|| unsupported_value_at(location, None, "excluded will-change identifier"))
    })?;
    Ok(CssWillChange::Features(
        CssWillChangeFeatures::try_new(features).expect("parser requires a nonempty list"),
    ))
}
