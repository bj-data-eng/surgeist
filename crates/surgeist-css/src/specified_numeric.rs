//! Exact, checked authored number, length, and percentage domains with shared numeric helpers.

use crate::{
    CssComponentValue, CssComponentValueRef, CssFlexCalculation, CssLengthCalculation,
    CssLengthPercentageCalculation, CssLengthUnit, CssNumberCalculation,
    CssNumericConstructionError, CssNumericConstructionErrorKind, CssPercentageCalculation,
    CssSpecifiedValueSerializationError, CssSpecifiedValueSerializationErrorKind,
    CssSpecifiedValueSerializationLimits, CssValueOrigin, CssValueTokenRef,
    specified_serialization::{SpecifiedSerializationContext, format_lexical_shift},
};

type ConstructionResult<T> = Result<T, CssNumericConstructionError>;
type SerializationResult<T> = Result<T, CssSpecifiedValueSerializationError>;

/// A nonnegative Grid flex breadth retaining an exact `fr` token or checked flex math.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssSpecifiedNonNegativeFlex {
    value: SpecifiedNumericValue<CssFlexCalculation>,
}

impl CssSpecifiedNonNegativeFlex {
    pub fn try_from_component(component: CssComponentValue) -> ConstructionResult<Self> {
        let CssComponentValueRef::Token(CssValueTokenRef::Dimension { number, unit }) =
            component.view()
        else {
            return Err(CssNumericConstructionError::at(
                CssNumericConstructionErrorKind::RootDomainMismatch,
                Some(&component),
            ));
        };
        if !unit.eq_ignore_ascii_case("fr") {
            return Err(CssNumericConstructionError::at(
                CssNumericConstructionErrorKind::RootDomainMismatch,
                Some(&component),
            ));
        }
        if crate::exact_decimal::LexicalDecimal::new(number.representation()).negative {
            return Err(CssNumericConstructionError::at(
                CssNumericConstructionErrorKind::OutOfRange,
                Some(&component),
            ));
        }
        Ok(Self {
            value: SpecifiedNumericValue::Literal(Box::new(component)),
        })
    }

    pub fn try_from_calculation(calculation: CssFlexCalculation) -> ConstructionResult<Self> {
        let root = significant_root(calculation.components())?;
        if matches!(root.view(), CssComponentValueRef::Token(_)) {
            return Self::try_from_component(root.clone());
        }
        if !matches!(root.view(), CssComponentValueRef::Function(_)) {
            return Err(CssNumericConstructionError::at(
                CssNumericConstructionErrorKind::RootDomainMismatch,
                Some(root),
            ));
        }
        Ok(Self {
            value: SpecifiedNumericValue::Calculation(calculation),
        })
    }

    pub fn literal_component(&self) -> Option<&CssComponentValue> {
        match &self.value {
            SpecifiedNumericValue::Literal(value) => Some(value),
            SpecifiedNumericValue::Calculation(_) => None,
        }
    }

    pub fn calculation(&self) -> Option<&CssFlexCalculation> {
        match &self.value {
            SpecifiedNumericValue::Literal(_) => None,
            SpecifiedNumericValue::Calculation(value) => Some(value),
        }
    }

    pub fn origin(&self) -> &CssValueOrigin {
        match &self.value {
            SpecifiedNumericValue::Literal(value) => value.origin(),
            SpecifiedNumericValue::Calculation(value) => value.origin(),
        }
    }

    pub fn serialize_specified(&self) -> SerializationResult<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> SerializationResult<String> {
        let mut context = SpecifiedSerializationContext::new(limits);
        let captured = self.capture_specified(&mut context)?;
        let mut output = String::new();
        context.append(&mut output, &captured)?;
        Ok(output)
    }

    pub(crate) fn capture_specified(
        &self,
        context: &mut SpecifiedSerializationContext,
    ) -> SerializationResult<String> {
        match &self.value {
            SpecifiedNumericValue::Literal(value) => {
                context.charge_input(1)?;
                context.charge_projection(1)?;
                let CssComponentValueRef::Token(CssValueTokenRef::Dimension { number, .. }) =
                    value.view()
                else {
                    unreachable!("checked fr token")
                };
                let limit = context.remaining_bytes().checked_sub(2).ok_or_else(|| {
                    CssSpecifiedValueSerializationError::new(
                        CssSpecifiedValueSerializationErrorKind::ByteLimit,
                    )
                })?;
                let mut output = format_lexical_shift(number.representation(), 0, limit)?;
                context.append_temporary(&mut output, "fr")?;
                Ok(output)
            }
            SpecifiedNumericValue::Calculation(value) => {
                crate::numeric::capture_specified(&value.expression, context).map(|(text, _)| text)
            }
        }
    }
}

