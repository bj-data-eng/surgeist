//! DOM3071's named compound algorithms, under one original Source root.
use super::*;
use crate::compound::{Edit, Variadic};
#[cfg(test)]
mod tests;

#[derive(Clone, Copy)]
enum Stage {
    Convert,
    Apply,
    Cleanup,
}
pub(super) struct Frame {
    receiver: Key,
    parent: Key,
    viable: Option<Key>,
    method: Variadic,
    nodes: select::ScratchVec<Key>,
    fragment: Option<Key>,
    index: usize,
    stage: Stage,
}
impl Frame {
    pub(super) fn freeze(&mut self, draft: &Draft, cancel: &AtomicBool) -> Result<(), DomError> {
        self.nodes.freeze(draft, cancel)
    }
    pub(super) fn checkpoint(
        &self,
        a: &mut source::checkpoint::Admission<'_>,
        cancel: &AtomicBool,
    ) -> Result<Self, DomError> {
        Ok(Self {
            receiver: self.receiver,
            parent: self.parent,
            viable: self.viable,
            method: self.method,
            nodes: self.nodes.checkpoint(a, cancel)?,
            fragment: self.fragment,
            index: self.index,
            stage: self.stage,
        })
    }
}
impl Draft {
    pub(super) fn compound_failure(&self) -> Option<&DomError> {
        match self.method.as_ref() {
            Some(method::Frame::CompoundFailure(error)) => Some(error),
            _ => None,
        }
    }
    fn prepare_compound_failure_slot(&mut self, cancel: &AtomicBool) -> Result<(), DomError> {
        if self.compound_failure().is_some() {
            return Ok(());
        }
        if self.method.is_some() {
            return Err(DomError::InvalidSourceOperation);
        }
        // Admit the inverse before validation can select an ordinary error.
        // Recording that error afterwards must not perform fallible work.
        if self.prepare_undo(cancel)? {
            self.record_source_undo_prepared(source::ControlUndo::Method(None));
        }
        Ok(())
    }
    fn compound_error(&mut self, error: DomError) -> Result<(), DomError> {
        match error {
            DomError::HierarchyRequest
            | DomError::NotFound
            | DomError::ForeignOwner
            | DomError::StaleNode
            | DomError::WrongNodeKind
            | DomError::InvalidAdjacentPosition => {
                if !matches!(self.method, Some(method::Frame::CompoundFailure(_))) {
                    self.method = Some(method::Frame::CompoundFailure(error));
                }
                Ok(())
            }
            error => Err(error),
        }
    }
    fn compound_text(
        &mut self,
        document: Key,
        value: &DocumentString,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<Key, DomError> {
        let (value, payload) = source::formation::shared_string(self, value, cancel)?;
        self.allocate_charged(NewNode::Text(value), document, limits, cancel, payload)
    }
    pub(super) fn compound_discard_unpublished(
        &mut self,
        key: Key,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.source_step(cancel)?;
        if self.published_id(key).is_none()
            && self.node(key)?.parent().is_none()
            && self.node(key)?.children().is_empty()
        {
            self.retire_node(key, cancel)?;
        }
        Ok(())
    }
    pub(super) fn compound_edit(
        &mut self,
        edit: &Edit,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.source_step(cancel)?;
        let receiver = self.resolve(edit.receiver().into())?;
        match edit {
            Edit::TextContent { value, .. } => match self.node(receiver)?.kind() {
                transaction::KindRead::Attr(_) => {
                    self.set_existing_attr_value(receiver, value, limits, cancel)
                }
                transaction::KindRead::Element(_)
                | transaction::KindRead::DocumentFragment
                | transaction::KindRead::ShadowRoot(_) => {
                    let (value, payload) = source::formation::shared_string(self, value, cancel)?;
                    self.string_replace_all(receiver, value, payload, limits, cancel)
                }
                transaction::KindRead::Text(value) | transaction::KindRead::Comment(value) => {
                    let count = value.len();
                    self.compound_replace_data(receiver, count, edit, limits, cancel)
                }
                transaction::KindRead::ProcessingInstruction(value) => {
                    let count = value.data().len();
                    self.compound_replace_data(receiver, count, edit, limits, cancel)
                }
                transaction::KindRead::Document(_) | transaction::KindRead::DocumentType(_) => {
                    Ok(())
                }
            },
            Edit::AdjacentText { position, data, .. } => {
                self.element(receiver)?;
                let text =
                    self.compound_text(self.node(receiver)?.document(), data, limits, cancel)?;
                self.push_effect(
                    select::Effect::CompoundTextCleanup(text),
                    self.effect_scoped,
                    cancel,
                )?;
                self.prepare_compound_failure_slot(cancel)?;
                let mut selected = None;
                for (index, name) in ["beforebegin", "afterbegin", "beforeend", "afterend"]
                    .into_iter()
                    .enumerate()
                {
                    self.source_step(cancel)?;
                    if position.len() == name.len() {
                        let mut equal = true;
                        for (unit, expected) in position.code_units().iter().zip(name.bytes()) {
                            self.source_step(cancel)?;
                            equal &= u8::try_from(*unit)
                                .is_ok_and(|unit| unit.eq_ignore_ascii_case(&expected));
                        }
                        if equal {
                            selected = Some(index);
                            break;
                        }
                    }
                }
                let parent = self.node(receiver)?.parent();
                let destination = match selected {
                    Some(0) => parent.map(|parent| (parent, Some(receiver))),
                    Some(1) => Some((receiver, self.node(receiver)?.children().first())),
                    Some(2) => Some((receiver, None)),
                    Some(3) => parent.map(|parent| (parent, self.next_sibling(receiver))),
                    _ => {
                        return self.compound_error(DomError::InvalidAdjacentPosition);
                    }
                };
                if let Some((parent, before)) = destination {
                    // Only the named algorithm's validation can select an
                    // ordinary terminal error. Hook/service refusals retain
                    // their canonical effects and existing retry contract.
                    match self.ensure_pre_insert(text, parent, before, None, false, cancel) {
                        Ok(()) => self.insert_validated(text, parent, before, limits, cancel)?,
                        Err(error) => {
                            return self.compound_error(error);
                        }
                    }
                } else {
                    self.run_effects(limits, cancel)?;
                }
                Ok(())
            }
            Edit::Variadic { method, nodes, .. } => {
                let child = matches!(
                    method,
                    Variadic::Before | Variadic::After | Variadic::ReplaceWith
                );
                let kind = self.node(receiver)?.kind();
                if child {
                    if !matches!(
                        kind,
                        transaction::KindRead::Element(_)
                            | transaction::KindRead::Text(_)
                            | transaction::KindRead::Comment(_)
                            | transaction::KindRead::ProcessingInstruction(_)
                            | transaction::KindRead::DocumentType(_)
                    ) {
                        return Err(DomError::WrongNodeKind);
                    }
                } else if !tree::parent_can_have_children(kind) {
                    return Err(DomError::WrongNodeKind);
                }
                let parent = if child {
                    let Some(parent) = self.node(receiver)?.parent() else {
                        return Ok(());
                    };
                    parent
                } else {
                    receiver
                };
                let viable = if child {
                    self.compound_viable(receiver, parent, *method, nodes, cancel)?
                } else {
                    None
                };
                let document = self.node(receiver)?.document();
                let mut converted = self.effect_vec();
                self.select_reserve(&mut converted, nodes.len(), cancel)?;
                // DOM3071: every string is converted before the first append.
                for operand in nodes {
                    self.source_step(cancel)?;
                    converted.push(match operand {
                        crate::DomNodeOrString::Node(id) => Key::Existing(*id),
                        crate::DomNodeOrString::String(value) => {
                            self.compound_text(document, value, limits, cancel)?
                        }
                    });
                }
                let fragment = if converted.len() == 1 {
                    None
                } else {
                    Some(self.allocate(NewNode::DocumentFragment, document, limits, cancel)?)
                };
                self.push_effect(
                    select::Effect::Compound(Frame {
                        receiver,
                        parent,
                        viable,
                        method: *method,
                        nodes: converted,
                        fragment,
                        index: 0,
                        stage: Stage::Convert,
                    }),
                    self.effect_scoped,
                    cancel,
                )?;
                self.run_effects(limits, cancel)
            }
        }
    }
    fn compound_replace_data(
        &mut self,
        receiver: Key,
        count: usize,
        edit: &Edit,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let Edit::TextContent { value, .. } = edit else {
            return Err(DomError::InvalidSourceOperation);
        };
        let (data, payload) = source::formation::shared_string(self, value, cancel)?;
        let receiver = self
            .published_id(receiver)
            .ok_or(DomError::InvalidSourceOperation)?;
        let result = self.apply(
            &Command::ReplaceData {
                node: receiver.into(),
                offset: 0,
                count,
                data,
            },
            limits,
            cancel,
        );
        drop(payload);
        result
    }
    fn compound_viable(
        &self,
        receiver: Key,
        parent: Key,
        method: Variadic,
        nodes: &[crate::DomNodeOrString],
        cancel: &AtomicBool,
    ) -> Result<Option<Key>, DomError> {
        let mut previous = None;
        let mut found = false;
        for key in self.node(parent)?.children().iter() {
            self.source_step(cancel)?;
            if key == receiver {
                if matches!(method, Variadic::Before) {
                    return Ok(previous);
                }
                found = true;
                continue;
            }
            let mut member = false;
            for operand in nodes {
                self.source_step(cancel)?;
                member |= matches!(operand,crate::DomNodeOrString::Node(id) if Some(*id)==self.published_id(key));
            }
            if !member {
                if found {
                    return Ok(Some(key));
                }
                previous = Some(key);
            }
        }
        Ok(None)
    }
    fn compound_next(&self, node: Key, cancel: &AtomicBool) -> Result<Option<Key>, DomError> {
        self.source_step(cancel)?;
        let Some(parent) = self.node(node)?.parent() else {
            return Ok(None);
        };
        let mut found = false;
        for child in self.node(parent)?.children().iter() {
            self.source_step(cancel)?;
            if found {
                return Ok(Some(child));
            }
            found = child == node;
        }
        Ok(None)
    }
    pub(super) fn compound_effect(
        &mut self,
        mut frame: Frame,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.source_step(cancel)?;
        if matches!(self.method, Some(method::Frame::CompoundFailure(_))) {
            frame.stage = Stage::Cleanup;
        } else if !matches!(frame.stage, Stage::Cleanup) {
            self.prepare_compound_failure_slot(cancel)?;
        }
        match frame.stage {
            Stage::Convert => {
                if let Some(fragment) = frame.fragment
                    && let Some(node) = frame.nodes.get(frame.index).copied()
                {
                    frame.index += 1;
                    self.push_effect(select::Effect::Compound(frame), self.effect_scoped, cancel)?;
                    let result = self.pre_insert(node, fragment, None, limits, cancel);
                    if let Err(error) = result {
                        self.compound_error(error)?;
                    }
                } else {
                    frame.stage = Stage::Apply;
                    self.push_effect(select::Effect::Compound(frame), self.effect_scoped, cancel)?;
                }
            }
            Stage::Apply => {
                let node = frame
                    .fragment
                    .or_else(|| frame.nodes.first().copied())
                    .ok_or(DomError::InvalidSourceOperation)?;
                let (parent, receiver, viable, method) =
                    (frame.parent, frame.receiver, frame.viable, frame.method);
                frame.stage = Stage::Cleanup;
                self.push_effect(select::Effect::Compound(frame), self.effect_scoped, cancel)?;
                let result = (|| match method {
                    Variadic::Append => self.pre_insert(node, parent, None, limits, cancel),
                    Variadic::Prepend => self.pre_insert(
                        node,
                        parent,
                        self.node(parent)?.children().first(),
                        limits,
                        cancel,
                    ),
                    Variadic::ReplaceChildren => {
                        self.replace_all_checked(node, parent, limits, cancel)
                    }
                    Variadic::Before => {
                        let before = match viable {
                            Some(previous) => self.compound_next(previous, cancel)?,
                            None => self.node(parent)?.children().first(),
                        };
                        self.pre_insert(node, parent, before, limits, cancel)
                    }
                    Variadic::After => self.pre_insert(node, parent, viable, limits, cancel),
                    Variadic::ReplaceWith => {
                        if self.node(receiver)?.parent() == Some(parent) {
                            self.replace_child(node, parent, receiver, limits, cancel)
                        } else {
                            self.pre_insert(node, parent, viable, limits, cancel)
                        }
                    }
                })();
                if let Err(error) = result {
                    self.compound_error(error)?;
                }
            }
            Stage::Cleanup => {
                for node in frame.nodes.iter().copied() {
                    self.compound_discard_unpublished(node, cancel)?;
                }
                if let Some(fragment) = frame.fragment {
                    self.compound_discard_unpublished(fragment, cancel)?;
                }
            }
        }
        Ok(())
    }
}
