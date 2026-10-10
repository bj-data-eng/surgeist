//! Selected Color4 Appendix B fallback, confined to direct property color slots.
use super::color::parse_color;
use crate::error::{Error, ErrorKind, unexpected_at};
use crate::numeric::NumericInputContext;
use crate::*;
use cssparser::{ParseError, Parser};

pub(super) static IMPLEMENTED_SHARED_VALUES: &[CssFeatureId] =
    &[CssFeatureId::new("official.value.quirky-color")];

pub(super) fn parse_property_value<'i, 't>(
    property: CssKnownProperty,
    authored: CssAuthoredDeclarationValue,
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
    context: CssParserContext,
) -> Result<CssKnownDeclaration, ParseError<'i, Error>> {
    if context.mode() != CssParserMode::Quirks {
        return super::parse_known_property_value(property, authored, input, numeric);
    }
    macro_rules! single {
        ($variant:ident, $wrapper:ident) => {{
            let value = direct_color(input, numeric)?;
            Ok(CssKnownDeclaration::from_value(
                CssKnownDeclarationValue::$variant(CssDeclaredValue::Value($wrapper::new(
                    authored, value,
                ))),
            ))
        }};
    }
    match property {
        CssKnownProperty::Color => single!(Color, CssColorPropertyValue),
        CssKnownProperty::BackgroundColor => {
            single!(BackgroundColor, CssBackgroundColorPropertyValue)
        }
        CssKnownProperty::BorderTopColor => single!(BorderTopColor, CssBorderTopColorPropertyValue),
        CssKnownProperty::BorderRightColor => {
            single!(BorderRightColor, CssBorderRightColorPropertyValue)
        }
        CssKnownProperty::BorderBottomColor => {
            single!(BorderBottomColor, CssBorderBottomColorPropertyValue)
        }
        CssKnownProperty::BorderLeftColor => {
            single!(BorderLeftColor, CssBorderLeftColorPropertyValue)
        }
        CssKnownProperty::BorderColor => {
            // Preserve ordinary logical-prefixed grammar; its targets do not grant a quirk.
            let state = input.state();
            let logical = input
                .try_parse(|input| input.expect_ident_matching("logical"))
                .is_ok();
            input.reset(&state);
            if logical {
                return super::parse_known_property_value(property, authored, input, numeric);
            }
            let mut colors = Vec::new();
            while !input.is_exhausted() {
                if colors.len() == 4 {
                    return Err(unexpected_at(input.current_source_location()));
                }
                colors.push(direct_color(input, numeric)?);
            }
            let value = CssBorderColorShorthand::try_new(CssBoxSideKind::Physical, colors)
                .ok_or_else(|| unexpected_at(input.current_source_location()))?;
            Ok(CssKnownDeclaration::from_value(
                CssKnownDeclarationValue::BorderColor(CssDeclaredValue::Value(
                    CssBorderColorPropertyValue::new(authored, value),
                )),
            ))
        }
        _ => super::parse_known_property_value(property, authored, input, numeric),
    }
}

fn direct_color<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssColor, ParseError<'i, Error>> {
    // Ordinary grammar wins, including escaped named colors and complete functions.
    // This fallback never enters a function or a referencing shorthand.
    let ordinary = match input.try_parse(|input| parse_color(input, numeric)) {
        Ok(color) => return Ok(color),
        Err(error) => error,
    };
    if let cssparser::ParseErrorKind::Custom(error) = &ordinary.kind {
        match error.kind() {
            ErrorKind::InvalidComponentValue(component)
                if crate::error::is_component_resource_error(component) =>
            {
                return Err(ordinary);
            }
            ErrorKind::NestingLimit(_) => return Err(ordinary),
            _ => {}
        }
    }
    input.skip_whitespace();
    let location = input.current_source_location();
    let offset = input.position().byte_index();
    let component = numeric.collect(input).map_err(|error| {
        let at = numeric.error_location(&error, location, offset);
        error.component_error().map_or_else(
            || unexpected_at(input.current_source_location()),
            |component| crate::error::invalid_component_value(at, component.clone()),
        )
    })?;
    quirky_hex(&component)
        .map(CssColor::from_hex)
        .ok_or(ordinary)
}

fn quirky_hex(component: &CssComponentValue) -> Option<CssHexColor> {
    let raw = component.token_representation()?;
    let digits = match component.view() {
        CssComponentValueRef::Token(CssValueTokenRef::Ident(_)) => {
            if !matches!(raw.len(), 3 | 6) {
                return None;
            }
            return CssHexColor::try_new(raw);
        }
        CssComponentValueRef::Token(CssValueTokenRef::Number(number)) => {
            integer_digits(number, "")?
        }
        CssComponentValueRef::Token(CssValueTokenRef::Dimension { number, .. }) => {
            // The unit's representation is the actual suffix, not its decoded identity.
            integer_digits(number, raw.get(number.representation().len()..)?)?
        }
        _ => return None,
    };
    CssHexColor::try_new(&digits)
}

fn integer_digits(number: CssNumericTokenRef<'_>, unit: &str) -> Option<String> {
    if number.kind() != CssNumericTokenKind::Integer {
        return None;
    }
    // Reuse the exact lexical integer owner: no tokenizer float or new numeric engine.
    // Values outside i32 cannot produce a six-character hexadecimal serialization.
    let value = crate::integer_value::exact_i32(number.representation())?;
    let integer = value.to_string();
    let length = integer.len().checked_add(unit.len())?;
    if length > 6 {
        return None;
    }
    let mut digits = String::with_capacity(6);
    for _ in length..6 {
        digits.push('0');
    }
    digits.push_str(&integer);
    digits.push_str(unit);
    Some(digits)
}
