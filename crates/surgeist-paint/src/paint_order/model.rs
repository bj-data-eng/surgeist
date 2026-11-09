//! Raw resolved paint requests and borrowed, validated order observations.
//!
//! Request constructors preserve facts without validating cross-table relations.
//! Slot IDs are local to these exact borrowed tables: an in-range ID copied from
//! another request cannot be distinguished from a local ID. Native adapters retain
//! their authentic geometry, formatter/run/line and appearance associations by
//! these slots; these records do not authenticate computed Style or infer geometry.

use std::fmt;

use crate::{BoxFacts, BoxId, StackingPlan};
use surgeist_layout::SourceIndex;

macro_rules! slot_id {
    ($name:ident, $description:literal) => {
        #[doc = $description]
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        pub struct $name(usize);
        impl $name {
            pub const fn new(index: usize) -> Self {
                Self(index)
            }
            pub const fn get(self) -> usize {
                self.0
            }
        }
    };
}

slot_id!(
    TextId,
    "An operation-local slot in the borrowed text-source table."
);
slot_id!(
    FragmentId,
    "An operation-local slot in the borrowed fragment table."
);
slot_id!(
    LineId,
    "An operation-local slot in the borrowed line table."
);
slot_id!(
    PrimitiveId,
    "An operation-local slot in the borrowed primitive table."
);

/// Explicit resolved box category, independent of geometry. CSS blockification
/// is already applied: floating, out-of-flow and item boxes use block categories.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BoxPaintKind {
    Block,
    Inline,
    InlineBlock,
    BlockReplaced,
    InlineReplaced,
    Table,
    InlineTable,
}

/// Resolved float participation; float side does not affect this paint order.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FloatParticipation {
    None,
    Floating,
}

/// Paint category for the box at the same slot in the BoxFacts table.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BoxPaint {
    kind: BoxPaintKind,
    float: FloatParticipation,
}
impl BoxPaint {
    pub const fn new(kind: BoxPaintKind, float: FloatParticipation) -> Self {
        Self { kind, float }
    }
    pub const fn kind(self) -> BoxPaintKind {
        self.kind
    }
    pub const fn float(self) -> FloatParticipation {
        self.float
    }
}

/// One logical text source, sharing its parent's source-order domain with boxes.
/// Several actual line/bidi occurrences can refer to the same source.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TextSource {
    parent: BoxId,
    source_index: SourceIndex,
}
impl TextSource {
    pub const fn new(parent: BoxId, source_index: SourceIndex) -> Self {
        Self {
            parent,
            source_index,
        }
    }
    pub const fn parent(self) -> BoxId {
        self.parent
    }
    pub const fn source_index(self) -> SourceIndex {
        self.source_index
    }
}

/// A distinct line association and its resolved logical order in one container
/// occurrence. Native local line indices may repeat across distinct run bindings;
/// the caller retains the exact association for each separate LineId slot.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PaintLine {
    container: FragmentId,
    logical_order: usize,
}
impl PaintLine {
    pub const fn new(container: FragmentId, logical_order: usize) -> Self {
        Self {
            container,
            logical_order,
        }
    }
    pub const fn container(self) -> FragmentId {
        self.container
    }
    pub const fn logical_order(self) -> usize {
        self.logical_order
    }
}

/// Semantic owner of an occurrence. A fragment is not another semantic box.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FragmentOwner {
    Box(BoxId),
    Text(TextId),
}

/// Explicit promoted block placement among a container's line/content entries.
/// Content order shares the container-local domain of PaintLine::logical_order;
/// it is independent of the fragment owner's logical occurrence order.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BlockInInlinePlacement {
    container: FragmentId,
    content_order: usize,
}
impl BlockInInlinePlacement {
    pub const fn new(container: FragmentId, content_order: usize) -> Self {
        Self {
            container,
            content_order,
        }
    }
    pub const fn container(self) -> FragmentId {
        self.container
    }
    pub const fn content_order(self) -> usize {
        self.content_order
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum FragmentData {
    Box {
        owner: BoxId,
        parent: Option<FragmentId>,
        line: Option<LineId>,
    },
    Text {
        owner: TextId,
        parent: FragmentId,
        line: LineId,
    },
    BlockInInline {
        owner: BoxId,
        placement: BlockInInlinePlacement,
    },
}

/// An exact caller-associated occurrence, including occurrences without geometry.
/// Direct parentage is logical occurrence containment; explicit block-in-inline
/// promotion names the actual containing fragment while preserving semantic box
/// ancestry separately. Neither relationship defines coordinate or clip ancestry.
/// Logical order is unique within its semantic owner, not a visual ordinal.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PaintFragment {
    data: FragmentData,
    logical_order: usize,
}
impl PaintFragment {
    pub(super) const fn data(self) -> FragmentData {
        self.data
    }

