//! Canonical names retain exact text and owner-qualified immutable identity.
use crate::store::{MaterializationId, MaterializationUsage, source::storage::Payload};
use crate::{DocumentString, DomError, DomOwnerId};
use std::{
    hash::{Hash, Hasher},
    sync::Arc,
};
mod attributes;
pub(crate) use attributes::{AttributeStorage, AttributeValues};

/// Lease-free target shared by canonical atoms and their external holders.
/// Its private materialization identity routes custody, never public matching.
#[derive(Debug)]
pub(crate) struct NameRecord {
    owner: DomOwnerId,
    text: DocumentString,
    identity: MaterializationId,
    // Private owner-index locator. Every live access also qualifies the owner
    // and materialization identity; this is inert after the owner is dropped.
    row: usize,
}
impl NameRecord {
    pub(crate) fn new(
        owner: DomOwnerId,
        text: DocumentString,
        identity: MaterializationId,
        row: usize,
    ) -> Self {
        Self {
            owner,
            text,
            identity,
            row,
        }
    }
    pub(crate) fn owner(&self) -> DomOwnerId {
        self.owner
    }
    pub(crate) fn text(&self) -> &DocumentString {
        &self.text
    }
    pub(crate) fn identity(&self) -> MaterializationId {
        self.identity
    }
    pub(crate) fn row(&self) -> usize {
        self.row
    }
    pub(crate) fn usage(text: &DocumentString) -> Result<MaterializationUsage, DomError> {
        let capacity = text.shared_code_units().capacity();
        let bytes = capacity
            .checked_mul(size_of::<u16>())
            .and_then(|bytes| bytes.checked_add(size_of::<Vec<u16>>()))
            .and_then(|bytes| bytes.checked_add(size_of::<Self>()))
            .ok_or(DomError::RepresentationExhausted)?;
        Ok(MaterializationUsage::new(bytes, capacity))
    }
}

/// Strong opaque name identity in one DOM owner, retaining only its own text.
/// Equality and hashing compare live identity; use `text` for value comparison.
#[derive(Clone)]
pub struct DomNameAtom(Arc<NameRecord>);
impl DomNameAtom {
    pub(crate) fn from_record(record: Arc<NameRecord>) -> Self {
        Self(record)
    }
    pub(crate) fn record(&self) -> &Arc<NameRecord> {
        &self.0
    }
    pub(crate) fn identity(&self) -> MaterializationId {
        self.0.identity()
    }
    pub fn owner(&self) -> DomOwnerId {
        self.0.owner()
    }
    pub fn text(&self) -> &DocumentString {
        self.0.text()
    }
    /// Compare identity only after qualifying both tokens to the same owner.
    pub fn same_identity(&self, other: &Self) -> Result<bool, DomError> {
        if self.owner() != other.owner() {
            return Err(DomError::ForeignOwner);
        }
        Ok(self == other)
    }
    fn same_text(&self, other: &Self) -> bool {
        self == other || self.text() == other.text()
    }
    #[cfg(test)]
    fn inspect_allocations(&self, graph: &mut crate::store::compact_tests::AllocationGraph) {
        graph.target(&self.0, "name.record");
        graph.string(self.text());
    }
}
impl std::fmt::Debug for DomNameAtom {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DomNameAtom")
            .field("owner", &self.owner())
            .field("text", self.text())
            .finish()
    }
}
impl PartialEq for DomNameAtom {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}
impl Eq for DomNameAtom {}
impl Hash for DomNameAtom {
    fn hash<H: Hasher>(&self, state: &mut H) {
        Arc::as_ptr(&self.0).hash(state);
    }
}

