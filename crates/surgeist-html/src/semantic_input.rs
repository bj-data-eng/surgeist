//! Borrowed semantic operands, independent of their owning storage.

use crate::{Attribute, DocumentString, Namespace, OrderedAttributes, QualifiedName};

/// Exact qualified-name components supplied to HTML language rules.
///
/// Getters borrow immutable text and perform no normalization. Namespace
/// absence is represented by `None`, including an originally empty URI.
/// Consumers request only the components selected by the rule, after any
/// applicable read admission; implementations must not eagerly read others.
pub trait HtmlNameInput {
    fn namespace_uri(&self) -> Option<&DocumentString>;
    fn prefix(&self) -> Option<&DocumentString>;
    fn local_name(&self) -> &DocumentString;
}

/// One exact attribute, without interpreting its value or name.
pub trait HtmlAttributeInput {
    type Name: HtmlNameInput + ?Sized;
    fn name(&self) -> &Self::Name;
    fn value(&self) -> &DocumentString;
}

/// Borrowed attributes in their original producer order.
///
/// `get` returns the unchanged attribute at every index below `len`. This
/// boundary neither proves nor repeats validation: callers retain their
/// original construction or capture qualification, including expanded-name
/// uniqueness. It does not deduplicate, sort or copy the collection.
pub trait HtmlAttributesInput {
    type Attribute: HtmlAttributeInput;
    fn len(&self) -> usize;
    fn get(&self, index: usize) -> Option<&Self::Attribute>;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl HtmlNameInput for QualifiedName {
    fn namespace_uri(&self) -> Option<&DocumentString> {
        self.namespace().uri()
    }
    fn prefix(&self) -> Option<&DocumentString> {
        self.prefix()
    }
    fn local_name(&self) -> &DocumentString {
        self.local_name()
    }
}

impl HtmlAttributeInput for Attribute {
    type Name = QualifiedName;
    fn name(&self) -> &Self::Name {
        self.name()
    }
    fn value(&self) -> &DocumentString {
        self.value()
    }
}

impl<A: HtmlAttributeInput> HtmlAttributesInput for [A] {
    type Attribute = A;
    fn len(&self) -> usize {
        self.len()
    }
    fn get(&self, index: usize) -> Option<&A> {
        self.get(index)
    }
}

impl HtmlAttributesInput for OrderedAttributes {
    type Attribute = Attribute;
    fn len(&self) -> usize {
        self.len()
    }
    fn get(&self, index: usize) -> Option<&Attribute> {
        self.as_slice().get(index)
    }
}

// Retained individual report evidence shares the supplied immutable text.
// These are not bank exports and allocate no array or UTF-16 backing.
pub(crate) fn capture_name(name: &(impl HtmlNameInput + ?Sized)) -> QualifiedName {
    let namespace = name
        .namespace_uri()
        .cloned()
        .map_or_else(Namespace::none, Namespace::from_uri);
    QualifiedName::new(namespace, name.prefix().cloned(), name.local_name().clone())
}

pub(crate) fn capture_attribute(raw: &(impl HtmlAttributeInput + ?Sized)) -> Attribute {
    Attribute::new(capture_name(raw.name()), raw.value().clone())
}

pub(crate) fn attributes_iter<A: HtmlAttributesInput + ?Sized>(
    attributes: &A,
) -> impl Iterator<Item = &A::Attribute> {
    (0..attributes.len()).filter_map(|index| attributes.get(index))
}

pub(crate) fn find_attribute<'a, A: HtmlAttributesInput + ?Sized>(
    attributes: &'a A,
    namespace: &Namespace,
    local: &DocumentString,
) -> Option<&'a A::Attribute> {
    attributes_iter(attributes).find(|attribute| {
        let name = attribute.name();
        name.namespace_uri() == namespace.uri() && name.local_name() == local
    })
}
