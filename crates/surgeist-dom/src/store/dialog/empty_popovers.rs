//! Read-only empty branches of the original Show popover algorithms.
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

    pub(super) fn dialog_method_known_empty_top_layer(
        &self,
        document: DocumentId,
        cancel: &AtomicBool,
    ) -> Result<bool, DomError> {
        self.source_step(cancel)?;
        let document = self.resolve(document.node().into())?;
        self.source_step(cancel)?;
        let node = self.node(document)?;
        if !matches!(&node.kind, DomNodeKind::Document(_)) {
            return Err(DomError::WrongNodeKind);
        }
        Ok(matches!(
            node.interactive_document.as_ref(),
            Some(DocumentInteractiveState::Known { top_layer, .. }) if top_layer.is_empty()
        ))
    }
}

#[cfg(test)]
mod tests;
