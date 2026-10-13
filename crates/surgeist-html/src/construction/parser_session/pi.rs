//! DOM PI pseudo-attributes. XML-stylesheet 20101028 §3, XML 1.0 Fifth Edition
//! productions 2–5 and 66 define this whole-string operation, not HTML recovery.
use super::*;

pub(super) struct PseudoAttribute {
    pub(super) name: DocumentString,
    pub(super) value: DocumentString,
    name_slot: usize,
    value_slot: usize,
}

fn xml_char(cp: u32) -> bool {
    matches!(cp, 9|10|13|0x20..=0xd7ff|0xe000..=0xfffd|0x10000..=0x10ffff)
}
fn name_start(cp: u32) -> bool {
    matches!(cp, 0x3a|0x41..=0x5a|0x5f|0x61..=0x7a|0xc0..=0xd6|0xd8..=0xf6|
        0xf8..=0x2ff|0x370..=0x37d|0x37f..=0x1fff|0x200c..=0x200d|
        0x2070..=0x218f|0x2c00..=0x2fef|0x3001..=0xd7ff|0xf900..=0xfdcf|
        0xfdf0..=0xfffd|0x10000..=0xeffff)
}
fn name_char(cp: u32) -> bool {
    name_start(cp) || matches!(cp,0x2d|0x2e|0x30..=0x39|0xb7|0x300..=0x36f|0x203f..=0x2040)
}
fn whitespace(cp: u32) -> bool {
    matches!(cp, 9 | 10 | 13 | 32)
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
    fn pi_scalar(
        &self,
        c: &mut ParseControl,
        node: usize,
        offset: usize,
    ) -> Result<Option<(u32, usize)>, SessionError> {
        c.charge(WorkCategory::RecordVisit, 2)?;
        let HtmlNodeKind::ProcessingInstruction(pi) = &self.records[node].kind else {
            return Err(SessionError::WrongToken);
        };
        let units = pi.data().code_units();
        if offset >= units.len() {
            return Ok(None);
        }
        c.charge(WorkCategory::InspectByte, 2)?;
        let first = units[offset];
        if (0xd800..=0xdbff).contains(&first) && offset + 1 < units.len() {
            c.charge(WorkCategory::InspectByte, 2)?;
            let second = units[offset + 1];
            if (0xdc00..=0xdfff).contains(&second) {
                return Ok(Some((
                    0x10000 + ((u32::from(first) - 0xd800) << 10) + u32::from(second) - 0xdc00,
                    2,
                )));
            }
        }
        Ok(Some((u32::from(first), 1)))
    }
    fn pi_skip_space(
        &self,
        c: &mut ParseControl,
        node: usize,
        offset: &mut usize,
    ) -> Result<bool, SessionError> {
        let start = *offset;
        while let Some((cp, width)) = self.pi_scalar(c, node, *offset)? {
            if !whitespace(cp) {
                break;
            }
            payload_write::<usize>(c, 1)?;
            *offset += width;
        }
        Ok(*offset != start)
    }
    fn pi_reference(
        &self,
        c: &mut ParseControl,
        node: usize,
        offset: &mut usize,
    ) -> Result<Option<u32>, SessionError> {
        payload_write::<usize>(c, 1)?;
        *offset += 1; // actual '&' inspected by the value owner
        if self.pi_scalar(c, node, *offset)?.map(|p| p.0) == Some(35) {
            payload_write::<usize>(c, 1)?;
            *offset += 1;
            let radix = if self.pi_scalar(c, node, *offset)?.map(|p| p.0) == Some(120) {
                payload_write::<usize>(c, 1)?;
                *offset += 1;
                16u32
            } else {
                10u32
            };
            let mut value = 0u32;
            let mut digits = 0usize;
            loop {
                let Some((cp, width)) = self.pi_scalar(c, node, *offset)? else {
                    return Ok(None);
                };
                if cp == 59 {
                    payload_write::<usize>(c, 1)?;
                    *offset += width;
                    return Ok((digits != 0 && xml_char(value)).then_some(value));
                }
                let digit = match cp {
                    48..=57 => cp - 48,
                    65..=70 if radix == 16 => cp - 65 + 10,
                    97..=102 if radix == 16 => cp - 97 + 10,
                    _ => return Ok(None),
                };
                let Some(next) = value.checked_mul(radix).and_then(|v| v.checked_add(digit)) else {
                    return Ok(None);
                };
                payload_write::<(u32, usize, usize)>(c, 3)?;
                value = next;
                digits += 1;
                *offset += width;
            }
        }
        // The closed five names are the complete external grammar alternatives.
        for (name, cp) in [
            (b"amp;".as_slice(), 38),
            (b"lt;".as_slice(), 60),
            (b"gt;".as_slice(), 62),
            (b"quot;".as_slice(), 34),
            (b"apos;".as_slice(), 39),
        ] {
            let mut matched = true;
            for (index, byte) in name.iter().enumerate() {
                c.charge(WorkCategory::InspectByte, 1)?;
                if self.pi_scalar(c, node, *offset + index)?.map(|p| p.0) != Some(u32::from(*byte))
                {
                    matched = false;
                    break;
                }
            }
            if matched {
                payload_write::<usize>(c, 1)?;
                *offset += name.len();
                return Ok(Some(cp));
            }
        }
        Ok(None)
    }
    fn parse_pi_pair(
        &mut self,
        c: &mut ParseControl,
        node: usize,
        map: usize,
        offset: &mut usize,
    ) -> Result<bool, SessionError> {
        let mut name = Utf16Buffer::new(c)?;
        let mut name_maps = OwnedVec::new(HtmlParseResource::SourceMapEntries)?;
        let parsed = (|| {
            let Some((cp, width)) = self.pi_scalar(c, node, *offset)? else {
                return Ok(false);
            };
            if !name_start(cp) {
                return Ok(false);
            }
            append_scalar(&mut name, c, cp)?;
            self.pi_piece_origins(
                c,
                node,
                *offset..*offset + width,
                0..name.len(),
                false,
                &mut name_maps,
            )?;
            payload_write::<usize>(c, 1)?;
            *offset += width;
            while let Some((cp, width)) = self.pi_scalar(c, node, *offset)? {
                if !name_char(cp) {
                    break;
                }
                let begin = name.len();
                append_scalar(&mut name, c, cp)?;
                self.pi_piece_origins(
                    c,
                    node,
                    *offset..*offset + width,
                    begin..name.len(),
                    false,
                    &mut name_maps,
                )?;
                payload_write::<usize>(c, 1)?;
                *offset += width;
            }
            self.pi_skip_space(c, node, offset)?;
            if self.pi_scalar(c, node, *offset)?.map(|p| p.0) != Some(61) {
                return Ok(false);
            }
            payload_write::<usize>(c, 1)?;
            *offset += 1;
            self.pi_skip_space(c, node, offset)?;
            Ok::<_, SessionError>(true)
        })();
        match parsed {
            Ok(true) => {}
            result => {
                name.release(c)?;
                name_maps.release(c)?;
                return result;
            }
        }
        let mut value = match Utf16Buffer::new(c) {
            Ok(v) => v,
            Err(stop) => {
                name.release(c)?;
                name_maps.release(c)?;
                return Err(stop.into());
            }
        };
        let mut value_maps = OwnedVec::new(HtmlParseResource::SourceMapEntries)?;
        let parsed = (|| {
            let Some((quote, _)) = self.pi_scalar(c, node, *offset)? else {
                return Ok(false);
            };
            if !matches!(quote, 34 | 39) {
                return Ok(false);
            }
            payload_write::<usize>(c, 1)?;
            *offset += 1;
            loop {
                let Some((cp, width)) = self.pi_scalar(c, node, *offset)? else {
                    return Ok(false);
                };
                if cp == quote {
                    payload_write::<usize>(c, 1)?;
                    *offset += width;
                    return Ok(true);
                }
                if cp == 38 {
                    let source_begin = *offset;
                    let output_begin = value.len();
                    let Some(cp) = self.pi_reference(c, node, offset)? else {
                        return Ok(false);
                    };
                    append_scalar(&mut value, c, cp)?;
                    self.pi_piece_origins(
                        c,
                        node,
                        source_begin..*offset,
                        output_begin..value.len(),
                        true,
                        &mut value_maps,
                    )?;
                } else {
                    if cp == 60 || !xml_char(cp) {
                        return Ok(false);
                    }
                    let output_begin = value.len();
                    append_scalar(&mut value, c, cp)?;
                    self.pi_piece_origins(
                        c,
                        node,
                        *offset..*offset + width,
                        output_begin..value.len(),
                        false,
                        &mut value_maps,
                    )?;
                    payload_write::<usize>(c, 1)?;
                    *offset += width;
                }
            }
        })();
        match parsed {
            Ok(true) => {}
            result => {
                name.release(c)?;
                value.release(c)?;
                name_maps.release(c)?;
                value_maps.release(c)?;
                return result;
            }
        }
        // Duplicate exact names invalidate the whole map, not only this pair.
        let duplicate = (|| -> Result<bool, SessionError> {
            let RetainedPayload::PiAttributes { values, .. } = &self.payloads.values[map] else {
                unreachable!()
            };
            for pair in values.as_slice() {
                c.charge(WorkCategory::RecordVisit, 1)?;
                if pi_units_equal(c, pair.name.code_units(), name.units())? {
                    return Ok(true);
                }
            }
            Ok(false)
        })();
        match duplicate {
            Ok(false) => {}
            result => {
                name.release(c)?;
                value.release(c)?;
                name_maps.release(c)?;
                value_maps.release(c)?;
                return result.map(|_| false);
            }
        }
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
        self.bind_string(name_slot, StringLocation::PiName(map, index));
        self.bind_string(value_slot, StringLocation::PiValue(map, index));
        Ok(true)
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
        let mut offset = 0usize;
        let valid = (|| -> Result<bool, SessionError> {
            self.pi_skip_space(c, node, &mut offset)?;
            if self.pi_scalar(c, node, offset)?.is_none() {
                return Ok(true);
            }
            loop {
                if !self.parse_pi_pair(c, node, map, &mut offset)? {
                    return Ok(false);
                }
                let separated = self.pi_skip_space(c, node, &mut offset)?;
                if self.pi_scalar(c, node, offset)?.is_none() {
                    return Ok(true);
                }
                if !separated {
                    return Ok(false);
                }
            }
        })()?;
        if !valid {
            self.clear_pi_map(c, map)?;
        }
        payload_write::<bool>(c, 1)?;
        let RetainedPayload::PiAttributes { ready, .. } = &mut self.payloads.values[map] else {
            unreachable!()
        };
        *ready = true;
        Ok(())
    }
    fn clear_pi_map(&mut self, c: &mut ParseControl, map: usize) -> Result<(), SessionError> {
        loop {
            c.charge(WorkCategory::RecordVisit, 1)?;
            let RetainedPayload::PiAttributes { values, .. } = &self.payloads.values[map] else {
                unreachable!()
            };
            if values.len() != 0 {
                payload_write::<[StringLocation; 2]>(c, 2)?;
            }
            let RetainedPayload::PiAttributes { values, .. } = &mut self.payloads.values[map]
            else {
                unreachable!()
            };
            let Some(pair) = values.try_pop(c)? else {
                return Ok(());
            };
            self.bind_string(pair.name_slot, StringLocation::Detached);
            self.bind_string(pair.value_slot, StringLocation::Detached);
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
        }
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
fn pi_units_equal(c: &mut ParseControl, left: &[u16], right: &[u16]) -> Result<bool, SessionError> {
    c.charge(WorkCategory::RecordVisit, 2)?;
    if left.len() != right.len() {
        return Ok(false);
    }
    for index in 0..left.len() {
        c.charge(WorkCategory::InspectByte, 4)?;
        if left[index] != right[index] {
            return Ok(false);
        }
    }
    Ok(true)
}
fn append_scalar(
    buffer: &mut Utf16Buffer,
    c: &mut ParseControl,
    cp: u32,
) -> Result<(), SessionError> {
    if cp <= 0xffff {
        buffer.push_unit(cp as u16, c)?;
    } else {
        let value = cp - 0x10000;
        buffer.extend_units(
            &[
                0xd800 | ((value >> 10) as u16),
                0xdc00 | ((value & 0x3ff) as u16),
            ],
            c,
        )?;
    }
    Ok(())
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
        let arena = match &storage.view {
            FrozenView::Document(d) => &d.arena,
            FrozenView::Fragment(f) => &f.arena,
        };
        c.charge(WorkCategory::ProviderCall, 1)?;
        c.charge(WorkCategory::RecordVisit, 1)?;
        if !Arc::ptr_eq(&node.identity, &arena.identity) || node.index >= arena.records.len() {
            return Err(SessionError::ForeignNode);
        }
        if !matches!(
            arena.records[node.index].kind,
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
                && *bound == node.index
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
