//! Shared parser-construction effects on the existing arena and controller.
//!
//! Nested frames execute depth first; intrinsic checkpoint tasks execute FIFO.
//! Work lists retain admitted snapshots in the same payload registry.

use super::select::SelectMetadata;
use super::*;

#[derive(Clone, Copy)]
pub(super) enum IntrinsicTask {
    UpdateSelect(usize),
}

/// Finite records reference actual admitted work-list slots in the registry.
/// No frame aliases native backing or owns an independent tree/controller.
#[derive(Clone)]
pub(super) enum EffectFrame {
    Details {
        node: usize,
        close: crate::HtmlDetailsExclusivity,
        root: usize,
        next: Option<usize>,
    },
    AdoptSubtree {
        root: usize,
        next: Option<usize>,
        document: DocumentRef,
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
        document: DocumentRef,
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
pub(super) enum WorkListKind {
    PostConnection,
    FragmentChildren,
    SelectedTargets,
    Options { detached: Option<usize> },
}

pub(super) struct ConstructionEffects {
    pub(super) form: Option<FormEffects>,
    pub(super) radio: Option<super::radio::RadioProgress>,
    pub(super) range: Option<super::range::RangeStaging>,
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
        payload_write::<Self>(c, 11)?;
        Ok(Self {
            form: None,
            radio: None,
            range: None,
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
        c.charge(WorkCategory::RecordVisit, 7)?;
        if self.range.is_some()
            || self.radio.is_some()
            || self.svg_wrapper.is_some()
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
        c.charge(WorkCategory::RecordVisit, 10)?;
        if self.range.is_some()
            || self.radio.is_some()
            || self.svg_wrapper.is_some()
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
        if let Some(range) = self.range {
            range.release(c)?;
        }
        self.frames.release(c)?;
        self.tasks.release(c)
    }
    pub(super) fn enqueue(
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

impl EffectFrame {
    fn admit_clone(&self, c: &mut ParseControl) -> Result<(), SessionError> {
        if let Self::AdoptSubtree { document, .. } | Self::CloneChildren { document, .. } = self {
            document.admit_clone(c)?;
        }
        Ok(())
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
        if let Some(parent) = self.effects.current.as_ref() {
            payload_write::<Option<EffectFrame>>(c, 1)?;
            parent.admit_clone(c)?;
            let parent = parent.clone();
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
    pub(super) fn write_current_effect(
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
        self.drain_construction_effects_with_meta_csp(c, &mut None)
    }
    pub(super) fn drain_construction_effects_with_meta_csp(
        &mut self,
        c: &mut ParseControl,
        service: &mut Option<&mut dyn crate::meta_csp::Service>,
    ) -> Result<(), SessionError> {
        let result = (|| {
            loop {
                c.require_identity(&self.control)?;
                c.charge(WorkCategory::RecordVisit, 2)?;
                if self.effects.form.is_some() {
                    self.step_form_effects_with_meta_csp(c, service)?;
                    continue;
                }
                if self.effects.current.is_some() {
                    c.admit_atomic_layout(mem::size_of::<EffectFrame>())?;
                    c.charge(WorkCategory::CopyByte, mem::size_of::<EffectFrame>() as u64)?;
                    // The wrapper below reports a missing frame as the pending
                    // effects invariant.
                    let frame = self
                        .effects
                        .current
                        .as_ref()
                        .ok_or(SessionError::WrongToken)?;
                    frame.admit_clone(c)?;
                    let frame = frame.clone();
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
            Err(
                error @ (SessionError::Stop(_)
                | SessionError::CspService
                | SessionError::CspProtocol(_)),
            ) => Err(error),
            Err(
                error @ (SessionError::ContextFactUnavailable(_)
                | SessionError::ContextReactionUnavailable),
            ) => {
                c.stop_unsupported_standalone(
                    crate::parser::control::StandaloneUnsupported::Provenance,
                );
                Err(error)
            }
            Err(error) => Err(super::latch_effect_fault(c, error)),
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
            EffectFrame::Details {
                node,
                close,
                root,
                next,
            } => self.step_details(c, node, close, root, next),
            EffectFrame::AdoptSubtree {
                root,
                next,
                document,
            } => {
                let Some(node) = next else {
                    return self.write_current_effect(c, None);
                };
                let binding = document.try_clone(c)?;
                if self.html_local(c, node, "style")? {
                    let next_document = document.try_clone(c)?;
                    self.record_association(
                        c,
                        node,
                        pending_association::OccurrenceKind::StyleDocumentChanged,
                        None,
                        Some(next_document),
                        false,
                    )?;
                }
                self.set_node_document(c, node, binding)?;
                self.adopt_registry(c, node, &document, root)?;
                self.start_img_update(
                    c,
                    node,
                    surgeist_html::HtmlImageUpdateFlags::new(false, false),
                )?;
                let next = self.next_view_node(c, root, node, None)?;
                let retained_document = document.try_clone(c)?;
                self.write_current_effect(
                    c,
                    Some(EffectFrame::AdoptSubtree {
                        root,
                        next,
                        document: retained_document,
                    }),
                )?;
                c.charge(WorkCategory::RecordVisit, 1)?;
                if let Some(shadow) = self
                    .construction
                    .node_at(node)
                    .ok_or_else(|| latch_registry_row(c))?
                    .shadow_root()
                {
                    c.charge(WorkCategory::ProviderCall, 1)?;
                    let contents = self
                        .construction
                        .storage(c)?
                        .node_by_result_id(shadow.contents())
                        .map_err(|_| SessionError::ForeignNode)?
                        .position();
                    self.shadow_state(c, contents)?;
                    let shadow_document = document.try_clone(c)?;
                    self.start_effect(
                        c,
                        EffectFrame::AdoptSubtree {
                            root: contents,
                            next: Some(contents),
                            document: shadow_document,
                        },
                    )?;
                }
                c.charge(WorkCategory::RecordVisit, 1)?;
                if self
                    .construction
                    .node_at(node)
                    .ok_or_else(|| latch_registry_row(c))?
                    .template_contents()
                    .is_some()
                {
                    let (_, template) = self.template_slot(c, node)?;
                    let target = self.appropriate_contents_document(c, &document)?;
                    if self.node_document_ref(c, template.contents)? != target {
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
                        cursor: next_cursor(cursor),
                    }),
                )?;
                c.charge(WorkCategory::RecordVisit, 1)?;
                if self
                    .construction
                    .node_at(node)
                    .ok_or_else(|| latch_registry_row(c))?
                    .parent()
                    .map(|node| node.position())
                    .is_some()
                {
                    self.remove_index(c, node)?;
                }
                Ok(())
            }
            EffectFrame::Topology(_) => Err(SessionError::WrongToken),
            frame @ (EffectFrame::UpdateSelect { .. }
            | EffectFrame::UpdateTarget { .. }
            | EffectFrame::CloneOptionChildren { .. }) => self.step_select_effect(c, frame),
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
                if cursor
                    == self
                        .construction
                        .node_at(source)
                        .ok_or_else(|| latch_registry_row(c))?
                        .child_count()
                {
                    self.write_current_effect(c, None)?;
                    if let Some((source, copy)) = self.clone_shadow(c, source, copy, &document)? {
                        self.start_effect(
                            c,
                            EffectFrame::CloneChildren {
                                source,
                                copy,
                                cursor: 0,
                                document,
                            },
                        )?;
                    }
                    return Ok(());
                }
                c.charge(WorkCategory::RecordVisit, 1)?;
                let child = self
                    .construction
                    .node_at(source)
                    .ok_or_else(|| latch_registry_row(c))?
                    .child_at(cursor)
                    .ok_or_else(|| latch_registry_row(c))?
                    .position();
                let child_document = document.try_clone(c)?;
                let copied = self.clone_node(c, child, child_document)?;
                let retained_document = document.try_clone(c)?;
                self.write_current_effect(
                    c,
                    Some(EffectFrame::CloneChildren {
                        source,
                        copy,
                        cursor: next_cursor(cursor),
                        document: retained_document,
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
                if self
                    .construction
                    .node_at(child)
                    .ok_or_else(|| latch_registry_row(c))?
                    .template_contents()
                    .is_some()
                {
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
                        cursor: next_cursor(cursor),
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
                                cursor: next_cursor(cursor),
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
                                cursor: next_cursor(cursor),
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

    pub(super) fn retain_work_list(
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
            let option_rule = if matches!(kind, WorkListKind::Options { .. }) {
                let mut recognized = None;
                for candidate in [
                    KnownHtmlElement::Option,
                    KnownHtmlElement::Select,
                    KnownHtmlElement::Hr,
                    KnownHtmlElement::Datalist,
                    KnownHtmlElement::Optgroup,
                ] {
                    if self.html_local(c, node, candidate.local_name())? {
                        recognized = Some(candidate);
                        break;
                    }
                }
                let inside_optgroup = recognized == Some(KnownHtmlElement::Optgroup)
                    && self.ancestor_optgroup(c, node, root, detached)?;
                Some(surgeist_html::bounded::select_option_rule(
                    recognized,
                    inside_optgroup,
                ))
            } else {
                None
            };
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
                WorkListKind::Options { .. } => {
                    option_rule
                        == Some(surgeist_html::bounded::HtmlSelectOptionRule::IncludeAndPrune)
                }
            };
            if include {
                let Some(RetainedPayload::WorkList(Some(list))) =
                    self.payloads.values.get_mut(slot)
                else {
                    return Err(latch_registry_row(c));
                };
                // An exact reserve per node would charge the whole list again
                // for every node it already holds.
                list.try_push_recover_geometric(c, node)
                    .map_err(|(stop, _)| SessionError::Stop(stop))?;
            }
            let excluded = matches!(kind, WorkListKind::FragmentChildren)
                || matches!(
                    option_rule,
                    Some(
                        surgeist_html::bounded::HtmlSelectOptionRule::IncludeAndPrune
                            | surgeist_html::bounded::HtmlSelectOptionRule::Prune
                    )
                );
            next = if excluded {
                next_ordinary_view_after_subtree(
                    ConstructionView::Active(self.construction.storage(c)?),
                    c,
                    root,
                    node,
                    detached,
                )?
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
                let Some(RetainedPayload::WorkList(Some(list))) =
                    self.payloads.values.get_mut(slot)
                else {
                    return Err(latch_registry_row(c));
                };
                list.try_push_recover(c, node)
                    .map_err(|(stop, _)| SessionError::Stop(stop))?;
                c.charge(WorkCategory::RecordWrite, 1)?;
                next = self.next_view_node(c, root, node, None)?;
            }
            c.charge(WorkCategory::RecordWrite, 1)?;
            cursor = next_cursor(cursor);
        }
        Ok(slot)
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
    pub(super) fn work_list_len(
        &self,
        c: &mut ParseControl,
        slot: usize,
    ) -> Result<usize, SessionError> {
        c.charge(WorkCategory::RecordVisit, 1)?;
        let Some(RetainedPayload::WorkList(Some(list))) = self.payloads.values.get(slot) else {
            return Err(SessionError::WrongToken);
        };
        list.require_disposal_owner(c)?;
        c.charge(WorkCategory::CopyByte, mem::size_of::<usize>() as u64)?;
        Ok(list.len())
    }
    pub(super) fn work_list_node(
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
    pub(super) fn release_work_list(
        &mut self,
        c: &mut ParseControl,
        slot: usize,
    ) -> Result<(), SessionError> {
        payload_write::<RetainedPayload>(c, 2)?;
        let row = self.payloads.row_mut(c, slot)?;
        // Only a work list that is still held is taken; any other row stays.
        let list = match row {
            RetainedPayload::WorkList(list) => list.take(),
            _ => None,
        };
        let Some(list) = list else {
            return Err(SessionError::WrongToken);
        };
        *row = RetainedPayload::Retired;
        list.release(c)?;
        Ok(())
    }
    pub(super) fn intrinsic_tree_hook(
        &mut self,
        c: &mut ParseControl,
        node: usize,
        detached: Option<usize>,
        removal: bool,
    ) -> Result<(), SessionError> {
        self.dialog_tree_hook(c, node, detached, removal)?;
        if base::is_base(
            self.construction
                .node_at(node)
                .ok_or_else(|| latch_registry_row(c))?,
        ) {
            self.base_change(c, node, detached, false)?;
        }
        if !removal && self.html_local(c, node, "input")? {
            self.radio_insertion(c, node)?;
        }
        if !removal && self.html_local(c, node, "details")? {
            self.start_details(c, node, crate::HtmlDetailsExclusivity::CloseGiven)?;
        } else {
            self.intrinsic_select_tree_hook(c, node, detached, removal)?;
        }
        Ok(())
    }
}

impl ParserArenaSession {
    pub(crate) fn perform_intrinsic_checkpoint(
        &mut self,
        c: &mut ParseControl,
        capability: &crate::parser::tree_builder::ConstructionCapability<'_>,
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
                // The branch above returned when `dequeue` reached the length.
                storage.effects.dequeue = next_cursor(storage.effects.dequeue);
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
            Err(error) => Err(super::latch_effect_fault(c, error)),
            Ok(()) => Ok(()),
        }
    }
}

#[cfg(test)]
#[path = "effects_tests.rs"]
pub(in crate::construction) mod tests;
