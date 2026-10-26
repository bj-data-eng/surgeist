//! Checked authored generalized conditions and conditional group fragments.
use crate::component_values::CssCanonicalBuilder;
use crate::*;
use std::fmt;

/// A leaf of a `@when` condition: one `<boolean-condition-in-parens>` operand of
/// Surgeist's bounded Conditional 5 authored profile that is not itself a
/// negation, junction or explicit grouping.
///
/// Supported tests and opaque terms remain symbolic and immutable. This does not
/// evaluate a condition or infer host support.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssWhenLeaf {
    MediaFeature(CssWhenMediaFeature),
    SupportsDeclaration(Box<CssSupportsDeclaration>),
    GeneralEnclosed(CssGeneralEnclosed),
}
/// Exactly one media feature inside its real `media()` function component.
#[derive(Clone, Debug, PartialEq)]
pub struct CssWhenMediaFeature {
    kind: CssWhenMediaFeatureKind,
    component: Box<CssComponentValue>,
}
/// Known typed feature syntax or a structurally valid feature with unknown truth.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssWhenMediaFeatureKind {
    Feature(CssMediaFeatureQuery),
    UnknownFeature(CssUnknownMediaFeature),
}
impl CssWhenMediaFeature {
    pub(crate) fn new(kind: CssWhenMediaFeatureKind, component: CssComponentValue) -> Self {
        Self {
            kind,
            component: Box::new(component),
        }
    }
    /// Checks the actual function and its single feature, without opaque fallback.
    pub fn try_from_component(
        component: CssComponentValue,
        limits: CssComponentValueLimits,
    ) -> Result<Self, CssConditionConstructionError> {
        let values = CssComponentValues::try_new(vec![component.clone()], limits)?;
        construct(values, |values| {
            reject_recovered(&values)?;
            crate::parser::construct_when_media_feature(component, limits)
        })
    }
    #[must_use]
    pub const fn kind(&self) -> &CssWhenMediaFeatureKind {
        &self.kind
    }
    #[must_use]
    pub const fn component(&self) -> &CssComponentValue {
        &self.component
    }
    #[must_use]
    pub fn origin(&self) -> &CssValueOrigin {
        self.component.origin()
    }
    #[must_use]
    pub fn position(&self) -> Option<CssSourcePosition> {
        crate::media::parsed_position(self.origin())
    }
}
/// A `@when` condition or general-enclosed component that failed checked admission
/// through its grammar, recovered input or a resource.
///
/// The failure is read through the `is_*` predicates and located by [`Self::origin`]; a
/// rejected component is exposed by [`Self::component_error`] and is the error's source.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssConditionConstructionError {
    kind: ConditionConstructionKind,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ConditionConstructionKind {
    InvalidConditionGrammar { origin: CssValueOrigin },
    InvalidMediaFeatureGrammar { origin: CssValueOrigin },
    RecoveredInput { origin: CssValueOrigin },
    WrongOuterComponent { origin: CssValueOrigin },
    Component(CssComponentValueError),
    WorkerUnavailable { origin: CssValueOrigin },
}
impl CssConditionConstructionError {
    crate::error::kind_predicates! { kind;
        /// Returns whether the components do not form a condition.
        is_invalid_condition_grammar => ConditionConstructionKind::InvalidConditionGrammar { .. },
        /// Returns whether a `media()` query does not form a media feature.
        is_invalid_media_feature_grammar =>
            ConditionConstructionKind::InvalidMediaFeatureGrammar { .. },
        /// Returns whether an implicitly closed component was supplied.
        is_recovered_input => ConditionConstructionKind::RecoveredInput { .. },
        /// Returns whether a general-enclosed component is neither a function nor a
        /// parenthesis block.
        is_wrong_outer_component => ConditionConstructionKind::WrongOuterComponent { .. },
        /// Returns whether the bounded deep-construction worker could not be started.
        is_worker_unavailable => ConditionConstructionKind::WorkerUnavailable { .. },
    }
    /// Returns the rejected component.
    #[must_use]
    pub const fn component_error(&self) -> Option<&CssComponentValueError> {
        match &self.kind {
            ConditionConstructionKind::Component(error) => Some(error),
            _ => None,
        }
    }
    /// Returns the responsible original or programmatic origin.
    #[must_use]
    pub const fn origin(&self) -> &CssValueOrigin {
        match &self.kind {
            ConditionConstructionKind::InvalidConditionGrammar { origin }
            | ConditionConstructionKind::InvalidMediaFeatureGrammar { origin }
            | ConditionConstructionKind::RecoveredInput { origin }
            | ConditionConstructionKind::WrongOuterComponent { origin }
            | ConditionConstructionKind::WorkerUnavailable { origin } => origin,
            ConditionConstructionKind::Component(error) => error.origin(),
        }
    }
    pub(crate) fn into_kind(self) -> ConditionConstructionKind {
        self.kind
    }
}
impl From<ConditionConstructionKind> for CssConditionConstructionError {
    fn from(kind: ConditionConstructionKind) -> Self {
        Self { kind }
    }
}
impl fmt::Display for CssConditionConstructionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let reason = match &self.kind {
            ConditionConstructionKind::WrongOuterComponent { .. } => {
                return f.write_str("expected a function or parenthesis component");
            }
            ConditionConstructionKind::InvalidConditionGrammar { .. } => {
                "the components do not form a condition"
            }
            ConditionConstructionKind::InvalidMediaFeatureGrammar { .. } => {
                "the media() query does not form a media feature"
            }
            ConditionConstructionKind::RecoveredInput { .. } => {
                "an implicitly closed component is not admitted"
            }
            ConditionConstructionKind::Component(_) => "a component is invalid",
            ConditionConstructionKind::WorkerUnavailable { .. } => {
                "the construction thread could not be started"
            }
        };
        write!(f, "invalid authored when construction: {reason}")
    }
}
impl std::error::Error for CssConditionConstructionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.component_error()
            .map(|error| error as &(dyn std::error::Error + 'static))
    }
}
impl From<CssComponentValueError> for CssConditionConstructionError {
    fn from(error: CssComponentValueError) -> Self {
        Self {
            kind: ConditionConstructionKind::Component(error),
        }
    }
}
pub(crate) fn reject_recovered(
    values: &CssComponentValues,
) -> Result<(), CssConditionConstructionError> {
    if let Some(origin) = values.first_implicit_origin() {
        return Err(CssConditionConstructionError::from(
            crate::when::ConditionConstructionKind::RecoveredInput {
                origin: origin.clone(),
            },
        ));
    }
    Ok(())
}
fn construct<T: Send>(
    values: CssComponentValues,
    action: impl FnOnce(CssComponentValues) -> Result<T, CssConditionConstructionError> + Send,
) -> Result<T, CssConditionConstructionError> {
    if values.nesting_depth() < 32 {
        return action(values);
    }
    let origin = values
        .items()
        .first()
        .map_or(CssValueOrigin::Programmatic, |value| value.origin().clone());
    let mut values = Some(values);
    std::thread::scope(|scope| {
        let worker = std::thread::Builder::new()
            .name("surgeist-css-when".into())
            .stack_size(16 * 1024 * 1024)
            .spawn_scoped(scope, || {
                action(values.take().expect("single construction worker"))
            })
            .map_err(|_| {
                CssConditionConstructionError::from(
                    crate::when::ConditionConstructionKind::WorkerUnavailable { origin },
                )
            })?;
        match worker.join() {
            Ok(result) => result,
            Err(panic) => std::panic::resume_unwind(panic),
        }
    })
}
impl CssCondition<CssWhenLeaf> {
    /// Checks the complete adopted algebra and its actual lexical input.
    pub fn try_from_components(
        values: CssComponentValues,
        limits: CssComponentValueLimits,
    ) -> Result<Self, CssConditionConstructionError> {
        Self::try_in_context(values, limits, CssParserContext::default())
    }
    pub(crate) fn try_in_context(
        values: CssComponentValues,
        limits: CssComponentValueLimits,
        context: CssParserContext,
    ) -> Result<Self, CssConditionConstructionError> {
        construct(values, |values| {
            values.validate_with_limits(limits)?;
            reject_recovered(&values)?;
            crate::parser::construct_when_condition(values, limits, context)
        })
    }
    pub(crate) fn append_specified(
        &self,
        context: &mut crate::specified_serialization::SpecifiedSerializationContext,
        output: &mut String,
    ) -> Result<(), CssSerializationError> {
        context.charge_nodes(1)?;
        crate::component_values::charge_specified_components(context, self.components())?;
        if context.output_suppressed() {
            return Ok(());
        }
        let mut out = CssCanonicalBuilder::new(context.remaining_bytes());
        out.push_components(crate::component_values::specified_root_components(
            self.components(),
        ))
        .map_err(crate::component_values::specified_component_error)?;
        let value = out
            .finish()
            .map_err(crate::component_values::specified_component_error)?;
        context.append(output, value.as_css())
    }
}
/// Checks condition text without evaluating its supported or opaque tests.
pub fn parse_when_condition(
    source: &str,
) -> Result<CssCondition<CssWhenLeaf>, CssConditionConstructionError> {
    parse_when_condition_with_limits(source, CssComponentValueLimits::default())
}
pub fn parse_when_condition_with_limits(
    source: &str,
    limits: CssComponentValueLimits,
) -> Result<CssCondition<CssWhenLeaf>, CssConditionConstructionError> {
    CssParserContext::default().parse_when_condition_with_limits(source, limits)
}
impl CssParserContext {
    pub fn parse_when_condition(
        self,
        source: &str,
    ) -> Result<CssCondition<CssWhenLeaf>, CssConditionConstructionError> {
        self.parse_when_condition_with_limits(source, CssComponentValueLimits::default())
    }
    pub fn parse_when_condition_with_limits(
        self,
        source: &str,
        limits: CssComponentValueLimits,
    ) -> Result<CssCondition<CssWhenLeaf>, CssConditionConstructionError> {
        CssCondition::<CssWhenLeaf>::try_in_context(
            parse_component_values_with_limits(source, limits)?,
            limits,
            self,
        )
    }
}

