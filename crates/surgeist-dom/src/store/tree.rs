use super::*;

pub(super) struct Insertion {
    moving: select::ScratchVec<Key>,
    fragment: bool,
    parent: Key,
    before: Option<Key>,
    index: usize,
    stage: u8,
    #[cfg(test)]
    _observation: scratch_observation::Moving,
}
pub(super) enum Frame {
    FullscreenRemoving {
        document: Key,
        flagged: select::ScratchVec<Key>,
        index: usize,
        returned: bool,
    },
    Destroy {
        closure: select::ScratchVec<Key>,
        index: usize,
        stage: u8,
    },
    Insert(Insertion),
    ReleaseMoving(Insertion),
    RemovedParent(Key),
    Walk {
        root: Key,
        old_parent: Option<Key>,
        pending: select::ScratchVec<Key>,
    },
    Node {
        key: Key,
        root: Key,
        old_parent: Option<Key>,
        stage: u8,
    },
    AdoptAfterRemoval {
        node: Key,
        document: Key,
    },
    Replace {
        node: Key,
        parent: Key,
        child: Key,
        before: Option<Key>,
        stage: u8,
    },
    ReplaceAll {
        key: Key,
        replacement: Option<Key>,
        inserted: bool,
    },
}

impl Frame {
    pub(super) fn checkpoint(
        &self,
        a: &mut source::checkpoint::Admission<'_>,
        cancel: &AtomicBool,
    ) -> Result<Self, DomError> {
        Ok(match self {
            Self::FullscreenRemoving {
                document,
                flagged,
                index,
                returned,
            } => Self::FullscreenRemoving {
                document: *document,
                flagged: flagged.checkpoint(a, cancel)?,
                index: *index,
                returned: *returned,
            },
            Self::Destroy {
                closure,
                index,
                stage,
            } => Self::Destroy {
                closure: closure.checkpoint(a, cancel)?,
                index: *index,
                stage: *stage,
            },
            Self::Insert(frame) => Self::Insert(frame.checkpoint(a, cancel)?),
            Self::ReleaseMoving(frame) => Self::ReleaseMoving(frame.checkpoint(a, cancel)?),
            Self::RemovedParent(key) => Self::RemovedParent(*key),
            Self::Walk {
                root,
                old_parent,
                pending,
            } => Self::Walk {
                root: *root,
                old_parent: *old_parent,
                pending: pending.checkpoint(a, cancel)?,
            },
            Self::Node {
                key,
                root,
                old_parent,
                stage,
            } => Self::Node {
                key: *key,
                root: *root,
                old_parent: *old_parent,
                stage: *stage,
            },
            Self::AdoptAfterRemoval { node, document } => Self::AdoptAfterRemoval {
                node: *node,
                document: *document,
            },
            Self::Replace {
                node,
                parent,
                child,
                before,
                stage,
            } => Self::Replace {
                node: *node,
                parent: *parent,
                child: *child,
                before: *before,
                stage: *stage,
            },
            Self::ReplaceAll {
                key,
                replacement,
                inserted,
            } => Self::ReplaceAll {
                key: *key,
                replacement: *replacement,
                inserted: *inserted,
            },
        })
    }
}
impl Insertion {
    fn checkpoint(
        &self,
        a: &mut source::checkpoint::Admission<'_>,
        cancel: &AtomicBool,
    ) -> Result<Self, DomError> {
        let moving = self.moving.checkpoint(a, cancel)?;
        #[cfg(test)]
        let observation = scratch_observation::Moving::new(
            moving.capacity() * size_of::<Key>(),
            moving.capacity() * size_of::<Key>(),
        );
        Ok(Self {
            moving,
            fragment: self.fragment,
            parent: self.parent,
            before: self.before,
            index: self.index,
            stage: self.stage,
            #[cfg(test)]
            _observation: observation,
        })
    }
}

pub(super) fn parent_can_have_children(kind: &DomNodeKind) -> bool {
    matches!(
        kind,
        DomNodeKind::Document(_)
            | DomNodeKind::DocumentFragment
            | DomNodeKind::ShadowRoot(_)
            | DomNodeKind::Element(_)
    )
}

impl Draft {
    pub(super) fn string_replace_all(
        &mut self,
        key: Key,
        raw: DocumentString,
        payload: Option<source::checkpoint::Charge>,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let replacement = if raw.is_empty() {
            drop(raw);
            None
        } else {
            Some(self.allocate(
                DomNodeKind::Text(raw),
                self.node(key)?.document,
                limits,
                cancel,
            )?)
        };
        // The replacement Text owns the operand before any nested hook can allocate.
        drop(payload);
        self.push_effect(
            select::Effect::Tree(Frame::ReplaceAll {
                key,
                replacement,
                inserted: false,
            }),
            self.effect_scoped,
            cancel,
        )?;
        self.run_effects(limits, cancel)
    }

    pub(super) fn shadow_connected(
        &self,
        mut key: Key,
        cancel: &AtomicBool,
    ) -> Result<bool, DomError> {
        loop {
            self.effect_step(cancel)?;
            match self.shadow_including_parent(key)? {
                Some(parent) => key = parent,
                None => return Ok(matches!(self.node(key)?.kind, DomNodeKind::Document(_))),
            }
        }
    }

