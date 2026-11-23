//! Layout-owned inline scope proof and ancestry-prefix break preference.

use crate::error::{
    LayoutErrorKindOf, LayoutErrorOf, LayoutErrorSiteOf, LayoutInternalInvariant, LayoutOperation,
};
use std::{
    cmp::Ordering,
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};
pub(crate) mod balance;
pub(crate) mod body;
mod declarations;
pub(crate) mod quality;
pub(crate) mod tail;
pub use declarations::{
    ChildOrderCompleteness, GeneratedBoxParticipation, GeneratedBoxRef, GeneratedChildBoxInput,
    InlineBoxMetricContribution, InlineBoxRef, InlineChildOrderInput, InlineDelimiterSide,
    InlineFormatterEdge, InlineGapAffinity, InlineGapAnchor, InlineGapGroupInput,
    NativeInlineBoxFamilyInput, WrapEdge, WrapEdges, WrappingInput,
};
pub(crate) use declarations::{EdgeEffect, EdgePreferences, WrappingMode};

use crate::accounting::{LayoutWork, Meter};
use surgeist_text::capture::CapturedScope;
mod canonical_order;
mod context;
pub(crate) use canonical_order::{
    CanonicalBox, CanonicalCoverage, CanonicalOrderBindings, CanonicalOrderInventory,
};
mod edge_plan;
pub(crate) mod family;
pub use family::{
    NativeInlineBoxEdgeView, NativeInlineBoxFamily, NativeInlineBoxFragment,
    NativeInlineBoxMemberView, NativeInlineBoxPieceView,
};
pub(crate) mod ordering;
pub(crate) use ordering::{NativeOrderingProjection, NativeOrderingSelection, NativeRunSource};
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
    Allocation,
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
    MissingGap,
    AmbiguousGap,
    InvalidGapAnchor,
    ConflictingGap,
    CyclicGap,
    GapOutsideSelection,
    /// The same formatter was requested more than once.
    DuplicateFamily,
    /// Supplied proof does not establish the requested complete generated closure.
    IncompleteFamily,
    /// A requested normal family's Contribution endpoint has nonzero flow advance.
    IncompatibleContribution,
    /// The fragment identities belong to different admitted families.
    DifferentFamily,
    /// The fragments belong to distinct bodies of the same family.
    DifferentBody,
}

impl core::fmt::Display for WrappingScopeError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(match self {
            Self::Allocation => "wrapping could not allocate storage or stay within its text work limits",
            Self::NodeNotReachable => "a node that the wrapping data names is not reachable in the flow's tree",
            Self::AmbiguousNode => "a node that the wrapping data names is reachable by more than one path",
            Self::EndpointRole => "a scope endpoint is not a node that can bound an inline scope",
            Self::DifferentFlow => "a scope, edge or node is declared for a flow that does not contain it or that cannot be an inline flow",
            Self::EndpointOrder => "scope endpoints are missing or not in source order among the flow's children",
            Self::DuplicateScope => "the same scope is declared more than once",
            Self::ConflictingScope => "two declarations of the same scope conflict",
            Self::CrossingScopes => "two scopes cross without nesting",
            Self::MissingCapture => "a text capture or wrapping context that a scope requires is missing",
            Self::ForeignCapture => "a scope names a text capture that belongs to another flow",
            Self::NonInlineScope => "a scope that is not an inline scope was supplied where an inline scope is required",
            Self::CaptureAssociation => "the association between a scope and its text capture is inconsistent",
            Self::UnsupportedIncidence => "wrapping does not support this flow, scope or edge, or its incidence cannot be established",
            Self::InconsistentIncidence => "the wrapping incidence, completion or ordering data is inconsistent or exceeds a representable range",
            Self::ChildOrder => "the child order of a box is inconsistent with the tree",
            Self::MissingChildOrder => "the child order that a box needs is missing or incomplete",
            Self::SourceProcessingRequired => "the source must be processed before this wrapping input can be admitted",
            Self::SourceAssociation => "a source association is inconsistent with the processed source",
            Self::PlanMismatch => "the wrapping input, receipt or endpoint ranks do not match the processed wrapping plan",
            Self::MissingBoxMetrics => "a box has no metric contribution",
            Self::InvalidBoxMetrics => "a box's metric contribution is invalid",
            Self::ConflictingBoxMetrics => "a box's metric contributions conflict",
            Self::MissingGap => "the declarations do not determine the gap that they require",
            Self::AmbiguousGap => "a gap is not determined uniquely",
            Self::InvalidGapAnchor => "a gap anchor, or a box that a gap names, is invalid",
            Self::ConflictingGap => "gap declarations, or the gaps derived from them, contradict each other",
            Self::CyclicGap => "gap anchors form a cycle",
            Self::GapOutsideSelection => "a gap lies outside the selected units",
            Self::DuplicateFamily => "the same formatter was requested more than once",
            Self::IncompleteFamily => "the supplied proof does not establish the requested complete generated closure",
            Self::IncompatibleContribution => "a requested normal family's contribution endpoint has a nonzero flow advance",
            Self::DifferentFamily => "the fragment identities belong to different admitted families",
            Self::DifferentBody => "the fragments belong to distinct bodies of the same family",
        })
    }
}

