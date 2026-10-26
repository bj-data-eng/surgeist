//! Checked authored supports construction and lexical serialization.
use crate::component_values::{CssCanonicalBuilder, CssCanonicalToken};
use crate::*;
use std::{fmt, ops::Range, sync::Arc};

/// One genuine at-keyword argument admitted by an authored `at-rule()` predicate.
///
/// The enclosing [`CssCondition<CssSupportsLeaf>`] owns grammar admission and retains the
/// complete function. Unknown names and `@charset` remain symbolic arguments;
/// this type does not determine whether a processor supports an at-rule.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssSupportsAtRule {
    component: Box<CssComponentValue>,
}
impl CssSupportsAtRule {
    pub(crate) fn from_component(component: CssComponentValue) -> Option<Self> {
        at_keyword_name(&component)?;
        Some(Self {
            component: Box::new(component),
        })
    }

    /// Returns the actual argument token, with its original spelling and origin.
    #[must_use]
    pub const fn component(&self) -> &CssComponentValue {
        &self.component
    }

    /// Returns the decoded authored name without ASCII case normalization.
    #[must_use]
    pub fn name(&self) -> &str {
        at_keyword_name(self.component()).expect("checked at-keyword argument")
    }

    #[must_use]
    pub const fn origin(&self) -> &CssValueOrigin {
        self.component().origin()
    }

    #[must_use]
    pub const fn position(&self) -> Option<CssSourcePosition> {
        crate::media::parsed_position(self.origin())
    }
}
fn at_keyword_name(component: &CssComponentValue) -> Option<&str> {
    match component.view() {
        CssComponentValueRef::Token(CssValueTokenRef::AtKeyword(name)) => Some(name),
        _ => None,
    }
}

/// A supports condition, named supports definition or test body that failed checked
/// construction through its grammar, recovered input or a resource.
///
/// The failure is read through the `is_*` predicates and located by [`Self::origin`]; a
/// rejected component is exposed by [`Self::component_error`] and is the error's source.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssSupportsConstructionError {
    kind: SupportsConstructionKind,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum SupportsConstructionKind {
    InvalidConditionGrammar { origin: CssValueOrigin },
    InvalidDeclarationGrammar { origin: CssValueOrigin },
    InvalidName { origin: CssValueOrigin },
    InvalidBodyGrammar { origin: CssValueOrigin },
    RecoveredInput { origin: CssValueOrigin },
    Component(CssComponentValueError),
    WorkerUnavailable { origin: CssValueOrigin },
}
impl CssSupportsConstructionError {
    crate::error::kind_predicates! { kind;
        /// Returns whether the components do not form a supports condition.
        is_invalid_condition_grammar => SupportsConstructionKind::InvalidConditionGrammar { .. },
        /// Returns whether the components do not form a supports declaration.
        is_invalid_declaration_grammar =>
            SupportsConstructionKind::InvalidDeclarationGrammar { .. },
        /// Returns whether a named supports condition name is invalid.
        is_invalid_name => SupportsConstructionKind::InvalidName { .. },
        /// Returns whether a named supports test body does not satisfy its grammar.
        is_invalid_body_grammar => SupportsConstructionKind::InvalidBodyGrammar { .. },
        /// Returns whether an implicitly closed component was supplied.
        is_recovered_input => SupportsConstructionKind::RecoveredInput { .. },
        /// Returns whether the bounded deep-construction worker could not be started.
        is_worker_unavailable => SupportsConstructionKind::WorkerUnavailable { .. },
    }
    /// Returns the rejected component.
    #[must_use]
    pub const fn component_error(&self) -> Option<&CssComponentValueError> {
        match &self.kind {
            SupportsConstructionKind::Component(error) => Some(error),
            _ => None,
        }
    }
    /// Returns the responsible original or programmatic origin.
    #[must_use]
    pub const fn origin(&self) -> &CssValueOrigin {
        match &self.kind {
            SupportsConstructionKind::InvalidConditionGrammar { origin }
            | SupportsConstructionKind::InvalidDeclarationGrammar { origin }
            | SupportsConstructionKind::InvalidName { origin }
            | SupportsConstructionKind::InvalidBodyGrammar { origin }
            | SupportsConstructionKind::RecoveredInput { origin }
            | SupportsConstructionKind::WorkerUnavailable { origin } => origin,
            SupportsConstructionKind::Component(error) => error.origin(),
        }
    }
}
impl CssSupportsConstructionError {
    pub(crate) fn into_kind(self) -> SupportsConstructionKind {
        self.kind
    }
}
impl From<SupportsConstructionKind> for CssSupportsConstructionError {
    fn from(kind: SupportsConstructionKind) -> Self {
        Self { kind }
    }
}
impl fmt::Display for CssSupportsConstructionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("invalid authored supports construction: ")?;
        f.write_str(match &self.kind {
            SupportsConstructionKind::InvalidConditionGrammar { .. } => {
                "the components do not form a supports condition"
            }
            SupportsConstructionKind::InvalidDeclarationGrammar { .. } => {
                "the components do not form a supports declaration"
            }
            SupportsConstructionKind::InvalidName { .. } => {
                "the named supports condition name is invalid"
            }
            SupportsConstructionKind::InvalidBodyGrammar { .. } => {
                "the named supports test body is invalid"
            }
            SupportsConstructionKind::RecoveredInput { .. } => {
                "an implicitly closed component is not admitted"
            }
            SupportsConstructionKind::Component(_) => "a component is invalid",
            SupportsConstructionKind::WorkerUnavailable { .. } => {
                "the construction thread could not be started"
            }
        })
    }
}
impl std::error::Error for CssSupportsConstructionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.component_error()
            .map(|error| error as &(dyn std::error::Error + 'static))
    }
}
impl From<CssComponentValueError> for CssSupportsConstructionError {
    fn from(error: CssComponentValueError) -> Self {
        Self {
            kind: SupportsConstructionKind::Component(error),
        }
    }
}

