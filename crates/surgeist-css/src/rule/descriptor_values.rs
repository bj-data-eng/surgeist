use crate::component::{
    CssComponentValueError, CssComponentValueLimits, CssComponentValues, CssSerializedOrigin,
    CssSerializedValue, CssValueOrigin,
};
use crate::diagnostic::CssError;
use crate::source::Provenance;
use std::fmt;

/// A descriptor value admitted from checked components, in one phase of one
/// descriptor family.
///
/// The family names its descriptor selector, the error it reports, whether a
/// value still awaits substitution, and the grammar that admits components;
/// [`admit`] and [`reenter`] own everything else of checked construction and
/// pending reentry.
pub(crate) trait CheckedDescriptorValue: Sized {
    /// The descriptor selector: a descriptor kind, or `()` for a family that
    /// has one descriptor.
    type Kind: Copy;
    /// The family's public error, built from the shared failure.
    type Error: From<DescriptorValueFailure>;

    /// The descriptor selector this value was admitted under.
    fn selector(&self) -> Self::Kind;

    /// Whether the value is symbolic and may be reentered after substitution.
    fn is_pending(&self) -> bool;

    /// Admits `components`, which are within limits and closed, under the
    /// family's descriptor grammar, retaining them with no raw-source window.
    /// `serialized` is their serialization, which positions of the failure index.
    fn from_checked(
        kind: Self::Kind,
        components: CssComponentValues,
        serialized: &CssSerializedValue,
    ) -> Result<Self, CssError>;
}

/// Admits `components` as a checked descriptor value: limits, then the refusal
/// of recovered components, then the descriptor grammar.
pub(crate) fn admit<V: CheckedDescriptorValue>(
    kind: V::Kind,
    components: CssComponentValues,
    limits: CssComponentValueLimits,
) -> Result<V, V::Error> {
    components
        .validate_with_limits(limits)
        .map_err(DescriptorValueFailure::component)?;
    components.ensure_closed(|origin| DescriptorValueFailure::recovered(origin.clone()))?;
    admit_checked(kind, components, limits)
}

/// Rechecks a replacement for a pending value without substituting: the
/// receiver must be pending, then the replacement faces the limits, the refusal
/// of recovered components and of residual substitutions, and the grammar.
pub(crate) fn reenter<V: CheckedDescriptorValue>(
    pending: &V,
    replacement: CssComponentValues,
    limits: CssComponentValueLimits,
) -> Result<V, V::Error> {
    if !pending.is_pending() {
        return Err(DescriptorValueFailure::not_pending().into());
    }
    replacement
        .validate_with_limits(limits)
        .map_err(DescriptorValueFailure::component)?;
    replacement.ensure_closed(|origin| DescriptorValueFailure::recovered(origin.clone()))?;
    if let Some(origin) = crate::parser::variables::first_substitution_origin(&replacement) {
        return Err(DescriptorValueFailure::residual(origin).into());
    }
    admit_checked(pending.selector(), replacement, limits)
}

fn admit_checked<V: CheckedDescriptorValue>(
    kind: V::Kind,
    components: CssComponentValues,
    limits: CssComponentValueLimits,
) -> Result<V, V::Error> {
    let serialized = components
        .serialize_tokens(limits)
        .map_err(DescriptorValueFailure::component)?;
    V::from_checked(kind, components, &serialized)
        .map_err(|error| DescriptorValueFailure::grammar(error, &serialized).into())
}

/// The shared failure of one checked component-native descriptor construction or
/// pending reentry, mapped to the supplied component origins and never to
/// fabricated source text.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct DescriptorValueFailure {
    kind: DescriptorFailureKind,
    origin: Provenance<Box<CssSerializedOrigin>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum DescriptorFailureKind {
    Component(Box<CssComponentValueError>),
    Grammar(Box<CssError>),
    RecoveredComponent,
    ResidualSubstitution,
    NotPending,
}

impl DescriptorValueFailure {
    pub(crate) fn component(error: CssComponentValueError) -> Self {
        Self {
            origin: Box::new(CssSerializedOrigin::Token(error.origin().clone())).into(),
            kind: DescriptorFailureKind::Component(Box::new(error)),
        }
    }

    pub(crate) fn grammar(error: CssError, serialized: &CssSerializedValue) -> Self {
        Self {
            origin: Box::new(error.component_input_origin(serialized)).into(),
            kind: DescriptorFailureKind::Grammar(Box::new(error)),
        }
    }

    pub(crate) fn recovered(origin: CssValueOrigin) -> Self {
        Self {
            kind: DescriptorFailureKind::RecoveredComponent,
            origin: Box::new(CssSerializedOrigin::Token(origin)).into(),
        }
    }

    pub(crate) fn residual(origin: CssValueOrigin) -> Self {
        Self {
            kind: DescriptorFailureKind::ResidualSubstitution,
            origin: Box::new(CssSerializedOrigin::Token(origin)).into(),
        }
    }

    pub(crate) fn not_pending() -> Self {
        Self {
            kind: DescriptorFailureKind::NotPending,
            origin: Box::new(CssSerializedOrigin::End(None)).into(),
        }
    }

    pub(crate) fn origin(&self) -> &CssSerializedOrigin {
        self.origin.inner()
    }

    pub(crate) fn component_error(&self) -> Option<&CssComponentValueError> {
        match &self.kind {
            DescriptorFailureKind::Component(error) => Some(error),
            _ => None,
        }
    }

    pub(crate) fn diagnostic(&self) -> Option<&CssError> {
        match &self.kind {
            DescriptorFailureKind::Grammar(error) => Some(error),
            _ => None,
        }
    }

    pub(crate) const fn is_recovered_component(&self) -> bool {
        matches!(self.kind, DescriptorFailureKind::RecoveredComponent)
    }

    pub(crate) const fn is_residual_substitution(&self) -> bool {
        matches!(self.kind, DescriptorFailureKind::ResidualSubstitution)
    }

    pub(crate) const fn is_not_pending(&self) -> bool {
        matches!(self.kind, DescriptorFailureKind::NotPending)
    }

    /// One summary sentence naming the owning rule, without the source's text.
    pub(crate) fn describe(&self, rule: &str, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "invalid {rule} descriptor value: ")?;
        formatter.write_str(match self.kind {
            DescriptorFailureKind::Component(_) => "a supplied component is invalid",
            DescriptorFailureKind::Grammar(_) => "the descriptor grammar rejected it",
            DescriptorFailureKind::RecoveredComponent => {
                "an implicitly closed component is not admitted"
            }
            DescriptorFailureKind::ResidualSubstitution => "it still contains a substitution",
            DescriptorFailureKind::NotPending => "it is not pending substitution",
        })
    }

    pub(crate) fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.kind {
            DescriptorFailureKind::Component(error) => Some(&**error),
            DescriptorFailureKind::Grammar(error) => Some(&**error),
            _ => None,
        }
    }
}
