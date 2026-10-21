//! One generic dimension literal and retained-math value.
//!
//! Values 4 WD 2024-03-12 §7 defines `<angle>`, `<time>`, `<frequency>` and
//! `<resolution>` each as "a dimension with units". Their ordinary literals
//! decode a unit through the one unit registry, and their values retain either
//! such a literal or root math. The public owners wrap these cores and keep
//! only their own ranges and grammar-specific exceptions.

use crate::{
    CssComponentValue, CssComponentValueError, CssComponentValueErrorKind, CssComponentValueRef,
    CssComponentValues, CssNumericConstructionError, CssNumericConstructionErrorKind,
    CssNumericTokenRef, CssNumericUnit, CssValueOrigin, CssValueTokenRef,
};

/// A unit enumeration owned by one dimension of the unit registry.
pub(crate) trait DimensionUnit: Copy + Eq + std::fmt::Debug {
    /// Selects this dimension's unit from a decoded registry unit.
    fn from_numeric(unit: CssNumericUnit) -> Option<Self>;
}

macro_rules! dimension_unit {
    ($($unit:ident => $dimension:ident),* $(,)?) => {$(
        impl DimensionUnit for crate::$unit {
            fn from_numeric(unit: CssNumericUnit) -> Option<Self> {
                match unit {
                    CssNumericUnit::$dimension(unit) => Some(unit),
                    _ => None,
                }
            }
        }
    )*};
}
dimension_unit!(
    CssAngleUnit => Angle,
    CssTimeUnit => Time,
    CssFrequencyUnit => Frequency,
    CssResolutionUnit => Resolution,
);

/// Decodes an ordinary dimension token whose unit belongs to `U`.
pub(crate) fn decode_unit<U: DimensionUnit>(
    component: &CssComponentValue,
) -> Result<U, CssComponentValueError> {
    let unit = match component.view() {
        CssComponentValueRef::Token(CssValueTokenRef::Dimension { unit, .. }) => {
            crate::numeric::units::parse(unit).and_then(U::from_numeric)
        }
        _ => None,
    };
    unit.ok_or_else(|| {
        CssComponentValueError::new(
            CssComponentValueErrorKind::InvalidToken,
            component.origin().clone(),
        )
    })
}

/// The exact authored coefficient of a checked dimension token.
pub(crate) fn dimension_number(component: &CssComponentValue) -> CssNumericTokenRef<'_> {
    let CssComponentValueRef::Token(CssValueTokenRef::Dimension { number, .. }) = component.view()
    else {
        unreachable!("checked dimension")
    };
    number
}

/// A public literal owner that a calculation's bare ordinary root re-enters.
pub(crate) trait DimensionRoot: Sized {
    fn from_root(component: CssComponentValue) -> Result<Self, CssNumericConstructionError>;
    fn origin(&self) -> &CssValueOrigin;
    fn structural_eq(&self, other: &Self) -> bool;
}

/// A typed root calculation of one dimension.
pub(crate) trait DimensionCalculation {
    fn components(&self) -> &CssComponentValues;
    fn origin(&self) -> &CssValueOrigin;
    fn expression(&self) -> &crate::numeric::CssCalculationExpression;
    /// The bare ordinary root, when the calculation is not math.
    fn literal_root_component(&self) -> Option<&CssComponentValue> {
        self.expression().literal_root()
    }
}

/// How a dimension value admits a calculation.
#[derive(Clone, Copy)]
pub(crate) enum Admission {
    /// Checked construction rejects components recovered by implicit closure.
    Checked,
    /// Parser admission retains recovery, which parsing reports separately.
    Recovering,
}

/// Rejects the first component recovered by implicit closure.
pub(crate) fn reject_recovered(
    values: &CssComponentValues,
) -> Result<(), CssNumericConstructionError> {
    match values.first_implicit_origin() {
        Some(origin) => Err(CssNumericConstructionError::at_origin(
            CssNumericConstructionErrorKind::RecoveredComponent,
            origin.clone(),
        )),
        None => Ok(()),
    }
}

/// An ordinary dimension literal or retained root math.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum DimensionValue<L, C> {
    Literal(L),
    Calculation(C),
}

impl<L: DimensionRoot, C: DimensionCalculation> DimensionValue<L, C> {
    pub(crate) fn literal(&self) -> Option<&L> {
        match self {
            Self::Literal(value) => Some(value),
            Self::Calculation(_) => None,
        }
    }

    pub(crate) fn calculation(&self) -> Option<&C> {
        match self {
            Self::Calculation(value) => Some(value),
            Self::Literal(_) => None,
        }
    }

    pub(crate) fn origin(&self) -> &CssValueOrigin {
        match self {
            Self::Literal(value) => value.origin(),
            Self::Calculation(value) => value.origin(),
        }
    }

    /// Checks recovery under checked admission, normalizes a bare ordinary
    /// root into the literal owner and retains math roots.
    pub(crate) fn from_calculation(
        calculation: C,
        admission: Admission,
    ) -> Result<Self, CssNumericConstructionError> {
        if matches!(admission, Admission::Checked) {
            reject_recovered(calculation.components())?;
        }
        if let Some(component) = calculation.literal_root_component() {
            return L::from_root(component.clone()).map(Self::Literal);
        }
        Ok(Self::Calculation(calculation))
    }

