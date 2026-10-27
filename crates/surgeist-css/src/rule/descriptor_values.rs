use crate::component::{
    CssComponentValueError, CssSerializedOrigin, CssSerializedValue, CssValueOrigin,
};
use crate::diagnostic::CssError;
use std::fmt;

/// The shared failure of one checked component-native descriptor construction or
/// pending reentry, mapped to the supplied component origins and never to
/// fabricated source text.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct DescriptorValueFailure {
    kind: DescriptorFailureKind,
    origin: Box<CssSerializedOrigin>,
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
            origin: Box::new(CssSerializedOrigin::Token(error.origin().clone())),
            kind: DescriptorFailureKind::Component(Box::new(error)),
        }
    }

    pub(crate) fn grammar(error: CssError, serialized: &CssSerializedValue) -> Self {
        Self {
            origin: Box::new(error.component_input_origin(serialized)),
            kind: DescriptorFailureKind::Grammar(Box::new(error)),
        }
    }

    pub(crate) fn recovered(origin: CssValueOrigin) -> Self {
        Self {
            kind: DescriptorFailureKind::RecoveredComponent,
            origin: Box::new(CssSerializedOrigin::Token(origin)),
        }
    }

    pub(crate) fn residual(origin: CssValueOrigin) -> Self {
        Self {
            kind: DescriptorFailureKind::ResidualSubstitution,
            origin: Box::new(CssSerializedOrigin::Token(origin)),
        }
    }

    pub(crate) fn not_pending() -> Self {
        Self {
            kind: DescriptorFailureKind::NotPending,
            origin: Box::new(CssSerializedOrigin::End(None)),
        }
    }

    pub(crate) const fn origin(&self) -> &CssSerializedOrigin {
        &self.origin
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