pub(crate) fn significant_root(
    calculation: &crate::CssComponentValues,
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

fn checked_literal(
    component: &CssComponentValue,
    allow_percentage: bool,
    nonnegative: bool,
    percentage_only: bool,
) -> ConstructionResult<()> {
    let (number, unit) = match component.view() {
        CssComponentValueRef::Token(CssValueTokenRef::Number(number)) if !percentage_only => {
            (number, LiteralUnit::Unitless)
        }
        CssComponentValueRef::Token(CssValueTokenRef::Dimension { number, unit })
            if !percentage_only =>
        {
            CssLengthUnit::from_css_unit(unit).ok_or_else(|| {
                CssNumericConstructionError::at(
                    CssNumericConstructionErrorKind::RootDomainMismatch,
                    Some(component),
                )
            })?;
            (number, LiteralUnit::Length)
        }
        CssComponentValueRef::Token(CssValueTokenRef::Percentage(number)) if allow_percentage => {
            (number, LiteralUnit::Percentage)
        }
        _ => {
            return Err(CssNumericConstructionError::at(
                CssNumericConstructionErrorKind::RootDomainMismatch,
                Some(component),
            ));
        }
    };
    let decimal = crate::exact_decimal::LexicalDecimal::new(number.representation());
    if matches!(unit, LiteralUnit::Unitless) && decimal.len != 0 {
        return Err(CssNumericConstructionError::at(
            CssNumericConstructionErrorKind::RootDomainMismatch,
            Some(component),
        ));
    }
    if nonnegative && decimal.negative {
        return Err(CssNumericConstructionError::at(
            CssNumericConstructionErrorKind::OutOfRange,
            Some(component),
        ));
    }
    Ok(())
}

enum LiteralUnit {
    Unitless,
    Length,
    Percentage,
}

fn visit_literal<'a>(
    component: &'a CssComponentValue,
    context: &mut SpecifiedSerializationContext,
) -> SerializationResult<(crate::CssNumericTokenRef<'a>, &'static str)> {
    context.charge_input(1)?;
    context.charge_projection(1)?;
    let parts = match component.view() {
        CssComponentValueRef::Token(CssValueTokenRef::Number(number)) => (number, ""),
        CssComponentValueRef::Token(CssValueTokenRef::Dimension { number, unit }) => (
            number,
            CssLengthUnit::from_css_unit(unit)
                .expect("checked length unit")
                .as_css_str(),
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
    let coefficient_limit = context
        .remaining_bytes()
        .checked_sub(suffix.len())
        .ok_or_else(|| {
            CssSpecifiedValueSerializationError::new(
                CssSpecifiedValueSerializationErrorKind::ByteLimit,
            )
        })?;
    let mut output = format_lexical_shift(number.representation(), 0, coefficient_limit)?;
    context.append_temporary(&mut output, suffix)?;
    Ok(output)
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum SpecifiedNumericValue<C> {
    Literal(Box<CssComponentValue>),
    Calculation(C),
}

/// A signed CSS `<number>` retaining its exact token or symbolic math.
/// Equality includes source provenance; semantic owners compare structure.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssSpecifiedNumber {
    value: SpecifiedNumericValue<CssNumberCalculation>,
}

impl CssSpecifiedNumber {
    /// Accepts an ordinary number token without narrowing to a machine float.
    pub fn try_from_component(component: CssComponentValue) -> ConstructionResult<Self> {
        if !matches!(
            component.view(),
            CssComponentValueRef::Token(CssValueTokenRef::Number(_))
        ) {
            return Err(CssNumericConstructionError::at(
                CssNumericConstructionErrorKind::RootDomainMismatch,
                Some(&component),
            ));
        }
        Ok(Self {
            value: SpecifiedNumericValue::Literal(Box::new(component)),
        })
    }

    /// Retains checked number math; a bare root reenters literal admission.
    pub fn try_from_calculation(calculation: CssNumberCalculation) -> ConstructionResult<Self> {
        let root = significant_root(calculation.components())?;
        if matches!(root.view(), CssComponentValueRef::Token(_)) {
            return Self::try_from_component(root.clone());
        }
        if !matches!(root.view(), CssComponentValueRef::Function(_)) {
            return Err(CssNumericConstructionError::at(
                CssNumericConstructionErrorKind::RootDomainMismatch,
                Some(root),
            ));
        }
        Ok(Self {
            value: SpecifiedNumericValue::Calculation(calculation),
        })
    }

    #[must_use]
    pub fn literal_component(&self) -> Option<&CssComponentValue> {
        match &self.value {
            SpecifiedNumericValue::Literal(value) => Some(value),
            SpecifiedNumericValue::Calculation(_) => None,
        }
    }

    #[must_use]
    pub fn calculation(&self) -> Option<&CssNumberCalculation> {
        match &self.value {
            SpecifiedNumericValue::Literal(_) => None,
            SpecifiedNumericValue::Calculation(value) => Some(value),
        }
    }

    #[must_use]
    pub fn origin(&self) -> &CssValueOrigin {
        match &self.value {
            SpecifiedNumericValue::Literal(value) => value.origin(),
            SpecifiedNumericValue::Calculation(value) => value.origin(),
        }
    }

    pub fn serialize_specified(&self) -> SerializationResult<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> SerializationResult<String> {
        let mut context = SpecifiedSerializationContext::new(limits);
        let captured = self.capture_specified(&mut context)?;
        let mut output = String::new();
        context.append(&mut output, &captured)?;
        Ok(output)
    }

    pub(crate) fn capture_specified(
        &self,
        context: &mut SpecifiedSerializationContext,
    ) -> SerializationResult<String> {
        match &self.value {
            SpecifiedNumericValue::Literal(value) => capture_literal(value, context),
            SpecifiedNumericValue::Calculation(value) => {
                crate::numeric::capture_specified(&value.expression, context).map(|(text, _)| text)
            }
        }
    }

    pub(crate) fn structural_eq(&self, other: &Self) -> bool {
        match (&self.value, &other.value) {
            (SpecifiedNumericValue::Literal(left), SpecifiedNumericValue::Literal(right)) => {
                left.structural_eq_ignoring_origin(right)
            }
            (
                SpecifiedNumericValue::Calculation(left),
                SpecifiedNumericValue::Calculation(right),
            ) => left.expression.structural_eq(&right.expression),
            _ => false,
        }
    }
}

/// A signed CSS `<percentage>` retaining its exact token or symbolic math.
/// Equality includes source provenance; semantic owners compare structure.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssSpecifiedPercentage {
    value: SpecifiedNumericValue<CssPercentageCalculation>,
}

impl CssSpecifiedPercentage {
    /// Accepts a percentage token without narrowing its coefficient to a machine float.
    pub fn try_from_component(component: CssComponentValue) -> ConstructionResult<Self> {
        checked_literal(&component, true, false, true)?;
        Ok(Self {
            value: SpecifiedNumericValue::Literal(Box::new(component)),
        })
    }

    /// Retains checked percentage math; a bare root reenters literal admission.
    pub fn try_from_calculation(calculation: CssPercentageCalculation) -> ConstructionResult<Self> {
        let root = significant_root(calculation.components())?;
        if matches!(root.view(), CssComponentValueRef::Token(_)) {
            return Self::try_from_component(root.clone());
        }
        if !matches!(root.view(), CssComponentValueRef::Function(_)) {
            return Err(CssNumericConstructionError::at(
                CssNumericConstructionErrorKind::RootDomainMismatch,
                Some(root),
            ));
        }
        Ok(Self {
            value: SpecifiedNumericValue::Calculation(calculation),
        })
    }

    /// Borrows the original ordinary percentage token, if present.
    #[must_use]
    pub fn literal_component(&self) -> Option<&CssComponentValue> {
        match &self.value {
            SpecifiedNumericValue::Literal(value) => Some(value),
            SpecifiedNumericValue::Calculation(_) => None,
        }
    }

    /// Borrows the symbolic checked percentage calculation, if present.
    #[must_use]
    pub fn calculation(&self) -> Option<&CssPercentageCalculation> {
        match &self.value {
            SpecifiedNumericValue::Literal(_) => None,
            SpecifiedNumericValue::Calculation(value) => Some(value),
        }
    }

    /// Returns the parsed or programmatic root origin.
    #[must_use]
    pub fn origin(&self) -> &CssValueOrigin {
        match &self.value {
            SpecifiedNumericValue::Literal(value) => value.origin(),
            SpecifiedNumericValue::Calculation(value) => value.origin(),
        }
    }

    /// Serializes the specified percentage with default resource limits.
    pub fn serialize_specified(&self) -> SerializationResult<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    /// Serializes atomically under the shared input, projection, and byte limits.
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> SerializationResult<String> {
        let mut context = SpecifiedSerializationContext::new(limits);
        let captured = self.capture_specified(&mut context)?;
        let mut output = String::new();
        context.append(&mut output, &captured)?;
        Ok(output)
    }

    pub(crate) fn capture_specified(
        &self,
        context: &mut SpecifiedSerializationContext,
    ) -> SerializationResult<String> {
        match &self.value {
            SpecifiedNumericValue::Literal(value) => capture_literal(value, context),
            SpecifiedNumericValue::Calculation(value) => {
                crate::numeric::capture_specified(&value.expression, context).map(|(text, _)| text)
            }
        }
    }

    pub(crate) fn structural_eq(&self, other: &Self) -> bool {
        match (&self.value, &other.value) {
            (SpecifiedNumericValue::Literal(left), SpecifiedNumericValue::Literal(right)) => {
                left.structural_eq_ignoring_origin(right)
            }
            (
                SpecifiedNumericValue::Calculation(left),
                SpecifiedNumericValue::Calculation(right),
            ) => left.expression.structural_eq(&right.expression),
            _ => false,
        }
    }
}

/// A nonnegative CSS `<number>` retaining its exact token or symbolic math.
/// Equality includes source provenance; semantic owners can compare structure separately.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssSpecifiedNonNegativeNumber {
    value: SpecifiedNumericValue<CssNumberCalculation>,
}

