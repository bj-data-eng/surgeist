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

pub(super) enum NormalizeFrame {
    Visit {
        root: Key,
        node: Key,
    },
    Transfer {
        root: Key,
        node: Key,
        length: usize,
    },
    Remove {
        root: Key,
        node: Key,
        nodes: select::ScratchVec<Key>,
        index: usize,
    },
}
impl NormalizeFrame {
    pub(super) fn freeze(&mut self, draft: &Draft, cancel: &AtomicBool) -> Result<(), DomError> {
        match self {
            Self::Remove { nodes, .. } => nodes.freeze(draft, cancel),
            _ => Ok(()),
        }
    }
    pub(super) fn checkpoint(
        &self,
        a: &mut source::checkpoint::Admission<'_>,
        cancel: &AtomicBool,
    ) -> Result<Self, DomError> {
        Ok(match self {
            Self::Visit { root, node } => Self::Visit {
                root: *root,
                node: *node,
            },
            Self::Transfer { root, node, length } => Self::Transfer {
                root: *root,
                node: *node,
                length: *length,
            },
            Self::Remove {
                root,
                node,
                nodes,
                index,
            } => Self::Remove {
                root: *root,
                node: *node,
                nodes: nodes.checkpoint(a, cancel)?,
                index: *index,
            },
        })
    }
}

impl Draft {
    pub(super) fn normalize(
        &mut self,
        root: Key,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.source_step(cancel)?;
        if let Some(node) = self.node(root)?.children().first() {
            self.push_effect(
                select::Effect::Normalize(NormalizeFrame::Visit { root, node }),
                self.effect_scoped,
                cancel,
            )?;
        }
        self.run_effects(limits, cancel)
    }

    fn text_next_descendant(
        &self,
        root: Key,
        mut node: Key,
        cancel: &AtomicBool,
    ) -> Result<Option<Key>, DomError> {
        self.source_step(cancel)?;
        if let Some(child) = self.node(node)?.children().first() {
            return Ok(Some(child));
        }
        while self.canonical_key(node) != self.canonical_key(root) {
            self.source_step(cancel)?;
            if let Some(next) = self.next_sibling(node) {
                return Ok(Some(next));
            }
            let Some(parent) = self.node(node)?.parent() else {
                return Ok(None);
            };
            node = parent;
        }
        Ok(None)
    }

    fn text_span(
        &self,
        node: Key,
        cancel: &AtomicBool,
    ) -> Result<(Key, std::ops::Range<usize>, usize), DomError> {
        let parent = self.node(node)?.parent().ok_or(DomError::NotFound)?;
        let children = self.node(parent)?.children();
        let mut position = None;
        for (index, child) in children.iter().enumerate() {
            self.source_step(cancel)?;
            if self.canonical_key(child) == self.canonical_key(node) {
                position = Some(index);
                break;
            }
        }
        let position = position.ok_or(DomError::NotFound)?;
        let mut start = position;
        while start > 0 {
            self.source_step(cancel)?;
            let previous = children.get(start - 1).ok_or(DomError::NotFound)?;
            if !matches!(self.node(previous)?.kind(), transaction::KindRead::Text(_)) {
                break;
            }
            start -= 1;
        }
        let mut end = position + 1;
        while let Some(next) = children.get(end) {
            self.source_step(cancel)?;
            if !matches!(self.node(next)?.kind(), transaction::KindRead::Text(_)) {
                break;
            }
            end += 1;
        }
        Ok((parent, start..end, position))
    }

