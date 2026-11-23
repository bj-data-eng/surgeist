use crate::fragmentation::input::{
    FragmentainerOf, FragmentationBreakKinds, FragmentationPosition, FragmentationStage,
    FragmentationStart,
};
use crate::scalar::{DefaultScalar, LayoutScalar};

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct FragmentationCoverage<K> {
    pub key: K,
    pub start: usize,
    pub end: usize,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FragmentationGraphicalCoverageOf<S: LayoutScalar = DefaultScalar> {
    pub start: S,
    pub end: S,
    pub extent: S,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum FragmentationPieceKindOf<S: LayoutScalar = DefaultScalar> {
    Content,
    /// Advances graphical coverage within the anchor. Source outside the
    /// anchor is consumed once; anchor coverage may repeat. Final coverage can
    /// also consume mode-certified trailing zero-source closures.
    GraphicalSlice {
        /// The occurrence represented by graphical coverage. Other source
        /// coverage in this piece is consumed normally and is not repeated.
        anchor: FragmentationPosition,
        coverage: FragmentationGraphicalCoverageOf<S>,
    },
    UnsliceableOverflow,
}
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct FragmentationPieceOf<K, S: LayoutScalar = DefaultScalar> {
    pub flow: usize,
    pub context: usize,
    pub fragmentainer: usize,
    /// Semantic source cursor before accepting this piece.
    pub start: FragmentationPosition,
    pub start_provenance: FragmentationStart,
    /// Semantic cursor after acceptance. A non-final graphical cut consumes
    /// preceding source and ends at its anchor.
    pub end: FragmentationPosition,
    /// Source represented by this piece, including full repeated graphical-slice coverage.
    pub coverage: Vec<FragmentationCoverage<K>>,
    pub block_offset: S,
    /// Accepted body extent. Occupied extent additionally includes both retained cloned edges.
    pub block_size: S,
    pub cloned_block_start: S,
    pub cloned_block_end: S,
    pub stage: FragmentationStage,
    pub kind: FragmentationPieceKindOf<S>,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FragmentationForcedSide {
    Left,
    Right,
    Recto,
    Verso,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FragmentationBreakReason {
    Forced {
        required: FragmentationBreakKinds,
        side: Option<FragmentationForcedSide>,
    },
    Unforced,
    Deferred,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct FragmentationBreakRecord {
    pub flow: usize,
    pub context: usize,
    pub from_fragmentainer: usize,
    pub to_fragmentainer: usize,
    pub position: FragmentationPosition,
    pub crossed: FragmentationBreakKinds,
    pub reason: FragmentationBreakReason,
    pub stage: FragmentationStage,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct FragmentationFrameOf<S: LayoutScalar = DefaultScalar> {
    pub context: usize,
    pub index: usize,
    pub frame: FragmentainerOf<S>,
}
/// A complete normalized scheduling result. It does not certify a native box,
/// Text fragment, or CompletedLayoutBatch. Continuation state remains private.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct FragmentationReportOf<K, S: LayoutScalar = DefaultScalar> {
    pub pieces: Vec<FragmentationPieceOf<K, S>>,
    pub breaks: Vec<FragmentationBreakRecord>,
    pub fragmentainers: Vec<FragmentationFrameOf<S>>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FragmentationInputError {
    ContextReference,
    ContextCycle,
    DuplicateOccurrence,
    InvalidPosition,
    InvalidGeometry,
    InvalidLineCounts,
    InvalidContentGap,
    InvalidBoundary,
    InvalidParentFragmentainer,
    InvalidWindow,
    InvalidSlice,
}

impl core::fmt::Display for FragmentationInputError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(match self {
            Self::ContextReference => {
                "a fragmentation context refers to a context that does not exist or does not match"
            }
            Self::ContextCycle => "fragmentation contexts refer to each other in a cycle",
            Self::DuplicateOccurrence => "an occurrence appears more than once",
            Self::InvalidPosition => "a fragmentation position is invalid",
            Self::InvalidGeometry => "a fragmentation geometry value is not finite or is negative",
            Self::InvalidLineCounts => "the orphan or widow count of a window is zero",
            Self::InvalidContentGap => "the content gap of a window is negative or not finite",
            Self::InvalidBoundary => "a fragmentation boundary is invalid",
            Self::InvalidParentFragmentainer => "a parent fragmentainer does not match its child",
            Self::InvalidWindow => "a fragmentation window is invalid",
            Self::InvalidSlice => "a fragmentation slice is invalid",
        })
    }
}

impl std::error::Error for FragmentationInputError {}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum FragmentationError<E> {
    InvalidInput(FragmentationInputError),
    Provider(E),
    MissingFragmentainer { context: usize, index: usize },
    MissingPageSide { context: usize, index: usize },
    MissingSlice { flow: usize },
    NoProgress { flow: usize, occurrence: usize },
    Arithmetic { flow: usize },
    Allocation,
}
