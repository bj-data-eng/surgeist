//! Output-only node views over the construction records.
//!
//! The published view, [`HtmlNodeKind`], carries shared strings and is returned by value from
//! [`HtmlNode::kind`]. The build view, [`ConstructionNodeKind`], carries borrowed code units and is
//! returned by value from `HtmlConstructionNode::kind`. Only this crate constructs
//! [`HtmlElementView`], [`HtmlAttributes`] and [`BuildStr`], and none of the views exposes how the
//! record stores its content.

use super::attribute_table::{AttrRow, PublishedAttr};
use super::names::NameEntry;
use super::rows::{NodeRow, RowKind};
use super::text::TextRun;
use super::tree::Leaf;
use super::*;
use crate::internal::elements::rel::HtmlLinkFlags;
use crate::internal::values::{Attribute, Namespace};
use std::slice;

/// The content of one published node, by value.
///
/// A text, comment, processing-instruction or doctype payload shares the node's stored strings; an
/// element borrows its name and attributes from the result.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlNodeKind<'a> {
    /// The Document node, carrying the compatibility mode selected when it was built.
    Document(HtmlDocumentMode),
    /// A document-fragment root.
    Fragment,
    /// An element with its exact qualified name and ordered raw attributes.
    Element(HtmlElementView<'a>),
    /// A text node; the payload preserves the exact UTF-16 code units.
    Text(DocumentString),
    /// A comment node; the payload preserves the exact UTF-16 code units.
    Comment(DocumentString),
    /// A processing instruction recorded with its exact target and data.
    ProcessingInstruction(HtmlProcessingInstruction),
    /// A document type declaration.
    Doctype(HtmlDoctype),
    /// A reference of the node's row that its result does not resolve: the cause names the kind
    /// of reference, an interned name, a text range, a run of attributes, a leaf or the row
    /// itself. Publication validates every such reference once and a result cannot be made without
    /// that pass, so a result that a storage published never reads as this; a consumer maps it to
    /// its typed defect error and never skips the node.
    Unreadable(HtmlConstructionCause),
}

/// A borrowed element: its exact name, ordered attributes and link history.
#[derive(Clone, Copy, Debug)]
pub struct HtmlElementView<'a> {
    entry: &'a NameEntry,
    attrs: &'a [PublishedAttr],
    link_flags: Option<HtmlLinkFlags>,
}
impl PartialEq for HtmlElementView<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.entry.name() == other.entry.name()
            && self.attrs == other.attrs
            && self.link_flags == other.link_flags
    }
}
impl Eq for HtmlElementView<'_> {}
impl<'a> HtmlElementView<'a> {
    pub(super) fn new(
        entry: &'a NameEntry,
        attrs: &'a [PublishedAttr],
        link_flags: Option<HtmlLinkFlags>,
    ) -> Self {
        Self {
            entry,
            attrs,
            link_flags,
        }
    }
    /// Exact qualified name.
    pub fn name(self) -> &'a QualifiedName {
        self.entry.name()
    }
    /// The key of the element's name, which the tests compare with the vocabulary.
    #[cfg(test)]
    pub(crate) fn key(self) -> crate::internal::values::element_key::ElementKey {
        self.entry.key()
    }
    /// The element's attributes in producer order.
    pub fn attributes(self) -> HtmlAttributes<'a> {
        HtmlAttributes { rows: self.attrs }
    }
    /// Actual construction flags. Records of a result that kept no link history return `None`.
    pub fn link_flags(self) -> Option<HtmlLinkFlags> {
        self.link_flags
    }
}

/// One attribute of a published element: its exact name and its raw value, which are borrowed from
/// the result that holds them.
///
/// The value is a range of the one block of the result, so cloning it shares that block and
/// copies no unit.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlAttribute<'a> {
    name: &'a QualifiedName,
    value: &'a DocumentString,
}
impl<'a> HtmlAttribute<'a> {
    fn of(row: &'a PublishedAttr) -> Self {
        Self {
            name: row.name(),
            value: row.value(),
        }
    }
    /// The qualified name.
    pub fn name(self) -> &'a QualifiedName {
        self.name
    }
    /// The exact raw value.
    pub fn value(self) -> &'a DocumentString {
        self.value
    }
}
impl crate::internal::values::semantic_input::HtmlAttributeInput for HtmlAttribute<'_> {
    type Name = QualifiedName;
    fn name(&self) -> &QualifiedName {
        self.name
    }
    fn value(&self) -> &DocumentString {
        self.value
    }
}

