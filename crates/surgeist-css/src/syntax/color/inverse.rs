//! Private specified-value relation for declaration-block inversion.
//!
//! This retains the owner's representation, spaces, profiles and symbolic math.
//! Diagnostic coordinates and exact default spellings are not value identity.

use super::*;
use crate::{
    CssParsedColorAlphaRef, CssProfileColorExpression, CssProfileColorExpressionRef,
    CssSpecifiedValueSerializationError as Error, CssSpecifiedValueSerializationErrorKind,
    exact_decimal::LexicalDecimal, specified_serialization::SpecifiedSerializationContext,
};

type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Copy, Eq, PartialEq)]
enum Role {
    Ordinary,
    Origin,
}
enum Work<'a> {
    Colors(&'a CssColor, &'a CssColor, Role),
    MixChildren(
        &'a [CssColorMixComponent],
        &'a [CssColorMixComponent],
        usize,
    ),
}
fn push<'a>(
    work: &mut Vec<Work<'a>>,
    value: Work<'a>,
    context: &mut SpecifiedSerializationContext,
) -> Result<()> {
    // Each scheduled comparison is admitted before reserving its real work slot.
    context.charge_projection(1)?;
    work.try_reserve(1)
        .map_err(|_| Error::new(CssSpecifiedValueSerializationErrorKind::CapacityOverflow))?;
    work.push(value);
    Ok(())
}
impl CssColor {
    pub(crate) fn specified_inverse_eq(
        &self,
        other: &Self,
        context: &mut SpecifiedSerializationContext,
    ) -> Result<bool> {
        use CssColorRepresentation as R;
        let mut work = Vec::new();
        push(
            &mut work,
            Work::Colors(self, other, Role::Ordinary),
            context,
        )?;
        while let Some(item) = work.pop() {
            let Work::Colors(left, right, role) = item else {
                let Work::MixChildren(left, right, index) = item else {
                    unreachable!()
                };
                if index < left.len() {
                    if index + 1 < left.len() {
                        push(
                            &mut work,
                            Work::MixChildren(left, right, index + 1),
                            context,
                        )?;
                    }
                    push(
                        &mut work,
                        Work::Colors(left[index].color(), right[index].color(), Role::Ordinary),
                        context,
                    )?;
                }
                continue;
            };
            let equal = match (&left.representation, &right.representation) {
                (R::CurrentColor, R::CurrentColor) | (R::Transparent, R::Transparent) => true,
                (R::Hex(a), R::Hex(b)) => a == b,
                (R::Named(a), R::Named(b)) => a == b,
                (R::System(a), R::System(b)) => a == b,
                (R::Rgb(a), R::Rgb(b)) => {
                    components_equal(&a.channels, &b.channels, context)?
                        && absolute_alpha_equal(a.alpha.as_ref(), b.alpha.as_ref(), role, context)?
                }
                (R::Hsl(a), R::Hsl(b)) => {
                    hue_equal(&a.hue, &b.hue, context)?
                        && component_equal(&a.saturation, &b.saturation, context)?
                        && component_equal(&a.lightness, &b.lightness, context)?
                        && absolute_alpha_equal(a.alpha.as_ref(), b.alpha.as_ref(), role, context)?
                }
                (R::Hwb(a), R::Hwb(b)) => {
                    hue_equal(&a.hue, &b.hue, context)?
                        && component_equal(&a.whiteness, &b.whiteness, context)?
                        && component_equal(&a.blackness, &b.blackness, context)?
                        && absolute_alpha_equal(a.alpha.as_ref(), b.alpha.as_ref(), role, context)?
                }
                (R::Lab(a), R::Lab(b)) | (R::Oklab(a), R::Oklab(b)) => {
                    component_equal(&a.lightness, &b.lightness, context)?
                        && component_equal(&a.a, &b.a, context)?
                        && component_equal(&a.b, &b.b, context)?
                        && absolute_alpha_equal(a.alpha.as_ref(), b.alpha.as_ref(), role, context)?
                }
                (R::Lch(a), R::Lch(b)) | (R::Oklch(a), R::Oklch(b)) => {
                    component_equal(&a.lightness, &b.lightness, context)?
                        && component_equal(&a.chroma, &b.chroma, context)?
                        && hue_equal(&a.hue, &b.hue, context)?
                        && absolute_alpha_equal(a.alpha.as_ref(), b.alpha.as_ref(), role, context)?
                }
                (R::Predefined(a), R::Predefined(b)) => {
                    a.color_space == b.color_space
                        && components_equal(&a.channels, &b.channels, context)?
                        && absolute_alpha_equal(a.alpha.as_ref(), b.alpha.as_ref(), role, context)?
                }
                (R::Custom(a), R::Custom(b)) => {
                    a.profile == b.profile
                        && components_equal(&a.channels, &b.channels, context)?
                        && absolute_alpha_equal(a.alpha.as_ref(), b.alpha.as_ref(), role, context)?
                }
                (R::DeviceCmyk(a), R::DeviceCmyk(b)) => {
                    components_equal(&a.channels, &b.channels, context)?
                        && absolute_alpha_equal(a.alpha.as_ref(), b.alpha.as_ref(), role, context)?
                }
                (R::Relative(a), R::Relative(b)) => {
                    if a.function != b.function
                        || a.environment != b.environment
                        || !relative_channels_equal(&a.channels, &b.channels, context)?
                        || !relative_alpha_equal(a.alpha.as_ref(), b.alpha.as_ref(), context)?
                    {
                        false
                    } else {
                        push(
                            &mut work,
                            Work::Colors(&a.source, &b.source, Role::Origin),
                            context,
                        )?;
                        true
                    }
                }
                (R::RelativeCustom(a), R::RelativeCustom(b)) => {
                    if a.profile != b.profile || a.channels.len() != b.channels.len() {
                        false
                    } else {
                        let mut equal = true;
                        for (a, b) in a.channels.iter().zip(&b.channels) {
                            if !profile_expression_equal(a, b, context)? {
                                equal = false;
                                break;
                            }
                        }
                        if !equal
                            || !profile_alpha_equal(a.alpha.as_ref(), b.alpha.as_ref(), context)?
                        {
                            false
                        } else {
                            push(
                                &mut work,
                                Work::Colors(&a.source, &b.source, Role::Origin),
                                context,
                            )?;
                            true
                        }
                    }
                }
                (R::Alpha(a), R::Alpha(b)) => {
                    if !relative_alpha_equal(a.alpha.as_ref(), b.alpha.as_ref(), context)? {
                        false
                    } else {
                        push(
                            &mut work,
                            Work::Colors(&a.source, &b.source, Role::Origin),
                            context,
                        )?;
                        true
                    }
                }
                (R::ColorMix(a), R::ColorMix(b)) => {
                    if a.components.len() != b.components.len()
                        || !interpolation_equal(a.interpolation.as_ref(), b.interpolation.as_ref())
                        || !mix_weights_equal(&a.components, &b.components, context)?
                    {
                        false
                    } else {
                        push(
                            &mut work,
                            Work::MixChildren(&a.components, &b.components, 0),
                            context,
                        )?;
                        true
                    }
                }
                (R::LightDark(a), R::LightDark(b)) => {
                    push(
                        &mut work,
                        Work::Colors(&a.dark, &b.dark, Role::Ordinary),
                        context,
                    )?;
                    push(
                        &mut work,
                        Work::Colors(&a.light, &b.light, Role::Ordinary),
                        context,
                    )?;
                    true
                }
                (R::ContrastColor(a), R::ContrastColor(b)) => {
                    push(
                        &mut work,
                        Work::Colors(&a.color, &b.color, Role::Ordinary),
                        context,
                    )?;
                    true
                }
                _ => false,
            };
            if !equal {
                return Ok(false);
            }
        }
        Ok(true)
    }
}

