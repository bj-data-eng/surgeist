//! Shared canonical empty-list proof and the original Show document adapter.
use super::*;

impl Draft {
    pub(super) fn dialog_method_node_document(
        &self,
        subject: Key,
        cancel: &AtomicBool,
    ) -> Result<DocumentId, DomError> {
        self.source_step(cancel)?;
        self.dialog_state(subject)?;
        let document = self.node(subject)?.document;
        self.source_step(cancel)?;
        if !matches!(&self.node(document)?.kind, DomNodeKind::Document(_)) {
            return Err(DomError::WrongNodeKind);
        }
        self.published_id(document)
            .map(DocumentId)
            .ok_or(DomError::UnavailableCreation)
    }

    pub(in crate::store) fn document_known_empty_popovers(
        &self,
        document: DocumentId,
        cancel: &AtomicBool,
    ) -> Result<bool, DomError> {
        self.source_step(cancel)?;
        let document = self.resolve(document.node().into())?;
        self.known_empty_popover_lists(document, cancel)
    }

    pub(in crate::store) fn known_empty_popover_lists(
        &self,
        document: Key,
        cancel: &AtomicBool,
    ) -> Result<bool, DomError> {
        self.source_step(cancel)?;
        let node = self.node(document)?;
        if !matches!(&node.kind, DomNodeKind::Document(_)) {
            return Err(DomError::WrongNodeKind);
        }
        let Some(DocumentInteractiveState::Known { top_layer, .. }) =
            node.interactive_document.as_ref()
        else {
            return Ok(false);
        };
        // The Auto/Hint lists filter actual showing popovers. Current admitted
        // HTML common state proves hidden independently of authored popover text;
        // ordinary raw modal Dialogs therefore do not enter either showing list.
        for member in top_layer {
            self.source_step(cancel)?;
            let element = self.element(*member)?;
            if element
                .common_state()
                .html()
                .is_some_and(|state| !state.popover_is_hidden())
            {
                return Ok(false);
            }
        }
        Ok(true)
    }
}

#[cfg(test)]
mod tests;
