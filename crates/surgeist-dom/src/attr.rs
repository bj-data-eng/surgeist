use crate::{
    DocumentId, DocumentString, DomAttrId, DomAttribute, DomError, DomNode, DomQualifiedName,
    Namespace, NodeId,
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

    /// Visit attributes in their stored order with metered canonical lookups.
    pub fn visit_with<E: From<DomError>>(
        self,
        mut step: impl FnMut() -> Result<(), E>,
        mut visit: impl FnMut(DomAttr<'a>) -> Result<(), E>,
    ) -> Result<(), E> {
        for &id in self.ids() {
            step()?;
            let node = self.element.snapshot.node_with(id, &mut step)?;
            let attribute = node.as_attr().ok_or(DomError::WrongNodeKind)?;
            visit(attribute)?;
        }
        Ok(())
    }

    pub fn get_named_item(self, qualified_name: &DocumentString) -> Option<DomAttr<'a>> {
        self.get_named_item_with(qualified_name, || Ok::<_, DomError>(()))
            .ok()
            .flatten()
    }

    pub fn get_named_item_ns(
        self,
        namespace: &Namespace,
        local_name: &DocumentString,
    ) -> Option<DomAttr<'a>> {
        self.get_named_item_ns_with(namespace, local_name, || Ok::<_, DomError>(()))
            .ok()
            .flatten()
    }

    /// Read the first matching qualified spelling in attribute order. For an
    /// HTML element in an HTML document, only the input is ASCII-lowercased.
    /// Admit each visited lookup entry and page and each compared code unit.
    pub fn get_named_item_with<E: From<DomError>>(
        self,
        qualified_name: &DocumentString,
        mut step: impl FnMut() -> Result<(), E>,
    ) -> Result<Option<DomAttr<'a>>, E> {
        let fold = self.element.html_element_in_html_document_with(&mut step)?;
        self.find_with(step, |attribute, step| {
            crate::names::qualified_equals_with(attribute.name(), qualified_name, fold, step)
        })
    }

    /// Read the first exact namespace and local name in attribute order.
    /// Admit each visited lookup entry and page and each compared code unit.
    pub fn get_named_item_ns_with<E: From<DomError>>(
        self,
        namespace: &Namespace,
        local_name: &DocumentString,
        step: impl FnMut() -> Result<(), E>,
    ) -> Result<Option<DomAttr<'a>>, E> {
        self.find_with(step, |attribute, step| {
            crate::names::expanded_equals_with(attribute.name(), namespace.uri(), local_name, step)
        })
    }

    /// Read the first exact local name in the null namespace without allocating
    /// a query string. Admit each visited lookup and compared code unit.
    pub fn get_null_namespace_attribute_with<E: From<DomError>>(
        self,
        local_name: &[u16],
        step: impl FnMut() -> Result<(), E>,
    ) -> Result<Option<DomAttr<'a>>, E> {
        self.find_with(step, |attribute, step| {
            let stored = attribute.name();
            if stored.namespace().is_some() {
                return Ok(false);
            }
            crate::names::same_code_units(stored.local_name().code_units(), local_name, step)
        })
    }

    pub(crate) fn find_with<E: From<DomError>, Step: FnMut() -> Result<(), E>>(
        self,
        mut step: Step,
        mut matches: impl FnMut(DomAttr<'a>, &mut Step) -> Result<bool, E>,
    ) -> Result<Option<DomAttr<'a>>, E> {
        for &id in self.ids() {
            let attribute = self
                .element
                .snapshot
                .node_with(id, &mut step)?
                .as_attr()
                .ok_or(DomError::WrongNodeKind)?;
            if matches(attribute, &mut step)? {
                return Ok(Some(attribute));
            }
        }
        Ok(None)
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
