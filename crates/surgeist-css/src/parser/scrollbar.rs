//! CSS Scrollbars 1 terminal color grammar.

use super::property_input::Parser;
use crate::diagnostic::error::property::ParseError;

use super::color::parse_color;
use crate::diagnostic::error::property::Error;
use crate::numeric::expression::NumericInputContext;
use crate::values::ui::CssScrollbarColor;

pub(super) fn parse_scrollbar_color<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssScrollbarColor, ParseError<'i, Error>> {
    if input
        .try_parse(|input| input.expect_ident_matching("auto"))
        .is_ok()
    {
        return Ok(CssScrollbarColor::auto());
    }
    input.skip_whitespace();
    let thumb = parse_color(input, numeric)?;
    input.skip_whitespace();
    let track = parse_color(input, numeric)?;
    Ok(CssScrollbarColor::new(thumb, track))
}
