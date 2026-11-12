use crate::{
    DocumentId, DocumentString, DomAttrId, DomAttribute, DomNode, DomQualifiedName, Namespace,
    NodeId,
};

/// Result of a canonical native Attr attachment.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DomAttributeAttachment {
    /// The incoming Attr was appended.
    Appended(DomAttrId),
    /// This Attr already occupied the selected position.
    Unchanged(DomAttrId),
    /// The previous Attr was detached and the incoming Attr took its position.
    Replaced(DomAttrId),
}

/// The exact Attr identity and value in an immutable DOM snapshot.
#[derive(Clone, Copy, Debug)]
pub struct DomAttr<'a> {
    pub(crate) id: DomAttrId,
    pub(crate) document: DocumentId,
    pub(crate) owner: Option<NodeId>,
    pub(crate) data: &'a DomAttribute,
}

impl surgeist_html::values::HtmlAttributeInput for DomAttr<'_> {
    type Name = DomQualifiedName;

    fn name(&self) -> &Self::Name {
        self.data.name()
    }

    fn value(&self) -> &DocumentString {
        self.data.value()
    }
}

/// The ordered attributes of one Element in an immutable DOM snapshot.
#[derive(Clone, Copy, Debug)]
pub struct DomNamedNodeMap<'a> {
    pub(crate) element: DomNode<'a>,
}

impl<'a> DomNamedNodeMap<'a> {
    fn ids(self) -> &'a [NodeId] {
        self.element.node.attribute_ids()
    }

    pub fn len(self) -> usize {
        self.ids().len()
    }

    pub fn is_empty(self) -> bool {
        self.ids().is_empty()
    }

    pub fn item(self, index: usize) -> Option<DomAttr<'a>> {
        self.element.snapshot.attr(*self.ids().get(index)?).ok()
    }

    pub fn iter(self) -> impl Iterator<Item = DomAttr<'a>> + 'a {
        self.ids()
            .iter()
            .filter_map(move |id| self.element.snapshot.attr(*id).ok())
    }

    pub fn get_named_item(self, qualified_name: &DocumentString) -> Option<DomAttr<'a>> {
        let fold = self.element.html_element_in_html_document();
        self.iter().find(|attribute| {
            if fold {
                crate::names::qualified_equals_lowercase_input(attribute.name(), qualified_name)
            } else {
                crate::names::qualified_equals(attribute.name(), qualified_name)
            }
        })
    }

    pub fn get_named_item_ns(
        self,
        namespace: &Namespace,
        local_name: &DocumentString,
    ) -> Option<DomAttr<'a>> {
        self.iter().find(|attribute| {
            attribute.name().namespace_uri() == namespace.uri()
                && attribute.name().local_name() == local_name
        })
    }
}

impl surgeist_html::values::HtmlAttributesInput for DomNamedNodeMap<'_> {
    type Attribute<'a>
        = DomAttr<'a>
    where
        Self: 'a;

    fn len(&self) -> usize {
        self.ids().len()
    }

    fn get(&self, index: usize) -> Option<Self::Attribute<'_>> {
        self.item(index)
    }
}

impl<'a> DomAttr<'a> {
    pub fn id(self) -> DomAttrId {
        self.id
    }

    pub fn owner_document(self) -> DocumentId {
        self.document
    }

    pub fn owner_element(self) -> Option<NodeId> {
        self.owner
    }

    pub fn name(self) -> &'a DomQualifiedName {
        self.data.name()
    }

    pub fn value(self) -> &'a DocumentString {
        self.data.value()
    }
}