    fn native_post_connection_steps(
        &mut self,
        roots: &[Key],
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        // DOM insert steps10–12 collect the whole static shadow-including list
        // before invoking any post-connection work. Template contents are owned
        // auxiliaries, not shadow-including descendants of their host.
        let mut list = self.effect_vec();
        let mut pending = self.effect_vec();
        self.select_reserve(&mut pending, roots.len(), cancel)?;
        for key in roots.iter().rev() {
            self.source_step(cancel)?;
            pending.push(*key);
        }
        while !pending.is_empty() {
            self.source_step(cancel)?;
            let key = pending.pop().expect("admitted pending node");
            self.effect_step(cancel)?;
            self.select_reserve(&mut list, 1, cancel)?;
            list.push(key);
            let node = self.node(key)?;
            self.select_reserve(&mut pending, node.children.len(), cancel)?;
            for child in node.children.iter().rev() {
                self.source_step(cancel)?;
                pending.push(*child);
            }
            if let Some(root) = node.shadow_root {
                self.select_reserve(&mut pending, 1, cancel)?;
                pending.push(root);
            }
        }
        self.push_effect(
            select::Effect::Post {
                list,
                index: 0,
                stage: 0,
            },
            self.effect_scoped,
            cancel,
        )?;
        self.run_effects(limits, cancel)
    }

    pub(super) fn adopt(
        &mut self,
        node: Key,
        document: Key,
        limits: DomLimits,
        cancelled: &AtomicBool,
    ) -> Result<(), DomError> {
        self.preflight_top_layer_adoption(node, document, true, cancelled)?;
        self.push_effect(
            select::Effect::Tree(Frame::AdoptAfterRemoval { node, document }),
            self.effect_scoped,
            cancelled,
        )?;
        self.remove(node, limits, cancelled)?;
        self.run_effects(limits, cancelled)
    }
    fn adopt_document(
        &mut self,
        node: Key,
        document: Key,
        limits: DomLimits,
        cancelled: &AtomicBool,
    ) -> Result<(), DomError> {
        if self.node(node)?.document == document {
            return Ok(());
        }
        self.preflight_top_layer_adoption(node, document, false, cancelled)?;
        let mut pending = self.effect_vec();
        self.select_reserve(&mut pending, 1, cancelled)?;
        pending.push((node, document));
        while !pending.is_empty() {
            self.source_step(cancelled)?;
            let (node, document) = pending.pop().expect("admitted pending adoption");
            self.effect_step(cancelled)?;
            let children = &self.node(node)?.children;
            self.select_reserve(&mut pending, children.len(), cancelled)?;
            for child in children.iter().rev() {
                self.source_step(cancelled)?;
                pending.push((*child, document));
            }
            if let Some(root) = self.node(node)?.shadow_root {
                self.select_reserve(&mut pending, 1, cancelled)?;
                pending.push((root, document));
            }
            self.node_mut(node).document = document;
            self.select_element_registry(node, cancelled)?;
            let binding = match &self.node(document)?.kind {
                DomNodeKind::Document(metadata) => metadata.custom_element_registry(),
                _ => return Err(DomError::WrongDocument),
            };
            if let DomNodeKind::ShadowRoot(state) = &mut self.node_mut(node).kind {
                state.select_global_registry(binding);
            }
            if self.node(node)?.image.is_some() {
                self.start_image_update(
                    node,
                    surgeist_html::HtmlImageUpdateFlags::new(false, false),
                    cancelled,
                )?;
            }
            if let Some(contents) =
                self.template_adoption_target(node, document, limits, cancelled)?
            {
                self.select_reserve(&mut pending, 1, cancelled)?;
                pending.push(contents);
            }
            // Attributes are value records owned by this Element, not Attr
            // nodes. Their node-document is consequently this same relation.
        }
        Ok(())
    }

    fn next_sibling(&self, node: Key) -> Option<Key> {
        let parent = self.node(node).expect("staged node").parent?;
        let children = &self.node(parent).expect("staged parent").children;
        let position = children
            .iter()
            .position(|child| *child == node)
            .expect("staged child");
        children.get(position + 1).copied()
    }

