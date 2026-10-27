//! Shared helpers of the specified numeric model: calculation admission,
//! exact literal comparison and literal capture.

use crate::component::{CssComponentValue, CssComponentValueRef, CssValueTokenRef};
use crate::numeric::expression::CssNumericConstructionErrorKind;
use crate::numeric::{CssLengthUnit, CssNumericConstructionError};
use crate::serialization::CssSerializationError;
use crate::serialization::numeric_formatting::format_css_number;
use crate::serialization::specified_serialization::{
    CssSpecifiedValueSerializationErrorKind, SpecifiedSerializationContext,
};

type ConstructionResult<T> = Result<T, CssNumericConstructionError>;
type SerializationResult<T> = Result<T, CssSerializationError>;

/// How a specified numeric domain admits a calculation.
#[derive(Clone, Copy)]
pub(crate) enum CalculationAdmission {
    /// Public checked construction admits only fully closed components.
    Checked,
    /// Parser admission retains implicit closure, reported as a diagnostic.
    Recovering,
}

/// The one calculation rule shared by every specified numeric domain.
///
/// Checked admission rejects the first component recovered by implicit
/// closure. A bare ordinary root then re-enters the domain's literal checks;
/// only a math-function root is retained as a calculation.
pub(crate) fn admit_calculation<C, T>(
    calculation: C,
    components: fn(&C) -> &crate::component::CssComponentValues,
    admission: CalculationAdmission,
    literal: fn(CssComponentValue) -> ConstructionResult<T>,
    retain: fn(C) -> T,
) -> ConstructionResult<T> {
    let values = components(&calculation);
    if matches!(admission, CalculationAdmission::Checked)
        && let Some(origin) = values.first_implicit_origin()
    {
        return Err(CssNumericConstructionError::at_origin(
            CssNumericConstructionErrorKind::RecoveredComponent,
            origin.clone(),
        ));
    }
    let root = significant_root(values)?;
    if matches!(root.view(), CssComponentValueRef::Token(_)) {
        return literal(root.clone());
    }
    if !matches!(root.view(), CssComponentValueRef::Function(_)) {
        return Err(CssNumericConstructionError::at(
            CssNumericConstructionErrorKind::RootDomainMismatch,
            Some(root),
        ));
    }
    Ok(retain(calculation))
}

pub(crate) fn significant_root(
    calculation: &crate::component::CssComponentValues,
) -> ConstructionResult<&CssComponentValue> {
    calculation
        .items()
        .iter()
        .find(|component| {
            !matches!(
                component.view(),
                CssComponentValueRef::Comment(_)
                    | CssComponentValueRef::Token(CssValueTokenRef::Whitespace(_))
            )
        })
        .ok_or_else(|| {
            CssNumericConstructionError::at(CssNumericConstructionErrorKind::EmptyValue, None)
        })
}

/// Compares retained ordinary coefficients and their emitted token kind/unit.
/// Formatting can round unequal coefficients to the same text.
pub(crate) fn ordinary_literal_equal(left: &CssComponentValue, right: &CssComponentValue) -> bool {
    use crate::serialization::exact_decimal::LexicalDecimal;
    match (left.view(), right.view()) {
        (
            CssComponentValueRef::Token(CssValueTokenRef::Number(left)),
            CssComponentValueRef::Token(CssValueTokenRef::Number(right)),
        )
        | (
            CssComponentValueRef::Token(CssValueTokenRef::Percentage(left)),
            CssComponentValueRef::Token(CssValueTokenRef::Percentage(right)),
        ) => LexicalDecimal::new(left.representation())
            .value_eq(&LexicalDecimal::new(right.representation())),
        (
            CssComponentValueRef::Token(CssValueTokenRef::Dimension {
                number: left,
                unit: left_unit,
            }),
            CssComponentValueRef::Token(CssValueTokenRef::Dimension {
                number: right,
                unit: right_unit,
            }),
        ) => {
            same_unit(left_unit, right_unit)
                && LexicalDecimal::new(left.representation())
                    .value_eq(&LexicalDecimal::new(right.representation()))
        }
        _ => false,
    }
}

/// Whether two dimension suffixes denote one unit. The unit registry decides
/// aliases (`x` is `dppx`) and ASCII case; a suffix outside the registry
/// compares case-insensitively with itself only.
fn same_unit(left: &str, right: &str) -> bool {
    use crate::numeric::units::parse;
    match (parse(left), parse(right)) {
        (Some(left), Some(right)) => left == right,
        (None, None) => left.eq_ignore_ascii_case(right),
        _ => false,
    }
}

/// A checked length's exact unitless zero has the implicit Px unit.
/// Other explicit units and values that merely round to zero remain distinct.
pub(crate) fn ordinary_length_literal_equal(
    left: &CssComponentValue,
    right: &CssComponentValue,
) -> bool {
    if ordinary_literal_equal(left, right) {
        return true;
    }
    let unitless_zero = |value: &CssComponentValue| {
        matches!(value.view(),
            CssComponentValueRef::Token(CssValueTokenRef::Number(number))
                if crate::serialization::exact_decimal::LexicalDecimal::new(number.representation()).is_zero())
    };
    let pixel_zero = |value: &CssComponentValue| {
        matches!(value.view(),
            CssComponentValueRef::Token(CssValueTokenRef::Dimension { number, unit })
                if unit.eq_ignore_ascii_case("px")
                    && crate::serialization::exact_decimal::LexicalDecimal::new(number.representation()).is_zero())
    };
    (unitless_zero(left) && pixel_zero(right)) || (unitless_zero(right) && pixel_zero(left))
}

