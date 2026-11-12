//! Output-only node views over the construction records.
//!
//! The published view, [`HtmlNodeKind`], carries shared strings and is returned by value from
//! [`HtmlNode::kind`]. The build view, [`ConstructionNodeKind`], carries borrowed code units and is
//! returned by value from `HtmlConstructionNode::kind`. Only this crate constructs
//! [`HtmlElementView`], [`HtmlAttributes`] and [`BuildStr`], and none of the views exposes how the
//! record stores its content.

use super::attribute_table::{AttrRow, PublishedAttr};
use super::*;
use crate::internal::elements::rel::HtmlLinkFlags;
use crate::internal::values::{Attribute, Namespace};
use std::{
    iter::{Skip, Take},
    slice,
};

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
}

/// A borrowed element: its exact name, ordered attributes and link history.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlElementView<'a> {
    content: &'a ElementContent<DocumentString>,
}
impl<'a> HtmlElementView<'a> {
    /// Exact qualified name.
    pub fn name(self) -> &'a QualifiedName {
        self.content.name()
    }
    /// The key of the element's name, which the tests compare with the vocabulary.
    #[cfg(test)]
    pub(crate) fn key(self) -> crate::internal::values::element_key::ElementKey {
        self.content.key()
    }
    /// The element's attributes in producer order.
    pub fn attributes(self) -> HtmlAttributes<'a> {
        HtmlAttributes {
            slice: self.content.attrs.as_ref(),
        }
    }
    /// Actual construction flags. Records of a result that kept no link history return `None`.
    pub fn link_flags(self) -> Option<HtmlLinkFlags> {
        self.content.link_flags
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

/// The attributes of an element that has none: the rows of no slice.
fn no_rows() -> Take<Skip<slice::Iter<'static, PublishedAttr>>> {
    const NONE: &[PublishedAttr] = &[];
    NONE.iter().skip(1).take(0)
}