macro_rules! conditional_rule {
    ($name:ident, $condition:ty, $borrowed:ty, $get:ident) => {
        /// A checked authored conditional group fragment. Complete enclosing assembly
        /// rechecks actual sibling adjacency and nearest-style placement atomically.
        #[derive(Clone, Debug, PartialEq)]
        pub struct $name {
            condition: $condition,
            rules: Vec<CssRule>,
            position: Option<CssSourcePosition>,
        }
        impl $name {
            pub(crate) fn new(
                condition: $condition,
                rules: Vec<CssRule>,
                position: CssSourcePosition,
            ) -> Self {
                Self {
                    condition,
                    rules,
                    position: Some(position),
                }
            }
            #[must_use]
            pub fn condition(&self) -> $borrowed {
                self.condition.$get()
            }
            #[must_use]
            pub fn rules(&self) -> &[CssRule] {
                &self.rules
            }
            pub(crate) fn rules_mut(&mut self) -> &mut Vec<CssRule> {
                &mut self.rules
            }
            #[must_use]
            pub const fn position(&self) -> Option<CssSourcePosition> {
                self.position
            }
            pub fn try_new(
                condition: $condition,
                rules: Vec<CssRule>,
                context: &CssNamespaceContext,
            ) -> Result<Self, CssRuleConstructionError> {
                Self::assemble(condition, rules, context, false)
            }
            /// Checks descendants under an explicit style ancestor. Enclosing assembly
            /// rechecks the actual ancestry; no selector or style claim is fabricated.
            pub fn try_new_in_style(
                condition: $condition,
                rules: Vec<CssRule>,
                context: &CssNamespaceContext,
            ) -> Result<Self, CssRuleConstructionError> {
                Self::assemble(condition, rules, context, true)
            }
            fn assemble(
                condition: $condition,
                rules: Vec<CssRule>,
                context: &CssNamespaceContext,
                style: bool,
            ) -> Result<Self, CssRuleConstructionError> {
                crate::rule_construction::conditional_group(&rules, context, style)?;
                Ok(Self {
                    condition,
                    rules,
                    position: None,
                })
            }
        }
    };
}
impl CssCondition<CssWhenLeaf> {
    fn borrow_condition(&self) -> &Self {
        self
    }
}
conditional_rule!(
    CssWhenRule,
    CssCondition<CssWhenLeaf>,
    &CssCondition<CssWhenLeaf>,
    borrow_condition
);
conditional_rule!(
    CssElseRule,
    Option<CssCondition<CssWhenLeaf>>,
    Option<&CssCondition<CssWhenLeaf>>,
    as_ref
);
