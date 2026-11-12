//! Output-only node views over the construction records.
//!
//! The published view, [`HtmlNodeKind`], carries shared strings and is returned by value from
//! [`HtmlNode::kind`]. The build view, [`ConstructionNodeKind`], carries borrowed code units and is
//! returned by value from `HtmlConstructionNode::kind`. Only this crate constructs
//! [`HtmlElementView`], [`HtmlAttributes`] and [`BuildStr`], and none of the views exposes how the
//! record stores its content.

use super::*;
use crate::internal::elements::rel::HtmlLinkFlags;
use crate::internal::values::{Attribute, Namespace};

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
    content: &'a ElementContent,
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
            attributes: self.content.attributes(),
        }
    }
    /// Actual construction flags. Records of a result that kept no link history return `None`.
    pub fn link_flags(self) -> Option<HtmlLinkFlags> {
        self.content.link_flags
    }
}

/// The ordered attributes of one element, borrowed from the result or storage that holds it.
///
/// Attributes keep their producer order and are unique by namespace and local name.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlAttributes<'a> {
    attributes: &'a OrderedAttributes,
}
impl<'a> HtmlAttributes<'a> {
    /// The number of attributes.
    pub fn len(self) -> usize {
        self.attributes.len()
    }
    /// Whether there are no attributes.
    pub fn is_empty(self) -> bool {
        self.attributes.is_empty()
    }
    /// The attribute at a zero-based index in producer order; `None` at or past `len`.
    pub fn get(self, index: usize) -> Option<&'a Attribute> {
        self.attributes.as_slice().get(index)
    }
    /// Iterates in producer order.
    pub fn iter(self) -> impl ExactSizeIterator<Item = &'a Attribute> + DoubleEndedIterator + 'a {
        self.attributes.as_slice().iter()
    }
    /// Finds an attribute by exact namespace and local-name identity.
    pub fn find(self, namespace: &Namespace, local_name: &DocumentString) -> Option<&'a Attribute> {
        self.attributes.get(namespace, local_name)
    }
    /// The attributes as an owned ordered collection, in producer order.
    pub fn to_ordered(self) -> OrderedAttributes {
        self.attributes.clone()
    }
}
impl crate::internal::values::semantic_input::HtmlAttributesInput for HtmlAttributes<'_> {
    type Attribute<'a>
        = &'a Attribute
    where
        Self: 'a;
    fn len(&self) -> usize {
        self.attributes.len()
    }
    fn get(&self, index: usize) -> Option<&Attribute> {
        self.attributes.as_slice().get(index)
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
    Element(HtmlElementView<'a>),
    /// A text node.
    Text(BuildText<'a>),
    /// A comment node.
    Comment(BuildText<'a>),
    /// A processing instruction recorded with its exact target and data.
    ProcessingInstruction(ConstructionProcessingInstruction<'a>),
    /// A document type declaration.
    Doctype(ConstructionDoctype<'a>),
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

impl<T> NodeContent<T> {
    /// Whether a record of this content can hold ordinary children.
    pub(super) fn is_container(&self) -> bool {
        matches!(self, Self::Document(_) | Self::Fragment | Self::Element(_))
    }
    /// The build view of this content, with the text of a text or comment record read by `text`.
    fn viewed<'a>(&'a self, text: impl Fn(&'a T) -> BuildText<'a>) -> ConstructionNodeKind<'a> {
        fn units(value: &DocumentString) -> BuildStr<'_> {
            BuildStr::new(value.code_units())
        }
        match self {
            Self::Document(mode) => ConstructionNodeKind::Document(*mode),
            Self::Fragment => ConstructionNodeKind::Fragment,
            Self::Element(content) => ConstructionNodeKind::Element(HtmlElementView { content }),
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
        self.viewed(|value| BuildText::Units(value.code_units()))
    }
}

impl NodeContent<TextRun> {
    /// The published content of this content: its text and comment units become strings of
    /// `block`. With `cursor` the units are those of a block packed in record order, whose next
    /// text begins at `*cursor`; without it they are the ranges the records name. Nothing of this
    /// content is moved.
    pub(super) fn published_with(
        &self,
        block: &TextBlock,
        cursor: &mut Option<usize>,
    ) -> Result<PublishedContent, HtmlConstructionError> {
        let mut string = |run: &TextRun| -> Result<DocumentString, HtmlConstructionError> {
            let range = match cursor {
                Some(next) => {
                    let start = *next;
                    let end = start
                        .checked_add(run.len())
                        .ok_or(HtmlConstructionCause::RepresentationExhausted.error())?;
                    *next = end;
                    start..end
                }
                None => run.text().range(),
            };
            #[cfg(test)]
            if crate::internal::amortized::fault::refuses_text_share() {
                return Err(HtmlConstructionCause::InvalidTextReference.error());
            }
            block
                .string(range)
                .map_err(|_| HtmlConstructionCause::InvalidTextReference.error())
        };
        Ok(match self {
            Self::Document(mode) => NodeContent::Document(*mode),
            Self::Fragment => NodeContent::Fragment,
            Self::Element(content) => NodeContent::Element(content.clone()),
            Self::Text(run) => NodeContent::Text(string(run)?),
            Self::Comment(run) => NodeContent::Comment(string(run)?),
            Self::ProcessingInstruction(value) => NodeContent::ProcessingInstruction(value.clone()),
            Self::Doctype(value) => NodeContent::Doctype(value.clone()),
        })
    }
    /// The build view of this content, whose text and comment units are read from `arena`.
    pub(super) fn building<'a>(&'a self, arena: &'a TextArena) -> ConstructionNodeKind<'a> {
        self.viewed(|run| {
            arena
                .get(run.text())
                .map_or(BuildText::Invalid, BuildText::Units)
        })
    }
}
