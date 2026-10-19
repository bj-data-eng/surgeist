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
        let document = self.node(subject)?.document();
        self.source_step(cancel)?;
        if !matches!(
            &self.node(document)?.kind(),
            transaction::KindRead::Document(_)
        ) {
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
        #[cfg(test)]
        let _observation = scale_metrics::ENABLED.then(|| discovery_tests::QueryRead::begin(self));
        self.source_step(cancel)?;
        let node = self.node(document)?;
        if !matches!(&node.kind(), transaction::KindRead::Document(_)) {
            return Err(DomError::WrongNodeKind);
        }
        let Some(transaction::InteractiveRead::Known { .. }) = node.interactive_document() else {
            return Ok(false);
        };
        // The Auto/Hint lists filter actual showing popovers. Current admitted
        // HTML common state proves hidden independently of authored popover text;
        // ordinary raw modal Dialogs therefore do not enter either showing list.
        Ok(!self.role_facts.has_any(
            transaction::role_facts::Role::ShowingPopover,
            self.live_key(document),
        ))
    }
}

#[cfg(test)]
mod discovery_tests;
#[cfg(test)]
mod tests;