    pub(super) fn normalize_effect(
        &mut self,
        frame: NormalizeFrame,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        match frame {
            NormalizeFrame::Visit { root, node } => {
                let length = match self.node(node)?.kind() {
                    transaction::KindRead::Text(data) => Some(data.len()),
                    transaction::KindRead::Document(_)
                    | transaction::KindRead::Element(_)
                    | transaction::KindRead::Attr(_)
                    | transaction::KindRead::DocumentFragment
                    | transaction::KindRead::ShadowRoot(_)
                    | transaction::KindRead::Comment(_)
                    | transaction::KindRead::ProcessingInstruction(_)
                    | transaction::KindRead::DocumentType(_) => None,
                };
                if length == Some(0) {
                    // The cursor leaves the removed node through its actual former
                    // tree position, never through the now-detached Text's parent.
                    if let Some(next) = self.text_next_descendant(root, node, cancel)? {
                        self.push_effect(
                            select::Effect::Normalize(NormalizeFrame::Visit { root, node: next }),
                            self.effect_scoped,
                            cancel,
                        )?;
                    }
                    self.remove(node, limits, cancel)?;
                } else if let Some(length) = length {
                    let (parent, span, position) = self.text_span(node, cancel)?;
                    let children = self.node(parent)?.children();
                    let mut size = 0usize;
                    for index in span.clone().filter(|index| *index != position) {
                        self.source_step(cancel)?;
                        let child = children.get(index).ok_or(DomError::NotFound)?;
                        let transaction::KindRead::Text(data) = self.node(child)?.kind() else {
                            return Err(DomError::WrongNodeKind);
                        };
                        size = size
                            .checked_add(data.len())
                            .ok_or(DomError::RepresentationExhausted)?;
                    }
                    let formation = source::formation::Formation::new(self, cancel)?;
                    let mut data = source::formation::utf16_buffer(
                        size,
                        limits.units,
                        formation.as_ref(),
                        cancel,
                    )?;
                    #[cfg(test)]
                    let mut copied = transaction::CopyObservation::default();
                    for index in span.filter(|index| *index != position) {
                        self.source_step(cancel)?;
                        let child = children.get(index).ok_or(DomError::NotFound)?;
                        let transaction::KindRead::Text(text) = self.node(child)?.kind() else {
                            return Err(DomError::WrongNodeKind);
                        };
                        for unit in text.code_units() {
                            self.source_step(cancel)?;
                            data.push(*unit);
                            #[cfg(test)]
                            copied.pushed::<u16>();
                        }
                    }
                    self.push_effect(
                        select::Effect::Normalize(NormalizeFrame::Transfer { root, node, length }),
                        self.effect_scoped,
                        cancel,
                    )?;
                    self.replace_data(node, length, 0, &data, limits, cancel)?;
                    drop(data);
                    drop(formation);
                } else if let Some(next) = self.text_next_descendant(root, node, cancel)? {
                    self.push_effect(
                        select::Effect::Normalize(NormalizeFrame::Visit { root, node: next }),
                        self.effect_scoped,
                        cancel,
                    )?;
                }
            }
            NormalizeFrame::Transfer {
                root,
                node,
                mut length,
            } => {
                // Replacement and all its hooks precede these fresh sibling reads.
                let mut current = self.next_sibling(node);
                while let Some(next) = current {
                    self.source_step(cancel)?;
                    let transaction::KindRead::Text(data) = self.node(next)?.kind() else {
                        break;
                    };
                    let size = data.len();
                    self.range_normalize_text(node, next, length, cancel)?;
                    length = length
                        .checked_add(size)
                        .ok_or(DomError::RepresentationExhausted)?;
                    current = self.next_sibling(next);
                }
                // Select the source's actual removal set after endpoint transfers.
                let (parent, span, position) = self.text_span(node, cancel)?;
                let mut nodes = self.effect_vec();
                self.select_reserve(&mut nodes, span.len().saturating_sub(1), cancel)?;
                for index in span.filter(|index| *index != position) {
                    self.source_step(cancel)?;
                    nodes.push(
                        self.node(parent)?
                            .children()
                            .get(index)
                            .ok_or(DomError::NotFound)?,
                    );
                }
                self.push_effect(
                    select::Effect::Normalize(NormalizeFrame::Remove {
                        root,
                        node,
                        nodes,
                        index: 0,
                    }),
                    self.effect_scoped,
                    cancel,
                )?;
            }
            NormalizeFrame::Remove {
                root,
                node,
                nodes,
                index,
            } => {
                if let Some(&next) = nodes.get(index) {
                    self.push_effect(
                        select::Effect::Normalize(NormalizeFrame::Remove {
                            root,
                            node,
                            nodes,
                            index: index + 1,
                        }),
                        self.effect_scoped,
                        cancel,
                    )?;
                    self.remove(next, limits, cancel)?;
                } else if let Some(next) = self.text_next_descendant(root, node, cancel)? {
                    self.push_effect(
                        select::Effect::Normalize(NormalizeFrame::Visit { root, node: next }),
                        self.effect_scoped,
                        cancel,
                    )?;
                }
            }
        }
        Ok(())
    }

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
        self.publish_character_data(
            node,
            (kind, previous_units),
            (offset, removed, data.len()),
            formation,
            limits,
            cancelled,
        )
    }

    pub(super) fn edit_processing_instruction_attribute(
        &mut self,
        node: Key,
        edit: &crate::DomProcessingInstructionAttributeEdit,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        use crate::DomProcessingInstructionAttributeEdit as Edit;
        let current = self.node(node)?;
        let transaction::KindRead::ProcessingInstruction(instruction) = current.kind() else {
            return Err(DomError::WrongNodeKind);
        };
        let (name, validate) = match edit {
            Edit::Set { name, .. } | Edit::Toggle { name, .. } => (name, true),
            Edit::Remove { name } => (name, false),
        };
        if validate
            && !crate::names::valid_attribute_local_name(name, &mut || self.source_step(cancel))?
        {
            return Err(DomError::InvalidCharacter);
        }
        let existing = instruction
            .attribute_with(name.code_units(), || self.source_step(cancel))?
            .map(|(index, _)| index);
        let toggle = match edit {
            Edit::Toggle { force, .. } => Some(force.unwrap_or(existing.is_none())),
            _ => None,
        };
        if let Some(present) = toggle
            && present == existing.is_some()
        {
            return self.register_node_ticket(
                NodeTicket::ProcessingInstructionAttributePresent(node, present),
                cancel,
            );
        }
        let formation = source::formation::Formation::new(self, cancel)?;
        let replacement = match edit {
            Edit::Set { value, .. } => Some(value),
            _ => None,
        };
        let mut attributes = Vec::new();
        let count = instruction.attributes().len();
        let adds = replacement.is_some() || toggle == Some(true);
        let count = match (existing, adds) {
            (None, true) => count
                .checked_add(1)
                .ok_or(DomError::RepresentationExhausted)?,
            (Some(_), false) => count - 1,
            _ => count,
        };
        source::formation::reserve(&mut attributes, count, formation.as_ref(), cancel)?;
        let share =
            |value: &DocumentString| source::formation::share(value, formation.as_ref(), cancel);
        for (index, (key, value)) in instruction.attributes().enumerate() {
            self.source_step(cancel)?;
            let value = if existing == Some(index) {
                replacement
            } else {
                Some(value)
            };
            if let Some(value) = value {
                attributes.push((share(key)?, share(value)?));
            }
        }
        if existing.is_none() && adds {
            let name = share(name)?;
            let value = match replacement {
                Some(value) => share(value)?,
                None => source::formation::string("", formation.as_ref(), cancel)?,
            };
            attributes.push((name, value));
        }
        let old_length = instruction.data().len();
        let previous_units = current.logical_units()?;
        let updated = instruction.with_authored_attributes(
            attributes,
            limits.units,
            formation.as_ref(),
            cancel,
        )?;
        let new_length = updated.data().len();
        if let Some(present) = toggle {
            self.register_node_ticket(
                NodeTicket::ProcessingInstructionAttributePresent(node, present),
                cancel,
            )?;
        }
        self.publish_character_data(
            node,
            (NewNode::ProcessingInstruction(updated), previous_units),
            (0, old_length, new_length),
            formation,
            limits,
            cancel,
        )
    }

    fn publish_character_data(
        &mut self,
        node: Key,
        (kind, previous_units): (NewNode, usize),
        (offset, removed, inserted): (usize, usize, usize),
        formation: Option<source::formation::Formation>,
        limits: DomLimits,
        cancelled: &AtomicBool,
    ) -> Result<(), DomError> {
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
        self.range_replace_data(node, offset, removed, inserted, cancelled)?;
        if let Some(parent) = self.node(node)?.parent() {
            self.children_changed(parent, limits, cancelled)?;
        }
        Ok(())
    }
}