fn components_equal(
    left: &[CssColorComponent],
    right: &[CssColorComponent],
    context: &mut SpecifiedSerializationContext,
) -> Result<bool> {
    if left.len() != right.len() {
        return Ok(false);
    }
    for (a, b) in left.iter().zip(right) {
        if !component_equal(a, b, context)? {
            return Ok(false);
        }
    }
    Ok(true)
}
fn component_equal(
    left: &CssColorComponent,
    right: &CssColorComponent,
    context: &mut SpecifiedSerializationContext,
) -> Result<bool> {
    use CssColorComponent as V;
    context.charge_projection(1)?;
    Ok(match (left, right) {
        (V::None, V::None) => true,
        (V::Number(a), V::Number(b)) => {
            crate::specified_numeric::ordinary_literal_equal(a.component(), b.component())
        }
        (V::Percentage(a), V::Percentage(b)) => {
            crate::specified_numeric::ordinary_literal_equal(a.component(), b.component())
        }
        (V::NumberCalculation(a), V::NumberCalculation(b)) => {
            a.expression.specified_inverse_eq(&b.expression, context)?
        }
        (V::HintedNumberCalculation(a), V::HintedNumberCalculation(b)) => {
            a.expression.specified_inverse_eq(&b.expression, context)?
        }
        (V::PercentageCalculation(a), V::PercentageCalculation(b)) => {
            a.expression.specified_inverse_eq(&b.expression, context)?
        }
        _ => false,
    })
}
fn hue_equal(
    left: &CssColorHue,
    right: &CssColorHue,
    context: &mut SpecifiedSerializationContext,
) -> Result<bool> {
    use CssColorHue as V;
    context.charge_projection(1)?;
    Ok(match (left, right) {
        (V::None, V::None) => true,
        (V::Number(a), V::Number(b)) => {
            crate::specified_numeric::ordinary_literal_equal(a.component(), b.component())
        }
        (V::Angle(a), V::Angle(b)) => {
            crate::specified_numeric::ordinary_literal_equal(a.component(), b.component())
        }
        (V::NumberCalculation(a), V::NumberCalculation(b)) => {
            a.expression.specified_inverse_eq(&b.expression, context)?
        }
        (V::AngleCalculation(a), V::AngleCalculation(b)) => {
            a.expression.specified_inverse_eq(&b.expression, context)?
        }
        _ => false,
    })
}
fn alpha_scalar(value: &CssColorComponent) -> LexicalDecimal<'_> {
    match value {
        CssColorComponent::Number(value) => LexicalDecimal::new(value.numeric().representation()),
        CssColorComponent::Percentage(value) => {
            LexicalDecimal::new(value.numeric().representation()).shifted(-2)
        }
        _ => unreachable!("parsed direct alpha scalar"),
    }
}
fn absolute_alpha_equal(
    left: Option<&CssColorComponent>,
    right: Option<&CssColorComponent>,
    role: Role,
    context: &mut SpecifiedSerializationContext,
) -> Result<bool> {
    use CssParsedColorAlphaRef as A;
    context.charge_projection(1)?;
    // Origin serialization retains explicit alpha and its authored numeric
    // phase. It is never reclassified through ordinary parsed alpha defaults.
    if role == Role::Origin {
        return match (left, right) {
            (None, None) => Ok(true),
            (Some(a), Some(b)) => component_equal(a, b, context),
            _ => Ok(false),
        };
    }
    Ok(
        match (
            crate::color_alpha::parsed_alpha(left),
            crate::color_alpha::parsed_alpha(right),
        ) {
            (A::Omitted, A::Omitted) | (A::Missing, A::Missing) => true,
            (A::Omitted, A::Scalar(v)) | (A::Scalar(v), A::Omitted) => v.is_one(),
            (A::Scalar(a), A::Scalar(b)) => {
                (a.is_zero() && b.is_zero())
                    || (a.is_one() && b.is_one())
                    || (a.is_interior()
                        && b.is_interior()
                        && alpha_scalar(a.authored_component())
                            .value_eq(&alpha_scalar(b.authored_component())))
            }
            (A::NumberCalculation(a), A::NumberCalculation(b)) => {
                a.expression.specified_inverse_eq(&b.expression, context)?
            }
            (A::HintedNumberCalculation(a), A::HintedNumberCalculation(b)) => {
                a.expression.specified_inverse_eq(&b.expression, context)?
            }
            (A::PercentageCalculation(a), A::PercentageCalculation(b)) => {
                a.expression.specified_inverse_eq(&b.expression, context)?
            }
            _ => false,
        },
    )
}
fn relative_channels_equal(
    left: &[CssRelativeColorExpression],
    right: &[CssRelativeColorExpression],
    context: &mut SpecifiedSerializationContext,
) -> Result<bool> {
    if left.len() != right.len() {
        return Ok(false);
    }
    for (a, b) in left.iter().zip(right) {
        if !relative_expression_equal(a, b, context)? {
            return Ok(false);
        }
    }
    Ok(true)
}
fn relative_expression_equal(
    left: &CssRelativeColorExpression,
    right: &CssRelativeColorExpression,
    context: &mut SpecifiedSerializationContext,
) -> Result<bool> {
    use CssRelativeColorExpressionValue as V;
    context.charge_projection(1)?;
    if left.environment != right.environment || left.result_domain != right.result_domain {
        return Ok(false);
    }
    Ok(match (&left.value, &right.value) {
        (V::None, V::None) => true,
        (V::Number(a), V::Number(b)) => {
            crate::specified_numeric::ordinary_literal_equal(a.component(), b.component())
        }
        (V::Percentage(a), V::Percentage(b)) => {
            crate::specified_numeric::ordinary_literal_equal(a.component(), b.component())
        }
        (V::Angle(a), V::Angle(b)) => {
            crate::specified_numeric::ordinary_literal_equal(a.component(), b.component())
        }
        (V::Channel(a), V::Channel(b)) => a == b,
        (V::Calculation(a), V::Calculation(b)) => {
            a.data.result_type == b.data.result_type
                && a.data.references == b.data.references
                && a.data
                    .expression
                    .specified_inverse_eq(&b.data.expression, context)?
        }
        _ => false,
    })
}
fn relative_alpha_equal(
    left: Option<&CssRelativeColorExpression>,
    right: Option<&CssRelativeColorExpression>,
    context: &mut SpecifiedSerializationContext,
) -> Result<bool> {
    context.charge_projection(1)?;
    match (left, right) {
        (None, None) => Ok(true),
        // An explicit override remains a source-bound expression. Omission
        // inherits the source and is not an ordinary opaque-alpha default.
        (None, Some(_)) | (Some(_), None) => Ok(false),
        (Some(a), Some(b)) => relative_expression_equal(a, b, context),
    }
}
fn profile_expression_equal(
    left: &CssProfileColorExpression,
    right: &CssProfileColorExpression,
    context: &mut SpecifiedSerializationContext,
) -> Result<bool> {
    use CssProfileColorExpressionRef as V;
    context.charge_projection(1)?;
    match (left.view(), right.view()) {
        (V::Literal(a), V::Literal(b)) => component_equal(a, b, context),
        (V::Reference(a), V::Reference(b)) => Ok(a == b),
        (V::Calculation(a), V::Calculation(b)) => a.specified_inverse_eq(b, context),
        _ => Ok(false),
    }
}
fn profile_alpha_equal(
    left: Option<&CssProfileColorExpression>,
    right: Option<&CssProfileColorExpression>,
    context: &mut SpecifiedSerializationContext,
) -> Result<bool> {
    context.charge_projection(1)?;
    match (left, right) {
        (None, None) => Ok(true),
        (None, Some(_)) | (Some(_), None) => Ok(false),
        (Some(a), Some(b)) => profile_expression_equal(a, b, context),
    }
}
fn interpolation_equal(
    left: Option<&CssColorInterpolation>,
    right: Option<&CssColorInterpolation>,
) -> bool {
    use ColorInterpolation as I;
    let default = CssColorInterpolationMethod::try_new(CssColorInterpolationSpace::Oklab, None)
        .expect("rectangular default");
    match (left.map(|v| &v.0), right.map(|v| &v.0)) {
        (Some(I::Custom(a)), Some(I::Custom(b))) => a == b,
        (Some(I::Custom(_)), _) | (_, Some(I::Custom(_))) => false,
        (a, b) => {
            let a = match a {
                Some(I::Predefined(a)) => *a,
                _ => default,
            };
            let b = match b {
                Some(I::Predefined(b)) => *b,
                _ => default,
            };
            a.space() == b.space() && a.effective_hue() == b.effective_hue()
        }
    }
}
fn mix_weights_equal(
    left: &[CssColorMixComponent],
    right: &[CssColorMixComponent],
    context: &mut SpecifiedSerializationContext,
) -> Result<bool> {
    let calculations = left
        .iter()
        .chain(right)
        .any(|v| v.weight().is_some_and(|v| v.calculation().is_some()));
    if calculations {
        // A symbolic weight disables the owner's omitted-share computation.
        for (a, b) in left.iter().zip(right) {
            context.charge_projection(1)?;
            let equal = match (a.weight(), b.weight()) {
                (None, None) => true,
                (Some(a), Some(b)) => match (&a.0, &b.0) {
                    (ColorMixWeight::Literal(a), ColorMixWeight::Literal(b)) => {
                        crate::specified_numeric::ordinary_literal_equal(
                            a.literal().component(),
                            b.literal().component(),
                        )
                    }
                    (ColorMixWeight::Calculation(a), ColorMixWeight::Calculation(b)) => {
                        a.expression.specified_inverse_eq(&b.expression, context)?
                    }
                    _ => false,
                },
                _ => false,
            };
            if !equal {
                return Ok(false);
            }
        }
        return Ok(true);
    }
    // These two real containers are additional inverse work; ordinary mix
    // serialization keeps its original helper order and tariff unchanged.
    context.charge_projection(left.len())?;
    let a = serialization::effective_mix_weights(
        left.iter().map(CssColorMixComponent::weight),
        context,
    )?;
    context.charge_projection(right.len())?;
    let b = serialization::effective_mix_weights(
        right.iter().map(CssColorMixComponent::weight),
        context,
    )?;
    for index in 0..left.len() {
        context.charge_projection(1)?;
        if !a.value(index).compare(b.value(index), context)?.is_eq() {
            return Ok(false);
        }
    }
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn color(source: &str) -> CssColor {
        let report = crate::parse_style_attribute(&format!("color:{source}"));
        assert!(report.is_clean(), "{source}: {report:?}");
        let crate::CssKnownPropertyValueRef::Color(value) = report.syntax()[0]
            .known()
            .unwrap()
            .property_value()
            .unwrap()
        else {
            panic!("color owner")
        };
        value.value().clone()
    }
    #[test]
    fn direct_default_relation_pays_exact_comparison_work_and_retains_roles() {
        let left = color("rgb(1 2 3)");
        let right = color("rgba(1, 2, 3, 100%)");
        // One color frame, three channel comparisons and one parsed-alpha
        // comparison. Borrowed scalar/default equality allocates no digits.
        let mut context = SpecifiedSerializationContext::new(
            crate::CssSpecifiedValueSerializationLimits::new(0, 5, 0),
        );
        assert!(left.specified_inverse_eq(&right, &mut context).unwrap());
        let mut context = SpecifiedSerializationContext::new(
            crate::CssSpecifiedValueSerializationLimits::new(0, 4, 0),
        );
        assert_eq!(
            left.specified_inverse_eq(&right, &mut context)
                .unwrap_err()
                .kind(),
            CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit
        );
        for (left, right) in [
            ("color(srgb 1 0 0 / .9999996)", "color(srgb 1 0 0 / 1)"),
            ("color(srgb 1 0 0 / 1e-1000)", "color(srgb 1 0 0 / 0)"),
            ("rgb(1 2 3 / none)", "rgb(1 2 3)"),
            ("rgb(1 2 3 / calc(1))", "rgb(1 2 3 / 1)"),
            ("alpha(from rgb(1 2 3))", "alpha(from rgb(1 2 3 / 1))"),
            ("alpha(from rgb(1 2 3 / 2))", "alpha(from rgb(1 2 3 / 1))"),
            (
                "rgb(from rgb(1 2 3 / .2) r g b)",
                "rgb(from rgb(1 2 3 / .2) r g b / 1)",
            ),
            ("rgb(from red r g b)", "rgb(from red r g b / alpha)"),
            ("color-mix(red, blue)", "color-mix(red 70%, blue 70%)"),
            (
                "color-mix(red 60%, green, blue, yellow)",
                "color-mix(red 60%, green 13.333333%, blue 13.333333%, yellow 13.333333%)",
            ),
            (
                "color-mix(red, blue)",
                "color-mix(red calc(50%), blue calc(50%))",
            ),
        ] {
            let mut context = SpecifiedSerializationContext::new(
                crate::CssSpecifiedValueSerializationLimits::default(),
            );
            assert!(
                !color(left)
                    .specified_inverse_eq(&color(right), &mut context)
                    .unwrap(),
                "{left} != {right}"
            );
        }
    }
    #[test]
    fn retained_math_comparison_admits_its_real_scratch_nodes_before_allocation() {
        let left = color("rgb(calc(1 + 2) 2 3)");
        let right = color("rgb(calc(1 + 2) 2 3)");
        // Color frame + three channels + alpha = P5. Exact numeric graph
        // comparison schedules Calc, Sum and two leaves into four real work
        // slots, another P4. No value projection, I/B, or rounded text is used.
        let mut context = SpecifiedSerializationContext::new(
            crate::CssSpecifiedValueSerializationLimits::new(0, 9, 0),
        );
        assert!(left.specified_inverse_eq(&right, &mut context).unwrap());
        let mut context = SpecifiedSerializationContext::new(
            crate::CssSpecifiedValueSerializationLimits::new(0, 8, 0),
        );
        assert_eq!(
            left.specified_inverse_eq(&right, &mut context)
                .unwrap_err()
                .kind(),
            CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit
        );
    }
}