impl CssSpecifiedNonNegativeNumber {
    /// Checks an ordinary number exactly, including signed zero and tiny negatives.
    pub fn try_from_component(component: CssComponentValue) -> ConstructionResult<Self> {
        let CssComponentValueRef::Token(CssValueTokenRef::Number(number)) = component.view() else {
            return Err(CssNumericConstructionError::at(
                CssNumericConstructionErrorKind::RootDomainMismatch,
                Some(&component),
            ));
        };
        if crate::exact_decimal::LexicalDecimal::new(number.representation()).negative {
            return Err(CssNumericConstructionError::at(
                CssNumericConstructionErrorKind::OutOfRange,
                Some(&component),
            ));
        }
        Ok(Self {
            value: SpecifiedNumericValue::Literal(Box::new(component)),
        })
    }

    /// Retains admitted number math; a bare root re-enters exact literal checks.
    pub fn try_from_calculation(calculation: CssNumberCalculation) -> ConstructionResult<Self> {
        let root = significant_root(calculation.components())?;
        if matches!(root.view(), CssComponentValueRef::Token(_)) {
            return Self::try_from_component(root.clone());
        }
        if !matches!(root.view(), CssComponentValueRef::Function(_)) {
            return Err(CssNumericConstructionError::at(
                CssNumericConstructionErrorKind::RootDomainMismatch,
                Some(root),
            ));
        }
        Ok(Self {
            value: SpecifiedNumericValue::Calculation(calculation),
        })
    }

