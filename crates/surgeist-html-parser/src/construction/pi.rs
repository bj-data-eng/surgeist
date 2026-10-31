//! DOM PI pseudo-attributes. XML-stylesheet 20101028 §3, XML 1.0 Fifth Edition
//! productions 2–5 and 66 define this whole-string operation, not HTML recovery.
use super::*;

pub(super) struct PseudoAttribute {
    pub(super) name: DocumentString,
    pub(super) value: DocumentString,
    name_slot: usize,
    value_slot: usize,
}

struct PendingField {
    buffer: Utf16Buffer,
    maps: OwnedVec<TextOrigin>,
}
impl PendingField {
    fn new(c: &mut ParseControl) -> Result<Self, SessionError> {
        let buffer = Utf16Buffer::new(c)?;
        match OwnedVec::new(HtmlParseResource::SourceMapEntries) {
            Ok(maps) => Ok(Self { buffer, maps }),
            Err(error) => {
                buffer.release(c)?;
                Err(error.into())
            }
        }
    }
    fn release(self, c: &mut ParseControl) -> Result<(), SessionError> {
        self.buffer.release(c)?;
        self.maps.release(c)?;
        Ok(())
    }
}
impl SessionStorage {
    fn raw_value(&self, c: &mut ParseControl, slot: usize) -> Result<DocumentString, SessionError> {
        c.charge(WorkCategory::RecordVisit, 1)?;
        let RetainedPayload::String {
            value, allocation, ..
        } = &self.payloads.values[slot]
        else {
            return Err(SessionError::WrongToken);
        };
        c.require_document_string(allocation, value)?;
        c.charge(WorkCategory::ProviderCall, 1)?;
        payload_write::<DocumentString>(c, 1)?;
        Ok(value.clone())
    }
    fn retire_raw(&mut self, c: &mut ParseControl, slot: usize) -> Result<(), SessionError> {
        self.retire_registered_string(c, slot)
    }
    pub(super) fn prepare_pi_map(&mut self, c: &mut ParseControl) -> Result<usize, SessionError> {
        let values = OwnedVec::new(HtmlParseResource::AuxiliaryEntries)?;
        self.prepare_payload_slot_owned(c)?;
        let map = self.payloads.values.len();
        self.payloads.values.push(RetainedPayload::PiAttributes {
            node: None,
            values,
            ready: false,
        });
        Ok(map)
    }
    pub(super) fn bind_pi_map(&mut self, map: usize, node: usize) {
        let RetainedPayload::PiAttributes { node: bound, .. } = &mut self.payloads.values[map]
        else {
            unreachable!()
        };
        *bound = Some(node);
    }

