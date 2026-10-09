use cssparser::{ParseError, ParseErrorKind, Parser, Token, match_ignore_ascii_case};

use super::color::{
    next_is_mix_weight, parse_authored_mix_weight, parse_color_interpolation_method,
};
use crate::error::{Error, basic};
use crate::numeric::NumericInputContext;
use crate::{
    CssFontPalette, CssFontPaletteMix, CssFontPaletteMixComponent,
    CssFontPaletteMixConstructionError, CssFontPaletteName,
};

pub(super) fn parse_font_palette<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssFontPalette, ParseError<'i, Error>> {
    let location = input.current_source_location();
    match input.next().map_err(basic)?.clone() {
        Token::Ident(name) => match_ignore_ascii_case! { &name,
            "normal" => Ok(CssFontPalette::Normal),
            "light" => Ok(CssFontPalette::Light),
            "dark" => Ok(CssFontPalette::Dark),
            _ => CssFontPaletteName::try_new(&name).map(CssFontPalette::Named)
                .map_err(|_| location.new_unexpected_token_error::<Error>(Token::Ident(name))),
        },
        Token::Function(name) if name.eq_ignore_ascii_case("palette-mix") => input
            .parse_nested_block(|input| parse_mix(input, numeric))
            .map(|value| CssFontPalette::Mix(Box::new(value))),
        token => Err(location.new_unexpected_token_error::<Error>(token)),
    }
}

fn parse_mix<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssFontPaletteMix, ParseError<'i, Error>> {
    let location = input.current_source_location();
    input.skip_whitespace();
    let start = input.position().byte_index();
    let interpolation = if input
        .try_parse(|input| input.expect_ident_matching("in"))
        .is_ok()
    {
        let value = parse_color_interpolation_method(input).map_err(palette_mix_error)?;
        input.expect_comma().map_err(basic)?;
        Some(value)
    } else {
        None
    };
    let mut components = vec![parse_component(input, numeric)?];
    while !input.is_exhausted() {
        input.expect_comma().map_err(basic)?;
        components.push(parse_component(input, numeric)?);
    }
    CssFontPaletteMix::try_new(interpolation, components).map_err(|error| {
        let kind = match error {
            CssFontPaletteMixConstructionError::NestingLimit => {
                crate::CssComponentValueErrorKind::NestingLimit
            }
            CssFontPaletteMixConstructionError::CapacityOverflow => {
                crate::CssComponentValueErrorKind::CapacityOverflow
            }
            CssFontPaletteMixConstructionError::EmptyComponents => {
                unreachable!("parser admits one component before construction")
            }
        };
        crate::error::invalid_component_value(
            location,
            crate::CssComponentValueError::new(
                kind,
                numeric
                    .origin_at(start)
                    .expect("palette argument token origin"),
            ),
        )
    })
}

fn parse_component<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssFontPaletteMixComponent, ParseError<'i, Error>> {
    let leading = if next_is_mix_weight(input) {
        Some(parse_authored_mix_weight(input, numeric).map_err(palette_mix_error)?)
    } else {
        None
    };
    let palette = parse_font_palette(input, numeric)?;
    let weight = if leading.is_none() && next_is_mix_weight(input) {
        Some(parse_authored_mix_weight(input, numeric).map_err(palette_mix_error)?)
    } else {
        leading
    };
    Ok(CssFontPaletteMixComponent::new(palette, weight))
}

// Shared interpolation and percentage admission have color-specific diagnostics.
// Keep their source location while allowing the owning property context to apply.
fn palette_mix_error<'i>(error: ParseError<'i, Error>) -> ParseError<'i, Error> {
    if matches!(&error.kind, ParseErrorKind::Custom(detail)
        if matches!(detail.kind(), crate::ErrorKind::InvalidColorSyntax(_)))
    {
        crate::error::unsupported_value_at(error.location, None, "invalid palette-mix argument")
    } else {
        error
    }
}