    pub const fn box_fragment(
        owner: BoxId,
        parent: Option<FragmentId>,
        line: Option<LineId>,
        logical_order: usize,
    ) -> Self {
        Self {
            data: FragmentData::Box {
                owner,
                parent,
                line,
            },
            logical_order,
        }
    }
    /// Text occurrences intrinsically require an occurrence parent and line.
    pub const fn text_fragment(
        owner: TextId,
        parent: FragmentId,
        line: LineId,
        logical_order: usize,
    ) -> Self {
        Self {
            data: FragmentData::Text {
                owner,
                parent,
                line,
            },
            logical_order,
        }
    }
    /// Raw explicit promotion; the operation validates the skipped semantic
    /// ancestry and the container's shared line/content ordering domain.
    pub const fn block_in_inline(
        owner: BoxId,
        container: FragmentId,
        content_order: usize,
        logical_order: usize,
    ) -> Self {
        Self {
            data: FragmentData::BlockInInline {
                owner,
                placement: BlockInInlinePlacement::new(container, content_order),
            },
            logical_order,
        }
    }
    pub const fn owner(self) -> FragmentOwner {
        match self.data {
            FragmentData::Box { owner, .. } | FragmentData::BlockInInline { owner, .. } => {
                FragmentOwner::Box(owner)
            }
            FragmentData::Text { owner, .. } => FragmentOwner::Text(owner),
        }
    }
    pub const fn parent(self) -> Option<FragmentId> {
        match self.data {
            FragmentData::Box { parent, .. } => parent,
            FragmentData::Text { parent, .. } => Some(parent),
            FragmentData::BlockInInline { placement, .. } => Some(placement.container()),
        }
    }
    pub const fn line(self) -> Option<LineId> {
        match self.data {
            FragmentData::Box { line, .. } => line,
            FragmentData::Text { line, .. } => Some(line),
            FragmentData::BlockInInline { .. } => None,
        }
    }
    pub const fn block_in_inline_placement(self) -> Option<BlockInInlinePlacement> {
        match self.data {
            FragmentData::BlockInInline { placement, .. } => Some(placement),
            FragmentData::Box { .. } | FragmentData::Text { .. } => None,
        }
    }
    pub const fn logical_order(self) -> usize {
        self.logical_order
    }
}

/// Already-resolved root background appearance, painted over the canvas once.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CanvasPaintKind {
    BackgroundColor,
    BackgroundImages,
}

/// Appearance role within one fragment. Images and borders can have several
/// concrete records in the producer's already-resolved appearance group.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FragmentPaintKind {
    BackgroundColor,
    BackgroundImages,
    Border,
    ReplacedContent,
    Text,
    Outline,
}

/// Applicable text decoration; upstream supplies applicability, Paint orders it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DecorationKind {
    Underline,
    Overline,
    LineThrough,
}

/// Closed semantic primitive categories; canvas text cannot be constructed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PrimitiveKind {
    Canvas(CanvasPaintKind),
    Fragment(FragmentPaintKind),
    Decoration(DecorationKind),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum PrimitiveData {
    Canvas(CanvasPaintKind),
    Fragment {
        fragment: FragmentId,
        kind: FragmentPaintKind,
    },
    Decoration {
        text_fragment: FragmentId,
        applying_box: BoxId,
        kind: DecorationKind,
    },
}

/// A borrowed-table record whose slot retains the caller's exact appearance and
/// geometry association. Record order is local to its target/role/applicator
/// group; no CSS phase or geometry-based sorting key is supplied by the caller.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PaintPrimitive {
    data: PrimitiveData,
    record_order: usize,
}
impl PaintPrimitive {
    pub(super) const fn data(self) -> PrimitiveData {
        self.data
    }

