//! Borrowed intrinsic parsed alpha alongside the unchanged authored color graph.

use crate::{
    CssColorComponent, CssNumberCalculation, CssPercentageCalculation, CssValueOrigin,
    exact_decimal::LexicalDecimal,
};

/// A borrowed ordinary color alpha at the intrinsic parsed-value phase.
///
/// Direct scalars follow [Color 4 §4.2](https://www.w3.org/TR/2026/CRD-css-color-4-20260908/#alpha-syntax):
/// percentages scale by 1/100 and out-of-range values clamp to [0, 1]. Omission
/// defaults to opaque while remaining distinguishable from explicit unity.
/// Missing alpha remains distinct from zero. Calculations retain their authored
/// identities and are not evaluated or finalized by this view.
///
/// The owning payload's `alpha()` accessor still borrows the original authored
/// representation. This view applies equally to parsing and checked construction;
/// it does not resolve relative colors, evaluate origin colors, or select a
/// serializer's later family-specific calculation phase.
///
/// ```
/// use surgeist_css::{CssColorComponent, CssColorNumberLiteral, CssComponentValue,
///     CssParsedColorAlphaRef, CssPredefinedColor, CssPredefinedColorSpace};
/// let number = |text| CssColorComponent::Number(
///     CssColorNumberLiteral::try_from_component(CssComponentValue::try_number(text).unwrap()).unwrap());
/// let color = CssPredefinedColor::try_new(CssPredefinedColorSpace::Srgb,
///     [number("1"), number("0"), number("0")], Some(number("2"))).unwrap();
/// let CssParsedColorAlphaRef::Scalar(alpha) = color.parsed_alpha() else { panic!("direct alpha") };
/// assert!(alpha.is_one());
/// assert_eq!(alpha.as_unit_f64(), 1.0);
/// assert_eq!(alpha.authored_component(), color.alpha().unwrap());
/// ```
///
/// External matches must leave room for future parsed-alpha branches:
///
/// ```compile_fail
/// use surgeist_css::CssParsedColorAlphaRef;
/// fn classify(alpha: CssParsedColorAlphaRef<'_>) -> bool {
///     match alpha {
///         CssParsedColorAlphaRef::Omitted => true,
///         CssParsedColorAlphaRef::Missing => false,
///         CssParsedColorAlphaRef::Scalar(_) => false,
///         CssParsedColorAlphaRef::NumberCalculation(_) => false,
///         CssParsedColorAlphaRef::PercentageCalculation(_) => false,
///     }
/// }
/// ```
#[non_exhaustive]
#[derive(Clone, Copy, Debug)]
pub enum CssParsedColorAlphaRef<'a> {
    /// Alpha was omitted and intrinsically defaults to opaque.
    Omitted,
    /// Alpha was explicitly authored as the missing component `none`.
    Missing,
    /// A direct number or percentage with exact bounded endpoint classification.
    Scalar(CssColorAlphaScalarRef<'a>),
    /// An authored number calculation awaiting its applicable range/evaluation phase.
    NumberCalculation(&'a CssNumberCalculation),
    /// An authored percentage calculation awaiting its applicable range/evaluation phase.
    PercentageCalculation(&'a CssPercentageCalculation),
}

#[derive(Clone, Copy, Debug)]
enum ScalarClass {
    Zero,
    One,
    Interior,
}

/// A borrowed direct alpha scalar with exact classification in [0, 1].
///
/// The original number or percentage and its provenance remain available.
/// Endpoint predicates describe the exact scaled and clamped decimal; they do
/// not derive their result from the approximate [`Self::as_unit_f64`] observation.
/// A positive interior decimal can underflow to zero or round to one in `f64`
/// while [`Self::is_interior`] remains true.
///
/// Values are obtained only from the owning color payload's `parsed_alpha()`.
/// The private representation prevents constructing a scalar from `none` or math:
///
/// ```compile_fail
/// use surgeist_css::{CssColorAlphaScalarRef, CssColorComponent};
/// let scalar = CssColorAlphaScalarRef { component: &CssColorComponent::None };
/// ```
#[derive(Clone, Copy, Debug)]
pub struct CssColorAlphaScalarRef<'a> {
    component: &'a CssColorComponent,
    class: ScalarClass,
}

impl<'a> CssColorAlphaScalarRef<'a> {
    /// Whether the exact scaled and clamped value is zero.
    #[must_use]
    pub const fn is_zero(&self) -> bool {
        matches!(self.class, ScalarClass::Zero)
    }

    /// Whether the exact scaled and clamped value is one.
    #[must_use]
    pub const fn is_one(&self) -> bool {
        matches!(self.class, ScalarClass::One)
    }

    /// Whether the exact scaled value lies strictly between zero and one.
    #[must_use]
    pub const fn is_interior(&self) -> bool {
        matches!(self.class, ScalarClass::Interior)
    }

    /// Borrows the original number or percentage without changing its spelling.
    #[must_use]
    pub const fn authored_component(&self) -> &'a CssColorComponent {
        self.component
    }

    /// Borrows the original parsed or explicitly programmatic provenance.
    #[must_use]
    pub fn origin(&self) -> &'a CssValueOrigin {
        match self.component {
            CssColorComponent::Number(value) => value.origin(),
            CssColorComponent::Percentage(value) => value.origin(),
            _ => unreachable!("checked direct alpha scalar"),
        }
    }

    /// Observes an approximate finite unit alpha in [0, 1].
    ///
    /// Exact zero and one return those endpoints, with positive zero. Interior
    /// percentages are divided by 100 after binary64 approximation. Rounding or
    /// underflow can produce an endpoint for an exact interior value, so use the
    /// exact predicates rather than this observation to classify endpoints.
    #[must_use]
    pub fn as_unit_f64(&self) -> f64 {
        match self.class {
            ScalarClass::Zero => 0.0,
            ScalarClass::One => 1.0,
            ScalarClass::Interior => {
                let (text, percentage) = match self.component {
                    CssColorComponent::Number(value) => (value.numeric().representation(), false),
                    CssColorComponent::Percentage(value) => {
                        (value.numeric().representation(), true)
                    }
                    _ => unreachable!("checked direct alpha scalar"),
                };
                let value = text.parse::<f64>().expect("checked finite decimal syntax");
                (if percentage { value / 100.0 } else { value }).clamp(0.0, 1.0)
            }
        }
    }
}

