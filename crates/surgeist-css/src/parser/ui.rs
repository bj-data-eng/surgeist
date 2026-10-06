use crate::error::{Error, basic, unsupported_value_at};
use crate::numeric::NumericInputContext;
use crate::ui::*;
use crate::{CssCaretColor, CssComponentValueRef, CssValueTokenRef};
use cssparser::{ParseError, Parser, match_ignore_ascii_case};

macro_rules! keyword_parser {
    ($fn:ident, $ty:ident, {$($text:literal => $variant:ident),+}) => {
        pub(super) fn $fn<'i, 't>(input: &mut Parser<'i, 't>) -> Result<$ty, ParseError<'i, Error>> {
            let location = input.current_source_location();
            let ident = input.expect_ident_cloned().map_err(basic)?;
            match_ignore_ascii_case! { &ident, $($text => Ok($ty::$variant),)+
                _ => Err(unsupported_value_at(location, None, concat!("unsupported ", stringify!($ty), " keyword"))), }
        }
    };
}
keyword_parser!(parse_caret_animation, CssCaretAnimation, { "auto" => Auto, "manual" => Manual });
keyword_parser!(parse_caret_shape, CssCaretShape, { "auto" => Auto, "bar" => Bar, "block" => Block, "underscore" => Underscore });
keyword_parser!(parse_interactivity, CssInteractivity, { "auto" => Auto, "inert" => Inert });
keyword_parser!(parse_appearance, CssAppearance, { "none" => None, "auto" => Auto, "base" => Base, "base-select" => BaseSelect,
    "searchfield" => Searchfield, "textarea" => Textarea, "checkbox" => Checkbox, "radio" => Radio,
    "menulist" => Menulist, "listbox" => Listbox, "meter" => Meter, "progress-bar" => ProgressBar,
    "button" => Button, "textfield" => Textfield, "menulist-button" => MenulistButton });
pub(super) fn parse_accent_color<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssAccentColor, ParseError<'i, Error>> {
    if input
        .try_parse(|input| input.expect_ident_matching("auto"))
        .is_ok()
    {
        return Ok(CssAccentColor::Auto);
    }
    super::color::parse_color(input, numeric).map(|color| CssAccentColor::Color(Box::new(color)))
}
pub(super) fn parse_caret<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssCaret, ParseError<'i, Error>> {
    let mut color = None;
    let mut animation = None;
    let mut shape = None;
    let mut autos = 0;
    while !input.is_exhausted() {
        let location = input.current_source_location();
        if input
            .try_parse(|input| input.expect_ident_matching("auto"))
            .is_ok()
        {
            autos += 1;
            continue;
        }
        if animation.is_none()
            && let Ok(value) = input.try_parse(parse_caret_animation)
        {
            animation = Some(value);
            continue;
        }
        if shape.is_none()
            && let Ok(value) = input.try_parse(parse_caret_shape)
        {
            shape = Some(value);
            continue;
        }
        if color.is_none() {
            match input.try_parse(|input| super::color::parse_color(input, numeric)) {
                Ok(value) => {
                    color = Some(CssCaretColor::Color(Box::new(value)));
                    continue;
                }
                Err(error) if crate::error::is_resource_parse_error(&error) => return Err(error),
                Err(_) => {}
            }
        }
        return Err(unsupported_value_at(
            location,
            None,
            "duplicate or unsupported caret role",
        ));
    }
    // Ambiguous auto fills vacant roles in canonical order after explicit roles.
    for _ in 0..autos {
        if color.is_none() {
            color = Some(CssCaretColor::Auto);
        } else if animation.is_none() {
            animation = Some(CssCaretAnimation::Auto);
        } else if shape.is_none() {
            shape = Some(CssCaretShape::Auto);
        } else {
            return Err(unsupported_value_at(
                input.current_source_location(),
                None,
                "too many caret components",
            ));
        }
    }
    CssCaret::try_new(color, animation, shape).ok_or_else(|| {
        unsupported_value_at(
            input.current_source_location(),
            None,
            "caret requires a component",
        )
    })
}
pub(super) fn parse_interest_delay_value<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssInterestDelayValue, ParseError<'i, Error>> {
    if input
        .try_parse(|input| input.expect_ident_matching("normal"))
        .is_ok()
    {
        return Ok(CssInterestDelayValue::Normal);
    }
    super::timing::parse_delay(input, numeric).map(CssInterestDelayValue::Time)
}
pub(super) fn parse_interest_delay<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssInterestDelay, ParseError<'i, Error>> {
    let start = parse_interest_delay_value(input, numeric)?;
    let end = if input.is_exhausted() {
        None
    } else {
        Some(parse_interest_delay_value(input, numeric)?)
    };
    Ok(CssInterestDelay::from_parser(start, end))
}
pub(super) fn parse_navigation<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssNavigation, ParseError<'i, Error>> {
    if input
        .try_parse(|input| input.expect_ident_matching("auto"))
        .is_ok()
    {
        return Ok(CssNavigation::Auto);
    }
    input.skip_whitespace();
    let location = input.current_source_location();
    let component = numeric
        .collect(input)
        .map_err(|error| numeric_error(numeric, &error, location))?;
    let id = CssNavigationId::try_from_component(component)
        .map_err(|_| unsupported_value_at(location, None, "navigation requires an ID selector"))?;
    let target = if input.is_exhausted() {
        None
    } else {
        input.skip_whitespace();
        let location = input.current_source_location();
        let component = numeric
            .collect(input)
            .map_err(|error| numeric_error(numeric, &error, location))?;
        Some(match component.view() {
            CssComponentValueRef::Token(CssValueTokenRef::Ident(value))
                if value.eq_ignore_ascii_case("current") =>
            {
                CssNavigationTarget::Current
            }
            CssComponentValueRef::Token(CssValueTokenRef::Ident(value))
                if value.eq_ignore_ascii_case("root") =>
            {
                CssNavigationTarget::Root
            }
            CssComponentValueRef::Token(CssValueTokenRef::String(value))
                if value.starts_with('_') =>
            {
                if matches!(numeric, NumericInputContext::Components(..)) {
                    return Err(unsupported_value_at(
                        location,
                        None,
                        "navigation target must not begin with underscore",
                    ));
                }
                CssNavigationTarget::LegacyName(CssLegacyNavigationTargetName::from_parser(
                    component,
                ))
            }
            CssComponentValueRef::Token(CssValueTokenRef::String(_)) => {
                CssNavigationTarget::Name(CssNavigationTargetName::from_parser(component))
            }
            _ => {
                return Err(unsupported_value_at(
                    location,
                    None,
                    "invalid navigation target",
                ));
            }
        })
    };
    Ok(CssNavigation::Id(Box::new(
        CssNavigationReference::from_parser(id, target),
    )))
}
fn numeric_error<'i>(
    numeric: &NumericInputContext<'_>,
    error: &crate::CssNumericConstructionError,
    location: cssparser::SourceLocation,
) -> ParseError<'i, Error> {
    if let Some(component) = error.component_error()
        && crate::error::is_component_resource_error(component)
    {
        return crate::error::invalid_component_value(location, component.clone());
    }
    let offset = match error.origin() {
        Some(crate::CssValueOrigin::Parsed(value)) => value.span().start().byte_offset().value(),
        _ => 0,
    };
    unsupported_value_at(
        numeric.error_location(error, location, offset),
        None,
        "invalid navigation component",
    )
}
