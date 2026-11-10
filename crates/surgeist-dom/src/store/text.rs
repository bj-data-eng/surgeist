//! DOM3071 Text algorithms over the canonical staged tree and live Range bank.
use super::*;

#[derive(Clone, Copy)]
pub(super) struct SplitFrame {
    node: Key,
    new_node: Key,
    parent: Option<Key>,
    offset: usize,
    count: usize,
    inserted: bool,
}

impl Draft {
    pub(super) fn split_text(
        &mut self,
        node: Key,
        offset: usize,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.source_step(cancel)?;
        let original = self.node(node)?;
        let transaction::KindRead::Text(data) = original.kind() else {
            return Err(DomError::WrongNodeKind);
        };
        if offset > data.len() {
            return Err(DomError::IndexSize);
        }
        let count = data.len() - offset;
        let range = data.range();
        let start = range
            .start
            .checked_add(offset)
            .ok_or(DomError::RepresentationExhausted)?;
        let tail = data
            .block()
            .string(start..range.end)
            .map_err(|_| DomError::RepresentationExhausted)?;
        let (tail, payload) = source::formation::shared_string(self, &tail, cancel)?;
        let new_node = self.allocate_charged(
            NewNode::Text(tail),
            original.document(),
            limits,
            cancel,
            payload,
        )?;
        self.register_ticket(new_node, cancel)?;
        let parent = self.node(node)?.parent();
        self.push_effect(
            select::Effect::SplitText(SplitFrame {
                node,
                new_node,
                parent,
                offset,
                count,
                inserted: false,
            }),
            self.effect_scoped,
            cancel,
        )?;
        self.run_effects(limits, cancel)
    }

    pub(super) fn split_text_effect(
        &mut self,
        mut frame: SplitFrame,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        if !frame.inserted {
            frame.inserted = true;
            self.push_effect(select::Effect::SplitText(frame), self.effect_scoped, cancel)?;
            if let Some(parent) = frame.parent {
                self.insert_validated(
                    frame.new_node,
                    parent,
                    self.next_sibling(frame.node),
                    limits,
                    cancel,
                )?;
            }
        } else {
            if let Some(parent) = frame.parent {
                self.range_split_text(frame.node, frame.new_node, parent, frame.offset, cancel)?;
            }
            self.replace_data(frame.node, frame.offset, frame.count, &[], limits, cancel)?;
        }
        Ok(())
    }

    pub(super) fn replace_data(
        &mut self,
        node: Key,
        offset: usize,
        count: usize,
        data: &[u16],
        limits: DomLimits,
        cancelled: &AtomicBool,
    ) -> Result<(), DomError> {
        let previous_units = self.node(node)?.logical_units()?;
        let fixed_units = match self.node(node)?.kind() {
            transaction::KindRead::ProcessingInstruction(value) => value.target().len(),
            _ => 0,
        };
        let previous = match self.node(node)?.kind() {
            transaction::KindRead::Text(data) | transaction::KindRead::Comment(data) => data,
            transaction::KindRead::ProcessingInstruction(value) => value.data(),
            _ => return Err(DomError::WrongNodeKind),
        };
        if offset > previous.len() {
            return Err(DomError::IndexSize);
        }
        let removed = count.min(previous.len() - offset);
        let length = (previous.len() - removed)
            .checked_add(data.len())
            .ok_or(DomError::RepresentationExhausted)?;
        let raw_units = (self.units - previous_units)
            .checked_add(fixed_units)
            .and_then(|units| units.checked_add(length))
            .ok_or(DomError::RepresentationExhausted)?;
        admit(raw_units, limits.units, DomResource::StringCodeUnits)?;
        #[cfg(test)]
        let growth = source::formation::mutation_observation::before(
            self,
            source::formation::mutation_observation::Site::ReplaceData,
            cancelled,
        )?;
        let formation = source::formation::Formation::new(self, cancelled)?;
        let mut value = match &formation {
            Some(formation) => {
                source::formation::utf16_buffer(length, limits.units, Some(formation), cancelled)?
            }
            None => {
                let mut output = Vec::new();
                reserve(&mut output, length)?;
                output
            }
        };
        #[cfg(test)]
        let mut copied = transaction::CopyObservation::default();
        for unit in previous.code_units()[..offset]
            .iter()
            .chain(data)
            .chain(&previous.code_units()[offset + removed..])
        {
            self.source_step(cancelled)?;
            value.push(*unit);
            #[cfg(test)]
            copied.pushed::<u16>();
        }
        let value = DocumentString::from_utf16(value);
        #[cfg(test)]
        source::formation::mutation_observation::grew(
            growth,
            size_of::<Vec<u16>>() + value.retained_units() * size_of::<u16>(),
        );
        let kind = match self.node(node)?.kind() {
            transaction::KindRead::Text(_) => NewNode::Text(value),
            transaction::KindRead::Comment(_) => NewNode::Comment(value),
            transaction::KindRead::ProcessingInstruction(instruction) => {
                let instruction = instruction.with_data(&value, formation.as_ref(), cancelled)?;
                let mut temporary = source::storage::Payload::default();
                temporary.string(&value)?;
                drop(value);
                if let Some(budget) = &formation {
                    budget.release_payload(temporary);
                }
                NewNode::ProcessingInstruction(instruction)
            }
            _ => return Err(DomError::WrongNodeKind),
        };
        let retained_occurrence = formation.is_some();
        let credit = match formation.and_then(source::formation::Formation::into_debit) {
            Some(credit) => credit,
            None => self
                .custody_ledger()
                .retain(custody::Usage::ZERO, |usage| {
                    self.admit_custody_usage(usage, cancelled)
                })?,
        };
        let held = self.admit_held_text(
            custody::Held::new(kind, credit),
            retained_occurrence,
            cancelled,
        )?;

        let units = (self.units - previous_units)
            .checked_add(new_kind_units(held.value())?)
            .ok_or(DomError::RepresentationExhausted)?;
        admit(units, limits.units, DomResource::StringCodeUnits)?;
        self.replace_character_data_retained(node, held, retained_occurrence, cancelled)?;
        self.units = units;
        self.range_replace_data(node, offset, removed, data.len(), cancelled)?;
        if let Some(parent) = self.node(node)?.parent() {
            self.children_changed(parent, limits, cancelled)?;
        }
        Ok(())
    }
}
