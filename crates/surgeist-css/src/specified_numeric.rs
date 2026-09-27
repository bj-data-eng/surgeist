//! Exact, checked authored length and percentage domains for CSS properties.

use crate::{
    CssComponentValue, CssComponentValueRef, CssLengthCalculation, CssLengthPercentageCalculation,
    CssLengthUnit, CssNumericConstructionError, CssNumericConstructionErrorKind,
    CssPercentageCalculation, CssSpecifiedValueSerializationError,
    CssSpecifiedValueSerializationErrorKind, CssSpecifiedValueSerializationLimits, CssValueOrigin,
    CssValueTokenRef,
    specified_serialization::{SpecifiedSerializationContext, format_lexical_shift},
};

type ConstructionResult<T> = Result<T, CssNumericConstructionError>;
type SerializationResult<T> = Result<T, CssSpecifiedValueSerializationError>;

fn significant_root(
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

fn capture_literal(
    component: &CssComponentValue,
    context: &mut SpecifiedSerializationContext,
) -> SerializationResult<String> {
    context.charge_input(1)?;
    context.charge_projection(1)?;
    let (number, suffix) = match component.view() {
        CssComponentValueRef::Token(CssValueTokenRef::Number(number)) => (number, ""),
        CssComponentValueRef::Token(CssValueTokenRef::Dimension { number, unit }) => (
            number,
            CssLengthUnit::from_css_unit(unit)
                .expect("checked length unit")
                .as_css_str(),
        ),
        CssComponentValueRef::Token(CssValueTokenRef::Percentage(number)) => (number, "%"),
        _ => unreachable!("checked specified length literal"),
    };
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

/// A signed CSS `<length>` that retains exact ordinary spelling or deferred math.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssSpecifiedLength {
    value: SpecifiedLengthValue<CssLengthCalculation>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum SpecifiedLengthValue<C> {
    Literal(Box<CssComponentValue>),
    Calculation(C),
}

impl CssSpecifiedLength {
    /// Checks a literal length or exact unitless zero without floating-point conversion.
    pub fn try_from_component(component: CssComponentValue) -> ConstructionResult<Self> {
        checked_literal(&component, false, false, false)?;
        Ok(Self {
            value: SpecifiedLengthValue::Literal(Box::new(component)),
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
            value: SpecifiedLengthValue::Calculation(calculation),
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
            SpecifiedLengthValue::Literal(component) => Some(component),
            SpecifiedLengthValue::Calculation(_) => None,
        }
    }

    /// Borrows the symbolic checked math root, when present.
    pub fn calculation(&self) -> Option<&CssLengthCalculation> {
        match &self.value {
            SpecifiedLengthValue::Literal(_) => None,
            SpecifiedLengthValue::Calculation(calculation) => Some(calculation),
        }
    }

    /// Returns the original parsed or programmatic root origin.
    pub fn origin(&self) -> &CssValueOrigin {
        match &self.value {
            SpecifiedLengthValue::Literal(component) => component.origin(),
            SpecifiedLengthValue::Calculation(calculation) => calculation.origin(),
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
            SpecifiedLengthValue::Literal(component) => capture_literal(component, context),
            SpecifiedLengthValue::Calculation(calculation) => {
                crate::numeric::capture_specified(&calculation.expression, context)
                    .map(|(text, _)| text)
            }
        }
    }
}

/// A nonnegative CSS `<length>` retaining exact ordinary spelling or deferred math.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssSpecifiedNonNegativeLength {
    value: SpecifiedLengthValue<CssLengthCalculation>,
}

impl CssSpecifiedNonNegativeLength {
    /// Checks a literal length or exact unitless zero without floating-point conversion.
    pub fn try_from_component(component: CssComponentValue) -> ConstructionResult<Self> {
        checked_literal(&component, false, true, false)?;
        Ok(Self {
            value: SpecifiedLengthValue::Literal(Box::new(component)),
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
            value: SpecifiedLengthValue::Calculation(calculation),
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
            SpecifiedLengthValue::Literal(component) => Some(component),
            SpecifiedLengthValue::Calculation(_) => None,
        }
    }

    /// Borrows the symbolic checked math root, when present.
    pub fn calculation(&self) -> Option<&CssLengthCalculation> {
        match &self.value {
            SpecifiedLengthValue::Literal(_) => None,
            SpecifiedLengthValue::Calculation(calculation) => Some(calculation),
        }
    }

    /// Returns the original parsed or programmatic root origin.
    pub fn origin(&self) -> &CssValueOrigin {
        match &self.value {
            SpecifiedLengthValue::Literal(component) => component.origin(),
            SpecifiedLengthValue::Calculation(calculation) => calculation.origin(),
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
            SpecifiedLengthValue::Literal(component) => capture_literal(component, context),
            SpecifiedLengthValue::Calculation(calculation) => {
                crate::numeric::capture_specified(&calculation.expression, context)
                    .map(|(text, _)| text)
            }
        }
    }

    pub(crate) fn structural_eq(&self, other: &Self) -> bool {
        match (&self.value, &other.value) {
            (SpecifiedLengthValue::Literal(left), SpecifiedLengthValue::Literal(right)) => {
                left.structural_eq_ignoring_origin(right)
            }
            (SpecifiedLengthValue::Calculation(left), SpecifiedLengthValue::Calculation(right)) => {
                left.structural_eq(right)
            }
            _ => false,
        }
    }
}

/// A signed CSS `<length-percentage>` retaining exact ordinary spelling or deferred math.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssSpecifiedLengthPercentage {
    value: SpecifiedLengthValue<CssLengthPercentageCalculation>,
}

impl CssSpecifiedLengthPercentage {
    /// Checks a literal length, percentage, or exact unitless zero without floating-point conversion.
    pub fn try_from_component(component: CssComponentValue) -> ConstructionResult<Self> {
        checked_literal(&component, true, false, false)?;
        Ok(Self {
            value: SpecifiedLengthValue::Literal(Box::new(component)),
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
            value: SpecifiedLengthValue::Calculation(calculation),
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
            SpecifiedLengthValue::Literal(component) => Some(component),
            SpecifiedLengthValue::Calculation(_) => None,
        }
    }

    /// Borrows the symbolic checked math root, when present.
    pub fn calculation(&self) -> Option<&CssLengthPercentageCalculation> {
        match &self.value {
            SpecifiedLengthValue::Literal(_) => None,
            SpecifiedLengthValue::Calculation(calculation) => Some(calculation),
        }
    }

    /// Returns the original parsed or programmatic root origin.
    pub fn origin(&self) -> &CssValueOrigin {
        match &self.value {
            SpecifiedLengthValue::Literal(component) => component.origin(),
            SpecifiedLengthValue::Calculation(calculation) => calculation.origin(),
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
            SpecifiedLengthValue::Literal(component) => capture_literal(component, context),
            SpecifiedLengthValue::Calculation(calculation) => {
                crate::numeric::capture_specified(&calculation.expression, context)
                    .map(|(text, _)| text)
            }
        }
    }

    pub(crate) fn structural_eq(&self, other: &Self) -> bool {
        match (&self.value, &other.value) {
            (SpecifiedLengthValue::Literal(left), SpecifiedLengthValue::Literal(right)) => {
                left.structural_eq_ignoring_origin(right)
            }
            (SpecifiedLengthValue::Calculation(left), SpecifiedLengthValue::Calculation(right)) => {
                left.structural_eq(right)
            }
            _ => false,
        }
    }
}

/// A nonnegative ordinary `<length-percentage>` or deferred checked math.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssSpecifiedNonNegativeLengthPercentage {
    value: SpecifiedLengthValue<CssLengthPercentageCalculation>,
}

impl CssSpecifiedNonNegativeLengthPercentage {
    /// Checks an ordinary value exactly, including negative values too small for `f32`.
    pub fn try_from_component(component: CssComponentValue) -> ConstructionResult<Self> {
        checked_literal(&component, true, true, false)?;
        Ok(Self {
            value: SpecifiedLengthValue::Literal(Box::new(component)),
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
            value: SpecifiedLengthValue::Calculation(calculation),
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
            SpecifiedLengthValue::Literal(component) => Some(component),
            SpecifiedLengthValue::Calculation(_) => None,
        }
    }

    /// Borrows the symbolic checked math root, when present.
    pub fn calculation(&self) -> Option<&CssLengthPercentageCalculation> {
        match &self.value {
            SpecifiedLengthValue::Literal(_) => None,
            SpecifiedLengthValue::Calculation(calculation) => Some(calculation),
        }
    }

    /// Returns the original parsed or programmatic root origin.
    pub fn origin(&self) -> &CssValueOrigin {
        match &self.value {
            SpecifiedLengthValue::Literal(component) => component.origin(),
            SpecifiedLengthValue::Calculation(calculation) => calculation.origin(),
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
            SpecifiedLengthValue::Literal(component) => capture_literal(component, context),
            SpecifiedLengthValue::Calculation(calculation) => {
                crate::numeric::capture_specified(&calculation.expression, context)
                    .map(|(text, _)| text)
            }
        }
    }

    pub(crate) fn structural_eq(&self, other: &Self) -> bool {
        match (&self.value, &other.value) {
            (SpecifiedLengthValue::Literal(left), SpecifiedLengthValue::Literal(right)) => {
                left.structural_eq_ignoring_origin(right)
            }
            (SpecifiedLengthValue::Calculation(left), SpecifiedLengthValue::Calculation(right)) => {
                left.structural_eq(right)
            }
            _ => false,
        }
    }
}

/// A nonnegative CSS `<percentage>` retaining exact literal spelling or checked math.
#[derive(Clone, Debug)]
pub struct CssSpecifiedNonNegativePercentage {
    value: SpecifiedLengthValue<CssPercentageCalculation>,
}

impl PartialEq for CssSpecifiedNonNegativePercentage {
    fn eq(&self, other: &Self) -> bool {
        match (&self.value, &other.value) {
            (SpecifiedLengthValue::Literal(left), SpecifiedLengthValue::Literal(right)) => {
                left.structural_eq_ignoring_origin(right)
            }
            (SpecifiedLengthValue::Calculation(left), SpecifiedLengthValue::Calculation(right)) => {
                left.expression.structural_eq(&right.expression)
            }
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
            value: SpecifiedLengthValue::Literal(Box::new(component)),
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
            value: SpecifiedLengthValue::Calculation(calculation),
        })
    }

    /// Borrows the exact literal token, if this is an ordinary percentage.
    pub fn literal_component(&self) -> Option<&CssComponentValue> {
        match &self.value {
            SpecifiedLengthValue::Literal(component) => Some(component),
            SpecifiedLengthValue::Calculation(_) => None,
        }
    }

    /// Borrows checked symbolic percentage math, when present.
    pub fn calculation(&self) -> Option<&CssPercentageCalculation> {
        match &self.value {
            SpecifiedLengthValue::Literal(_) => None,
            SpecifiedLengthValue::Calculation(calculation) => Some(calculation),
        }
    }

    /// Returns the root's parsed or programmatic origin.
    pub fn origin(&self) -> &CssValueOrigin {
        match &self.value {
            SpecifiedLengthValue::Literal(component) => component.origin(),
            SpecifiedLengthValue::Calculation(calculation) => calculation.origin(),
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
            SpecifiedLengthValue::Literal(component) => capture_literal(component, context),
            SpecifiedLengthValue::Calculation(calculation) => {
                crate::numeric::capture_specified(&calculation.expression, context)
                    .map(|(text, _)| text)
            }
        }
    }
}