impl std::error::Error for WrappingScopeError {}

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
    pub(crate) fn frontier<T>(&self, meter: &Meter) -> PreferenceFrontier<T> {
        PreferenceFrontier {
            ancestry: Arc::clone(&self.ancestry),
            entries: FrontierEntries::Inside(BTreeMap::new()),
            serial: 0,
            meter: meter.clone(),
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

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum NumericWrappingError {
    Geometry {
        value: f64,
        reason: crate::error::LayoutTextNumericError,
    },
    Allocation,
    Baseline(crate::node_input::InlineBaselineError),
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
pub(crate) fn numeric_wrapping_error<Node, S: crate::scalar::LayoutScalar, M>(
    node: Node,
    error: NumericWrappingError,
) -> LayoutErrorOf<Node, S, M> {
    match error {
        NumericWrappingError::Geometry { value, reason } => LayoutErrorOf::new(
            LayoutErrorSiteOf::Node(node),
            LayoutOperation::ChildLayout,
            LayoutErrorKindOf::TextNumeric { value, reason },
        ),
        NumericWrappingError::Allocation => LayoutErrorOf::new(
            LayoutErrorSiteOf::Node(node),
            LayoutOperation::InlineWrapping,
            LayoutErrorKindOf::Allocation,
        ),
        NumericWrappingError::Baseline(error) => LayoutErrorOf::new(
            LayoutErrorSiteOf::Node(node),
            LayoutOperation::ChildLayout,
            LayoutErrorKindOf::InvalidInput(crate::error::LayoutInvalidInputOf::InlineBaseline(
                error,
            )),
        ),
        NumericWrappingError::Group(error) => error.layout(node),
        NumericWrappingError::Scope(error) => wrapping_error(node, error),
        NumericWrappingError::FeedbackCycle => LayoutErrorOf::new(
            LayoutErrorSiteOf::Node(node),
            LayoutOperation::InlineWrapping,
            LayoutErrorKindOf::InlineFeedbackCycle,
        ),
        NumericWrappingError::InvalidBalanceBasis => LayoutErrorOf::new(
            LayoutErrorSiteOf::Node(node),
            LayoutOperation::InlineWrapping,
            LayoutErrorKindOf::InternalInvariant(LayoutInternalInvariant::InvalidInlineBalance),
        ),
        NumericWrappingError::InvalidTailBasis => LayoutErrorOf::new(
            LayoutErrorSiteOf::Node(node),
            LayoutOperation::InlineWrapping,
            LayoutErrorKindOf::InternalInvariant(
                LayoutInternalInvariant::InvalidInlineTailSelection,
            ),
        ),
        NumericWrappingError::InvalidPrettyBasis => LayoutErrorOf::new(
            LayoutErrorSiteOf::Node(node),
            LayoutOperation::InlineWrapping,
            LayoutErrorKindOf::InternalInvariant(
                LayoutInternalInvariant::InvalidInlinePrettySelection,
            ),
        ),
        NumericWrappingError::InvalidAtomicReceiverBasis => LayoutErrorOf::new(
            LayoutErrorSiteOf::Node(node),
            LayoutOperation::InlineWrapping,
            LayoutErrorKindOf::InternalInvariant(
                LayoutInternalInvariant::InvalidAtomicReceiverBasis,
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
    meter: Meter,
}
struct FrontierEntry<T> {
    class: AvoidClass,
    serial: usize,
    receipt: T,
}
enum FrontierEntries<T> {
    Inside(BTreeMap<usize, FrontierEntry<T>>),
    Mixed(MixedEntries<T>),
}
struct MixedEntries<T> {
    entries: Vec<FrontierEntry<T>>,
    // Accepted historical IDs conservatively cover every retained edge. Old
    // removed IDs may miss an optimization, but cannot alter a preference.
    seen_edges: BTreeSet<usize>,
    has_empty_edges: bool,
}
impl<T> MixedEntries<T> {
    fn from_inside(entries: BTreeMap<usize, FrontierEntry<T>>) -> Self {
        Self {
            has_empty_edges: !entries.is_empty(),
            entries: entries.into_values().collect(),
            seen_edges: BTreeSet::new(),
        }
    }
}
impl AvoidClass {
    fn interval(&self) -> (usize, usize) {
        self.scope
            .map_or((0, usize::MAX), |i| self.ancestry.intervals[i])
    }
    fn subset(&self, other: &Self, meter: &Meter) -> bool {
        meter.charge(LayoutWork::WrappingSubsetTest, 1);
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
        self.meter.charge(LayoutWork::WrappingOffer, 1);
        if !Arc::ptr_eq(&self.ancestry, &class.ancestry) {
            return Err(WrappingScopeError::InconsistentIncidence);
        }
        if !class.edges.is_empty() && matches!(self.entries, FrontierEntries::Inside(_)) {
            // Empty-edge ancestors that already dominated a receipt continue
            // to dominate it when edge constraints arrive. Transfer only the
            // retained frontier, keeping its true normal-preference serials.
            let prior =
                std::mem::replace(&mut self.entries, FrontierEntries::Inside(BTreeMap::new()));
            self.entries = match prior {
                FrontierEntries::Inside(entries) => {
                    FrontierEntries::Mixed(MixedEntries::from_inside(entries))
                }
                mixed => mixed,
            };
        }
        let meter = &self.meter;
        match &mut self.entries {
            FrontierEntries::Inside(entries) => {
                let (key, end) = class.interval();
                if let Some((_, prior)) = entries.range(..key).next_back()
                    && key < prior.class.interval().1
                {
                    return Ok(());
                }
                if entries.get(&key).is_some_and(|prior| {
                    prior.class.subset(&class, meter)
                        && class.subset(&prior.class, meter)
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
            FrontierEntries::Mixed(mixed) => {
                // Two disjoint nonempty edge sets cannot include each other,
                // irrespective of their inside ancestry. Shared/repeated Text
                // incidences still take the complete comparison path below.
                let disjoint = !class.edges.is_empty()
                    && !mixed.has_empty_edges
                    && class
                        .edges
                        .iter()
                        .all(|edge| !mixed.seen_edges.contains(edge));
                if !disjoint {
                    if mixed.entries.iter().any(|prior| {
                        prior.class.subset(&class, meter) && !class.subset(&prior.class, meter)
                    }) {
                        return Ok(());
                    }
                    if mixed.entries.iter().any(|prior| {
                        prior.class.subset(&class, meter)
                            && class.subset(&prior.class, meter)
                            && prefer(&prior.receipt, &receipt) == Ordering::Less
                    }) {
                        return Ok(());
                    }
                    mixed
                        .entries
                        .retain(|prior| !class.subset(&prior.class, meter));
                }
                mixed.seen_edges.extend(class.edges.iter().copied());
                // A nonempty class cannot remove an empty-edge survivor;
                // replacing it with another empty class preserves this guard.
                mixed.has_empty_edges |= class.edges.is_empty();
                mixed.entries.push(FrontierEntry {
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
            FrontierEntries::Mixed(mixed) => mixed.entries.into_iter().reduce(preferred),
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
