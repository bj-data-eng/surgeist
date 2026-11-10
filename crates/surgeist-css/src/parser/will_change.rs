//! CSS Will Change 1 §2 authored grammar.

use super::syntax_types::Stream;
use crate::diagnostic::error::property::{Error, basic, unexpected_at};
use crate::parser::syntax_types::SyntaxError;
use crate::values::ui::{
    CssWillChange, CssWillChangeFeature, CssWillChangeFeatures, CssWillChangePropertyName,
};

pub(super) fn parse_will_change<'i>(
    input: &mut Stream<'i>,
) -> Result<CssWillChange, SyntaxError<'i, Error>> {
    if input.attempt(|input| input.consume_keyword("auto")).is_ok() {
        return Ok(CssWillChange::Auto);
    }
    let features = input.comma_segments(|input| {
        let location = input.site();
        let ident = input.consume_ident().map_err(basic)?;
        if ident.eq_ignore_ascii_case("scroll-position") {
            return Ok(CssWillChangeFeature::ScrollPosition);
        }
        if ident.eq_ignore_ascii_case("contents") {
            return Ok(CssWillChangeFeature::Contents);
        }
        CssWillChangePropertyName::try_new(ident.to_string())
            .map(CssWillChangeFeature::Property)
            .ok_or_else(|| unexpected_at(location))
    })?;
    Ok(CssWillChange::Features(
        CssWillChangeFeatures::try_new(features).expect("parser requires a nonempty list"),
    ))
}
