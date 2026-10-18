//! Form scalar readback and the canonical identity-bearing past names map.
use crate::{DocumentString, DomNode, NodeId};
#[cfg(test)]
pub(crate) mod allocation_observation;

/// Checked native Form view at one immutable canonical revision.
#[derive(Clone, Copy, Debug)]
pub struct DomForm<'a> {
    pub(crate) node: DomNode<'a>,
}
impl<'a> DomForm<'a> {
    pub fn id(self) -> NodeId {
        self.node.id()
    }
    pub fn state(self) -> surgeist_html::HtmlFormNativeState {
        self.node
            .as_element()
            .expect("checked Form")
            .form_state()
            .expect("checked Form")
    }
    /// Retained exact-name aliases. Native creation/clone starts empty; this
    /// surface does not perform named lookup or install past-name history.
    pub fn past_names(self) -> impl ExactSizeIterator<Item = (&'a DocumentString, NodeId)> {
        self.node
            .node
            .form_past_names
            .as_ref()
            .expect("checked Form")
            .entries()
            .iter()
            .map(|entry| (&entry.name, entry.target))
    }
}
#[derive(Debug)]
pub(crate) struct FormPastNames<K>(
    Option<std::sync::Arc<crate::store::payload::Payload<History<K>>>>,
);
impl<K: PartialEq> PartialEq for FormPastNames<K> {
    fn eq(&self, other: &Self) -> bool {
        self.same_storage(other) || self.entries() == other.entries()
    }
}
impl<K: Eq> Eq for FormPastNames<K> {}
#[derive(Debug)]
pub(crate) struct History<K> {
    pub entries: Vec<FormPastName<K>>,
    pub logical_units: usize,
}
impl<K: PartialEq> PartialEq for History<K> {
    fn eq(&self, other: &Self) -> bool {
        self.entries == other.entries
    }
}
impl<K: Eq> Eq for History<K> {}
impl<K> Clone for FormPastNames<K> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct FormPastName<K> {
    pub(crate) name: DocumentString,
    pub(crate) target: K,
}
impl<K> FormPastNames<K> {
    pub(crate) fn fresh() -> Self {
        Self(None)
    }
    pub(crate) fn from_payload(
        value: std::sync::Arc<crate::store::payload::Payload<History<K>>>,
    ) -> Self {
        let result = Self(Some(value));
        #[cfg(test)]
        if crate::store::scale_metrics::ENABLED {
            allocation_observation::constructed(&result);
        }
        result
    }
    pub(crate) fn payload(
        &self,
    ) -> Option<&std::sync::Arc<crate::store::payload::Payload<History<K>>>> {
        self.0.as_ref()
    }
    pub(crate) fn entries(&self) -> &[FormPastName<K>] {
        self.0
            .as_ref()
            .map_or(&[], |value| value.value.entries.as_slice())
    }
    pub(crate) fn capacity(&self) -> usize {
        self.0
            .as_ref()
            .map_or(0, |value| value.value.entries.capacity())
    }
    pub(crate) fn logical_units(&self) -> usize {
        self.0.as_ref().map_or(0, |value| value.value.logical_units)
    }
    pub(crate) fn own_usage(&self) -> crate::store::payload::Usage {
        self.0
            .as_ref()
            .map_or(crate::store::payload::Usage::ZERO, |value| value.own_usage)
    }
    pub(crate) fn same_storage(&self, other: &Self) -> bool {
        match (&self.0, &other.0) {
            (None, None) => true,
            (Some(a), Some(b)) => std::sync::Arc::ptr_eq(a, b),
            _ => false,
        }
    }
    #[cfg(test)]
    pub(crate) fn from_entries(entries: Vec<FormPastName<K>>) -> Self {
        if entries.capacity() == 0 {
            return Self::fresh();
        }
        Self::from_payload(crate::store::payload::fixture_history(entries))
    }
    #[cfg(test)]
    pub(crate) fn inspect_allocations(
        &self,
        graph: &mut crate::store::compact_tests::AllocationGraph,
    ) {
        if let Some(value) = &self.0 {
            graph.target(value, "form.names.target");
            graph.allocation(
                value.value.entries.as_ptr() as usize,
                "form.names.buffer",
                value.value.entries.capacity() * size_of::<FormPastName<K>>(),
            );
        }
    }
}
