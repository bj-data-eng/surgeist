//! Borrowed semantic operands, independent of their owning storage.

use crate::{
    internal::values::Attribute, internal::values::DocumentString, internal::values::Namespace,
    internal::values::OrderedAttributes, internal::values::QualifiedName,
};

/// Exact qualified-name components supplied to HTML language rules.
///
/// Getters borrow immutable text and perform no normalization. Namespace
/// absence is represented by `None`, including an originally empty URI.
/// Consumers request only the components selected by the rule, after any
/// applicable read admission; implementations must not eagerly read others.
pub trait HtmlNameInput {
    /// The namespace URI, or `None` for no namespace (including an originally empty URI).
    fn namespace_uri(&self) -> Option<&DocumentString>;
    /// The namespace prefix, or `None`. A prefix never changes the expanded name's identity.
    fn prefix(&self) -> Option<&DocumentString>;
    /// The local name, with its exact UTF-16 code units.
    fn local_name(&self) -> &DocumentString;
}

/// One exact attribute, without interpreting its value or name.
pub trait HtmlAttributeInput {
    /// The attribute's name type.
    type Name: HtmlNameInput + ?Sized;
    /// The attribute's qualified name components.
    fn name(&self) -> &Self::Name;
    /// The attribute's exact value, unnormalized.
    fn value(&self) -> &DocumentString;
}

impl<T: HtmlAttributeInput + ?Sized> HtmlAttributeInput for &T {
    type Name = T::Name;
    fn name(&self) -> &Self::Name {
        (**self).name()
    }
    fn value(&self) -> &DocumentString {
        (**self).value()
    }
}

/// Borrowed attributes in their original producer order.
///
/// `get` returns, by value, a view of the unchanged attribute at every index
/// below `len`. A view's `name` and `value` borrow from the collection or from
/// a field the view holds; they are never a copy of the code units, and a
/// collection that stores whole attributes lends them without cloning. A view
/// need not be `Clone` or `Copy`. Because `value` borrows from the view, a
/// caller that needs a value after dropping the view keeps the view.
/// This boundary neither proves nor repeats validation: callers retain their
/// original construction or capture qualification, including expanded-name
/// uniqueness. It does not deduplicate, sort or copy the collection.
pub trait HtmlAttributesInput {
    /// The view of one attribute, which borrows from the collection.
    type Attribute<'a>: HtmlAttributeInput
    where
        Self: 'a;
    /// The number of attributes, which bounds the valid indices for `get`.
    fn len(&self) -> usize;
    /// The attribute at a zero-based index in producer order; `None` for an index at or past `len`.
    fn get(&self, index: usize) -> Option<Self::Attribute<'_>>;
    /// Whether the collection has no attributes (`len() == 0`).
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
    type Attribute<'a>
        = &'a A
    where
        A: 'a;
    fn len(&self) -> usize {
        self.len()
    }
    fn get(&self, index: usize) -> Option<&A> {
        self.get(index)
    }
}

impl HtmlAttributesInput for OrderedAttributes {
    type Attribute<'a> = &'a Attribute;
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

/// The value of a view that a caller keeps in a local, borrowed from that local.
pub(crate) fn view_value<A: HtmlAttributeInput>(view: &Option<A>) -> Option<&DocumentString> {
    view.as_ref().map(HtmlAttributeInput::value)
}

pub(crate) fn attributes_iter<A: HtmlAttributesInput + ?Sized>(
    attributes: &A,
) -> impl Iterator<Item = A::Attribute<'_>> {
    (0..attributes.len()).filter_map(|index| attributes.get(index))
}

pub(crate) fn find_attribute<'a, A: HtmlAttributesInput + ?Sized>(
    attributes: &'a A,
    namespace: &Namespace,
    local: &DocumentString,
) -> Option<A::Attribute<'a>> {
    attributes_iter(attributes).find(|attribute| {
        let name = attribute.name();
        name.namespace_uri() == namespace.uri() && name.local_name() == local
    })
}
