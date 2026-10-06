//! Exact, checked authored number, length, and percentage domains with shared numeric helpers.

use crate::{
    CssComponentValue, CssComponentValueRef, CssFlexCalculation, CssLengthCalculation,
    CssLengthPercentageCalculation, CssLengthUnit, CssNumberCalculation,
    CssNumericConstructionError, CssNumericConstructionErrorKind, CssPercentageCalculation,
    CssSpecifiedValueSerializationError, CssSpecifiedValueSerializationErrorKind,
    CssSpecifiedValueSerializationLimits, CssValueOrigin, CssValueTokenRef,
    numeric_formatting::format_css_number, specified_serialization::SpecifiedSerializationContext,
};

type ConstructionResult<T> = Result<T, CssNumericConstructionError>;
type SerializationResult<T> = Result<T, CssSpecifiedValueSerializationError>;

/// An unrestricted authored flex dimension retaining an exact `fr` token or checked Flex math.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssSpecifiedFlex {
    value: SpecifiedNumericValue<CssFlexCalculation>,
}

impl CssSpecifiedFlex {
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

    pub fn try_from_component(component: CssComponentValue) -> ConstructionResult<Self> {
        let CssComponentValueRef::Token(CssValueTokenRef::Dimension { unit, .. }) =
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
        Ok(Self {
            value: SpecifiedNumericValue::Literal(Box::new(component)),
        })
    }

    pub fn try_from_calculation(calculation: CssFlexCalculation) -> ConstructionResult<Self> {
        if let Some(origin) = calculation.components().first_implicit_origin() {
            return Err(CssNumericConstructionError::at_origin(
                CssNumericConstructionErrorKind::RecoveredComponent,
                origin.clone(),
            ));
        }
        Self::from_parser_calculation(calculation)
    }

    pub(crate) fn from_parser_calculation(
        calculation: CssFlexCalculation,
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

    /// Emits ordinary and finite calculated numbers with at most six fractional
    /// places, nearest with ties away from zero. Authored values remain unchanged;
    /// calculation arithmetic, precision, range and symbolic behavior retain
    /// their existing contracts.
    pub fn serialize_specified(&self) -> SerializationResult<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    /// Shares cumulative visits and budgets actual rounded text.
    /// Failure leaves authored values unchanged; arithmetic and traversal costs are retained.
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> SerializationResult<String> {
        let mut writer = crate::specified_rule_serialization::SpecifiedRuleWriter::new(limits);
        self.append_to_rule_writer(&mut writer)?;
        Ok(writer.css)
    }

    pub(crate) fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSpecifiedValueSerializationError> {
        let context = &mut writer.context;
        let captured = self.capture_specified(context)?;
        let output = &mut writer.css;
        context.append(output, &captured)?;
        Ok(())
    }

    pub(crate) fn capture_specified(
        &self,
        context: &mut SpecifiedSerializationContext,
    ) -> SerializationResult<String> {
        match &self.value {
            SpecifiedNumericValue::Literal(value) => {
                context.charge_input(1)?;
                context.charge_projection(1)?;
                if context.output_suppressed() {
                    return Ok(String::new());
                }
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
                let mut output = format_css_number(number.representation(), 0, limit)?;
                context.append_temporary(&mut output, "fr")?;
                Ok(output)
            }
            SpecifiedNumericValue::Calculation(value) => {
                crate::numeric::capture_specified(&value.expression, context).map(|(text, _)| text)
            }
        }
    }
}

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

    /// Emits ordinary and finite calculated numbers with at most six fractional
    /// places, nearest with ties away from zero. Authored values remain unchanged;
    /// calculation arithmetic, precision, range and symbolic behavior retain
    /// their existing contracts.
    pub fn serialize_specified(&self) -> SerializationResult<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    /// Shares cumulative visits and budgets actual rounded text.
    /// Failure leaves authored values unchanged; arithmetic and traversal costs are retained.
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> SerializationResult<String> {
        let mut writer = crate::specified_rule_serialization::SpecifiedRuleWriter::new(limits);
        self.append_to_rule_writer(&mut writer)?;
        Ok(writer.css)
    }

    pub(crate) fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSpecifiedValueSerializationError> {
        let context = &mut writer.context;
        let captured = self.capture_specified(context)?;
        let output = &mut writer.css;
        context.append(output, &captured)?;
        Ok(())
    }

