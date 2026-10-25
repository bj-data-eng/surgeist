//! Layout-owned inline scope proof and ancestry-prefix break preference.

use std::{collections::BTreeMap, sync::Arc};

use surgeist_text::CapturedScope;
mod context;
pub(crate) use crate::error::wrapping_error;
pub(crate) use context::{
    CanonicalScopeProof, NumericWrapping, ScopeRow, WrappingContext, WrappingScopeProof,
    native_span,
};

/// Noninherited preference for permitted breaks inside one actual inline box.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum WrapInside {
    #[default]
    Auto,
    Avoid,
}

/// Actual native endpoints or a canonical captured inline scope.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WrappingScopeInput<Node> {
    NativePair {
        start: Node,
        end: Node,
        wrap_inside: WrapInside,
    },
    Captured {
        scope: CapturedScope,
        wrap_inside: WrapInside,
    },
}

/// A refusal to associate a requested wrapping scope with this immutable flow.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum WrappingScopeError {
    NodeNotReachable,
    AmbiguousNode,
    EndpointRole,
    DifferentFlow,
    EndpointOrder,
    DuplicateScope,
    ConflictingScope,
    CrossingScopes,
    MissingCapture,
    ForeignCapture,
    NonInlineScope,
    CaptureAssociation,
    UnsupportedIncidence,
    InconsistentIncidence,
}

#[derive(Debug, Default)]
struct Ancestry {
    intervals: Vec<(usize, usize)>,
}

/// Issued only by the immutable scope context, never supplied by a caller.
#[derive(Clone, Debug)]
pub(crate) struct AvoidClass {
    ancestry: Arc<Ancestry>,
    scope: Option<usize>,
}

impl PartialEq for AvoidClass {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.ancestry, &other.ancestry) && self.scope == other.scope
    }
}
impl Eq for AvoidClass {}
impl AvoidClass {
    pub(crate) fn frontier<T>(&self) -> PreferenceFrontier<T> {
        PreferenceFrontier {
            ancestry: Arc::clone(&self.ancestry),
            entries: BTreeMap::new(),
            serial: 0,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ScopeEndpoint<Node> {
    Local(Node),
    BeforeRun,
    AfterRun,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct NativePairProjection<Node> {
    pub(crate) start: ScopeEndpoint<Node>,
    pub(crate) end: ScopeEndpoint<Node>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum NumericWrappingError {
    Scope(WrappingScopeError),
    FeedbackCycle,
}
impl From<WrappingScopeError> for NumericWrappingError {
    fn from(error: WrappingScopeError) -> Self {
        Self::Scope(error)
    }
}
pub(crate) fn numeric_wrapping_error<Node, S: crate::LayoutScalar, M>(
    node: Node,
    error: NumericWrappingError,
) -> crate::LayoutErrorOf<Node, S, M> {
    match error {
        NumericWrappingError::Scope(error) => wrapping_error(node, error),
        NumericWrappingError::FeedbackCycle => crate::LayoutErrorOf::new(
            crate::LayoutErrorSiteOf::Node(node),
            crate::LayoutOperation::InlineWrapping,
            crate::LayoutErrorKindOf::InlineFeedbackCycle,
        ),
    }
}

/// Retains the normal preferred real receipt in each nondominated scope class.
/// Insertion order is the consuming fitter's established normal preference.
pub(crate) struct PreferenceFrontier<T> {
    ancestry: Arc<Ancestry>,
    entries: BTreeMap<usize, (usize, usize, T)>,
    serial: usize,
}
impl<T> PreferenceFrontier<T> {
    pub(crate) fn consider(
        &mut self,
        class: AvoidClass,
        receipt: T,
    ) -> Result<(), WrappingScopeError> {
        if !Arc::ptr_eq(&self.ancestry, &class.ancestry) {
            return Err(WrappingScopeError::InconsistentIncidence);
        }
        let (key, end) = class
            .scope
            .map_or((0, usize::MAX), |i| self.ancestry.intervals[i]);
        if let Some((_, &(prior_end, _, _))) = self.entries.range(..key).next_back()
            && key < prior_end
        {
            return Ok(());
        }
        let dominated: Vec<_> = self.entries.range(key..end).map(|(&k, _)| k).collect();
        for key in dominated {
            self.entries.remove(&key);
        }
        self.entries.insert(key, (end, self.serial, receipt));
        self.serial += 1;
        Ok(())
    }
    pub(crate) fn into_preferred(self) -> Option<T> {
        self.entries
            .into_values()
            .max_by_key(|(_, serial, _)| *serial)
            .map(|(_, _, receipt)| receipt)
    }
}

#[cfg(test)]
mod tests;
