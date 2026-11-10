//! Where the parser created script, style and event handler content attributes.
//!
//! A parser-created element keeps one offset: where its start tag ended, which
//! is where a browser's parser reads the position of a script, an inline style
//! sheet and an inline event handler. The line and column are computed from the
//! offset when a reader asks for them.

use super::report::ReportStorage;
use super::*;
use std::collections::HashMap;
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
            if let Some(handler) = handler {
                let applies = handler.applicability()
                    == surgeist_html::vocabulary::HtmlEventHandlerApplicability::AllHtmlElements
                    || self.html_local(c, node, "body")?;
                if applies {
                    let (Ok(node), Ok(attribute)) = (u32::try_from(node), u32::try_from(index))
                    else {
                        return Err(c
                            .stop_representation(HtmlParseResource::AuxiliaryEntries)
                            .into());
                    };
                    self.push_event_handler_start(
                        c,
                        payload_registry::EventHandlerStart {
                            node,
                            attribute,
                            offset,
                        },
                    )?;
                }
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
        if let Some(slot) = self.handler_starts
            && let Some(RetainedPayload::EventHandlerStarts(list)) = self.payloads.values.get(slot)
            && list
                .as_slice()
                .iter()
                .rev()
                .take_while(|earlier| earlier.node == start.node)
                .any(|earlier| earlier.attribute == start.attribute)
        {
            // The creation recorded it already.
            return Ok(());
        }
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

/// The start positions of a report by node, built when the first is read: one
/// pass over the registry, then a constant-time read per node. The index is not
/// part of the accounted parse; it holds one entry per script, style and event
/// handler content attribute the parser created.
#[derive(Default)]
pub(crate) struct StartIndex {
    script: HashMap<HtmlNodeId, usize>,
    style: HashMap<HtmlNodeId, usize>,
    handlers: HashMap<HtmlNodeId, Vec<(usize, usize)>>,
}

impl StartIndex {
    fn build(storage: &FrozenStorage) -> Self {
        let id_of = |index: usize| storage.view.node_at(index).map(|node| node.as_node().id());
        let mut index = Self::default();
        for row in &storage.payloads.values {
            match row {
                RetainedPayload::Script {
                    node: Some(node),
                    metadata,
                } => {
                    if let (Some(id), Some(start)) = (id_of(*node), metadata.start) {
                        index.script.insert(id, start);
                    }
                }
                RetainedPayload::Style { node, state } => {
                    if let (Some(id), Some(start)) = (id_of(*node), state.start) {
                        index.style.insert(id, start);
                    }
                }
                RetainedPayload::EventHandlerStarts(list) => {
                    for start in list.as_slice() {
                        let (Ok(node), Ok(attribute)) = (
                            usize::try_from(start.node),
                            usize::try_from(start.attribute),
                        ) else {
                            continue;
                        };
                        if let Some(id) = id_of(node) {
                            index
                                .handlers
                                .entry(id)
                                .or_default()
                                .push((attribute, start.offset));
                        }
                    }
                }
                _ => {}
            }
        }
        for starts in index.handlers.values_mut() {
            starts.sort_unstable();
        }
        index
    }
}

/// The start positions a report holds, read by node.
impl ReportStorage {
    fn starts(&self) -> &StartIndex {
        self.starts
            .get_or_init(|| StartIndex::build(self.history().storage()))
    }
    /// The offset at which the start tag of the parser-created script `id`
    /// ended.
    pub(crate) fn script_start(&self, id: &HtmlNodeId) -> Option<usize> {
        self.starts().script.get(id).copied()
    }
    /// The offset at which the start tag of the parser-created style `id`
    /// ended.
    pub(crate) fn style_start(&self, id: &HtmlNodeId) -> Option<usize> {
        self.starts().style.get(id).copied()
    }
    /// The attribute indices and start offsets of the event handler content
    /// attributes of `id`, in attribute order.
    pub(crate) fn event_handler_starts<'a>(
        &'a self,
        id: &'a HtmlNodeId,
    ) -> impl Iterator<Item = (usize, usize)> + 'a {
        self.starts()
            .handlers
            .get(id)
            .into_iter()
            .flatten()
            .copied()
    }
}
