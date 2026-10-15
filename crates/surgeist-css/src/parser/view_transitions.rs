//! CSS View Transitions 1 authored property grammar.

use super::property_input::Parser;
use crate::error::property::{Error, ParseError, basic, unexpected_at};
use crate::{CssCustomIdent, CssViewTransitionIdent, CssViewTransitionName};

pub(super) fn parse_view_transition_name<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> Result<CssViewTransitionName, ParseError<'i, Error>> {
    let location = input.current_source_location();
    let name = input.expect_ident_cloned().map_err(basic)?;
    if name.eq_ignore_ascii_case("none") {
        return Ok(CssViewTransitionName::None);
    }
    CssCustomIdent::try_new(name.to_string())
        .and_then(CssViewTransitionIdent::try_new)
        .map(CssViewTransitionName::Custom)
        .ok_or_else(|| unexpected_at(location))
}