    fn ensure_pre_insert(
        &self,
        node: Key,
        parent: Key,
        child: Option<Key>,
        exclude: Option<Key>,
        cancelled: &AtomicBool,
    ) -> Result<(), DomError> {
        if !parent_can_have_children(&self.node(parent)?.kind) {
            return Err(DomError::HierarchyRequest);
        }
        let mut ancestor = Some(parent);
        while let Some(key) = ancestor {
            self.effect_step(cancelled)?;
            if key == node {
                return Err(DomError::HierarchyRequest);
            }
            ancestor = self.host_including_parent(key)?;
        }
        if child
            .is_some_and(|child| self.node(child).expect("resolved child").parent != Some(parent))
        {
            return Err(DomError::NotFound);
        }
        let kind = &self.node(node)?.kind;
        if matches!(kind, DomNodeKind::Document(_)) {
            return Err(DomError::HierarchyRequest);
        }
        if !matches!(self.node(parent)?.kind, DomNodeKind::Document(_)) {
            return if matches!(kind, DomNodeKind::DocumentType(_)) {
                Err(DomError::HierarchyRequest)
            } else {
                Ok(())
            };
        }
        match kind {
            DomNodeKind::Text(_) => return Err(DomError::HierarchyRequest),
            DomNodeKind::Comment(_) | DomNodeKind::ProcessingInstruction(_) => return Ok(()),
            DomNodeKind::DocumentFragment | DomNodeKind::ShadowRoot(_) => {
                let mut elements = 0;
                for child in &self.node(node)?.children {
                    self.effect_step(cancelled)?;
                    match self.node(*child)?.kind {
                        DomNodeKind::Text(_) => return Err(DomError::HierarchyRequest),
                        DomNodeKind::Element(_) => elements += 1,
                        _ => {}
                    }
                    if elements > 1 {
                        return Err(DomError::HierarchyRequest);
                    }
                }
                if elements == 0 {
                    return Ok(());
                }
            }
            _ => {}
        }
        let children = &self.node(parent)?.children;
        let reference = child.map(|child| {
            children
                .iter()
                .position(|key| *key == child)
                .expect("validated reference")
        });
        for (position, candidate) in children.iter().copied().enumerate() {
            self.effect_step(cancelled)?;
            let candidate_kind = &self.node(candidate)?.kind;
            let invalid = if matches!(kind, DomNodeKind::DocumentType(_)) {
                (matches!(candidate_kind, DomNodeKind::DocumentType(_))
                    && Some(candidate) != exclude)
                    || (matches!(candidate_kind, DomNodeKind::Element(_))
                        && reference
                            .map_or(Some(candidate) != exclude, |reference| position < reference))
            } else {
                (matches!(candidate_kind, DomNodeKind::Element(_)) && Some(candidate) != exclude)
                    || (matches!(candidate_kind, DomNodeKind::DocumentType(_))
                        && reference.is_some_and(|reference| {
                            position > reference
                                || position == reference && Some(candidate) != exclude
                        }))
            };
            if invalid {
                return Err(DomError::HierarchyRequest);
            }
        }
        Ok(())
    }

    pub(super) fn pre_insert(
        &mut self,
        node: Key,
        parent: Key,
        mut before: Option<Key>,
        limits: DomLimits,
        cancelled: &AtomicBool,
    ) -> Result<(), DomError> {
        self.ensure_pre_insert(node, parent, before, None, cancelled)?;
        if before == Some(node) {
            before = self.next_sibling(node);
        }
        self.insert_validated(node, parent, before, limits, cancelled)
    }

    pub(super) fn replace_child(
        &mut self,
        node: Key,
        parent: Key,
        child: Key,
        limits: DomLimits,
        cancelled: &AtomicBool,
    ) -> Result<(), DomError> {
        self.ensure_pre_insert(node, parent, Some(child), Some(child), cancelled)?;
        let mut before = self.next_sibling(child);
        if before == Some(node) {
            before = self.next_sibling(node);
        }
        self.push_effect(
            select::Effect::Tree(Frame::Replace {
                node,
                parent,
                child,
                before,
                stage: 0,
            }),
            self.effect_scoped,
            cancelled,
        )?;
        self.run_effects(limits, cancelled)
    }

    fn insert_validated(
        &mut self,
        node: Key,
        parent: Key,
        before: Option<Key>,
        limits: DomLimits,
        cancelled: &AtomicBool,
    ) -> Result<(), DomError> {
        let fragment = matches!(
            self.node(node)?.kind,
            DomNodeKind::DocumentFragment | DomNodeKind::ShadowRoot(_)
        );
        #[cfg(test)]
        let ledger_before_moving = self.select_temporary_bytes.load(Ordering::Relaxed);
        let mut moving = self.effect_vec();
        if fragment {
            let children = &self.node(node)?.children;
            self.select_reserve(&mut moving, children.len(), cancelled)?;
            moving.extend(children.iter().copied());
        } else {
            self.select_reserve(&mut moving, 1, cancelled)?;
            moving.push(node);
        }
        #[cfg(test)]
        let _moving_observation = scratch_observation::Moving::new(
            moving.capacity() * size_of::<Key>(),
            self.select_temporary_bytes.load(Ordering::Relaxed) - ledger_before_moving,
        );
        if !moving.is_empty() {
            let children = &self.node(parent)?.children;
            let admitted = self.source_reserve_amount::<Key>(
                children.len(),
                children.capacity(),
                moving.len(),
                cancelled,
            )?;
            #[cfg(test)]
            let growth = if self.node(parent)?.children.len() + moving.len()
                > self.node(parent)?.children.capacity()
            {
                source::formation::mutation_observation::before(
                    self,
                    source::formation::mutation_observation::Site::Children,
                    cancelled,
                )?
            } else {
                None
            };
            source::reserve_bank(&mut self.node_mut(parent).children, moving.len(), admitted)?;
            #[cfg(test)]
            source::formation::mutation_observation::grew(
                growth,
                self.node(parent)?.children.capacity() * size_of::<Key>(),
            );
            self.source_admit(0, cancelled)?;
        }
        self.push_effect(
            select::Effect::Tree(Frame::Insert(Insertion {
                moving,
                fragment,
                parent,
                before,
                index: 0,
                stage: 0,
                #[cfg(test)]
                _observation: _moving_observation,
            })),
            self.effect_scoped,
            cancelled,
        )?;
        self.run_effects(limits, cancelled)
    }