/// Canonical expanded name with separately preserved qualified-spelling prefix.
/// Semantic equality compares exact values, including across different owners.
#[derive(Clone, Debug)]
pub struct DomQualifiedName {
    namespace: Option<DomNameAtom>,
    prefix: Option<DocumentString>,
    local: DomNameAtom,
}
impl DomQualifiedName {
    /// The ingress owner has already validated and case-processed components.
    pub(crate) fn from_atoms(
        namespace: Option<DomNameAtom>,
        prefix: Option<DocumentString>,
        local: DomNameAtom,
    ) -> Result<Self, DomError> {
        if namespace
            .as_ref()
            .is_some_and(|namespace| namespace.owner() != local.owner())
        {
            return Err(DomError::ForeignOwner);
        }
        Ok(Self {
            namespace: namespace.filter(|namespace| !namespace.text().is_empty()),
            prefix,
            local,
        })
    }
    pub fn owner(&self) -> DomOwnerId {
        self.local.owner()
    }
    pub fn namespace(&self) -> Option<&DomNameAtom> {
        self.namespace.as_ref()
    }
    pub fn namespace_uri(&self) -> Option<&DocumentString> {
        self.namespace.as_ref().map(DomNameAtom::text)
    }
    pub fn prefix(&self) -> Option<&DocumentString> {
        self.prefix.as_ref()
    }
    pub fn local_name(&self) -> &DocumentString {
        self.local.text()
    }
    pub fn local_name_atom(&self) -> &DomNameAtom {
        &self.local
    }
    /// Compare namespace/local identity in the same owner; prefixes do not
    /// participate in expanded-name identity.
    pub fn same_expanded_identity(&self, other: &Self) -> Result<bool, DomError> {
        if self.owner() != other.owner() {
            return Err(DomError::ForeignOwner);
        }
        Ok(self.namespace == other.namespace && self.local == other.local)
    }
    pub(crate) fn logical_units(&self) -> Option<usize> {
        self.local_name()
            .len()
            .checked_add(self.namespace_uri().map_or(0, DocumentString::len))?
            .checked_add(self.prefix().map_or(0, DocumentString::len))
    }
    /// Conservative text occurrences only. Record targets have their separate
    /// registry/containing-object custody and are not charged again here.
    pub(crate) fn retained_payload(&self) -> Result<Payload, DomError> {
        let mut payload = Payload::default();
        for text in self
            .namespace_uri()
            .into_iter()
            .chain(self.prefix())
            .chain([self.local_name()])
        {
            payload.string(text)?;
        }
        Ok(payload)
    }
    #[cfg(test)]
    pub(crate) fn inspect_allocations(
        &self,
        graph: &mut crate::store::compact_tests::AllocationGraph,
    ) {
        if let Some(namespace) = self.namespace() {
            namespace.inspect_allocations(graph);
        }
        if let Some(prefix) = self.prefix() {
            graph.string(prefix);
        }
        self.local.inspect_allocations(graph);
    }
}
impl PartialEq for DomQualifiedName {
    fn eq(&self, other: &Self) -> bool {
        let namespace = match (&self.namespace, &other.namespace) {
            (None, None) => true,
            (Some(left), Some(right)) => left.same_text(right),
            _ => false,
        };
        namespace && self.prefix == other.prefix && self.local.same_text(&other.local)
    }
}
impl Eq for DomQualifiedName {}
impl Hash for DomQualifiedName {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.namespace_uri().hash(state);
        self.prefix.hash(state);
        self.local_name().hash(state);
    }
}
impl surgeist_html::HtmlNameInput for DomQualifiedName {
    fn namespace_uri(&self) -> Option<&DocumentString> {
        self.namespace_uri()
    }
    fn prefix(&self) -> Option<&DocumentString> {
        self.prefix()
    }
    fn local_name(&self) -> &DocumentString {
        self.local_name()
    }
}

/// One canonical ordered attribute record; its value is never a mandatory atom.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct DomAttribute {
    name: DomQualifiedName,
    value: DocumentString,
}
impl DomAttribute {
    pub(crate) fn from_parts(name: DomQualifiedName, value: DocumentString) -> Self {
        Self { name, value }
    }
    pub fn name(&self) -> &DomQualifiedName {
        &self.name
    }
    pub fn value(&self) -> &DocumentString {
        &self.value
    }
    pub(crate) fn logical_units(&self) -> Option<usize> {
        self.name.logical_units()?.checked_add(self.value.len())
    }
    pub(crate) fn retained_payload(&self) -> Result<Payload, DomError> {
        let mut payload = self.name.retained_payload()?;
        payload.string(&self.value)?;
        Ok(payload)
    }
}
impl surgeist_html::HtmlAttributeInput for DomAttribute {
    type Name = DomQualifiedName;
    fn name(&self) -> &Self::Name {
        self.name()
    }
    fn value(&self) -> &DocumentString {
        self.value()
    }
}

#[cfg(test)]
mod tests;