    /// Borrows the original ordinary number token, if present.
    pub fn literal_component(&self) -> Option<&CssComponentValue> {
        match &self.value {
            SpecifiedNumericValue::Literal(value) => Some(value),
            _ => None,
        }
    }

    /// Borrows the symbolic checked number calculation, if present.
    pub fn calculation(&self) -> Option<&CssNumberCalculation> {
        match &self.value {
            SpecifiedNumericValue::Calculation(value) => Some(value),
            _ => None,
        }
    }

    /// Returns the parsed or programmatic root origin.
    pub fn origin(&self) -> &CssValueOrigin {
        match &self.value {
            SpecifiedNumericValue::Literal(value) => value.origin(),
            SpecifiedNumericValue::Calculation(value) => value.origin(),
        }
    }

    /// Serializes the specified number with default resource limits.
    pub fn serialize_specified(&self) -> SerializationResult<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    /// Serializes the specified number atomically under explicit resource limits.
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> SerializationResult<String> {
        let mut context = SpecifiedSerializationContext::new(limits);
        let captured = self.capture_specified(&mut context)?;
        let mut output = String::new();
        context.append(&mut output, &captured)?;
        Ok(output)
    }

    pub(crate) fn capture_specified(
        &self,
        context: &mut SpecifiedSerializationContext,
    ) -> SerializationResult<String> {
        match &self.value {
            SpecifiedNumericValue::Literal(value) => capture_literal(value, context),
            SpecifiedNumericValue::Calculation(value) => {
                crate::numeric::capture_specified(&value.expression, context).map(|(text, _)| text)
            }
        }
    }

    pub(crate) fn structural_eq(&self, other: &Self) -> bool {
        match (&self.value, &other.value) {
            (SpecifiedNumericValue::Literal(left), SpecifiedNumericValue::Literal(right)) => {
                left.structural_eq_ignoring_origin(right)
            }
            (
                SpecifiedNumericValue::Calculation(left),
                SpecifiedNumericValue::Calculation(right),
            ) => left.expression.structural_eq(&right.expression),
            _ => false,
        }
    }
}

/// A signed CSS `<length>` that retains exact ordinary spelling or deferred math.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssSpecifiedLength {
    value: SpecifiedNumericValue<CssLengthCalculation>,
}

