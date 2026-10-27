//! Relative and custom-profile color expression admission.

use crate::component::{
    CssComponentValueLimits, CssComponentValueRef, CssComponentValues, CssValueOrigin,
    CssValueTokenRef,
};
use crate::numeric::expression::{
    AdmissionPolicy, CalculationComponents, CalculationRoot, CssCalculationExpression,
    CssNumericConstructionErrorKind, NodeKind, NumericInputContext, Result, construct_with_policy,
    trivia, validate_components,
};
use crate::numeric::projection::{
    NumericProjectionOutcome, NumericProjectionScale, capture_specified_scaled,
};
use crate::numeric::{
    CssCalculationExpressionRef, CssCalculationType, CssNumericConstructionError,
};

/// Checked authored expression in a relative color's result slot.
impl crate::color::CssRelativeColorExpression {
    pub fn try_from_components(
        values: CssComponentValues,
        environment: crate::color::CssRelativeColorEnvironment,
        result_domain: crate::color::CssRelativeColorResultDomain,
        limits: CssComponentValueLimits,
    ) -> Result<Self> {
        construct_relative_color_expression(
            values,
            environment,
            result_domain,
            limits,
            AdmissionPolicy::Strict,
        )
    }
    pub(crate) fn from_parser_components(
        values: CalculationComponents<'_>,
        context: &NumericInputContext<'_>,
        environment: crate::color::CssRelativeColorEnvironment,
        result_domain: crate::color::CssRelativeColorResultDomain,
    ) -> Result<Self> {
        let policy = match context.ordinary() {
            NumericInputContext::QuirkyLengths(_) => unreachable!("ordinary numeric provenance"),
            NumericInputContext::Parsed(_) => AdmissionPolicy::RecoveredSyntax,
            NumericInputContext::Components(..) | NumericInputContext::Checked => {
                AdmissionPolicy::Strict
            }
        };
        construct_relative_color_expression(
            values,
            environment,
            result_domain,
            CssComponentValueLimits::default(),
            policy,
        )
    }
}
fn construct_relative_color_expression<'input>(
    values: impl Into<CalculationComponents<'input>>,
    environment: crate::color::CssRelativeColorEnvironment,
    result_domain: crate::color::CssRelativeColorResultDomain,
    limits: CssComponentValueLimits,
    policy: AdmissionPolicy,
) -> Result<crate::color::CssRelativeColorExpression> {
    use crate::color::CssRelativeColorExpressionValue as V;
    use crate::color::CssRelativeColorResultDomain as D;
    let values = values.into();
    validate_components(values.items(), limits, policy)?;
    let mut significant = values
        .items()
        .iter()
        .enumerate()
        .filter(|(_, c)| !trivia(c));
    let (index, component) = significant.next().ok_or_else(|| {
        CssNumericConstructionError::at(CssNumericConstructionErrorKind::EmptyValue, None)
    })?;
    if let Some((extra_index, extra)) = significant.next() {
        return Err(CssNumericConstructionError::at(
            CssNumericConstructionErrorKind::MultipleValues,
            Some(extra),
        )
        .with_path(Some(vec![extra_index].into_boxed_slice())));
    }
    let fail = |kind| {
        CssNumericConstructionError::at(kind, Some(component))
            .with_path(Some(vec![index].into_boxed_slice()))
    };
    if !environment.is_consistent()
        || (environment == crate::color::CssRelativeColorEnvironment::Alpha
            && result_domain != D::Alpha)
    {
        return Err(fail(CssNumericConstructionErrorKind::InvalidArgumentType));
    }
    let origin = component.origin().clone();
    let value = match component.view() {
        CssComponentValueRef::Token(CssValueTokenRef::Ident(name))
            if name.eq_ignore_ascii_case("none") =>
        {
            V::None
        }
        CssComponentValueRef::Token(CssValueTokenRef::Ident(name)) => {
            let (channel, _) = crate::color::syntax::recognition::numeric_relative_channel(environment, name)
                .ok_or_else(|| fail(CssNumericConstructionErrorKind::InvalidArgumentType))?;
            V::Channel(channel)
        }
        CssComponentValueRef::Token(CssValueTokenRef::Number(_)) => V::Number(
            crate::color::CssColorNumberLiteral::try_from_component(component.clone()).map_err(
                |error| {
                    CssNumericConstructionError::component(error)
                        .with_path(Some(vec![index].into_boxed_slice()))
                },
            )?,
        ),
        CssComponentValueRef::Token(CssValueTokenRef::Percentage(_)) if result_domain != D::Hue => {
            V::Percentage(
                crate::color::CssColorPercentageLiteral::try_from_component(component.clone()).map_err(
                    |error| {
                        CssNumericConstructionError::component(error)
                            .with_path(Some(vec![index].into_boxed_slice()))
                    },
                )?,
            )
        }
        CssComponentValueRef::Token(CssValueTokenRef::Dimension { .. })
            if result_domain == D::Hue =>
        {
            V::Angle(
                crate::numeric::CssNumericLiteral::<crate::numeric::CssAngle>::from_component_token(
                    component.clone(),
                )
                .map_err(|error| {
                    CssNumericConstructionError::component(error)
                        .with_path(Some(vec![index].into_boxed_slice()))
                })?,
            )
        }
        CssComponentValueRef::Function(_) => {
            let authored = values.authored()?;
            let expression = construct_with_policy(
                values,
                CalculationRoot::Relative(environment, result_domain),
                limits,
                policy,
            )?;
            V::Calculation(crate::color::CssRelativeColorCalculation::from_expression(
                authored, expression,
            ))
        }
        _ => return Err(fail(CssNumericConstructionErrorKind::RootDomainMismatch)),
    };
    Ok(crate::color::CssRelativeColorExpression::new(
        environment,
        result_domain,
        value,
        origin,
    ))
}

