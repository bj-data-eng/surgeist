//! Selected Decoration 4 intrinsic grammars.
use super::color::parse_color;
use super::combinators::{Combinator, Tail, fill, keyword, parse_group};
use super::property_input::Stream;
use super::values::parse_length_percentage;
use crate::diagnostic::error::property::{Error, SyntaxError, basic, unexpected_at};
use crate::values::text::{
    CssTextDecorationErrorLine, CssTextDecorationSkip, CssTextDecorationSkipBox,
    CssTextDecorationSkipInk, CssTextDecorationSkipInset, CssTextDecorationSkipSelf,
    CssTextDecorationSkipSpaces, CssTextEmphasis, CssTextEmphasisFill, CssTextEmphasisMark,
    CssTextEmphasisPosition, CssTextEmphasisShape, CssTextEmphasisSkip, CssTextEmphasisString,
    CssTextEmphasisStyle, CssTextEmphasisVertical, CssTextShadow, CssTextShadowLayer,
    CssTextShadowList, CssTextSide, CssTextUnderlineOffset, CssTextUnderlineOffsetLength,
    CssTextUnderlinePosition, CssUnderlinePosition, CssUnderlinePositionVertical,
};
use surgeist_css_parser::match_ignore_ascii_case;

macro_rules! keyword {
    ($parser:ident, $ty:ident, $($text:literal => $variant:ident),+ $(,)?) => {
        pub(super) fn $parser<'i>(input: &mut Stream<'i>) -> std::result::Result<$ty, SyntaxError<'i, Error>> {
            let keyword_location = input.site();
            let ident = input.consume_ident().map_err(basic)?;
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
pub(crate) fn parse_text_underline_position<'i>(
    input: &mut Stream<'i>,
) -> std::result::Result<CssTextUnderlinePosition, SyntaxError<'i, Error>> {
    if input.attempt(|input| input.consume_keyword("auto")).is_ok() {
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
        .ok_or_else(|| unexpected_at(input.site()))
}
pub(crate) fn parse_text_underline_offset<'i>(
    input: &mut Stream<'i>,
    numeric: &crate::numeric::expression::NumericInputContext<'_>,
) -> std::result::Result<CssTextUnderlineOffset, SyntaxError<'i, Error>> {
    if input.attempt(|input| input.consume_keyword("auto")).is_ok() {
        return Ok(CssTextUnderlineOffset::Auto);
    }
    parse_length_percentage(input, numeric).map(|value| {
        CssTextUnderlineOffset::Length(CssTextUnderlineOffsetLength::from_parsed(value))
    })
}
pub(crate) fn parse_text_decoration_skip_spaces<'i>(
    input: &mut Stream<'i>,
) -> std::result::Result<CssTextDecorationSkipSpaces, SyntaxError<'i, Error>> {
    let keyword_location = input.site();
    let ident = input.consume_ident().map_err(basic)?;
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
    if input.only_whitespace_remains() {
        return Ok(if start {
            CssTextDecorationSkipSpaces::Start
        } else {
            CssTextDecorationSkipSpaces::End
        });
    }
    input
        .consume_keyword(if start { "end" } else { "start" })
        .map_err(basic)?;
    Ok(CssTextDecorationSkipSpaces::StartEnd)
}
fn parse_emphasis_mark<'i>(
    input: &mut Stream<'i>,
) -> std::result::Result<CssTextEmphasisMark, SyntaxError<'i, Error>> {
    input.discard_whitespace();
    let first_origin = input.site().value_origin();
    let mut fill = input.attempt(parse_emphasis_fill).ok();
    let mut fill_origin = fill.and(first_origin);
    input.discard_whitespace();
    let next_origin = input.site().value_origin();
    let shape = input.attempt(parse_emphasis_shape).ok();
    let shape_origin = shape.and(next_origin);
    if fill.is_none() {
        input.discard_whitespace();
        let origin = input.site().value_origin();
        fill = input.attempt(parse_emphasis_fill).ok();
        fill_origin = fill.and(origin);
    }
    CssTextEmphasisMark::from_parsed(fill, shape, fill_origin, shape_origin)
        .ok_or_else(|| unexpected_at(input.site()))
}
pub(crate) fn parse_text_emphasis_style<'i>(
    input: &mut Stream<'i>,
    numeric: &crate::numeric::expression::NumericInputContext<'_>,
) -> std::result::Result<CssTextEmphasisStyle, SyntaxError<'i, Error>> {
    if input.attempt(|input| input.consume_keyword("none")).is_ok() {
        return Ok(CssTextEmphasisStyle::None);
    }
    if let Ok(mark) = input.attempt(parse_emphasis_mark) {
        return Ok(CssTextEmphasisStyle::Mark(mark));
    }
    input.discard_whitespace();
    let location = input.site();
    let component = input
        .collect_numeric_component(numeric)
        .map_err(|_| unexpected_at(location))?;
    CssTextEmphasisString::from_parsed(component)
        .map(CssTextEmphasisStyle::String)
        .ok_or_else(|| unexpected_at(location))
}
pub(crate) fn parse_text_emphasis<'i>(
    input: &mut Stream<'i>,
    numeric: &crate::numeric::expression::NumericInputContext<'_>,
) -> std::result::Result<CssTextEmphasis, SyntaxError<'i, Error>> {
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
pub(crate) fn parse_text_emphasis_position<'i>(
    input: &mut Stream<'i>,
    _numeric: &crate::numeric::expression::NumericInputContext<'_>,
) -> std::result::Result<CssTextEmphasisPosition, SyntaxError<'i, Error>> {
    input.discard_whitespace();
    let first_origin = input.site().value_origin();
    let side_first = input.attempt(parse_text_side).ok();
    let mut side_origin = side_first.and(first_origin);
    input.discard_whitespace();
    let vertical_origin = input.site().value_origin();
    let vertical = parse_emphasis_vertical(input)?;
    let side = if side_first.is_some() {
        side_first
    } else {
        input.discard_whitespace();
        let origin = input.site().value_origin();
        let side = input.attempt(parse_text_side).ok();
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
pub(crate) fn parse_text_emphasis_skip<'i>(
    input: &mut Stream<'i>,
) -> std::result::Result<CssTextEmphasisSkip, SyntaxError<'i, Error>> {
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
        .ok_or_else(|| unexpected_at(input.site()))
}
pub(crate) fn parse_text_shadow<'i>(
    input: &mut Stream<'i>,
    numeric: &crate::numeric::expression::NumericInputContext<'_>,
) -> std::result::Result<CssTextShadow, SyntaxError<'i, Error>> {
    if input.attempt(|input| input.consume_keyword("none")).is_ok() {
        return Ok(CssTextShadow::None);
    }
    let shadows = input.comma_segments(|input| {
        let shadow = super::box_model::parse_shadow(input, numeric)?;
        CssTextShadowLayer::from_parsed(shadow).ok_or_else(|| unexpected_at(input.site()))
    })?;
    CssTextShadowList::from_parsed(shadows)
        .map(CssTextShadow::Shadows)
        .ok_or_else(|| unexpected_at(input.site()))
}