pub(crate) fn parsed_alpha(alpha: Option<&CssColorComponent>) -> CssParsedColorAlphaRef<'_> {
    match alpha {
        None => CssParsedColorAlphaRef::Omitted,
        Some(CssColorComponent::None) => CssParsedColorAlphaRef::Missing,
        Some(component @ (CssColorComponent::Number(_) | CssColorComponent::Percentage(_))) => {
            let (text, percentage) = match component {
                CssColorComponent::Number(value) => (value.numeric().representation(), false),
                CssColorComponent::Percentage(value) => (value.numeric().representation(), true),
                _ => unreachable!("direct alpha component"),
            };
            let lexical = LexicalDecimal::new(text);
            let class = if lexical.len == 0 || lexical.negative {
                ScalarClass::Zero
            } else if lexical.at_least_unit_endpoint(percentage) {
                ScalarClass::One
            } else {
                ScalarClass::Interior
            };
            CssParsedColorAlphaRef::Scalar(CssColorAlphaScalarRef { component, class })
        }
        Some(CssColorComponent::NumberCalculation(value)) => {
            CssParsedColorAlphaRef::NumberCalculation(value)
        }
        Some(CssColorComponent::PercentageCalculation(value)) => {
            CssParsedColorAlphaRef::PercentageCalculation(value)
        }
    }
}
