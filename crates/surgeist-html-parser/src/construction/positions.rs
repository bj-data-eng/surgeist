//! Where the parser created script, style and event handler content attributes.
//!
//! A parser-created element keeps one offset: where its start tag ended, which
//! is where a browser's parser reads the position of a script, an inline style
//! sheet and an inline event handler. The line and column are computed from the
//! offset when a reader asks for them.

use super::report::ReportStorage;
use super::*;
use surgeist_html::{
    construction::{HtmlNodeId, HtmlNodeKind},
    values::Namespace,
    vocabulary::KnownHtmlEventHandler,
};

impl SessionStorage {
    /// Records the event handler content attributes of `node` from the
    /// attribute at `from` on, each with the offset at which the tag being
    /// processed ended. A parse of a source-less session has no such offset and
    /// records none.
    pub(super) fn record_event_handler_starts(
        &mut self,
        c: &mut ParseControl,
        node: usize,
        from: usize,
    ) -> Result<(), SessionError> {
        let Some(offset) = self.token_end else {
            return Ok(());
        };
        c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        let body = self.html_local(c, node, "body")?;
        let mut index = from;
        loop {
            c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
            let HtmlNodeKind::Element(element) = self
                .construction
                .node_at(node)
                .ok_or(SessionError::ForeignNode)?
                .kind()
            else {
                return Ok(());
            };
            if element.name().namespace() != &Namespace::html() {
                return Ok(());
            }
            let Some(attribute) = element.attributes().as_slice().get(index) else {
                return Ok(());
            };
            let handler = if attribute.name().namespace().uri().is_none() {
                // The names that start with `on` are the only candidates; the
                // list of the vocabulary decides which of them are handlers.
                c.charge_unit_inspection()?;
                KnownHtmlEventHandler::from_local_units(attribute.name().local_name().code_units())
            } else {
                None
            };
            if let Some(handler) = handler
                && (handler.applicability()
                    == surgeist_html::vocabulary::HtmlEventHandlerApplicability::AllHtmlElements
                    || body)
            {
                self.push_event_handler_start(
                    c,
                    payload_registry::EventHandlerStart {
                        node,
                        attribute: index,
                        offset,
                    },
                )?;
            }
            index = index.checked_add(1).ok_or_else(|| latch_registry_row(c))?;
        }
    }
    /// The number of attributes the element `target` has.
    pub(super) fn attribute_count(
        &self,
        c: &mut ParseControl,
        target: &ParserNode,
    ) -> Result<usize, SessionError> {
        c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        let index = self.index(c, target)?;
        match self
            .construction
            .node_at(index)
            .ok_or(SessionError::ForeignNode)?
            .kind()
        {
            HtmlNodeKind::Element(element) => Ok(element.attributes().len()),
            _ => Ok(0),
        }
    }
    /// Records the event handler content attributes a merge appended to
    /// `target` after its first `before`.
    pub(super) fn record_merged_event_handlers(
        &mut self,
        c: &mut ParseControl,
        target: &ParserNode,
        before: usize,
    ) -> Result<(), SessionError> {
        let index = self.index(c, target)?;
        self.record_event_handler_starts(c, index, before)
    }
    fn push_event_handler_start(
        &mut self,
        c: &mut ParseControl,
        start: payload_registry::EventHandlerStart,
    ) -> Result<(), SessionError> {
        let slot = match self.handler_starts {
            Some(slot) => slot,
            None => {
                let mut list = OwnedVec::new(HtmlParseResource::AuxiliaryEntries)?;
                list.require_owner(c)?;
                let prepared = match self.prepare_payload_slot_owned(c) {
                    Ok(prepared) => prepared,
                    Err(error) => {
                        list.release(c)?;
                        return Err(error);
                    }
                };
                let slot = self
                    .payloads
                    .push_row(prepared, RetainedPayload::EventHandlerStarts(list))
                    .map_err(|_| latch_registry_row(c))?;
                self.handler_starts = Some(slot);
                slot
            }
        };
        let Some(RetainedPayload::EventHandlerStarts(list)) = self.payloads.values.get_mut(slot)
        else {
            return Err(latch_registry_row(c));
        };
        list.try_push_geometric(c, start)?;
        Ok(())
    }
}

/// The start positions a report holds, read by node.
impl ReportStorage {
    /// The offset at which the start tag of the parser-created script `id`
    /// ended.
    pub(crate) fn script_start(&self, id: &HtmlNodeId) -> Option<usize> {
        let storage = self.history().storage();
        storage.payloads.values.iter().find_map(|row| match row {
            RetainedPayload::Script {
                node: Some(node),
                metadata,
            } => Self::is_node(storage, *node, id)
                .then_some(metadata.start)
                .flatten(),
            _ => None,
        })
    }
    /// The offset at which the start tag of the parser-created style `id`
    /// ended.
    pub(crate) fn style_start(&self, id: &HtmlNodeId) -> Option<usize> {
        let storage = self.history().storage();
        storage.payloads.values.iter().find_map(|row| match row {
            RetainedPayload::Style { node, state } => Self::is_node(storage, *node, id)
                .then_some(state.start)
                .flatten(),
            _ => None,
        })
    }
    fn is_node(storage: &FrozenStorage, index: usize, id: &HtmlNodeId) -> bool {
        storage
            .view
            .node_at(index)
            .is_some_and(|node| node.as_node().id() == *id)
    }
    /// The attribute indices and start offsets of the event handler content
    /// attributes of `id`, in the order the parser created them.
    pub(crate) fn event_handler_starts<'a>(
        &'a self,
        id: &'a HtmlNodeId,
    ) -> impl Iterator<Item = (usize, usize)> + 'a {
        let storage = self.history().storage();
        let starts = storage.payloads.values.iter().find_map(|row| match row {
            RetainedPayload::EventHandlerStarts(list) => Some(list.as_slice()),
            _ => None,
        });
        starts.into_iter().flatten().filter_map(move |start| {
            Self::is_node(storage, start.node, id).then_some((start.attribute, start.offset))
        })
    }
}
