//! Intrinsic select state and construction work owned by the existing arena.
//! Governing HTML 877d6146487f: select/option/selectedcontent processing.

use super::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct SelectState {
    pub(crate) user_validity: bool,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct OptionState {
    pub(crate) selectedness: bool,
    pub(crate) dirtiness: bool,
    nearest_select: Option<usize>,
}
impl OptionState {
    pub(super) fn fresh() -> Self {
        Self {
            selectedness: false,
            dirtiness: false,
            nearest_select: None,
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct SelectedcontentState {
    pub(crate) disabled: bool,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum SelectMetadata {
    Select(SelectState),
    Option(OptionState),
    Selectedcontent(SelectedcontentState),
}
pub(super) fn initial_metadata(
    c: &mut ParseControl,
    tag: &Tag,
) -> Result<Option<SelectMetadata>, SessionError> {
    for (name, state) in [
        (
            "select",
            Some(SelectMetadata::Select(SelectState {
                user_validity: false,
            })),
        ),
        (
            "option",
            Some(SelectMetadata::Option(OptionState {
                selectedness: false,
                dirtiness: false,
                nearest_select: None,
            })),
        ),
        (
            "selectedcontent",
            Some(SelectMetadata::Selectedcontent(SelectedcontentState {
                disabled: false,
            })),
        ),
        ("optgroup", None),
    ] {
        if exact_ascii(c, Some(tag.name.value.text()), name)? {
            return Ok(state);
        }
    }
    Err(SessionError::WrongToken)
}
#[derive(Clone, Copy)]
pub(super) enum IntrinsicTask {
    UpdateSelect(usize),
}

/// Finite records reference actual admitted work-list slots in the registry.
/// No frame aliases native backing or owns an independent tree/controller.
#[derive(Clone, Copy)]
pub(super) enum EffectFrame {
    AdoptSubtree {
        root: usize,
        next: Option<usize>,
        document: usize,
    },
    PatchRemoval {
        list: usize,
        cursor: usize,
    },
    Topology(FormEffects),
    UpdateSelect {
        select: usize,
        list: usize,
        cursor: usize,
    },
    UpdateTarget {
        option: Option<usize>,
        target: usize,
        fragment: Option<usize>,
        stage: TargetStage,
    },
    CloneChildren {
        source: usize,
        copy: usize,
        cursor: usize,
        document: usize,
    },
    CloneOptionChildren {
        option: usize,
        fragment: usize,
        cursor: usize,
    },
    AppendClone {
        parent: usize,
        copy: usize,
    },
    PostConnection {
        list: usize,
        cursor: usize,
        cause: InsertionCause,
    },
    ScriptPreparation {
        node: usize,
        detached: Option<usize>,
    },
    FragmentInsertion {
        parent: usize,
        list: usize,
        cursor: usize,
        stage: FragmentStage,
    },
    InsertionPostConnection {
        roots: PostConnectionRoots,
        cause: InsertionCause,
    },
}
#[derive(Clone, Copy)]
pub(super) enum FragmentStage {
    Remove,
    Insert,
}
#[derive(Clone, Copy)]
pub(super) enum PostConnectionRoots {
    Single(usize),
    Fragment(usize),
}
#[derive(Clone, Copy)]
pub(super) enum TargetStage {
    Prepare,
    Replace,
    Insert,
}
#[derive(Clone, Copy)]
enum WorkListKind {
    PostConnection,
    FragmentChildren,
    SelectedTargets,
    Options { detached: Option<usize> },
}

pub(super) struct ConstructionEffects {
    pub(super) form: Option<FormEffects>,
    frames: OwnedVec<EffectFrame>,
    current: Option<EffectFrame>,
    transfer: Option<EffectFrame>,
    tasks: OwnedVec<Option<QueuedTask>>,
    dequeue: usize,
    task: Option<QueuedTask>,
    checkpoint: bool,
    pub(super) svg_wrapper: Option<usize>,
}
struct QueuedTask {
    task: IntrinsicTask,
    cause: Option<super::provenance::ConstructionCauseRef>,
}
impl ConstructionEffects {
    pub(super) fn new(c: &mut ParseControl) -> Result<Self, SessionError> {
        payload_write::<Self>(c, 9)?;
        Ok(Self {
            form: None,
            frames: OwnedVec::new(HtmlParseResource::AuxiliaryEntries)?,
            current: None,
            transfer: None,
            tasks: OwnedVec::new(HtmlParseResource::AuxiliaryEntries)?,
            dequeue: 0,
            task: None,
            checkpoint: false,
            svg_wrapper: None,
        })
    }
    pub(super) fn require_idle(&self, c: &mut ParseControl) -> Result<(), SessionError> {
        c.charge(WorkCategory::RecordVisit, 5)?;
        if self.svg_wrapper.is_some()
            || self.current.is_some()
            || self.transfer.is_some()
            || self.frames.len() != 0
            || self.checkpoint
        {
            return Err(c
                .fail_invariant(crate::parser::control::Invariant::PendingFormEffects)
                .into());
        }
        Ok(())
    }
    pub(super) fn require_freezable(&self, c: &mut ParseControl) -> Result<(), SessionError> {
        c.charge(WorkCategory::RecordVisit, 8)?;
        if self.svg_wrapper.is_some()
            || self.dequeue != self.tasks.len()
            || self.task.is_some()
            || self.checkpoint
            || self.current.is_some()
            || self.transfer.is_some()
            || self.frames.len() != 0
            || self.form.is_some()
        {
            return Err(SessionError::PendingIntrinsicWork);
        }
        Ok(())
    }
    #[cfg(test)]
    pub(super) fn retained_post_connection(&self) -> Option<(usize, usize)> {
        match self.current {
            Some(EffectFrame::PostConnection { list, cursor, .. }) => Some((list, cursor)),
            _ => None,
        }
    }
    pub(super) fn release(self, c: &mut ParseControl) -> Result<(), ParseStop> {
        self.frames.release(c)?;
        self.tasks.release(c)
    }
    fn enqueue(
        &mut self,
        c: &mut ParseControl,
        task: IntrinsicTask,
        cause: Option<super::provenance::ConstructionCauseRef>,
    ) -> Result<(), SessionError> {
        // Tasks carry no nested allocation; failed push returns the finite value.
        self.tasks
            .try_push_recover(c, Some(QueuedTask { task, cause }))
            .map_err(|(stop, _)| SessionError::Stop(stop))
    }
}

impl SessionStorage {
    pub(super) fn fragment_insertion_parent(
        &self,
        c: &mut ParseControl,
        parent: Option<usize>,
    ) -> Result<bool, SessionError> {
        c.charge(WorkCategory::RecordVisit, 1)?;
        if let Some(EffectFrame::FragmentInsertion {
            parent: actual,
            stage: FragmentStage::Insert,
            ..
        }) = self.effects.current.as_ref()
        {
            c.charge(WorkCategory::CopyByte, mem::size_of::<usize>() as u64)?;
            return Ok(parent == Some(*actual));
        }
        Ok(false)
    }
    pub(super) fn start_effect(
        &mut self,
        c: &mut ParseControl,
        frame: EffectFrame,
    ) -> Result<(), SessionError> {
        // The receiving transfer owns the child before either parent is moved.
        payload_write::<Option<EffectFrame>>(c, 1)?;
        if self.effects.transfer.is_some() {
            return Err(c
                .fail_invariant(crate::parser::control::Invariant::PendingFormEffects)
                .into());
        }
        self.effects.transfer = Some(frame);
        if let Some(parent) = self.effects.current {
            payload_write::<Option<EffectFrame>>(c, 1)?;
            self.effects
                .frames
                .try_push_recover(c, parent)
                .map_err(|(stop, _)| SessionError::Stop(stop))?;
            self.effects.current = None;
        }
        if let Some(parent) = self.effects.form {
            payload_write::<Option<FormEffects>>(c, 1)?;
            self.effects
                .frames
                .try_push_recover(c, EffectFrame::Topology(parent))
                .map_err(|(stop, _)| SessionError::Stop(stop))?;
            self.effects.form = None;
        }
        payload_write::<Option<EffectFrame>>(c, 2)?;
        self.effects.current = self.effects.transfer.take();
        Ok(())
    }
    fn write_current_effect(
        &mut self,
        c: &mut ParseControl,
        frame: Option<EffectFrame>,
    ) -> Result<(), SessionError> {
        payload_write::<Option<EffectFrame>>(c, 1)?;
        self.effects.current = frame;
        Ok(())
    }
    pub(super) fn drain_construction_effects(
        &mut self,
        c: &mut ParseControl,
    ) -> Result<(), SessionError> {
        let result = (|| {
            loop {
                c.require_identity(&self.control)?;
                c.charge(WorkCategory::RecordVisit, 2)?;
                if self.effects.form.is_some() {
                    self.step_form_effects(c)?;
                    continue;
                }
                if self.effects.current.is_some() {
                    c.admit_atomic_layout(mem::size_of::<EffectFrame>())?;
                    c.charge(WorkCategory::CopyByte, mem::size_of::<EffectFrame>() as u64)?;
                    let frame = self.effects.current.expect("admitted present frame");
                    self.step_effect(c, frame)?;
                    continue;
                }
                payload_write::<Option<EffectFrame>>(c, 1)?;
                let frame = self.effects.frames.try_pop(c)?;
                if let Some(EffectFrame::Topology(form)) = frame {
                    payload_write::<Option<FormEffects>>(c, 1)?;
                    self.effects.form = Some(form);
                } else if let Some(frame) = frame {
                    self.write_current_effect(c, Some(frame))?;
                } else {
                    return Ok(());
                }
            }
        })();
        match result {
            Err(SessionError::Stop(stop)) => Err(SessionError::Stop(stop)),
            Err(_) => Err(c
                .fail_invariant(crate::parser::control::Invariant::PendingFormEffects)
                .into()),
            Ok(()) => Ok(()),
        }
    }
    /// Initial attribute ID changes cannot create child construction frames.
    /// They use the same canonical phase owner while clone creation is suspended.
    pub(super) fn drain_form_effects(&mut self, c: &mut ParseControl) -> Result<(), SessionError> {
        while let Some(form) = self.effects.form {
            c.charge(WorkCategory::RecordVisit, 1)?;
            if form.stage != FormEffectStage::IdChange {
                return self.drain_construction_effects(c);
            }
            self.step_form_effects(c)?;
        }
        Ok(())
    }
    fn step_effect(
        &mut self,
        c: &mut ParseControl,
        frame: EffectFrame,
    ) -> Result<(), SessionError> {
        match frame {
            EffectFrame::AdoptSubtree {
                root,
                next,
                document,
            } => {
                let Some(node) = next else {
                    return self.write_current_effect(c, None);
                };
                self.set_node_document(c, node, document)?;
                let next = self.next_view_node(c, root, node, None)?;
                self.write_current_effect(
                    c,
                    Some(EffectFrame::AdoptSubtree {
                        root,
                        next,
                        document,
                    }),
                )?;
                c.charge(WorkCategory::RecordVisit, 1)?;
                if self.records[node].template_contents.is_some() {
                    let (_, template) = self.template_slot(c, node)?;
                    let target = self.appropriate_contents_document(c, document)?;
                    if self.node_document_index(c, template.contents)? != target {
                        self.start_effect(
                            c,
                            EffectFrame::AdoptSubtree {
                                root: template.contents,
                                next: Some(template.contents),
                                document: target,
                            },
                        )?;
                    }
                }
                Ok(())
            }
            EffectFrame::PatchRemoval { list, cursor } => {
                let Some(node) = self.work_list_node(c, list, cursor)? else {
                    self.release_work_list(c, list)?;
                    return self.write_current_effect(c, None);
                };
                self.write_current_effect(
                    c,
                    Some(EffectFrame::PatchRemoval {
                        list,
                        cursor: cursor + 1,
                    }),
                )?;
                c.charge(WorkCategory::RecordVisit, 1)?;
                if self.records[node].parent.is_some() {
                    self.remove_index(c, node)?;
                }
                Ok(())
            }
            EffectFrame::Topology(_) => Err(SessionError::WrongToken),
            EffectFrame::UpdateSelect {
                select,
                list,
                cursor,
            } => {
                let Some(target) = self.work_list_node(c, list, cursor)? else {
                    self.release_work_list(c, list)?;
                    return self.write_current_effect(c, None);
                };
                self.write_current_effect(
                    c,
                    Some(EffectFrame::UpdateSelect {
                        select,
                        list,
                        cursor: cursor + 1,
                    }),
                )?;
                let (_, SelectMetadata::Selectedcontent(state)) =
                    self.select_metadata(c, target)?
                else {
                    return Err(SessionError::SelectMetadataUnavailable);
                };
                if state.disabled {
                    return Ok(());
                }
                let option = self.first_selected_option(c, select)?;
                self.start_effect(
                    c,
                    EffectFrame::UpdateTarget {
                        option,
                        target,
                        fragment: None,
                        stage: TargetStage::Prepare,
                    },
                )
            }
            EffectFrame::UpdateTarget {
                option,
                target,
                stage: TargetStage::Prepare,
                ..
            } => {
                if let Some(option) = option {
                    self.qualify_clone_subtree(c, option)?;
                }
                let document = option
                    .map(|option| self.node_document_index(c, option))
                    .transpose()?;
                let fragment = if option.is_some() {
                    Some(
                        self.allocate_in_document_owned(
                            c,
                            HtmlNodeKind::Fragment,
                            false,
                            0,
                            0,
                            document,
                        )?
                        .index,
                    )
                } else {
                    None
                };
                self.write_current_effect(
                    c,
                    Some(EffectFrame::UpdateTarget {
                        option,
                        target,
                        fragment,
                        stage: TargetStage::Replace,
                    }),
                )?;
                if let (Some(option), Some(fragment)) = (option, fragment) {
                    self.start_effect(
                        c,
                        EffectFrame::CloneOptionChildren {
                            option,
                            fragment,
                            cursor: 0,
                        },
                    )?;
                }
                Ok(())
            }
            EffectFrame::UpdateTarget {
                option,
                target,
                fragment,
                stage: TargetStage::Replace,
            } => {
                c.charge(WorkCategory::RecordVisit, 1)?;
                if self.records[target].children.is_empty() {
                    if fragment.is_none() {
                        return self.write_current_effect(c, None);
                    }
                    return self.write_current_effect(
                        c,
                        Some(EffectFrame::UpdateTarget {
                            option,
                            target,
                            fragment,
                            stage: TargetStage::Insert,
                        }),
                    );
                }
                c.charge(WorkCategory::RecordVisit, 1)?;
                let child = self.records[target].children[0];
                self.remove_index(c, child)
            }
            EffectFrame::UpdateTarget {
                target,
                fragment: Some(fragment),
                stage: TargetStage::Insert,
                ..
            } => {
                // DOM insert snapshots the fragment's top-level nodes, removes
                // all of them, then finishes all sibling insertion hooks before
                // one parent childrenChanged and static post-connection list.
                let list = self.retain_work_list(c, fragment, WorkListKind::FragmentChildren)?;
                self.write_current_effect(
                    c,
                    Some(EffectFrame::FragmentInsertion {
                        parent: target,
                        list,
                        cursor: 0,
                        stage: FragmentStage::Remove,
                    }),
                )
            }
            EffectFrame::UpdateTarget { fragment: None, .. } => Err(SessionError::WrongToken),
            EffectFrame::CloneOptionChildren {
                option,
                fragment,
                cursor,
            } => {
                c.charge(WorkCategory::RecordVisit, 1)?;
                if cursor == self.records[option].children.len() {
                    return self.write_current_effect(c, None);
                }
                c.charge(WorkCategory::RecordVisit, 1)?;
                let child = self.records[option].children[cursor];
                let document = self.node_document_index(c, child)?;
                let copy = self.clone_node(c, child, document)?;
                self.write_current_effect(
                    c,
                    Some(EffectFrame::CloneOptionChildren {
                        option,
                        fragment,
                        cursor: cursor + 1,
                    }),
                )?;
                self.start_effect(
                    c,
                    EffectFrame::AppendClone {
                        parent: fragment,
                        copy,
                    },
                )?;
                self.start_effect(
                    c,
                    EffectFrame::CloneChildren {
                        source: child,
                        copy,
                        cursor: 0,
                        document,
                    },
                )?;
                self.start_template_clone(c, child, copy)
            }
            EffectFrame::AppendClone { parent, copy } => {
                self.write_current_effect(c, None)?;
                self.insert_indices(c, parent, copy, None)
            }
            EffectFrame::CloneChildren {
                source,
                copy,
                cursor,
                document,
            } => {
                c.charge(WorkCategory::RecordVisit, 1)?;
                if cursor == self.records[source].children.len() {
                    return self.write_current_effect(c, None);
                }
                c.charge(WorkCategory::RecordVisit, 1)?;
                let child = self.records[source].children[cursor];
                let copied = self.clone_node(c, child, document)?;
                self.write_current_effect(
                    c,
                    Some(EffectFrame::CloneChildren {
                        source,
                        copy,
                        cursor: cursor + 1,
                        document,
                    }),
                )?;
                // The source shallow clone is appended before its subtree,
                // exactly as DOM clone specifies. Insert's intrinsic effects
                // precede the retained child traversal.
                self.start_effect(
                    c,
                    EffectFrame::CloneChildren {
                        source: child,
                        copy: copied,
                        cursor: 0,
                        document,
                    },
                )?;
                if self.records[child].template_contents.is_some() {
                    self.start_effect(
                        c,
                        EffectFrame::AppendClone {
                            parent: copy,
                            copy: copied,
                        },
                    )?;
                    self.start_template_clone(c, child, copied)
                } else {
                    self.insert_indices(c, copy, copied, None)
                }
            }
            EffectFrame::PostConnection {
                list,
                cursor,
                cause,
            } => {
                let Some(node) = self.work_list_node(c, list, cursor)? else {
                    self.release_work_list(c, list)?;
                    return self.write_current_effect(c, None);
                };
                self.write_current_effect(
                    c,
                    Some(EffectFrame::PostConnection {
                        list,
                        cursor: cursor + 1,
                        cause,
                    }),
                )?;
                if !self.connected(c, node, None)? {
                    return Ok(());
                }
                if self.svg_script_local(c, node)? && cause == InsertionCause::NonParser {
                    self.process_svg_owned(c, node)?;
                }
                if self.html_local(c, node, "option")? {
                    self.update_for_option(c, node)?;
                } else if self.html_local(c, node, "selectedcontent")? {
                    self.connect_selectedcontent(c, node)?;
                } else if self.html_local(c, node, "script")? {
                    self.start_effect(
                        c,
                        EffectFrame::ScriptPreparation {
                            node,
                            detached: None,
                        },
                    )?;
                }
                Ok(())
            }
            EffectFrame::ScriptPreparation { node, detached } => {
                let (_, state) = script_metadata(&self.payloads, c, node)?;
                if state.parser_document.is_none() {
                    self.prepare_script_owned(c, node, detached)?;
                }
                self.write_current_effect(c, None)
            }
            EffectFrame::FragmentInsertion {
                parent,
                list,
                cursor,
                stage,
            } => {
                let node = self.work_list_node(c, list, cursor)?;
                match (stage, node) {
                    (FragmentStage::Remove, Some(node)) => {
                        self.write_current_effect(
                            c,
                            Some(EffectFrame::FragmentInsertion {
                                parent,
                                list,
                                cursor: cursor + 1,
                                stage,
                            }),
                        )?;
                        self.remove_index(c, node)
                    }
                    (FragmentStage::Remove, None) => self.write_current_effect(
                        c,
                        Some(EffectFrame::FragmentInsertion {
                            parent,
                            list,
                            cursor: 0,
                            stage: FragmentStage::Insert,
                        }),
                    ),
                    (FragmentStage::Insert, Some(node)) => {
                        self.write_current_effect(
                            c,
                            Some(EffectFrame::FragmentInsertion {
                                parent,
                                list,
                                cursor: cursor + 1,
                                stage,
                            }),
                        )?;
                        self.insert_indices(c, parent, node, None)
                    }
                    (FragmentStage::Insert, None) => {
                        self.refresh_textarea(c, parent, None, false)?;
                        self.write_current_effect(
                            c,
                            Some(EffectFrame::InsertionPostConnection {
                                roots: PostConnectionRoots::Fragment(list),
                                cause: InsertionCause::NonParser,
                            }),
                        )?;
                        if self.html_local(c, parent, "script")?
                            && self.connected(c, parent, None)?
                        {
                            self.start_effect(
                                c,
                                EffectFrame::ScriptPreparation {
                                    node: parent,
                                    detached: None,
                                },
                            )?;
                        }
                        Ok(())
                    }
                }
            }
            EffectFrame::InsertionPostConnection { roots, cause } => {
                let post = match roots {
                    PostConnectionRoots::Single(root) => {
                        self.post_connection_frame_with_cause(c, root, cause)?
                    }
                    PostConnectionRoots::Fragment(roots) => {
                        let list = self.retain_fragment_post_connection(c, roots)?;
                        self.release_work_list(c, roots)?;
                        EffectFrame::PostConnection {
                            list,
                            cursor: 0,
                            cause,
                        }
                    }
                };
                self.write_current_effect(c, Some(post))
            }
        }
    }

    fn retain_work_list(
        &mut self,
        c: &mut ParseControl,
        root: usize,
        kind: WorkListKind,
    ) -> Result<usize, SessionError> {
        let list = OwnedVec::new(HtmlParseResource::AuxiliaryEntries)?;
        if let Err(error) = self.prepare_payload_slot_owned(c) {
            list.release(c)?;
            return Err(error);
        }
        let slot = self.payloads.values.len();
        self.payloads
            .values
            .push(RetainedPayload::WorkList(Some(list)));
        let detached = match kind {
            WorkListKind::Options { detached } => detached,
            _ => None,
        };
        let mut next = match kind {
            WorkListKind::PostConnection => Some(root),
            _ => self.next_view_node(c, root, root, detached)?,
        };
        while let Some(node) = next {
            let include = match kind {
                WorkListKind::PostConnection | WorkListKind::FragmentChildren => true,
                WorkListKind::SelectedTargets => {
                    if self.html_local(c, node, "selectedcontent")? {
                        let (_, SelectMetadata::Selectedcontent(state)) =
                            self.select_metadata(c, node)?
                        else {
                            return Err(SessionError::SelectMetadataUnavailable);
                        };
                        !state.disabled
                    } else {
                        false
                    }
                }
                WorkListKind::Options { .. } => self.html_local(c, node, "option")?,
            };
            if include {
                let RetainedPayload::WorkList(Some(list)) = &mut self.payloads.values[slot] else {
                    unreachable!()
                };
                list.try_push_recover(c, node)
                    .map_err(|(stop, _)| SessionError::Stop(stop))?;
            }
            let excluded = matches!(kind, WorkListKind::FragmentChildren)
                || (matches!(kind, WorkListKind::Options { .. })
                    && (include
                        || self.html_local(c, node, "select")?
                        || self.html_local(c, node, "hr")?
                        || self.html_local(c, node, "datalist")?
                        || (self.html_local(c, node, "optgroup")?
                            && self.ancestor_optgroup(c, node, root, detached)?)));
            next = if excluded {
                next_ordinary_view_after_subtree(&self.records, c, root, node, detached)?
            } else {
                self.next_view_node(c, root, node, detached)?
            };
        }
        Ok(slot)
    }
    fn retain_fragment_post_connection(
        &mut self,
        c: &mut ParseControl,
        roots: usize,
    ) -> Result<usize, SessionError> {
        let list = OwnedVec::new(HtmlParseResource::AuxiliaryEntries)?;
        if let Err(error) = self.prepare_payload_slot_owned(c) {
            list.release(c)?;
            return Err(error);
        }
        let slot = self.payloads.values.len();
        self.payloads
            .values
            .push(RetainedPayload::WorkList(Some(list)));
        c.charge(WorkCategory::RecordWrite, 1)?;
        let mut cursor = 0;
        while let Some(root) = self.work_list_node(c, roots, cursor)? {
            c.charge(WorkCategory::RecordWrite, 1)?;
            let mut next = Some(root);
            while let Some(node) = next {
                let RetainedPayload::WorkList(Some(list)) = &mut self.payloads.values[slot] else {
                    unreachable!()
                };
                list.try_push_recover(c, node)
                    .map_err(|(stop, _)| SessionError::Stop(stop))?;
                c.charge(WorkCategory::RecordWrite, 1)?;
                next = self.next_view_node(c, root, node, None)?;
            }
            c.charge(WorkCategory::RecordWrite, 1)?;
            cursor += 1;
        }
        Ok(slot)
    }
    fn ancestor_optgroup(
        &self,
        c: &mut ParseControl,
        node: usize,
        select: usize,
        detached: Option<usize>,
    ) -> Result<bool, SessionError> {
        let mut next = self.ordinary_parent(c, node, detached)?;
        for _ in 0..self.records.len() {
            let Some(ancestor) = next else {
                return Ok(false);
            };
            c.charge(WorkCategory::RecordVisit, 1)?;
            if ancestor == select {
                return Ok(false);
            }
            if self.html_local(c, ancestor, "optgroup")? {
                return Ok(true);
            }
            next = self.ordinary_parent(c, ancestor, detached)?;
        }
        Err(SessionError::Cycle)
    }
    fn post_connection_frame_with_cause(
        &mut self,
        c: &mut ParseControl,
        root: usize,
        cause: InsertionCause,
    ) -> Result<EffectFrame, SessionError> {
        let list = self.retain_work_list(c, root, WorkListKind::PostConnection)?;
        Ok(EffectFrame::PostConnection {
            list,
            cursor: 0,
            cause,
        })
    }
    fn work_list_node(
        &self,
        c: &mut ParseControl,
        slot: usize,
        cursor: usize,
    ) -> Result<Option<usize>, SessionError> {
        c.charge(WorkCategory::RecordVisit, 1)?;
        let Some(RetainedPayload::WorkList(Some(list))) = self.payloads.values.get(slot) else {
            return Err(SessionError::WrongToken);
        };
        list.require_disposal_owner(c)?;
        c.charge(WorkCategory::RecordVisit, 1)?;
        if let Some(node) = list.as_slice().get(cursor) {
            c.charge(WorkCategory::CopyByte, mem::size_of::<usize>() as u64)?;
            Ok(Some(*node))
        } else {
            Ok(None)
        }
    }
    fn release_work_list(&mut self, c: &mut ParseControl, slot: usize) -> Result<(), SessionError> {
        payload_write::<RetainedPayload>(c, 2)?;
        let RetainedPayload::WorkList(Some(list)) =
            mem::replace(&mut self.payloads.values[slot], RetainedPayload::Retired)
        else {
            return Err(SessionError::WrongToken);
        };
        list.release(c)?;
        Ok(())
    }
    fn first_selected_option(
        &mut self,
        c: &mut ParseControl,
        select: usize,
    ) -> Result<Option<usize>, SessionError> {
        let list = self.retain_work_list(c, select, WorkListKind::Options { detached: None })?;
        payload_write::<(usize, Option<usize>)>(c, 2)?;
        let mut cursor = 0;
        let mut selected = None;
        while let Some(node) = self.work_list_node(c, list, cursor)? {
            let (_, SelectMetadata::Option(state)) = self.select_metadata(c, node)? else {
                return Err(SessionError::SelectMetadataUnavailable);
            };
            if state.selectedness {
                payload_write::<Option<usize>>(c, 1)?;
                selected = Some(node);
                break;
            }
            payload_write::<usize>(c, 1)?;
            cursor += 1;
        }
        self.release_work_list(c, list)?;
        Ok(selected)
    }
    fn update_select(&mut self, c: &mut ParseControl, select: usize) -> Result<(), SessionError> {
        if self.form_attribute(c, select, "multiple")?.is_some() {
            return Ok(());
        }
        let list = self.retain_work_list(c, select, WorkListKind::SelectedTargets)?;
        self.start_effect(
            c,
            EffectFrame::UpdateSelect {
                select,
                list,
                cursor: 0,
            },
        )
    }
    pub(super) fn update_for_option(
        &mut self,
        c: &mut ParseControl,
        option: usize,
    ) -> Result<(), SessionError> {
        let (_, SelectMetadata::Option(state)) = self.select_metadata(c, option)? else {
            return Err(SessionError::SelectMetadataUnavailable);
        };
        if state.selectedness
            && let Some(select) = state.nearest_select
        {
            self.update_select(c, select)?;
        }
        Ok(())
    }
    pub(super) fn intrinsic_tree_hook(
        &mut self,
        c: &mut ParseControl,
        node: usize,
        detached: Option<usize>,
        removal: bool,
    ) -> Result<(), SessionError> {
        if self.html_local(c, node, "option")? {
            let (slot, SelectMetadata::Option(mut state)) = self.select_metadata(c, node)? else {
                return Err(SessionError::SelectMetadataUnavailable);
            };
            let old = state.nearest_select;
            let new = self.nearest_select(c, node, detached)?;
            state.nearest_select = new;
            self.set_select_metadata(c, slot, SelectMetadata::Option(state))?;
            if old != new {
                for select in [old, new].into_iter().flatten() {
                    let changed = self.selectedness_setting(c, select, detached)?;
                    let (_, SelectMetadata::Option(state)) = self.select_metadata(c, node)? else {
                        return Err(SessionError::SelectMetadataUnavailable);
                    };
                    if removal && (changed || state.selectedness) {
                        let cause = self
                            .evidence
                            .as_ref()
                            .map(|evidence| evidence.current_cause(c))
                            .transpose()?
                            .flatten();
                        self.effects
                            .enqueue(c, IntrinsicTask::UpdateSelect(select), cause)?;
                    }
                }
            }
        } else if !removal && self.html_local(c, node, "selectedcontent")? {
            self.recalculate_selectedcontent(c, node)?;
        }
        Ok(())
    }
    fn recalculate_selectedcontent(
        &mut self,
        c: &mut ParseControl,
        node: usize,
    ) -> Result<Option<usize>, SessionError> {
        let (slot, SelectMetadata::Selectedcontent(mut state)) = self.select_metadata(c, node)?
        else {
            return Err(SessionError::SelectMetadataUnavailable);
        };
        state.disabled = false;
        self.set_select_metadata(c, slot, SelectMetadata::Selectedcontent(state))?;
        let mut nearest = None;
        let mut next = self.ordinary_parent(c, node, None)?;
        for _ in 0..self.records.len() {
            let Some(ancestor) = next else {
                return Ok(nearest);
            };
            if self.html_local(c, ancestor, "select")? {
                if nearest.is_none() {
                    payload_write::<Option<usize>>(c, 1)?;
                    nearest = Some(ancestor);
                    next = self.ordinary_parent(c, ancestor, None)?;
                    continue;
                }
                state.disabled = true;
            } else if self.html_local(c, ancestor, "option")?
                || self.html_local(c, ancestor, "selectedcontent")?
            {
                state.disabled = true;
            }
            if state.disabled {
                self.set_select_metadata(c, slot, SelectMetadata::Selectedcontent(state))?;
                return Ok(nearest);
            }
            next = self.ordinary_parent(c, ancestor, None)?;
        }
        Err(SessionError::Cycle)
    }
    fn connect_selectedcontent(
        &mut self,
        c: &mut ParseControl,
        target: usize,
    ) -> Result<(), SessionError> {
        let select = self.recalculate_selectedcontent(c, target)?;
        let (_, SelectMetadata::Selectedcontent(state)) = self.select_metadata(c, target)? else {
            return Err(SessionError::SelectMetadataUnavailable);
        };
        if !state.disabled
            && let Some(select) = select
            && self.form_attribute(c, select, "multiple")?.is_none()
        {
            let option = self.first_selected_option(c, select)?;
            self.start_effect(
                c,
                EffectFrame::UpdateTarget {
                    option,
                    target,
                    fragment: None,
                    stage: TargetStage::Prepare,
                },
            )?;
        }
        Ok(())
    }
    pub(super) fn initial_select_reset(
        &mut self,
        c: &mut ParseControl,
        node: usize,
    ) -> Result<(), SessionError> {
        if !self.html_local(c, node, "select")? {
            return Ok(());
        }
        let (slot, SelectMetadata::Select(mut state)) = self.select_metadata(c, node)? else {
            return Err(SessionError::SelectMetadataUnavailable);
        };
        state.user_validity = false;
        self.set_select_metadata(c, slot, SelectMetadata::Select(state))?;
        self.selectedness_setting(c, node, None)?;
        // Fresh parser-created select has no children at syntax3537 reset.
        c.charge(WorkCategory::RecordVisit, 1)?;
        if !self.records[node].children.is_empty() {
            return Err(SessionError::WrongToken);
        }
        Ok(())
    }
    pub(super) fn initial_select_attribute(
        &mut self,
        c: &mut ParseControl,
        node: usize,
        position: usize,
    ) -> Result<(), SessionError> {
        c.charge(WorkCategory::RecordVisit, 3)?;
        let HtmlNodeKind::Element(element) = &self.records[node].kind else {
            return Err(SessionError::WrongToken);
        };
        let attribute = element
            .attributes()
            .as_slice()
            .get(position)
            .ok_or(SessionError::WrongToken)?;
        let selected = exact_ascii(c, Some(attribute.name().local_name()), "selected")?;
        let disabled = exact_ascii(c, Some(attribute.name().local_name()), "disabled")?;
        let size = exact_ascii(c, Some(attribute.name().local_name()), "size")?;
        let multiple = exact_ascii(c, Some(attribute.name().local_name()), "multiple")?;
        if self.html_local(c, node, "option")? {
            let (slot, SelectMetadata::Option(mut state)) = self.select_metadata(c, node)? else {
                return Err(SessionError::SelectMetadataUnavailable);
            };
            if selected && !state.dirtiness {
                state.selectedness = true;
                self.set_select_metadata(c, slot, SelectMetadata::Option(state))?;
            }
            if (selected || disabled)
                && let Some(select) = state.nearest_select
            {
                self.selectedness_setting(c, select, None)?;
            }
        } else if self.html_local(c, node, "select")? && (size || multiple) {
            self.selectedness_setting(c, node, None)?;
        } else if self.html_local(c, node, "optgroup")?
            && disabled
            && let Some(select) = self.nearest_select(c, node, None)?
        {
            self.selectedness_setting(c, select, None)?;
        }
        Ok(())
    }
    pub(super) fn select_metadata(
        &self,
        c: &mut ParseControl,
        node: usize,
    ) -> Result<(usize, SelectMetadata), SessionError> {
        retained_select_metadata(&self.payloads, c, node)
    }
    pub(super) fn set_select_metadata(
        &mut self,
        c: &mut ParseControl,
        slot: usize,
        value: SelectMetadata,
    ) -> Result<(), SessionError> {
        c.charge(WorkCategory::RecordVisit, 1)?;
        payload_write::<SelectMetadata>(c, 1)?;
        let Some(RetainedPayload::SelectFamily { state, .. }) = self.payloads.values.get_mut(slot)
        else {
            return Err(SessionError::SelectMetadataUnavailable);
        };
        *state = value;
        Ok(())
    }
    pub(super) fn nearest_select(
        &self,
        c: &mut ParseControl,
        node: usize,
        detached: Option<usize>,
    ) -> Result<Option<usize>, SessionError> {
        let mut next = self.ordinary_parent(c, node, detached)?;
        let mut group = false;
        for _ in 0..self.records.len() {
            let Some(node) = next else { return Ok(None) };
            for barrier in ["datalist", "hr", "option"] {
                if self.html_local(c, node, barrier)? {
                    return Ok(None);
                }
            }
            if self.html_local(c, node, "optgroup")? {
                if group {
                    return Ok(None);
                }
                payload_write::<bool>(c, 1)?;
                group = true;
            }
            if self.html_local(c, node, "select")? {
                return Ok(Some(node));
            }
            next = self.ordinary_parent(c, node, detached)?;
        }
        Err(SessionError::Cycle)
    }
    pub(super) fn option_disabled(
        &self,
        c: &mut ParseControl,
        node: usize,
        detached: Option<usize>,
    ) -> Result<bool, SessionError> {
        if self.form_attribute(c, node, "disabled")?.is_some() {
            return Ok(true);
        }
        let mut next = self.ordinary_parent(c, node, detached)?;
        for _ in 0..self.records.len() {
            let Some(node) = next else { return Ok(false) };
            for barrier in ["option", "select", "datalist", "hr"] {
                if self.html_local(c, node, barrier)? {
                    return Ok(false);
                }
            }
            if self.html_local(c, node, "optgroup")? {
                return Ok(self.form_attribute(c, node, "disabled")?.is_some());
            }
            next = self.ordinary_parent(c, node, detached)?;
        }
        Err(SessionError::Cycle)
    }
    pub(super) fn select_size(
        &self,
        c: &mut ParseControl,
        node: usize,
    ) -> Result<usize, SessionError> {
        let multiple = self.form_attribute(c, node, "multiple")?.is_some();
        let fallback = if multiple { 4 } else { 1 };
        let Some(value) = self.form_attribute(c, node, "size")? else {
            return Ok(fallback);
        };
        payload_write::<crate::numeric::IntegerPrefixScanner>(c, 1)?;
        payload_write::<usize>(c, 1)?;
        let mut scanner = crate::numeric::IntegerPrefixScanner::new(true);
        let mut position = 0;
        while position < value.len() {
            c.charge(WorkCategory::InspectByte, 2)?;
            payload_write::<crate::numeric::IntegerPrefixScanner>(c, 1)?;
            let unit = value.code_units()[position];
            if !scanner.feed(unit) {
                break;
            }
            payload_write::<usize>(c, 1)?;
            position += 1;
        }
        let Ok(prefix) = scanner.finish() else {
            return Ok(fallback);
        };
        payload_write::<(usize, usize)>(c, 2)?;
        let mut result = 0usize;
        let mut position = prefix.start;
        while position < prefix.end {
            c.charge(WorkCategory::InspectByte, 2)?;
            let unit = value.code_units()[position];
            if prefix.negative && unit != b'0' as u16 {
                return Ok(fallback);
            }
            payload_write::<usize>(c, 1)?;
            result = (result * 10 + usize::from(unit - b'0' as u16)).min(2);
            payload_write::<usize>(c, 1)?;
            position += 1;
        }
        Ok(result)
    }
    pub(super) fn selectedness_setting(
        &mut self,
        c: &mut ParseControl,
        select: usize,
        detached: Option<usize>,
    ) -> Result<bool, SessionError> {
        if self.form_attribute(c, select, "multiple")?.is_some() {
            return Ok(false);
        }
        let list = self.retain_work_list(c, select, WorkListKind::Options { detached })?;
        let mut cursor = 0;
        payload_write::<(Option<usize>, Option<usize>, bool, usize)>(c, 4)?;
        let mut first = None;
        let mut last = None;
        let mut changed = false;
        while let Some(node) = self.work_list_node(c, list, cursor)? {
            let (_, metadata) = self.select_metadata(c, node)?;
            let SelectMetadata::Option(state) = metadata else {
                return Err(SessionError::SelectMetadataUnavailable);
            };
            if state.selectedness {
                if let Some(previous) = last {
                    let (slot, SelectMetadata::Option(mut previous_state)) =
                        self.select_metadata(c, previous)?
                    else {
                        return Err(SessionError::SelectMetadataUnavailable);
                    };
                    previous_state.selectedness = false;
                    self.set_select_metadata(c, slot, SelectMetadata::Option(previous_state))?;
                    payload_write::<bool>(c, 1)?;
                    changed = true;
                }
                payload_write::<Option<usize>>(c, 1)?;
                last = Some(node);
            }
            if first.is_none() && !self.option_disabled(c, node, detached)? {
                payload_write::<Option<usize>>(c, 1)?;
                first = Some(node);
            }
            payload_write::<usize>(c, 1)?;
            cursor += 1;
        }
        if last.is_none()
            && let Some(first) = first
            && self.select_size(c, select)? == 1
        {
            let (slot, SelectMetadata::Option(mut state)) = self.select_metadata(c, first)? else {
                return Err(SessionError::SelectMetadataUnavailable);
            };
            state.selectedness = true;
            self.set_select_metadata(c, slot, SelectMetadata::Option(state))?;
            payload_write::<bool>(c, 1)?;
            changed = true;
        }
        self.release_work_list(c, list)?;
        Ok(changed)
    }
}

impl ParserArenaSession {
    pub(crate) fn perform_intrinsic_checkpoint(
        &mut self,
        c: &mut ParseControl,
        capability: &crate::parser::tree_builder::StandaloneCapability<'_>,
    ) -> Result<(), SessionError> {
        capability.validate_session(c, self)?;
        let storage = self.active_mut()?;
        c.charge(WorkCategory::RecordVisit, 1)?;
        if storage.effects.checkpoint {
            return Ok(());
        }
        storage.require(c)?;
        let return_cause = storage
            .evidence
            .as_ref()
            .map(|evidence| evidence.current_cause(c))
            .transpose()?
            .flatten();
        payload_write::<bool>(c, 1)?;
        storage.effects.checkpoint = true;
        let result = (|| {
            loop {
                c.require_identity(&storage.control)?;
                c.charge(WorkCategory::RecordVisit, 2)?;
                if storage.effects.dequeue == storage.effects.tasks.len() {
                    if let Some(cause) = &return_cause {
                        storage
                            .evidence
                            .as_mut()
                            .ok_or(SessionError::WrongToken)?
                            .activate_cause(c, cause)?;
                    }
                    payload_write::<bool>(c, 1)?;
                    storage.effects.checkpoint = false;
                    return Ok(());
                }
                let cursor = storage.effects.dequeue;
                payload_write::<Option<QueuedTask>>(c, 2)?;
                payload_write::<usize>(c, 1)?;
                c.charge(WorkCategory::RecordVisit, 1)?;
                let slot = storage
                    .effects
                    .tasks
                    .get_mut(cursor, c)?
                    .ok_or(SessionError::WrongToken)?;
                let task = slot.take().ok_or(SessionError::WrongToken)?;
                if let Some(cause) = &task.cause {
                    storage
                        .evidence
                        .as_mut()
                        .ok_or(SessionError::WrongToken)?
                        .activate_cause(c, cause)?;
                }
                let operation = task.task;
                storage.effects.task = Some(task);
                storage.effects.dequeue += 1;
                match operation {
                    IntrinsicTask::UpdateSelect(select) => storage.update_select(c, select)?,
                }
                storage.drain_construction_effects(c)?;
                payload_write::<Option<QueuedTask>>(c, 1)?;
                storage.effects.task = None;
            }
        })();
        match result {
            Err(SessionError::Stop(stop)) => Err(SessionError::Stop(stop)),
            Err(_) => Err(c
                .fail_invariant(crate::parser::control::Invariant::PendingFormEffects)
                .into()),
            Ok(()) => Ok(()),
        }
    }
    pub(crate) fn select_state(
        &self,
        c: &mut ParseControl,
        node: &ParserNode,
    ) -> Result<SelectState, SessionError> {
        let storage = self.active()?;
        let node = storage.index(c, node)?;
        match storage.select_metadata(c, node)?.1 {
            SelectMetadata::Select(state) => Ok(state),
            _ => Err(SessionError::SelectMetadataUnavailable),
        }
    }
    pub(crate) fn option_state(
        &self,
        c: &mut ParseControl,
        node: &ParserNode,
    ) -> Result<OptionState, SessionError> {
        let storage = self.active()?;
        let node = storage.index(c, node)?;
        match storage.select_metadata(c, node)?.1 {
            SelectMetadata::Option(state) => Ok(state),
            _ => Err(SessionError::SelectMetadataUnavailable),
        }
    }
    pub(crate) fn selectedcontent_state(
        &self,
        c: &mut ParseControl,
        node: &ParserNode,
    ) -> Result<SelectedcontentState, SessionError> {
        let storage = self.active()?;
        let node = storage.index(c, node)?;
        match storage.select_metadata(c, node)?.1 {
            SelectMetadata::Selectedcontent(state) => Ok(state),
            _ => Err(SessionError::SelectMetadataUnavailable),
        }
    }
    pub(crate) fn option_nearest_select(
        &self,
        c: &mut ParseControl,
        node: &ParserNode,
    ) -> Result<Option<ParserNode>, SessionError> {
        self.option_state(c, node)?
            .nearest_select
            .map(|index| self.active()?.node(c, index))
            .transpose()
    }
}

fn retained_select_metadata(
    payloads: &PayloadStorage,
    c: &mut ParseControl,
    node: usize,
) -> Result<(usize, SelectMetadata), SessionError> {
    for (slot, payload) in payloads.values.iter().enumerate() {
        c.charge(WorkCategory::RecordVisit, 1)?;
        if let RetainedPayload::SelectFamily {
            node: Some(owner),
            state,
        } = payload
            && *owner == node
        {
            c.charge(
                WorkCategory::CopyByte,
                mem::size_of::<SelectMetadata>() as u64,
            )?;
            return Ok((slot, *state));
        }
    }
    Err(SessionError::SelectMetadataUnavailable)
}

impl FrozenArena {
    fn retained_select_state(
        &self,
        c: &mut ParseControl,
        node: &HtmlNodeId,
    ) -> Result<SelectMetadata, SessionError> {
        let storage = self.storage.as_ref().ok_or(SessionError::Consumed)?;
        c.require_identity(&storage.control)?;
        let arena = match &storage.view {
            FrozenView::Document(view) => &view.arena,
            FrozenView::Fragment(view) => &view.arena,
        };
        c.charge(WorkCategory::ProviderCall, 1)?;
        c.charge(WorkCategory::RecordVisit, 1)?;
        if !Arc::ptr_eq(&node.identity, &arena.identity) || node.index >= arena.records.len() {
            return Err(SessionError::ForeignNode);
        }
        Ok(retained_select_metadata(&storage.payloads, c, node.index)?.1)
    }
    pub(crate) fn select_state(
        &self,
        c: &mut ParseControl,
        node: &HtmlNodeId,
    ) -> Result<SelectState, SessionError> {
        match self.retained_select_state(c, node)? {
            SelectMetadata::Select(state) => Ok(state),
            _ => Err(SessionError::SelectMetadataUnavailable),
        }
    }
    pub(crate) fn option_state(
        &self,
        c: &mut ParseControl,
        node: &HtmlNodeId,
    ) -> Result<OptionState, SessionError> {
        match self.retained_select_state(c, node)? {
            SelectMetadata::Option(state) => Ok(state),
            _ => Err(SessionError::SelectMetadataUnavailable),
        }
    }
    pub(crate) fn selectedcontent_state(
        &self,
        c: &mut ParseControl,
        node: &HtmlNodeId,
    ) -> Result<SelectedcontentState, SessionError> {
        match self.retained_select_state(c, node)? {
            SelectMetadata::Selectedcontent(state) => Ok(state),
            _ => Err(SessionError::SelectMetadataUnavailable),
        }
    }
    pub(crate) fn option_nearest_select(
        &self,
        c: &mut ParseControl,
        node: &HtmlNodeId,
    ) -> Result<Option<HtmlNodeId>, SessionError> {
        let owner = self.option_state(c, node)?.nearest_select;
        self.resolve_form_owner(c, owner)
    }
}

#[cfg(test)]
#[path = "select_tests.rs"]
mod tests;
