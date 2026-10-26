//! Selected Decoration 4 intrinsic grammars.
use super::color::parse_color;
use super::combinators::{Combinator, Tail, fill, keyword, parse_group};
use super::property_input::Parser;
use super::values::parse_length_percentage;
use crate::error::property::{Error, ParseError, basic, unexpected_at};
use crate::*;
use cssparser::match_ignore_ascii_case;

macro_rules! keyword {
    ($parser:ident, $ty:ident, $($text:literal => $variant:ident),+ $(,)?) => {
        pub(super) fn $parser<'i, 't>(input: &mut Parser<'i, 't>) -> std::result::Result<$ty, ParseError<'i, Error>> {
            let keyword_location = input.current_source_location();
            let ident = input.expect_ident_cloned().map_err(basic)?;
            match_ignore_ascii_case! { &ident, $($text => Ok($ty::$variant),)+ _ => Err(unexpected_at(keyword_location)) }
        }
    };
}
keyword!(parse_decoration_error, CssTextDecorationErrorLine, "spelling-error" => SpellingError, "grammar-error" => GrammarError);
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
    // `[ from-font | under ] || [ left | right ]`.
    let (mut vertical, mut side) = (None, None);
    parse_group::<2>(
        input,
        Combinator::AnyOf,
        Tail::Reject,
        |_| false,
        |operand, input| match operand {
            0 => fill(&mut vertical, input, parse_underline_vertical),
            1 => fill(&mut side, input, parse_text_side),
            _ => unreachable!("two underline-position operands"),
        },
    )?;
    CssUnderlinePosition::try_new(vertical, side)
        .map(CssTextUnderlinePosition::Position)
        .ok_or_else(|| unexpected_at(input.current_source_location()))
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
    parse_length_percentage(input, numeric).map(|value| {
        CssTextUnderlineOffset::Length(CssTextUnderlineOffsetLength::from_parsed(value))
    })
}
pub(super) fn parse_text_decoration_skip_spaces<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssTextDecorationSkipSpaces, ParseError<'i, Error>> {
    let keyword_location = input.current_source_location();
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
        return Err(unexpected_at(keyword_location));
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
) -> std::result::Result<CssTextEmphasisMark, ParseError<'i, Error>> {
    input.skip_whitespace();
    let first_origin = input.position().value_origin();
    let mut fill = input.try_parse(parse_emphasis_fill).ok();
    let mut fill_origin = fill.and(first_origin);
    input.skip_whitespace();
    let next_origin = input.position().value_origin();
    let shape = input.try_parse(parse_emphasis_shape).ok();
    let shape_origin = shape.and(next_origin);
    if fill.is_none() {
        input.skip_whitespace();
        let origin = input.position().value_origin();
        fill = input.try_parse(parse_emphasis_fill).ok();
        fill_origin = fill.and(origin);
    }
    CssTextEmphasisMark::from_parsed(fill, shape, fill_origin, shape_origin)
        .ok_or_else(|| unexpected_at(input.current_source_location()))
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
    if let Ok(mark) = input.try_parse(parse_emphasis_mark) {
        return Ok(CssTextEmphasisStyle::Mark(mark));
    }
    input.skip_whitespace();
    let location = input.current_source_location();
    let component = input
        .collect_numeric_component(numeric)
        .map_err(|_| unexpected_at(location))?;
    CssTextEmphasisString::from_parsed(component)
        .map(CssTextEmphasisStyle::String)
        .ok_or_else(|| unexpected_at(location))
}
pub(super) fn parse_text_emphasis<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssTextEmphasis, ParseError<'i, Error>> {
    // `<'text-emphasis-style'> || <'text-emphasis-color'>`.
    let (mut style, mut color) = (None, None);
    parse_group::<2>(
        input,
        Combinator::AnyOf,
        Tail::Reject,
        |_| false,
        |operand, input| match operand {
            0 => fill(&mut style, input, |input| {
                parse_text_emphasis_style(input, numeric)
            }),
            1 => fill(&mut color, input, |input| parse_color(input, numeric)),
            _ => unreachable!("two text-emphasis operands"),
        },
    )?;
    Ok(CssTextEmphasis::new(style, color))
}
pub(super) fn parse_text_emphasis_position<'i, 't>(
    input: &mut Parser<'i, 't>,
    _numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssTextEmphasisPosition, ParseError<'i, Error>> {
    input.skip_whitespace();
    let first_origin = input.position().value_origin();
    let side_first = input.try_parse(parse_text_side).ok();
    let mut side_origin = side_first.and(first_origin);
    input.skip_whitespace();
    let vertical_origin = input.position().value_origin();
    let vertical = parse_emphasis_vertical(input)?;
    let side = if side_first.is_some() {
        side_first
    } else {
        input.skip_whitespace();
        let origin = input.position().value_origin();
        let side = input.try_parse(parse_text_side).ok();
        side_origin = side.and(origin);
        side
    };
    Ok(CssTextEmphasisPosition::from_parsed(
        vertical,
        side,
        vertical_origin.expect("emphasis vertical token origin"),
        side_origin,
    ))
}
pub(super) fn parse_text_emphasis_skip<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssTextEmphasisSkip, ParseError<'i, Error>> {
    // `spaces || punctuation || symbols || narrow`.
    const KEYWORDS: [&str; 4] = ["spaces", "punctuation", "symbols", "narrow"];
    let mut slots = [None; 4];
    let flags = parse_group::<4>(
        input,
        Combinator::AnyOf,
        Tail::Reject,
        |_| false,
        |operand, input| keyword(&mut slots[operand], input, &[(KEYWORDS[operand], ())]),
    )?;
    CssTextEmphasisSkip::try_new(flags[0], flags[1], flags[2], flags[3])
        .ok_or_else(|| unexpected_at(input.current_source_location()))
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
    let shadows = input.parse_comma_separated(|input| {
        let shadow = super::box_model::parse_shadow(input, numeric)?;
        CssTextShadowLayer::from_parsed(shadow)
            .ok_or_else(|| unexpected_at(input.current_source_location()))
    })?;
    CssTextShadowList::from_parsed(shadows)
        .map(CssTextShadow::Shadows)
        .ok_or_else(|| unexpected_at(input.current_source_location()))
}
