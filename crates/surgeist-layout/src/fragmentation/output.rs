use super::input::*;
use crate::{DefaultScalar, LayoutScalar};

#[derive(Clone, Debug, PartialEq)]
pub struct FragmentationCoverage<K> {
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
pub enum FragmentationPieceKindOf<S: LayoutScalar = DefaultScalar> {
    Content,
    /// Repeats the same semantic occurrence coverage; only graphical coverage advances.
    GraphicalSlice {
        coverage: FragmentationGraphicalCoverageOf<S>,
    },
    UnsliceableOverflow,
}
#[derive(Clone, Debug, PartialEq)]
pub struct FragmentationPieceOf<K, S: LayoutScalar = DefaultScalar> {
    pub flow: usize,
    pub context: usize,
    pub fragmentainer: usize,
    /// Semantic source cursor before accepting this piece.
    pub start: FragmentationPosition,
    /// Semantic cursor after acceptance. A non-final graphical slice leaves it unchanged.
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
pub enum FragmentationForcedSide {
    Left,
    Right,
    Recto,
    Verso,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FragmentationBreakReason {
    Forced {
        required: FragmentationBreakKinds,
        side: Option<FragmentationForcedSide>,
    },
    Unforced,
    Deferred,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FragmentationBreakRecord {
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
pub struct FragmentationFrameOf<S: LayoutScalar = DefaultScalar> {
    pub context: usize,
    pub index: usize,
    pub frame: FragmentainerOf<S>,
}
/// A complete normalized scheduling result. It does not certify a native box,
/// Text fragment, or CompletedLayoutBatch. Continuation state remains private.
#[derive(Clone, Debug, PartialEq)]
pub struct FragmentationReportOf<K, S: LayoutScalar = DefaultScalar> {
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
#[derive(Clone, Debug, PartialEq)]
pub enum FragmentationError<E> {
    InvalidInput(FragmentationInputError),
    Provider(E),
    MissingFragmentainer { context: usize, index: usize },
    MissingPageSide { context: usize, index: usize },
    MissingSlice { flow: usize },
    NoProgress { flow: usize, occurrence: usize },
    Arithmetic { flow: usize },
    Allocation,
}
