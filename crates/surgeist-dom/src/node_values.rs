use std::sync::atomic::AtomicBool;

use surgeist_html::bounded::lexical::{
    HtmlPiPseudoAttributeAction as Action, HtmlPiPseudoAttributeCursor as Cursor,
    HtmlPiPseudoAttributeError as CursorError, HtmlPiPseudoAttributeField as Field,
    HtmlPiPseudoAttributeName as Name, HtmlPiPseudoAttributeReply as Reply, is_xml_name_code_point,
    is_xml_name_start_code_point,
};
use surgeist_html::bounded::values::{
    HtmlNameIndex as NameIndex, HtmlNameIndexError as IndexError,
    HtmlNameIndexHasher as NameHasher, HtmlNameIndexLookup as Lookup,
    HtmlNameIndexSlot as NameSlot, HtmlNameIndexTable as NameTable,
};

use crate::store::source::{
    formation::{self, Formation},
    storage::Payload,
};
use crate::{DomError, store::SharedVec};
use std::mem::size_of;
use surgeist_html::values::DocumentString;

/// Canonical PI target, CharacterData and the map derived from that exact data.
/// Construction applies DOM's XML Name rule, independently of HTML authoring.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DomProcessingInstruction {
    target: DocumentString,
    data: DocumentString,
    attributes: SharedVec<(DocumentString, DocumentString)>,
}

impl DomProcessingInstruction {
    #[cfg(test)]
    pub(crate) fn inspect_compact_baseline(
        &self,
        graph: &mut crate::store::compact_tests::AllocationGraph,
    ) {
        graph.string(&self.target);
        graph.string(&self.data);
        self.attributes.inspect_named_allocations(
            graph,
            "pi.attributes.target",
            "pi.attributes.buffer",
        );
        for (name, value) in self.attributes.iter() {
            graph.string(name);
            graph.string(value);
        }
    }

    pub(crate) fn retained_payload(
        &self,
        mut step: impl FnMut() -> Result<(), DomError>,
    ) -> Result<crate::store::source::storage::Payload, DomError> {
        let mut payload = crate::store::source::storage::Payload::default();
        payload.string(&self.target)?;
        payload.string(&self.data)?;
        payload.add_bytes(self.attributes.retained_bytes()?)?;
        for (name, value) in self.attributes.iter() {
            step()?;
            payload.string(name)?;
            payload.string(value)?;
        }
        Ok(payload)
    }
    pub(crate) fn from_parsed(
        target: DocumentString,
        data: DocumentString,
        attributes: Vec<(DocumentString, DocumentString)>,
    ) -> Self {
        let result = Self {
            target,
            data,
            attributes: SharedVec::from(attributes),
        };
        #[cfg(test)]
        if crate::store::scale_metrics::ENABLED && result.attributes.has_backing() {
            crate::store::empty_sequence_observation::allocated::<(DocumentString, DocumentString)>(
                crate::store::empty_sequence_observation::Kind::PiAttributes,
                result.attributes.capacity(),
            );
        }
        result
    }

    pub(crate) fn new(
        target: &DocumentString,
        data: &DocumentString,
        budget: Option<&Formation>,
        cancelled: &AtomicBool,
    ) -> Result<Self, DomError> {
        let valid_target = if budget.is_some() {
            let mut stopped = None;
            let mut input = target.code_units().iter();
            let units = std::iter::from_fn(|| {
                if stopped.is_some() || input.as_slice().is_empty() {
                    return None;
                }
                if let Err(error) = formation::step(budget, cancelled) {
                    stopped = Some(error);
                    return None;
                }
                input.next().copied()
            });
            let valid = valid_target(char::decode_utf16(units).map(|point| match point {
                Ok(point) => u32::from(point),
                Err(error) => u32::from(error.unpaired_surrogate()),
            }));
            if let Some(error) = stopped {
                return Err(error);
            }
            valid
        } else {
            valid_target(target.code_points())
        };
        if !valid_target {
            return Err(DomError::InvalidCharacter);
        }
        if budget.is_some() {
            let mut question = false;
            for unit in data.code_units() {
                formation::step(budget, cancelled)?;
                let unit = *unit;
                if question && unit == 0x3e {
                    return Err(DomError::InvalidCharacter);
                }
                question = unit == 0x3f;
            }
        } else if data
            .code_units()
            .windows(2)
            .any(|pair| pair == [0x3f, 0x3e])
        {
            return Err(DomError::InvalidCharacter);
        }
        Self::from_data(target, data, budget, cancelled)
    }

