//! Border color pairs and four-side authored grammar.

use super::property_input::Stream;
use super::side_shapes::{parse_four_sides, parse_side_pair};
use crate::diagnostic::error::property::{Error, SyntaxError};

use super::color::parse_color;

use crate::color::CssColor;
use crate::numeric::expression::NumericInputContext;
use crate::values::common::{CssFourSides, CssSidePair};

pub(super) fn parse_border_color_pair<'i>(
    input: &mut Stream<'i>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssSidePair<CssColor>, SyntaxError<'i, Error>> {
    parse_side_pair(input, |input| parse_color(input, numeric))
}

pub(super) fn parse_border_colors<'i>(
    input: &mut Stream<'i>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssFourSides<CssColor>, SyntaxError<'i, Error>> {
    parse_four_sides(input, |input| parse_color(input, numeric))
}
