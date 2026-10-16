//! Canonical registry identity, distinct from capture-local registry identities.

use crate::{DomOwnerId, DomSnapshot};

/// One actual immutable global registry object in the canonical owner.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DomRegistryId {
    pub(crate) owner: DomOwnerId,
    pub(crate) slot: usize,
    pub(crate) generation: u64,
}
impl DomRegistryId {
    pub const fn owner(self) -> DomOwnerId {
        self.owner
    }
}

/// A registry at one exact immutable canonical snapshot.
#[derive(Clone, Copy, Debug)]
pub struct DomRegistry<'a> {
    pub(crate) snapshot: &'a DomSnapshot,
    pub(crate) id: DomRegistryId,
}
impl<'a> DomRegistry<'a> {
    pub const fn id(self) -> DomRegistryId {
        self.id
    }
    pub fn scope(self) -> surgeist_html_semantics::HtmlRegistryScope {
        self.record()
            .capture
            .registry_scope(&self.record().local)
            .expect("the controlled registry factory established its exact Global registry")
    }
    pub fn definition_count(self) -> usize {
        match self
            .record()
            .capture
            .definitions(&self.record().local)
            .expect("the canonical record owns its exact capture-local registry")
        {
            surgeist_html_semantics::HtmlCapturedDefinitions::Complete(definitions) => {
                definitions.len()
            }
            surgeist_html_semantics::HtmlCapturedDefinitions::Unavailable => {
                unreachable!("controlled complete registry")
            }
        }
    }
    pub const fn is_scoped(self) -> bool {
        false
    }
    pub const fn definition_is_running(self) -> bool {
        false
    }
    pub const fn scoped_document_count(self) -> usize {
        0
    }
    pub const fn when_defined_count(self) -> usize {
        0
    }
    /// Consume the original producer's exact namespace/local/is lookup over
    /// this real complete registry. A known miss retains its non-null identity.
    pub fn lookup_definition(
        self,
        namespace: &crate::Namespace,
        local_name: &crate::DocumentString,
        is_value: Option<&crate::DocumentString>,
    ) -> Result<
        Option<&'a surgeist_html_semantics::HtmlCustomElementDefinition>,
        surgeist_html_semantics::HtmlCaptureError,
    > {
        let record = self.record();
        record.capture.lookup_definition(
            &surgeist_html_semantics::HtmlCapturedRegistry::Registry(record.local.clone()),
            namespace,
            local_name,
            is_value,
        )
    }
    fn record(self) -> &'a crate::store::registry::Record {
        self.snapshot
            .state
            .registries
            .as_ref()
            .expect("checked registry bank")
            .get(self.id)
            .expect("checked registry generation")
    }
}