    /// CharacterData replacement does not rerun PI initialization's syntax
    /// rejection; even data containing ?> is retained and its map recomputed.
    pub(crate) fn with_data(
        &self,
        data: &DocumentString,
        budget: Option<&Formation>,
        cancelled: &AtomicBool,
    ) -> Result<Self, DomError> {
        Self::from_data(&self.target, data, budget, cancelled)
    }

    fn from_data(
        target: &DocumentString,
        data: &DocumentString,
        budget: Option<&Formation>,
        cancelled: &AtomicBool,
    ) -> Result<Self, DomError> {
        let target = formation::share(target, budget, cancelled)?;
        let data = formation::share(data, budget, cancelled)?;
        let attributes = parse_attributes(&data, budget, cancelled)?;
        if let Some(budget) = budget
            && attributes.capacity() != 0
        {
            budget.retain_bytes(
                size_of::<Vec<(DocumentString, DocumentString)>>(),
                cancelled,
            )?;
        }
        let result = Self {
            target,
            data,
            attributes: SharedVec::from(attributes),
        };
        #[cfg(test)]
        if crate::store::scale_metrics::ENABLED && result.attributes.has_backing() {
            crate::store::empty_sequence_observation::allocated::<(DocumentString, DocumentString)>(
                crate::store::empty_sequence_observation::Kind::PiAttributes,
                result.attributes.capacity(),
            );
        }
        Ok(result)
    }

    pub fn target(&self) -> &DocumentString {
        &self.target
    }
    pub fn data(&self) -> &DocumentString {
        &self.data
    }
    /// Whole-string XML pseudo-attributes, in source order. Invalid data has an
    /// empty map; it is never replaced by partially parsed successful fields.
    pub fn attributes(&self) -> impl ExactSizeIterator<Item = (&DocumentString, &DocumentString)> {
        self.attributes.iter().map(|(name, value)| (name, value))
    }
    pub fn attribute(&self, name: &DocumentString) -> Option<&DocumentString> {
        self.attribute_with(name.code_units(), || Ok::<_, std::convert::Infallible>(()))
            .unwrap_or_else(|never| match never {})
            .map(|(_, value)| value)
    }
    /// Find an exact pseudo-attribute, charging each inspected entry and
    /// compared UTF-16 unit. The index is in the canonical map order.
    pub fn attribute_with<E>(
        &self,
        name: &[u16],
        mut step: impl FnMut() -> Result<(), E>,
    ) -> Result<Option<(usize, &DocumentString)>, E> {
        for (index, (key, value)) in self.attributes().enumerate() {
            step()?;
            if key.len() != name.len() {
                continue;
            }
            let mut equal = true;
            for (left, right) in key.code_units().iter().zip(name) {
                step()?;
                if left != right {
                    equal = false;
                    break;
                }
            }
            if equal {
                return Ok(Some((index, value)));
            }
        }
        Ok(None)
    }
    pub(crate) fn with_authored_attributes(
        &self,
        attributes: Vec<(DocumentString, DocumentString)>,
        limit: usize,
        budget: Option<&Formation>,
        cancel: &AtomicBool,
    ) -> Result<Self, DomError> {
        let mut length = attributes.len().saturating_sub(1);
        for (name, value) in &attributes {
            formation::step(budget, cancel)?;
            length = length
                .checked_add(name.len())
                .and_then(|n| n.checked_add(3))
                .ok_or(DomError::RepresentationExhausted)?;
            for &unit in value.code_units() {
                formation::step(budget, cancel)?;
                length = length
                    .checked_add(pi_escape(unit).map_or(1, |escape| escape.len()))
                    .ok_or(DomError::RepresentationExhausted)?;
            }
        }
        let mut data = formation::utf16_buffer(length, limit, budget, cancel)?;
        for (name, value) in &attributes {
            if !data.is_empty() {
                formation::step(budget, cancel)?;
                data.push(0x20);
            }
            for &unit in name.code_units() {
                formation::step(budget, cancel)?;
                data.push(unit);
            }
            formation::step(budget, cancel)?;
            data.extend([0x3d, 0x22]);
            for &unit in value.code_units() {
                formation::step(budget, cancel)?;
                if let Some(escape) = pi_escape(unit) {
                    data.extend(escape.iter().copied().map(u16::from));
                } else {
                    data.push(unit);
                }
            }
            formation::step(budget, cancel)?;
            data.push(0x22);
        }
        if let Some(budget) = budget
            && attributes.capacity() != 0
        {
            budget.retain_bytes(size_of::<Vec<(DocumentString, DocumentString)>>(), cancel)?;
        }
        Ok(Self {
            target: formation::share(&self.target, budget, cancel)?,
            data: DocumentString::from_utf16(data),
            attributes: SharedVec::from(attributes),
        })
    }
    pub(crate) fn units(&self) -> Option<usize> {
        self.attributes.iter().try_fold(
            self.target.len().checked_add(self.data.len())?,
            |total, (name, value)| total.checked_add(name.len())?.checked_add(value.len()),
        )
    }
}