    pub(super) fn native_removing_steps(
        &mut self,
        root: Key,
        old_parent: Key,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let document = self.node(root)?.document;
        let flagged = self.fullscreen_removing_members(root, cancel)?;
        let mut pending = self.effect_vec();
        self.select_reserve(&mut pending, 1, cancel)?;
        pending.push(root);
        self.push_effect(
            select::Effect::Tree(Frame::Walk {
                root,
                old_parent: Some(old_parent),
                pending,
            }),
            self.effect_scoped,
            cancel,
        )?;
        self.push_effect(
            select::Effect::Tree(Frame::FullscreenRemoving {
                document,
                flagged,
                index: 0,
                returned: false,
            }),
            self.effect_scoped,
            cancel,
        )?;
        self.run_effects(limits, cancel)
    }

    pub(super) fn tree_effect(
        &mut self,
        frame: Frame,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        match frame {
            Frame::FullscreenRemoving {
                document,
                flagged,
                index,
                returned,
            } => {
                if let Some(&key) = flagged.get(index) {
                    self.source_step(cancel)?;
                    if !returned && self.fullscreen_model_target(document, cancel)? == Some(key) {
                        if self.source_limits.is_none()
                            && self.document_fully_active(document, cancel)?
                        {
                            return Err(DomError::FullscreenModelUnavailable {
                                step: crate::DomFullscreenModelStep::ExitFullscreen,
                            });
                        }
                        // Save the real outer removal continuation before the
                        // called method can expose its step8 handoff.
                        self.push_effect(
                            select::Effect::Tree(Frame::FullscreenRemoving {
                                document,
                                flagged,
                                index,
                                returned: true,
                            }),
                            self.effect_scoped,
                            cancel,
                        )?;
                        self.exit_fullscreen_boundary(document, false, cancel)?;
                    } else {
                        if !returned {
                            self.unfullscreen_element_model(key, cancel)?;
                        }
                        self.remove_top_layer_immediately(key, cancel)?;
                        self.push_effect(
                            select::Effect::Tree(Frame::FullscreenRemoving {
                                document,
                                flagged,
                                index: index + 1,
                                returned: false,
                            }),
                            self.effect_scoped,
                            cancel,
                        )?;
                    }
                }
            }
            Frame::Destroy {
                closure,
                index,
                stage,
            } => {
                if let Some(&key) = closure.get(index) {
                    if stage == 0 {
                        // Every real child removal finishes its source hooks
                        // before the next child or retirement cancellation.
                        if let Some(&child) = self.node(key)?.children.first() {
                            self.push_effect(
                                select::Effect::Tree(Frame::Destroy {
                                    closure,
                                    index,
                                    stage,
                                }),
                                self.effect_scoped,
                                cancel,
                            )?;
                            self.remove(child, limits, cancel)?;
                        } else {
                            self.push_effect(
                                select::Effect::Tree(Frame::Destroy {
                                    closure,
                                    index: index + 1,
                                    stage,
                                }),
                                self.effect_scoped,
                                cancel,
                            )?;
                        }
                    } else if stage == 1 {
                        self.cancel_select_tasks(key, cancel)?;
                        self.cancel_details_intentions(key, cancel)?;
                        self.cancel_image_update(key, limits, cancel)?;
                        self.push_effect(
                            select::Effect::Tree(Frame::Destroy {
                                closure,
                                index: index + 1,
                                stage,
                            }),
                            self.effect_scoped,
                            cancel,
                        )?;
                    } else {
                        check_cancel(cancel)?;
                        self.retire_template_contents_target(key, cancel)?;
                        if let Some(target) = self.published_id(key) {
                            self.source_step(cancel)?;
                            let document = self.node(key)?.document;
                            let amount = self.source_reserve_amount::<(NodeId, Key)>(
                                self.dispatch_retirements.len(),
                                self.dispatch_retirements.capacity(),
                                1,
                                cancel,
                            )?;
                            reserve(&mut self.dispatch_retirements, amount.unwrap_or(1))?;
                            self.dispatch_retirements.push((target, document));
                            self.source_admit(0, cancel)?;
                        }
                        self.retire_autofocus_candidate(key, cancel)?;
                        self.retire_fullscreen_membership(key, cancel)?;
                        self.retire_range_container(key, cancel)?;
                        let node = match key {
                            Key::Existing(i) => self.existing[i].take(),
                            Key::Created(i) => self.created[i].take(),
                        }
                        .expect("owned closure");
                        self.nodes -= 1;
                        self.units -= kind_units(&node.kind)?;
                        self.units -= node.base.as_ref().map_or(0, crate::base::BaseState::units);
                        self.units -= node
                            .hyperlink
                            .as_ref()
                            .map_or(0, crate::hyperlink::HyperlinkState::units);
                        self.units -= node
                            .encoding_url_document
                            .as_ref()
                            .map_or(0, crate::hyperlink::EncodingUrlDocument::units);
                        self.units -= node
                            .base_document
                            .as_ref()
                            .map_or(0, crate::base::BaseDocument::units);
                        self.units -= form::name_units(node.form_past_names.as_ref(), cancel)?;
                        self.retire_form_relations(key, cancel)?;
                        self.slot_retirement(key, limits, cancel)?;
                        self.retire_reflected_target(key, cancel)?;
                        self.retire_template_patch_target(key, cancel)?;
                        if matches!(&node.kind, DomNodeKind::Element(element) if element.style_state().is_some())
                        {
                            self.retire_style_render_blocking(key, cancel)?;
                        }
                        self.push_effect(
                            select::Effect::Tree(Frame::Destroy {
                                closure,
                                index: index + 1,
                                stage,
                            }),
                            self.effect_scoped,
                            cancel,
                        )?;
                    }
                } else if stage < 2 {
                    self.push_effect(
                        select::Effect::Tree(Frame::Destroy {
                            closure,
                            index: 0,
                            stage: stage + 1,
                        }),
                        self.effect_scoped,
                        cancel,
                    )?;
                }
            }

            Frame::RemovedParent(parent) => self.children_changed(parent, limits, cancel)?,
            Frame::AdoptAfterRemoval { node, document } => {
                self.adopt_document(node, document, limits, cancel)?
            }
            Frame::Replace {
                node,
                parent,
                child,
                before,
                stage,
            } => {
                if stage == 0 {
                    self.push_effect(
                        select::Effect::Tree(Frame::Replace {
                            node,
                            parent,
                            child,
                            before,
                            stage: 1,
                        }),
                        self.effect_scoped,
                        cancel,
                    )?;
                    self.adopt(node, self.node(parent)?.document, limits, cancel)?;
                } else if stage == 1 {
                    self.push_effect(
                        select::Effect::Tree(Frame::Replace {
                            node,
                            parent,
                            child,
                            before,
                            stage: 2,
                        }),
                        self.effect_scoped,
                        cancel,
                    )?;
                    self.remove(child, limits, cancel)?;
                } else {
                    self.insert_validated(node, parent, before, limits, cancel)?;
                }
            }
            Frame::ReplaceAll {
                key,
                replacement,
                inserted,
            } => {
                if let Some(&child) = self.node(key)?.children.first().filter(|_| !inserted) {
                    self.push_effect(
                        select::Effect::Tree(Frame::ReplaceAll {
                            key,
                            replacement,
                            inserted,
                        }),
                        self.effect_scoped,
                        cancel,
                    )?;
                    self.remove(child, limits, cancel)?;
                } else if !inserted && let Some(text) = replacement {
                    self.pre_insert(text, key, None, limits, cancel)?;
                }
            }
            Frame::Walk {
                root,
                old_parent,
                mut pending,
            } => {
                if let Some(key) = pending.pop() {
                    #[cfg(test)]
                    if old_parent.is_none()
                        && let Some(trace) = &mut self.insertion_trace
                    {
                        trace.visited += 1;
                        if trace.visited == trace.cancel_after {
                            cancel.store(true, Ordering::Release);
                        }
                    }
                    self.effect_step(cancel)?;
                    let node = self.node(key)?;
                    self.select_reserve(&mut pending, node.children.len(), cancel)?;
                    pending.extend(node.children.iter().rev().copied());
                    if let Some(shadow) = node.shadow_root {
                        self.select_reserve(&mut pending, 1, cancel)?;
                        pending.push(shadow);
                    }
                    self.push_effect(
                        select::Effect::Tree(Frame::Walk {
                            root,
                            old_parent,
                            pending,
                        }),
                        self.effect_scoped,
                        cancel,
                    )?;
                    self.push_effect(
                        select::Effect::Tree(Frame::Node {
                            key,
                            root,
                            old_parent,
                            stage: 0,
                        }),
                        self.effect_scoped,
                        cancel,
                    )?;
                }
            }
            Frame::Node {
                key,
                root,
                old_parent,
                stage,
            } => {
                if let Some(parent) = old_parent {
                    if stage < 9 {
                        self.push_effect(
                            select::Effect::Tree(Frame::Node {
                                key,
                                root,
                                old_parent,
                                stage: stage + 1,
                            }),
                            self.effect_scoped,
                            cancel,
                        )?;
                    }
                    match stage {
                        0 => self.navigable_removing_node(key, cancel)?,
                        1 => self.script_removing_node(key, cancel)?,
                        2 => self.select_removal(key, limits, cancel)?,
                        3 => self.image_tree_change(
                            key,
                            surgeist_html::HtmlImageTreeChange::Removed,
                            root,
                            Some(parent),
                            cancel,
                        )?,
                        4 => self.control_removing_node(key, cancel)?,
                        5 => self.base_tree_change(key, limits, cancel)?,
                        6 => self.hyperlink_tree_change(key, Some(parent), cancel)?,
                        7 => self.style_tree_change(key, old_parent, limits, cancel)?,
                        9 => self.dialog_removal(key, limits, cancel)?,
                        8 => {
                            if matches!(&self.node(key)?.kind, DomNodeKind::Element(element) if element.is_meta())
                            {
                                // The shadow-inclusive removal walk also visits
                                // hosted trees. Only a Meta whose ORDINARY root
                                // was the removed root belonged to the former
                                // ordinary Document tree.
                                let old_document =
                                    if crate::slot::root_admitted(self, key, &mut || {
                                        self.meta_step(cancel)
                                    })? == root
                                    {
                                        let former_root =
                                            crate::slot::root_admitted(self, parent, &mut || {
                                                self.meta_step(cancel)
                                            })?;
                                        matches!(
                                            self.node(former_root)?.kind,
                                            DomNodeKind::Document(_)
                                        )
                                        .then_some(former_root)
                                    } else {
                                        None
                                    };
                                self.meta_removed(key, old_document, limits, cancel)?;
                            }
                        }
                        _ => (),
                    }
                } else {
                    if stage < 10 {
                        self.push_effect(
                            select::Effect::Tree(Frame::Node {
                                key,
                                root,
                                old_parent,
                                stage: stage + 1,
                            }),
                            self.effect_scoped,
                            cancel,
                        )?;
                    }
                    match stage {
                        0 => self.insertion_nonce(key, cancel)?,
                        1 => self.control_insertion_node(key, cancel)?,
                        2 => self.image_tree_change(
                            key,
                            surgeist_html::HtmlImageTreeChange::Inserted,
                            root,
                            None,
                            cancel,
                        )?,
                        3 => self.base_tree_change(key, limits, cancel)?,
                        4 => self.hyperlink_tree_change(key, None, cancel)?,
                        5 => self.select_insertion(key, limits, cancel)?,
                        6 => self.details_exclusivity(
                            key,
                            surgeist_html::HtmlDetailsExclusivity::CloseGiven,
                            limits,
                            cancel,
                        )?,
                        7 => self.style_tree_change(key, None, limits, cancel)?,
                        8 => self.meta_inserted(key, limits, cancel)?,
                        9 => self.dialog_insertion(key, limits, cancel)?,
                        10 => self.autofocus_insertion(key, cancel)?,
                        _ => (),
                    }
                }
            }
            Frame::Insert(mut frame) => {
                if frame.stage == 0 {
                    if frame.fragment
                        && let Some(&child) = frame.moving.get(frame.index)
                    {
                        frame.index += 1;
                        self.push_effect(
                            select::Effect::Tree(Frame::Insert(frame)),
                            self.effect_scoped,
                            cancel,
                        )?;
                        self.remove(child, limits, cancel)?;
                    } else {
                        frame.stage = 1;
                        frame.index = 0;
                        // DOM insert adjusts by the complete insertion list
                        // before the first per-node adoption/insertion hook.
                        self.range_insert(frame.parent, frame.before, frame.moving.len(), cancel)?;
                        self.push_effect(
                            select::Effect::Tree(Frame::Insert(frame)),
                            self.effect_scoped,
                            cancel,
                        )?;
                    }
                } else if let Some(&child) = frame.moving.get(frame.index) {
                    if frame.stage == 1 {
                        frame.stage = 2;
                        let document = self.node(frame.parent)?.document;
                        self.push_effect(
                            select::Effect::Tree(Frame::Insert(frame)),
                            self.effect_scoped,
                            cancel,
                        )?;
                        self.adopt(child, document, limits, cancel)?;
                    } else {
                        let target = self.node_mut(frame.parent);
                        let index = frame.before.map_or(target.children.len(), |reference| {
                            target
                                .children
                                .iter()
                                .position(|key| *key == reference)
                                .expect("retained reference")
                        });
                        target.children.insert(index, child);
                        self.node_mut(child).parent = Some(frame.parent);
                        if !self.is_private(frame.parent) {
                            self.expose_source_subtree(child, cancel)?;
                        }
                        self.slot_insertion(child, frame.parent, limits, cancel)?;
                        frame.index += 1;
                        frame.stage = 1;
                        self.push_effect(
                            select::Effect::Tree(Frame::Insert(frame)),
                            self.effect_scoped,
                            cancel,
                        )?;
                        let mut pending = self.effect_vec();
                        self.select_reserve(&mut pending, 1, cancel)?;
                        pending.push(child);
                        self.push_effect(
                            select::Effect::Tree(Frame::Walk {
                                root: child,
                                old_parent: None,
                                pending,
                            }),
                            self.effect_scoped,
                            cancel,
                        )?;
                    }
                } else if frame.stage != 3 {
                    frame.stage = 3;
                    let parent = frame.parent;
                    self.push_effect(
                        select::Effect::Tree(Frame::Insert(frame)),
                        self.effect_scoped,
                        cancel,
                    )?;
                    self.children_changed(parent, limits, cancel)?;
                } else if !frame.moving.is_empty() {
                    let mut roots = self.effect_vec();
                    self.select_reserve(&mut roots, frame.moving.len(), cancel)?;
                    roots.extend(frame.moving.iter().copied());
                    self.push_effect(
                        select::Effect::Tree(Frame::ReleaseMoving(frame)),
                        self.effect_scoped,
                        cancel,
                    )?;
                    self.native_post_connection_steps(&roots, limits, cancel)?;
                }
            }
            Frame::ReleaseMoving(frame) => {
                drop(frame);
            }
        }
        Ok(())
    }
    fn insertion_nonce(&self, key: Key, cancel: &AtomicBool) -> Result<(), DomError> {
        use surgeist_html::{
            HtmlBrowsingContextPresence, HtmlHeaderDeliveredCsp, HtmlNonceConnection,
            HtmlNonceInsertionEffect,
        };
        let node = self.node(key)?;
        let DomNodeKind::Element(element) = &node.kind else {
            return Ok(());
        };
        let DomNodeKind::Document(document) = &self.node(node.document)?.kind else {
            unreachable!("canonical node document")
        };
        // This is an established native environment fact, independent of
        // class or connectedness. No CSP observation is invented.
        let connection = match document.document_environment().browsing_context() {
            HtmlBrowsingContextPresence::Absent => HtmlNonceConnection::NotBrowsingContextConnected,
            HtmlBrowsingContextPresence::Unavailable | HtmlBrowsingContextPresence::Present => {
                HtmlNonceConnection::Unavailable
            }
        };
        let mut content = None;
        for attribute in element.attributes().iter() {
            self.effect_step(cancel)?;
            if attribute.name().namespace().uri().is_none()
                && attribute.name().local_name().code_units() == [0x6e, 0x6f, 0x6e, 0x63, 0x65]
            {
                content = Some(attribute.value());
                break;
            }
        }
        match surgeist_html::nonce_insertion_effect(
            element.name(),
            content,
            connection,
            HtmlHeaderDeliveredCsp::Unavailable,
        ) {
            HtmlNonceInsertionEffect::KeepContent => {}
            HtmlNonceInsertionEffect::Unavailable(missing) => {
                return Err(DomError::NonceEnvironmentUnavailable(missing));
            }
            HtmlNonceInsertionEffect::HideContentPreservingSlot => {
                unreachable!("native null browsing context cannot hide nonce content")
            }
        }
        Ok(())
    }
}

