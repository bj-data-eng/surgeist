//! The grammar-specific number-zero exception to strict authored angles.
use crate::{
    CssAngle, CssAngleUnit, CssComponentValue, CssComponentValueError, CssComponentValueErrorKind,
    CssComponentValueRef, CssNumeric, CssNumericConstructionError, CssNumericTokenRef,
    CssValueOrigin, CssValueTokenRef,
};

pub(crate) const fn suffix(unit: CssAngleUnit) -> &'static str {
    crate::numeric::units::suffix(crate::CssNumericUnit::Angle(unit))
}
fn invalid(component: &CssComponentValue) -> CssComponentValueError {
    CssComponentValueError::new(
        CssComponentValueErrorKind::InvalidToken,
        component.origin().clone(),
    )
}

/// One exact Number token with zero coefficient, retaining signed spelling and provenance.
///
/// # Equality
///
/// Every zero literal has the same exact value and no unit, so all are equal
/// (CSS Typed OM 1 "equal numeric values"); spelling and origin are ignored.
#[derive(Clone, Debug)]
pub struct CssZeroLiteral {
    component: Box<CssComponentValue>,
}

impl PartialEq for CssZeroLiteral {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl Eq for CssZeroLiteral {}
impl CssZeroLiteral {
    /// Requires an exact Number token whose coefficient is zero.
    pub fn try_from_component(
        component: CssComponentValue,
    ) -> Result<Self, CssComponentValueError> {
        if let CssComponentValueRef::Token(CssValueTokenRef::Number(number)) = component.view()
            && crate::exact_decimal::LexicalDecimal::new(number.representation()).is_zero()
        {
            return Ok(Self {
                component: Box::new(component),
            });
        }
        Err(invalid(&component))
    }
    /// Borrows the original signed or exponent-bearing zero spelling.
    pub fn numeric(&self) -> CssNumericTokenRef<'_> {
        let CssComponentValueRef::Token(CssValueTokenRef::Number(number)) = self.component.view()
        else {
            unreachable!("checked zero number")
        };
        number
    }
    /// Borrows the original checked component.
    pub const fn component(&self) -> &CssComponentValue {
        &self.component
    }
    /// Borrows the original component provenance.
    pub const fn origin(&self) -> &CssValueOrigin {
        self.component.origin()
    }
}

/// A checked angle or the literal number-zero exception selected by a consumer grammar.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssAngleOrZero {
    Angle(CssNumeric<CssAngle>),
    Zero(CssZeroLiteral),
}
impl CssAngleOrZero {
    /// Borrows the selected branch's original provenance.
    pub fn origin(&self) -> &CssValueOrigin {
        match self {
            Self::Angle(value) => value.origin(),
            Self::Zero(value) => value.origin(),
        }
    }
    pub(crate) fn from_parser_component(
        component: CssComponentValue,
        context: &crate::numeric::NumericInputContext<'_>,
    ) -> Result<Self, CssNumericConstructionError> {
        if matches!(
            component.view(),
            CssComponentValueRef::Token(CssValueTokenRef::Number(_))
        ) {
            return CssZeroLiteral::try_from_component(component)
                .map(Self::Zero)
                .map_err(CssNumericConstructionError::component);
        }
        CssNumeric::<CssAngle>::from_parser_component(component, context).map(Self::Angle)
    }
}
