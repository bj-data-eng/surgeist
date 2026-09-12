//! Checked authored supports construction and lexical serialization.
use crate::component_values::{CssCanonicalBuilder, CssCanonicalToken};
use crate::*;
use std::{fmt, ops::Range, sync::Arc};

/// A supports grammar, recovered-input, or resource construction failure.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssSupportsConstructionError {
    InvalidConditionGrammar {
        origin: CssValueOrigin,
    },
    InvalidDeclarationGrammar {
        origin: CssValueOrigin,
    },
    RecoveredInput {
        origin: CssValueOrigin,
    },
    Component(CssComponentValueError),
    /// The bounded deep-construction worker could not be started.
    WorkerUnavailable {
        origin: CssValueOrigin,
    },
}
impl CssSupportsConstructionError {
    pub fn origin(&self) -> &CssValueOrigin {
        match self {
            Self::InvalidConditionGrammar { origin }
            | Self::InvalidDeclarationGrammar { origin }
            | Self::RecoveredInput { origin }
            | Self::WorkerUnavailable { origin } => origin,
            Self::Component(error) => error.origin(),
        }
    }
}
impl fmt::Display for CssSupportsConstructionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid authored supports construction: {self:?}")
    }
}
impl std::error::Error for CssSupportsConstructionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Component(error) => Some(error),
            _ => None,
        }
    }
}
impl From<CssComponentValueError> for CssSupportsConstructionError {
    fn from(error: CssComponentValueError) -> Self {
        Self::Component(error)
    }
}

/// Each returned node selects siblings in one immutable lexical root.
#[derive(Clone)]
pub(crate) struct SupportsLexical {
    root: Arc<CssComponentValues>,
    parent: Vec<usize>,
    range: Range<usize>,
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
            .map_err(|_| CssSupportsConstructionError::WorkerUnavailable {
                origin: CssValueOrigin::Programmatic,
            })?;
        match worker.join() {
            Ok(result) => result,
            Err(panic) => std::panic::resume_unwind(panic),
        }
    })
}
impl CssSupportsCondition {
    pub fn try_from_components(
        values: CssComponentValues,
        context: &CssNamespaceContext,
    ) -> Result<Self, CssSupportsConstructionError> {
        Self::try_from_components_with_limits(values, context, CssComponentValueLimits::default())
    }
    pub fn try_from_components_with_limits(
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
    pub fn serialize(&self) -> Result<CssSerializedValue, CssComponentValueError> {
        self.serialize_with_limit(usize::MAX)
    }
    pub fn serialize_with_limit(
        &self,
        max_css_bytes: usize,
    ) -> Result<CssSerializedValue, CssComponentValueError> {
        let mut out = CssCanonicalBuilder::new(max_css_bytes);
        if self.bare_declaration() {
            out.push_grammar(CssCanonicalToken::OpenParen, &CssValueOrigin::Programmatic)?;
        }
        out.push_components(self.components())?;
        if self.bare_declaration() {
            out.push_grammar(CssCanonicalToken::CloseParen, &CssValueOrigin::Programmatic)?;
        }
        out.finish()
    }
}
impl CssSupportsDeclaration {
    /// Accepts a bare declaration, excluding its surrounding parentheses.
    pub fn try_from_components(
        values: CssComponentValues,
    ) -> Result<Self, CssSupportsConstructionError> {
        Self::try_from_components_with_limits(values, CssComponentValueLimits::default())
    }
    pub fn try_from_components_with_limits(
        values: CssComponentValues,
        limits: CssComponentValueLimits,
    ) -> Result<Self, CssSupportsConstructionError> {
        construct(values, |values| {
            crate::parser::construct_supports_declaration(values, limits)
        })
    }
    /// Preserves original lexical spelling and trivia, including terminal importance.
    /// Parsed implicit closures serialize explicitly; checked construction rejects them.
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
}
