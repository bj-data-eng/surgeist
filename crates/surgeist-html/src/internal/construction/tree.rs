//! The published form of a construction result: one 20-byte row for each node, the children of
//! every row in one flat array, the associations of template and shadow contents in one table, and
//! the strings, names and attributes the rows name.
//!
//! Only publication makes a [`Tree`], and it checks every name, text range, run and leaf reference
//! once before it makes one, so the lookups of a reader resolve in a tree a storage made. A lookup
//! that does not resolve is reported as the typed [`HtmlNodeKind::Unreadable`] and never as a
//! value of a node.

use super::attribute_table::PublishedAttr;
use super::*;
use crate::internal::elements::rel::HtmlLinkFlags;
use std::{ops::Range, slice};

/// A range of the block of a result: the units of a text or comment row.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct Span {
    start: u32,
    len: u32,
}
impl Span {
    /// The range `range`, or `None` when a bound does not fit 32 bits.
    pub(super) fn new(range: Range<usize>) -> Option<Self> {
        let len = range.end.checked_sub(range.start)?;
        Some(Self {
            start: u32::try_from(range.start).ok()?,
            len: u32::try_from(len).ok()?,
        })
    }
    fn range(self) -> Range<usize> {
        // A `u32` fits `usize` on every supported target.
        let start = self.start as usize;
        start..start.saturating_add(self.len as usize)
    }
}

/// The attributes of one published element: a range of the attribute rows of the result.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct AttrSpan {
    start: u32,
    len: u32,
}
impl AttrSpan {
    /// The `len` rows from `start`, or `None` when a bound does not fit 32 bits.
    pub(super) fn new(start: usize, len: usize) -> Option<Self> {
        Some(Self {
            start: u32::try_from(start).ok()?,
            len: u32::try_from(len).ok()?,
        })
    }
    fn range(self) -> Range<usize> {
        let start = self.start as usize;
        start..start.saturating_add(self.len as usize)
    }
}

/// The properties of a shadow root packed into one byte: the bits of [`Self::pack`] and no other.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct PackedShadow(u8);
impl PackedShadow {
    const CLOSED: u8 = 1;
    const MANUAL: u8 = 2;
    const DELEGATES_FOCUS: u8 = 4;
    const CLONABLE: u8 = 8;
    const SERIALIZABLE: u8 = 16;

    /// The packed bits.
    pub(super) fn bits(self) -> u8 {
        self.0
    }
    /// The properties a row holds as the bits `bits`.
    pub(super) fn from_bits(bits: u8) -> Self {
        Self(bits)
    }
    pub(super) fn pack(properties: HtmlShadowRootProperties) -> Self {
        let mut bits = 0;
        if properties.mode == HtmlShadowRootMode::Closed {
            bits |= Self::CLOSED;
        }
        if properties.slot_assignment == HtmlSlotAssignment::Manual {
            bits |= Self::MANUAL;
        }
        if properties.delegates_focus {
            bits |= Self::DELEGATES_FOCUS;
        }
        if properties.clonable {
            bits |= Self::CLONABLE;
        }
        if properties.serializable {
            bits |= Self::SERIALIZABLE;
        }
        Self(bits)
    }
    pub(super) fn unpack(self) -> HtmlShadowRootProperties {
        let has = |bit: u8| self.0 & bit != 0;
        HtmlShadowRootProperties {
            mode: if has(Self::CLOSED) {
                HtmlShadowRootMode::Closed
            } else {
                HtmlShadowRootMode::Open
            },
            slot_assignment: if has(Self::MANUAL) {
                HtmlSlotAssignment::Manual
            } else {
                HtmlSlotAssignment::Named
            },
            delegates_focus: has(Self::DELEGATES_FOCUS),
            clonable: has(Self::CLONABLE),
            serializable: has(Self::SERIALIZABLE),
        }
    }
}

/// The owner of a fragment row: the template or the shadow host that holds it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct RowOwner {
    pub(super) node: NodeIx,
    pub(super) shadow_host: bool,
}

