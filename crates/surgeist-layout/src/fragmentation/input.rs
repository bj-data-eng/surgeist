use crate::{DefaultScalar, FlowAxes, LayoutScalar};

/// Types crossed by a fragmentation boundary. Spread always includes Page.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct FragmentationBreakKinds(u8);

impl FragmentationBreakKinds {
    pub const NONE: Self = Self(0);
    pub const PAGE: Self = Self(1);
    pub const COLUMN: Self = Self(2);
    pub const REGION: Self = Self(4);
    pub const SPREAD: Self = Self(9);

    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
    pub const fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }
    pub(crate) const fn is_empty(self) -> bool {
        self.0 == 0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FragmentationKind {
    Page,
    Column,
    Region,
}
impl FragmentationKind {
    pub const fn breaks(self) -> FragmentationBreakKinds {
        match self {
            Self::Page => FragmentationBreakKinds::PAGE,
            Self::Column => FragmentationBreakKinds::COLUMN,
            Self::Region => FragmentationBreakKinds::REGION,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FragmentationPhysicalPageSide {
    Left,
    Right,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FragmentationProgressionPageSide {
    Recto,
    Verso,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FragmentationPageSide {
    pub physical: FragmentationPhysicalPageSide,
    pub progression: FragmentationProgressionPageSide,
}

/// An ordered context has one block-flow direction. Its parent, if any, is a
/// context index in the same input; it is not another live source tree.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FragmentationContext {
    pub kind: FragmentationKind,
    pub parent: Option<usize>,
    pub flow_axes: FlowAxes,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FragmentainerOf<S: LayoutScalar = DefaultScalar> {
    pub inline_size: S,
    pub block_size: S,
    /// Boundary crossed on entry. Noninitial frames cross at least one type.
    pub breaks: FragmentationBreakKinds,
    pub parent_fragmentainer: Option<usize>,
    pub page_side: Option<FragmentationPageSide>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FragmentationOccurrence<K> {
    pub key: K,
    /// Immutable source units, independent of layout width. Zero is valid:
    /// occurrence order still distinguishes and consumes a zero-source box.
    pub source_extent: usize,
    /// Internal opportunities, including internal forced breaks, are ignored.
    pub monolithic: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FragmentationFlowOf<K, S: LayoutScalar = DefaultScalar> {
    pub context: usize,
    pub start_fragmentainer: usize,
    pub initial_block_offset: S,
    pub occurrences: Vec<FragmentationOccurrence<K>>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FragmentationInputOf<K, S: LayoutScalar = DefaultScalar> {
    pub contexts: Vec<FragmentationContext>,
    /// Flows advance independently, even when they share fragmentainers.
    pub flows: Vec<FragmentationFlowOf<K, S>>,
}

/// A position in one input flow. This is source evidence, not a resume token.
/// offset is in occurrence source units; a boundary uses offset zero in the
/// following occurrence. occurrences.len() with offset zero is the flow end.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct FragmentationPosition {
    pub occurrence: usize,
    pub offset: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FragmentationStage {
    Strict,
    RelaxLineCounts,
    RelaxAvoidance,
    TruncateBlockEnd,
    TruncateBlockStart,
    Emergency,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FragmentationBreak {
    Auto,
    Avoid,
    AvoidPage,
    AvoidColumn,
    AvoidRegion,
    Page,
    Column,
    Region,
    Left,
    Right,
    Recto,
    Verso,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FragmentationBreakInside {
    Auto,
    Avoid,
    AvoidPage,
    AvoidColumn,
    AvoidRegion,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FragmentationRole {
    Normal,
    Float,
    Absolute,
}

/// Applicable before/after control after mode-owned boundary propagation.
/// source_order is the actual flow order, never a geometric or key ordering.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FragmentationBreakControl {
    pub value: FragmentationBreak,
    pub role: FragmentationRole,
    pub source_order: usize,
}

/// A mode-established boundary at which before/after controls can force a break.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FragmentationControlKind {
    Sibling,
    ModeSpecific,
}

/// Complete controls at the query's current source cursor, without measuring content.
#[derive(Clone, Debug, PartialEq)]
pub struct FragmentationBoundaryControls {
    pub kind: FragmentationControlKind,
    pub before: Vec<FragmentationBreakControl>,
    pub after: Vec<FragmentationBreakControl>,
}

/// Actual scheduling transition preceding the queried or accepted content.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FragmentationStart {
    /// Entry to this source flow, including a nonzero initial frame offset.
    FlowStart,
    /// Source or graphical content has already been accepted in this frame.
    /// This includes accepted zero-sized source occurrences.
    InFrame,
    /// The frame was actually entered by the recorded scheduler transition.
    AfterBreak(super::FragmentationBreakReason),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FragmentationOpportunityKindOf<S: LayoutScalar = DefaultScalar> {
    Sibling,
    Lines {
        before: usize,
        after: usize,
        orphans: usize,
        widows: usize,
    },
    ContentEdge {
        gap: S,
    },
    /// A legal opportunity supplied by another formatting model.
    ModeSpecific,
}

/// A mode-qualified break cut. Geometric coverage advances independently of
/// immutable source units; its anchor identifies the represented occurrence.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FragmentationCutOf<S: LayoutScalar = DefaultScalar> {
    Source(FragmentationPosition),
    /// A strict interior cut; completion uses the following source boundary.
    Graphical {
        anchor: FragmentationPosition,
        progress: FragmentationGraphicalProgressOf<S>,
    },
}
impl<S: LayoutScalar> FragmentationCutOf<S> {
    #[must_use]
    pub const fn source_position(self) -> FragmentationPosition {
        match self {
            Self::Source(position) => position,
            Self::Graphical { anchor, .. } => anchor,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct FragmentationOpportunityOf<S: LayoutScalar = DefaultScalar> {
    pub cut: FragmentationCutOf<S>,
    /// Used block coordinate relative to the query's original source cursor.
    pub block_offset: S,
    pub kind: FragmentationOpportunityKindOf<S>,
    pub before: Vec<FragmentationBreakControl>,
    pub after: Vec<FragmentationBreakControl>,
    pub ancestors: Vec<FragmentationBreakInside>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FragmentationFallback {
    Sliceable,
    Unsliceable,
}

/// Graphical coverage is independent of semantic source units and pixels.
/// extent stays fixed for the current occurrence across layout width changes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FragmentationGraphicalProgressOf<S: LayoutScalar = DefaultScalar> {
    pub consumed: S,
    pub extent: S,
}

/// Explicit mode-owned progress. Graphical progress keeps the same semantic
/// occurrence coverage until its final slice; it never manufactures a Text cut.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FragmentationEmergencySliceOf<S: LayoutScalar = DefaultScalar> {
    Source {
        end: FragmentationPosition,
        block_size: S,
    },
    Graphical {
        end: S,
        extent: S,
        block_size: S,
    },
}

/// The actual source endpoint covered by one complete from-origin measurement.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FragmentationWindowEnd {
    FlowEnd,
    /// The reply includes all controls at this terminal potential boundary.
    PotentialControl(FragmentationPosition),
}

/// Measured first-occurrence content, capped by the actual measured window end.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FragmentationMeasuredHeadOf<S: LayoutScalar = DefaultScalar> {
    /// The lesser of the next occurrence boundary and the measured window end.
    pub end: FragmentationPosition,
    pub block_size: S,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FragmentationWindowOf<S: LayoutScalar = DefaultScalar> {
    pub end: FragmentationWindowEnd,
    /// Extent of exactly the covered source span, excluding cloned edge budgets.
    pub block_size: S,
    pub head: FragmentationMeasuredHeadOf<S>,
    pub fallback: FragmentationFallback,
    pub opportunities: Vec<FragmentationOpportunityOf<S>>,
    pub cloned_block_start: S,
    pub cloned_block_end: S,
    /// Required only when Emergency must slice a non-fitting head.
    pub emergency_slice: Option<FragmentationEmergencySliceOf<S>>,
}

/// A potential control can be disclosed before measuring semantically unreachable
/// successor content. Replacements measure the whole span from the same origin;
/// discovery alone neither consumes source nor commits a measured prefix.
#[derive(Clone, Debug, PartialEq)]
pub enum FragmentationReplyOf<S: LayoutScalar = DefaultScalar> {
    ControlAtStart(FragmentationBoundaryControls),
    Measured(FragmentationWindowOf<S>),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FragmentationQueryOf<S: LayoutScalar = DefaultScalar> {
    pub flow: usize,
    pub context: usize,
    pub fragmentainer_index: usize,
    pub fragmentainer: FragmentainerOf<S>,
    pub position: FragmentationPosition,
    pub graphical_progress: Option<FragmentationGraphicalProgressOf<S>>,
    pub used_block_offset: S,
    pub available_block_size: S,
    pub stage: FragmentationStage,
    /// Discovery must extend strictly beyond this cursor. This does not change
    /// the formatting origin, available space, or start provenance.
    pub scan_after: Option<FragmentationPosition>,
    pub start_provenance: FragmentationStart,
}

/// Substitutable mode boundary. Queries are against one immutable caller-owned
/// source inventory. Results describe current constraints and measured source,
/// not native Layout geometry, Text authentication, or committed publication.
/// A missing fragmentainer fails the complete operation; no prefix is returned.
/// Context sequences must eventually cross their matching ancestor boundaries
/// or return None. Repeated requests for a frame retain identical constraints.
/// A partial reply establishes a real terminal control boundary under those
/// constraints. Every replacement includes all still-legal opportunities from
/// the original query cursor; previously measured geometry is replaced, not added.
pub trait FragmentationProvider<S: LayoutScalar> {
    type Error;
    fn fragmentainer(
        &mut self,
        context: usize,
        index: usize,
    ) -> Result<Option<FragmentainerOf<S>>, Self::Error>;
    fn layout(
        &mut self,
        query: FragmentationQueryOf<S>,
    ) -> Result<FragmentationReplyOf<S>, Self::Error>;
}
