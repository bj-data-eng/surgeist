//! Layout-owned inline scope proof and ancestry-prefix break preference.

use std::{cmp::Ordering, collections::BTreeMap, sync::Arc};
pub(crate) mod balance;
pub(crate) mod body;
mod declarations;
pub(crate) mod quality;
pub(crate) mod tail;
pub use declarations::{
    ChildOrderCompleteness, GeneratedBoxParticipation, GeneratedBoxRef, GeneratedChildBoxInput,
    InlineBoxMetricContribution, InlineBoxRef, InlineChildOrderInput, WrapEdge, WrapEdges,
    WrappingInput,
};
pub(crate) use declarations::{EdgeEffect, EdgePreferences, WrappingMode};

use surgeist_text::CapturedScope;
mod canonical_order;
mod context;
pub(crate) use canonical_order::{
    CanonicalBox, CanonicalCoverage, CanonicalOrderBindings, CanonicalOrderInventory,
};
mod edge_plan;
mod source;
pub(crate) use crate::error::wrapping_error;
pub(crate) use context::{
    CanonicalScopeProof, NumericWrapping, ScopeRow, WrappingContext, WrappingScopeProof,
    native_span,
};
pub use source::{
    ProcessedWrappingSource, ResolvedWrapEdges, SourceOccurrenceAssociations,
    SourceScopeAssociation, SourceWrappingScopeInput, WrapEdgeSource, WrapEdgeSourceNodeView,
    process_wrapping_source,
};

/// Noninherited preference for permitted breaks inside one actual inline box.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum WrapInside {
    #[default]
    Auto,
    Avoid,
}

/// Actual native endpoints or a canonical captured inline scope.
#[derive(Clone, Debug, PartialEq)]
pub enum WrappingScopeInput<Node> {
    NativePair {
        start: Node,
        end: Node,
        wrap_inside: WrapInside,
        edges: WrapEdges,
        metric_contribution: Option<InlineBoxMetricContribution>,
    },
    Captured {
        scope: CapturedScope,
        wrap_inside: WrapInside,
        edges: WrapEdges,
        metric_contribution: Option<InlineBoxMetricContribution>,
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
    SourceAssociation,
    PlanMismatch,
    MissingBoxMetrics,
    InvalidBoxMetrics,
    ConflictingBoxMetrics,
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
    Baseline(crate::InlineBaselineError),
    Group(crate::group_alignment::NumericError),
    Scope(WrappingScopeError),
    FeedbackCycle,
    InvalidBalanceBasis,
    InvalidTailBasis,
    InvalidPrettyBasis,
    InvalidAtomicReceiverBasis,
}
impl From<crate::group_alignment::NumericError> for NumericWrappingError {
    fn from(error: crate::group_alignment::NumericError) -> Self {
        Self::Group(error)
    }
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
        NumericWrappingError::Baseline(error) => crate::LayoutErrorOf::new(
            crate::LayoutErrorSiteOf::Node(node),
            crate::LayoutOperation::ChildLayout,
            crate::LayoutErrorKindOf::InvalidInput(crate::LayoutInvalidInputOf::InlineBaseline(
                error,
            )),
        ),
        NumericWrappingError::Group(error) => error.layout(node),
        NumericWrappingError::Scope(error) => wrapping_error(node, error),
        NumericWrappingError::FeedbackCycle => crate::LayoutErrorOf::new(
            crate::LayoutErrorSiteOf::Node(node),
            crate::LayoutOperation::InlineWrapping,
            crate::LayoutErrorKindOf::InlineFeedbackCycle,
        ),
        NumericWrappingError::InvalidBalanceBasis => crate::LayoutErrorOf::new(
            crate::LayoutErrorSiteOf::Node(node),
            crate::LayoutOperation::InlineWrapping,
            crate::LayoutErrorKindOf::InternalInvariant(
                crate::LayoutInternalInvariant::InvalidInlineBalance,
            ),
        ),
        NumericWrappingError::InvalidTailBasis => crate::LayoutErrorOf::new(
            crate::LayoutErrorSiteOf::Node(node),
            crate::LayoutOperation::InlineWrapping,
            crate::LayoutErrorKindOf::InternalInvariant(
                crate::LayoutInternalInvariant::InvalidInlineTailSelection,
            ),
        ),
        NumericWrappingError::InvalidPrettyBasis => crate::LayoutErrorOf::new(
            crate::LayoutErrorSiteOf::Node(node),
            crate::LayoutOperation::InlineWrapping,
            crate::LayoutErrorKindOf::InternalInvariant(
                crate::LayoutInternalInvariant::InvalidInlinePrettySelection,
            ),
        ),
        NumericWrappingError::InvalidAtomicReceiverBasis => crate::LayoutErrorOf::new(
            crate::LayoutErrorSiteOf::Node(node),
            crate::LayoutOperation::InlineWrapping,
            crate::LayoutErrorKindOf::InternalInvariant(
                crate::LayoutInternalInvariant::InvalidAtomicReceiverBasis,
            ),
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
        #[cfg(test)]
        work_tests::record_subset();
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
        self.consider_ranked(class, receipt, |_, _| Ordering::Equal)
    }

    /// Rank only equivalent avoidance classes here; strict dominance always
    /// wins independently of numerical preference. Less means left preferred.
    pub(crate) fn consider_ranked(
        &mut self,
        class: AvoidClass,
        receipt: T,
        prefer: impl Fn(&T, &T) -> Ordering,
    ) -> Result<(), WrappingScopeError> {
        #[cfg(test)]
        work_tests::record_offer();
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
                if entries.get(&key).is_some_and(|prior| {
                    prior.class.subset(&class)
                        && class.subset(&prior.class)
                        && prefer(&prior.receipt, &receipt) == Ordering::Less
                }) {
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
                if entries.iter().any(|prior| {
                    prior.class.subset(&class)
                        && class.subset(&prior.class)
                        && prefer(&prior.receipt, &receipt) == Ordering::Less
                }) {
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
        self.into_preferred_ranked(|_, _| Ordering::Equal)
    }

    /// Compare only the surviving nondominated receipts. Exact rank ties keep
    /// the consuming fitter's established latest normal-preference serial.
    pub(crate) fn into_preferred_ranked(self, prefer: impl Fn(&T, &T) -> Ordering) -> Option<T> {
        let preferred = |prior: FrontierEntry<T>, next: FrontierEntry<T>| match prefer(
            &next.receipt,
            &prior.receipt,
        ) {
            Ordering::Less => next,
            Ordering::Equal if next.serial > prior.serial => next,
            _ => prior,
        };
        match self.entries {
            FrontierEntries::Inside(entries) => entries.into_values().reduce(preferred),
            FrontierEntries::Mixed(entries) => entries.into_iter().reduce(preferred),
        }
        .map(|entry| entry.receipt)
    }
}

#[cfg(test)]
mod edge_preference_tests;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod work_tests;