    pub(crate) fn capture_specified(
        &self,
        context: &mut SpecifiedSerializationContext,
    ) -> SerializationResult<String> {
        match &self.value {
            SpecifiedNumericValue::Literal(value) => {
                context.charge_input(1)?;
                context.charge_projection(1)?;
                if context.output_suppressed() {
                    return Ok(String::new());
                }
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
                let mut output = format_css_number(number.representation(), 0, limit)?;
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

/// Compares retained ordinary coefficients and their emitted token kind/unit.
/// Formatting can round unequal coefficients to the same text.
pub(crate) fn ordinary_literal_equal(left: &CssComponentValue, right: &CssComponentValue) -> bool {
    use crate::exact_decimal::LexicalDecimal;
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
            left_unit.eq_ignore_ascii_case(right_unit)
                && LexicalDecimal::new(left.representation())
                    .value_eq(&LexicalDecimal::new(right.representation()))
        }
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
                if crate::exact_decimal::LexicalDecimal::new(number.representation()).len == 0)
    };
    let pixel_zero = |value: &CssComponentValue| {
        matches!(value.view(),
            CssComponentValueRef::Token(CssValueTokenRef::Dimension { number, unit })
                if unit.eq_ignore_ascii_case("px")
                    && crate::exact_decimal::LexicalDecimal::new(number.representation()).len == 0)
    };
    (unitless_zero(left) && pixel_zero(right)) || (unitless_zero(right) && pixel_zero(left))
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
    if context.output_suppressed() {
        return Ok(String::new());
    }
    let coefficient_limit = context
        .remaining_bytes()
        .checked_sub(suffix.len())
        .ok_or_else(|| {
            CssSpecifiedValueSerializationError::new(
                CssSpecifiedValueSerializationErrorKind::ByteLimit,
            )
        })?;
    let mut output = format_css_number(number.representation(), 0, coefficient_limit)?;
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

    /// Emits ordinary and finite calculated numbers with at most six fractional
    /// places, nearest with ties away from zero. Authored values remain unchanged;
    /// calculation arithmetic, precision, range and symbolic behavior retain
    /// their existing contracts.
    pub fn serialize_specified(&self) -> SerializationResult<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    /// Shares cumulative visits and budgets actual rounded text.
    /// Failure leaves authored values unchanged; arithmetic and traversal costs are retained.
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> SerializationResult<String> {
        let mut writer = crate::specified_rule_serialization::SpecifiedRuleWriter::new(limits);
        self.append_to_rule_writer(&mut writer)?;
        Ok(writer.css)
    }

    pub(crate) fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSpecifiedValueSerializationError> {
        let context = &mut writer.context;
        let captured = self.capture_specified(context)?;
        let output = &mut writer.css;
        context.append(output, &captured)?;
        Ok(())
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
    /// Emits ordinary and finite calculated numbers with at most six fractional
    /// places, nearest with ties away from zero. Authored values remain unchanged;
    /// calculation arithmetic, precision, range and symbolic behavior retain
    /// their existing contracts.
    pub fn serialize_specified(&self) -> SerializationResult<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    /// Serializes atomically under the shared input, projection, and byte limits.
    /// Shares cumulative visits and budgets actual rounded text.
    /// Failure leaves authored values unchanged; arithmetic and traversal costs are retained.
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> SerializationResult<String> {
        let mut writer = crate::specified_rule_serialization::SpecifiedRuleWriter::new(limits);
        self.append_to_rule_writer(&mut writer)?;
        Ok(writer.css)
    }

    pub(crate) fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSpecifiedValueSerializationError> {
        let context = &mut writer.context;
        let captured = self.capture_specified(context)?;
        let output = &mut writer.css;
        context.append(output, &captured)?;
        Ok(())
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
    /// Emits ordinary and finite calculated numbers with at most six fractional
    /// places, nearest with ties away from zero. Authored values remain unchanged;
    /// calculation arithmetic, precision, range and symbolic behavior retain
    /// their existing contracts.
    pub fn serialize_specified(&self) -> SerializationResult<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    /// Serializes the specified number atomically under explicit resource limits.
    /// Shares cumulative visits and budgets actual rounded text.
    /// Failure leaves authored values unchanged; arithmetic and traversal costs are retained.
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> SerializationResult<String> {
        let mut writer = crate::specified_rule_serialization::SpecifiedRuleWriter::new(limits);
        self.append_to_rule_writer(&mut writer)?;
        Ok(writer.css)
    }

    pub(crate) fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSpecifiedValueSerializationError> {
        let context = &mut writer.context;
        let captured = self.capture_specified(context)?;
        let output = &mut writer.css;
        context.append(output, &captured)?;
        Ok(())
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

// Length domains additionally own the contextual Number-as-px interpretation.
// It cannot be constructed by the ordinary unscoped public constructors.
#[derive(Clone, Debug, Eq, PartialEq)]
enum SpecifiedLengthValue<C> {
    Literal(Box<CssComponentValue>),
    QuirkyNumber(Box<CssComponentValue>),
    Calculation(C),
}

fn capture_quirky_length(
    component: &CssComponentValue,
    context: &mut SpecifiedSerializationContext,
) -> SerializationResult<String> {
    let (number, _) = visit_literal(component, context)?;
    if context.output_suppressed() {
        return Ok(String::new());
    }
    let coefficient_limit = context.remaining_bytes().checked_sub(2).ok_or_else(|| {
        CssSpecifiedValueSerializationError::new(CssSpecifiedValueSerializationErrorKind::ByteLimit)
    })?;
    let mut output = format_css_number(number.representation(), 0, coefficient_limit)?;
    context.append_temporary(&mut output, "px")?;
    Ok(output)
}

/// A signed CSS `<length>` that retains exact ordinary spelling or deferred math.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssSpecifiedLength {
    value: SpecifiedLengthValue<CssLengthCalculation>,
}

impl CssSpecifiedLength {
    // Intrinsic narrowing consumes the checked interpretation, not fresh syntax.
    pub(crate) fn try_nonnegative(&self) -> ConstructionResult<CssSpecifiedNonNegativeLength> {
        match &self.value {
            SpecifiedLengthValue::Literal(component) => {
                CssSpecifiedNonNegativeLength::try_from_component((**component).clone())
            }
            SpecifiedLengthValue::QuirkyNumber(component) => {
                let CssComponentValueRef::Token(CssValueTokenRef::Number(number)) =
                    component.view()
                else {
                    unreachable!("checked quirky Number")
                };
                if crate::exact_decimal::LexicalDecimal::new(number.representation()).negative {
                    return Err(CssNumericConstructionError::at(
                        CssNumericConstructionErrorKind::OutOfRange,
                        Some(component),
                    ));
                }
                Ok(CssSpecifiedNonNegativeLength {
                    value: self.value.clone(),
                })
            }
            SpecifiedLengthValue::Calculation(calculation) => {
                CssSpecifiedNonNegativeLength::try_from_calculation(calculation.clone())
            }
        }
    }

    /// Whether an original Number token has the contextual pixel interpretation.
    /// `literal_component()` still returns that unchanged authored Number token.
    pub fn is_quirky_length(&self) -> bool {
        matches!(self.value, SpecifiedLengthValue::QuirkyNumber(_))
    }

    pub(crate) fn from_property_component(
        component: CssComponentValue,
        numeric: &crate::numeric::NumericInputContext<'_>,
    ) -> ConstructionResult<Self> {
        if !numeric.allows_quirky_lengths()
            || !matches!(
                component.view(),
                CssComponentValueRef::Token(CssValueTokenRef::Number(_))
            )
        {
            return Self::try_from_component(component);
        }
        let CssComponentValueRef::Token(CssValueTokenRef::Number(number)) = component.view() else {
            unreachable!("checked Number token")
        };
        let decimal = crate::exact_decimal::LexicalDecimal::new(number.representation());
        // Ordinary exact zero keeps its ordinary serialization and identity.
        if decimal.len == 0 {
            return Self::try_from_component(component);
        }
        Ok(Self {
            value: SpecifiedLengthValue::QuirkyNumber(Box::new(component)),
        })
    }

    pub(crate) fn structural_eq(&self, other: &Self) -> bool {
        match (&self.value, &other.value) {
            (SpecifiedLengthValue::Literal(left), SpecifiedLengthValue::Literal(right))
            | (
                SpecifiedLengthValue::QuirkyNumber(left),
                SpecifiedLengthValue::QuirkyNumber(right),
            ) => left.structural_eq_ignoring_origin(right),
            (SpecifiedLengthValue::Calculation(left), SpecifiedLengthValue::Calculation(right)) => {
                left.structural_eq(right)
            }
            _ => false,
        }
    }

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
            SpecifiedLengthValue::Literal(component)
            | SpecifiedLengthValue::QuirkyNumber(component) => Some(component),
            SpecifiedLengthValue::Calculation(_) => None,
        }
    }

    /// Borrows the symbolic checked math root, when present.
    pub fn calculation(&self) -> Option<&CssLengthCalculation> {
        match &self.value {
            SpecifiedLengthValue::Literal(_) | SpecifiedLengthValue::QuirkyNumber(_) => None,
            SpecifiedLengthValue::Calculation(calculation) => Some(calculation),
        }
    }

    /// Returns the original parsed or programmatic root origin.
    pub fn origin(&self) -> &CssValueOrigin {
        match &self.value {
            SpecifiedLengthValue::Literal(component)
            | SpecifiedLengthValue::QuirkyNumber(component) => component.origin(),
            SpecifiedLengthValue::Calculation(calculation) => calculation.origin(),
        }
    }

    /// Serializes the canonical specified length with default resource limits.
    /// Emits ordinary and finite calculated numbers with at most six fractional
    /// places, nearest with ties away from zero. Authored values remain unchanged;
    /// calculation arithmetic, precision, range and symbolic behavior retain
    /// their existing contracts.
    pub fn serialize_specified(&self) -> SerializationResult<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    /// Serializes the canonical specified length under explicit resource limits.
    /// Shares cumulative visits and budgets actual rounded text.
    /// Failure leaves authored values unchanged; arithmetic and traversal costs are retained.
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> SerializationResult<String> {
        let mut writer = crate::specified_rule_serialization::SpecifiedRuleWriter::new(limits);
        self.append_to_rule_writer(&mut writer)?;
        Ok(writer.css)
    }

    pub(crate) fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSpecifiedValueSerializationError> {
        let context = &mut writer.context;
        let captured = self.capture_specified(context)?;
        let output = &mut writer.css;
        context.append(output, &captured)?;
        Ok(())
    }

    pub(crate) fn capture_specified(
        &self,
        context: &mut SpecifiedSerializationContext,
    ) -> SerializationResult<String> {
        match &self.value {
            SpecifiedLengthValue::Literal(component) => capture_literal(component, context),
            SpecifiedLengthValue::QuirkyNumber(component) => {
                capture_quirky_length(component, context)
            }
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
    // Nonnegative Length is an intrinsic subset of signed Length. Preserve the
    // entire authored state, including contextual pixels and original recovery.
    pub(crate) fn into_signed(self) -> CssSpecifiedLength {
        CssSpecifiedLength { value: self.value }
    }

    /// Whether an original Number token has the contextual pixel interpretation.
    /// `literal_component()` still returns that unchanged authored Number token.
    pub fn is_quirky_length(&self) -> bool {
        matches!(self.value, SpecifiedLengthValue::QuirkyNumber(_))
    }

    pub(crate) fn from_property_component(
        component: CssComponentValue,
        numeric: &crate::numeric::NumericInputContext<'_>,
    ) -> ConstructionResult<Self> {
        if !numeric.allows_quirky_lengths()
            || !matches!(
                component.view(),
                CssComponentValueRef::Token(CssValueTokenRef::Number(_))
            )
        {
            return Self::try_from_component(component);
        }
        let CssComponentValueRef::Token(CssValueTokenRef::Number(number)) = component.view() else {
            unreachable!("checked Number token")
        };
        let decimal = crate::exact_decimal::LexicalDecimal::new(number.representation());
        // Ordinary exact zero keeps its ordinary serialization and identity.
        if decimal.len == 0 {
            return Self::try_from_component(component);
        }
        if decimal.negative {
            return Err(CssNumericConstructionError::at(
                CssNumericConstructionErrorKind::OutOfRange,
                Some(&component),
            ));
        }
        Ok(Self {
            value: SpecifiedLengthValue::QuirkyNumber(Box::new(component)),
        })
    }

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
            SpecifiedLengthValue::Literal(component)
            | SpecifiedLengthValue::QuirkyNumber(component) => Some(component),
            SpecifiedLengthValue::Calculation(_) => None,
        }
    }

    /// Borrows the symbolic checked math root, when present.
    pub fn calculation(&self) -> Option<&CssLengthCalculation> {
        match &self.value {
            SpecifiedLengthValue::Literal(_) | SpecifiedLengthValue::QuirkyNumber(_) => None,
            SpecifiedLengthValue::Calculation(calculation) => Some(calculation),
        }
    }

    /// Returns the original parsed or programmatic root origin.
    pub fn origin(&self) -> &CssValueOrigin {
        match &self.value {
            SpecifiedLengthValue::Literal(component)
            | SpecifiedLengthValue::QuirkyNumber(component) => component.origin(),
            SpecifiedLengthValue::Calculation(calculation) => calculation.origin(),
        }
    }

    /// Serializes the canonical specified length with default resource limits.
    /// Emits ordinary and finite calculated numbers with at most six fractional
    /// places, nearest with ties away from zero. Authored values remain unchanged;
    /// calculation arithmetic, precision, range and symbolic behavior retain
    /// their existing contracts.
    pub fn serialize_specified(&self) -> SerializationResult<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    /// Serializes the canonical specified length under explicit resource limits.
    /// Shares cumulative visits and budgets actual rounded text.
    /// Failure leaves authored values unchanged; arithmetic and traversal costs are retained.
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> SerializationResult<String> {
        let mut writer = crate::specified_rule_serialization::SpecifiedRuleWriter::new(limits);
        self.append_to_rule_writer(&mut writer)?;
        Ok(writer.css)
    }

    pub(crate) fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSpecifiedValueSerializationError> {
        let context = &mut writer.context;
        let captured = self.capture_specified(context)?;
        let output = &mut writer.css;
        context.append(output, &captured)?;
        Ok(())
    }

    pub(crate) fn capture_specified(
        &self,
        context: &mut SpecifiedSerializationContext,
    ) -> SerializationResult<String> {
        match &self.value {
            SpecifiedLengthValue::Literal(component) => capture_literal(component, context),
            SpecifiedLengthValue::QuirkyNumber(component) => {
                capture_quirky_length(component, context)
            }
            SpecifiedLengthValue::Calculation(calculation) => {
                crate::numeric::capture_specified(&calculation.expression, context)
                    .map(|(text, _)| text)
            }
        }
    }

    pub(crate) fn structural_eq(&self, other: &Self) -> bool {
        match (&self.value, &other.value) {
            (SpecifiedLengthValue::Literal(left), SpecifiedLengthValue::Literal(right))
            | (
                SpecifiedLengthValue::QuirkyNumber(left),
                SpecifiedLengthValue::QuirkyNumber(right),
            ) => left.structural_eq_ignoring_origin(right),
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
    /// Whether an original Number token has the contextual pixel interpretation.
    /// `literal_component()` still returns that unchanged authored Number token.
    pub fn is_quirky_length(&self) -> bool {
        matches!(self.value, SpecifiedLengthValue::QuirkyNumber(_))
    }

    pub(crate) fn from_property_component(
        component: CssComponentValue,
        numeric: &crate::numeric::NumericInputContext<'_>,
    ) -> ConstructionResult<Self> {
        if !numeric.allows_quirky_lengths()
            || !matches!(
                component.view(),
                CssComponentValueRef::Token(CssValueTokenRef::Number(_))
            )
        {
            return Self::try_from_component(component);
        }
        let CssComponentValueRef::Token(CssValueTokenRef::Number(number)) = component.view() else {
            unreachable!("checked Number token")
        };
        let decimal = crate::exact_decimal::LexicalDecimal::new(number.representation());
        // Ordinary exact zero keeps its ordinary serialization and identity.
        if decimal.len == 0 {
            return Self::try_from_component(component);
        }
        Ok(Self {
            value: SpecifiedLengthValue::QuirkyNumber(Box::new(component)),
        })
    }

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
            SpecifiedLengthValue::Literal(component)
            | SpecifiedLengthValue::QuirkyNumber(component) => Some(component),
            SpecifiedLengthValue::Calculation(_) => None,
        }
    }

    /// Borrows the symbolic checked math root, when present.
    pub fn calculation(&self) -> Option<&CssLengthPercentageCalculation> {
        match &self.value {
            SpecifiedLengthValue::Literal(_) | SpecifiedLengthValue::QuirkyNumber(_) => None,
            SpecifiedLengthValue::Calculation(calculation) => Some(calculation),
        }
    }

    /// Returns the original parsed or programmatic root origin.
    pub fn origin(&self) -> &CssValueOrigin {
        match &self.value {
            SpecifiedLengthValue::Literal(component)
            | SpecifiedLengthValue::QuirkyNumber(component) => component.origin(),
            SpecifiedLengthValue::Calculation(calculation) => calculation.origin(),
        }
    }

    /// Serializes the canonical specified value with default resource limits.
    /// Emits ordinary and finite calculated numbers with at most six fractional
    /// places, nearest with ties away from zero. Authored values remain unchanged;
    /// calculation arithmetic, precision, range and symbolic behavior retain
    /// their existing contracts.
    pub fn serialize_specified(&self) -> SerializationResult<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    /// Serializes the canonical specified value under explicit resource limits.
    /// Shares cumulative visits and budgets actual rounded text.
    /// Failure leaves authored values unchanged; arithmetic and traversal costs are retained.
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> SerializationResult<String> {
        let mut writer = crate::specified_rule_serialization::SpecifiedRuleWriter::new(limits);
        self.append_to_rule_writer(&mut writer)?;
        Ok(writer.css)
    }

    pub(crate) fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSpecifiedValueSerializationError> {
        let context = &mut writer.context;
        let captured = self.capture_specified(context)?;
        let output = &mut writer.css;
        context.append(output, &captured)?;
        Ok(())
    }

    pub(crate) fn capture_specified(
        &self,
        context: &mut SpecifiedSerializationContext,
    ) -> SerializationResult<String> {
        match &self.value {
            SpecifiedLengthValue::Literal(component) => capture_literal(component, context),
            SpecifiedLengthValue::QuirkyNumber(component) => {
                capture_quirky_length(component, context)
            }
            SpecifiedLengthValue::Calculation(calculation) => {
                crate::numeric::capture_specified(&calculation.expression, context)
                    .map(|(text, _)| text)
            }
        }
    }

    pub(crate) fn structural_eq(&self, other: &Self) -> bool {
        match (&self.value, &other.value) {
            (SpecifiedLengthValue::Literal(left), SpecifiedLengthValue::Literal(right))
            | (
                SpecifiedLengthValue::QuirkyNumber(left),
                SpecifiedLengthValue::QuirkyNumber(right),
            ) => left.structural_eq_ignoring_origin(right),
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
    /// Whether an original Number token has the contextual pixel interpretation.
    /// `literal_component()` still returns that unchanged authored Number token.
    pub fn is_quirky_length(&self) -> bool {
        matches!(self.value, SpecifiedLengthValue::QuirkyNumber(_))
    }

    pub(crate) fn from_property_component(
        component: CssComponentValue,
        numeric: &crate::numeric::NumericInputContext<'_>,
    ) -> ConstructionResult<Self> {
        if !numeric.allows_quirky_lengths()
            || !matches!(
                component.view(),
                CssComponentValueRef::Token(CssValueTokenRef::Number(_))
            )
        {
            return Self::try_from_component(component);
        }
        let CssComponentValueRef::Token(CssValueTokenRef::Number(number)) = component.view() else {
            unreachable!("checked Number token")
        };
        let decimal = crate::exact_decimal::LexicalDecimal::new(number.representation());
        // Ordinary exact zero keeps its ordinary serialization and identity.
        if decimal.len == 0 {
            return Self::try_from_component(component);
        }
        if decimal.negative {
            return Err(CssNumericConstructionError::at(
                CssNumericConstructionErrorKind::OutOfRange,
                Some(&component),
            ));
        }
        Ok(Self {
            value: SpecifiedLengthValue::QuirkyNumber(Box::new(component)),
        })
    }

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
            SpecifiedLengthValue::Literal(component)
            | SpecifiedLengthValue::QuirkyNumber(component) => Some(component),
            SpecifiedLengthValue::Calculation(_) => None,
        }
    }

    /// Borrows the symbolic checked math root, when present.
    pub fn calculation(&self) -> Option<&CssLengthPercentageCalculation> {
        match &self.value {
            SpecifiedLengthValue::Literal(_) | SpecifiedLengthValue::QuirkyNumber(_) => None,
            SpecifiedLengthValue::Calculation(calculation) => Some(calculation),
        }
    }

    /// Returns the original parsed or programmatic root origin.
    pub fn origin(&self) -> &CssValueOrigin {
        match &self.value {
            SpecifiedLengthValue::Literal(component)
            | SpecifiedLengthValue::QuirkyNumber(component) => component.origin(),
            SpecifiedLengthValue::Calculation(calculation) => calculation.origin(),
        }
    }

    /// Serializes the canonical specified value with default resource limits.
    /// Emits ordinary and finite calculated numbers with at most six fractional
    /// places, nearest with ties away from zero. Authored values remain unchanged;
    /// calculation arithmetic, precision, range and symbolic behavior retain
    /// their existing contracts.
    pub fn serialize_specified(&self) -> SerializationResult<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    /// Serializes the canonical specified value under explicit resource limits.
    /// Shares cumulative visits and budgets actual rounded text.
    /// Failure leaves authored values unchanged; arithmetic and traversal costs are retained.
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> SerializationResult<String> {
        let mut writer = crate::specified_rule_serialization::SpecifiedRuleWriter::new(limits);
        self.append_to_rule_writer(&mut writer)?;
        Ok(writer.css)
    }

    pub(crate) fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSpecifiedValueSerializationError> {
        let context = &mut writer.context;
        let captured = self.capture_specified(context)?;
        let output = &mut writer.css;
        context.append(output, &captured)?;
        Ok(())
    }

    pub(crate) fn capture_specified(
        &self,
        context: &mut SpecifiedSerializationContext,
    ) -> SerializationResult<String> {
        match &self.value {
            SpecifiedLengthValue::Literal(component) => capture_literal(component, context),
            SpecifiedLengthValue::QuirkyNumber(component) => {
                capture_quirky_length(component, context)
            }
            SpecifiedLengthValue::Calculation(calculation) => {
                crate::numeric::capture_specified(&calculation.expression, context)
                    .map(|(text, _)| text)
            }
        }
    }

    pub(crate) fn structural_eq(&self, other: &Self) -> bool {
        match (&self.value, &other.value) {
            (SpecifiedLengthValue::Literal(left), SpecifiedLengthValue::Literal(right))
            | (
                SpecifiedLengthValue::QuirkyNumber(left),
                SpecifiedLengthValue::QuirkyNumber(right),
            ) => left.structural_eq_ignoring_origin(right),
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
    /// Emits ordinary and finite calculated numbers with at most six fractional
    /// places, nearest with ties away from zero. Authored values remain unchanged;
    /// calculation arithmetic, precision, range and symbolic behavior retain
    /// their existing contracts.
    pub fn serialize_specified(&self) -> SerializationResult<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    /// Serializes atomically under shared input, projection, and byte budgets.
    /// Shares cumulative visits and budgets actual rounded text.
    /// Failure leaves authored values unchanged; arithmetic and traversal costs are retained.
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> SerializationResult<String> {
        let mut writer = crate::specified_rule_serialization::SpecifiedRuleWriter::new(limits);
        self.append_to_rule_writer(&mut writer)?;
        Ok(writer.css)
    }

    pub(crate) fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSpecifiedValueSerializationError> {
        let context = &mut writer.context;
        let captured = self.capture_specified(context)?;
        let output = &mut writer.css;
        context.append(output, &captured)?;
        Ok(())
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
macro_rules! append_checked_numeric {
    ($($owner:ident),* $(,)?) => { $(
        impl $owner {
            pub(crate) fn append_specified(&self, context: &mut SpecifiedSerializationContext, output: &mut String) -> SerializationResult<()> {
                let captured = self.capture_specified(context)?;
                context.append(output, &captured)
            }
        }
    )* };
}
append_checked_numeric!(
    CssSpecifiedLength,
    CssSpecifiedNonNegativeLength,
    CssSpecifiedLengthPercentage,
    CssSpecifiedNonNegativeLengthPercentage,
    CssSpecifiedNonNegativeNumber,
    CssSpecifiedNonNegativePercentage,
    CssSpecifiedPercentage
);

#[cfg(test)]
mod ordinary_literal_comparison_tests {
    use super::*;

    fn component(text: &str) -> CssComponentValue {
        let values = crate::parse_component_values(text).unwrap();
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
