//! Retained operands for a constructor reached by detached element creation.
use super::*;
use crate::element::ElementName;

pub(in crate::store) struct Creation {
    pub(in crate::store) document: DocumentId,
    pub(in crate::store) name: payload::Sealed<ElementName>,
    pub(in crate::store) definition: crate::DomRegistryDefinitionId,
}

pub(super) struct Pending {
    request: DomSourceCustomElementRequest,
    name: payload::Sealed<ElementName>,
}

impl Pending {
    pub(super) fn new(
        operation: DomSourceOperationId,
        command: usize,
        draft: &Draft,
        creation: Creation,
        cancel: &AtomicBool,
    ) -> Result<Self, DomError> {
        let Creation {
            document,
            name,
            definition,
        } = creation;
        draft.source_step(cancel)?;
        let global = draft
            .node(Key::Existing(document.node()))?
            .relevant_global()
            .ok_or(DomError::HostedSourceUnavailable)?;
        Ok(Self {
            request: DomSourceCustomElementRequest {
                operation,
                command,
                document,
                global,
                definition,
            },
            name,
        })
    }
}

impl DomSourceLease<'_> {
    /// Borrow the normalized name from the original factory while the
    /// constructor request is pending.
    pub fn custom_element_creation_name(
        &self,
        request: DomSourceCustomElementRequest,
        cancel: &AtomicBool,
    ) -> Result<&crate::DomQualifiedName, DomError> {
        let operation = self
            .dom
            .source_operation
            .as_deref()
            .filter(|operation| operation.id == self.id)
            .ok_or(DomError::InvalidSourceOperation)?;
        let pending = operation
            .custom_element
            .as_ref()
            .filter(|pending| pending.request == request)
            .ok_or(DomError::InvalidSourceOperation)?;
        operation.draft.source_step(cancel)?;
        Ok(&pending.name.value.value.value)
    }
}

impl Operation {
    pub(super) fn waiting_custom_element(&self) -> Option<DomSourceCustomElementRequest> {
        self.custom_element.as_ref().map(|pending| pending.request)
    }
}
