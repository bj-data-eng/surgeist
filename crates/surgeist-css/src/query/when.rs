//! Checked authored generalized conditions and conditional group fragments.
use crate::CssParserContext;
use crate::component::values::serialize::CssCanonicalBuilder;
use crate::component::{
    CssComponentValue, CssComponentValueError, CssComponentValueLimits, CssComponentValues,
    CssValueOrigin,
};
use crate::query::media::{CssMediaFeatureQuery, CssUnknownMediaFeature};
use crate::query::supports::CssSupportsDeclaration;
use crate::query::{CssCondition, CssGeneralEnclosed};
use crate::serialization::CssSerializationError;
use crate::source::CssSourcePosition;
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
            crate::parser::queries::construct_when_media_feature(component, limits)
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
        crate::query::media::construction::parsed_position(self.origin())
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
    crate::diagnostic::error::kind_predicates! { kind;
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
            crate::query::when::ConditionConstructionKind::RecoveredInput {
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
                    crate::query::when::ConditionConstructionKind::WorkerUnavailable { origin },
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
            crate::parser::when::construct_when_condition(values, limits, context)
        })
    }
    pub(crate) fn append_specified(
        &self,
        context: &mut crate::serialization::specified_serialization::SpecifiedSerializationContext,
        output: &mut String,
    ) -> Result<(), CssSerializationError> {
        context.charge_nodes(1)?;
        crate::component::values::charge_specified_components(context, self.components())?;
        if context.output_suppressed() {
            return Ok(());
        }
        let mut out = CssCanonicalBuilder::new(context.remaining_bytes());
        out.push_components(crate::component::values::specified_root_components(
            self.components(),
        ))
        .map_err(crate::component::values::specified_component_error)?;
        let value = out
            .finish()
            .map_err(crate::component::values::specified_component_error)?;
        context.append(output, value.as_css())
    }
}
impl CssCondition<CssWhenLeaf> {
    pub(crate) fn borrow_condition(&self) -> &Self {
        self
    }
}
impl CssCondition<CssWhenLeaf> {
    /// Checks condition text without evaluating its supported or opaque tests.
    ///
    /// The parser supplies the document mode and the component limits.
    ///
    /// # Errors
    ///
    /// Returns the typed construction failure of the first test the grammar rejects.
    pub fn parse(
        parser: &crate::CssParser,
        source: impl AsRef<str>,
    ) -> Result<Self, CssConditionConstructionError> {
        let limits = parser.component_limits();
        Self::try_in_context(
            crate::component::values::parse_component_values_with_limits(source.as_ref(), limits)?,
            limits,
            parser.context(),
        )
    }
}
