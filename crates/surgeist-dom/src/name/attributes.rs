//! One immutable canonical attribute array with lease-free cached receipts.
use super::DomAttribute;
use crate::DomError;
use crate::store::{MaterializationUsage, payload, source::storage::Payload};
use std::{ops::Deref, sync::Arc};

pub(crate) struct AttributeValues {
    entries: Vec<DomAttribute>,
    logical_units: usize,
    text: Payload,
}
impl std::fmt::Debug for AttributeValues {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AttributeValues")
            .field("entries", &self.entries)
            .field("logical_units", &self.logical_units)
            .finish_non_exhaustive()
    }
}
impl AttributeValues {
    /// Fields arrive under the constructing owner's admitted temporary custody.
    /// Compute their receipt once; aliases and scalar edits read these caches.
    pub(crate) fn new(
        entries: Vec<DomAttribute>,
        mut step: impl FnMut() -> Result<(), DomError>,
    ) -> Result<Self, DomError> {
        let mut logical_units = 0usize;
        let mut text = Payload::default();
        for attribute in &entries {
            step()?;
            logical_units = logical_units
                .checked_add(
                    attribute
                        .logical_units()
                        .ok_or(DomError::RepresentationExhausted)?,
                )
                .ok_or(DomError::RepresentationExhausted)?;
            text.append(attribute.retained_payload()?)?;
        }
        Ok(Self {
            entries,
            logical_units,
            text,
        })
    }
    pub(crate) fn entries(&self) -> &[DomAttribute] {
        &self.entries
    }
    #[cfg(test)]
    pub(crate) fn capacity(&self) -> usize {
        self.entries.capacity()
    }
    pub(crate) fn logical_units(&self) -> usize {
        self.logical_units
    }
    pub(crate) fn text_payload(&self) -> Payload {
        self.text
    }
    /// Only this actual target and its one Vec buffer; text occurrences and
    /// NameRecord child controls are retained by their separate owners.
    pub(crate) fn shallow_usage(&self) -> Result<MaterializationUsage, DomError> {
        let bytes = self
            .entries
            .capacity()
            .checked_mul(size_of::<DomAttribute>())
            .and_then(|bytes| bytes.checked_add(size_of::<payload::Payload<Self>>()))
            .ok_or(DomError::RepresentationExhausted)?;
        Ok(MaterializationUsage::new(bytes, 0))
    }
}
impl PartialEq for AttributeValues {
    fn eq(&self, other: &Self) -> bool {
        self.entries == other.entries
    }
}
impl Eq for AttributeValues {}

/// Absence owns no allocation. A present target retains its actual capacity,
/// including a reserved empty array, without another wrapper allocation.
#[derive(Clone, Debug, Default)]
pub(crate) struct AttributeStorage(Option<Arc<payload::Payload<AttributeValues>>>);
impl AttributeStorage {
    pub(crate) fn from_payload(value: Arc<payload::Payload<AttributeValues>>) -> Self {
        Self(Some(value))
    }
    pub(crate) fn payload(&self) -> Option<&Arc<payload::Payload<AttributeValues>>> {
        self.0.as_ref()
    }
    pub(crate) fn entries(&self) -> &[DomAttribute] {
        self.0.as_ref().map_or(&[], |value| value.value.entries())
    }
    pub(crate) fn as_slice(&self) -> &[DomAttribute] {
        self.entries()
    }
    #[cfg(test)]
    pub(crate) fn capacity(&self) -> usize {
        self.0.as_ref().map_or(0, |value| value.value.capacity())
    }
    pub(crate) fn logical_units(&self) -> usize {
        self.0
            .as_ref()
            .map_or(0, |value| value.value.logical_units())
    }
    pub(crate) fn text_payload(&self) -> Payload {
        self.0
            .as_ref()
            .map_or_else(Payload::default, |value| value.value.text_payload())
    }
    pub(crate) fn own_usage(&self) -> MaterializationUsage {
        self.0
            .as_ref()
            .map_or(MaterializationUsage::ZERO, |value| value.own_usage)
    }
    pub(crate) fn same_storage(&self, other: &Self) -> bool {
        match (&self.0, &other.0) {
            (None, None) => true,
            (Some(left), Some(right)) => Arc::ptr_eq(left, right),
            _ => false,
        }
    }
    #[cfg(test)]
    pub(crate) fn from_entries(entries: Vec<DomAttribute>) -> Self {
        if entries.capacity() == 0 {
            return Self::default();
        }
        let value = AttributeValues::new(entries, || Ok(())).unwrap();
        let own_usage = value.shallow_usage().unwrap();
        Self::from_payload(Arc::new(payload::Payload {
            identity: crate::store::MaterializationId::fresh().unwrap(),
            own_usage,
            value,
        }))
    }
    #[cfg(test)]
    pub(crate) fn inspect_allocations(
        &self,
        graph: &mut crate::store::compact_tests::AllocationGraph,
    ) {
        if let Some(value) = &self.0 {
            graph.target(value, "attributes.target");
            graph.buffer(&value.value.entries, "attributes.buffer");
        }
    }
}
impl Deref for AttributeStorage {
    type Target = [DomAttribute];
    fn deref(&self) -> &Self::Target {
        self.entries()
    }
}
impl PartialEq for AttributeStorage {
    fn eq(&self, other: &Self) -> bool {
        self.same_storage(other) || self.entries() == other.entries()
    }
}
impl Eq for AttributeStorage {}
