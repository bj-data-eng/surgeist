//! The build form of a node: one 32-byte row for each node of a storage that builds, with the links
//! that give its place among its siblings, and the table of its template and shadow associations.
//!
//! A row holds its parent, its first and last child, its next and previous sibling, its kind, its
//! flags, and two 32-bit words whose meaning the kind gives:
//!
//! | Kind | `a` | `b` | Flags |
//! | --- | --- | --- | --- |
//! | Document | 0, the mode is in the kind | 0 | `KNOWN_NULL_FOCUS` |
//! | Fragment | the shadow properties, with bit 7 set for shadow contents, and 0 otherwise | the owner | `OWNED` |
//! | Element | the position of the name in the name table | the run of the attributes, 0 for none | `LINK_PRESENT`, `LINK_ENABLED`, `HAS_ASSOC` |
//! | Text, Comment | the start of the text in the arena | its length | none |
//! | Processing instruction, Doctype | the position among the leaves | 0 | none |
//!
//! A text or comment has no children, so the slot of its first child holds the units reserved
//! after it. No other kind keeps a value there. Depth is a column of its own, which a storage keeps
//! while it builds, so a row stays 32 bytes.

use super::attribute_table::RunIx;
use super::text::{TextRef, TextRun};
use super::tree::PackedShadow;
use super::*;

/// What a row holds besides its place in the tree.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum RowKind {
    /// The Document node, with its compatibility mode.
    Document(HtmlDocumentMode),
    Fragment,
    Element,
    Text,
    Comment,
    ProcessingInstruction,
    Doctype,
}
impl RowKind {
    /// Whether a row of this kind can hold ordinary children.
    pub(super) fn is_container(self) -> bool {
        matches!(self, Self::Document(_) | Self::Fragment | Self::Element)
    }
    /// Whether the units of a row of this kind are a range of the text arena.
    pub(super) fn is_text(self) -> bool {
        matches!(self, Self::Text | Self::Comment)
    }
    /// Whether a row of this kind names a leaf of the storage.
    pub(super) fn is_leaf(self) -> bool {
        matches!(self, Self::ProcessingInstruction | Self::Doctype)
    }
}

/// The flags of a row.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(super) struct NodeFlags(u8);
impl NodeFlags {
    /// The element is a `link` element and holds its link history.
    pub(super) const LINK_PRESENT: u8 = 1;
    /// The link was explicitly enabled.
    pub(super) const LINK_ENABLED: u8 = 2;
    /// The node is the owner of an entry of the association table.
    pub(super) const HAS_ASSOC: u8 = 4;
    /// The fragment has an owner, which its `b` names.
    pub(super) const OWNED: u8 = 8;
    /// The Document's focused-area designation is known null.
    pub(super) const KNOWN_NULL_FOCUS: u8 = 16;

    pub(super) fn has(self, flag: u8) -> bool {
        self.0 & flag != 0
    }
    pub(super) fn set(&mut self, flag: u8, on: bool) {
        if on {
            self.0 |= flag;
        } else {
            self.0 &= !flag;
        }
    }
    pub(super) fn of(flags: &[u8]) -> Self {
        Self(flags.iter().fold(0, |all, flag| all | flag))
    }
}

/// The bit of a fragment's `a` that says it is shadow contents.
const SHADOW_PRESENT: u32 = 0x80;

/// One node of a storage that builds.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct NodeRow {
    pub(super) parent: Option<NodeIx>,
    pub(super) first_child: Option<NodeIx>,
    pub(super) last_child: Option<NodeIx>,
    pub(super) next: Option<NodeIx>,
    pub(super) prev: Option<NodeIx>,
    pub(super) kind: RowKind,
    pub(super) flags: NodeFlags,
    pub(super) a: u32,
    pub(super) b: u32,
}
#[cfg(target_pointer_width = "64")]
const _: () = assert!(size_of::<NodeRow>() == 32);

impl NodeRow {
    fn bare(kind: RowKind, flags: NodeFlags, a: u32, b: u32) -> Self {
        Self {
            parent: None,
            first_child: None,
            last_child: None,
            next: None,
            prev: None,
            kind,
            flags,
            a,
            b,
        }
    }
    /// The Document row of `mode`, whose focused-area designation is known null.
    pub(super) fn document(mode: HtmlDocumentMode) -> Self {
        Self::bare(
            RowKind::Document(mode),
            NodeFlags::of(&[NodeFlags::KNOWN_NULL_FOCUS]),
            0,
            0,
        )
    }
    /// The fragment row, with the properties of shadow contents when it is some.
    pub(super) fn fragment(shadow: Option<HtmlShadowRootProperties>) -> Self {
        let a = shadow.map_or(0, |properties| {
            SHADOW_PRESENT | u32::from(PackedShadow::pack(properties).bits())
        });
        Self::bare(RowKind::Fragment, NodeFlags::default(), a, 0)
    }
    /// The element row of the name entry `name`, with the history of a `link`.
    pub(super) fn element(
        name: u32,
        link_flags: Option<crate::internal::elements::rel::HtmlLinkFlags>,
    ) -> Self {
        let mut flags = NodeFlags::default();
        flags.set(NodeFlags::LINK_PRESENT, link_flags.is_some());
        flags.set(
            NodeFlags::LINK_ENABLED,
            link_flags.is_some_and(|flags| flags.explicitly_enabled()),
        );
        Self::bare(RowKind::Element, flags, name, 0)
    }
    /// The text or comment row of `run`.
    pub(super) fn text(kind: RowKind, run: TextRun) -> Self {
        let mut row = Self::bare(
            kind,
            NodeFlags::default(),
            run.text().start(),
            run.text().stored_len(),
        );
        row.first_child = NodeIx::from_stored(run.room());
        row
    }
    /// The processing-instruction or doctype row that names the leaf at `position`.
    pub(super) fn leaf(kind: RowKind, position: u32) -> Self {
        Self::bare(kind, NodeFlags::default(), position, 0)
    }

