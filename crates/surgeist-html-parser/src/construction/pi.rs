//! DOM PI pseudo-attributes. XML-stylesheet 20101028 §3, XML 1.0 Fifth Edition
//! productions 2–5 and 66 define this whole-string operation, not HTML recovery.
use super::*;
#[cfg(test)]
use surgeist_html::construction::HtmlNodeId;
use surgeist_html::{construction::ConstructionNodeKind, values::DocumentString};

pub(super) struct PseudoAttribute {
    pub(super) name: DocumentString,
    pub(super) value: DocumentString,
    name_slot: usize,
    value_slot: usize,
}

struct PendingField {
    buffer: Utf16Buffer,
    maps: OwnedVec<HtmlParseTextOrigin>,
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
        // Both owners are released; the buffer's failure is returned first.
        let buffer = self.buffer.release(c).map_err(SessionError::from);
        let maps = self.maps.release(c).map_err(SessionError::from);
        buffer.and(maps)
    }
}
impl SessionStorage {
    fn raw_value(&self, c: &mut ParseControl, slot: usize) -> Result<DocumentString, SessionError> {
        c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        let RetainedPayload::String {
            value, allocation, ..
        } = self.payloads.row(c, slot)?
        else {
            return Err(SessionError::WrongToken);
        };
        c.require_document_string(allocation, value)?;
        c.charge(HtmlParseWorkCategory::ProviderCall, 1)?;
        payload_write::<DocumentString>(c, 1)?;
        Ok(value.clone())
    }
    fn retire_raw(&mut self, c: &mut ParseControl, slot: usize) -> Result<(), SessionError> {
        self.retire_registered_string(c, slot)
    }
    pub(super) fn prepare_pi_map(&mut self, c: &mut ParseControl) -> Result<usize, SessionError> {
        let values = OwnedVec::new(HtmlParseResource::AuxiliaryEntries)?;
        let prepared = self.prepare_payload_slot_owned(c)?;
        self.payloads
            .push_row(
                prepared,
                RetainedPayload::PiAttributes {
                    node: None,
                    values,
                    ready: false,
                },
            )
            .map_err(|_| latch_registry_row(c))
    }
    pub(super) fn bind_pi_map(
        &mut self,
        c: &mut ParseControl,
        map: usize,
        node: usize,
    ) -> Result<(), SessionError> {
        let RetainedPayload::PiAttributes { node: bound, .. } = self.payloads.row_mut(c, map)?
        else {
            return Err(latch_registry_row(c));
        };
        *bound = Some(node);
        self.payloads
            .join_row(map)
            .map_err(|_| latch_registry_row(c))?;
        Ok(())
    }