/// The iterator of the attributes of a published element.
struct PublishedAttributes<'a>(Take<Skip<slice::Iter<'a, PublishedAttr>>>);
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
    slice: Option<&'a RowSlice>,
}
impl<'a> HtmlAttributes<'a> {
    /// The number of attributes.
    pub fn len(self) -> usize {
        self.slice.map_or(0, RowSlice::len)
    }
    /// Whether there are no attributes.
    pub fn is_empty(self) -> bool {
        self.len() == 0
    }
    /// The attribute at a zero-based index in producer order; `None` at or past `len`.
    pub fn get(self, index: usize) -> Option<HtmlAttribute<'a>> {
        self.slice?.get(index).map(HtmlAttribute::of)
    }
    /// Iterates in producer order.
    pub fn iter(
        self,
    ) -> impl ExactSizeIterator<Item = HtmlAttribute<'a>> + DoubleEndedIterator + 'a {
        PublishedAttributes(match self.slice {
            Some(slice) => slice.iter(),
            None => no_rows(),
        })
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
    Published(Option<&'a RowSlice>),
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
            AttrSource::Published(slice) => Ok(slice.map_or(0, RowSlice::len)),
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
            AttrSource::Published(slice) => Ok(slice
                .and_then(|slice| slice.get(index))
                .map(BuildAttribute::published)),
            AttrSource::Invalid => Err(HtmlConstructionCause::InvalidAttributeReference.error()),
        }
    }
    /// Iterates in producer order, with the refusal of [`Self::len`].
    pub fn iter(self) -> Result<BuildAttributeIter<'a>, HtmlConstructionError> {
        Ok(BuildAttributeIter(match self.source {
            AttrSource::Rows { rows, text } => IterSource::Rows(rows.iter(), text),
            AttrSource::Published(Some(slice)) => IterSource::Published(slice.iter()),
            AttrSource::Published(None) => IterSource::Published(no_rows()),
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
    Published(Take<Skip<slice::Iter<'a, PublishedAttr>>>),
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
        content: &'a ElementContent<TextRun>,
        table: &'a AttributeTable,
        text: &'a TextArena,
    ) -> Self {
        Self {
            name: content.name(),
            #[cfg(test)]
            key: content.key(),
            link_flags: content.link_flags,
            attributes: BuildAttributes {
                source: match table.live(content.attrs) {
                    Ok(rows) => AttrSource::Rows { rows, text },
                    Err(_) => AttrSource::Invalid,
                },
            },
        }
    }
    /// The view of an element record of a published result.
    fn published(content: &'a ElementContent<DocumentString>) -> Self {
        Self {
            name: content.name(),
            #[cfg(test)]
            key: content.key(),
            link_flags: content.link_flags,
            attributes: BuildAttributes {
                source: AttrSource::Published(content.attrs.as_ref()),
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

impl<T: Cell> NodeContent<T> {
    /// Whether a record of this content can hold ordinary children.
    pub(super) fn is_container(&self) -> bool {
        matches!(self, Self::Document(_) | Self::Fragment | Self::Element(_))
    }
    /// The build view of this content, with the text of a text or comment record read by `text`
    /// and the view of an element read by `element`.
    fn viewed<'a>(
        &'a self,
        text: impl Fn(&'a T) -> BuildText<'a>,
        element: impl Fn(&'a ElementContent<T>) -> ConstructionElementView<'a>,
    ) -> ConstructionNodeKind<'a> {
        fn units(value: &DocumentString) -> BuildStr<'_> {
            BuildStr::new(value.code_units())
        }
        match self {
            Self::Document(mode) => ConstructionNodeKind::Document(*mode),
            Self::Fragment => ConstructionNodeKind::Fragment,
            Self::Element(content) => ConstructionNodeKind::Element(element(content)),
            Self::Text(value) => ConstructionNodeKind::Text(text(value)),
            Self::Comment(value) => ConstructionNodeKind::Comment(text(value)),
            Self::ProcessingInstruction(value) => {
                ConstructionNodeKind::ProcessingInstruction(ConstructionProcessingInstruction {
                    target: units(value.target()),
                    data: units(value.data()),
                })
            }
            Self::Doctype(value) => ConstructionNodeKind::Doctype(ConstructionDoctype {
                name: units(value.name()),
                public_id: value.public_id().map(units),
                system_id: value.system_id().map(units),
            }),
        }
    }
}

impl NodeContent<DocumentString> {
    /// The published view of this content.
    pub(super) fn published(&self) -> HtmlNodeKind<'_> {
        match self {
            Self::Document(mode) => HtmlNodeKind::Document(*mode),
            Self::Fragment => HtmlNodeKind::Fragment,
            Self::Element(content) => HtmlNodeKind::Element(HtmlElementView { content }),
            Self::Text(value) => HtmlNodeKind::Text(value.clone()),
            Self::Comment(value) => HtmlNodeKind::Comment(value.clone()),
            Self::ProcessingInstruction(value) => {
                HtmlNodeKind::ProcessingInstruction(value.clone())
            }
            Self::Doctype(value) => HtmlNodeKind::Doctype(value.clone()),
        }
    }
    /// The build view of this published content, which reads each text as its borrowed units.
    pub(super) fn building_published(&self) -> ConstructionNodeKind<'_> {
        self.viewed(
            |value| BuildText::Units(value.code_units()),
            ConstructionElementView::published,
        )
    }
}

impl NodeContent<TextRun> {
    /// The build view of this content, whose text and comment units and attribute values are
    /// read from `text`, and whose attributes are the rows of `table`.
    pub(super) fn building<'a>(
        &'a self,
        text: &'a TextArena,
        table: &'a AttributeTable,
    ) -> ConstructionNodeKind<'a> {
        self.viewed(
            |run| {
                text.get(run.text())
                    .map_or(BuildText::Invalid, BuildText::Units)
            },
            |content| ConstructionElementView::building(content, table, text),
        )
    }
}