/// The iterator of the attributes of a published element.
struct PublishedAttributes<'a>(slice::Iter<'a, PublishedAttr>);
impl<'a> Iterator for PublishedAttributes<'a> {
    type Item = HtmlAttribute<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(HtmlAttribute::of)
    }
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.0.nth(n).map(HtmlAttribute::of)
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}
impl DoubleEndedIterator for PublishedAttributes<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.next_back().map(HtmlAttribute::of)
    }
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        self.0.nth_back(n).map(HtmlAttribute::of)
    }
}
impl ExactSizeIterator for PublishedAttributes<'_> {}

/// The ordered attributes of one element, borrowed from the result or storage that holds it.
///
/// Attributes keep their producer order and are unique by namespace and local name.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlAttributes<'a> {
    rows: &'a [PublishedAttr],
}
impl<'a> HtmlAttributes<'a> {
    /// The number of attributes.
    pub fn len(self) -> usize {
        self.rows.len()
    }
    /// Whether there are no attributes.
    pub fn is_empty(self) -> bool {
        self.len() == 0
    }
    /// The attribute at a zero-based index in producer order; `None` at or past `len`.
    pub fn get(self, index: usize) -> Option<HtmlAttribute<'a>> {
        self.rows.get(index).map(HtmlAttribute::of)
    }
    /// Iterates in producer order.
    pub fn iter(
        self,
    ) -> impl ExactSizeIterator<Item = HtmlAttribute<'a>> + DoubleEndedIterator + 'a {
        PublishedAttributes(self.rows.iter())
    }
    /// Finds an attribute by exact namespace and local-name identity.
    pub fn find(
        self,
        namespace: &Namespace,
        local_name: &DocumentString,
    ) -> Option<HtmlAttribute<'a>> {
        self.iter().find(|attribute| {
            attribute.name().namespace() == namespace && attribute.name().local_name() == local_name
        })
    }
    /// The attributes as an owned ordered collection, in producer order. The names and values
    /// are shared with the result, and no unit is copied.
    pub fn to_ordered(self) -> OrderedAttributes {
        // The collection has room for exactly the attributes: one that collected an iterator of
        // unknown exact size would reserve more, and holders charge the capacity.
        let mut attributes = Vec::with_capacity(self.len());
        attributes.extend(
            self.iter().map(|attribute| {
                Attribute::new(attribute.name().clone(), attribute.value().clone())
            }),
        );
        OrderedAttributes::from_unique(attributes)
    }
}
impl crate::internal::values::semantic_input::HtmlAttributesInput for HtmlAttributes<'_> {
    type Attribute<'a>
        = HtmlAttribute<'a>
    where
        Self: 'a;
    fn len(&self) -> usize {
        HtmlAttributes::len(*self)
    }
    fn get(&self, index: usize) -> Option<HtmlAttribute<'_>> {
        HtmlAttributes::get(*self, index)
    }
}

/// A borrowed run of UTF-16 code units of a record that construction is still extending.
///
/// It names the units the record holds when the view is made and holds no share of their block, so
/// it cannot outlive the borrow of the storage.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BuildStr<'a>(&'a [u16]);
impl<'a> BuildStr<'a> {
    pub(super) fn new(units: &'a [u16]) -> Self {
        Self(units)
    }
    /// The exact code units, including nulls and isolated surrogates.
    pub fn code_units(self) -> &'a [u16] {
        self.0
    }
    /// The number of code units.
    pub fn len(self) -> usize {
        self.0.len()
    }
    /// Whether there are no code units.
    pub fn is_empty(self) -> bool {
        self.0.is_empty()
    }
}

/// The text or comment of a record of a construction storage: the units it holds when the view is
/// made, or the refusal that the record names units its storage does not hold.
///
/// Like [`BuildStr`] it holds no share of a block, so it cannot outlive the borrow of the storage.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BuildText<'a> {
    /// The units of the text.
    Units(&'a [u16]),
    /// The record names a range its text arena does not hold. A storage never makes one; the
    /// view reports it as a typed refusal instead of a text.
    Invalid,
}
impl<'a> BuildText<'a> {
    /// The exact code units, including nulls and isolated surrogates, or
    /// [`HtmlConstructionCause::InvalidTextReference`] for a record whose range its arena does
    /// not hold.
    pub fn code_units(self) -> Result<&'a [u16], HtmlConstructionError> {
        match self {
            Self::Units(units) => Ok(units),
            Self::Invalid => Err(HtmlConstructionCause::InvalidTextReference.error()),
        }
    }
    /// The number of code units, with the refusal of [`Self::code_units`].
    pub fn len(self) -> Result<usize, HtmlConstructionError> {
        self.code_units().map(<[u16]>::len)
    }
    /// Whether there are no code units, with the refusal of [`Self::code_units`].
    pub fn is_empty(self) -> Result<bool, HtmlConstructionError> {
        self.code_units().map(<[u16]>::is_empty)
    }
}