    pub const fn canvas(kind: CanvasPaintKind, record_order: usize) -> Self {
        Self {
            data: PrimitiveData::Canvas(kind),
            record_order,
        }
    }
    pub const fn fragment(
        fragment: FragmentId,
        kind: FragmentPaintKind,
        record_order: usize,
    ) -> Self {
        Self {
            data: PrimitiveData::Fragment { fragment, kind },
            record_order,
        }
    }
    pub const fn decoration(
        text_fragment: FragmentId,
        applying_box: BoxId,
        kind: DecorationKind,
        record_order: usize,
    ) -> Self {
        Self {
            data: PrimitiveData::Decoration {
                text_fragment,
                applying_box,
                kind,
            },
            record_order,
        }
    }
    pub const fn kind(self) -> PrimitiveKind {
        match self.data {
            PrimitiveData::Canvas(kind) => PrimitiveKind::Canvas(kind),
            PrimitiveData::Fragment { kind, .. } => PrimitiveKind::Fragment(kind),
            PrimitiveData::Decoration { kind, .. } => PrimitiveKind::Decoration(kind),
        }
    }
    pub const fn fragment_id(self) -> Option<FragmentId> {
        match self.data {
            PrimitiveData::Canvas(_) => None,
            PrimitiveData::Fragment { fragment, .. } => Some(fragment),
            PrimitiveData::Decoration { text_fragment, .. } => Some(text_fragment),
        }
    }
    pub const fn applying_box(self) -> Option<BoxId> {
        match self.data {
            PrimitiveData::Decoration { applying_box, .. } => Some(applying_box),
            PrimitiveData::Canvas(_) | PrimitiveData::Fragment { .. } => None,
        }
    }
    pub const fn record_order(self) -> usize {
        self.record_order
    }
}

/// Raw borrowed resolved facts. Construction does not validate cross-record
/// relations or authenticate Style. Geometry and native source associations stay
/// in caller-owned immutable maps keyed by the corresponding operation-local IDs.
#[derive(Clone, Copy, Debug)]
pub struct PaintRequest<'a> {
    boxes: &'a [BoxFacts],
    box_paint: &'a [BoxPaint],
    texts: &'a [TextSource],
    lines: &'a [PaintLine],
    fragments: &'a [PaintFragment],
    primitives: &'a [PaintPrimitive],
    root: BoxId,
}
impl<'a> PaintRequest<'a> {
    pub const fn new(
        boxes: &'a [BoxFacts],
        box_paint: &'a [BoxPaint],
        texts: &'a [TextSource],
        lines: &'a [PaintLine],
        fragments: &'a [PaintFragment],
        primitives: &'a [PaintPrimitive],
        root: BoxId,
    ) -> Self {
        Self {
            boxes,
            box_paint,
            texts,
            lines,
            fragments,
            primitives,
            root,
        }
    }
    pub const fn boxes(self) -> &'a [BoxFacts] {
        self.boxes
    }
    pub const fn box_paint(self) -> &'a [BoxPaint] {
        self.box_paint
    }
    pub const fn texts(self) -> &'a [TextSource] {
        self.texts
    }
    pub const fn lines(self) -> &'a [PaintLine] {
        self.lines
    }
    pub const fn fragments(self) -> &'a [PaintFragment] {
        self.fragments
    }
    pub const fn primitives(self) -> &'a [PaintPrimitive] {
        self.primitives
    }
    pub const fn root(self) -> BoxId {
        self.root
    }
}

/// A member of the mixed immediate-child source-order domain.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SourceChild {
    Box(BoxId),
    Text(TextId),
}

/// One entry in the shared container-local line/promoted-block order domain.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContentEntry {
    Line(LineId),
    PromotedBlock(FragmentId),
}

/// The resolved category invariant rejected for a particular box.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BoxCategoryError {
    RootMustBeBlock,
    FloatingRoot,
    FloatingInline,
    OutOfFlowInline,
    InlineItem,
    FloatingItem,
    ReplacedHasChildren,
}

