use super::color::parse_color;
use super::combinators::{Combinator, Tail, commit, fill, keyword, parse_group};
use super::property_input::Stream;
use crate::diagnostic::CssFeatureId;
use crate::diagnostic::error::property::{Error, SyntaxError, basic, unexpected_at};
use surgeist_css_parser::match_ignore_ascii_case;

use super::values::{parse_shadow_length, parse_shadow_nonnegative_length};
use crate::values::box_model::CssBoxDecorationBreak;
use crate::values::effects::{CssBoxShadow, CssBoxShadowList, CssDropShadow, CssShadow};

pub(super) static IMPLEMENTED_SHARED_VALUES: &[CssFeatureId] =
    &[CssFeatureId::new("official.value.box-edge-keywords")];

pub(crate) fn parse_box_decoration_break<'i>(
    input: &mut Stream<'i>,
) -> std::result::Result<CssBoxDecorationBreak, SyntaxError<'i, Error>> {
    let keyword_location = input.site();
    let ident = input.consume_ident().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "slice" => Ok(CssBoxDecorationBreak::Slice),
        "clone" => Ok(CssBoxDecorationBreak::Clone),
        _ => Err(unexpected_at(keyword_location)),
    }
}

pub(crate) fn parse_box_shadow<'i>(
    input: &mut Stream<'i>,
    numeric: &crate::numeric::expression::NumericInputContext<'_>,
) -> std::result::Result<CssBoxShadow, SyntaxError<'i, Error>> {
    let state = input.mark();
    if let Ok(ident) = input.attempt(Stream::consume_ident)
        && ident.eq_ignore_ascii_case("none")
        && input.only_whitespace_remains()
    {
        return Ok(CssBoxShadow::None);
    }
    input.restore(state);

    let shadows = input.comma_segments(|input| parse_shadow(input, numeric))?;

    Ok(CssBoxShadow::Shadows(
        CssBoxShadowList::try_new(shadows).expect("box-shadow parser records at least one shadow"),
    ))
}

pub(super) fn parse_shadow<'i>(
    input: &mut Stream<'i>,
    numeric: &crate::numeric::expression::NumericInputContext<'_>,
) -> Result<CssShadow, SyntaxError<'i, Error>> {
    // `<color>? && [ <length>{2} <length [0,∞]>? <length>? ] && inset?`: the
    // && grammar reorders whole operands, never the lengths within one.
    let (mut inset, mut color, mut lengths) = (None, None, None);
    parse_group::<3>(
        input,
        Combinator::AllOf {
            optional: [true, true, false],
        },
        Tail::Reject,
        |_| false,
        |operand, input| match operand {
            0 => keyword(&mut inset, input, &[("inset", ())]),
            1 => fill(&mut color, input, |input| parse_color(input, numeric)),
            2 => commit(
                &mut lengths,
                input,
                |_| true,
                |input| {
                    let x = parse_shadow_length(input, numeric)?;
                    let y = parse_shadow_length(input, numeric)?;
                    let blur = input
                        .attempt(|input| parse_shadow_nonnegative_length(input, numeric))
                        .ok();
                    let spread = if blur.is_some() {
                        input
                            .attempt(|input| parse_shadow_length(input, numeric))
                            .ok()
                    } else {
                        None
                    };
                    Ok((x, y, blur, spread))
                },
            ),
            _ => unreachable!("three shadow operands"),
        },
    )?;
    let (x, y, blur, spread) = lengths.expect("required shadow lengths");
    Ok(
        CssShadow::try_new(inset.is_some(), x, y, blur, spread, color)
            .expect("parser requires blur before spread"),
    )
}

pub(super) fn parse_drop_shadow<'i>(
    input: &mut Stream<'i>,
    numeric: &crate::numeric::expression::NumericInputContext<'_>,
) -> Result<CssDropShadow, SyntaxError<'i, Error>> {
    // `<color>? && [ <length>{2} <length [0,∞]>? ]`.
    let (mut color, mut lengths) = (None, None);
    parse_group::<2>(
        input,
        Combinator::AllOf {
            optional: [true, false],
        },
        Tail::Reject,
        |_| false,
        |operand, input| match operand {
            0 => fill(&mut color, input, |input| parse_color(input, numeric)),
            1 => commit(
                &mut lengths,
                input,
                |_| true,
                |input| {
                    let x = parse_shadow_length(input, numeric)?;
                    let y = parse_shadow_length(input, numeric)?;
                    let standard_deviation = input
                        .attempt(|input| parse_shadow_nonnegative_length(input, numeric))
                        .ok();
                    Ok((x, y, standard_deviation))
                },
            ),
            _ => unreachable!("two drop-shadow operands"),
        },
    )?;
    let (x, y, standard_deviation) = lengths.expect("required drop-shadow lengths");
    Ok(CssDropShadow::new(x, y, standard_deviation, color))
}