/// The content of one record of a construction storage, by value.
///
/// Its text is [`BuildText`], so the view hands out no string that construction can still
/// extend.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConstructionNodeKind<'a> {
    /// The Document node, carrying the compatibility mode selected when it was built.
    Document(HtmlDocumentMode),
    /// A document-fragment root.
    Fragment,
    /// An element with its exact qualified name and ordered raw attributes.
    Element(ConstructionElementView<'a>),
    /// A text node.
    Text(BuildText<'a>),
    /// A comment node.
    Comment(BuildText<'a>),
    /// A processing instruction recorded with its exact target and data.
    ProcessingInstruction(ConstructionProcessingInstruction<'a>),
    /// A document type declaration.
    Doctype(ConstructionDoctype<'a>),
    /// A reference of a published record's row that its result does not resolve; see
    /// [`HtmlNodeKind::Unreadable`].
    Unreadable(HtmlConstructionCause),
}

/// One attribute of a record of a construction storage: its exact name, and its value as the
/// units it holds when the view is made or the refusal that its row names a value its text arena
/// does not hold.
///
/// Like [`BuildText`] it holds no share of a block, so it cannot outlive the borrow of the
/// storage.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BuildAttribute<'a> {
    name: &'a QualifiedName,
    value: BuildText<'a>,
}
impl<'a> BuildAttribute<'a> {
    fn of(row: &'a AttrRow, text: &'a TextArena) -> Self {
        Self {
            name: row.name(),
            value: text
                .get(row.value())
                .map_or(BuildText::Invalid, BuildText::Units),
        }
    }
    fn published(row: &'a PublishedAttr) -> Self {
        Self {
            name: row.name(),
            value: BuildText::Units(row.value().code_units()),
        }
    }
    /// The qualified name.
    pub fn name(self) -> &'a QualifiedName {
        self.name
    }
    /// The exact value units, or [`HtmlConstructionCause::InvalidTextReference`] from
    /// [`BuildText::code_units`] for a row whose range its arena does not hold.
    pub fn value(self) -> BuildText<'a> {
        self.value
    }
}

/// Where the attributes of a build view are read from.
#[derive(Clone, Copy, Debug)]
enum AttrSource<'a> {
    /// The live rows of the run of a record of a storage that builds, with the arena its values
    /// are ranges of.
    Rows {
        rows: &'a [AttrRow],
        text: &'a TextArena,
    },
    /// The attributes of a record of a published result.
    Published(&'a [PublishedAttr]),
    /// The record names a run its storage does not hold.
    Invalid,
}

/// The ordered attributes of an element record of a construction storage.
///
/// A record of a storage that builds names its attributes by a run of the storage's table, and
/// the view reports the refusal [`HtmlConstructionCause::InvalidAttributeReference`] where the
/// storage does not hold it. A storage never makes such a record.
#[derive(Clone, Copy, Debug)]
pub struct BuildAttributes<'a> {
    source: AttrSource<'a>,
}
impl<'a> BuildAttributes<'a> {
    /// The number of attributes, or the refusal that the record names a run its storage does not
    /// hold.
    pub fn len(self) -> Result<usize, HtmlConstructionError> {
        match self.source {
            AttrSource::Rows { rows, .. } => Ok(rows.len()),
            AttrSource::Published(rows) => Ok(rows.len()),
            AttrSource::Invalid => Err(HtmlConstructionCause::InvalidAttributeReference.error()),
        }
    }
    /// Whether there are no attributes, with the refusal of [`Self::len`].
    pub fn is_empty(self) -> Result<bool, HtmlConstructionError> {
        self.len().map(|len| len == 0)
    }
    /// The attribute at a zero-based index in producer order; `None` at or past `len`. The
    /// refusal is that of [`Self::len`].
    pub fn get(self, index: usize) -> Result<Option<BuildAttribute<'a>>, HtmlConstructionError> {
        match self.source {
            AttrSource::Rows { rows, text } => {
                Ok(rows.get(index).map(|row| BuildAttribute::of(row, text)))
            }
            AttrSource::Published(rows) => Ok(rows.get(index).map(BuildAttribute::published)),
            AttrSource::Invalid => Err(HtmlConstructionCause::InvalidAttributeReference.error()),
        }
    }
    /// Iterates in producer order, with the refusal of [`Self::len`].
    pub fn iter(self) -> Result<BuildAttributeIter<'a>, HtmlConstructionError> {
        Ok(BuildAttributeIter(match self.source {
            AttrSource::Rows { rows, text } => IterSource::Rows(rows.iter(), text),
            AttrSource::Published(rows) => IterSource::Published(rows.iter()),
            AttrSource::Invalid => {
                return Err(HtmlConstructionCause::InvalidAttributeReference.error());
            }
        }))
    }
}
impl PartialEq for BuildAttributes<'_> {
    fn eq(&self, other: &Self) -> bool {
        match (self.iter(), other.iter()) {
            (Ok(left), Ok(right)) => left.eq(right),
            (Err(left), Err(right)) => left == right,
            _ => false,
        }
    }
}
impl Eq for BuildAttributes<'_> {}