    pub(super) fn parse_pi_map(
        &mut self,
        c: &mut ParseControl,
        node: usize,
        map: usize,
    ) -> Result<(), SessionError> {
        use surgeist_html::bounded::lexical::{
            HtmlPiPseudoAttributeAction as Action,
            HtmlPiPseudoAttributeCursor as HtmlParsePendingAssociationCursor,
            HtmlPiPseudoAttributeField as Field, HtmlPiPseudoAttributeName as Name,
            HtmlPiPseudoAttributeReply as Reply,
        };
        use surgeist_html::bounded::values::{
            HtmlNameIndex as NameIndex, HtmlNameIndexError as IndexError,
            HtmlNameIndexHasher as NameHasher, HtmlNameIndexLookup as Lookup,
            HtmlNameIndexSlot as NameSlot, HtmlNameIndexTable as NameTable,
        };
        c.charge(HtmlParseWorkCategory::RecordVisit, 2)?;
        let ConstructionNodeKind::ProcessingInstruction(pi) = &self
            .construction
            .node_at(node)
            .ok_or_else(|| latch_registry_row(c))?
            .kind()
        else {
            return Err(SessionError::WrongToken);
        };
        let data_len = pi.data().len();
        let mut cursor = HtmlParsePendingAssociationCursor::new(data_len);
        let mut name: Option<PendingField> = None;
        let mut value: Option<PendingField> = None;
        // The index of committed names: a table at most half full of pair numbers plus one,
        // probed from a randomly keyed hash of the name, so that no input can choose names that
        // collide. It exists once the first name is complete and is released with the fields.
        let hasher = NameHasher::new();
        let mut name_index = NameIndex::new(HtmlParsePendingAssociationCursor::name_index_slots(
            data_len,
        ));
        let mut table: Option<OwnedVec<NameSlot>> = None;
        let mut origins = provenance::PiOriginWalk::new(node);
        let result = (|| {
            while let Some(action) = cursor.action() {
                payload_write::<HtmlParsePendingAssociationCursor>(c, 1)?;
                let reply = match action {
                    Action::Read(offset) => {
                        c.charge(HtmlParseWorkCategory::RecordVisit, 2)?;
                        let ConstructionNodeKind::ProcessingInstruction(pi) = &self
                            .construction
                            .node_at(node)
                            .ok_or_else(|| latch_registry_row(c))?
                            .kind()
                        else {
                            return Err(SessionError::WrongToken);
                        };
                        let unit = match pi.data().code_units().get(offset) {
                            Some(&unit) => {
                                c.charge_unit_inspection()?;
                                Some(unit)
                            }
                            None if offset == pi.data().len() => None,
                            None => return Err(latch_registry_row(c)),
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
                        pending.buffer.extend_units(
                            units.get(..len).ok_or_else(|| latch_registry_row(c))?,
                            c,
                        )?;
                        self.pi_piece_origins(
                            c,
                            &mut origins,
                            start..end,
                            begin..pending.buffer.len(),
                            reference,
                            &mut pending.maps,
                        )?;
                        Reply::Ack
                    }
                    Action::FindName => {
                        let slots = name_index.slots();
                        if table.is_none() {
                            table = Some(OwnedVec::new(HtmlParseResource::AuxiliaryEntries)?);
                            for _ in 0..slots.get() {
                                table
                                    .as_mut()
                                    .ok_or(SessionError::WrongToken)?
                                    .try_push_geometric_toward(
                                        c,
                                        NameSlot::default(),
                                        slots.get(),
                                    )?;
                            }
                        }
                        let current = name
                            .as_ref()
                            .ok_or(SessionError::WrongToken)?
                            .buffer
                            .units();
                        for _ in current {
                            c.charge_unit_inspection()?;
                        }
                        let RetainedPayload::PiAttributes { values, .. } =
                            self.payloads.row(c, map)?
                        else {
                            return Err(SessionError::WrongToken);
                        };
                        let bank = table
                            .as_mut()
                            .ok_or(SessionError::WrongToken)?
                            .as_mut_slice(c)?;
                        let mut index = NameTable::new(&mut name_index, bank)
                            .map_err(|_| latch_registry_row(c))?;
                        let lookup = index.find_or_insert(
                            hasher.hash(current),
                            values.len(),
                            c,
                            |c| Ok(c.charge(HtmlParseWorkCategory::RecordVisit, 1)?),
                            |c, pair| {
                                let existing = values
                                    .as_slice()
                                    .get(pair)
                                    .ok_or(SessionError::WrongToken)?
                                    .name
                                    .code_units();
                                if existing.len() != current.len() {
                                    return Ok(false);
                                }
                                for (left, right) in existing.iter().zip(current) {
                                    c.charge_unit_inspection()?;
                                    c.charge_unit_inspection()?;
                                    if left != right {
                                        return Ok(false);
                                    }
                                }
                                Ok(true)
                            },
                        );
                        Reply::Pair(match lookup {
                            Ok(Lookup::Found(pair)) => Some(pair),
                            Ok(Lookup::Inserted) => None,
                            Err(IndexError::Consumer(error)) => return Err(error),
                            Err(_) => return Err(latch_registry_row(c)),
                        })
                    }
                    Action::NameLength(index) => {
                        c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
                        let RetainedPayload::PiAttributes { values, .. } =
                            self.payloads.row(c, map)?
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
                        c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
                        let units = match which {
                            Name::Current => name
                                .as_ref()
                                .ok_or(SessionError::WrongToken)?
                                .buffer
                                .units(),
                            Name::Pair(index) => {
                                let RetainedPayload::PiAttributes { values, .. } =
                                    self.payloads.row(c, map)?
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
                        let Some(&unit) = units.get(offset) else {
                            return Err(SessionError::WrongToken);
                        };
                        c.charge_unit_inspection()?;
                        Reply::Unit(Some(unit))
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
                            self.payloads.row_mut(c, map)?
                        else {
                            return Err(SessionError::WrongToken);
                        };
                        *ready = true;
                        Reply::Ack
                    }
                };
                cursor.reply(reply).map_err(|e| match e {
                    surgeist_html::bounded::lexical::HtmlPiPseudoAttributeError::Integrity => {
                        SessionError::WrongToken
                    }
                    surgeist_html::bounded::lexical::HtmlPiPseudoAttributeError::Representation => {
                        SessionError::Stop(
                            c.stop_representation(HtmlParseResource::AuxiliaryEntries),
                        )
                    }
                })?;
            }
            Ok(())
        })();
        // Only actual transient owners are reclaimed after a fixed terminal stop.
        // Every release runs; the first one that fails is the error returned.
        let mut released = Ok(());
        if let Some(field) = name {
            released = field.release(c);
        }
        if let Some(field) = value {
            let next = field.release(c);
            released = released.and(next);
        }
        if let Some(bank) = table {
            let next = bank.release(c).map_err(SessionError::from);
            released = released.and(next);
        }
        let next = origins.release(c);
        released = released.and(next);
        released?;
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
        let RetainedPayload::PiAttributes { values, .. } = self.payloads.row_mut(c, map)? else {
            return Err(latch_registry_row(c));
        };
        let index = values.len();
        if let Err((stop, pair)) = values.try_push_recover_geometric(
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
        c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        payload_write::<[StringLocation; 2]>(c, 2)?;
        let RetainedPayload::PiAttributes { values, .. } = self.payloads.row_mut(c, map)? else {
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
        if let Some(slot) = self
            .payloads
            .joined(c, node, Join::ProcessingInstructionAttributes)?
        {
            c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
            if let Some(RetainedPayload::PiAttributes {
                node: Some(bound),
                values,
                ready,
            }) = self.payloads.values.get(slot)
                && *bound == node
            {
                if !ready {
                    return Err(SessionError::ProcessingInstructionMapUnavailable);
                }
                for pair in values.as_slice() {
                    c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
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
        c.charge(HtmlParseWorkCategory::ProviderCall, 1)?;
        c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        let position = arena
            .node_by_result_id(node)
            .map_err(|_| SessionError::ForeignNode)?
            .position();
        if !matches!(
            arena.node_at(position).unwrap().kind(),
            ConstructionNodeKind::ProcessingInstruction(_)
        ) {
            return Err(SessionError::WrongToken);
        }
        for payload in &storage.payloads.values {
            c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
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
                    c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
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