impl CssSpecifiedLength {
    pub(crate) fn structural_eq(&self, other: &Self) -> bool {
        match (&self.value, &other.value) {
            (SpecifiedNumericValue::Literal(left), SpecifiedNumericValue::Literal(right)) => {
                left.structural_eq_ignoring_origin(right)
            }
            (
                SpecifiedNumericValue::Calculation(left),
                SpecifiedNumericValue::Calculation(right),
            ) => left.structural_eq(right),
            _ => false,
        }
    }

    /// Checks a literal length or exact unitless zero without floating-point conversion.
    pub fn try_from_component(component: CssComponentValue) -> ConstructionResult<Self> {
        checked_literal(&component, false, false, false)?;
        Ok(Self {
            value: SpecifiedNumericValue::Literal(Box::new(component)),
        })
    }

    /// Retains checked length math; a bare numeric root re-enters literal admission.
    pub fn try_from_calculation(calculation: CssLengthCalculation) -> ConstructionResult<Self> {
        let root = significant_root(calculation.components())?;
        if matches!(root.view(), CssComponentValueRef::Token(_)) {
            return Self::try_from_component(root.clone());
        }
        if !matches!(root.view(), CssComponentValueRef::Function(_)) {
            return Err(CssNumericConstructionError::at(
                CssNumericConstructionErrorKind::RootDomainMismatch,
                Some(root),
            ));
        }
        Ok(Self {
            value: SpecifiedNumericValue::Calculation(calculation),
        })
    }

    /// Constructs the exact unitless-zero initial length.
    #[must_use]
    pub fn zero() -> Self {
        Self::try_from_component(CssComponentValue::try_number("0").expect("zero token"))
            .expect("zero length")
    }

    /// Borrows the original ordinary token, when this is a literal.
    pub fn literal_component(&self) -> Option<&CssComponentValue> {
        match &self.value {
            SpecifiedNumericValue::Literal(component) => Some(component),
            SpecifiedNumericValue::Calculation(_) => None,
        }
    }

    /// Borrows the symbolic checked math root, when present.
    pub fn calculation(&self) -> Option<&CssLengthCalculation> {
        match &self.value {
            SpecifiedNumericValue::Literal(_) => None,
            SpecifiedNumericValue::Calculation(calculation) => Some(calculation),
        }
    }

    /// Returns the original parsed or programmatic root origin.
    pub fn origin(&self) -> &CssValueOrigin {
        match &self.value {
            SpecifiedNumericValue::Literal(component) => component.origin(),
            SpecifiedNumericValue::Calculation(calculation) => calculation.origin(),
        }
    }

    /// Serializes the canonical specified length with default resource limits.
    pub fn serialize_specified(&self) -> SerializationResult<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    /// Serializes the canonical specified length under explicit resource limits.
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> SerializationResult<String> {
        let mut context = SpecifiedSerializationContext::new(limits);
        let captured = self.capture_specified(&mut context)?;
        let mut output = String::new();
        context.append(&mut output, &captured)?;
        Ok(output)
    }

    pub(crate) fn capture_specified(
        &self,
        context: &mut SpecifiedSerializationContext,
    ) -> SerializationResult<String> {
        match &self.value {
            SpecifiedNumericValue::Literal(component) => capture_literal(component, context),
            SpecifiedNumericValue::Calculation(calculation) => {
                crate::numeric::capture_specified(&calculation.expression, context)
                    .map(|(text, _)| text)
            }
        }
    }
}

/// A nonnegative CSS `<length>` retaining exact ordinary spelling or deferred math.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssSpecifiedNonNegativeLength {
    value: SpecifiedNumericValue<CssLengthCalculation>,
}

impl CssSpecifiedNonNegativeLength {
    /// Checks a literal length or exact unitless zero without floating-point conversion.
    pub fn try_from_component(component: CssComponentValue) -> ConstructionResult<Self> {
        checked_literal(&component, false, true, false)?;
        Ok(Self {
            value: SpecifiedNumericValue::Literal(Box::new(component)),
        })
    }

    /// Retains checked length math; a bare numeric root re-enters literal admission.
    pub fn try_from_calculation(calculation: CssLengthCalculation) -> ConstructionResult<Self> {
        let root = significant_root(calculation.components())?;
        if matches!(root.view(), CssComponentValueRef::Token(_)) {
            return Self::try_from_component(root.clone());
        }
        if !matches!(root.view(), CssComponentValueRef::Function(_)) {
            return Err(CssNumericConstructionError::at(
                CssNumericConstructionErrorKind::RootDomainMismatch,
                Some(root),
            ));
        }
        Ok(Self {
            value: SpecifiedNumericValue::Calculation(calculation),
        })
    }

    /// Constructs the exact unitless-zero initial length.
    #[must_use]
    pub fn zero() -> Self {
        Self::try_from_component(CssComponentValue::try_number("0").expect("zero token"))
            .expect("zero length")
    }

