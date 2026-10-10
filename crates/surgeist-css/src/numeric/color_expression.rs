//! Relative and custom-profile color expression admission.

use super::{
    AdmissionPolicy, CalculationRoot, CssCalculationExpression, CssCalculationExpressionRef,
    CssCalculationType, CssNumericConstructionError, CssNumericConstructionErrorKind, NodeKind,
    NumericInputContext, NumericProjectionOutcome, NumericProjectionScale, Result,
    capture_specified_scaled, construct_with_policy, trivia, validate_components,
};
use crate::{
    CssComponentValueLimits, CssComponentValueRef, CssComponentValues, CssValueOrigin,
    CssValueTokenRef,
};

/// Checked authored expression in a relative color's result slot.
impl crate::CssRelativeColorExpression {
    pub fn try_from_components(
        values: CssComponentValues,
        environment: crate::CssRelativeColorEnvironment,
        result_domain: crate::CssRelativeColorResultDomain,
    ) -> Result<Self> {
        Self::try_from_components_with_limits(
            values,
            environment,
            result_domain,
            CssComponentValueLimits::default(),
        )
    }
    pub fn try_from_components_with_limits(
        values: CssComponentValues,
        environment: crate::CssRelativeColorEnvironment,
        result_domain: crate::CssRelativeColorResultDomain,
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
        values: CssComponentValues,
        context: &NumericInputContext<'_>,
        environment: crate::CssRelativeColorEnvironment,
        result_domain: crate::CssRelativeColorResultDomain,
    ) -> Result<Self> {
        let policy = match context.ordinary() {
            NumericInputContext::QuirkyLengths(_) => unreachable!("ordinary numeric provenance"),
            NumericInputContext::Parsed(_) => AdmissionPolicy::RecoveredSyntax,
            NumericInputContext::Components(..) => AdmissionPolicy::Strict,
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
fn construct_relative_color_expression(
    values: CssComponentValues,
    environment: crate::CssRelativeColorEnvironment,
    result_domain: crate::CssRelativeColorResultDomain,
    limits: CssComponentValueLimits,
    policy: AdmissionPolicy,
) -> Result<crate::CssRelativeColorExpression> {
    use crate::{CssRelativeColorExpressionValue as V, CssRelativeColorResultDomain as D};
    validate_components(&values, limits, policy)?;
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
        || (environment == crate::CssRelativeColorEnvironment::Alpha && result_domain != D::Alpha)
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
            let (channel, _) = crate::syntax::color::numeric_relative_channel(environment, name)
                .ok_or_else(|| fail(CssNumericConstructionErrorKind::InvalidArgumentType))?;
            V::Channel(channel)
        }
        CssComponentValueRef::Token(CssValueTokenRef::Number(_)) => V::Number(
            crate::CssColorNumberLiteral::try_from_component(component.clone()).map_err(
                |error| {
                    CssNumericConstructionError::component(error)
                        .with_path(Some(vec![index].into_boxed_slice()))
                },
            )?,
        ),
        CssComponentValueRef::Token(CssValueTokenRef::Percentage(_)) if result_domain != D::Hue => {
            V::Percentage(
                crate::CssColorPercentageLiteral::try_from_component(component.clone()).map_err(
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
                crate::CssAngleLiteral::try_from_component(component.clone()).map_err(|error| {
                    CssNumericConstructionError::component(error)
                        .with_path(Some(vec![index].into_boxed_slice()))
                })?,
            )
        }
        CssComponentValueRef::Function(_) => {
            let serialized = values
                .serialize()
                .map_err(CssNumericConstructionError::component)?;
            let authored = crate::CssAuthoredDeclarationValue::new(serialized.as_css());
            let expression = construct_with_policy(
                values,
                CalculationRoot::Relative(environment, result_domain),
                limits,
                policy,
            )?;
            V::Calculation(crate::CssRelativeColorCalculation::from_expression(
                authored, expression,
            ))
        }
        _ => return Err(fail(CssNumericConstructionErrorKind::RootDomainMismatch)),
    };
    Ok(crate::CssRelativeColorExpression::new(
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
    components: CssComponentValues,
    origin: CssValueOrigin,
}
#[derive(Clone, Debug, PartialEq)]
enum ProfileExpression {
    Literal(crate::CssColorComponent),
    Reference(crate::CssColorProfileComponentName),
    Calculation(CssProfileColorCalculation),
}
#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
pub enum CssProfileColorExpressionRef<'a> {
    Literal(&'a crate::CssColorComponent),
    Reference(&'a crate::CssColorProfileComponentName),
    Calculation(&'a CssProfileColorCalculation),
}
impl CssProfileColorExpression {
    pub fn try_from_components(values: CssComponentValues) -> Result<Self> {
        Self::from_components(values, AdmissionPolicy::Strict)
    }
    pub(crate) fn from_parser_components(
        values: CssComponentValues,
        context: &NumericInputContext<'_>,
    ) -> Result<Self> {
        Self::from_components(
            values,
            match context.ordinary() {
                NumericInputContext::QuirkyLengths(_) => {
                    unreachable!("ordinary numeric provenance")
                }
                NumericInputContext::Parsed(_) => AdmissionPolicy::RecoveredSyntax,
                NumericInputContext::Components(..) => AdmissionPolicy::Strict,
            },
        )
    }
    fn from_components(values: CssComponentValues, policy: AdmissionPolicy) -> Result<Self> {
        validate_components(&values, CssComponentValueLimits::default(), policy)?;
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
                ProfileExpression::Literal(crate::CssColorComponent::None)
            }
            CssComponentValueRef::Token(CssValueTokenRef::Ident(name)) => {
                ProfileExpression::Reference(
                    crate::CssColorProfileComponentName::try_new(name).ok_or_else(|| {
                        CssNumericConstructionError::at(
                            CssNumericConstructionErrorKind::InvalidArgumentType,
                            Some(component),
                        )
                    })?,
                )
            }
            CssComponentValueRef::Token(
                CssValueTokenRef::Number(_) | CssValueTokenRef::Percentage(_),
            ) => ProfileExpression::Literal(
                crate::color_scalar::component(component.clone())
                    .map_err(CssNumericConstructionError::component)?,
            ),
            _ => {
                let expression = construct_with_policy(
                    values.clone(),
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
        Ok(Self {
            value,
            components: values,
            origin,
        })
    }
    pub fn view(&self) -> CssProfileColorExpressionRef<'_> {
        match &self.value {
            ProfileExpression::Literal(v) => CssProfileColorExpressionRef::Literal(v),
            ProfileExpression::Reference(v) => CssProfileColorExpressionRef::Reference(v),
            ProfileExpression::Calculation(v) => CssProfileColorExpressionRef::Calculation(v),
        }
    }
    pub fn origin(&self) -> &CssValueOrigin {
        &self.origin
    }
    pub fn components(&self) -> &CssComponentValues {
        &self.components
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssProfileColorCalculation {
    expression: Box<CssCalculationExpression>,
    references: Vec<crate::CssColorProfileComponentName>,
}
impl CssProfileColorCalculation {
    // The new declaration-block inverse uses the owning graph identity while
    // leaving authored components and diagnostic coordinates intact.
    pub(crate) fn specified_inverse_eq(
        &self,
        other: &Self,
        context: &mut crate::specified_serialization::SpecifiedSerializationContext,
    ) -> std::result::Result<bool, crate::CssSpecifiedValueSerializationError> {
        if self.references != other.references {
            return Ok(false);
        }
        self.expression
            .specified_inverse_eq(&other.expression, context)
    }

    pub fn expression(&self) -> CssCalculationExpressionRef<'_> {
        self.expression.as_ref().as_ref()
    }
    pub fn references(&self) -> &[crate::CssColorProfileComponentName] {
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
        context: &mut crate::specified_serialization::SpecifiedSerializationContext,
    ) -> std::result::Result<
        (String, NumericProjectionOutcome),
        crate::CssSpecifiedValueSerializationError,
    > {
        capture_specified_scaled(&self.expression, scale, context)
    }
}

#[cfg(test)]
mod tests {
    use crate::numeric::prepare_specified;
    use crate::{
        CssSpecifiedValueSerializationLimits as Limits, parse_component_values,
        specified_serialization::SpecifiedSerializationContext,
    };

    #[test]
    fn suppressed_profile_preparation_retains_identity_for_later_emission() {
        let value = crate::CssProfileColorExpression::try_from_components(
            parse_component_values("calc(Cyan)").unwrap(),
        )
        .unwrap();
        let crate::CssProfileColorExpressionRef::Calculation(value) = value.view() else {
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