/// Each returned node selects siblings in one immutable lexical root.
#[derive(Clone)]
pub(crate) struct SupportsLexical {
    root: Arc<CssComponentValues>,
    parent: Vec<usize>,
    range: Range<usize>,
}
impl CssRetainedSources for SupportsLexical {
    fn visit_retained_sources(
        &self,
        sources: &mut CssSourceRetention,
    ) -> Result<(), CssSourceRetentionError> {
        // A selected lexical window still owns its complete shared root.
        sources.visit(self.root.as_ref())
    }
}
impl fmt::Debug for SupportsLexical {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("SupportsLexical")
            .field(&self.items())
            .finish()
    }
}
impl PartialEq for SupportsLexical {
    fn eq(&self, other: &Self) -> bool {
        self.items() == other.items()
    }
}
impl SupportsLexical {
    pub(crate) fn root(values: CssComponentValues) -> Self {
        let length = values.items().len();
        Self {
            root: Arc::new(values),
            parent: Vec::new(),
            range: 0..length,
        }
    }
    pub(crate) fn items(&self) -> &[CssComponentValue] {
        let values = if self.parent.is_empty() {
            self.root.items()
        } else {
            match self
                .root
                .component_at_path(&self.parent)
                .expect("validated lexical parent")
                .view()
            {
                CssComponentValueRef::Function(v) => v.values().items(),
                CssComponentValueRef::Block(v) => v.values().items(),
                _ => unreachable!("lexical parent owns children"),
            }
        };
        &values[self.range.clone()]
    }
    pub(crate) fn select(&self, range: Range<usize>) -> Self {
        Self {
            root: Arc::clone(&self.root),
            parent: self.parent.clone(),
            range: self.range.start + range.start..self.range.start + range.end,
        }
    }
    pub(crate) fn children(&self, index: usize) -> Self {
        let mut parent = self.parent.clone();
        parent.push(self.range.start + index);
        let length = match self.items()[index].view() {
            CssComponentValueRef::Function(v) => v.values().items().len(),
            CssComponentValueRef::Block(v) => v.values().items().len(),
            _ => unreachable!("checked lexical enclosure"),
        };
        Self {
            root: Arc::clone(&self.root),
            parent,
            range: 0..length,
        }
    }
    pub(crate) fn first_origin(&self) -> &CssValueOrigin {
        self.items()
            .iter()
            .find(|v| !trivia(v))
            .map_or(&CssValueOrigin::Programmatic, CssComponentValue::origin)
    }
}
pub(crate) fn trivia(value: &CssComponentValue) -> bool {
    matches!(
        value.view(),
        CssComponentValueRef::Comment(_)
            | CssComponentValueRef::Token(CssValueTokenRef::Whitespace(_))
    )
}
fn construct<T: Send>(
    values: CssComponentValues,
    action: impl FnOnce(CssComponentValues) -> Result<T, CssSupportsConstructionError> + Send,
) -> Result<T, CssSupportsConstructionError> {
    if values.nesting_depth() < 32 {
        return action(values);
    }
    // Keep ownership outside the closure until the worker starts.
    let mut values = Some(values);
    std::thread::scope(|scope| {
        let worker = std::thread::Builder::new()
            .name("surgeist-css-supports".into())
            .stack_size(16 * 1024 * 1024)
            .spawn_scoped(scope, || {
                action(values.take().expect("single worker owns input"))
            })
            .map_err(|_| {
                CssSupportsConstructionError::from(
                    crate::supports::SupportsConstructionKind::WorkerUnavailable {
                        origin: CssValueOrigin::Programmatic,
                    },
                )
            })?;
        match worker.join() {
            Ok(result) => result,
            Err(panic) => std::panic::resume_unwind(panic),
        }
    })
}
impl CssCondition<CssSupportsLeaf> {
    pub(crate) fn try_from_components_in_context(
        values: CssComponentValues,
        namespaces: &CssNamespaceContext,
        limits: CssComponentValueLimits,
        parser_context: CssParserContext,
    ) -> Result<Self, CssSupportsConstructionError> {
        construct(values, |values| {
            crate::parser::construct_supports_condition_with_context(
                values,
                namespaces,
                limits,
                parser_context,
            )
        })
    }