    /// Borrows the original ordinary token, when this is a literal.
    pub fn literal_component(&self) -> Option<&CssComponentValue> {
        match &self.value {
            SpecifiedNumericValue::Literal(component) => Some(component),
            SpecifiedNumericValue::Calculation(_) => None,
        }
    }

    /// Borrows the symbolic checked math root, when present.
    pub fn calculation(&self) -> Option<&CssLengthCalculation> {
        match &self.value {
            SpecifiedNumericValue::Literal(_) => None,
            SpecifiedNumericValue::Calculation(calculation) => Some(calculation),
        }
    }

    /// Returns the original parsed or programmatic root origin.
    pub fn origin(&self) -> &CssValueOrigin {
        match &self.value {
            SpecifiedNumericValue::Literal(component) => component.origin(),
            SpecifiedNumericValue::Calculation(calculation) => calculation.origin(),
        }
    }

    /// Serializes the canonical specified length with default resource limits.
    pub fn serialize_specified(&self) -> SerializationResult<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    /// Serializes the canonical specified length under explicit resource limits.
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> SerializationResult<String> {
        let mut context = SpecifiedSerializationContext::new(limits);
        let captured = self.capture_specified(&mut context)?;
        let mut output = String::new();
        context.append(&mut output, &captured)?;
        Ok(output)
    }

    pub(crate) fn capture_specified(
        &self,
        context: &mut SpecifiedSerializationContext,
    ) -> SerializationResult<String> {
        match &self.value {
            SpecifiedNumericValue::Literal(component) => capture_literal(component, context),
            SpecifiedNumericValue::Calculation(calculation) => {
                crate::numeric::capture_specified(&calculation.expression, context)
                    .map(|(text, _)| text)
            }
        }
    }

    pub(crate) fn structural_eq(&self, other: &Self) -> bool {
        match (&self.value, &other.value) {
            (SpecifiedNumericValue::Literal(left), SpecifiedNumericValue::Literal(right)) => {
                left.structural_eq_ignoring_origin(right)
            }
            (
                SpecifiedNumericValue::Calculation(left),
                SpecifiedNumericValue::Calculation(right),
            ) => left.structural_eq(right),
            _ => false,
        }
    }
}

/// A signed CSS `<length-percentage>` retaining exact ordinary spelling or deferred math.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssSpecifiedLengthPercentage {
    value: SpecifiedNumericValue<CssLengthPercentageCalculation>,
}

impl CssSpecifiedLengthPercentage {
    /// Checks a literal length, percentage, or exact unitless zero without floating-point conversion.
    pub fn try_from_component(component: CssComponentValue) -> ConstructionResult<Self> {
        checked_literal(&component, true, false, false)?;
        Ok(Self {
            value: SpecifiedNumericValue::Literal(Box::new(component)),
        })
    }

    /// Retains checked deferred math; bare numeric roots re-enter literal admission.
    pub fn try_from_calculation(
        calculation: CssLengthPercentageCalculation,
    ) -> ConstructionResult<Self> {
        let root = significant_root(calculation.components())?;
        if matches!(root.view(), CssComponentValueRef::Token(_)) {
            return Self::try_from_component(root.clone());
        }
        if !matches!(root.view(), CssComponentValueRef::Function(_)) {
            return Err(CssNumericConstructionError::at(
                CssNumericConstructionErrorKind::RootDomainMismatch,
                Some(root),
            ));
        }
        Ok(Self {
            value: SpecifiedNumericValue::Calculation(calculation),
        })
    }

    /// Constructs the exact unitless-zero initial length-percentage.
    #[must_use]
    pub fn zero() -> Self {
        Self::try_from_component(CssComponentValue::try_number("0").expect("zero token"))
            .expect("zero length-percentage")
    }

    /// Borrows the ordinary token, when this is a literal.
    pub fn literal_component(&self) -> Option<&CssComponentValue> {
        match &self.value {
            SpecifiedNumericValue::Literal(component) => Some(component),
            SpecifiedNumericValue::Calculation(_) => None,
        }
    }

    /// Borrows the symbolic checked math root, when present.
    pub fn calculation(&self) -> Option<&CssLengthPercentageCalculation> {
        match &self.value {
            SpecifiedNumericValue::Literal(_) => None,
            SpecifiedNumericValue::Calculation(calculation) => Some(calculation),
        }
    }

    /// Returns the original parsed or programmatic root origin.
    pub fn origin(&self) -> &CssValueOrigin {
        match &self.value {
            SpecifiedNumericValue::Literal(component) => component.origin(),
            SpecifiedNumericValue::Calculation(calculation) => calculation.origin(),
        }
    }

