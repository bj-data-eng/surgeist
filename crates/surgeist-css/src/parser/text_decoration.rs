//! Selected Decoration 4 intrinsic grammars.
use super::color::parse_color;
use super::values::parse_length_percentage;
use crate::error::{Error, basic, unsupported_value, unsupported_value_at};
use crate::*;
use cssparser::{ParseError, Parser, match_ignore_ascii_case};

macro_rules! keyword {
    ($parser:ident, $ty:ident, $($text:literal => $variant:ident),+ $(,)?) => {
        pub(super) fn $parser<'i, 't>(input: &mut Parser<'i, 't>) -> std::result::Result<$ty, ParseError<'i, Error>> {
            let ident = input.expect_ident_cloned().map_err(basic)?;
            match_ignore_ascii_case! { &ident, $($text => Ok($ty::$variant),)+ _ => Err(unsupported_value(input, None, "invalid decoration keyword")) }
        }
    };
}
keyword!(parse_decoration_error, CssTextDecorationError, "spelling-error" => SpellingError, "grammar-error" => GrammarError);
keyword!(parse_underline_vertical, CssUnderlinePositionVertical, "from-font" => FromFont, "under" => Under);
keyword!(parse_text_side, CssTextSide, "left" => Left, "right" => Right);
keyword!(parse_text_decoration_skip, CssTextDecorationSkip, "none" => None, "auto" => Auto);
keyword!(parse_text_decoration_skip_self, CssTextDecorationSkipSelf, "none" => None, "objects" => Objects);
keyword!(parse_text_decoration_skip_box, CssTextDecorationSkipBox, "none" => None, "all" => All);
keyword!(parse_text_decoration_skip_inset, CssTextDecorationSkipInset, "none" => None, "auto" => Auto);
keyword!(parse_text_decoration_skip_ink, CssTextDecorationSkipInk, "none" => None, "auto" => Auto, "all" => All);
keyword!(parse_emphasis_fill, CssTextEmphasisFill, "filled" => Filled, "open" => Open);
keyword!(parse_emphasis_shape, CssTextEmphasisShape, "dot" => Dot, "circle" => Circle, "double-circle" => DoubleCircle, "triangle" => Triangle, "sesame" => Sesame);
keyword!(parse_emphasis_vertical, CssTextEmphasisVertical, "over" => Over, "under" => Under);
pub(super) fn parse_text_underline_position<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssTextUnderlinePosition, ParseError<'i, Error>> {
    if input
        .try_parse(|input| input.expect_ident_matching("auto"))
        .is_ok()
    {
        return Ok(CssTextUnderlinePosition::Auto);
    }
    let mut vertical = None;
    let mut side = None;
    while !input.is_exhausted() {
        if let Ok(value) = input.try_parse(parse_underline_vertical) {
            if vertical.replace(value).is_some() {
                return Err(unsupported_value(
                    input,
                    None,
                    "duplicate underline vertical",
                ));
            }
        } else {
            let value = parse_text_side(input)?;
            if side.replace(value).is_some() {
                return Err(unsupported_value(input, None, "duplicate underline side"));
            }
        }
    }
    CssUnderlinePosition::try_new(vertical, side)
        .map(CssTextUnderlinePosition::Position)
        .ok_or_else(|| unsupported_value(input, None, "empty underline position"))
}
pub(super) fn parse_text_underline_offset<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssTextUnderlineOffset, ParseError<'i, Error>> {
    if input
        .try_parse(|input| input.expect_ident_matching("auto"))
        .is_ok()
    {
        return Ok(CssTextUnderlineOffset::Auto);
    }
    parse_length_percentage(input, numeric, "text-underline-offset").map(|value| {
        CssTextUnderlineOffset::Length(CssTextUnderlineOffsetLength::from_parsed(value))
    })
}
pub(super) fn parse_text_decoration_skip_spaces<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssTextDecorationSkipSpaces, ParseError<'i, Error>> {
    let ident = input.expect_ident_cloned().map_err(basic)?;
    if ident.eq_ignore_ascii_case("none") {
        return Ok(CssTextDecorationSkipSpaces::None);
    }
    if ident.eq_ignore_ascii_case("all") {
        return Ok(CssTextDecorationSkipSpaces::All);
    }
    let start = ident.eq_ignore_ascii_case("start");
    let end = ident.eq_ignore_ascii_case("end");
    if !start && !end {
        return Err(unsupported_value(input, None, "invalid skip spaces"));
    }
    if input.is_exhausted() {
        return Ok(if start {
            CssTextDecorationSkipSpaces::Start
        } else {
            CssTextDecorationSkipSpaces::End
        });
    }
    input
        .expect_ident_matching(if start { "end" } else { "start" })
        .map_err(basic)?;
    Ok(CssTextDecorationSkipSpaces::StartEnd)
}
fn parse_emphasis_mark<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssTextEmphasisMark, ParseError<'i, Error>> {
    input.skip_whitespace();
    let first_origin = numeric.origin_at(input.position().byte_index());
    let mut fill = input.try_parse(parse_emphasis_fill).ok();
    let mut fill_origin = fill.and(first_origin);
    input.skip_whitespace();
    let next_origin = numeric.origin_at(input.position().byte_index());
    let shape = input.try_parse(parse_emphasis_shape).ok();
    let shape_origin = shape.and(next_origin);
    if fill.is_none() {
        input.skip_whitespace();
        let origin = numeric.origin_at(input.position().byte_index());
        fill = input.try_parse(parse_emphasis_fill).ok();
        fill_origin = fill.and(origin);
    }
    CssTextEmphasisMark::from_parsed(fill, shape, fill_origin, shape_origin)
        .ok_or_else(|| unsupported_value(input, None, "empty emphasis mark"))
}
pub(super) fn parse_text_emphasis_style<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssTextEmphasisStyle, ParseError<'i, Error>> {
    if input
        .try_parse(|input| input.expect_ident_matching("none"))
        .is_ok()
    {
        return Ok(CssTextEmphasisStyle::None);
    }
    if let Ok(mark) = input.try_parse(|input| parse_emphasis_mark(input, numeric)) {
        return Ok(CssTextEmphasisStyle::Mark(mark));
    }
    input.skip_whitespace();
    let location = input.current_source_location();
    let component = numeric
        .collect(input)
        .map_err(|_| unsupported_value_at(location, None, "expected emphasis string"))?;
    CssTextEmphasisString::from_parsed(component)
        .map(CssTextEmphasisStyle::String)
        .ok_or_else(|| unsupported_value_at(location, None, "invalid emphasis style"))
}
pub(super) fn parse_text_emphasis<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssTextEmphasis, ParseError<'i, Error>> {
    let mut style = None;
    let mut color = None;
    while !input.is_exhausted() {
        if style.is_none()
            && let Ok(value) = input.try_parse(|input| parse_text_emphasis_style(input, numeric))
        {
            style = Some(value);
            continue;
        }
        if color.is_none()
            && let Ok(value) = input.try_parse(|input| parse_color(input, numeric))
        {
            color = Some(value);
            continue;
        }
        return Err(unsupported_value(
            input,
            None,
            "invalid emphasis shorthand component",
        ));
    }
    if style.is_none() && color.is_none() {
        return Err(unsupported_value(input, None, "empty emphasis"));
    }
    Ok(CssTextEmphasis::new(style, color))
}
pub(super) fn parse_text_emphasis_position<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssTextEmphasisPosition, ParseError<'i, Error>> {
    input.skip_whitespace();
    let first_origin = numeric.origin_at(input.position().byte_index());
    let side_first = input.try_parse(parse_text_side).ok();
    let mut side_origin = side_first.and(first_origin);
    input.skip_whitespace();
    let vertical_origin = numeric.origin_at(input.position().byte_index());
    let vertical = parse_emphasis_vertical(input)?;
    let side = if side_first.is_some() {
        side_first
    } else {
        input.skip_whitespace();
        let origin = numeric.origin_at(input.position().byte_index());
        let side = input.try_parse(parse_text_side).ok();
        side_origin = side.and(origin);
        side
    };
    Ok(CssTextEmphasisPosition::from_parsed(
        vertical,
        side,
        vertical_origin.unwrap_or(CssValueOrigin::Programmatic),
        side_origin,
    ))
}
pub(super) fn parse_text_emphasis_skip<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssTextEmphasisSkip, ParseError<'i, Error>> {
    let mut flags = [false; 4];
    while !input.is_exhausted() {
        let ident = input.expect_ident_cloned().map_err(basic)?;
        let index = match_ignore_ascii_case! { &ident, "spaces" => 0, "punctuation" => 1, "symbols" => 2, "narrow" => 3, _ => return Err(unsupported_value(input, None, "invalid emphasis skip")) };
        if std::mem::replace(&mut flags[index], true) {
            return Err(unsupported_value(input, None, "duplicate emphasis skip"));
        }
    }
    CssTextEmphasisSkip::try_new(flags[0], flags[1], flags[2], flags[3])
        .ok_or_else(|| unsupported_value(input, None, "empty emphasis skip"))
}
pub(super) fn parse_text_shadow<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssTextShadow, ParseError<'i, Error>> {
    if input
        .try_parse(|input| input.expect_ident_matching("none"))
        .is_ok()
    {
        return Ok(CssTextShadow::None);
    }
    let mut shadows = Vec::new();
    loop {
        let shadow = super::box_model::parse_shadow(input, numeric)?;
        shadows.push(
            CssTextShadowLayer::from_parsed(shadow)
                .ok_or_else(|| unsupported_value(input, None, "negative text-shadow spread"))?,
        );
        if input.try_parse(Parser::expect_comma).is_err() {
            break;
        }
        if input.is_exhausted() {
            return Err(unsupported_value(input, None, "empty text-shadow layer"));
        }
    }
    CssTextShadowList::from_parsed(shadows)
        .map(CssTextShadow::Shadows)
        .ok_or_else(|| unsupported_value(input, None, "empty text-shadow list"))
}
