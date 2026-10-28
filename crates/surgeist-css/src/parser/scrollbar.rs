//! CSS Scrollbars 1 terminal color grammar.

use super::syntax_types::Stream;
use crate::parser::syntax_types::SyntaxError;

use super::color::parse_color;
use crate::diagnostic::error::property::Error;
use crate::numeric::expression::NumericInputContext;
use crate::values::ui::CssScrollbarColor;

pub(super) fn parse_scrollbar_color<'i>(
    input: &mut Stream<'i>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssScrollbarColor, SyntaxError<'i, Error>> {
    if input.attempt(|input| input.consume_keyword("auto")).is_ok() {
        return Ok(CssScrollbarColor::auto());
    }
    input.discard_whitespace();
    let thumb = parse_color(input, numeric)?;
    input.discard_whitespace();
    let track = parse_color(input, numeric)?;
    Ok(CssScrollbarColor::new(thumb, track))
}
