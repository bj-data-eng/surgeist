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

#[derive(Clone, Debug, PartialEq)]
pub struct FragmentationOpportunityOf<S: LayoutScalar = DefaultScalar> {
    pub position: FragmentationPosition,
    /// Used block coordinate relative to this remaining layout window.
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

#[derive(Clone, Debug, PartialEq)]
pub struct FragmentationWindowOf<S: LayoutScalar = DefaultScalar> {
    /// Remaining flow extent, excluding the truncatable cloned edge budgets.
    pub block_size: S,
    /// Remaining first occurrence extent under the current constraints.
    pub head_block_size: S,
    pub fallback: FragmentationFallback,
    pub opportunities: Vec<FragmentationOpportunityOf<S>>,
    pub cloned_block_start: S,
    pub cloned_block_end: S,
    /// Required only when Emergency must slice a non-fitting head.
    pub emergency_slice: Option<FragmentationEmergencySliceOf<S>>,
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
}

/// Substitutable mode boundary. Queries are against one immutable caller-owned
/// source inventory. Results describe current constraints and remaining source,
/// not native Layout geometry, Text authentication, or committed publication.
/// A missing fragmentainer fails the complete operation; no prefix is returned.
/// Context sequences must eventually cross their matching ancestor boundaries
/// or return None. Repeated requests for a frame retain identical constraints.
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
    ) -> Result<FragmentationWindowOf<S>, Self::Error>;
}
