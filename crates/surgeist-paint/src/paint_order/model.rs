//! Raw resolved paint requests and borrowed, validated order observations.
//!
//! Request constructors preserve facts without validating cross-table relations.
//! Slot IDs are local to these exact borrowed tables: an in-range ID copied from
//! another request cannot be distinguished from a local ID. Native adapters retain
//! their authentic geometry, formatter/run/line and appearance associations by
//! these slots; these records do not authenticate computed Style or infer geometry.

use core::error::Error;
use std::fmt;

use crate::stacking::{BoxFacts, BoxId};
use surgeist_layout::node::SourceIndex;

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
impl BoxPaintKind {
    fn label(self) -> &'static str {
        match self {
            Self::Block => "block",
            Self::Inline => "inline",
            Self::InlineBlock => "inline-block",
            Self::BlockReplaced => "block replaced",
            Self::InlineReplaced => "inline replaced",
            Self::Table => "table",
            Self::InlineTable => "inline table",
        }
    }
}
impl fmt::Display for BoxCategoryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::RootMustBeBlock => "the root box must have a block category",
            Self::FloatingRoot => "the root box must not float",
            Self::FloatingInline => "a floating box must have a block category, not an inline one",
            Self::OutOfFlowInline => {
                "an out-of-flow box must have a block category, not an inline one"
            }
            Self::InlineItem => "a flex or grid item must have a block category, not an inline one",
            Self::FloatingItem => "a flex or grid item must not float",
            Self::ReplacedHasChildren => "a replaced box must not have children",
        })
    }
}
impl Error for BoxCategoryError {}

struct Owner(FragmentOwner);
impl fmt::Display for Owner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            FragmentOwner::Box(id) => write!(f, "box {}", id.get()),
            FragmentOwner::Text(id) => write!(f, "text {}", id.get()),
        }
    }
}
struct Child(SourceChild);
impl fmt::Display for Child {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            SourceChild::Box(id) => write!(f, "box {}", id.get()),
            SourceChild::Text(id) => write!(f, "text {}", id.get()),
        }
    }
}
struct Entry(ContentEntry);
impl fmt::Display for Entry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            ContentEntry::Line(id) => write!(f, "line {}", id.get()),
            ContentEntry::PromotedBlock(id) => write!(f, "promoted block fragment {}", id.get()),
        }
    }
}

