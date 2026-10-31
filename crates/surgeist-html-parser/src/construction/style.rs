//! Actual Style creation/stack history. CSSOM association processing is pending.
use super::*;

/// This immutable parser profile has not completed Style's CSSOM association
/// operation. Pending is distinct from a proven null associated stylesheet.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StyleAssociation {
    Pending,
}
pub(super) struct StyleMetadata {
    pub(super) state: surgeist_html::HtmlStyleState,
    pub(super) parser_document: Option<DocumentRef>,
    pub(super) cause: Option<provenance::ConstructionCauseRef>,
}
impl SessionStorage {
    pub(super) fn initialize_style(
        &mut self,
        c: &mut ParseControl,
        node: usize,
    ) -> Result<(), SessionError> {
        if !self.html_local(c, node, "style")? {
            return Ok(());
        }
        let cause = self.associated_cause(c)?;
        self.prepare_payload_slot_owned(c)?;
        payload_write::<StyleMetadata>(c, 1)?;
        self.payloads
            .push_row(RetainedPayload::Style {
                node,
                state: StyleMetadata {
                    state: surgeist_html::HtmlStyleState::fresh(),
                    parser_document: None,
                    cause,
                },
            })
            .map_err(|_| latch_registry_row(c))?;
        self.record_association(
            c,
            node,
            super::pending_association::OccurrenceKind::StyleCreated,
            None,
            None,
            false,
        )?;
        Ok(())
    }
    pub(super) fn style_slot(
        &self,
        c: &mut ParseControl,
        node: usize,
    ) -> Result<Option<usize>, SessionError> {
        if !self.html_local(c, node, "style")? {
            return Ok(None);
        }
        self.payloads
            .joined(c, node, Join::Style)?
            .map(Some)
            .ok_or(SessionError::WrongToken)
    }
    /// Called only by the sealed original create-element-for-token operation.
    /// Template contents have another current owner; their creating parser is
    /// still this actual parser Document, never the template's inert Document.
    pub(super) fn mark_document_parser_style(
        &mut self,
        c: &mut ParseControl,
        node: usize,
    ) -> Result<(), SessionError> {
        let Some(slot) = self.style_slot(c, node)? else {
            return Ok(());
        };
        self.document_slot(c, 0)?;
        let cause = self.associated_cause(c)?;
        payload_write::<StyleMetadata>(c, 1)?;
        let Some(RetainedPayload::Style { state, .. }) = self.payloads.values.get_mut(slot) else {
            return Err(latch_registry_row(c));
        };
        state.state = surgeist_html::HtmlStyleState::document_parser_created();
        state.parser_document = Some(DocumentRef::Owned(0));
        state.cause = cause;
        self.record_association(
            c,
            node,
            super::pending_association::OccurrenceKind::StyleParserCreated,
            None,
            None,
            false,
        )?;
        Ok(())
    }
    pub(super) fn style_stack_changed(
        &mut self,
        c: &mut ParseControl,
        node: usize,
        added: bool,
    ) -> Result<(), SessionError> {
        let Some(slot) = self.style_slot(c, node)? else {
            return Ok(());
        };
        let cause = self.associated_cause(c)?;
        payload_write::<StyleMetadata>(c, 1)?;
        let Some(RetainedPayload::Style { state, .. }) = self.payloads.values.get_mut(slot) else {
            return Err(latch_registry_row(c));
        };
        if added {
            state.state.parser_stack_added()
        } else {
            state.state.parser_stack_removed()
        }
        .map_err(|_| SessionError::WrongToken)?;
        state.cause = cause;
        if !added {
            self.association_style_trigger(
                c,
                node,
                surgeist_html::HtmlStyleTrigger::ParserPopped,
                None,
            )?;
        }
        Ok(())
    }
}
impl ParserArenaSession {
    /// Notification after the actual original stack push has committed. No
    /// external caller can manufacture this source history or a parser Document.
    pub(crate) fn parser_stack_added(
        &mut self,
        c: &mut ParseControl,
        node: &ParserNode,
    ) -> Result<(), SessionError> {
        let storage = self.active_mut()?;
        let index = storage.index(c, node)?;
        storage.style_stack_changed(c, index, true)
    }
}