    /// Serializes the canonical specified value with default resource limits.
    pub fn serialize_specified(&self) -> SerializationResult<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    /// Serializes the canonical specified value under explicit resource limits.
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> SerializationResult<String> {
        let mut context = SpecifiedSerializationContext::new(limits);
        let captured = self.capture_specified(&mut context)?;
        let mut output = String::new();
        context.append(&mut output, &captured)?;
        Ok(output)
    }

    pub(crate) fn capture_specified(
        &self,
        context: &mut SpecifiedSerializationContext,
    ) -> SerializationResult<String> {
        match &self.value {
            SpecifiedNumericValue::Literal(component) => capture_literal(component, context),
            SpecifiedNumericValue::Calculation(calculation) => {
                crate::numeric::capture_specified(&calculation.expression, context)
                    .map(|(text, _)| text)
            }
        }
    }

    pub(crate) fn structural_eq(&self, other: &Self) -> bool {
        match (&self.value, &other.value) {
            (SpecifiedNumericValue::Literal(left), SpecifiedNumericValue::Literal(right)) => {
                left.structural_eq_ignoring_origin(right)
            }
            (
                SpecifiedNumericValue::Calculation(left),
                SpecifiedNumericValue::Calculation(right),
            ) => left.structural_eq(right),
            _ => false,
        }
    }
}

/// A nonnegative ordinary `<length-percentage>` or deferred checked math.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssSpecifiedNonNegativeLengthPercentage {
    value: SpecifiedNumericValue<CssLengthPercentageCalculation>,
}

impl CssSpecifiedNonNegativeLengthPercentage {
    /// Checks an ordinary value exactly, including negative values too small for `f32`.
    pub fn try_from_component(component: CssComponentValue) -> ConstructionResult<Self> {
        checked_literal(&component, true, true, false)?;
        Ok(Self {
            value: SpecifiedNumericValue::Literal(Box::new(component)),
        })
    }

    /// Retains deferred math; bare numeric roots re-enter ordinary range checks.
    pub fn try_from_calculation(
        calculation: CssLengthPercentageCalculation,
    ) -> ConstructionResult<Self> {
        let root = significant_root(calculation.components())?;
        if matches!(root.view(), CssComponentValueRef::Token(_)) {
            return Self::try_from_component(root.clone());
        }
        if !matches!(root.view(), CssComponentValueRef::Function(_)) {
            return Err(CssNumericConstructionError::at(
                CssNumericConstructionErrorKind::RootDomainMismatch,
                Some(root),
            ));
        }
        Ok(Self {
            value: SpecifiedNumericValue::Calculation(calculation),
        })
    }

    /// Constructs the exact unitless-zero initial length-percentage.
    #[must_use]
    pub fn zero() -> Self {
        Self::try_from_component(CssComponentValue::try_number("0").expect("zero token"))
            .expect("zero length-percentage")
    }

    /// Borrows the original ordinary token, when this is a literal.
    pub fn literal_component(&self) -> Option<&CssComponentValue> {
        match &self.value {
            SpecifiedNumericValue::Literal(component) => Some(component),
            SpecifiedNumericValue::Calculation(_) => None,
        }
    }

    /// Borrows the symbolic checked math root, when present.
    pub fn calculation(&self) -> Option<&CssLengthPercentageCalculation> {
        match &self.value {
            SpecifiedNumericValue::Literal(_) => None,
            SpecifiedNumericValue::Calculation(calculation) => Some(calculation),
        }
    }

    /// Returns the original parsed or programmatic root origin.
    pub fn origin(&self) -> &CssValueOrigin {
        match &self.value {
            SpecifiedNumericValue::Literal(component) => component.origin(),
            SpecifiedNumericValue::Calculation(calculation) => calculation.origin(),
        }
    }

    /// Serializes the canonical specified value with default resource limits.
    pub fn serialize_specified(&self) -> SerializationResult<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    /// Serializes the canonical specified value under explicit resource limits.
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> SerializationResult<String> {
        let mut context = SpecifiedSerializationContext::new(limits);
        let captured = self.capture_specified(&mut context)?;
        let mut output = String::new();
        context.append(&mut output, &captured)?;
        Ok(output)
    }

    pub(crate) fn capture_specified(
        &self,
        context: &mut SpecifiedSerializationContext,
    ) -> SerializationResult<String> {
        match &self.value {
            SpecifiedNumericValue::Literal(component) => capture_literal(component, context),
            SpecifiedNumericValue::Calculation(calculation) => {
                crate::numeric::capture_specified(&calculation.expression, context)
                    .map(|(text, _)| text)
            }
        }
    }

    pub(crate) fn structural_eq(&self, other: &Self) -> bool {
        match (&self.value, &other.value) {
            (SpecifiedNumericValue::Literal(left), SpecifiedNumericValue::Literal(right)) => {
                left.structural_eq_ignoring_origin(right)
            }
            (
                SpecifiedNumericValue::Calculation(left),
                SpecifiedNumericValue::Calculation(right),
            ) => left.structural_eq(right),
            _ => false,
        }
    }
}