impl fmt::Display for PaintInputError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::BoxPaintCount { boxes, box_paint } => {
                write!(
                    f,
                    "the box paint table has {box_paint} records for {boxes} boxes"
                )
            }
            Self::UnsupportedBoxKind { box_id, kind } => write!(
                f,
                "box {} has the {} category, which paint order does not support",
                box_id.get(),
                kind.label()
            ),
            Self::InvalidBoxCategory { box_id, .. } => {
                write!(f, "box {} has an invalid paint category", box_id.get())
            }
            Self::MissingTextParent { text, parent } => write!(
                f,
                "text {} names parent box {}, which is not in the box table",
                text.get(),
                parent.get()
            ),
            Self::InvalidTextParent { text, parent } => write!(
                f,
                "text {} names parent box {}, which cannot directly contain text",
                text.get(),
                parent.get()
            ),
            Self::DuplicateMixedSourceIndex {
                parent,
                source_index,
                first,
                other,
            } => write!(
                f,
                "box {} has two children, {} and {}, with source index {}",
                parent.get(),
                Child(first),
                Child(other),
                source_index.get()
            ),
            Self::MissingFragmentOwner { fragment, owner } => write!(
                f,
                "fragment {} names owner {}, which is not in its table",
                fragment.get(),
                Owner(owner)
            ),
            Self::MissingFragmentParent { fragment, parent } => write!(
                f,
                "fragment {} names parent fragment {}, which is not in the fragment table",
                fragment.get(),
                parent.get()
            ),
            Self::InvalidFragmentParent {
                fragment,
                parent: Some(parent),
            } => write!(
                f,
                "fragment {} has parent fragment {}, which does not match its owner's parent",
                fragment.get(),
                parent.get()
            ),
            Self::InvalidFragmentParent {
                fragment,
                parent: None,
            } => write!(
                f,
                "fragment {} has no parent fragment although its owner has a parent",
                fragment.get()
            ),
            Self::InvalidBlockInInlinePlacement {
                fragment,
                container,
            } => write!(
                f,
                "fragment {} is not a valid block-in-inline placement in container fragment {}",
                fragment.get(),
                container.get()
            ),
            Self::UnsupportedFragmentTopology {
                fragment,
                container,
                through,
            } => write!(
                f,
                "fragment {} reaches container fragment {} through box {}, a wrapper that paint order cannot skip",
                fragment.get(),
                container.get(),
                through.get()
            ),
            Self::DuplicateFragmentOrder {
                owner,
                logical_order,
                first,
                other,
            } => write!(
                f,
                "fragments {} and {} of {} share logical order {logical_order}",
                first.get(),
                other.get(),
                Owner(owner)
            ),
            Self::InconsistentFragmentOrder { first, other } => write!(
                f,
                "fragments {} and {} are ordered inconsistently with their parents and lines",
                first.get(),
                other.get()
            ),
            Self::MissingLineContainer { line, container } => write!(
                f,
                "line {} names container fragment {}, which is not in the fragment table",
                line.get(),
                container.get()
            ),
            Self::InvalidLineContainer { line, container } => write!(
                f,
                "line {} names container fragment {}, which cannot contain lines",
                line.get(),
                container.get()
            ),
            Self::DuplicateLineOrder {
                container,
                logical_order,
                first,
                other,
            } => write!(
                f,
                "lines {} and {} in container fragment {} share logical order {logical_order}",
                first.get(),
                other.get(),
                container.get()
            ),
            Self::DuplicateContentOrder {
                container,
                logical_order,
                first,
                other,
            } => write!(
                f,
                "{} and {} in container fragment {} share content order {logical_order}",
                Entry(first),
                Entry(other),
                container.get()
            ),
            Self::MissingFragmentLine { fragment, line } => write!(
                f,
                "fragment {} names line {}, which is not in the line table",
                fragment.get(),
                line.get()
            ),
            Self::MissingLineIncidence { fragment } => write!(
                f,
                "fragment {} must be on a line but names none",
                fragment.get()
            ),
            Self::UnexpectedLineIncidence { fragment, line } => write!(
                f,
                "fragment {} names line {} but does not belong on a line",
                fragment.get(),
                line.get()
            ),
            Self::InvalidLineIncidence { fragment, line } => write!(
                f,
                "fragment {} names line {}, which does not contain it",
                fragment.get(),
                line.get()
            ),
            Self::MissingPrimitiveFragment {
                primitive,
                fragment,
            } => write!(
                f,
                "primitive {} names fragment {}, which is not in the fragment table",
                primitive.get(),
                fragment.get()
            ),
            Self::InvalidPrimitiveTarget {
                primitive,
                fragment,
            } => write!(
                f,
                "primitive {} targets fragment {}, whose owner cannot receive its paint kind",
                primitive.get(),
                fragment.get()
            ),
            Self::RootFragmentBackground {
                primitive,
                fragment,
            } => write!(
                f,
                "primitive {} paints the root background on fragment {}; root backgrounds use canvas primitives",
                primitive.get(),
                fragment.get()
            ),
            Self::MissingDecorationApplicator {
                primitive,
                applying_box,
            } => write!(
                f,
                "decoration primitive {} names applying box {}, which is not in the box table",
                primitive.get(),
                applying_box.get()
            ),
            Self::InvalidDecorationApplicator {
                primitive,
                text_fragment,
                applying_box,
            } => write!(
                f,
                "decoration primitive {} names box {}, which does not decorate text fragment {}",
                primitive.get(),
                applying_box.get(),
                text_fragment.get()
            ),
            Self::DuplicatePrimitiveOrder {
                first,
                other,
                record_order,
            } => write!(
                f,
                "primitives {} and {} have the same record order {record_order} for the same target and paint role (and applying box, for a decoration)",
                first.get(),
                other.get()
            ),
        }
    }
}
impl Error for PaintInputError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidBoxCategory { reason, .. } => Some(reason),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub(crate) enum OrderEvent {
    BeginContext(BoxId),
    Draw(PrimitiveId),
    EndContext(BoxId),
}

pub use super::{BoxView, FragmentView, LineView, PaintOrder, PaintOrderEvent, PrimitiveView};

#[cfg(test)]
#[path = "model_tests.rs"]
mod tests;