#[cfg(test)]
#[derive(Clone, Copy)]
pub(super) struct InsertionTrace {
    pub visited: usize,
    pub cancel_after: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cancellation_during_common_descendant_insertion_keeps_publication_unchanged() {
        let mut dom = Dom::new(DomLimits::new(16, 2048, 32, 0, 0, 8, 8192).unwrap()).unwrap();
        let cancel = AtomicBool::new(false);
        let mut batch = DomBatch::new();
        let document = batch.create_document();
        let parent = batch.create_element_ns(
            &document,
            crate::Namespace::from_uri("http://www.w3.org/1999/xhtml"),
            "div",
        );
        let child = batch.create_element_ns(
            &document,
            crate::Namespace::from_uri("http://www.w3.org/2000/svg"),
            "g",
        );
        batch.set_attribute(&child, "nonce", "child");
        batch.insert_before(&child, &parent, None);
        let commit = dom.apply(batch, &cancel).unwrap();
        let parent = commit.created(&parent).unwrap().unwrap();
        let before = dom.snapshot();
        let usage = dom.usage();
        let mut draft = Draft::new(&dom, Arc::new(BatchIdentity), &AtomicBool::new(false)).unwrap();
        let key = draft.resolve(parent.into()).unwrap();
        draft
            .set_nonce(key, &"unpublished".into(), dom.limits(), &cancel)
            .unwrap();
        draft.insertion_trace = Some(InsertionTrace {
            visited: 0,
            cancel_after: 2,
        });
        let document = draft.node(key).unwrap().document;
        assert_eq!(
            draft.pre_insert(key, document, None, dom.limits(), &cancel),
            Err(DomError::Cancelled)
        );
        assert_eq!(draft.insertion_trace.as_ref().unwrap().visited, 2);
        drop(draft);
        assert_eq!(*dom.state, *before.state);
        assert_eq!(dom.usage(), usage);
    }
}