/// A nonnegative CSS `<percentage>` retaining exact literal spelling or checked math.
#[derive(Clone, Debug)]
pub struct CssSpecifiedNonNegativePercentage {
    value: SpecifiedNumericValue<CssPercentageCalculation>,
}

impl PartialEq for CssSpecifiedNonNegativePercentage {
    fn eq(&self, other: &Self) -> bool {
        match (&self.value, &other.value) {
            (SpecifiedNumericValue::Literal(left), SpecifiedNumericValue::Literal(right)) => {
                left.structural_eq_ignoring_origin(right)
            }
            (
                SpecifiedNumericValue::Calculation(left),
                SpecifiedNumericValue::Calculation(right),
            ) => left.expression.structural_eq(&right.expression),
            _ => false,
        }
    }
}

impl Eq for CssSpecifiedNonNegativePercentage {}

impl CssSpecifiedNonNegativePercentage {
    /// Checks a percentage token exactly, including negative zero and tiny negatives.
    pub fn try_from_component(component: CssComponentValue) -> ConstructionResult<Self> {
        checked_literal(&component, true, true, true)?;
        Ok(Self {
            value: SpecifiedNumericValue::Literal(Box::new(component)),
        })
    }

    /// Retains checked percentage math; a bare token re-enters literal validation.
    pub fn try_from_calculation(calculation: CssPercentageCalculation) -> ConstructionResult<Self> {
        let root = significant_root(calculation.components())?;
        if matches!(root.view(), CssComponentValueRef::Token(_)) {
            return Self::try_from_component(root.clone());
        }
        if !matches!(root.view(), CssComponentValueRef::Function(_)) {
            return Err(CssNumericConstructionError::at(
                CssNumericConstructionErrorKind::RootDomainMismatch,
                Some(root),
            ));
        }
        Ok(Self {
            value: SpecifiedNumericValue::Calculation(calculation),
        })
    }

    /// Borrows the exact literal token, if this is an ordinary percentage.
    pub fn literal_component(&self) -> Option<&CssComponentValue> {
        match &self.value {
            SpecifiedNumericValue::Literal(component) => Some(component),
            SpecifiedNumericValue::Calculation(_) => None,
        }
    }

    /// Borrows checked symbolic percentage math, when present.
    pub fn calculation(&self) -> Option<&CssPercentageCalculation> {
        match &self.value {
            SpecifiedNumericValue::Literal(_) => None,
            SpecifiedNumericValue::Calculation(calculation) => Some(calculation),
        }
    }

    /// Returns the root's parsed or programmatic origin.
    pub fn origin(&self) -> &CssValueOrigin {
        match &self.value {
            SpecifiedNumericValue::Literal(component) => component.origin(),
            SpecifiedNumericValue::Calculation(calculation) => calculation.origin(),
        }
    }

    /// Serializes the canonical specified percentage with default limits.
    pub fn serialize_specified(&self) -> SerializationResult<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    /// Serializes atomically under shared input, projection, and byte budgets.
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> SerializationResult<String> {
        let mut context = SpecifiedSerializationContext::new(limits);
        let captured = self.capture_specified(&mut context)?;
        let mut output = String::new();
        context.append(&mut output, &captured)?;
        Ok(output)
    }

    pub(crate) fn capture_specified(
        &self,
        context: &mut SpecifiedSerializationContext,
    ) -> SerializationResult<String> {
        match &self.value {
            SpecifiedNumericValue::Literal(component) => capture_literal(component, context),
            SpecifiedNumericValue::Calculation(calculation) => {
                crate::numeric::capture_specified(&calculation.expression, context)
                    .map(|(text, _)| text)
            }
        }
    }
}

// Enclosing syntax owners use the same cumulative projection and output budget.
macro_rules! append_checked_length {
    ($($owner:ident),* $(,)?) => { $(
        impl $owner {
            pub(crate) fn append_specified(&self, context: &mut SpecifiedSerializationContext, output: &mut String) -> SerializationResult<()> {
                if context.output_suppressed() {
                    // Suppression is selected only for independently proved literal initials.
                    let literal = self.literal_component().expect("suppressed simple literal");
                    visit_literal(literal, context)?;
                    return Ok(());
                }
                let captured = self.capture_specified(context)?;
                context.append(output, &captured)
            }
        }
    )* };
}
append_checked_length!(
    CssSpecifiedLength,
    CssSpecifiedNonNegativeLength,
    CssSpecifiedLengthPercentage,
    CssSpecifiedNonNegativeLengthPercentage
);
