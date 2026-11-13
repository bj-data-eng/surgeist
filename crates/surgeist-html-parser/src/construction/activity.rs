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
                c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
                if matches!(
                    self.payloads.facts.documents.row(slot).map(|row| row.state),
                    Some(template::DocumentState::InertTemplate)
                ) {
                    // The actual template-owner constructor creates this
                    // independent Document without a navigable or host ingress.
                    return Ok(Some(false));
                }
                match self.completion_during_effects(c) {
                    Ok(_) => {}
                    Err(SessionError::CompletionMetadataUnavailable) => {
                        return Ok(None);
                    }
                    Err(error) => return Err(error),
                }
                // This original closed constructor issues the fresh ordinary
                // parser Document, including the distinct §13.4 auxiliary
                // Document. Its completion belongs to the Document at record 0
                // and cannot qualify another owner.
                Ok((document == 0).then_some(false))
            }
            DocumentRef::Captured(document) => {
                c.charge(HtmlParseWorkCategory::RecordVisit, 3)?;
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
                    surgeist_html::security::nonce::HtmlBrowsingContextPresence::Absent => {
                        Some(false)
                    }
                    surgeist_html::security::nonce::HtmlBrowsingContextPresence::Present
                    | surgeist_html::security::nonce::HtmlBrowsingContextPresence::Unavailable => {
                        None
                    }
                })
            }
        }
    }
}
