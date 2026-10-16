//! One role-qualified activity observation for reached original-source hooks.
use super::*;

impl SessionStorage {
    pub(super) fn document_fully_active(
        &self,
        c: &mut ParseControl,
        node: usize,
    ) -> Result<Option<bool>, SessionError> {
        let document = match self.node_document_ref(c, node) {
            Ok(document) => document,
            Err(SessionError::NodeDocumentUnavailable) => {
                return Ok(None);
            }
            Err(error) => return Err(error),
        };
        match document {
            DocumentRef::Owned(document) => {
                let slot = self.document_slot(c, document)?;
                c.charge(WorkCategory::RecordVisit, 1)?;
                if matches!(
                    self.payloads.values[slot],
                    RetainedPayload::Document {
                        state: template::DocumentState::InertTemplate,
                        ..
                    }
                ) {
                    // The actual template-owner constructor creates this
                    // independent Document without a navigable or host ingress.
                    return Ok(Some(false));
                }
                let completion = match self.completion_slot_during_effects(c) {
                    Ok(slot) => slot,
                    Err(SessionError::CompletionMetadataUnavailable) => {
                        return Ok(None);
                    }
                    Err(error) => return Err(error),
                };
                c.charge(WorkCategory::RecordVisit, 1)?;
                // This original closed constructor issues the fresh ordinary
                // parser Document, including the distinct §13.4 auxiliary
                // Document. Its completion record cannot qualify another owner.
                Ok(match self.payloads.values[completion] {
                    RetainedPayload::StandaloneDocumentCompletion { node, .. }
                        if node == document =>
                    {
                        Some(false)
                    }
                    _ => None,
                })
            }
            DocumentRef::Captured(document) => {
                c.charge(WorkCategory::RecordVisit, 3)?;
                let capture = self
                    .fragment
                    .as_ref()
                    .ok_or(SessionError::ContextUnavailable)?
                    .context
                    .capture();
                let node = capture
                    .node(document.subject())
                    .map_err(|_| SessionError::ContextUnavailable)?;
                let crate::HtmlCapturedNodeKind::Document(facts) = node.kind() else {
                    return Err(SessionError::NotDocument);
                };
                // HTML #fully-active and #nav-bc: an absent actual browsing
                // context rules out activity. Present does not establish it.
                Ok(match facts.nonce_environment().browsing_context() {
                    surgeist_html::HtmlBrowsingContextPresence::Absent => Some(false),
                    surgeist_html::HtmlBrowsingContextPresence::Present
                    | surgeist_html::HtmlBrowsingContextPresence::Unavailable => None,
                })
            }
        }
    }
}