/// What a row holds besides its place in the tree.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum RowContent {
    /// The Document node, with its mode and whether its focused-area designation is known null.
    Document {
        mode: HtmlDocumentMode,
        known_null_focus: bool,
    },
    /// A fragment root, with its shadow properties when it is shadow contents and its owner when a
    /// template or a shadow host holds it.
    Fragment {
        shadow: Option<PackedShadow>,
        owner: Option<RowOwner>,
    },
    /// An element: its link history, the position of its name in the name table, and the
    /// position plus one of its run of attributes, absent for an element without attributes.
    Element {
        link_flags: Option<HtmlLinkFlags>,
        name: u32,
        run: Option<NonZeroU32>,
    },
    /// A text node.
    Text(Span),
    /// A comment node.
    Comment(Span),
    /// A processing instruction: its position among the leaves of the result.
    ProcessingInstruction(u32),
    /// A document type declaration: its position among the leaves of the result.
    Doctype(u32),
}

/// One node of a published result.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct PublishedRow {
    pub(super) parent: Option<NodeIx>,
    /// The end of this row's children in the child array. They start where the children of the
    /// previous row end, and the first row's start at zero.
    pub(super) child_end: u32,
    pub(super) content: RowContent,
}
const _: () = assert!(size_of::<PublishedRow>() == 20);

/// How an association relates its owner to its contents.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum AssocKind {
    /// The owner is a template element that owns the contents fragment.
    Owned,
    /// The owner is a template element that refers to shadow contents.
    ShadowReference,
    /// The owner is a shadow host and the contents are its shadow root, with their properties.
    Shadow(PackedShadow),
}

/// One association of an element with a contents fragment.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct Assoc {
    pub(super) owner: NodeIx,
    pub(super) contents: NodeIx,
    pub(super) kind: AssocKind,
}
const _: () = assert!(size_of::<Assoc>() == 12);

/// A published processing instruction or doctype, which a row names by its place among the leaves.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) enum Leaf {
    ProcessingInstruction(HtmlProcessingInstruction),
    Doctype(HtmlDoctype),
}