    /// Admits one parsed or checked property component.
    ///
    /// An ordinary token takes the literal path. Otherwise the numeric
    /// context's admission policy applies; `closure_first` additionally
    /// reports a strict context's recovered closure before math admission.
    pub(crate) fn from_parser_component(
        component: CssComponentValue,
        context: &crate::numeric::NumericInputContext<'_>,
        root: crate::numeric::CalculationRoot,
        closure_first: bool,
        calculation: fn(crate::numeric::CssCalculationExpression) -> C,
    ) -> Result<Self, CssNumericConstructionError> {
        if matches!(component.view(), CssComponentValueRef::Token(_)) {
            return L::from_root(component).map(Self::Literal);
        }
        let values = CssComponentValues::try_new(vec![component])
            .map_err(CssNumericConstructionError::component)?;
        if closure_first {
            match context.ordinary() {
                crate::numeric::NumericInputContext::QuirkyLengths(_) => {
                    unreachable!("ordinary numeric provenance")
                }
                crate::numeric::NumericInputContext::Parsed(_) => {}
                crate::numeric::NumericInputContext::Components(..)
                | crate::numeric::NumericInputContext::Checked => reject_recovered(&values)?,
            }
        }
        let expression = context.admit(values, root)?;
        Self::from_calculation(calculation(expression), Admission::Recovering)
    }

    /// The first recovered closure in retained math.
    pub(crate) fn first_implicit_origin(&self) -> Option<&CssValueOrigin> {
        self.calculation()
            .and_then(|value| value.components().first_implicit_origin())
    }

    pub(crate) fn ensure_closed(&self) -> Result<(), CssNumericConstructionError> {
        match self.calculation() {
            Some(value) => reject_recovered(value.components()),
            None => Ok(()),
        }
    }

    pub(crate) fn structural_eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Literal(left), Self::Literal(right)) => left.structural_eq(right),
            (Self::Calculation(left), Self::Calculation(right)) => {
                left.expression().structural_eq(right.expression())
            }
            _ => false,
        }
    }
}

/// Declares a public ordinary dimension literal over [`DimensionLiteral`].
macro_rules! dimension_literal_owner {
    ($(#[$meta:meta])* $owner:ident, $unit:ident, $suffix:path) => {
        $(#[$meta])*
        #[derive(Clone, Debug, Eq, PartialEq)]
        pub struct $owner {
            component: Box<crate::CssComponentValue>,
            unit: crate::$unit,
        }
        impl $owner {
            /// Constructs an exact coefficient with the selected unit.
            pub fn try_new(
                number: &str,
                unit: crate::$unit,
            ) -> Result<Self, crate::CssComponentValueError> {
                Self::try_from_component(crate::CssComponentValue::try_dimension(
                    number,
                    $suffix(unit),
                )?)
            }
            /// Requires one dimension token whose unit decodes into this dimension.
            pub fn try_from_component(
                component: crate::CssComponentValue,
            ) -> Result<Self, crate::CssComponentValueError> {
                let unit = crate::dimension::decode_unit(&component)?;
                Ok(Self {
                    component: Box::new(component),
                    unit,
                })
            }
            /// Returns the exact authored numeric token.
            pub fn numeric(&self) -> crate::CssNumericTokenRef<'_> {
                crate::dimension::dimension_number(&self.component)
            }
            /// Returns the decoded unit without converting the coefficient.
            pub const fn unit(&self) -> crate::$unit {
                self.unit
            }
            /// Borrows the original checked component.
            pub const fn component(&self) -> &crate::CssComponentValue {
                &self.component
            }
            /// Borrows the original component provenance.
            pub const fn origin(&self) -> &crate::CssValueOrigin {
                self.component.origin()
            }
        }
        impl crate::dimension::DimensionRoot for $owner {
            fn from_root(
                component: crate::CssComponentValue,
            ) -> Result<Self, crate::CssNumericConstructionError> {
                Self::try_from_component(component)
                    .map_err(crate::CssNumericConstructionError::component)
            }
            fn origin(&self) -> &crate::CssValueOrigin {
                self.component.origin()
            }
            fn structural_eq(&self, other: &Self) -> bool {
                self.unit == other.unit
                    && self
                        .component
                        .structural_eq_ignoring_origin(&other.component)
            }
        }
    };
}
pub(crate) use dimension_literal_owner;

/// Declares a public dimension value over [`DimensionValue`].
macro_rules! dimension_value_owner {
    ($(#[$meta:meta])* $owner:ident, $literal:ident, $calculation:ident $(, { $($extra:item)* })?) => {
        $(#[$meta])*
        #[derive(Clone, Debug, Eq, PartialEq)]
        pub struct $owner {
            value: crate::dimension::DimensionValue<crate::$literal, crate::$calculation>,
        }
        impl $owner {
            /// Retains an already checked ordinary literal.
            pub fn from_literal(literal: crate::$literal) -> Self {
                Self {
                    value: crate::dimension::DimensionValue::Literal(literal),
                }
            }
            /// Rejects recovered closure, normalizes an ordinary root and retains math roots.
            pub fn try_from_calculation(
                calculation: crate::$calculation,
            ) -> Result<Self, crate::CssNumericConstructionError> {
                crate::dimension::DimensionValue::from_calculation(
                    calculation,
                    crate::dimension::Admission::Checked,
                )
                .map(|value| Self { value })
            }
            /// Borrows the ordinary branch, when present.
            pub fn literal(&self) -> Option<&crate::$literal> {
                self.value.literal()
            }
            /// Borrows the retained math branch, when present.
            pub fn calculation(&self) -> Option<&crate::$calculation> {
                self.value.calculation()
            }
            /// Borrows the selected branch's original provenance.
            pub fn origin(&self) -> &crate::CssValueOrigin {
                self.value.origin()
            }
            $($($extra)*)?
        }
    };
}
pub(crate) use dimension_value_owner;
