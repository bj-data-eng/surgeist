//! Selected Color4 Appendix B fallback, confined to direct property color slots.
use super::color::parse_color;
use super::side_shapes::parse_four_sides;
use super::syntax_types::Stream;
use crate::color::{CssColor, CssHexColor};
use crate::component::{
    CssComponentValue, CssComponentValueRef, CssNumericTokenKind, CssNumericTokenRef,
    CssValueTokenRef,
};
use crate::declaration::CssAuthoredDeclarationValue;
use crate::declaration::syntax::CssDeclaredValue;
use crate::diagnostic::CssFeatureId;
use crate::diagnostic::error::ErrorKind;
use crate::diagnostic::error::property::{Error, unexpected_at};
use crate::numeric::expression::NumericInputContext;
use crate::parser::property_input::StreamExt;
use crate::parser::syntax_types::SyntaxError;
use crate::property::properties::CssKnownDeclarationValue;
use crate::property::{CssKnownDeclaration, CssKnownProperty, CssPropertyValue};
use crate::{CssParserContext, CssParserMode};

pub(super) static IMPLEMENTED_SHARED_VALUES: &[CssFeatureId] =
    &[CssFeatureId::new("official.value.quirky-color")];

pub(super) fn parse_property_value<'i>(
    property: CssKnownProperty,
    authored: CssAuthoredDeclarationValue,
    input: &mut Stream<'i>,
    numeric: &NumericInputContext<'_>,
    context: CssParserContext,
) -> Result<CssKnownDeclaration, SyntaxError<'i, Error>> {
    if context.mode() != CssParserMode::Quirks {
        return super::parse_known_property_value(property, authored, input, numeric);
    }
    macro_rules! single {
        ($variant:ident) => {{
            let value = direct_color(input, numeric)?;
            Ok(CssKnownDeclaration::from_value(
                CssKnownDeclarationValue::$variant(CssDeclaredValue::Value(CssPropertyValue::new(
                    authored, value,
                ))),
            ))
        }};
    }
    match property {
        CssKnownProperty::Color => single!(Color),
        CssKnownProperty::BackgroundColor => {
            single!(BackgroundColor)
        }
        CssKnownProperty::BorderTopColor => single!(BorderTopColor),
        CssKnownProperty::BorderRightColor => {
            single!(BorderRightColor)
        }
        CssKnownProperty::BorderBottomColor => {
            single!(BorderBottomColor)
        }
        CssKnownProperty::BorderLeftColor => {
            single!(BorderLeftColor)
        }
        CssKnownProperty::BorderColor => {
            // Preserve ordinary logical-prefixed grammar; its targets do not grant a quirk.
            let state = input.mark();
            let logical = input
                .attempt(|input| input.consume_keyword("logical"))
                .is_ok();
            input.restore(state);
            if logical {
                return super::parse_known_property_value(property, authored, input, numeric);
            }
            let value = parse_four_sides(input, |input| direct_color(input, numeric))?;
            Ok(CssKnownDeclaration::from_value(
                CssKnownDeclarationValue::BorderColor(CssDeclaredValue::Value(
                    CssPropertyValue::new(authored, value),
                )),
            ))
        }
        _ => super::parse_known_property_value(property, authored, input, numeric),
    }
}

fn direct_color<'i>(
    input: &mut Stream<'i>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssColor, SyntaxError<'i, Error>> {
    // Ordinary grammar wins, including escaped named colors and complete functions.
    // This fallback never enters a function or a referencing shorthand.
    let ordinary = match input.attempt(|input| parse_color(input, numeric)) {
        Ok(color) => return Ok(color),
        Err(error) => error,
    };
    if let Some(error) = ordinary.custom_value() {
        match &error.kind {
            ErrorKind::InvalidComponentValue(component)
                if crate::diagnostic::error::is_component_resource_error(component) =>
            {
                return Err(ordinary);
            }
            ErrorKind::NestingLimit(_) => return Err(ordinary),
            _ => {}
        }
    }
    input.discard_whitespace();
    let location = input.site();
    let offset = input.site();
    let component = input
        .collect_component()
        .map_err(crate::numeric::CssNumericConstructionError::component)
        .map_err(|error| {
            let at = numeric.property_error_location(&error, location, offset);
            error.component_error().map_or_else(
                || unexpected_at(input.site()),
                |component| {
                    crate::diagnostic::error::property::invalid_component_value(
                        at,
                        component.clone(),
                    )
                },
            )
        })?;
    quirky_hex(component.as_ref())
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
    let value = crate::numeric::integer_value::exact_i32(number.representation())?;
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