#[derive(Clone, Debug)]
enum IterSource<'a> {
    Rows(slice::Iter<'a, AttrRow>, &'a TextArena),
    Published(slice::Iter<'a, PublishedAttr>),
}

/// The attributes of a record of a construction storage, in producer order.
#[derive(Clone, Debug)]
pub struct BuildAttributeIter<'a>(IterSource<'a>);
impl<'a> Iterator for BuildAttributeIter<'a> {
    type Item = BuildAttribute<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.0 {
            IterSource::Rows(rows, text) => rows.next().map(|row| BuildAttribute::of(row, text)),
            IterSource::Published(rows) => rows.next().map(BuildAttribute::published),
        }
    }
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        match &mut self.0 {
            IterSource::Rows(rows, text) => rows.nth(n).map(|row| BuildAttribute::of(row, text)),
            IterSource::Published(rows) => rows.nth(n).map(BuildAttribute::published),
        }
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        match &self.0 {
            IterSource::Rows(rows, _) => rows.size_hint(),
            IterSource::Published(rows) => rows.size_hint(),
        }
    }
}
impl DoubleEndedIterator for BuildAttributeIter<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        match &mut self.0 {
            IterSource::Rows(rows, text) => {
                rows.next_back().map(|row| BuildAttribute::of(row, text))
            }
            IterSource::Published(rows) => rows.next_back().map(BuildAttribute::published),
        }
    }
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        match &mut self.0 {
            IterSource::Rows(rows, text) => {
                rows.nth_back(n).map(|row| BuildAttribute::of(row, text))
            }
            IterSource::Published(rows) => rows.nth_back(n).map(BuildAttribute::published),
        }
    }
}
impl ExactSizeIterator for BuildAttributeIter<'_> {}

/// A borrowed element record of a construction storage: its exact name, ordered attributes and
/// link history.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ConstructionElementView<'a> {
    name: &'a QualifiedName,
    #[cfg(test)]
    key: crate::internal::values::element_key::ElementKey,
    link_flags: Option<HtmlLinkFlags>,
    attributes: BuildAttributes<'a>,
}
impl<'a> ConstructionElementView<'a> {
    /// Exact qualified name.
    pub fn name(self) -> &'a QualifiedName {
        self.name
    }
    /// The key of the element's name, which the tests compare with the vocabulary.
    #[cfg(test)]
    pub(crate) fn key(self) -> crate::internal::values::element_key::ElementKey {
        self.key
    }
    /// The element's attributes in producer order.
    pub fn attributes(self) -> BuildAttributes<'a> {
        self.attributes
    }
    /// Actual construction flags. Records of a result that kept no link history return `None`.
    pub fn link_flags(self) -> Option<HtmlLinkFlags> {
        self.link_flags
    }
    /// The view of an element record of a storage that builds, whose attributes are the rows of
    /// its run in `table` and whose values are ranges of `text`.
    fn building(
        entry: &'a NameEntry,
        row: &NodeRow,
        table: &'a AttributeTable,
        text: &'a TextArena,
    ) -> Self {
        Self {
            name: entry.name(),
            #[cfg(test)]
            key: entry.key(),
            link_flags: row.link_flags(),
            attributes: BuildAttributes {
                source: match table.live(row.run()) {
                    Ok(rows) => AttrSource::Rows { rows, text },
                    Err(_) => AttrSource::Invalid,
                },
            },
        }
    }
    /// The view of an element record of a published result.
    pub(super) fn published(
        entry: &'a NameEntry,
        rows: &'a [PublishedAttr],
        link_flags: Option<HtmlLinkFlags>,
    ) -> Self {
        Self {
            name: entry.name(),
            #[cfg(test)]
            key: entry.key(),
            link_flags,
            attributes: BuildAttributes {
                source: AttrSource::Published(rows),
            },
        }
    }
}

