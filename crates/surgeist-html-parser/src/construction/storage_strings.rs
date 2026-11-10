//! Managed string registration, text insertion and merging, and string disposal.

use super::*;
use surgeist_html::{construction::HtmlNodeKind, values::DocumentString};

impl SessionStorage {
    pub(super) fn check_text(
        control: &mut ParseControl,
        text: &mut Text,
    ) -> Result<(), SessionError> {
        control.require_document_string(text.value.allocation(), text.value.text())?;
        text.origins.require_owner(control)?;
        control.charge(HtmlParseWorkCategory::ProviderCall, 1)?;
        if text.value.text().block().holders() != 1 {
            return Err(SessionError::AliasedPayload);
        }
        Ok(())
    }
    pub(super) fn register_string(
        &mut self,
        control: &mut ParseControl,
        value: FrozenUtf16,
        origins: Option<OwnedVec<HtmlParseTextOrigin>>,
    ) -> Result<(DocumentString, usize), SessionError> {
        if let Err(error) = self.require(control) {
            value.release(control)?;
            if let Some(origins) = origins {
                origins.release(control)?;
            }
            return Err(error);
        }
        self.register_string_owned(control, value, origins)
    }
    pub(super) fn register_string_owned(
        &mut self,
        control: &mut ParseControl,
        value: FrozenUtf16,
        origins: Option<OwnedVec<HtmlParseTextOrigin>>,
    ) -> Result<(DocumentString, usize), SessionError> {
        let origins = if !control.detail().records_graph() {
            // No group names this value, and no origin was recorded for it.
            if let Some(origins) = origins {
                origins.release(control)?;
            }
            None
        } else if let Some(origins) = origins {
            let evidence = match self.origin_evidence(control) {
                Ok(evidence) => evidence,
                Err(error) => {
                    origins.release(control)?;
                    value.release(control)?;
                    return Err(error);
                }
            };
            match evidence.retain_group(control, origins, value.text().len()) {
                Ok(group) => Some(group),
                Err(error) => {
                    value.release(control)?;
                    return Err(error);
                }
            }
        } else if let Some(evidence) = &mut self.evidence {
            match evidence.rule_group(control, value.text().len()) {
                Ok(group) => Some(group),
                Err(error) => {
                    value.release(control)?;
                    return Err(error);
                }
            }
        } else {
            None
        };
        self.register_group_string_owned(control, value, origins)
    }
    /// The group is committed in the sole evidence owner before the actual
    /// native backing slot is reserved. A refused slot leaves no logical slot
    /// reservation and the original backing is reclaimed by this owner.
    pub(super) fn register_group_string_owned(
        &mut self,
        control: &mut ParseControl,
        value: FrozenUtf16,
        origins: Option<OriginGroupRef>,
    ) -> Result<(DocumentString, usize), SessionError> {
        let prepared = match self.prepare_payload_slot_owned(control) {
            Ok(prepared) => prepared,
            Err(error) => {
                value.release(control)?;
                return Err(error);
            }
        };
        let (value, allocation) = value.into_parts();
        let index = self
            .payloads
            .push_row(
                prepared,
                RetainedPayload::String {
                    value,
                    allocation,
                    origins,
                    location: StringLocation::Detached,
                },
            )
            .map_err(|_| latch_registry_row(control))?;
        let Some(RetainedPayload::String {
            value,
            allocation,
            origins,
            ..
        }) = self.payloads.values.get_mut(index)
        else {
            return Err(latch_registry_row(control));
        };
        control.move_document_string_to_managed(allocation, value)?;
        if let Some(origins) = origins {
            self.evidence
                .as_mut()
                .ok_or(SessionError::WrongToken)?
                .bind_leaf_value(control, origins, index)?;
        }
        control.charge(HtmlParseWorkCategory::ProviderCall, 1)?;
        control.charge(
            HtmlParseWorkCategory::CopyByte,
            mem::size_of::<DocumentString>() as u64,
        )?;
        Ok((value.clone(), index))
    }
    pub(super) fn register_text(
        &mut self,
        control: &mut ParseControl,
        text: Text,
    ) -> Result<(DocumentString, usize), SessionError> {
        self.register_string(control, text.value, Some(text.origins))
    }
    /// Implements the pinned character-insertion operation at an already
    /// selected insertion location. Foster-parent selection belongs to the
    /// later tree builder. The returned offset locates the original token text
    /// inside the native text node; retained origins remain exact token facts.
    pub(super) fn insert_text(
        &mut self,
        control: &mut ParseControl,
        parent: &ParserNode,
        before: Option<&ParserNode>,
        input: &mut Option<Text>,
    ) -> Result<Option<TextInsertion>, SessionError> {
        let parent_index = self.index(control, parent)?;
        let before_index = before.map(|node| self.index(control, node)).transpose()?;
        self.container(control, parent_index)?;
        control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        if let Some(before) = before_index {
            control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
            if self
                .construction
                .node_at(before)
                .ok_or_else(|| latch_registry_row(control))?
                .parent()
                .map(|node| node.position())
                != Some(parent_index)
            {
                return Err(SessionError::WrongSibling);
            }
        }
        let text = input.as_mut().ok_or(SessionError::Consumed)?;
        Self::check_text(control, text)?;
        control.charge(HtmlParseWorkCategory::RecordVisit, 2)?;
        if text.value.text().is_empty()
            || matches!(
                self.construction
                    .node_at(parent_index)
                    .ok_or_else(|| latch_registry_row(control))?
                    .kind(),
                HtmlNodeKind::Document(_)
            )
        {
            let Some(text) = input.take() else {
                return Err(SessionError::Consumed);
            };
            text.release(control)?;
            return Ok(None);
        }
        let mut preceding = None;
        if before_index.is_none() {
            // Appending: the preceding sibling is the last child, which needs
            // no walk over the parent's other children.
            control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
            let parent = self
                .construction
                .node_at(parent_index)
                .ok_or_else(|| latch_registry_row(control))?;
            if let Some(last) = parent.child_count().checked_sub(1) {
                preceding = Some(
                    parent
                        .child_at(last)
                        .ok_or_else(|| latch_registry_row(control))?
                        .position(),
                );
            }
        } else {
            // Walk back from the last child to the reference sibling, so the
            // work is the children behind it and not the ones in front of it;
            // the entry just before it is the preceding sibling.
            let parent = self
                .construction
                .node_at(parent_index)
                .ok_or_else(|| latch_registry_row(control))?;
            let mut position = parent.child_count();
            loop {
                let Some(found) = position.checked_sub(1) else {
                    return Err(latch_registry_row(control));
                };
                control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
                let child = parent
                    .child_at(found)
                    .ok_or_else(|| latch_registry_row(control))?
                    .position();
                position = found;
                if Some(child) == before_index {
                    break;
                }
            }
            if let Some(previous) = position.checked_sub(1) {
                control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
                preceding = Some(
                    parent
                        .child_at(previous)
                        .ok_or_else(|| latch_registry_row(control))?
                        .position(),
                );
            }
        }
        control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        let preceding = match preceding {
            Some(index) => match self.construction.node_at(index).map(|record| record.kind()) {
                Some(HtmlNodeKind::Text(_)) => Some(index),
                Some(_) => None,
                None => return Err(latch_registry_row(control)),
            },
            None => None,
        };
        // The preceding Text record and the registry row holding its string.
        let preceding = match preceding {
            Some(index) => Some((
                index,
                self.field_slot(control, StringLocation::Text(index))?,
            )),
            None => None,
        };
        let dispose_old = match preceding {
            Some((_, slot)) => self.releasable_string(control, slot, true)?,
            None => false,
        };
        let prepared = if preceding.is_none() {
            let document = match self.node_document_ref(control, parent_index) {
                Ok(document) => Some(document),
                Err(SessionError::NodeDocumentUnavailable)
                    if matches!(
                        self.construction
                            .node_at(0)
                            .ok_or_else(|| latch_registry_row(control))?
                            .kind(),
                        HtmlNodeKind::Fragment
                    ) =>
                {
                    None
                }
                Err(error) => return Err(error),
            };
            self.prepare_document_allocation(control, document, false)?
        } else {
            None
        };
        let Some(text) = input.take() else {
            return Err(SessionError::Consumed);
        };
        let (value, ticket) = self.register_text(control, text)?;
        if let Some((index, old_slot)) = preceding {
            self.append_registered_text(
                control,
                text_append::RegisteredTextAppend {
                    parent: parent_index,
                    index,
                    old_slot,
                    incoming_slot: ticket,
                    incoming: value,
                    dispose_old,
                    at_end: before_index.is_none(),
                },
            )
            .map(Some)
        } else {
            control.charge(HtmlParseWorkCategory::RecordWrite, 2)?;
            payload_write::<Option<usize>>(control, 1)?;
            let node = self.allocate_prepared_owned(
                control,
                creation_command(HtmlNodeKind::Text(value)),
                false,
                0,
                0,
                prepared,
            )?;
            self.bind_string(ticket, StringLocation::Text(node.index))
                .map_err(|_| latch_registry_row(control))?;
            self.insert(control, parent, &node, before, InsertionCause::NonParser)?;
            Ok(Some(TextInsertion {
                _node: node,
                _offset: 0,
            }))
        }
    }
    pub(super) fn releasable_string(
        &self,
        control: &mut ParseControl,
        slot: usize,
        native: bool,
    ) -> Result<bool, SessionError> {
        control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        let Some(RetainedPayload::String {
            value,
            allocation,
            origins,
            ..
        }) = self.payloads.values.get(slot)
        else {
            return Err(latch_registry_row(control));
        };
        control.require_document_string(allocation, value)?;
        control.charge(HtmlParseWorkCategory::ProviderCall, 1)?;
        if value.block().holders() != if native { 2 } else { 1 } {
            return Err(SessionError::AliasedPayload);
        }
        if let Some(group) = origins {
            let evidence = self.evidence.as_ref().ok_or(SessionError::WrongToken)?;
            control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
            control.charge(
                HtmlParseWorkCategory::InspectByte,
                mem::size_of::<provenance::OriginGroup>() as u64,
            )?;
            let group = evidence.require_group(group)?;
            Ok(!matches!(group.body, provenance::GroupBody::Leaf { .. }) || group.units == 0)
        } else {
            Ok(true)
        }
    }
    /// Called only after the admitted native/raw slot commit has removed every
    /// live alias. Leaf backings remain owned; aggregate history retains IDs,
    /// not another materialized prefix. This deallocation never polls useful
    /// work or refunds cumulative work/high-water observations.
    pub(super) fn dispose_aggregate_string(
        &mut self,
        control: &mut ParseControl,
        slot: usize,
    ) -> Result<(), SessionError> {
        let Some(row) = self.payloads.values.get_mut(slot) else {
            return Err(latch_registry_row(control));
        };
        let (value, allocation, origins) = match mem::replace(row, RetainedPayload::Retired) {
            RetainedPayload::String {
                value,
                allocation,
                origins,
                ..
            } => (value, allocation, origins),
            // A row of another kind is handed back where it was found.
            other => {
                *row = other;
                return Err(SessionError::WrongToken);
            }
        };
        drop(value);
        control.release_document_string(allocation)?;
        if let Some(origins) = origins {
            *row = RetainedPayload::HistoricalString { origins };
        }
        Ok(())
    }
    /// Retired raw fields may be unique immutable history leaves. Keep those
    /// actual allocations; only reconstructed aggregate backings are redundant.
    pub(super) fn retire_registered_string(
        &mut self,
        control: &mut ParseControl,
        slot: usize,
    ) -> Result<(), SessionError> {
        let dispose = self.releasable_string(control, slot, false)?;
        if dispose {
            payload_write::<RetainedPayload>(control, 2)?;
        }
        self.retire_field_group(control, slot)?;
        if dispose {
            self.dispose_aggregate_string(control, slot)?;
        }
        Ok(())
    }
    pub(super) fn bind_string(
        &mut self,
        index: usize,
        location: StringLocation,
    ) -> Result<(), SessionError> {
        let Some(RetainedPayload::String {
            location: current, ..
        }) = self.payloads.values.get_mut(index)
        else {
            return Err(SessionError::registry_row());
        };
        *current = location;
        match location {
            StringLocation::Text(node) => self.payloads.join_text(node, index)?,
            // The join names the first row bound to the instruction, as a scan
            // of the registry does.
            StringLocation::ProcessingInstructionData(_) => self.payloads.join_row(index)?,
            _ => {}
        }
        Ok(())
    }
}
