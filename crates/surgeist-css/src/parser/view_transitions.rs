//! CSS View Transitions 1 authored property grammar.

use super::property_input::Stream;
use crate::diagnostic::error::property::{Error, SyntaxError, basic, unexpected_at};
use crate::values::common::CssCustomIdent;
use crate::values::view_transition::{CssViewTransitionIdent, CssViewTransitionName};

pub(crate) fn parse_view_transition_name<'i>(
    input: &mut Stream<'i>,
) -> Result<CssViewTransitionName, SyntaxError<'i, Error>> {
    let location = input.site();
    let name = input.consume_ident().map_err(basic)?;
    if name.eq_ignore_ascii_case("none") {
        return Ok(CssViewTransitionName::None);
    }
    CssCustomIdent::try_new(name.to_string())
        .and_then(CssViewTransitionIdent::try_new)
        .map(CssViewTransitionName::Custom)
        .ok_or_else(|| unexpected_at(location))
}
