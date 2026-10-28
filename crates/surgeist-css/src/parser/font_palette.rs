use super::property_input::Stream;
use crate::diagnostic::error::property::SyntaxError;
use cssparser::Token;
use surgeist_css_parser::match_ignore_ascii_case;

use super::color::{
    next_is_mix_weight, parse_authored_mix_weight, parse_color_interpolation_method,
};
use crate::diagnostic::error::property::{Error, basic};
use crate::numeric::expression::NumericInputContext;
use crate::rule::font_palette_values::CssFontPaletteName;
use crate::values::font::{CssFontPalette, CssFontPaletteMix, CssFontPaletteMixComponent};

pub(crate) fn parse_font_palette<'i>(
    input: &mut Stream<'i>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssFontPalette, SyntaxError<'i, Error>> {
    let location = input.site();
    match input.consume_after_whitespace().map_err(basic)?.clone() {
        Token::Ident(name) => match_ignore_ascii_case! { &name,
            "normal" => Ok(CssFontPalette::Normal),
            "light" => Ok(CssFontPalette::Light),
            "dark" => Ok(CssFontPalette::Dark),
            _ => CssFontPaletteName::try_new(&name).map(CssFontPalette::Named)
                .map_err(|_| location.unexpected::<Error>(Token::Ident(name))),
        },
        Token::Function(name) if name.eq_ignore_ascii_case("palette-mix") => input
            .within_block(|input| parse_mix(input, numeric))
            .map(|value| CssFontPalette::Mix(Box::new(value))),
        token => Err(location.unexpected::<Error>(token)),
    }
}

fn parse_mix<'i>(
    input: &mut Stream<'i>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssFontPaletteMix, SyntaxError<'i, Error>> {
    let location = input.site();
    input.discard_whitespace();
    let start = input.site();
    let interpolation = if input.attempt(|input| input.consume_keyword("in")).is_ok() {
        let value = parse_color_interpolation_method(input).map_err(palette_mix_error)?;
        input.consume_comma().map_err(basic)?;
        Some(value)
    } else {
        None
    };
    let mut components = vec![parse_component(input, numeric)?];
    while !input.only_whitespace_remains() {
        input.consume_comma().map_err(basic)?;
        components.push(parse_component(input, numeric)?);
    }
    CssFontPaletteMix::try_new(interpolation, components).map_err(|error| {
        assert!(
            !error.is_empty_components(),
            "parser admits one component before construction"
        );
        let kind = if error.is_nesting_limit() {
            crate::component::values::CssComponentValueErrorKind::NestingLimit
        } else {
            crate::component::values::CssComponentValueErrorKind::CapacityOverflow
        };
        crate::diagnostic::error::property::invalid_component_value(
            location,
            crate::component::CssComponentValueError::new(
                kind,
                start.value_origin().expect("palette argument token origin"),
            ),
        )
    })
}

fn parse_component<'i>(
    input: &mut Stream<'i>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssFontPaletteMixComponent, SyntaxError<'i, Error>> {
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
fn palette_mix_error<'i>(error: SyntaxError<'i, Error>) -> SyntaxError<'i, Error> {
    if error.custom_value().is_some_and(|detail| {
        matches!(
            &detail.kind,
            crate::diagnostic::error::ErrorKind::InvalidColorSyntax(_)
        )
    }) {
        crate::diagnostic::error::property::unexpected_at(error.at())
    } else {
        error
    }
}