/// Typed cross-record refusal. No variant wraps a planning/accounting failure.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PaintInputError {
    BoxPaintCount {
        boxes: usize,
        box_paint: usize,
    },
    UnsupportedBoxKind {
        box_id: BoxId,
        kind: BoxPaintKind,
    },
    InvalidBoxCategory {
        box_id: BoxId,
        reason: BoxCategoryError,
    },
    MissingTextParent {
        text: TextId,
        parent: BoxId,
    },
    InvalidTextParent {
        text: TextId,
        parent: BoxId,
    },
    DuplicateMixedSourceIndex {
        parent: BoxId,
        source_index: SourceIndex,
        first: SourceChild,
        other: SourceChild,
    },
    MissingFragmentOwner {
        fragment: FragmentId,
        owner: FragmentOwner,
    },
    MissingFragmentParent {
        fragment: FragmentId,
        parent: FragmentId,
    },
    InvalidFragmentParent {
        fragment: FragmentId,
        parent: Option<FragmentId>,
    },
    InvalidBlockInInlinePlacement {
        fragment: FragmentId,
        container: FragmentId,
    },
    /// The raw input does not identify a skippable anonymous formatting wrapper.
    /// This is a native-input capability boundary, not an invalid-CSS verdict.
    UnsupportedFragmentTopology {
        fragment: FragmentId,
        container: FragmentId,
        through: BoxId,
    },
    DuplicateFragmentOrder {
        owner: FragmentOwner,
        logical_order: usize,
        first: FragmentId,
        other: FragmentId,
    },
    InconsistentFragmentOrder {
        first: FragmentId,
        other: FragmentId,
    },
    MissingLineContainer {
        line: LineId,
        container: FragmentId,
    },
    InvalidLineContainer {
        line: LineId,
        container: FragmentId,
    },
    DuplicateLineOrder {
        container: FragmentId,
        logical_order: usize,
        first: LineId,
        other: LineId,
    },
    DuplicateContentOrder {
        container: FragmentId,
        logical_order: usize,
        first: ContentEntry,
        other: ContentEntry,
    },
    MissingFragmentLine {
        fragment: FragmentId,
        line: LineId,
    },
    MissingLineIncidence {
        fragment: FragmentId,
    },
    UnexpectedLineIncidence {
        fragment: FragmentId,
        line: LineId,
    },
    InvalidLineIncidence {
        fragment: FragmentId,
        line: LineId,
    },
    MissingPrimitiveFragment {
        primitive: PrimitiveId,
        fragment: FragmentId,
    },
    InvalidPrimitiveTarget {
        primitive: PrimitiveId,
        fragment: FragmentId,
    },
    RootFragmentBackground {
        primitive: PrimitiveId,
        fragment: FragmentId,
    },
    MissingDecorationApplicator {
        primitive: PrimitiveId,
        applying_box: BoxId,
    },
    InvalidDecorationApplicator {
        primitive: PrimitiveId,
        text_fragment: FragmentId,
        applying_box: BoxId,
    },
    DuplicatePrimitiveOrder {
        first: PrimitiveId,
        other: PrimitiveId,
        record_order: usize,
    },
}
impl fmt::Display for PaintInputError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid resolved paint input: {self:?}")
    }
}
impl std::error::Error for PaintInputError {}

#[derive(Debug)]
pub(crate) enum OrderEvent {
    BeginContext(BoxId),
    Draw(PrimitiveId),
    EndContext(BoxId),
}

/// Complete validated ordering borrowing its exact request. Actual contexts are
/// nested atomic intervals; these events are not a coordinate/clip tree or a
/// backend command stream. The result deliberately does not implement Clone.
#[derive(Debug)]
pub struct PaintOrder<'a> {
    pub(crate) request: PaintRequest<'a>,
    pub(crate) stacking: StackingPlan<'a>,
    pub(crate) events: Vec<OrderEvent>,
}
impl PaintOrder<'_> {
    pub const fn request(&self) -> PaintRequest<'_> {
        self.request
    }
    pub const fn stacking(&self) -> &StackingPlan<'_> {
        &self.stacking
    }
    pub fn events(&self) -> impl ExactSizeIterator<Item = PaintOrderEvent<'_>> {
        self.events.iter().map(|event| match *event {
            OrderEvent::BeginContext(id) => {
                PaintOrderEvent::BeginContext(BoxView { plan: self, id })
            }
            OrderEvent::Draw(id) => PaintOrderEvent::Draw(PrimitiveView { plan: self, id }),
            OrderEvent::EndContext(id) => PaintOrderEvent::EndContext(BoxView { plan: self, id }),
        })
    }
    pub fn box_view(&self, id: BoxId) -> Option<BoxView<'_>> {
        self.request
            .boxes
            .get(id.get())
            .map(|_| BoxView { plan: self, id })
    }
    pub fn fragment_view(&self, id: FragmentId) -> Option<FragmentView<'_>> {
        self.request
            .fragments
            .get(id.get())
            .map(|_| FragmentView { plan: self, id })
    }
    pub fn line_view(&self, id: LineId) -> Option<LineView<'_>> {
        self.request
            .lines
            .get(id.get())
            .map(|_| LineView { plan: self, id })
    }
    pub fn primitive_view(&self, id: PrimitiveId) -> Option<PrimitiveView<'_>> {
        self.request
            .primitives
            .get(id.get())
            .map(|_| PrimitiveView { plan: self, id })
    }
}