fn visit_literal<'a>(
    component: &'a CssComponentValue,
    context: &mut SpecifiedSerializationContext,
) -> SerializationResult<(crate::component::CssNumericTokenRef<'a>, &'static str)> {
    context.charge_nodes(1)?;
    let parts = match component.view() {
        CssComponentValueRef::Token(CssValueTokenRef::Number(number)) => (number, ""),
        CssComponentValueRef::Token(CssValueTokenRef::Dimension { number, unit }) => (
            number,
            CssLengthUnit::from_css_unit(unit)
                .expect("checked length unit")
                .as_str(),
        ),
        CssComponentValueRef::Token(CssValueTokenRef::Percentage(number)) => (number, "%"),
        _ => unreachable!("checked specified numeric literal"),
    };
    Ok(parts)
}

pub(crate) fn capture_literal(
    component: &CssComponentValue,
    context: &mut SpecifiedSerializationContext,
) -> SerializationResult<String> {
    let (number, suffix) = visit_literal(component, context)?;
    if context.output_suppressed() {
        return Ok(String::new());
    }
    let coefficient_limit = context
        .remaining_bytes()
        .checked_sub(suffix.len())
        .ok_or_else(|| {
            CssSerializationError::new(CssSpecifiedValueSerializationErrorKind::ByteLimit)
        })?;
    let mut output = format_css_number(number.representation(), 0, coefficient_limit)?;
    context.append_temporary(&mut output, suffix)?;
    Ok(output)
}

pub(crate) fn capture_quirky_length(
    component: &CssComponentValue,
    context: &mut SpecifiedSerializationContext,
) -> SerializationResult<String> {
    let (number, _) = visit_literal(component, context)?;
    if context.output_suppressed() {
        return Ok(String::new());
    }
    let coefficient_limit = context.remaining_bytes().checked_sub(2).ok_or_else(|| {
        CssSerializationError::new(CssSpecifiedValueSerializationErrorKind::ByteLimit)
    })?;
    let mut output = format_css_number(number.representation(), 0, coefficient_limit)?;
    context.append_temporary(&mut output, "px")?;
    Ok(output)
}

#[cfg(test)]
mod ordinary_literal_comparison_tests {
    use super::*;

    fn component(text: &str) -> CssComponentValue {
        let values = crate::component::values::parse_component_values(text).unwrap();
        let [value] = values.items() else {
            panic!("one retained component")
        };
        value.clone()
    }

    #[test]
    fn coefficient_equality_retains_token_kind_unit_and_original_components() {
        for (left, right) in [
            ("+1.0", "01e0"),
            (".123456410%", "12345641e-8%"),
            (".12345641PX", "12345641e-8px"),
            ("2x", "2.0DPPX"),
            ("-0e999999999999999999999999999999999999999px", "+0px"),
        ] {
            let left = component(left);
            let right = component(right);
            let before = (left.clone(), right.clone());
            assert_ne!(left, right);
            assert!(ordinary_literal_equal(&left, &right));
            assert!(ordinary_literal_equal(&right, &left));
            assert_eq!((left, right), before);
        }
        for (left, right) in [
            (".12345641px", ".12345642px"),
            ("1", "1%"),
            ("1px", "1em"),
            ("1in", "96px"),
            ("0px", "0%"),
            ("auto", "auto"),
        ] {
            assert!(!ordinary_literal_equal(&component(left), &component(right)));
        }
    }

    #[test]
    fn implicit_pixels_normalize_only_exact_unitless_length_zero() {
        for left in ["0", "-0.00", "+0e999999999999999999999999999999999999999"] {
            for right in [
                "0px",
                "-0PX",
                "+0e-999999999999999999999999999999999999999px",
            ] {
                let left = component(left);
                let right = component(right);
                let before = (left.clone(), right.clone());
                assert!(!ordinary_literal_equal(&left, &right));
                assert!(ordinary_length_literal_equal(&left, &right));
                assert!(ordinary_length_literal_equal(&right, &left));
                assert_eq!((left, right), before);
            }
        }
        for (left, right) in [
            ("0", "0em"),
            ("0px", "0cm"),
            ("0", "0%"),
            (".0000001px", "0px"),
            ("-.0000001px", "0"),
        ] {
            assert!(!ordinary_length_literal_equal(
                &component(left),
                &component(right)
            ));
        }
        assert!(ordinary_length_literal_equal(
            &component("-0em"),
            &component("0EM")
        ));
    }

    #[test]
    fn unbounded_exponent_spelling_is_compared_without_serialized_expansion() {
        for (left, right) in [
            (
                "10e170141183460469231731687303715884105726px",
                "0.1e170141183460469231731687303715884105728PX",
            ),
            (
                "0.1e-170141183460469231731687303715884105727px",
                "10e-170141183460469231731687303715884105729px",
            ),
            (
                "1e-10000000000000000000000000000000000000000px",
                "10e-10000000000000000000000000000000000000001px",
            ),
        ] {
            assert!(ordinary_literal_equal(&component(left), &component(right)));
        }
        assert!(!ordinary_literal_equal(
            &component("1e-10000000000000000000000000000000000000000px"),
            &component("1e-10000000000000000000000000000000000000001px"),
        ));
    }
}
