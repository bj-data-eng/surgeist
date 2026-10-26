//! Layout-owned inline scope proof and ancestry-prefix break preference.

use std::{collections::BTreeMap, sync::Arc};
mod declarations;
pub use declarations::{
    ChildOrderCompleteness, GeneratedBoxParticipation, GeneratedBoxRef, GeneratedChildBoxInput,
    InlineBoxRef, InlineChildOrderInput, WrapEdge, WrapEdges, WrappingInput,
};
pub(crate) use declarations::{EdgeEffect, EdgePreferences, WrappingMode};

use surgeist_text::CapturedScope;
mod context;
mod edge_plan;
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
        edges: WrapEdges,
    },
    Captured {
        scope: CapturedScope,
        wrap_inside: WrapInside,
        edges: WrapEdges,
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
    ChildOrder,
    MissingChildOrder,
    SourceProcessingRequired,
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
    edges: Vec<usize>,
}

impl PartialEq for AvoidClass {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.ancestry, &other.ancestry)
            && self.scope == other.scope
            && self.edges == other.edges
    }
}
impl Eq for AvoidClass {}
impl AvoidClass {
    pub(crate) fn add_edge(&mut self, edge: usize) {
        if !self.edges.contains(&edge) {
            self.edges.push(edge);
        }
    }
    pub(crate) fn frontier<T>(&self) -> PreferenceFrontier<T> {
        PreferenceFrontier {
            ancestry: Arc::clone(&self.ancestry),
            entries: FrontierEntries::Inside(BTreeMap::new()),
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
    entries: FrontierEntries<T>,
    serial: usize,
}
struct FrontierEntry<T> {
    class: AvoidClass,
    serial: usize,
    receipt: T,
}
enum FrontierEntries<T> {
    Inside(BTreeMap<usize, FrontierEntry<T>>),
    Mixed(Vec<FrontierEntry<T>>),
}
impl AvoidClass {
    fn interval(&self) -> (usize, usize) {
        self.scope
            .map_or((0, usize::MAX), |i| self.ancestry.intervals[i])
    }
    fn subset(&self, other: &Self) -> bool {
        let inside = match (self.scope, other.scope) {
            (None, _) => true,
            (Some(_), None) => false,
            (Some(_), Some(_)) => {
                let (a, ae) = self.interval();
                let (b, be) = other.interval();
                a <= b && be <= ae
            }
        };
        inside && self.edges.iter().all(|e| other.edges.contains(e))
    }
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
        if !class.edges.is_empty() && matches!(self.entries, FrontierEntries::Inside(_)) {
            // Empty-edge ancestors that already dominated a receipt continue
            // to dominate it when edge constraints arrive. Transfer only the
            // retained frontier, keeping its true normal-preference serials.
            let prior = std::mem::replace(&mut self.entries, FrontierEntries::Mixed(Vec::new()));
            self.entries = match prior {
                FrontierEntries::Inside(entries) => {
                    FrontierEntries::Mixed(entries.into_values().collect())
                }
                mixed => mixed,
            };
        }
        match &mut self.entries {
            FrontierEntries::Inside(entries) => {
                let (key, end) = class.interval();
                if let Some((_, prior)) = entries.range(..key).next_back()
                    && key < prior.class.interval().1
                {
                    return Ok(());
                }
                let dominated = entries
                    .range(key..end)
                    .map(|(&key, _)| key)
                    .collect::<Vec<_>>();
                for key in dominated {
                    entries.remove(&key);
                }
                entries.insert(
                    key,
                    FrontierEntry {
                        class,
                        serial: self.serial,
                        receipt,
                    },
                );
            }
            FrontierEntries::Mixed(entries) => {
                if entries
                    .iter()
                    .any(|prior| prior.class.subset(&class) && !class.subset(&prior.class))
                {
                    return Ok(());
                }
                entries.retain(|prior| !class.subset(&prior.class));
                entries.push(FrontierEntry {
                    class,
                    serial: self.serial,
                    receipt,
                });
            }
        }
        self.serial += 1;
        Ok(())
    }
    pub(crate) fn into_preferred(self) -> Option<T> {
        match self.entries {
            FrontierEntries::Inside(entries) => entries
                .into_values()
                .max_by_key(|entry| entry.serial)
                .map(|entry| entry.receipt),
            FrontierEntries::Mixed(entries) => entries
                .into_iter()
                .max_by_key(|entry| entry.serial)
                .map(|entry| entry.receipt),
        }
    }
}

#[cfg(test)]
mod edge_preference_tests;
#[cfg(test)]
mod tests;
