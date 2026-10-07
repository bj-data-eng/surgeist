//! Checked authored generalized conditions and conditional group fragments.
use crate::component_values::CssCanonicalBuilder;
use crate::supports::SupportsLexical;
use crate::*;
use std::fmt;

/// One condition admitted by Surgeist's bounded Conditional 5 authored profile.
/// Grouping, supported tests and opaque terms remain symbolic and immutable.
/// This does not evaluate a condition or infer host support.
///
/// ```compile_fail
/// use surgeist_css::{CssWhenCondition, CssWhenConditionKind};
/// fn forge(mut value: CssWhenCondition, kind: CssWhenConditionKind) {
///     value.kind = Box::new(kind);
/// }
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct CssWhenCondition {
    kind: Box<CssWhenConditionKind>,
    lexical: SupportsLexical,
    origin: CssValueOrigin,
}
/// Inspectable authored shape; constructing a kind cannot construct a condition.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssWhenConditionKind {
    Parenthesized(Box<CssWhenCondition>),
    Not(Box<CssWhenCondition>),
    And(CssWhenConditionList),
    Or(CssWhenConditionList),
    MediaFeature(CssWhenMediaFeature),
    SupportsDeclaration(Box<CssSupportsDeclaration>),
    GeneralEnclosed(CssGeneralEnclosed),
}
/// At least two ordered terms joined by one homogeneous operator.
#[derive(Clone, Debug, PartialEq)]
pub struct CssWhenConditionList {
    conditions: Vec<CssWhenCondition>,
}
impl CssWhenConditionList {
    pub(crate) fn new(conditions: Vec<CssWhenCondition>) -> Self {
        debug_assert!(conditions.len() >= 2);
        Self { conditions }
    }
    #[must_use]
    pub fn conditions(&self) -> &[CssWhenCondition] {
        &self.conditions
    }
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
    ) -> Result<Self, CssWhenConstructionError> {
        Self::try_from_component_with_limits(component, CssComponentValueLimits::default())
    }
    pub fn try_from_component_with_limits(
        component: CssComponentValue,
        limits: CssComponentValueLimits,
    ) -> Result<Self, CssWhenConstructionError> {
        let values = CssComponentValues::try_new_with_limits(vec![component.clone()], limits)?;
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
/// A typed grammar, recovered-input or resource failure during checked admission.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssWhenConstructionError {
    InvalidConditionGrammar { origin: CssValueOrigin },
    InvalidMediaFeatureGrammar { origin: CssValueOrigin },
    RecoveredInput { origin: CssValueOrigin },
    Component(CssComponentValueError),
    WorkerUnavailable { origin: CssValueOrigin },
}
impl CssWhenConstructionError {
    #[must_use]
    pub fn origin(&self) -> &CssValueOrigin {
        match self {
            Self::InvalidConditionGrammar { origin }
            | Self::InvalidMediaFeatureGrammar { origin }
            | Self::RecoveredInput { origin }
            | Self::WorkerUnavailable { origin } => origin,
            Self::Component(error) => error.origin(),
        }
    }
}
impl fmt::Display for CssWhenConstructionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid authored when construction: {self:?}")
    }
}
impl std::error::Error for CssWhenConstructionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Component(error) => Some(error),
            _ => None,
        }
    }
}
impl From<CssComponentValueError> for CssWhenConstructionError {
    fn from(error: CssComponentValueError) -> Self {
        Self::Component(error)
    }
}
pub(crate) fn reject_recovered(
    values: &CssComponentValues,
) -> Result<(), CssWhenConstructionError> {
    if let Some(origin) = values.first_implicit_origin() {
        return Err(CssWhenConstructionError::RecoveredInput {
            origin: origin.clone(),
        });
    }
    Ok(())
}
fn construct<T: Send>(
    values: CssComponentValues,
    action: impl FnOnce(CssComponentValues) -> Result<T, CssWhenConstructionError> + Send,
) -> Result<T, CssWhenConstructionError> {
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
            .map_err(|_| CssWhenConstructionError::WorkerUnavailable { origin })?;
        match worker.join() {
            Ok(result) => result,
            Err(panic) => std::panic::resume_unwind(panic),
        }
    })
}
impl CssWhenCondition {
    pub(crate) fn new(kind: CssWhenConditionKind, lexical: SupportsLexical) -> Self {
        let origin = lexical.first_origin().clone();
        Self {
            kind: Box::new(kind),
            lexical,
            origin,
        }
    }
    pub(crate) fn into_kind(self) -> CssWhenConditionKind {
        *self.kind
    }
    #[must_use]
    pub const fn kind(&self) -> &CssWhenConditionKind {
        &self.kind
    }
    #[must_use]
    pub fn components(&self) -> &[CssComponentValue] {
        self.lexical.items()
    }
    #[must_use]
    pub const fn origin(&self) -> &CssValueOrigin {
        &self.origin
    }
    #[must_use]
    pub fn position(&self) -> Option<CssSourcePosition> {
        crate::media::parsed_position(&self.origin)
    }
    /// Checks the complete adopted algebra and its actual lexical input.
    pub fn try_from_components(
        values: CssComponentValues,
    ) -> Result<Self, CssWhenConstructionError> {
        Self::try_from_components_with_limits(values, CssComponentValueLimits::default())
    }
    pub fn try_from_components_with_limits(
        values: CssComponentValues,
        limits: CssComponentValueLimits,
    ) -> Result<Self, CssWhenConstructionError> {
        Self::try_in_context(values, limits, CssParserContext::default())
    }
    pub(crate) fn try_in_context(
        values: CssComponentValues,
        limits: CssComponentValueLimits,
        context: CssParserContext,
    ) -> Result<Self, CssWhenConstructionError> {
        construct(values, |values| {
            values.validate_with_limits(limits)?;
            reject_recovered(&values)?;
            crate::parser::construct_when_condition(values, limits, context)
        })
    }
    /// Preserves original spelling, trivia and every grouping pair.
    pub fn serialize(&self) -> Result<CssSerializedValue, CssComponentValueError> {
        self.serialize_with_limit(usize::MAX)
    }
    pub fn serialize_with_limit(
        &self,
        max_css_bytes: usize,
    ) -> Result<CssSerializedValue, CssComponentValueError> {
        let mut out = CssCanonicalBuilder::new(max_css_bytes);
        out.push_components(self.components())?;
        out.finish()
    }
    pub(crate) fn append_specified(
        &self,
        context: &mut crate::specified_serialization::SpecifiedSerializationContext,
        output: &mut String,
    ) -> Result<(), CssSpecifiedValueSerializationError> {
        context.charge_input(1)?;
        context.charge_projection(1)?;
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
pub fn parse_when_condition(source: &str) -> Result<CssWhenCondition, CssWhenConstructionError> {
    parse_when_condition_with_limits(source, CssComponentValueLimits::default())
}
pub fn parse_when_condition_with_limits(
    source: &str,
    limits: CssComponentValueLimits,
) -> Result<CssWhenCondition, CssWhenConstructionError> {
    CssParserContext::default().parse_when_condition_with_limits(source, limits)
}
impl CssParserContext {
    pub fn parse_when_condition(
        self,
        source: &str,
    ) -> Result<CssWhenCondition, CssWhenConstructionError> {
        self.parse_when_condition_with_limits(source, CssComponentValueLimits::default())
    }
    pub fn parse_when_condition_with_limits(
        self,
        source: &str,
        limits: CssComponentValueLimits,
    ) -> Result<CssWhenCondition, CssWhenConstructionError> {
        CssWhenCondition::try_in_context(
            parse_component_values_with_limits(source, limits)?,
            limits,
            self,
        )
    }
}

macro_rules! conditional_rule {
    ($name:ident, $condition:ty, $borrowed:ty, $get:ident, $rules:ty, $rule_ref:ty, $item:ty, $list:ident, $convert:path) => {
        /// A checked authored conditional group fragment. Complete enclosing assembly
        /// rechecks actual sibling adjacency and nearest-style placement atomically.
        #[derive(Clone, Debug, PartialEq)]
        pub struct $name {
            condition: $condition,
            rules: $rules,
            position: Option<CssSourcePosition>,
        }
        impl $name {
            pub(crate) fn new(
                condition: $condition,
                rules: $rules,
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
            pub fn rules(&self) -> $rule_ref {
                &self.rules
            }
            pub(crate) fn rules_mut(&mut self) -> &mut $rules {
                &mut self.rules
            }
            #[must_use]
            pub const fn position(&self) -> Option<CssSourcePosition> {
                self.position
            }
            pub fn try_new(
                condition: $condition,
                rules: Vec<$item>,
                context: &CssNamespaceContext,
            ) -> Result<Self, CssRuleConstructionError> {
                Self::assemble(condition, rules, context, false)
            }
            /// Checks descendants under an explicit style ancestor. Enclosing assembly
            /// rechecks the actual ancestry; no selector or style claim is fabricated.
            pub fn try_new_in_style(
                condition: $condition,
                rules: Vec<$item>,
                context: &CssNamespaceContext,
            ) -> Result<Self, CssRuleConstructionError> {
                Self::assemble(condition, rules, context, true)
            }
            fn assemble(
                condition: $condition,
                rules: Vec<$item>,
                context: &CssNamespaceContext,
                style: bool,
            ) -> Result<Self, CssRuleConstructionError> {
                crate::rule_construction::conditional_group(
                    crate::rule_construction::List::$list(&rules),
                    context,
                    style,
                )?;
                Ok(Self {
                    condition,
                    rules: $convert(rules),
                    position: None,
                })
            }
        }
    };
}
impl CssWhenCondition {
    fn borrow_condition(&self) -> &Self {
        self
    }
}
conditional_rule!(
    CssWhenRule,
    CssWhenCondition,
    &CssWhenCondition,
    borrow_condition,
    Vec<CssRule>,
    &[CssRule],
    CssRule,
    Ordinary,
    std::convert::identity
);
conditional_rule!(
    CssElseRule,
    Option<CssWhenCondition>,
    Option<&CssWhenCondition>,
    as_ref,
    Vec<CssRule>,
    &[CssRule],
    CssRule,
    Ordinary,
    std::convert::identity
);
conditional_rule!(
    CssScopedWhenRule,
    CssWhenCondition,
    &CssWhenCondition,
    borrow_condition,
    CssScopedRuleList,
    &CssScopedRuleList,
    CssScopedRule,
    Scoped,
    CssScopedRuleList::from_rules
);
conditional_rule!(
    CssScopedElseRule,
    Option<CssWhenCondition>,
    Option<&CssWhenCondition>,
    as_ref,
    CssScopedRuleList,
    &CssScopedRuleList,
    CssScopedRule,
    Scoped,
    CssScopedRuleList::from_rules
);
