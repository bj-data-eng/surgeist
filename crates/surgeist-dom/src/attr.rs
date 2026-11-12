use crate::{DocumentId, DocumentString, DomAttrId, DomAttribute, DomQualifiedName, NodeId};

/// The exact Attr identity and value in an immutable DOM snapshot.
#[derive(Clone, Copy, Debug)]
pub struct DomAttr<'a> {
    pub(crate) id: DomAttrId,
    pub(crate) document: DocumentId,
    pub(crate) owner: Option<NodeId>,
    pub(crate) data: &'a DomAttribute,
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