/// The content of a row with every reference it names resolved.
enum Resolved<'a> {
    Document(HtmlDocumentMode),
    Fragment,
    Element {
        entry: &'a Arc<NameEntry>,
        attrs: &'a [PublishedAttr],
        link_flags: Option<HtmlLinkFlags>,
    },
    Text(Span, &'a [u16]),
    Comment(Span, &'a [u16]),
    ProcessingInstruction(&'a HtmlProcessingInstruction),
    Doctype(&'a HtmlDoctype),
}

/// The immutable arena of a published result.
///
/// Rows are in the order the records were created. The children of every row are one run of
/// `children`, which holds each row that has a parent once, grouped by the parent in index order.
/// The associations are sorted by owner and, for one owner, by kind.
#[derive(Debug)]
pub(super) struct Tree {
    pub(super) rows: Vec<PublishedRow>,
    pub(super) children: Vec<NodeIx>,
    pub(super) associations: Vec<Assoc>,
    pub(super) names: NameTable,
    pub(super) attrs: Vec<PublishedAttr>,
    pub(super) runs: Vec<AttrSpan>,
    pub(super) block: TextBlock,
    pub(super) leaves: Vec<Leaf>,
    pub(super) usage: HtmlConstructionUsage,
    /// The serial of the storage that published this tree, which its node ids carry.
    pub(super) serial: u64,
}

impl Tree {
    pub(super) fn id(&self, index: NodeIx) -> HtmlNodeId {
        HtmlNodeId {
            tree: self.serial,
            index,
        }
    }

    /// The record at `position`, or `None` past the last record.
    pub(super) fn has_record(&self, position: usize) -> Option<NodeIx> {
        NodeIx::new(position).filter(|index| index.position() < self.rows.len())
    }

    /// The record index a builder handle names: `ForeignBuilder` for another storage's handle and
    /// `UnknownNode` for an index past the records.
    pub(super) fn index(&self, handle: &HtmlBuilderNode) -> Result<NodeIx, HtmlConstructionError> {
        if handle.tree != self.serial {
            return Err(HtmlConstructionCause::ForeignBuilder.into());
        }
        if handle.index.position() >= self.rows.len() {
            return Err(HtmlConstructionCause::UnknownNode.into());
        }
        Ok(handle.index)
    }

    /// The record index a result id names, or `ForeignHtmlResult` for the id of another storage
    /// or one past the records.
    pub(super) fn lookup_index(&self, id: &HtmlNodeId) -> Result<NodeIx, ForeignHtmlResult> {
        if self.serial != id.tree || id.index.position() >= self.rows.len() {
            return Err(ForeignHtmlResult);
        }
        Ok(id.index)
    }

    pub(super) fn lookup(&self, id: &HtmlNodeId) -> Result<HtmlNode<'_>, ForeignHtmlResult> {
        Ok(self.node(self.lookup_index(id)?))
    }

    /// The view of the row `index`, with its children. A row or a child run the tree does not hold
    /// gives a view without a row, which reads as [`HtmlNodeKind::Unreadable`]; a tree a storage
    /// published has none.
    pub(super) fn node(&self, index: NodeIx) -> HtmlNode<'_> {
        let position = index.position();
        let start = match position.checked_sub(1) {
            None => Some(0),
            Some(before) => self.rows.get(before).map(|row| row.child_end),
        };
        let resolved = self.rows.get(position).and_then(|row| {
            let run = self.children.get(start? as usize..row.child_end as usize)?;
            Some((row, run))
        });
        match resolved {
            Some((row, children)) => HtmlNode {
                tree: self,
                index,
                row: Some(row),
                children,
            },
            None => HtmlNode {
                tree: self,
                index,
                row: None,
                children: &[],
            },
        }
    }

    /// The entries of the association table whose owner is `owner`.
    pub(super) fn associations_of(&self, owner: NodeIx) -> impl Iterator<Item = &Assoc> {
        let first = self
            .associations
            .partition_point(|entry| entry.owner.0 < owner.0);
        self.associations
            .iter()
            .skip(first)
            .take_while(move |entry| entry.owner == owner)
    }

    /// The content of `row` with every reference it names resolved, or the cause of the one that
    /// does not.
    fn resolve(&self, row: &PublishedRow) -> Result<Resolved<'_>, HtmlConstructionCause> {
        let text = |span: Span| {
            self.block
                .get(span.range())
                .ok_or(HtmlConstructionCause::InvalidTextReference)
        };
        let leaf = |position: u32| usize::try_from(position).ok();
        Ok(match row.content {
            RowContent::Document { mode, .. } => Resolved::Document(mode),
            RowContent::Fragment { .. } => Resolved::Fragment,
            RowContent::Element {
                link_flags,
                name,
                run,
            } => {
                let entry = usize::try_from(name)
                    .ok()
                    .and_then(|position| self.names.get(position))
                    .ok_or(HtmlConstructionCause::InvalidNameReference)?;
                let attrs = match run {
                    None => &[][..],
                    Some(run) => usize::try_from(run.get())
                        .ok()
                        .and_then(|position| position.checked_sub(1))
                        .and_then(|position| self.runs.get(position))
                        .and_then(|span| self.attrs.get(span.range()))
                        .ok_or(HtmlConstructionCause::InvalidAttributeReference)?,
                };
                Resolved::Element {
                    entry,
                    attrs,
                    link_flags,
                }
            }
            RowContent::Text(span) => Resolved::Text(span, text(span)?),
            RowContent::Comment(span) => Resolved::Comment(span, text(span)?),
            RowContent::ProcessingInstruction(position) => {
                match leaf(position).and_then(|position| self.leaves.get(position)) {
                    Some(Leaf::ProcessingInstruction(value)) => {
                        Resolved::ProcessingInstruction(value)
                    }
                    _ => return Err(HtmlConstructionCause::InvalidLeafReference),
                }
            }
            RowContent::Doctype(position) => {
                match leaf(position).and_then(|position| self.leaves.get(position)) {
                    Some(Leaf::Doctype(value)) => Resolved::Doctype(value),
                    _ => return Err(HtmlConstructionCause::InvalidLeafReference),
                }
            }
        })
    }

    /// The published content of `row`.
    pub(super) fn kind<'a>(&'a self, row: &PublishedRow) -> HtmlNodeKind<'a> {
        let string = |span: Span| {
            self.block
                .string(span.range())
                .map_err(|_| HtmlConstructionCause::InvalidTextReference)
        };
        let read = || -> Result<HtmlNodeKind<'a>, HtmlConstructionCause> {
            Ok(match self.resolve(row)? {
                Resolved::Document(mode) => HtmlNodeKind::Document(mode),
                Resolved::Fragment => HtmlNodeKind::Fragment,
                Resolved::Element {
                    entry,
                    attrs,
                    link_flags,
                } => HtmlNodeKind::Element(HtmlElementView::new(entry, attrs, link_flags)),
                Resolved::Text(span, _) => HtmlNodeKind::Text(string(span)?),
                Resolved::Comment(span, _) => HtmlNodeKind::Comment(string(span)?),
                Resolved::ProcessingInstruction(value) => {
                    HtmlNodeKind::ProcessingInstruction(value.clone())
                }
                Resolved::Doctype(value) => HtmlNodeKind::Doctype(value.clone()),
            })
        };
        read().unwrap_or_else(HtmlNodeKind::Unreadable)
    }

    /// The content of `row` for a view of a record, with its text as borrowed code units.
    pub(super) fn build_kind<'a>(&'a self, row: &PublishedRow) -> ConstructionNodeKind<'a> {
        match self.resolve(row) {
            Err(cause) => ConstructionNodeKind::Unreadable(cause),
            Ok(Resolved::Document(mode)) => ConstructionNodeKind::Document(mode),
            Ok(Resolved::Fragment) => ConstructionNodeKind::Fragment,
            Ok(Resolved::Element {
                entry,
                attrs,
                link_flags,
            }) => ConstructionNodeKind::Element(ConstructionElementView::published(
                entry, attrs, link_flags,
            )),
            Ok(Resolved::Text(_, units)) => ConstructionNodeKind::Text(BuildText::Units(units)),
            Ok(Resolved::Comment(_, units)) => {
                ConstructionNodeKind::Comment(BuildText::Units(units))
            }
            Ok(Resolved::ProcessingInstruction(value)) => {
                ConstructionNodeKind::ProcessingInstruction(ConstructionProcessingInstruction::of(
                    value,
                ))
            }
            Ok(Resolved::Doctype(value)) => {
                ConstructionNodeKind::Doctype(ConstructionDoctype::of(value))
            }
        }
    }
}

