//! Border color pairs and four-side authored grammar.

use cssparser::{ParseError, Parser};

use super::color::parse_color;
use crate::border_color::CssParsedBorderColorShorthand;
use crate::error::{Error, unsupported_value};
use crate::numeric::NumericInputContext;
use crate::{CssBorderColorPair, CssBoxSideKind};

pub(super) fn parse_border_color_pair<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssBorderColorPair, ParseError<'i, Error>> {
    let start = parse_color(input, numeric)?.into_parts().0;
    let authored_end = if input.is_exhausted() {
        None
    } else {
        Some(parse_color(input, numeric)?.into_parts().0)
    };
    Ok(CssBorderColorPair::new(start, authored_end))
}

pub(super) fn parse_border_colors<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssParsedBorderColorShorthand, ParseError<'i, Error>> {
    let kind = if input
        .try_parse(|input| input.expect_ident_matching("logical"))
        .is_ok()
    {
        CssBoxSideKind::Logical
    } else {
        CssBoxSideKind::Physical
    };
    let mut colors = Vec::new();
    while !input.is_exhausted() {
        colors.push(parse_color(input, numeric)?);
        if colors.len() == 4 && !input.is_exhausted() {
            return Err(unsupported_value(
                input,
                None,
                "border-color accepts at most four colors",
            ));
        }
    }
    CssParsedBorderColorShorthand::try_new(kind, colors)
        .ok_or_else(|| unsupported_value(input, None, "border-color requires one to four colors"))
}