/// The target and data of a processing-instruction record, as borrowed code units.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ConstructionProcessingInstruction<'a> {
    target: BuildStr<'a>,
    data: BuildStr<'a>,
}
impl<'a> ConstructionProcessingInstruction<'a> {
    pub(super) fn of(value: &'a HtmlProcessingInstruction) -> Self {
        Self {
            target: BuildStr::new(value.target().code_units()),
            data: BuildStr::new(value.data().code_units()),
        }
    }
    /// Exact target.
    pub fn target(self) -> BuildStr<'a> {
        self.target
    }
    /// Exact data.
    pub fn data(self) -> BuildStr<'a> {
        self.data
    }
}

/// The name and identifiers of a doctype record, as borrowed code units.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ConstructionDoctype<'a> {
    name: BuildStr<'a>,
    public_id: Option<BuildStr<'a>>,
    system_id: Option<BuildStr<'a>>,
}
impl<'a> ConstructionDoctype<'a> {
    pub(super) fn of(value: &'a HtmlDoctype) -> Self {
        let units = |value: &'a DocumentString| BuildStr::new(value.code_units());
        Self {
            name: units(value.name()),
            public_id: value.public_id().map(units),
            system_id: value.system_id().map(units),
        }
    }
    /// Exact doctype name.
    pub fn name(self) -> BuildStr<'a> {
        self.name
    }
    /// Exact optional public identifier; absent and empty stay distinct.
    pub fn public_id(self) -> Option<BuildStr<'a>> {
        self.public_id
    }
    /// Exact optional system identifier; absent and empty stay distinct.
    pub fn system_id(self) -> Option<BuildStr<'a>> {
        self.system_id
    }
}

impl BuildArena {
    /// The build view of the content of `row`, whose text and comment units and attribute values
    /// are read from `text`, whose attributes are the rows of `table`, and whose names and leaves
    /// are read from this arena. A reference the storage does not hold is reported as the typed
    /// [`ConstructionNodeKind::Unreadable`] and never as an ordinary value; a text reference is
    /// reported by [`BuildText::Invalid`].
    pub(super) fn kind_of<'a>(
        &'a self,
        row: &'a NodeRow,
        text: &'a TextArena,
        table: &'a AttributeTable,
    ) -> ConstructionNodeKind<'a> {
        let units = |run: TextRun| {
            text.get(run.text())
                .map_or(BuildText::Invalid, BuildText::Units)
        };
        let leaf = || {
            usize::try_from(row.a)
                .ok()
                .and_then(|position| self.leaves.get(position))
        };
        match row.kind {
            RowKind::Document(mode) => ConstructionNodeKind::Document(mode),
            RowKind::Fragment => ConstructionNodeKind::Fragment,
            RowKind::Element => {
                let entry = usize::try_from(row.a)
                    .ok()
                    .and_then(|position| self.names.get(position));
                match entry {
                    Some(entry) => ConstructionNodeKind::Element(
                        ConstructionElementView::building(entry, row, table, text),
                    ),
                    None => ConstructionNodeKind::Unreadable(
                        HtmlConstructionCause::InvalidNameReference,
                    ),
                }
            }
            RowKind::Text | RowKind::Comment => match row.text_run() {
                Some(run) if row.kind == RowKind::Text => ConstructionNodeKind::Text(units(run)),
                Some(run) => ConstructionNodeKind::Comment(units(run)),
                None => ConstructionNodeKind::Unreadable(HtmlConstructionCause::UnknownNode),
            },
            RowKind::ProcessingInstruction => match leaf() {
                Some(Leaf::ProcessingInstruction(value)) => {
                    ConstructionNodeKind::ProcessingInstruction(
                        ConstructionProcessingInstruction::of(value),
                    )
                }
                _ => ConstructionNodeKind::Unreadable(HtmlConstructionCause::InvalidLeafReference),
            },
            RowKind::Doctype => match leaf() {
                Some(Leaf::Doctype(value)) => {
                    ConstructionNodeKind::Doctype(ConstructionDoctype::of(value))
                }
                _ => ConstructionNodeKind::Unreadable(HtmlConstructionCause::InvalidLeafReference),
            },
        }
    }
}