// A test-only independent observation of the actual moving allocation's full
// lifetime. Charge already present in the live ledger is subtracted once, so a
// tracked bank is never double-counted. This hook changes no admission decision.
#[cfg(test)]
pub(super) mod scratch_observation {
    use std::cell::Cell;
    thread_local! {
        static MISSING_MOVING: Cell<usize> = const { Cell::new(0) };
        static GAP: Cell<bool> = const { Cell::new(false) };
        static GROWTH_GAP: Cell<bool> = const { Cell::new(false) };
    }
    pub struct Moving(usize);
    impl Moving {
        pub fn new(capacity_bytes: usize, registered_bytes: usize) -> Self {
            let missing = capacity_bytes.saturating_sub(registered_bytes);
            MISSING_MOVING.set(MISSING_MOVING.get() + missing);
            Self(missing)
        }
    }
    impl Drop for Moving {
        fn drop(&mut self) {
            MISSING_MOVING.set(MISSING_MOVING.get() - self.0);
        }
    }
    pub fn observe(admitted: usize, limit: usize) {
        if admitted <= limit && admitted + MISSING_MOVING.get() > limit {
            GAP.set(true);
        }
    }
    pub fn observe_growth(before: usize, actual_new: usize, after: usize, limit: usize) {
        if after <= limit && before + actual_new > limit {
            GROWTH_GAP.set(true);
        }
    }
    pub fn reset() {
        assert_eq!(MISSING_MOVING.get(), 0);
        GAP.set(false);
        GROWTH_GAP.set(false);
    }
    pub fn gap() -> bool {
        GAP.get()
    }
    pub fn growth_gap() -> bool {
        GROWTH_GAP.get()
    }
}