fn pi_escape(unit: u16) -> Option<&'static [u8]> {
    match unit {
        0x26 => Some(b"&amp;"),
        0x3c => Some(b"&lt;"),
        0x3e => Some(b"&gt;"),
        0x22 => Some(b"&quot;"),
        _ => None,
    }
}

/// Source-defined DocumentType name and exact public/system identifiers.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DomDocumentType {
    name: DocumentString,
    public_id: DocumentString,
    system_id: DocumentString,
}
impl DomDocumentType {
    pub(crate) fn from_parsed(
        name: DocumentString,
        public_id: DocumentString,
        system_id: DocumentString,
    ) -> Self {
        Self {
            name,
            public_id,
            system_id,
        }
    }

    pub(crate) fn new(
        name: &DocumentString,
        public_id: &DocumentString,
        system_id: &DocumentString,
        budget: Option<&Formation>,
        cancelled: &AtomicBool,
    ) -> Result<Self, DomError> {
        // DOM §1.4 deliberately permits the empty name and non-XML spellings.
        for unit in name.code_units() {
            if budget.is_some() {
                formation::step(budget, cancelled)?;
            }
            if matches!(*unit, 0 | 9 | 10 | 12 | 13 | 32 | 0x3e) {
                return Err(DomError::InvalidCharacter);
            }
        }
        Ok(Self {
            name: formation::share(name, budget, cancelled)?,
            public_id: formation::share(public_id, budget, cancelled)?,
            system_id: formation::share(system_id, budget, cancelled)?,
        })
    }

    pub fn name(&self) -> &DocumentString {
        &self.name
    }
    pub fn public_id(&self) -> &DocumentString {
        &self.public_id
    }
    pub fn system_id(&self) -> &DocumentString {
        &self.system_id
    }
    pub(crate) fn units(&self) -> Option<usize> {
        self.name
            .len()
            .checked_add(self.public_id.len())?
            .checked_add(self.system_id.len())
    }
}

fn valid_target(mut points: impl Iterator<Item = u32>) -> bool {
    points.next().is_some_and(is_xml_name_start_code_point) && points.all(is_xml_name_code_point)
}

fn pair_payload(pair: &(DocumentString, DocumentString)) -> Result<Payload, DomError> {
    let mut payload = Payload::default();
    payload.string(&pair.0)?;
    payload.string(&pair.1)?;
    Ok(payload)
}

fn release_scratch(buffer: Vec<u16>, budget: Option<&Formation>) -> Result<(), DomError> {
    let payload = Payload {
        bytes: buffer
            .capacity()
            .checked_mul(size_of::<u16>())
            .ok_or(DomError::RepresentationExhausted)?,
        units: buffer.capacity(),
    };
    drop(buffer);
    if let Some(budget) = budget {
        budget.release_payload(payload);
    }
    Ok(())
}