/// Borrowed observations, bound to the validated plan that issued their views.
#[derive(Clone, Copy, Debug)]
pub enum PaintOrderEvent<'plan> {
    BeginContext(BoxView<'plan>),
    Draw(PrimitiveView<'plan>),
    EndContext(BoxView<'plan>),
}

#[derive(Clone, Copy, Debug)]
pub struct BoxView<'plan> {
    plan: &'plan PaintOrder<'plan>,
    id: BoxId,
}
impl<'plan> BoxView<'plan> {
    pub const fn id(self) -> BoxId {
        self.id
    }
    pub fn facts(self) -> &'plan BoxFacts {
        &self.plan.request.boxes[self.id.get()]
    }
    pub fn paint(self) -> &'plan BoxPaint {
        &self.plan.request.box_paint[self.id.get()]
    }
    pub fn parent(self) -> Option<Self> {
        self.facts().parent().map(|id| Self {
            plan: self.plan,
            id,
        })
    }
}

#[derive(Clone, Copy, Debug)]
pub struct FragmentView<'plan> {
    plan: &'plan PaintOrder<'plan>,
    id: FragmentId,
}
impl<'plan> FragmentView<'plan> {
    pub const fn id(self) -> FragmentId {
        self.id
    }
    pub fn record(self) -> &'plan PaintFragment {
        &self.plan.request.fragments[self.id.get()]
    }
    pub fn owner(self) -> BoxView<'plan> {
        let id = match self.record().owner() {
            FragmentOwner::Box(id) => id,
            FragmentOwner::Text(id) => self.plan.request.texts[id.get()].parent(),
        };
        BoxView {
            plan: self.plan,
            id,
        }
    }
    pub fn text_source(self) -> Option<&'plan TextSource> {
        match self.record().owner() {
            FragmentOwner::Box(_) => None,
            FragmentOwner::Text(id) => Some(&self.plan.request.texts[id.get()]),
        }
    }
    pub fn parent(self) -> Option<Self> {
        self.record().parent().map(|id| Self {
            plan: self.plan,
            id,
        })
    }
    pub fn line(self) -> Option<LineView<'plan>> {
        self.record().line().map(|id| LineView {
            plan: self.plan,
            id,
        })
    }
}

#[derive(Clone, Copy, Debug)]
pub struct LineView<'plan> {
    plan: &'plan PaintOrder<'plan>,
    id: LineId,
}
impl<'plan> LineView<'plan> {
    pub const fn id(self) -> LineId {
        self.id
    }
    pub fn record(self) -> &'plan PaintLine {
        &self.plan.request.lines[self.id.get()]
    }
    pub fn container(self) -> FragmentView<'plan> {
        FragmentView {
            plan: self.plan,
            id: self.record().container(),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct PrimitiveView<'plan> {
    plan: &'plan PaintOrder<'plan>,
    id: PrimitiveId,
}
impl<'plan> PrimitiveView<'plan> {
    pub const fn id(self) -> PrimitiveId {
        self.id
    }
    pub fn record(self) -> &'plan PaintPrimitive {
        &self.plan.request.primitives[self.id.get()]
    }
    pub fn fragment(self) -> Option<FragmentView<'plan>> {
        self.record().fragment_id().map(|id| FragmentView {
            plan: self.plan,
            id,
        })
    }
    pub fn owner(self) -> BoxView<'plan> {
        self.fragment().map_or(
            BoxView {
                plan: self.plan,
                id: self.plan.request.root,
            },
            FragmentView::owner,
        )
    }
    pub fn applying_box(self) -> Option<BoxView<'plan>> {
        self.record().applying_box().map(|id| BoxView {
            plan: self.plan,
            id,
        })
    }
}

#[cfg(test)]
#[path = "model_tests.rs"]
mod tests;