#[cfg(test)]
mod scratch_lifetime_tests {
    use super::*;
    fn attempt(scratch: usize, deep: bool) -> (bool, bool, bool) {
        let cancel = AtomicBool::new(false);
        let configured = DomLimits::new(2048, 32768, 4096, 4096, 4096, 16, 1_000_000)
            .unwrap()
            .with_select_processing(128, 4_000_000, 10_000_000)
            .unwrap();
        let mut dom = Dom::new(configured).unwrap();
        let mut b = DomBatch::new();
        let d = b.create_document();
        let select = b.create_element_ns(&d, crate::Namespace::html(), "select");
        let option = b.create_element_ns(&d, crate::Namespace::html(), "option");
        if deep {
            let mut chain = Vec::new();
            for _ in 0..32 {
                chain.push(b.create_element_ns(&d, crate::Namespace::html(), "span"));
            }
            b.insert_before(&chain[0], &option, None);
            for pair in chain.windows(2) {
                b.insert_before(&pair[1], &pair[0], None);
            }
            let text = b.create_text(&d, "x");
            b.insert_before(&text, chain.last().unwrap(), None);
        } else {
            for _ in 0..64 {
                let text = b.create_text(&d, "x");
                b.insert_before(&text, &option, None);
            }
        }
        let content = b.create_element_ns(&d, crate::Namespace::html(), "selectedcontent");
        b.insert_before(&option, &select, None);
        b.insert_before(&content, &select, None);
        b.insert_before(&select, &d, None);
        let c = dom.apply(b, &cancel).unwrap();
        let select = c.created(&select).unwrap().unwrap();
        let content = c.created(&content).unwrap().unwrap();
        dom.set_limits(
            configured
                .with_select_processing(128, scratch, 10_000_000)
                .unwrap(),
        )
        .unwrap();
        let before = dom.snapshot();
        let usage = dom.usage();
        scratch_observation::reset();
        let mut b = DomBatch::new();
        b.set_select_selected_index(select, 0);
        let result = dom.apply(b, &cancel);
        match &result {
            Ok(_) => {
                let s = dom.snapshot();
                assert_eq!(
                    s.node(content).unwrap().children().len(),
                    if deep { 1 } else { 64 }
                );
                if deep {
                    let mut node = s.node(content).unwrap().children()[0];
                    for _ in 0..32 {
                        node = s.node(node).unwrap().children()[0];
                    }
                    assert_eq!(s.node(node).unwrap().kind(), &DomNodeKind::Text("x".into()));
                } else {
                    for id in s.node(content).unwrap().children() {
                        assert_eq!(s.node(*id).unwrap().kind(), &DomNodeKind::Text("x".into()));
                    }
                }
            }
            Err(error) => {
                assert_eq!(
                    *error,
                    DomError::ResourceLimit {
                        resource: DomResource::SelectScratch,
                        limit: scratch
                    }
                );
                assert_eq!(dom.snapshot().stamp(), before.stamp());
                assert_eq!(dom.usage(), usage);
                assert_eq!(
                    dom.snapshot().node(content).unwrap().children(),
                    before.node(content).unwrap().children()
                );
            }
        }
        (
            result.is_ok(),
            scratch_observation::gap(),
            scratch_observation::growth_gap(),
        )
    }
    fn boundary(deep: bool) {
        // Discover an actually accepted boundary rather than asserting an
        // incidental byte constant or requiring these allocations forever.
        let (mut low, mut high) = (0, 4_000_000);
        while low < high {
            let middle = low + (high - low) / 2;
            if attempt(middle, deep).0 {
                high = middle;
            } else {
                low = middle + 1;
            }
        }
        let (committed, moving_gap, growth_gap) = attempt(low, deep);
        eprintln!(
            "deep={deep}; minimum accepted scratch={low}; committed={committed}; moving gap={moving_gap}; actual frame growth gap={growth_gap}"
        );
        assert!(committed);
        assert!(
            !moving_gap,
            "committed while the actual live moving bank exceeded admission"
        );
        assert!(
            !growth_gap,
            "committed while old plus actual new frame capacity exceeded admission"
        );
        if low > 0 {
            assert!(!attempt(low - 1, deep).0);
        }
    }
    #[test]
    fn moving_capacity_remains_admitted_through_every_later_insert_reservation() {
        boundary(false);
    }
    #[test]
    fn nested_copy_admits_actual_effect_frame_growth_overlap() {
        boundary(true);
    }
}