    pub(super) fn parse_pi_map(
        &mut self,
        c: &mut ParseControl,
        node: usize,
        map: usize,
    ) -> Result<(), SessionError> {
        use surgeist_html::bounded::{
            HtmlPiPseudoAttributeAction as Action, HtmlPiPseudoAttributeCursor as Cursor,
            HtmlPiPseudoAttributeField as Field, HtmlPiPseudoAttributeName as Name,
            HtmlPiPseudoAttributeReply as Reply,
        };
        c.charge(WorkCategory::RecordVisit, 2)?;
        let HtmlNodeKind::ProcessingInstruction(pi) = &self
            .construction
            .node_at(node)
            .ok_or_else(|| latch_registry_row(c))?
            .kind()
        else {
            return Err(SessionError::WrongToken);
        };
        let mut cursor = Cursor::new(pi.data().len());
        let mut name: Option<PendingField> = None;
        let mut value: Option<PendingField> = None;
        let result = (|| {
            while let Some(action) = cursor.action() {
                payload_write::<Cursor>(c, 1)?;
                let reply = match action {
                    Action::Read(offset) => {
                        c.charge(WorkCategory::RecordVisit, 2)?;
                        let HtmlNodeKind::ProcessingInstruction(pi) = &self
                            .construction
                            .node_at(node)
                            .ok_or_else(|| latch_registry_row(c))?
                            .kind()
                        else {
                            return Err(SessionError::WrongToken);
                        };
                        let unit = if offset < pi.data().len() {
                            c.charge(WorkCategory::InspectByte, 2)?;
                            Some(pi.data().code_units()[offset])
                        } else if offset == pi.data().len() {
                            None
                        } else {
                            return Err(SessionError::WrongToken);
                        };
                        Reply::Unit(unit)
                    }
                    Action::Step => Reply::Ack,
                    Action::Begin(field) => {
                        let pending = PendingField::new(c)?;
                        match field {
                            Field::Name => name = Some(pending),
                            Field::Value => value = Some(pending),
                        }
                        Reply::Ack
                    }
                    Action::Emit {
                        field,
                        units,
                        len,
                        start,
                        end,
                        reference,
                    } => {
                        let pending = match field {
                            Field::Name => name.as_mut(),
                            Field::Value => value.as_mut(),
                        }
                        .ok_or(SessionError::WrongToken)?;
                        let begin = pending.buffer.len();
                        pending.buffer.extend_units(&units[..len], c)?;
                        self.pi_piece_origins(
                            c,
                            node,
                            start..end,
                            begin..pending.buffer.len(),
                            reference,
                            &mut pending.maps,
                        )?;
                        Reply::Ack
                    }
                    Action::NameLength(index) => {
                        c.charge(WorkCategory::RecordVisit, 1)?;
                        let RetainedPayload::PiAttributes { values, .. } =
                            &self.payloads.values[map]
                        else {
                            return Err(SessionError::WrongToken);
                        };
                        Reply::Length(
                            values
                                .as_slice()
                                .get(index)
                                .ok_or(SessionError::WrongToken)?
                                .name
                                .len(),
                        )
                    }
                    Action::NameUnit(which, offset) => {
                        c.charge(WorkCategory::RecordVisit, 1)?;
                        let units = match which {
                            Name::Current => name
                                .as_ref()
                                .ok_or(SessionError::WrongToken)?
                                .buffer
                                .units(),
                            Name::Pair(index) => {
                                let RetainedPayload::PiAttributes { values, .. } =
                                    &self.payloads.values[map]
                                else {
                                    return Err(SessionError::WrongToken);
                                };
                                values
                                    .as_slice()
                                    .get(index)
                                    .ok_or(SessionError::WrongToken)?
                                    .name
                                    .code_units()
                            }
                        };
                        if offset >= units.len() {
                            return Err(SessionError::WrongToken);
                        }
                        c.charge(WorkCategory::InspectByte, 2)?;
                        Reply::Unit(Some(units[offset]))
                    }
                    Action::Commit => {
                        self.commit_pi_pair(
                            c,
                            node,
                            map,
                            name.take().ok_or(SessionError::WrongToken)?,
                            value.take().ok_or(SessionError::WrongToken)?,
                        )?;
                        Reply::Ack
                    }
                    Action::Discard => {
                        if let Some(field) = name.take() {
                            field.release(c)?;
                        }
                        if let Some(field) = value.take() {
                            field.release(c)?;
                        }
                        Reply::Ack
                    }
                    Action::Pop => {
                        self.pop_pi_pair(c, map)?;
                        Reply::Ack
                    }
                    Action::Ready(_) => {
                        payload_write::<bool>(c, 1)?;
                        let RetainedPayload::PiAttributes { ready, .. } =
                            &mut self.payloads.values[map]
                        else {
                            return Err(SessionError::WrongToken);
                        };
                        *ready = true;
                        Reply::Ack
                    }
                };
                cursor.reply(reply).map_err(|e| match e {
                    surgeist_html::bounded::HtmlPiPseudoAttributeError::Integrity => {
                        SessionError::WrongToken
                    }
                    surgeist_html::bounded::HtmlPiPseudoAttributeError::Representation => {
                        SessionError::Stop(
                            c.representation::<()>(HtmlParseResource::AuxiliaryEntries)
                                .unwrap_err(),
                        )
                    }
                })?;
            }
            Ok(())
        })();
        // Only actual transient owners are reclaimed after a fixed terminal stop.
        if let Some(field) = name {
            field.release(c)?;
        }
        if let Some(field) = value {
            field.release(c)?;
        }
        result
    }
    fn commit_pi_pair(
        &mut self,
        c: &mut ParseControl,
        node: usize,
        map: usize,
        name: PendingField,
        value: PendingField,
    ) -> Result<(), SessionError> {
        let PendingField {
            buffer: name,
            maps: name_maps,
        } = name;
        let PendingField {
            buffer: value,
            maps: value_maps,
        } = value;
        let name = match name.freeze(c) {
            Ok(n) => n,
            Err(stop) => {
                value.release(c)?;
                name_maps.release(c)?;
                value_maps.release(c)?;
                return Err(stop.into());
            }
        };
        let name_slot = match self.register_pi_field(c, node, name, name_maps) {
            Ok(n) => n,
            Err(e) => {
                value.release(c)?;
                value_maps.release(c)?;
                return Err(e);
            }
        };
        let value = match value.freeze(c) {
            Ok(value) => value,
            Err(stop) => {
                value_maps.release(c)?;
                return Err(stop.into());
            }
        };
        let value_slot = self.register_pi_field(c, node, value, value_maps)?;
        payload_write::<[StringLocation; 2]>(c, 2)?;
        let name = self.raw_value(c, name_slot)?;
        let value = self.raw_value(c, value_slot)?;
        let RetainedPayload::PiAttributes { values, .. } = &mut self.payloads.values[map] else {
            unreachable!()
        };
        let index = values.len();
        if let Err((stop, pair)) = values.try_push_recover(
            c,
            PseudoAttribute {
                name,
                value,
                name_slot,
                value_slot,
            },
        ) {
            drop(pair);
            return Err(stop.into());
        }
        self.bind_string(name_slot, StringLocation::PiName(map, index))
            .map_err(|_| latch_registry_row(c))?;
        self.bind_string(value_slot, StringLocation::PiValue(map, index))
            .map_err(|_| latch_registry_row(c))?;
        Ok(())
    }