/// One nonbinding expression in a custom profile's component environment.
#[derive(Clone, Debug, PartialEq)]
pub struct CssProfileColorExpression {
    value: ProfileExpression,
    origin: CssValueOrigin,
}
#[derive(Clone, Debug, PartialEq)]
enum ProfileExpression {
    Literal {
        value: crate::color::CssColorComponent,
        components: CssComponentValues,
    },
    Reference {
        value: crate::color::CssColorProfileComponentName,
        components: CssComponentValues,
    },
    Calculation(CssProfileColorCalculation),
}
#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
pub enum CssProfileColorExpressionRef<'a> {
    Literal(&'a crate::color::CssColorComponent),
    Reference(&'a crate::color::CssColorProfileComponentName),
    Calculation(&'a CssProfileColorCalculation),
}
impl CssProfileColorExpression {
    pub fn try_from_components(values: CssComponentValues) -> Result<Self> {
        Self::from_components(values.into(), AdmissionPolicy::Strict)
    }
    pub(crate) fn from_parser_components(
        values: CalculationComponents<'_>,
        context: &NumericInputContext<'_>,
    ) -> Result<Self> {
        Self::from_components(
            values,
            match context.ordinary() {
                NumericInputContext::QuirkyLengths(_) => {
                    unreachable!("ordinary numeric provenance")
                }
                NumericInputContext::Parsed(_) => AdmissionPolicy::RecoveredSyntax,
                NumericInputContext::Components(..) | NumericInputContext::Checked => {
                    AdmissionPolicy::Strict
                }
            },
        )
    }
    fn from_components(values: CalculationComponents<'_>, policy: AdmissionPolicy) -> Result<Self> {
        let limits = CssComponentValueLimits::default();
        validate_components(values.items(), limits, policy)?;
        let mut significant = values.items().iter().filter(|c| !trivia(c));
        let component = significant.next().ok_or_else(|| {
            CssNumericConstructionError::at(CssNumericConstructionErrorKind::EmptyValue, None)
        })?;
        if let Some(extra) = significant.next() {
            return Err(CssNumericConstructionError::at(
                CssNumericConstructionErrorKind::MultipleValues,
                Some(extra),
            ));
        }
        let origin = component.origin().clone();
        let value = match component.view() {
            CssComponentValueRef::Token(CssValueTokenRef::Ident(name))
                if name.eq_ignore_ascii_case("none") =>
            {
                ProfileExpression::Literal {
                    value: crate::color::CssColorComponent::None,
                    components: values.retain(limits)?,
                }
            }
            CssComponentValueRef::Token(CssValueTokenRef::Ident(name)) => {
                let value =
                    crate::color::CssColorProfileComponentName::try_new(name).ok_or_else(|| {
                        CssNumericConstructionError::at(
                            CssNumericConstructionErrorKind::InvalidArgumentType,
                            Some(component),
                        )
                    })?;
                ProfileExpression::Reference {
                    value,
                    components: values.retain(limits)?,
                }
            }
            CssComponentValueRef::Token(
                CssValueTokenRef::Number(_) | CssValueTokenRef::Percentage(_),
            ) => {
                let value = crate::color::color_scalar::component(component.clone())
                    .map_err(CssNumericConstructionError::component)?;
                ProfileExpression::Literal {
                    value,
                    components: values.retain(limits)?,
                }
            }
            _ => {
                let expression = construct_with_policy(
                    values,
                    CalculationRoot::ProfileRelative,
                    CssComponentValueLimits::default(),
                    policy,
                )?;
                let mut references = Vec::new();
                let mut pending = vec![&expression];
                while let Some(node) = pending.pop() {
                    match &node.kind {
                        NodeKind::ProfileChannel(name) => references.push(name.clone()),
                        NodeKind::Sum(v) => pending.extend(v.iter().rev().map(|(_, e)| e)),
                        NodeKind::Product(v) => pending.extend(v.iter().rev().map(|(_, e)| e)),
                        NodeKind::Group(e) => pending.push(e),
                        NodeKind::Function { args, .. } => {
                            pending.extend(args.iter().rev().flatten())
                        }
                        _ => {}
                    }
                }
                ProfileExpression::Calculation(CssProfileColorCalculation {
                    expression: Box::new(expression),
                    references,
                })
            }
        };
        Ok(Self { value, origin })
    }
    pub fn view(&self) -> CssProfileColorExpressionRef<'_> {
        match &self.value {
            ProfileExpression::Literal { value, .. } => {
                CssProfileColorExpressionRef::Literal(value)
            }
            ProfileExpression::Reference { value, .. } => {
                CssProfileColorExpressionRef::Reference(value)
            }
            ProfileExpression::Calculation(v) => CssProfileColorExpressionRef::Calculation(v),
        }
    }
    pub fn origin(&self) -> &CssValueOrigin {
        &self.origin
    }
    pub fn components(&self) -> &CssComponentValues {
        match &self.value {
            ProfileExpression::Literal { components, .. }
            | ProfileExpression::Reference { components, .. } => components,
            ProfileExpression::Calculation(value) => value.components(),
        }
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssProfileColorCalculation {
    expression: Box<CssCalculationExpression>,
    references: Vec<crate::color::CssColorProfileComponentName>,
}
impl CssProfileColorCalculation {
    /// Lends the actual checked node before its separately admitted inspection.
    pub(crate) fn checked_expression(&self) -> &CssCalculationExpression {
        &self.expression
    }

    // The new declaration-block inverse uses the owning graph identity while
    // leaving authored components and diagnostic coordinates intact.
    pub(crate) fn specified_inverse_eq(
        &self,
        other: &Self,
        context: &mut crate::serialization::specified_serialization::SpecifiedSerializationContext,
    ) -> std::result::Result<bool, crate::serialization::CssSerializationError> {
        if self.references != other.references {
            return Ok(false);
        }
        self.expression
            .specified_inverse_eq(&other.expression, context)
    }

    pub fn expression(&self) -> CssCalculationExpressionRef<'_> {
        self.expression.as_ref().as_ref()
    }
    pub fn references(&self) -> &[crate::color::CssColorProfileComponentName] {
        &self.references
    }
    pub fn components(&self) -> &CssComponentValues {
        self.expression.components.as_ref().expect("checked root")
    }
    pub fn origin(&self) -> &CssValueOrigin {
        &self.expression.origin
    }
    pub fn result_type(&self) -> CssCalculationType {
        self.expression.result_type()
    }
    pub(crate) fn capture_specified(
        &self,
        scale: NumericProjectionScale,
        context: &mut crate::serialization::specified_serialization::SpecifiedSerializationContext,
    ) -> std::result::Result<
        (String, NumericProjectionOutcome),
        crate::serialization::CssSerializationError,
    > {
        capture_specified_scaled(&self.expression, scale, context)
    }
}

#[cfg(test)]
mod tests {
    use crate::numeric::projection::prepare_specified;
    use crate::serialization::CssSerializationLimits as Limits;
    use crate::serialization::specified_serialization::SpecifiedSerializationContext;

    #[test]
    fn suppressed_profile_preparation_retains_identity_for_later_emission() {
        let value = crate::color::CssProfileColorExpression::try_from_components(
            crate::component::values::parse_component_values("calc(Cyan)").unwrap(),
        )
        .unwrap();
        let crate::color::CssProfileColorExpressionRef::Calculation(value) = value.view() else {
            panic!("profile calculation");
        };
        let origin = value.origin().clone();
        let mut context = SpecifiedSerializationContext::new(Limits::new(100, 100, 4));
        context.replace_output_suppression(true);
        let prepared = prepare_specified(&value.expression, &mut context).unwrap();
        assert!(prepared.outcome().context_dependent);
        assert_eq!(prepared.outcome().scalar_value, None);
        assert_eq!(context.remaining_bytes(), 4);
        assert!(context.replace_output_suppression(false));
        let mut output = String::new();
        prepared.append(&mut context, &mut output, false).unwrap();
        assert_eq!(output, "Cyan");
        assert_eq!(context.remaining_bytes(), 0);
        assert_eq!(value.origin(), &origin);
    }
}