#[cfg(test)]
impl Tree {
    /// A tree of the rows `rows` and no other table, which no storage published, for the tests of
    /// what a reader answers for a reference the tree does not hold.
    pub(super) fn forged(rows: Vec<PublishedRow>) -> Self {
        Self {
            rows,
            children: Vec::new(),
            associations: Vec::new(),
            names: NameTable::new(),
            attrs: Vec::new(),
            runs: Vec::new(),
            block: TextBlock::default(),
            leaves: Vec::new(),
            usage: HtmlConstructionUsage::default(),
            serial: 0,
        }
    }
}

/// The ids of the children of a node, from both ends. `nth` and `nth_back` skip by the slice's own
/// constant-time step, so the child at an index is reached without a visit of the ones before it.
#[derive(Clone, Debug)]
pub(super) struct ChildIds<'a> {
    serial: u64,
    ids: slice::Iter<'a, NodeIx>,
}
impl<'a> ChildIds<'a> {
    pub(super) fn new(serial: u64, ids: &'a [NodeIx]) -> Self {
        Self {
            serial,
            ids: ids.iter(),
        }
    }
    fn id(&self, index: &NodeIx) -> HtmlNodeId {
        #[cfg(test)]
        mapped::count();
        HtmlNodeId {
            tree: self.serial,
            index: *index,
        }
    }
}
impl Iterator for ChildIds<'_> {
    type Item = HtmlNodeId;
    fn next(&mut self) -> Option<Self::Item> {
        self.ids.next().map(|index| self.id(index))
    }
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.ids.nth(n).map(|index| self.id(index))
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.ids.size_hint()
    }
}
impl DoubleEndedIterator for ChildIds<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.ids.next_back().map(|index| self.id(index))
    }
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        self.ids.nth_back(n).map(|index| self.id(index))
    }
}
impl ExactSizeIterator for ChildIds<'_> {}

/// The number of child ids made on this thread, for the tests of what a lookup by index visits.
#[cfg(test)]
pub(super) mod mapped {
    use std::cell::Cell;
    thread_local! {
        static MADE: Cell<usize> = const { Cell::new(0) };
    }
    pub(super) fn count() {
        MADE.with(|made| made.set(made.get().saturating_add(1)));
    }
    pub(crate) fn made() -> usize {
        MADE.with(Cell::get)
    }
}