    fn pop_pi_pair(&mut self, c: &mut ParseControl, map: usize) -> Result<(), SessionError> {
        c.charge(WorkCategory::RecordVisit, 1)?;
        payload_write::<[StringLocation; 2]>(c, 2)?;
        let RetainedPayload::PiAttributes { values, .. } = &mut self.payloads.values[map] else {
            return Err(SessionError::WrongToken);
        };
        let pair = values.try_pop(c)?.ok_or(SessionError::WrongToken)?;
        self.bind_string(pair.name_slot, StringLocation::Detached)
            .map_err(|_| latch_registry_row(c))?;
        self.bind_string(pair.value_slot, StringLocation::Detached)
            .map_err(|_| latch_registry_row(c))?;
        let PseudoAttribute {
            name,
            value,
            name_slot,
            value_slot,
        } = pair;
        drop(name);
        drop(value);
        self.retire_raw(c, name_slot)?;
        self.retire_raw(c, value_slot)?;
        Ok(())
    }
    pub(super) fn pi_attribute<'a>(
        &'a self,
        c: &mut ParseControl,
        node: usize,
        name: &str,
    ) -> Result<Option<&'a DocumentString>, SessionError> {
        for payload in &self.payloads.values {
            c.charge(WorkCategory::RecordVisit, 1)?;
            if let RetainedPayload::PiAttributes {
                node: Some(bound),
                values,
                ready,
            } = payload
                && *bound == node
            {
                if !ready {
                    return Err(SessionError::ProcessingInstructionMapUnavailable);
                }
                for pair in values.as_slice() {
                    c.charge(WorkCategory::RecordVisit, 1)?;
                    if exact_ascii(c, Some(&pair.name), name)? {
                        return Ok(Some(&pair.value));
                    }
                }
                return Ok(None);
            }
        }
        Err(SessionError::ProcessingInstructionMapUnavailable)
    }
}
impl FrozenArena {
    #[cfg(test)]
    pub(crate) fn processing_instruction_attribute<'a>(
        &'a self,
        c: &mut ParseControl,
        node: &HtmlNodeId,
        name: &DocumentString,
    ) -> Result<Option<&'a DocumentString>, SessionError> {
        let storage = self.storage.as_ref().ok_or(SessionError::Consumed)?;
        c.require_identity(&storage.control)?;
        let arena = &storage.view;
        c.charge(WorkCategory::ProviderCall, 1)?;
        c.charge(WorkCategory::RecordVisit, 1)?;
        let position = arena
            .node_by_result_id(node)
            .map_err(|_| SessionError::ForeignNode)?
            .position();
        if !matches!(
            arena.node_at(position).unwrap().kind(),
            HtmlNodeKind::ProcessingInstruction(_)
        ) {
            return Err(SessionError::WrongToken);
        }
        for payload in &storage.payloads.values {
            c.charge(WorkCategory::RecordVisit, 1)?;
            if let RetainedPayload::PiAttributes {
                node: Some(bound),
                values,
                ready,
            } = payload
                && *bound == position
            {
                if !ready {
                    return Err(SessionError::ProcessingInstructionMapUnavailable);
                }
                for pair in values.as_slice() {
                    c.charge(WorkCategory::RecordVisit, 1)?;
                    if exact_string_equal(c, &pair.name, name)? {
                        return Ok(Some(&pair.value));
                    }
                }
                return Ok(None);
            }
        }
        Err(SessionError::ProcessingInstructionMapUnavailable)
    }
}