    /// The shadow properties of a fragment row, or `None` for any other row.
    pub(super) fn shadow(&self) -> Option<PackedShadow> {
        (self.kind == RowKind::Fragment && self.a & SHADOW_PRESENT != 0)
            .then(|| PackedShadow::from_bits((self.a & !SHADOW_PRESENT) as u8))
    }
    /// The owner of a fragment row that has one.
    pub(super) fn owner(&self) -> Option<NodeIx> {
        if self.kind == RowKind::Fragment && self.flags.has(NodeFlags::OWNED) {
            NodeIx::from_stored(self.b)
        } else {
            None
        }
    }
    /// The same fragment row, owned by `owner`, or by none.
    pub(super) fn set_owner(&mut self, owner: Option<NodeIx>) {
        self.flags.set(NodeFlags::OWNED, owner.is_some());
        self.b = owner.map_or(0, NodeIx::stored);
    }
    /// The run of the attributes of an element row.
    pub(super) fn run(&self) -> Option<RunIx> {
        if self.kind == RowKind::Element {
            RunIx::from_stored(self.b)
        } else {
            None
        }
    }
    /// The link history of an element row that holds one.
    pub(super) fn link_flags(&self) -> Option<crate::internal::elements::rel::HtmlLinkFlags> {
        if self.kind != RowKind::Element || !self.flags.has(NodeFlags::LINK_PRESENT) {
            return None;
        }
        let mut flags = crate::internal::elements::rel::HtmlLinkFlags::fresh();
        if self.flags.has(NodeFlags::LINK_ENABLED) {
            flags.disabled_removed();
        }
        Some(flags)
    }
    /// The run of the text of a text or comment row.
    pub(super) fn text_run(&self) -> Option<TextRun> {
        self.kind.is_text().then(|| {
            TextRun::from_parts(
                TextRef::from_stored(self.a, self.b),
                self.first_child.map_or(0, NodeIx::stored),
            )
        })
    }
    /// The same text or comment row, holding `run`.
    pub(super) fn set_text_run(&mut self, run: TextRun) {
        self.a = run.text().start();
        self.b = run.text().stored_len();
        self.first_child = NodeIx::from_stored(run.room());
    }
    /// The position among the leaves of a processing-instruction or doctype row.
    pub(super) fn leaf_position(&self) -> Option<u32> {
        self.kind.is_leaf().then_some(self.a)
    }
    /// The first child of a row that can hold children; a row that cannot has none.
    pub(super) fn first(&self) -> Option<NodeIx> {
        if self.kind.is_container() {
            self.first_child
        } else {
            None
        }
    }
    /// The last child of a row that can hold children; a row that cannot has none.
    pub(super) fn last(&self) -> Option<NodeIx> {
        if self.kind.is_container() {
            self.last_child
        } else {
            None
        }
    }
    /// Whether the row has nothing below it among its ordinary children.
    pub(super) fn is_childless(&self) -> bool {
        !self.kind.is_container() || self.first_child.is_none()
    }
}

/// How an association relates its owner to its contents.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum BuildAssocKind {
    /// The owner is a template element that owns the contents.
    Owned,
    /// The owner is a template element that refers to shadow contents.
    ShadowReference,
    /// The owner is a shadow host and the contents are its shadow root.
    Shadow,
}
impl BuildAssocKind {
    /// Whether the entry is the contents of a template, owned or a reference.
    pub(super) fn is_contents(self) -> bool {
        matches!(self, Self::Owned | Self::ShadowReference)
    }
}

/// One association of an element with a contents fragment.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct BuildAssoc {
    pub(super) owner: NodeIx,
    pub(super) contents: NodeIx,
    pub(super) kind: BuildAssocKind,
}
#[cfg(target_pointer_width = "64")]
const _: () = assert!(size_of::<BuildAssoc>() == 12);

/// The table of the associations of a storage, sorted by owner and, for one owner, by kind.
pub(super) trait AssocTable {
    /// The position of the first entry that sorts at or after `owner` and `kind`.
    fn place(&self, owner: NodeIx, kind: BuildAssocKind) -> usize;
    /// The entries whose owner is `owner`.
    fn of(&self, owner: NodeIx) -> &[BuildAssoc];
}
fn rank(kind: BuildAssocKind) -> u8 {
    match kind {
        BuildAssocKind::Owned | BuildAssocKind::ShadowReference => 0,
        BuildAssocKind::Shadow => 1,
    }
}
impl AssocTable for [BuildAssoc] {
    fn place(&self, owner: NodeIx, kind: BuildAssocKind) -> usize {
        self.partition_point(|entry| {
            (entry.owner.stored(), rank(entry.kind)) < (owner.stored(), rank(kind))
        })
    }
    fn of(&self, owner: NodeIx) -> &[BuildAssoc] {
        let first = self.partition_point(|entry| entry.owner.stored() < owner.stored());
        let rest = self.get(first..).unwrap_or_default();
        let len = rest.partition_point(|entry| entry.owner == owner);
        rest.get(..len).unwrap_or_default()
    }
}