    pub fn try_from_components(
        values: CssComponentValues,
        context: &CssNamespaceContext,
        limits: CssComponentValueLimits,
    ) -> Result<Self, CssSupportsConstructionError> {
        construct(values, |values| {
            crate::parser::construct_supports_condition(values, context, limits)
        })
    }
    /// Serializes original lexical spelling, trivia and grouping. Bare import
    /// declarations acquire programmatic parentheses to form a standalone condition.
    ///
    /// `limits.max_css_bytes()` caps the output, including inserted syntax; the other component
    /// limits do not apply.
    pub fn serialize_tokens(
        &self,
        limits: crate::CssComponentValueLimits,
    ) -> Result<CssSerializedValue, CssComponentValueError> {
        let max_css_bytes = limits.max_css_bytes();
        let mut out = CssCanonicalBuilder::new(max_css_bytes);
        self.emit_lexical(&mut out, self.components())?;
        out.finish()
    }
    pub(crate) fn append_specified(
        &self,
        context: &mut crate::specified_serialization::SpecifiedSerializationContext,
        output: &mut String,
    ) -> Result<(), CssSerializationError> {
        // The lexical region owns one aggregate; typed views do not add nodes.
        // Standalone parentheses for bare import forms are owner punctuation:
        // they consume bytes, without inventing retained/projection graph nodes.
        context.charge_nodes(1)?;
        crate::component_values::charge_specified_components(context, self.components())?;
        if context.output_suppressed() {
            return Ok(());
        }
        let mut out = CssCanonicalBuilder::new(context.remaining_bytes());
        self.emit_lexical(
            &mut out,
            crate::component_values::specified_root_components(self.components()),
        )
        .map_err(crate::component_values::specified_component_error)?;
        let value = out
            .finish()
            .map_err(crate::component_values::specified_component_error)?;
        context.append(output, value.as_css())
    }
    fn emit_lexical(
        &self,
        out: &mut CssCanonicalBuilder,
        components: &[CssComponentValue],
    ) -> Result<(), CssComponentValueError> {
        if matches!(
            self.authored_form(),
            crate::SupportsAuthoredForm::BareDeclaration | crate::SupportsAuthoredForm::BareName
        ) {
            out.push_grammar(CssCanonicalToken::OpenParen, &CssValueOrigin::Programmatic)?;
        }
        out.push_components(components)?;
        if matches!(
            self.authored_form(),
            crate::SupportsAuthoredForm::BareDeclaration | crate::SupportsAuthoredForm::BareName
        ) {
            out.push_grammar(CssCanonicalToken::CloseParen, &CssValueOrigin::Programmatic)?;
        }
        Ok(())
    }
}
impl CssSupportsDeclaration {
    pub(crate) fn try_from_components_in_context(
        values: CssComponentValues,
        context: CssParserContext,
    ) -> Result<Self, CssSupportsConstructionError> {
        construct(values, |values| {
            crate::parser::construct_supports_declaration_with_context(
                values,
                CssComponentValueLimits::default(),
                context,
            )
        })
    }
    /// Accepts a bare declaration, excluding its surrounding parentheses.
    pub fn try_from_components(
        values: CssComponentValues,
        limits: CssComponentValueLimits,
    ) -> Result<Self, CssSupportsConstructionError> {
        construct(values, |values| {
            crate::parser::construct_supports_declaration(values, limits)
        })
    }
    /// Preserves original lexical spelling and trivia, including terminal importance.
    /// Parsed implicit closures serialize explicitly; checked construction rejects them.
    ///
    /// `limits.max_css_bytes()` caps the output, including inserted syntax; the other component
    /// limits do not apply.
    pub fn serialize_tokens(
        &self,
        limits: crate::CssComponentValueLimits,
    ) -> Result<CssSerializedValue, CssComponentValueError> {
        let max_css_bytes = limits.max_css_bytes();
        let mut out = CssCanonicalBuilder::new(max_css_bytes);
        out.push_components(self.components())?;
        out.finish()
    }
}