// Cursor requests served by `parse_attributes` on this thread, for work-growth tests.
#[cfg(test)]
thread_local! {
    static PI_REQUESTS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

fn release_table(table: Vec<NameSlot>, budget: Option<&Formation>) -> Result<(), DomError> {
    let payload = Payload {
        bytes: table
            .capacity()
            .checked_mul(size_of::<NameSlot>())
            .ok_or(DomError::RepresentationExhausted)?,
        units: 0,
    };
    drop(table);
    if let Some(budget) = budget {
        budget.release_payload(payload);
    }
    Ok(())
}

type Pair = (DocumentString, DocumentString);

/// The cursor asked about a pair, or a unit of a name, that the driver does not hold.
fn cursor_mismatch() -> DomError {
    DomError::PseudoAttributeCursor(CursorError::Integrity)
}

/// The length of the committed pair `index`'s name.
fn pair_name_length(pairs: &[Pair], index: usize) -> Result<usize, DomError> {
    pairs
        .get(index)
        .map(|(name, _)| name.len())
        .ok_or_else(cursor_mismatch)
}

/// The unit at `offset` of the current name or of the committed pair `which`'s name.
fn name_unit(name: &[u16], pairs: &[Pair], which: Name, offset: usize) -> Result<u16, DomError> {
    match which {
        Name::Current => name.get(offset).copied(),
        Name::Pair(index) => pairs
            .get(index)
            .and_then(|(existing, _)| existing.code_units().get(offset).copied()),
    }
    .ok_or_else(cursor_mismatch)
}

fn parse_attributes(
    data: &DocumentString,
    budget: Option<&Formation>,
    cancelled: &AtomicBool,
) -> Result<Vec<(DocumentString, DocumentString)>, DomError> {
    let mut cursor = Cursor::new(data.len());
    let mut name = Vec::new();
    let mut value = Vec::new();
    #[cfg(test)]
    let mut pi_writes = crate::store::CopyObservation::default();
    let mut pairs: Vec<(DocumentString, DocumentString)> = Vec::new();
    // The index of committed names: a table at most half full of pair numbers plus one, probed from
    // a randomly keyed hash of the name, so that no input can choose names that collide. It is
    // reserved when the first name is complete, under the same custody as the other banks.
    let hasher = NameHasher::new();
    let mut name_index = NameIndex::new(Cursor::name_index_slots(data.len()));
    let slots = name_index.slots();
    let mut table: Vec<NameSlot> = Vec::new();
    while let Some(action) = cursor.action() {
        formation::step(budget, cancelled)?;
        #[cfg(test)]
        PI_REQUESTS.with(|requests| requests.set(requests.get() + 1));
        let reply = match action {
            Action::Read(offset) => Reply::Unit(data.code_units().get(offset).copied()),
            Action::Step => Reply::Ack,
            Action::Begin(field) => {
                match field {
                    Field::Name => name.clear(),
                    Field::Value => value.clear(),
                }
                Reply::Ack
            }
            Action::Emit {
                field, units, len, ..
            } => {
                #[cfg(test)]
                let observed = match field {
                    Field::Name => name.len() + len > name.capacity(),
                    Field::Value => value.len() + len > value.capacity(),
                }
                .then(|| source_observation::before(&name, &value, &pairs, budget.is_some()))
                .flatten();
                let target = match field {
                    Field::Name => &mut name,
                    Field::Value => &mut value,
                };
                formation::reserve_utf16(target, len, budget, cancelled)?;
                #[cfg(test)]
                source_observation::grew(
                    observed,
                    target.capacity() * std::mem::size_of::<u16>(),
                    target.capacity(),
                );
                target.extend_from_slice(units.get(..len).ok_or_else(cursor_mismatch)?);
                #[cfg(test)]
                pi_writes.extended::<u16>(len);
                Reply::Ack
            }
            Action::FindName => {
                if table.is_empty() {
                    #[cfg(test)]
                    let observed =
                        source_observation::before(&name, &value, &pairs, budget.is_some());
                    formation::reserve(&mut table, slots.get(), budget, cancelled)?;
                    for _ in 0..slots.get() {
                        formation::step(budget, cancelled)?;
                    }
                    table.resize_with(slots.get(), NameSlot::default);
                    #[cfg(test)]
                    source_observation::grew(
                        observed,
                        table.capacity() * std::mem::size_of::<NameSlot>(),
                        0,
                    );
                }
                for _ in 0..name.len() {
                    formation::step(budget, cancelled)?;
                }
                let mut index = NameTable::new(&mut name_index, &mut table)
                    .map_err(|_| DomError::RepresentationExhausted)?;
                let lookup = index.find_or_insert(
                    hasher.hash(&name),
                    pairs.len(),
                    &mut (),
                    |_| formation::step(budget, cancelled),
                    |_, pair| {
                        let Some((existing, _)) = pairs.get(pair) else {
                            return Ok(false);
                        };
                        if existing.len() != name.len() {
                            return Ok(false);
                        }
                        for (left, right) in existing.code_units().iter().zip(&name) {
                            formation::step(budget, cancelled)?;
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
                    // The table is sized from the data length, which bounds its pairs.
                    Err(_) => return Err(DomError::RepresentationExhausted),
                })
            }
            Action::NameLength(index) => Reply::Length(pair_name_length(&pairs, index)?),
            Action::NameUnit(which, offset) => {
                Reply::Unit(Some(name_unit(&name, &pairs, which, offset)?))
            }
            Action::Commit => {
                #[cfg(test)]
                let observed = (pairs.len() == pairs.capacity())
                    .then(|| source_observation::before(&name, &value, &pairs, budget.is_some()))
                    .flatten();
                formation::reserve(&mut pairs, 1, budget, cancelled)?;
                #[cfg(test)]
                source_observation::grew(
                    observed,
                    pairs.capacity() * std::mem::size_of::<(DocumentString, DocumentString)>(),
                    0,
                );
                #[cfg(test)]
                let observed = source_observation::before(&name, &value, &pairs, budget.is_some());
                // The buffer allocations keep their existing custody through
                // mem::take. Only their new out-of-line Vec headers are new.
                if let Some(budget) = budget {
                    budget.retain_bytes(2 * size_of::<Vec<u16>>(), cancelled)?;
                }
                pairs.push((
                    DocumentString::from_utf16(std::mem::take(&mut name)),
                    DocumentString::from_utf16(std::mem::take(&mut value)),
                ));
                #[cfg(test)]
                pi_writes.pushed::<(DocumentString, DocumentString)>();
                #[cfg(test)]
                source_observation::grew(observed, 2 * std::mem::size_of::<Vec<u16>>(), 0);
                Reply::Ack
            }
            Action::Discard => {
                name.clear();
                value.clear();
                Reply::Ack
            }
            Action::Pop => {
                let pair = pairs.pop().ok_or_else(cursor_mismatch)?;
                let payload = if budget.is_some() {
                    Some(pair_payload(&pair)?)
                } else {
                    None
                };
                drop(pair);
                if let (Some(budget), Some(payload)) = (budget, payload) {
                    budget.release_payload(payload);
                }
                Reply::Ack
            }
            Action::Ready(valid) => {
                if !valid {
                    if let Some(budget) = budget {
                        while let Some(pair) = pairs.last() {
                            budget.step(cancelled)?;
                            let payload = pair_payload(pair)?;
                            drop(pairs.pop());
                            budget.release_payload(payload);
                        }
                    } else {
                        pairs.clear();
                    }
                }
                Reply::Ack
            }
        };
        match cursor.reply(reply) {
            Ok(()) => {}
            Err(CursorError::Representation) => return Err(DomError::RepresentationExhausted),
            Err(error @ CursorError::Integrity) => {
                return Err(DomError::PseudoAttributeCursor(error));
            }
        }
    }
    // Cleared/discarded buffers still own their capacity until this actual drop.
    // Committed strings and the pairs backing stay with the caller's Formation.
    release_scratch(name, budget)?;
    release_scratch(value, budget)?;
    release_table(table, budget)?;
    Ok(pairs)
}

#[cfg(test)]
pub(crate) mod source_observation {
    use super::*;
    use std::cell::{Cell, RefCell};

    struct Context {
        live: Box<dyn Fn() -> (usize, usize)>,
        limit: usize,
        unit_limit: usize,
    }
    #[derive(Clone, Copy, Debug)]
    pub(crate) struct Growth {
        pub required: usize,
        pub limit: usize,
        pub required_units: usize,
        pub unit_limit: usize,
    }
    pub(crate) struct Binding {
        previous: Option<Context>,
    }
    impl Drop for Binding {
        fn drop(&mut self) {
            CONTEXT.with(|context| *context.borrow_mut() = self.previous.take());
        }
    }
    thread_local! {
        static ACTIVE: Cell<bool> = const { Cell::new(false) };
        static CONTEXT: RefCell<Option<Context>> = const { RefCell::new(None) };
        static LAST: Cell<Option<Growth>> = const { Cell::new(None) };
    }
    pub(crate) fn active() -> bool {
        ACTIVE.get()
    }
    pub(crate) fn start() {
        ACTIVE.set(true);
        LAST.set(None);
    }
    pub(crate) fn finish() -> Option<Growth> {
        ACTIVE.set(false);
        LAST.take()
    }
    pub(crate) fn bind(
        live: impl Fn() -> (usize, usize) + 'static,
        limit: usize,
        unit_limit: usize,
    ) -> Binding {
        let next = ACTIVE.get().then_some(Context {
            live: Box::new(live),
            limit,
            unit_limit,
        });
        let previous = CONTEXT.with(|context| context.replace(next));
        Binding { previous }
    }
    pub(super) fn before(
        name: &Vec<u16>,
        value: &Vec<u16>,
        pairs: &Vec<(DocumentString, DocumentString)>,
        registered: bool,
    ) -> Option<Growth> {
        CONTEXT.with(|context| {
            context.borrow().as_ref().map(|context| {
                let (temporary, units) = if registered {
                    (0, 0)
                } else {
                    let units = name.capacity()
                        + value.capacity()
                        + pairs
                            .iter()
                            .map(|(name, value)| name.retained_units() + value.retained_units())
                            .sum::<usize>();
                    (
                        units * size_of::<u16>()
                            + pairs.capacity() * size_of::<(DocumentString, DocumentString)>()
                            + pairs.len() * 2 * size_of::<Vec<u16>>(),
                        units,
                    )
                };
                let (bytes, live_units) = (context.live)();
                Growth {
                    required: bytes + temporary,
                    limit: context.limit,
                    required_units: live_units + units,
                    unit_limit: context.unit_limit,
                }
            })
        })
    }
    pub(super) fn grew(before: Option<Growth>, new: usize, units: usize) {
        if let Some(mut actual) = before {
            actual.required += new;
            actual.required_units += units;
            if let Some(old) = LAST.get() {
                actual.required = actual.required.max(old.required);
                actual.required_units = actual.required_units.max(old.required_units);
            }
            LAST.set(Some(actual));
        }
    }
}

#[cfg(test)]
mod source_tests {
    use super::*;
    use crate::{Dom, DomBatch, DomLimits, DomSourceLimits};

    #[test]
    fn ordinary_pi_validation_precedes_cancellation_and_replacement_keeps_original_grammar() {
        let cancelled = AtomicBool::new(true);
        for (target, data) in [("1invalid", ""), ("valid", "?>")] {
            assert!(matches!(
                DomProcessingInstruction::new(
                    &DocumentString::from(target),
                    &DocumentString::from(data),
                    None,
                    &cancelled
                ),
                Err(DomError::InvalidCharacter)
            ));
        }
        let active = AtomicBool::new(false);
        let instruction = DomProcessingInstruction::new(
            &DocumentString::from("xml-stylesheet"),
            &DocumentString::from("a='first' b=\"second\""),
            None,
            &active,
        )
        .unwrap();
        let attributes: Vec<_> = instruction
            .attributes()
            .map(|(n, v)| (n.clone(), v.clone()))
            .collect();
        assert_eq!(
            attributes,
            vec![
                (DocumentString::from("a"), DocumentString::from("first")),
                (DocumentString::from("b"), DocumentString::from("second"))
            ]
        );
        let replacement = instruction
            .with_data(&DocumentString::from("a='complete' invalid"), None, &active)
            .unwrap();
        assert_eq!(replacement.target(), instruction.target());
        assert_eq!(
            replacement.data(),
            &DocumentString::from("a='complete' invalid")
        );
        assert_eq!(replacement.attributes().len(), 0);
        let replacement = instruction
            .with_data(&DocumentString::from("?>"), None, &active)
            .unwrap();
        assert_eq!(replacement.data(), &DocumentString::from("?>"));
        assert_eq!(replacement.attributes().len(), 0);
    }
    #[test]
    fn ordinary_doctype_keeps_relaxed_name_and_exact_identifiers() {
        let public = DocumentString::from_utf16(vec![0xd800, 0, 0xdc00]);
        let system = DocumentString::from("system > id");
        for name in ["", "1:not-an-xml-name"] {
            let doctype = DomDocumentType::new(
                &DocumentString::from(name),
                &public,
                &system,
                None,
                &AtomicBool::new(true),
            )
            .unwrap();
            assert_eq!(doctype.name(), &DocumentString::from(name));
            assert_eq!(doctype.public_id(), &public);
            assert_eq!(doctype.system_id(), &system);
        }
        for name in ["white space", "bad>", "bad\0"] {
            assert!(matches!(
                DomDocumentType::new(
                    &DocumentString::from(name),
                    &public,
                    &system,
                    None,
                    &AtomicBool::new(true)
                ),
                Err(DomError::InvalidCharacter)
            ));
        }
    }
    /// XML-stylesheet 20101028 section 3 makes a repeated name an error, so each name is checked
    /// against the earlier ones; the requests that this takes must grow no faster than the pairs.
    #[test]
    fn pseudo_attribute_requests_grow_linearly_with_distinct_names() {
        let requests: Vec<usize> = [250, 500, 1000, 2000]
            .into_iter()
            .map(|pairs| {
                let names: Vec<String> = (0..pairs).map(|n| format!("p{n}=''")).collect();
                let data = DocumentString::from(names.join(" "));
                PI_REQUESTS.with(|requests| requests.set(0));
                let parsed = parse_attributes(&data, None, &AtomicBool::new(false)).unwrap();
                assert_eq!(parsed.len(), pairs);
                PI_REQUESTS.with(std::cell::Cell::get)
            })
            .collect();
        for pair in requests.windows(2) {
            assert!(
                pair[1] * 10 <= pair[0] * 22,
                "doubling the pairs grew the requests from {} to {}: {requests:?}",
                pair[0],
                pair[1]
            );
        }
    }
    #[test]
    fn source_pi_success_and_invalid_cleanup_keep_the_original_cursor_results() {
        let cancel = AtomicBool::new(false);
        let mut dom = Dom::new(
            DomLimits::new(
                32,
                100_000,
                32,
                0,
                0,
                8,
                100_000,
                crate::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
            )
            .unwrap(),
        )
        .unwrap();
        let mut setup = DomBatch::new();
        let document = setup.create_document();
        let created = dom.apply(setup, &cancel).unwrap();
        let document = created.document(&document).unwrap();
        let mut batch = DomBatch::new();
        let valid =
            batch.create_processing_instruction(document, "pi", "a='&amp;&#65;' z='second'");
        let invalid =
            batch.create_processing_instruction(document, "pi", "a='first' b='second' broken");
        let empty = batch.create_processing_instruction(document, "pi", "");
        let public = DocumentString::from_utf16(vec![0xd800, 0, 0xdc00]);
        let doctype = batch.create_document_type(document, "", public.clone(), "system > id");
        let mut lease = dom
            .begin_source_commands(
                batch,
                DomSourceLimits::new(16_000_000, 1_000_000, 10_000_000, 64, 4096, 4096, 1),
                &cancel,
            )
            .unwrap();
        let crate::DomSourceProgress::Complete(done) = lease.advance(&cancel).unwrap() else {
            panic!("no external service is reached");
        };
        let valid = done.commit().created(&valid).unwrap().unwrap();
        let invalid = done.commit().created(&invalid).unwrap().unwrap();
        let empty = done.commit().created(&empty).unwrap().unwrap();
        let doctype = done.commit().created(&doctype).unwrap().unwrap();
        drop(lease);
        let snapshot = dom.snapshot();
        let crate::DomNodeKind::ProcessingInstruction(valid) = snapshot.node(valid).unwrap().kind()
        else {
            panic!("PI");
        };
        assert_eq!(
            valid
                .attributes()
                .map(|(n, v)| (n.clone(), v.clone()))
                .collect::<Vec<_>>(),
            vec![
                (DocumentString::from("a"), DocumentString::from("&A")),
                (DocumentString::from("z"), DocumentString::from("second"))
            ]
        );
        for node in [invalid, empty] {
            let crate::DomNodeKind::ProcessingInstruction(pi) = snapshot.node(node).unwrap().kind()
            else {
                panic!("PI");
            };
            assert_eq!(pi.attributes().len(), 0);
        }
        let crate::DomNodeKind::DocumentType(doctype) = snapshot.node(doctype).unwrap().kind()
        else {
            panic!("doctype");
        };
        assert_eq!(doctype.name(), &DocumentString::default());
        assert_eq!(doctype.public_id(), &public);
        assert_eq!(doctype.system_id(), &DocumentString::from("system > id"));
        assert!(dom.pending_source_edit().is_none());
    }
    struct PiRun {
        growth: Option<source_observation::Growth>,
        error: Option<DomError>,
    }
    fn actual_pi_growth(bytes: usize, units: usize) -> PiRun {
        let cancel = AtomicBool::new(false);
        let mut dom = Dom::new(
            DomLimits::new(
                16,
                100_000,
                32,
                0,
                0,
                8,
                100_000,
                crate::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
            )
            .unwrap(),
        )
        .unwrap();
        let mut setup = DomBatch::new();
        let doc = setup.create_document();
        let created = dom.apply(setup, &cancel).unwrap();
        let doc = created.document(&doc).unwrap();
        let before = dom.snapshot();
        let usage = dom.usage();
        let mut batch = DomBatch::new();
        batch.create_processing_instruction(
            doc,
            "pi",
            format!("a='{}' b='literal'", "x".repeat(4096)),
        );
        let limits = DomSourceLimits::new(bytes, units, 10_000_000, 64, 4096, 4096, 1);
        let mut lease = match dom.begin_source_commands(batch, limits, &cancel) {
            Ok(lease) => lease,
            Err(error) => {
                return PiRun {
                    growth: None,
                    error: Some(error),
                };
            }
        };
        source_observation::start();
        let result = lease.advance(&cancel);
        let observed = source_observation::finish();
        let error = result.err();
        if error.is_some() {
            assert_eq!(lease.snapshot().stamp(), before.stamp());
        }
        drop(lease);
        if error.is_some() {
            assert_eq!(dom.snapshot().stamp(), before.stamp());
            assert_eq!(dom.usage(), usage);
        }
        assert!(dom.pending_source_edit().is_none());
        PiRun {
            growth: observed,
            error,
        }
    }
    #[test]
    fn source_pi_pseudoattributes_admit_actual_old_and_new_banks_before_growth() {
        let high = actual_pi_growth(16_000_000, 1_000_000);
        assert!(
            high.error.is_none(),
            "high allowance must complete: {:?}",
            high.error
        );
        let high = high.growth.expect("actual PI cursor buffer growth");
        let exact = actual_pi_growth(high.required, 1_000_000);
        let admitted = exact
            .growth
            .expect("exact allowance reaches the actual cursor peak");
        // Distinct canonical owners can have different custody radix paths.
        // This public Source check verifies its actual admitted envelope; the
        // factory test prices exact/one-short on one original owner.
        assert!(admitted.required <= admitted.limit);
        assert_eq!(admitted.limit, high.required);
        // As with units, this receipt describes the cursor bank envelope.
        // A later canonical publication may need a greater total allowance.
        assert!(matches!(
            exact.error,
            None | Some(DomError::ResourceLimit {
                resource: crate::DomResource::SourceRetainedBytes,
                ..
            })
        ));
        let short = actual_pi_growth(high.required - 1, 1_000_000);
        assert!(
            matches!(short.error,Some(DomError::ResourceLimit{resource:crate::DomResource::SourceRetainedBytes,limit}) if limit==high.required-1)
        );
        if let Some(actual) = short.growth {
            assert!(
                actual.required <= actual.limit,
                "PI graph grew to {} bytes under {}",
                actual.required,
                actual.limit
            );
        }
    }
    #[test]
    fn source_pi_utf16_cursor_peak_is_admitted_exactly_and_one_short_stops_before_growth() {
        let high = actual_pi_growth(16_000_000, 1_000_000);
        assert!(high.error.is_none());
        let high = high.growth.expect("actual UTF16 output banks");
        assert!(high.required_units > 4096);
        let exact = actual_pi_growth(16_000_000, high.required_units);
        let admitted = exact
            .growth
            .expect("exact allowance reaches the actual cursor peak");
        assert_eq!(admitted.required_units, high.required_units);
        assert_eq!(admitted.unit_limit, high.required_units);
        // This is the cursor's actual bank envelope. Subsequent canonical
        // publication may require a larger whole-operation allowance.
        assert!(matches!(
            exact.error,
            None | Some(DomError::ResourceLimit {
                resource: crate::DomResource::SourceRetainedUnits,
                ..
            })
        ));
        let short = actual_pi_growth(16_000_000, high.required_units - 1);
        assert!(
            matches!(short.error,Some(DomError::ResourceLimit{resource:crate::DomResource::SourceRetainedUnits,limit}) if limit==high.required_units-1)
        );
        if let Some(actual) = short.growth {
            assert!(actual.required_units <= actual.unit_limit);
            assert!(actual.required_units < high.required_units);
        }
    }
}

#[cfg(test)]
mod cursor_mismatch_tests {
    use super::*;

    fn pair(name: &str) -> Pair {
        (DocumentString::from(name), DocumentString::from("v"))
    }

    /// A question about a pair or unit the driver does not hold is a disagreement with the cursor,
    /// reported as the cursor's own integrity cause; one it does hold is answered exactly.
    #[test]
    fn requests_outside_the_held_names_report_the_cursor_integrity_cause() {
        let mismatch = DomError::PseudoAttributeCursor(CursorError::Integrity);
        let pairs = [pair("ab"), pair("c")];
        assert_eq!(pair_name_length(&pairs, 0), Ok(2));
        assert_eq!(pair_name_length(&pairs, 1), Ok(1));
        assert_eq!(pair_name_length(&pairs, 2), Err(mismatch.clone()));
        let current = [u16::from(b'x'), u16::from(b'y')];
        assert_eq!(
            name_unit(&current, &pairs, Name::Current, 1),
            Ok(u16::from(b'y'))
        );
        assert_eq!(
            name_unit(&current, &pairs, Name::Current, 2),
            Err(mismatch.clone())
        );
        assert_eq!(
            name_unit(&current, &pairs, Name::Pair(0), 1),
            Ok(u16::from(b'b'))
        );
        assert_eq!(
            name_unit(&current, &pairs, Name::Pair(0), 2),
            Err(mismatch.clone())
        );
        assert_eq!(name_unit(&current, &pairs, Name::Pair(2), 0), Err(mismatch));
    }
}
