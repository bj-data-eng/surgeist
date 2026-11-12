use super::*;

#[derive(Clone, Copy)]
pub(in crate::store) struct MoveRecords {
    node: Key,
    old_parent: Key,
    old_previous: Option<Key>,
    old_next: Option<Key>,
    new_parent: Key,
    new_previous: Option<Key>,
    new_next: Option<Key>,
}

impl MoveRecords {
    pub(super) fn queue(self, draft: &mut Draft, cancel: &AtomicBool) -> Result<(), DomError> {
        draft.queue_child_list_record(
            child_list::MutationInput {
                target: self.old_parent,
                added: &[],
                removed: &[self.node],
                previous: self.old_previous,
                next: self.old_next,
            },
            cancel,
        )?;
        draft.queue_child_list_record(
            child_list::MutationInput {
                target: self.new_parent,
                added: &[self.node],
                removed: &[],
                previous: self.new_previous,
                next: self.new_next,
            },
            cancel,
        )
    }
}

impl Draft {
    pub(in crate::store) fn move_before(
        &mut self,
        node: Key,
        new_parent: Key,
        before: Option<Key>,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let before = if before == Some(node) {
            match self.node(node)?.parent() {
                Some(parent) => {
                    let index = self.child_list_position(parent, node, cancel)?;
                    self.node(parent)?.children().get(index + 1)
                }
                None => None,
            }
        } else {
            before
        };
        let old_parent = self.ensure_move(node, new_parent, before, cancel)?;

        self.range_pre_remove(node, old_parent, cancel)?;
        self.node_iterator_pre_remove(node, old_parent, cancel)?;
        let old_index = self.child_list_position(old_parent, node, cancel)?;
        let children = self.node(old_parent)?.children();
        let old_previous = old_index
            .checked_sub(1)
            .and_then(|index| children.get(index));
        let old_next = children.get(old_index + 1);
        self.relations_remove(
            old_parent,
            transaction::RelationField::Children,
            old_index,
            1,
            cancel,
        )?;
        let mut fields = self.common_fields_mut(node, cancel)?;
        fields.parent = None;
        fields.commit(cancel)?;
        self.slot_removal(node, old_parent, limits, cancel)?;
        self.child_list_removed(node, old_parent, None, cancel)?;

        self.range_insert(new_parent, before, 1, cancel)?;
        let index = match before {
            Some(child) => self.child_list_position(new_parent, child, cancel)?,
            None => self.node(new_parent)?.children().len(),
        };
        let children = self.node(new_parent)?.children();
        let new_previous = index.checked_sub(1).and_then(|index| children.get(index));
        self.relations_insert(
            new_parent,
            transaction::RelationField::Children,
            index,
            &[node],
            cancel,
        )?;
        let mut fields = self.common_fields_mut(node, cancel)?;
        fields.parent = Some(new_parent);
        fields.commit(cancel)?;
        self.slot_insertion(node, new_parent, limits, cancel)?;

        self.push_effect(
            select::Effect::Tree(Frame::MoveRecords(MoveRecords {
                node,
                old_parent,
                old_previous,
                old_next,
                new_parent,
                new_previous,
                new_next: before,
            })),
            self.effect_scoped,
            cancel,
        )?;
        let mut pending = source::effects::ScratchStack::new(self);
        pending.push(node, self, cancel)?;
        self.push_effect(
            select::Effect::Tree(Frame::Walk {
                root: node,
                hook: TreeHook::Move { old_parent },
                pending,
            }),
            self.effect_scoped,
            cancel,
        )?;
        self.run_effects(limits, cancel)
    }

    fn ensure_move(
        &self,
        node: Key,
        parent: Key,
        before: Option<Key>,
        cancel: &AtomicBool,
    ) -> Result<Key, DomError> {
        if !parent_can_have_children(self.node(parent)?.kind()) {
            return Err(DomError::HierarchyRequest);
        }
        if self.shadow_including_root(parent, cancel)?
            != self.shadow_including_root(node, cancel)?
        {
            return Err(DomError::HierarchyRequest);
        }
        let mut ancestor = Some(parent);
        while let Some(candidate) = ancestor {
            self.source_step(cancel)?;
            if candidate == node {
                return Err(DomError::HierarchyRequest);
            }
            ancestor = self.host_including_parent(candidate)?;
        }
        if let Some(child) = before
            && self.node(child)?.parent() != Some(parent)
        {
            return Err(DomError::NotFound);
        }
        let kind = self.node(node)?.kind();
        if !matches!(
            kind,
            KindRead::Element(_)
                | KindRead::Text(_)
                | KindRead::Comment(_)
                | KindRead::ProcessingInstruction(_)
        ) {
            return Err(DomError::HierarchyRequest);
        }
        if matches!(self.node(parent)?.kind(), KindRead::Document(_)) {
            match kind {
                KindRead::Text(_) => return Err(DomError::HierarchyRequest),
                KindRead::Element(_) => self.ensure_document_move(parent, before, cancel)?,
                _ => {}
            }
        }
        self.node(node)?.parent().ok_or(DomError::HierarchyRequest)
    }

    fn ensure_document_move(
        &self,
        parent: Key,
        before: Option<Key>,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let mut at_or_after_reference = false;
        for child in self.node(parent)?.children().iter() {
            self.source_step(cancel)?;
            at_or_after_reference |= Some(child) == before;
            match self.node(child)?.kind() {
                KindRead::Element(_) => return Err(DomError::HierarchyRequest),
                KindRead::DocumentType(_) if at_or_after_reference => {
                    return Err(DomError::HierarchyRequest);
                }
                _ => {}
            }
        }
        Ok(())
    }

    pub(super) fn native_moving_node(
        &mut self,
        node: Key,
        root: Key,
        old_parent: Key,
        stage: u8,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        if stage == 0 {
            self.image_tree_change(
                node,
                surgeist_html::elements::image::HtmlImageTreeChange::Moved,
                root,
                Some(old_parent),
                cancel,
            )?;
            self.hyperlink_moving_node(node, cancel)?;
            self.push_effect(
                select::Effect::Tree(Frame::Node {
                    key: node,
                    root,
                    hook: TreeHook::Move { old_parent },
                    stage: 1,
                }),
                self.effect_scoped,
                cancel,
            )?;
            self.select_moving_node(node, limits, cancel)?;
        } else {
            self.control_moving_node(node, cancel)?;
        }
        Ok(())
    }
}
