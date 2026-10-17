//! One canonical retained synchronous source operation. The lease grants access;
//! all mandatory continuation and acknowledged external publication stays in Dom.
mod dialog;
pub(crate) mod dialog_queue;
mod focus;
pub(in crate::store) mod hosted;
mod request_close;
mod scope;
#[cfg(test)]
mod scope_tests;
mod watcher;
use super::Invocation;
use super::*;
use crate::source::*;
use crate::style::*;
use scope::EntryScope;
use surgeist_cssom::{CssomCommit, CssomPublication, CssomSheetTicket, CssomSnapshot};
use surgeist_html::{HtmlStyleUpdateAction as Action, HtmlStyleUpdateReply as Reply};

// One public edit stays inline through admission. Named operations retain their
// own original controllers. Only owner-local engine tests supply command lists.
enum OperationRoot {
    Edit([Command; 1]),
    NativeInstallation,
    SelectCompletion(NodeId),
    DialogTask(crate::DomDialogTaskInfo),
    #[cfg(test)]
    Commands(Vec<Command>),
}
impl OperationRoot {
    fn capacity(&self) -> usize {
        match self {
            #[cfg(test)]
            Self::Commands(commands) => commands.capacity(),
            _ => 0,
        }
    }
}
impl std::ops::Deref for OperationRoot {
    type Target = [Command];
    fn deref(&self) -> &[Command] {
        match self {
            Self::Edit(commands) => commands,
            Self::NativeInstallation | Self::SelectCompletion(_) | Self::DialogTask(_) => &[],
            #[cfg(test)]
            Self::Commands(commands) => commands,
        }
    }
}
impl<'a> IntoIterator for &'a OperationRoot {
    type Item = &'a Command;
    type IntoIter = std::slice::Iter<'a, Command>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
pub(in crate::store) struct Operation {
    id: DomSourceOperationId,
    entry_scope: EntryScope,
    limits: DomSourceLimits,
    draft: dialog::OperationDraft,
    commands: OperationRoot,
    index: usize,
    started: bool,
    published: bool,
    event_exposed: bool,
    event_work: std::sync::Weak<crate::source::event::EventWork>,
    dialog_queue: Option<crate::DomElementTaskId>,
    request_close: Option<request_close::Call>,
    request_close_result: Option<DocumentString>,
    cancelled: bool,
    phases: u64,
    headroom: Option<headroom::Reservation>,
    command_admitted: bool,
    outputs: usize,
    identities: u64,
    context: Option<(u64, Arc<DomStyleSource>)>,
    request: Option<DomStyleRequest>,
    child: Option<hosted::Child>,
    hosted_global: Option<(Key, crate::DomHostedGlobalId, Arc<crate::hosted::Work>)>,
    receipt: Option<Receipt>,
    owner_binding: Option<DomStyleCssomOwnerBinding>,
    historical: Vec<Option<NodeId>>,
    select: Option<SelectRoot>,
    installation: Option<installation::NativeInstallation>,
    canonical_storage: storage::Payload,
    command_storage: storage::Payload,
    canonical_history_bytes: usize,
    custody_bytes: usize,
    canonical_aux_bytes: usize,
}
enum Receipt {
    CloseWatcher {
        publication: crate::DomCloseWatcherRequestPublication,
        applied: bool,
    },
    DialogFocus {
        publication: crate::DomDialogFocusPublication,
        applied: bool,
    },
    Watcher {
        request: DomSourceDialogRequest,
        watcher: crate::DomCloseWatcherId,
        applied: bool,
    },
    Inline {
        observation: crate::DomStyleInlineCspObservation,
        bytes: usize,
        units: usize,
    },
    Cssom {
        revision: surgeist_cssom::CssomRevision,
        sheet: Option<surgeist_cssom::CssomSheetId>,
    },
}
/// Execution permission for exactly one DOM-owned source operation. Drop parks
/// a published operation; it never runs cleanup or discards its mandatory tail.
pub struct DomSourceLease<'a> {
    dom: &'a mut Dom,
    id: DomSourceOperationId,
    complete: bool,
    select_authorization: Option<Arc<crate::select::Authorization>>,
    task_authorization: Option<Arc<crate::element_tasks::Binding>>,
}
impl Drop for DomSourceLease<'_> {
    fn drop(&mut self) {
        if !self.complete
            && self
                .dom
                .source_operation
                .as_ref()
                .is_some_and(|o| !o.published && !o.event_exposed && o.observation().is_none())
        {
            self.dom.source_operation = self.dom.source_parents.pop();
        }
    }
}
impl Dom {
    /// Whether this canonical owner retains any actual mutable Style CSSOM root.
    /// Historical snapshots and parsed pending associations do not count.
    pub fn has_style_cssom_roots(&self) -> bool {
        !self.style_cssom_roots.is_empty()
    }
    /// Reissues access to the actual already-retained mutable owner of a live
    /// Document or ShadowRoot. This does not create a binding. The bank scan is
    /// bounded by this DOM's finite node allowance; duplicating the private Arc
    /// handle allocates no payload. A historical snapshot cannot supply custody.
    pub fn style_cssom_root(&self, root: NodeId) -> Result<Option<DomStyleCssomRoot>, DomError> {
        if self.state.incomplete_source.is_some() {
            return Err(DomError::IncompleteSourceOperation);
        }
        let node = self.state.node(self.owner, root)?;
        if !matches!(
            node.kind,
            DomNodeKind::Document(_) | DomNodeKind::ShadowRoot(_)
        ) {
            return Err(DomError::WrongNodeKind);
        }
        Ok(self
            .style_cssom_roots
            .iter()
            .find(|entry| entry.root() == root)
            .map(DomStyleCssomRoot::duplicate))
    }
    pub(in crate::store) fn source_owner_custody_bytes(&self) -> Result<usize, DomError> {
        let style = self
            .style_cssom_roots
            .capacity()
            .checked_mul(size_of::<DomStyleCssomRoot>())
            .and_then(|n| n.checked_add(size_of::<Vec<DomStyleCssomRoot>>()))
            .and_then(|n| {
                n.checked_add(
                    self.style_cssom_roots
                        .len()
                        .checked_mul(DomStyleCssomRoot::retained_owner_bytes())?,
                )
            })
            .ok_or(DomError::RepresentationExhausted)?;
        style
            .checked_add(crate::element_tasks::retained(&self.element_tasks)?)
            .and_then(|bytes| {
                bytes.checked_add(
                    self.hosted_source
                        .as_ref()
                        .map_or(0, crate::hosted::SourceObservation::runtime_bytes),
                )
            })
            .ok_or(DomError::RepresentationExhausted)
    }
    pub(in crate::store) fn require_sync_source_idle(&self) -> Result<(), DomError> {
        if self.source_operation.is_some() || !self.source_parents.is_empty() {
            Err(DomError::SourceOperationPending)
        } else {
            Ok(())
        }
    }
    pub(in crate::store) fn require_source_idle(&self) -> Result<(), DomError> {
        self.require_sync_source_idle()?;
        if self.state.incomplete_hosted.is_some() {
            Err(DomError::SourceOperationPending)
        } else {
            Ok(())
        }
    }
    /// Reentry belongs only to original roots whose complete formation and
    /// publication join the retained Source envelopes. Legacy controllers keep
    /// the strict idle gate until their actual output lifetime is integrated.
    pub(in crate::store) fn require_source_entry(&self) -> Result<(), DomError> {
        match self.source_operation.as_ref() {
            Some(operation)
                if operation.permits_reentry() && self.state.incomplete_hosted.is_none() =>
            {
                Ok(())
            }
            Some(_) => Err(DomError::SourceOperationPending),
            None => self.require_source_idle(),
        }
    }
    pub fn pending_source_edit(&self) -> Option<DomSourceStatus> {
        self.source_operation.as_ref().map(|operation| {
            let mut status = operation.status();
            status.parent = self.source_parents.last().map(|parent| parent.id);
            status
        })
    }
    /// Begin exactly one requested edit through the original canonical engine.
    pub fn begin_source_edit(
        &mut self,
        edit: DomSourceEdit,
        limits: DomSourceLimits,
        cancel: &AtomicBool,
    ) -> Result<DomSourceLease<'_>, DomError> {
        self.begin_source_root(
            OperationRoot::Edit([edit.command]),
            Arc::new(BatchIdentity),
            limits,
            cancel,
        )
    }
    #[cfg(test)]
    pub(crate) fn begin_source_commands(
        &mut self,
        batch: DomBatch,
        limits: DomSourceLimits,
        cancel: &AtomicBool,
    ) -> Result<DomSourceLease<'_>, DomError> {
        self.begin_source_root(
            OperationRoot::Commands(batch.commands),
            batch.owner,
            limits,
            cancel,
        )
    }
    fn begin_source_root(
        &mut self,
        commands: OperationRoot,
        owner: Arc<BatchIdentity>,
        limits: DomSourceLimits,
        cancel: &AtomicBool,
    ) -> Result<DomSourceLease<'_>, DomError> {
        self.require_source_entry()?;
        check_cancel(cancel)?;
        self.next_source_operation
            .checked_add(1)
            .ok_or(DomError::RepresentationExhausted)?;
        let original_limits = limits;
        let limits = self.source_child_limits(limits, cancel)?;
        let nested = self
            .source_operation
            .as_ref()
            .is_some_and(|op| op.permits_reentry());
        if nested {
            self.push_source_observer(cancel)?;
        }
        let limits = if nested {
            match self.source_rebase_limits(original_limits, cancel) {
                Ok(limits) => limits,
                Err(error) => {
                    self.source_operation = self.source_parents.pop();
                    return Err(error);
                }
            }
        } else {
            limits
        };
        let formed =
            Draft::new_source_root(self, owner, &commands, commands.capacity(), limits, cancel)
                .and_then(|(draft, canonical, command)| {
                    let aux = self
                        .source_retained_stack_bytes()?
                        .checked_add(
                            self.source_dispatch_custody_bytes(|| draft.source_step(cancel))?,
                        )
                        .ok_or(DomError::RepresentationExhausted)?;
                    Ok((draft, canonical, command, aux))
                });
        let (draft, canonical_storage, command_storage, canonical_aux_bytes) = match formed {
            Ok(formed) => formed,
            Err(error) => {
                if nested {
                    self.source_operation = self.source_parents.pop();
                }
                return Err(error);
            }
        };
        let id = DomSourceOperationId {
            owner: self.owner,
            serial: self.next_source_operation,
        };
        let entry_scope = match EntryScope::capture(self, commands.entry_receiver(), &draft, cancel)
        {
            Ok(scope) => scope,
            Err(error) => {
                if nested {
                    self.source_operation = self.source_parents.pop();
                }
                return Err(error);
            }
        };
        let mut op = Box::new(Operation {
            id,
            entry_scope,
            limits: original_limits,
            draft: dialog::OperationDraft::Segment(draft),
            commands,
            index: 0,
            started: false,
            published: false,
            event_exposed: false,
            event_work: std::sync::Weak::new(),
            dialog_queue: None,
            request_close: None,
            request_close_result: None,
            cancelled: false,
            phases: 0,
            headroom: None,
            command_admitted: false,
            outputs: 0,
            identities: 0,
            context: None,
            request: None,
            child: None,
            hosted_global: None,
            receipt: None,
            owner_binding: None,
            historical: Vec::new(),
            select: None,
            installation: None,
            canonical_storage,
            canonical_history_bytes: self.history_bytes,
            command_storage,
            custody_bytes: 0,
            canonical_aux_bytes,
        });
        let admitted = (|| {
            op.custody_bytes = self.source_owner_custody_bytes()?;
            op.refresh_admission(cancel)
        })();
        if let Err(error) = admitted {
            if nested {
                self.source_operation = self.source_parents.pop();
            }
            return Err(error);
        }
        let next = self
            .next_source_operation
            .checked_add(1)
            .ok_or(DomError::RepresentationExhausted)?;
        self.source_operation = Some(op);
        self.next_source_operation = next;
        Ok(DomSourceLease {
            dom: self,
            id,
            complete: false,
            select_authorization: None,
            task_authorization: None,
        })
    }
    pub fn resume_source_edit(
        &mut self,
        id: DomSourceOperationId,
        limits: DomSourceLimits,
        cancel: &AtomicBool,
    ) -> Result<DomSourceLease<'_>, DomError> {
        if self
            .source_operation
            .as_ref()
            .is_some_and(|o| o.select.is_some())
        {
            return Err(DomError::InvalidNativeTask);
        }
        if self
            .source_operation
            .as_ref()
            .is_some_and(|operation| matches!(operation.commands, OperationRoot::DialogTask(_)))
        {
            return Err(DomError::InvalidSourceOperation);
        }
        self.resume_source_permission(id, limits, cancel)
    }
    fn resume_source_permission(
        &mut self,
        id: DomSourceOperationId,
        limits: DomSourceLimits,
        cancel: &AtomicBool,
    ) -> Result<DomSourceLease<'_>, DomError> {
        check_cancel(cancel)?;
        if id.owner != self.owner {
            return Err(DomError::ForeignOwner);
        }
        let observed = self
            .source_operation
            .as_ref()
            .filter(|operation| operation.id == id)
            .ok_or(DomError::InvalidSourceOperation)?
            .observed_event_work();
        if let Some(work) = observed {
            let mut policy = dialog::ResumeWorkPolicy::new(work, limits.work)?;
            self.admit_source_resume(id, limits, cancel)?;
            self.source_operation
                .as_mut()
                .expect("qualified original observation")
                .admit_observation_limits(limits, cancel)?;
            policy.commit();
            return Ok(DomSourceLease {
                dom: self,
                id,
                complete: false,
                select_authorization: None,
                task_authorization: None,
            });
        }
        self.admit_source_resume(id, limits, cancel)?;
        let effective = self.source_rebase_limits(limits, cancel)?;
        let aux_header = self
            .source_retained_stack_bytes()?
            .checked_add(size_of::<crate::store::dispatch::CustodyBank>())
            .ok_or(DomError::RepresentationExhausted)?;
        let custody_bytes = self.source_owner_custody_bytes()?;
        let dispatch = &self.dispatch_custody;
        let op = self
            .source_operation
            .as_mut()
            .filter(|o| o.id == id)
            .ok_or(DomError::InvalidSourceOperation)?;
        op.custody_bytes = custody_bytes;
        // Bounds are absolute; original spent work never resets on resume.
        if limits.work < op.draft.source_work.load(Ordering::Relaxed)
            || limits.phases
                < op.phases
                    .checked_add(
                        op.headroom
                            .as_ref()
                            .map_or(0, |headroom| headroom.remaining_phases),
                    )
                    .ok_or(DomError::RepresentationExhausted)?
            || limits.outputs < op.outputs
        {
            return Err(DomError::InvalidLimit(DomResource::SourceRetainedBytes));
        }
        if op.published {
            self.revision
                .0
                .checked_add(
                    op.headroom
                        .as_ref()
                        .map_or(1, |headroom| headroom.remaining_phases),
                )
                .ok_or(DomError::RevisionExhausted)?;
        }
        let previous = op.draft.source_limits.replace(effective);
        let aux = dispatch
            .retained_bytes(|| op.draft.source_step(cancel))
            .and_then(|bytes| {
                aux_header
                    .checked_add(bytes)
                    .ok_or(DomError::RepresentationExhausted)
            });
        match aux {
            Ok(bytes) => op.canonical_aux_bytes = bytes,
            Err(error) => {
                op.draft.source_limits = previous;
                return Err(error);
            }
        }
        if let Err(e) = op.refresh_admission(cancel) {
            op.draft.source_limits = previous;
            return Err(e);
        }
        op.limits = limits;
        Ok(DomSourceLease {
            dom: self,
            id,
            complete: false,
            select_authorization: None,
            task_authorization: None,
        })
    }
}
#[derive(Clone, Copy)]
enum PublicationTarget {
    None,
    Style(Frame),
    Dialog(crate::store::dialog::PendingDialog),
}
struct PublishedSegment {
    commit: DomCommit,
    dialog: Option<crate::DomSourceDialogRequest>,
}
impl Operation {
    #[cfg(test)]
    pub(in crate::store) fn semantic_usage_for_test(&self) -> (usize, usize) {
        (
            self.draft.select_semantic_work.get(),
            self.draft.select_semantic_bytes.get(),
        )
    }
    pub(in crate::store) fn admit_dom_limits(&self, limits: DomLimits) -> Result<(), DomError> {
        if let Some(observation) = self.observation() {
            return observation.usage.admit_limits(limits);
        }
        let draft = &self.draft;
        admit(draft.nodes, limits.nodes, DomResource::ManagedNodes)?;
        admit(draft.units, limits.units, DomResource::StringCodeUnits)?;
        draft
            .input_processing
            .admit_limits(limits.input_scratch, limits.input_work)?;
        if draft.base_work.get() > limits.base_work {
            return Err(DomError::BaseWorkLimit {
                limit: limits.base_work,
            });
        }
        if draft.encoding_url_work.get() > limits.encoding_url_work {
            return Err(DomError::EncodingUrlWorkLimit {
                limit: limits.encoding_url_work,
            });
        }
        if draft.meta_work.get() > limits.meta_work {
            return Err(DomError::Meta(crate::DomMetaError::WorkLimit {
                limit: limits.meta_work,
            }));
        }
        // A native-checkpoint root has its own explicit Select context. It
        // must not be replaced by the ordinary batch's independent defaults.
        if self.select.is_none() {
            if draft.select_work.get() > limits.select_work {
                return Err(DomError::SelectWorkLimit {
                    limit: limits.select_work,
                });
            }
            if draft.select_work.get() != 0 {
                let live = draft
                    .effect_bytes
                    .checked_add(draft.select_temporary_bytes.load(Ordering::Relaxed))
                    .ok_or(DomError::RepresentationExhausted)?;
                admit(live, limits.select_scratch, DomResource::SelectScratch)?;
            }
        }
        if let Some((_, semantics)) = limits.select_value_queries {
            for (resource, limit, used) in [
                (
                    surgeist_html_semantics::HtmlSemanticResource::Work,
                    semantics.work(),
                    draft.select_semantic_work.get(),
                ),
                (
                    surgeist_html_semantics::HtmlSemanticResource::AllocatedBytes,
                    semantics.allocated_bytes(),
                    draft.select_semantic_bytes.get(),
                ),
            ] {
                if used > limit {
                    return Err(DomError::InvalidSelectSemanticLimit {
                        resource,
                        limit,
                        used,
                    });
                }
            }
        }
        let mut base = 0usize;
        let mut encoding = 0usize;
        for node in draft.existing.iter().chain(&draft.created).flatten() {
            base = base
                .checked_add(
                    node.base_document
                        .as_ref()
                        .map_or(0, crate::base::BaseDocument::records),
                )
                .ok_or(DomError::RepresentationExhausted)?;
            encoding = encoding
                .checked_add(
                    node.encoding_url_document
                        .as_ref()
                        .map_or(0, crate::hyperlink::EncodingUrlDocument::records),
                )
                .ok_or(DomError::RepresentationExhausted)?;
        }
        admit(
            base,
            limits.base_observations,
            DomResource::BaseObservations,
        )?;
        admit(
            encoding,
            limits.encoding_url_observations,
            DomResource::EncodingUrlObservations,
        )?;
        admit(
            draft
                .native_tasks
                .iter()
                .filter(|t| matches!(t, select::Pending::Update { .. }))
                .count(),
            limits.select_tasks,
            DomResource::SelectTasks,
        )?;
        admit(
            draft.details_intentions.len(),
            limits.details_intentions,
            DomResource::DetailsToggleIntentions,
        )?;
        let (transitions, records, pseudos) = draft.view_transitions.usage()?;
        admit(
            transitions,
            limits.transitions,
            DomResource::ViewTransitions,
        )?;
        admit(records, limits.records, DomResource::ViewTransitionRecords)?;
        admit(pseudos, limits.pseudos, DomResource::ViewTransitionPseudos)
    }
    pub(in crate::store) fn update_dom_limits(&mut self, limits: DomLimits) {
        if self.observation().is_some() {
            return;
        }
        let draft = &mut self.draft;
        draft
            .input_processing
            .update_limits(limits.input_scratch, limits.input_work);
        draft.base_work_limit = limits.base_work;
        draft.encoding_url_work_limit = limits.encoding_url_work;
        draft.meta_work_limit = limits.meta_work;
        if self.select.is_none() {
            draft.select_work_limit = limits.select_work;
            draft.select_scratch_limit = limits.select_scratch;
        }
    }
    pub(in crate::store) fn status(&self) -> DomSourceStatus {
        if let Some(observation) = self.observation() {
            return observation.status(self);
        }
        DomSourceStatus {
            operation: self.id,
            published: self.published,
            cancelled: self.cancelled,
            cleanup: self.request_close.and_then(|call| call.failure),
            work: self.draft.source_work.load(Ordering::Relaxed),
            phases: self.phases,
            parent: None,
            phase: if self
                .request_close
                .is_some_and(|call| call.failure.is_some())
            {
                crate::DomSourcePhase::PendingCleanup
            } else if self.draft.dialog_method.is_some()
                || self.draft.pending_dialog_hook().is_some()
            {
                crate::DomSourcePhase::DialogService
            } else {
                crate::DomSourcePhase::Staging
            },
        }
    }
    fn input_bytes(&self) -> Result<usize, DomError> {
        let event_payload = self.event_payload()?;
        let mut bytes = self
            .canonical_storage
            .bytes
            .checked_add(self.command_storage.bytes)
            .and_then(|n| n.checked_add(self.custody_bytes))
            .and_then(|n| n.checked_add(self.canonical_aux_bytes))
            .and_then(|n| n.checked_add(event_payload.bytes))
            .and_then(|n| n.checked_add(self.canonical_history_bytes))
            // Draft owns its inline header in source_storage_bytes already.
            .and_then(|n| n.checked_add(size_of::<Self>().checked_sub(size_of::<Draft>())?))
            .ok_or(DomError::RepresentationExhausted)?
            .checked_add(
                self.commands
                    .capacity()
                    .checked_mul(size_of::<Command>())
                    .ok_or(DomError::RepresentationExhausted)?,
            )
            .and_then(|n| n.checked_add(self.historical.capacity() * size_of::<Option<NodeId>>()))
            .ok_or(DomError::RepresentationExhausted)?;
        if let Some(installation) = &self.installation
            && installation.rows_payload.is_none()
        {
            bytes = bytes
                .checked_add(
                    installation
                        .rows
                        .capacity()
                        .checked_mul(size_of::<installation::Row>())
                        .ok_or(DomError::RepresentationExhausted)?,
                )
                .ok_or(DomError::RepresentationExhausted)?;
        }
        for c in &self.commands {
            if let Command::AssignSlot { incoming, .. } = c {
                bytes = bytes
                    .checked_add(
                        incoming
                            .capacity()
                            .checked_mul(size_of::<NodeOperand>())
                            .ok_or(DomError::RepresentationExhausted)?,
                    )
                    .ok_or(DomError::RepresentationExhausted)?;
            }
        }
        if let Some(binding) = &self.owner_binding {
            bytes = bytes
                .checked_add(binding.retained_bytes()?)
                .ok_or(DomError::RepresentationExhausted)?;
        }
        if let Some(Receipt::Inline {
            bytes: retained, ..
        }) = &self.receipt
        {
            bytes = bytes
                .checked_add(*retained)
                .ok_or(DomError::RepresentationExhausted)?;
        }
        if let Some((_, s)) = &self.context {
            bytes = bytes
                .checked_add(s.retained_bytes()?)
                .ok_or(DomError::RepresentationExhausted)?;
        }
        if let Some(order) = self.request.as_ref().and_then(|r| r.sheet_order.as_ref()) {
            bytes = bytes
                .checked_add(size_of::<Vec<DomStyleSheetPlacement>>())
                .and_then(|n| {
                    n.checked_add(
                        order
                            .capacity()
                            .checked_mul(size_of::<DomStyleSheetPlacement>())?,
                    )
                })
                .ok_or(DomError::RepresentationExhausted)?;
        }
        if let Some(value) = &self.request_close_result {
            let mut payload = storage::Payload::default();
            payload.string(value)?;
            bytes = bytes
                .checked_add(payload.bytes)
                .ok_or(DomError::RepresentationExhausted)?;
        }
        Ok(bytes)
    }
    fn refresh_admission(&mut self, cancel: &AtomicBool) -> Result<(), DomError> {
        if self.observation().is_some() {
            return Ok(());
        }
        self.draft.source_other_bytes = self.input_bytes()?;
        let mut units = self
            .event_payload()?
            .units
            .checked_add(
                self.canonical_storage
                    .units
                    .checked_add(self.command_storage.units)
                    .ok_or(DomError::RepresentationExhausted)?,
            )
            .ok_or(DomError::RepresentationExhausted)?;
        if let Some((_, source)) = &self.context {
            units = units
                .checked_add(source.retained_units()?)
                .ok_or(DomError::RepresentationExhausted)?;
        }
        if let Some(Receipt::Inline { units: policy, .. }) = &self.receipt {
            units = units
                .checked_add(*policy)
                .ok_or(DomError::RepresentationExhausted)?;
        }
        if let Some(value) = &self.request_close_result {
            let mut payload = storage::Payload::default();
            payload.string(value)?;
            units = units
                .checked_add(payload.units)
                .ok_or(DomError::RepresentationExhausted)?;
        }
        self.draft.source_other_units = units;
        self.draft.source_admit(0, cancel)
    }
    fn admit_current_command(&mut self, dom: &Dom, cancel: &AtomicBool) -> Result<(), DomError> {
        let entry = if let Some(select) = &self.select {
            headroom::Entry::Select {
                target: self.draft.resolve(select.target.into())?,
            }
        } else if let Some(installation) = &self.installation {
            headroom::Entry::Native {
                wrapper: installation.wrapper,
                parent: installation.parent,
                before: installation.before,
            }
        } else {
            headroom::Entry::Command(
                self.commands
                    .get(self.index)
                    .ok_or(DomError::StyleProtocol)?,
            )
        };
        let reservation =
            self.draft
                .admit_source_command_headroom(dom, entry, self.phases, cancel)?;
        self.headroom = Some(reservation);
        self.command_admitted = true;
        Ok(())
    }
    fn apply_command(&mut self, limits: DomLimits, cancel: &AtomicBool) -> Result<(), DomError> {
        self.draft.source_step(cancel)?;
        let command = &self.commands[self.index];
        self.draft.apply(command, limits, cancel)
    }
    fn frame(&self) -> Result<Frame, DomError> {
        match self.draft.effects.last().map(|f| &f.effect) {
            Some(select::Effect::Style(frame)) => Ok(*frame),
            _ => Err(DomError::StyleProtocol),
        }
    }
    fn source(
        &self,
        state: &State,
        allocated: &[Option<NodeId>],
        stamp: DomStamp,
        frame: Frame,
        budget: Option<&formation::Formation>,
        cancel: &AtomicBool,
    ) -> Result<Arc<DomStyleSource>, DomError> {
        if let Some((invocation, source)) = &self.context
            && matches!(frame.invocation, Invocation::Exposed(actual) if *invocation == actual)
        {
            return Ok(source.clone());
        }
        let id = match frame.key {
            Key::Existing(slot) => NodeId {
                owner: self.id.owner,
                slot,
                generation: self.draft.generations[slot],
            },
            Key::Created(i) => allocated[i].ok_or(DomError::UnavailableCreation)?,
        };
        let n = state.node(self.id.owner, id)?;
        let mut length = 0usize;
        for child in &n.children {
            self.draft.source_step(cancel)?;
            if let DomNodeKind::Text(s) = &state.node(self.id.owner, *child)?.kind {
                length = length
                    .checked_add(s.len())
                    .ok_or(DomError::RepresentationExhausted)?;
            }
        }
        let mut root = id;
        while let Some(parent) = state.node(self.id.owner, root)?.parent {
            self.draft.source_step(cancel)?;
            root = parent;
        }
        let root_kind = match &state.node(self.id.owner, root)?.kind {
            DomNodeKind::Document(_) => DomStyleRootKind::Document,
            DomNodeKind::ShadowRoot(_) => DomStyleRootKind::ShadowRoot,
            _ => DomStyleRootKind::Detached,
        };
        let DomNodeKind::Document(metadata) = &state.node(self.id.owner, n.document.node())?.kind
        else {
            return Err(DomError::WrongDocument);
        };
        let DomNodeKind::Element(element) = &n.kind else {
            return Err(DomError::WrongNodeKind);
        };
        let base_url = self
            .draft
            .style_document_base(self.draft.node(frame.key)?.document, cancel)?;
        let media = self.draft.control_attribute(
            frame.key,
            surgeist_html::KnownHtmlAttribute::Media,
            cancel,
        )?;
        let title = self.draft.control_attribute(
            frame.key,
            surgeist_html::KnownHtmlAttribute::Title,
            cancel,
        )?;
        let blocking = self.draft.control_attribute(
            frame.key,
            surgeist_html::KnownHtmlAttribute::Blocking,
            cancel,
        )?;
        let nonce = element.nonce();
        let mut bytes = size_of::<DomStyleSource>()
            .checked_add(size_of::<Vec<u16>>())
            .ok_or(DomError::RepresentationExhausted)?;
        let mut units = metadata
            .retained_payload_units()?
            .checked_add(length)
            .ok_or(DomError::RepresentationExhausted)?;
        for string in [Some(base_url), media, title, blocking, nonce]
            .into_iter()
            .flatten()
        {
            self.draft.source_step(cancel)?;
            units = units
                .checked_add(string.shared_code_units().capacity())
                .ok_or(DomError::RepresentationExhausted)?;
            bytes = bytes
                .checked_add(size_of::<Vec<u16>>())
                .and_then(|n| {
                    n.checked_add(
                        string
                            .shared_code_units()
                            .capacity()
                            .checked_mul(size_of::<u16>())?,
                    )
                })
                .ok_or(DomError::RepresentationExhausted)?;
        }
        // The default nonce has a real empty Vec backing too. Form it only
        // after that header has been admitted along with the complete capture.
        if nonce.is_none() {
            bytes = bytes
                .checked_add(size_of::<Vec<u16>>())
                .ok_or(DomError::RepresentationExhausted)?;
        }
        bytes = bytes
            .checked_add(metadata.retained_payload_bytes()?)
            .ok_or(DomError::RepresentationExhausted)?;
        let budget = budget.expect("source publication has formation admission");
        budget.retain_payload(storage::Payload { bytes, units }, cancel)?;
        let mut text = Vec::new();
        formation::reserve(&mut text, length, Some(budget), cancel)?;
        budget.retain_payload(
            storage::Payload {
                bytes: 0,
                units: text
                    .capacity()
                    .checked_sub(length)
                    .ok_or(DomError::RepresentationExhausted)?,
            },
            cancel,
        )?;
        for child in &n.children {
            self.draft.source_step(cancel)?;
            if let DomNodeKind::Text(s) = &state.node(self.id.owner, *child)?.kind {
                for unit in s.code_units() {
                    self.draft.source_step(cancel)?;
                    text.push(*unit);
                }
            }
        }
        let source = Arc::new(DomStyleSource {
            operation: self.id,
            ordinal: match frame.invocation {
                Invocation::Unissued => self.draft.source_invocation,
                Invocation::Exposed(ordinal) => ordinal,
            },
            stamp,
            metadata: metadata.clone(),
            root_kind,
            element: id,
            document: n.document,
            root,
            text: Arc::new(text),
            base_url: base_url.clone(),
            media: media.cloned(),
            title: title.cloned(),
            blocking: blocking.cloned(),
            nonce: nonce.cloned().unwrap_or_default(),
        });
        Ok(source)
    }
    fn visit_sheet_order(
        &self,
        state: &State,
        source: &DomStyleSource,
        cancel: &AtomicBool,
        mut visit: impl FnMut(DomStyleSheetPlacement) -> Result<(), DomError>,
    ) -> Result<(), DomError> {
        let mut current = source.root;
        loop {
            self.draft.source_step(cancel)?;
            let node = state.node(self.id.owner, current)?;
            if current == source.element {
                visit(DomStyleSheetPlacement::ThisStyle)?;
            } else if let DomNodeKind::Element(element) = &node.kind
                && let Some(style) = element.style_state()
            {
                if style.availability == DomStyleSheetAvailability::PendingParsedProcessing {
                    return Err(DomError::StyleProcessingUnavailable);
                }
                if let Some(sheet) = &style.sheet {
                    if sheet.root != source.root {
                        return Err(DomError::StyleReceiptMismatch);
                    }
                    visit(DomStyleSheetPlacement::Existing(sheet.id.clone()))?;
                }
            }
            if let Some(child) = node.children.first() {
                current = *child;
                continue;
            }
            loop {
                if current == source.root {
                    return Ok(());
                }
                let parent = state
                    .node(self.id.owner, current)?
                    .parent
                    .ok_or(DomError::WrongNodeKind)?;
                let children = &state.node(self.id.owner, parent)?.children;
                let mut next = None;
                let mut found = false;
                for sibling in children {
                    self.draft.source_step(cancel)?;
                    if found {
                        next = Some(*sibling);
                        break;
                    }
                    found = *sibling == current;
                }
                if !found {
                    return Err(DomError::WrongNodeKind);
                }
                if let Some(next) = next {
                    current = next;
                    break;
                }
                current = parent;
            }
        }
    }
    fn sheet_order(
        &self,
        state: &State,
        source: &DomStyleSource,
        formation: Option<&formation::Formation>,
        cancel: &AtomicBool,
    ) -> Result<Arc<Vec<DomStyleSheetPlacement>>, DomError> {
        let mut count = 0usize;
        self.visit_sheet_order(state, source, cancel, |_| {
            count = count
                .checked_add(1)
                .ok_or(DomError::RepresentationExhausted)?;
            Ok(())
        })?;
        let budget = formation.ok_or(DomError::StyleProtocol)?;
        budget.retain_bytes(size_of::<Vec<DomStyleSheetPlacement>>(), cancel)?;
        let mut order = Vec::new();
        formation::reserve(&mut order, count, Some(budget), cancel)?;
        self.visit_sheet_order(state, source, cancel, |entry| {
            order.push(entry);
            Ok(())
        })?;
        Ok(Arc::new(order))
    }
    fn history_after(
        &self,
        dom: &Dom,
        changed: bool,
        changes: usize,
        cancel: &AtomicBool,
    ) -> Result<usize, DomError> {
        if !changed {
            return Ok(self.canonical_history_bytes);
        }
        let mut bytes = changes
            .checked_mul(size_of::<DomChange>())
            .and_then(|n| n.checked_add(size_of::<HistoryEntry>()))
            .ok_or(DomError::RepresentationExhausted)?;
        if dom.limits.history_entries == 0 || bytes > dom.limits.history_bytes {
            return Ok(0);
        }
        let mut entries = 1usize;
        let mut next = dom.history.as_deref();
        while let Some(entry) = next {
            self.draft.source_step(cancel)?;
            if entries == dom.limits.history_entries
                || entry.bytes > dom.limits.history_bytes - bytes
            {
                break;
            }
            bytes = bytes
                .checked_add(entry.bytes)
                .ok_or(DomError::RepresentationExhausted)?;
            entries += 1;
            next = entry.older.as_deref();
        }
        Ok(bytes)
    }
    fn publication(
        &mut self,
        dom: &mut Dom,
        incomplete: bool,
        target: PublicationTarget,
        cancel: &AtomicBool,
    ) -> Result<PublishedSegment, DomError> {
        self.publication_with_hosted(dom, incomplete, target, None, cancel)
            .map_err(|error| match error {
                crate::hosted::DomHostedError::Dom(error) => error,
                _ => {
                    unreachable!("a segment without a hosted permit produces only canonical errors")
                }
            })
    }
    fn publication_with_hosted(
        &mut self,
        dom: &mut Dom,
        incomplete: bool,
        target: PublicationTarget,
        hosted: Option<(
            &crate::DomHostedSource,
            &crate::hosted::Request,
            crate::DomHostedSourceRevision,
        )>,
        cancel: &AtomicBool,
    ) -> Result<PublishedSegment, crate::hosted::DomHostedError> {
        let frame = match target {
            PublicationTarget::Style(frame) => Some(frame),
            _ => None,
        };
        // A proposed ordinal stays private through every fallible formation.
        // Only successful canonical publication/request exposure issues it.
        let invocation = frame.map(|frame| frame.invocation).or(match target {
            PublicationTarget::Dialog(pending) => pending.invocation,
            _ => None,
        });
        let issued_next = match invocation {
            Some(Invocation::Unissued) => Some(
                self.draft
                    .source_invocation
                    .checked_add(1)
                    .ok_or(DomError::RepresentationExhausted)?,
            ),
            Some(Invocation::Exposed(_)) | None => None,
        };
        self.refresh_admission(cancel)?;
        self.draft.prepare_transition_phase(cancel)?;
        let mut prepared = self.draft.finish(dom, cancel)?;
        let incomplete_owner = if incomplete {
            Some(self.id)
        } else {
            let mut owner = None;
            for parent in dom.source_parents.iter().rev() {
                self.draft.source_step(cancel)?;
                if parent.published {
                    owner = Some(parent.id);
                    break;
                }
            }
            owner
        };
        prepared.state.incomplete_source = incomplete_owner;
        if prepared.state.incomplete_source != dom.state.incomplete_source {
            formation::reserve(
                &mut prepared.changes,
                1,
                prepared.formation.as_ref(),
                cancel,
            )?;
            prepared.changes.push(DomChange::SourceOperation);
        }
        let limits = self.draft.source_limits.expect("source limits");
        // Closing revision space is held before the first external phase, so a
        // later service cannot knowingly strand an already published command.
        if let Some((key, global, work)) = &self.hosted_global {
            let source = hosted
                .map(|gate| gate.0)
                .ok_or(DomError::HostedSourceUnavailable)?;
            let document = DocumentId(match key {
                Key::Existing(slot) => NodeId {
                    owner: dom.owner,
                    slot: *slot,
                    generation: self.draft.generations[*slot],
                },
                Key::Created(index) => {
                    prepared.allocation_ids[*index].ok_or(DomError::UnavailableCreation)?
                }
            });
            prepared.state.hosted_globals =
                Some(crate::store::hosted::GlobalDocumentIndex::establish(
                    dom.state.hosted_globals.as_deref(),
                    *global,
                    document,
                    source,
                    work,
                    prepared.formation.as_ref(),
                    cancel,
                )?);
        }
        let units = prepared.state.units;
        prepared.state.units = dom.state.units;
        let changed = prepared.state != *dom.state;
        prepared.state.units = units;
        let remaining = if incomplete {
            self.headroom
                .as_ref()
                .ok_or(DomError::StyleProtocol)?
                .remaining_phases
        } else {
            u64::from(changed)
        };
        if incomplete && remaining == 0 {
            return Err(DomError::StyleProtocol.into());
        }
        let required_phases = self
            .phases
            .checked_add(remaining)
            .ok_or(DomError::RepresentationExhausted)?;
        if required_phases > limits.phases {
            return Err(DomError::SourcePublicationLimit.into());
        }
        if changed {
            dom.revision
                .0
                .checked_add(remaining)
                .and_then(|next| {
                    dom.source_reserved_closing()
                        .ok()
                        .and_then(|closing| next.checked_add(closing))
                })
                .ok_or(DomError::RevisionExhausted)?;
        }
        let outputs = self
            .outputs
            .checked_add(prepared.changes.len())
            .ok_or(DomError::RepresentationExhausted)?;
        if outputs
            .checked_add(usize::from(incomplete))
            .is_none_or(|required| required > limits.outputs)
        {
            return Err(DomError::SourceOutputLimit.into());
        }
        let ids = std::mem::take(&mut prepared.allocation_ids);
        let count = ids
            .iter()
            .enumerate()
            .filter(|(i, id)| {
                id.is_some() && self.draft.phase_map.as_ref().expect("source map").ids[*i].is_none()
            })
            .count();
        let mut identities = self
            .identities
            .checked_add(u64::try_from(count).map_err(|_| DomError::RepresentationExhausted)?)
            .ok_or(DomError::RepresentationExhausted)?;
        if identities > limits.identities {
            return Err(DomError::SourceIdentityLimit.into());
        }
        let mut historical = Vec::new();
        formation::reserve(
            &mut historical,
            self.draft.tickets.len(),
            prepared.formation.as_ref(),
            cancel,
        )?;
        for (i, key) in self.draft.tickets.iter().enumerate() {
            self.draft.source_step(cancel)?;
            let Key::Created(k) = key else {
                unreachable!("creation ticket")
            };
            historical.push(self.historical.get(i).copied().flatten().or(ids[*k]));
        }
        let dialog = if let PublicationTarget::Dialog(pending) = target {
            let subject = match pending.subject {
                Key::Existing(slot) => NodeId {
                    owner: self.id.owner,
                    slot,
                    generation: self.draft.generations[slot],
                },
                Key::Created(index) => ids[index].ok_or(DomError::UnavailableCreation)?,
            };
            prepared.state.node(self.id.owner, subject)?;
            Some(crate::DomSourceDialogRequest {
                operation: self.id,
                subject,
                action: pending.action,
                occurrence: pending.invocation.map(|invocation| {
                    let occurrence = crate::source::watcher::HookOccurrence {
                        command: self.index,
                        ordinal: match invocation {
                            Invocation::Unissued => self.draft.source_invocation,
                            Invocation::Exposed(ordinal) => ordinal,
                        },
                    };
                    if matches!(
                        pending.action,
                        surgeist_html::HtmlDialogAction::RememberFocusedElement
                            | surgeist_html::HtmlDialogAction::RunDialogFocusingSteps
                    ) {
                        crate::source::watcher::DialogOccurrence::Method(occurrence)
                    } else {
                        crate::source::watcher::DialogOccurrence::Hook(occurrence)
                    }
                }),
            })
        } else {
            None
        };
        // Form every source-time input against this PRIVATE final mapping.
        // A refusal here retains the previous receipt and rolls back the segment;
        // no published command is stranded behind input allocation or lookup.
        let source = frame
            .map(|frame| {
                self.source(
                    &prepared.state,
                    &ids,
                    DomStamp {
                        owner: dom.owner,
                        revision: DomRevision(
                            dom.revision
                                .0
                                .checked_add(u64::from(changed))
                                .ok_or(DomError::RevisionExhausted)?,
                        ),
                    },
                    frame,
                    prepared.formation.as_ref(),
                    cancel,
                )
            })
            .transpose()?;
        let request = if let (Some(source), Some(frame)) = (&source, frame) {
            let action = frame.cursor.action();
            let state = self.draft.element(frame.key)?.style_state().expect("Style");
            let sheet = state.sheet.clone();
            let association = if action == DomStyleAction::Update(Action::CreateSheet) {
                let serial = state
                    .next_association
                    .checked_add(1)
                    .ok_or(DomError::RepresentationExhausted)?;
                identities = identities
                    .checked_add(1)
                    .ok_or(DomError::RepresentationExhausted)?;
                if identities > limits.identities {
                    return Err(DomError::SourceIdentityLimit.into());
                }
                Some(DomStyleAssociationId {
                    element: source.element,
                    serial,
                })
            } else {
                sheet.as_ref().map(|s| s.association)
            };
            Some(DomStyleRequest {
                source: source.clone(),
                action,
                cssom: None,
                sheet,
                association,
                sheet_order: if action == DomStyleAction::Update(Action::CreateSheet) {
                    Some(self.sheet_order(
                        &prepared.state,
                        source,
                        prepared.formation.as_ref(),
                        cancel,
                    )?)
                } else {
                    None
                },
            })
        } else {
            None
        };
        let child = if self.child.is_none() {
            self.draft
                .hosted_pending
                .map(|hook| {
                    hosted::Child::form(
                        self,
                        &prepared.state,
                        &ids,
                        DomStamp {
                            owner: dom.owner,
                            revision: DomRevision(
                                dom.revision
                                    .0
                                    .checked_add(u64::from(changed))
                                    .ok_or(DomError::RevisionExhausted)?,
                            ),
                        },
                        hook,
                        prepared.formation.as_ref(),
                        cancel,
                    )
                })
                .transpose()?
        } else {
            None
        };
        let canonical_storage = if changed {
            storage::state(&prepared.state, || self.draft.source_step(cancel))?
        } else {
            self.canonical_storage
        };
        // Newly formed source inputs already retain their byte/unit admission
        // in this candidate's Formation. Reused input is charged by context.
        self.draft.source_admit_units(0, cancel)?;
        let canonical_history_bytes =
            self.history_after(dom, changed, prepared.changes.len(), cancel)?;
        // Nothing after canonical publication is fallible. Preserve original
        // keys/serials and consume only evidence that has actually published.
        if changed
            && self
                .headroom
                .as_ref()
                .is_some_and(|headroom| headroom.remaining_phases == 0)
        {
            return Err(DomError::StyleProtocol.into());
        }
        let permit = hosted
            .map(|(source, request, revision)| source.permit(request, revision, false, cancel))
            .transpose()?;
        let owner_custody = dom.source_owner_custody_bytes()?;
        let commit = dom.publish(prepared, cancel)?;
        self.draft.dispatch_retirements.clear();
        if let Some(permit) = permit {
            permit.commit();
        }
        if self.hosted_global.take().is_some() {
            self.draft.hosted_globals = dom.state.hosted_globals.clone();
        }
        // Retirement only removes entries; capacity is retained and this cannot
        // overflow after the same table was admitted before publication.
        self.custody_bytes = owner_custody;
        self.canonical_storage = canonical_storage;
        self.canonical_history_bytes = canonical_history_bytes;
        self.draft
            .phase_map
            .as_mut()
            .expect("source map")
            .ids
            .iter_mut()
            .zip(ids)
            .for_each(|(old, new)| {
                if old.is_none() {
                    *old = new;
                }
            });
        let draft: &mut Draft = &mut self.draft;
        let private = &draft.phase_map.as_ref().expect("source map").private;
        let mut published = dom.state.native_tasks.iter();
        for pending in &mut draft.native_tasks {
            if matches!(pending, select::Pending::Update {target:Key::Created(i),..} if private[*i])
            {
                continue;
            }
            let published = published.next().expect("visible task");
            if let (
                select::Pending::Update { serial, .. },
                select::Task::Update { serial: actual, .. },
            ) = (pending, published)
            {
                *serial = Some(*actual);
            }
        }
        self.draft.view_transitions.acknowledge_publication(
            &dom.state.view_transitions,
            &commit.transition_ids,
            &commit.record_ids,
        );
        self.draft
            .acknowledge_details_publication(&dom.state.details_intentions);
        self.draft.cancelled_signals.clear();
        self.draft.cancelled_selects.clear();
        self.draft.cancelled_details.clear();
        self.draft.cancelled_images = None;
        if let Some(next) = issued_next {
            self.draft.source_invocation = next;
            if let Some(occurrence) = dialog.as_ref().and_then(|request| request.method()) {
                self.draft
                    .dialog_method
                    .as_mut()
                    .expect("exposed actual Show")
                    .invocation = Invocation::Exposed(occurrence.ordinal);
            } else {
                match &mut self
                    .draft
                    .effects
                    .last_mut()
                    .expect("exposed actual frame")
                    .effect
                {
                    select::Effect::Style(actual) => {
                        actual.invocation = Invocation::Exposed(
                            source.as_ref().expect("issued Style source").ordinal,
                        )
                    }
                    select::Effect::DialogHook(actual) => {
                        actual.invocation = Invocation::Exposed(
                            dialog
                                .as_ref()
                                .and_then(|request| request.hook())
                                .expect("issued Hook request")
                                .ordinal,
                        )
                    }
                    _ => unreachable!("qualified external invocation"),
                }
            }
        }
        if let Some(source) = source {
            self.context = Some((source.ordinal, source));
        }
        self.request = request;
        if let Some(child) = child {
            self.draft.source_invocation = child.request.occurrence;
            self.draft.hosted_pending = None;
            self.child = Some(child);
        }
        self.historical = historical;
        self.outputs = outputs;
        self.identities = identities;
        if changed && let Some(headroom) = &mut self.headroom {
            headroom.remaining_phases -= 1;
        }
        self.phases += u64::from(changed);
        self.published = true;
        Ok(PublishedSegment { commit, dialog })
    }
}
impl DomSourceLease<'_> {
    pub fn status(&self) -> DomSourceStatus {
        self.dom.pending_source_edit().expect("live lease")
    }
    pub fn operation(&self) -> DomSourceOperationId {
        self.id
    }
    pub fn request_cancel(&mut self) {
        let op = self.dom.source_operation.as_mut().expect("live lease");
        op.cancelled = true;
        // Unstarted commands have no canonical obligations. The current command
        // retains every mandatory source frame until explicit recovery finishes.
    }
    pub fn snapshot(&self) -> DomSnapshot {
        self.dom.snapshot()
    }
    pub fn advance(&mut self, cancel: &AtomicBool) -> Result<DomSourceProgress, DomError> {
        check_cancel(cancel)?;
        if !self
            .dom
            .source_operation
            .as_ref()
            .is_some_and(|operation| operation.id == self.id)
        {
            return Err(DomError::InvalidSourceOperation);
        }
        self.dom
            .source_operation
            .as_mut()
            .expect("qualified original operation")
            .entry_scope
            .seal();
        if self
            .dom
            .source_operation
            .as_ref()
            .is_some_and(|operation| matches!(operation.commands, OperationRoot::DialogTask(_)))
            && !self.task_authorization.as_ref().is_some_and(|binding| {
                self.dom
                    .element_tasks
                    .as_ref()
                    .and_then(std::sync::Weak::upgrade)
                    .is_some_and(|actual| Arc::ptr_eq(binding, &actual))
            })
        {
            return Err(DomError::InvalidSourceOperation);
        }
        // Independent public capabilities may commit concurrently. Keep the
        // exact owner locked from receipt validation through canonical source
        // publication; this segment executes no external callbacks.
        if self
            .dom
            .source_operation
            .as_ref()
            .is_some_and(|op| op.observation().is_some())
        {
            return self.advance_dialog_observation(cancel);
        }
        let root = self
            .dom
            .source_operation
            .as_ref()
            .and_then(|op| {
                matches!(op.receipt, Some(Receipt::Cssom { .. })).then(|| {
                    let request = op.request.as_ref().ok_or(DomError::StyleProtocol)?;
                    self.cssom_root(request)
                })
            })
            .transpose()?;
        let locked = root
            .as_ref()
            .map(DomStyleCssomRoot::lock)
            .transpose()
            .map_err(DomStyleCssomRootError::into_dom_error)?;
        if let Some(store) = &locked {
            let op = self
                .dom
                .source_operation
                .as_ref()
                .ok_or(DomError::InvalidSourceOperation)?;
            if let Some(Receipt::Cssom { revision, .. }) = &op.receipt
                && store.snapshot().revision() != revision
            {
                return Err(DomError::StyleReceiptMismatch);
            }
        }
        let custody = self.dom.source_owner_custody_bytes()?;
        let mut op = self
            .dom
            .source_operation
            .take()
            .ok_or(DomError::InvalidSourceOperation)?;
        op.custody_bytes = custody;
        let result = self.advance_operation(&mut op, cancel);
        if self.complete || matches!(&result, Ok(DomSourceProgress::Complete(_))) {
            self.complete = true;
        } else {
            self.dom.source_operation = Some(op);
        }
        if self.complete {
            self.dom.source_operation = self.dom.source_parents.pop();
        }
        result
    }
    fn advance_operation(
        &mut self,
        op: &mut Operation,
        cancel: &AtomicBool,
    ) -> Result<DomSourceProgress, DomError> {
        if !op.event_exposed
            && let OperationRoot::DialogTask(task) = op.commands
        {
            return op.observe_dialog_task(self.dom, task, cancel);
        }
        if op.cancelled && !op.published && !op.event_exposed {
            // No canonical phase or external effect occurred. Return the actual
            // current stamp directly; a second Draft/candidate would only copy
            // the unchanged tree while the original admitted operation lives.
            op.refresh_admission(cancel)?;
            let formation = formation::Formation::new(&op.draft, cancel)?;
            let budget = formation.as_ref().expect("source cancellation admission");
            budget.retain_bytes(size_of::<DomSourceCompletion>(), cancel)?;
            let mut ticket_count = 0usize;
            for command in &op.commands {
                op.draft.source_step(cancel)?;
                if matches!(
                    command,
                    Command::Create { .. }
                        | Command::Import { .. }
                        | Command::AttachShadow { .. }
                        | Command::CreateHtmlDocument { .. }
                        | Command::CreateXmlDocument { .. }
                ) {
                    ticket_count = ticket_count
                        .checked_add(1)
                        .ok_or(DomError::RepresentationExhausted)?;
                }
            }
            let mut published = Vec::new();
            formation::reserve(&mut published, ticket_count, formation.as_ref(), cancel)?;
            for _ in 0..ticket_count {
                op.draft.source_step(cancel)?;
                published.push(None);
            }
            let stamp = DomStamp {
                owner: self.dom.owner,
                revision: self.dom.revision,
            };
            check_cancel(cancel)?;
            let commit = DomCommit {
                owner: op.draft.owner.clone(),
                created: Vec::new(),
                documents: Vec::new(),
                transition_ids: Vec::new(),
                record_ids: Vec::new(),
                changes: DomChanges {
                    from: stamp,
                    to: stamp,
                    changes: Arc::from([]),
                },
            };
            return Ok(DomSourceProgress::Complete(DomSourceCompletion {
                operation: op.id,
                cancelled: true,
                published,
                commit,
                installation: None,
            }));
        }
        if op.child.as_ref().is_some_and(hosted::Child::returned) {
            op.child = None;
        }
        if let Some(child) = &op.child {
            return Ok(DomSourceProgress::Request(DomSourceRequest::HostedChild(
                child.request.clone(),
            )));
        }
        if let Some(request) = &op.request
            && op.receipt.is_none()
        {
            return Ok(DomSourceProgress::Request(DomSourceRequest::Style(
                request.clone(),
            )));
        }
        loop {
            if op.request_close.is_some()
                && !op.request_close.as_ref().is_some_and(|call| call.nested)
                && let Some(progress) = self.advance_request_close(op, cancel)?
            {
                return Ok(progress);
            }
            op.refresh_admission(cancel)?;
            if op.published
                && !op.started
                && !op.cancelled
                && !op.command_admitted
                && op.index < op.commands.len()
            {
                op.admit_current_command(self.dom, cancel)?;
            }
            let checkpoint = op.draft.source_checkpoint(op.input_bytes()?, 0, cancel)?;
            let started = op.started;
            let command_index = op.index;
            let external_applied = op.external_receipt_applied();
            let result = (|| {
                if op.receipt.is_some() {
                    op.apply_receipt(cancel)?;
                }
                if !op.started
                    && !op.cancelled
                    && let Some(select) = &op.select
                {
                    let auth = self
                        .select_authorization
                        .as_ref()
                        .ok_or(DomError::InvalidNativeTask)?;
                    if auth.id != select.operation
                        || auth.target != select.target
                        || auth.status() != 0
                    {
                        return Err(DomError::InvalidNativeTask);
                    }
                    let key = op.draft.resolve(select.target.into())?;
                    op.draft
                        .schedule_selectedcontent(key, self.dom.limits, cancel)
                } else if !op.started
                    && !op.cancelled
                    && let Some(installation) = &op.installation
                {
                    op.draft.pre_insert(
                        installation.wrapper,
                        installation.parent,
                        installation.before,
                        self.dom.limits,
                        cancel,
                    )
                } else if op.started && op.draft.effects.len() > op.draft.effect_floor {
                    op.draft.run_effects(self.dom.limits, cancel)
                } else if let Some(frame) = op.draft.dialog_method {
                    let input = if op.request_close.as_ref().is_some_and(|call| call.nested) {
                        crate::store::dialog::MethodInput::Close(op.request_close_result.as_ref())
                    } else {
                        match &op.commands[op.index] {
                            Command::RequestCloseDialog { result, .. } => {
                                crate::store::dialog::MethodInput::RequestClose(result.as_ref())
                            }
                            Command::CloseDialog { result, .. } => {
                                crate::store::dialog::MethodInput::Close(result.as_ref())
                            }
                            _ => crate::store::dialog::MethodInput::Show,
                        }
                    };
                    op.draft
                        .dialog_method_boundary(frame.subject, input, self.dom.limits, cancel)
                } else if op.started {
                    op.draft.run_effects(self.dom.limits, cancel)
                } else if op.index < op.commands.len() && !op.cancelled {
                    op.apply_command(self.dom.limits, cancel)
                } else {
                    Ok(())
                }
            })();
            match result {
                Err(DomError::NeedsEventfulOperation {
                    subject: Some(subject),
                    action,
                }) => {
                    let observed = op.observe_dialog_method(self.dom, subject, action, cancel);
                    if observed.is_err() {
                        checkpoint.restore(&mut op.draft);
                        op.restore_external_receipt(external_applied);
                    }
                    return observed;
                }
                Err(DomError::DialogProcessingUnavailable {
                    action:
                        surgeist_html::HtmlDialogAction::RequestWatcherClose {
                            require_history_action_activation,
                        },
                }) => {
                    let started_call =
                        self.begin_request_close(op, require_history_action_activation, cancel);
                    if let Err(error) = started_call {
                        checkpoint.restore(&mut op.draft);
                        op.restore_external_receipt(external_applied);
                        op.started = started;
                        return Err(error);
                    }
                    drop(checkpoint);
                    continue;
                }
                Err(DomError::DialogProcessingUnavailable { action })
                    if op.draft.pending_dialog(action).is_some() =>
                {
                    let pending = op
                        .draft
                        .pending_dialog(action)
                        .expect("qualified actual Dialog stop");
                    if matches!(
                        action,
                        surgeist_html::HtmlDialogAction::QueueToggle { .. }
                            | surgeist_html::HtmlDialogAction::QueueClose
                    ) {
                        let subject = op
                            .draft
                            .published_id(pending.subject)
                            .ok_or(DomError::UnavailableCreation)?;
                        drop(checkpoint);
                        return Ok(DomSourceProgress::Request(DomSourceRequest::Dialog(
                            crate::DomSourceDialogRequest {
                                operation: op.id,
                                subject,
                                action,
                                occurrence: None,
                            },
                        )));
                    }
                    if !op.command_admitted {
                        checkpoint.restore(&mut op.draft);
                        op.restore_external_receipt(external_applied);
                        op.started = started;
                        op.admit_current_command(self.dom, cancel)?;
                        continue;
                    }
                    op.started = true;
                    let published = match op.publication(
                        self.dom,
                        true,
                        PublicationTarget::Dialog(pending),
                        cancel,
                    ) {
                        Ok(published) => published,
                        Err(error) => {
                            checkpoint.restore(&mut op.draft);
                            op.restore_external_receipt(external_applied);
                            op.started = started;
                            return Err(error);
                        }
                    };
                    drop(checkpoint);
                    op.receipt = None;
                    return Ok(DomSourceProgress::Request(DomSourceRequest::Dialog(
                        published
                            .dialog
                            .expect("actual request formed before publication"),
                    )));
                }
                Err(DomError::StyleCompositionRequired(_))
                | Err(DomError::StyleOwnerAttributesCompositionRequired)
                | Err(DomError::StyleDisabledCompositionRequired) => {
                    if !op.command_admitted {
                        checkpoint.restore(&mut op.draft);
                        op.restore_external_receipt(external_applied);
                        op.started = started;
                        op.admit_current_command(self.dom, cancel)?;
                        continue;
                    }
                    op.started = true;
                    let frame = op.frame()?;
                    if let Err(e) =
                        op.publication(self.dom, true, PublicationTarget::Style(frame), cancel)
                    {
                        checkpoint.restore(&mut op.draft);
                        op.restore_external_receipt(external_applied);
                        op.started = started;
                        return Err(e);
                    }
                    // The old receipt remains retained until its DOM segment has
                    // actually published; a failed segment never reissues CSSOM.
                    op.receipt = None;
                    op.owner_binding = None;
                    let request = op
                        .request
                        .as_ref()
                        .expect("request formed before publication")
                        .clone();
                    return Ok(DomSourceProgress::Request(DomSourceRequest::Style(request)));
                }
                Err(DomError::HostedCompositionRequired) => {
                    if !op.command_admitted {
                        checkpoint.restore(&mut op.draft);
                        op.restore_external_receipt(external_applied);
                        op.started = started;
                        op.admit_current_command(self.dom, cancel)?;
                        continue;
                    }
                    op.started = true;
                    if let Err(error) =
                        op.publication(self.dom, true, PublicationTarget::None, cancel)
                    {
                        checkpoint.restore(&mut op.draft);
                        op.restore_external_receipt(external_applied);
                        op.started = started;
                        return Err(error);
                    }
                    let child = op.child.as_ref().ok_or(DomError::HostedSourceUnavailable)?;
                    return Ok(DomSourceProgress::Request(DomSourceRequest::HostedChild(
                        child.request.clone(),
                    )));
                }
                Err(e) => {
                    checkpoint.restore(&mut op.draft);
                    op.restore_external_receipt(external_applied);
                    op.started = started;
                    return Err(e);
                }
                Ok(()) => {
                    if op.draft.dialog_method.is_some() {
                        continue;
                    }
                    if op.request_close.as_ref().is_some_and(|call| call.nested) {
                        let call = op.request_close.as_mut().expect("nested original call");
                        call.nested = false;
                        call.cursor
                            .reply(surgeist_html::HtmlCloseWatcherRequestReply::Applied)
                            .map_err(DomError::CloseWatcherProtocol)?;
                        op.draft.dialog_method = Some(call.outer);
                        op.request_close_result = None;
                        op.receipt = None;
                        continue;
                    }
                    // The real current command has drained. Excess conservative
                    // capacity is released while one actual final close remains.
                    if let Some(headroom) = &mut op.headroom {
                        headroom.remaining_phases = 1;
                    }

                    if !matches!(
                        op.receipt,
                        Some(
                            Receipt::Watcher { .. }
                                | Receipt::DialogFocus { .. }
                                | Receipt::CloseWatcher { .. }
                        )
                    ) {
                        op.receipt = None;
                        op.request = None;
                        op.owner_binding = None;
                    }
                    if op.index < op.commands.len() && (!op.cancelled || op.started) {
                        op.index += 1;
                        op.started = false;
                        op.command_admitted = false;
                        op.context = None;
                    }
                    if op.index == op.commands.len() || op.cancelled {
                        if let Some(select) = &mut op.select
                            && (!op.cancelled || op.started)
                            && !select.finished
                        {
                            if !matches!(op.draft.native_tasks.first(), Some(select::Pending::Update {serial:Some(serial),target}) if *serial==select.operation.serial && op.draft.resolve(select.target.into()).is_ok_and(|key|key==*target))
                            {
                                return Err(DomError::InvalidNativeTask);
                            }
                            op.draft.native_tasks.remove(0);
                            select.finished = true;
                        }
                        let installation_output = op.installation_output(self.dom, cancel)?;
                        let commit = match op.publication(
                            self.dom,
                            false,
                            PublicationTarget::None,
                            cancel,
                        ) {
                            Ok(published) => published.commit,
                            Err(error) => {
                                checkpoint.restore(&mut op.draft);
                                op.restore_external_receipt(external_applied);
                                op.started = started;
                                op.index = command_index;
                                return Err(error);
                            }
                        };
                        op.receipt = None;
                        op.request = None;
                        op.owner_binding = None;
                        drop(checkpoint);
                        let installation = installation_output.map(|root| {
                            let input = op.installation.as_mut().expect("installation root");
                            let bindings = std::mem::take(&mut input.bindings);
                            let payload = input.bindings_payload.take();
                            let result = crate::DomNativeHtmlInstallation {
                                destination: input.destination,
                                source_root: input.source_root.clone(),
                                root,
                                snapshot: self.dom.snapshot(),
                                changes: commit.changes.clone(),
                                bindings,
                            };
                            drop(payload); // The completed result now owns the real bank.
                            result
                        });
                        if op.select.as_ref().is_some_and(|s| s.finished) {
                            self.select_authorization
                                .as_ref()
                                .expect("Select permission")
                                .consume();
                            self.dom.active_task = std::sync::Weak::new();
                        }
                        return Ok(DomSourceProgress::Complete(DomSourceCompletion {
                            operation: op.id,
                            cancelled: op.cancelled,
                            published: std::mem::take(&mut op.historical),
                            commit,
                            installation,
                        }));
                    }
                }
            }
        }
    }
    pub fn accept_inline_csp(
        &mut self,
        request: &DomStyleRequest,
        observation: &DomStyleInlineCspObservation,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let op = self.dom.source_operation.as_mut().expect("live lease");
        let expected = op.request.as_ref().ok_or(DomError::StyleProtocol)?;
        if expected.action != DomStyleAction::Update(Action::CheckInlineCsp)
            || !Arc::ptr_eq(&expected.source, &request.source)
            || observation.source.operation != request.source.operation
            || observation.source.ordinal != request.source.ordinal
            || observation.source.stamp != request.source.stamp
            || observation.source.element != request.source.element
            || !Arc::ptr_eq(&observation.source.text, &request.source.text)
            || !Arc::ptr_eq(
                observation.source.nonce.shared_code_units(),
                request.source.nonce.shared_code_units(),
            )
            || op.receipt.is_some()
        {
            return Err(DomError::StyleFactsMismatch);
        }
        op.refresh_admission(cancel)?;
        op.draft.source_step(cancel)?;
        // Observation owns its source clone as well as its policy bank. Its
        // inline header is already in the admitted Operation layout; charge
        // every retained out-of-line occurrence and its UTF16 capacity here.
        let source_bytes = observation
            .source
            .retained_bytes()?
            .checked_sub(size_of::<DomStyleSource>())
            .ok_or(DomError::RepresentationExhausted)?;
        let mut bytes = size_of::<Vec<DocumentString>>()
            .checked_add(source_bytes)
            .ok_or(DomError::RepresentationExhausted)?
            .checked_add(
                observation
                    .policies
                    .capacity()
                    .checked_mul(size_of::<DocumentString>())
                    .ok_or(DomError::RepresentationExhausted)?,
            )
            .ok_or(DomError::RepresentationExhausted)?;
        op.draft.source_admit(bytes, cancel)?;
        let mut units = observation.source.retained_units()?;
        for policy in observation.policies.iter() {
            op.draft.source_step(cancel)?;
            let capacity = policy.shared_code_units().capacity();
            bytes = bytes
                .checked_add(size_of::<Vec<u16>>())
                .and_then(|n| n.checked_add(capacity.checked_mul(size_of::<u16>())?))
                .ok_or(DomError::RepresentationExhausted)?;
            units = units
                .checked_add(capacity)
                .ok_or(DomError::RepresentationExhausted)?;
            op.draft.source_admit(bytes, cancel)?;
            for _ in policy.code_units() {
                op.draft.source_step(cancel)?;
            }
        }
        op.draft.source_admit(bytes, cancel)?;
        op.draft.source_admit_units(units, cancel)?;
        op.receipt = Some(Receipt::Inline {
            observation: observation.clone(),
            bytes,
            units,
        });
        Ok(())
    }
    /// Retains the exact mutable owner before any external mutation. Rebinding
    /// a live canonical root to another store is refused even at equal revision.
    pub fn bind_cssom_root(
        &mut self,
        request: &DomStyleRequest,
        root: &DomStyleCssomRoot,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let op = self.dom.source_operation.as_mut().expect("live lease");
        let expected = op.request.as_ref().ok_or(DomError::StyleProtocol)?;
        if !Arc::ptr_eq(&expected.source, &request.source)
            || expected.action != request.action
            || expected.cssom_root() != Some(root.root())
            || op.receipt.is_some()
            || !matches!(
                expected.source.root_kind,
                DomStyleRootKind::Document | DomStyleRootKind::ShadowRoot
            ) && expected.sheet.is_none()
        {
            return Err(DomError::StyleReceiptMismatch);
        }
        op.refresh_admission(cancel)?;
        for existing in &self.dom.style_cssom_roots {
            op.draft.source_step(cancel)?;
            if existing.root() == root.root() {
                return if existing.same_owner(root) {
                    Ok(())
                } else {
                    Err(DomError::StyleReceiptMismatch)
                };
            }
        }
        self.dom.state.node(self.dom.owner, root.root())?;
        // Readability is established before installing canonical custody.
        root.snapshot()
            .map_err(DomStyleCssomRootError::into_dom_error)?;
        let count = self
            .dom
            .style_cssom_roots
            .len()
            .checked_add(1)
            .ok_or(DomError::RepresentationExhausted)?;
        let requested = count
            .checked_mul(size_of::<DomStyleCssomRoot>())
            .and_then(|n| n.checked_add(DomStyleCssomRoot::retained_owner_bytes()))
            .ok_or(DomError::RepresentationExhausted)?;
        op.draft.source_admit(requested, cancel)?;
        let mut replacement = Vec::new();
        replacement
            .try_reserve_exact(count)
            .map_err(|_| DomError::AllocationFailed)?;
        let additional = replacement
            .capacity()
            .checked_mul(size_of::<DomStyleCssomRoot>())
            .and_then(|n| n.checked_add(DomStyleCssomRoot::retained_owner_bytes()))
            .ok_or(DomError::RepresentationExhausted)?;
        op.draft.source_admit(additional, cancel)?;
        for existing in &self.dom.style_cssom_roots {
            op.draft.source_step(cancel)?;
            replacement.push(existing.duplicate());
        }
        op.draft.source_step(cancel)?;
        replacement.push(root.duplicate());
        let custody_bytes = replacement
            .capacity()
            .checked_mul(size_of::<DomStyleCssomRoot>())
            .and_then(|n| n.checked_add(size_of::<Vec<DomStyleCssomRoot>>()))
            .and_then(|n| {
                n.checked_add(count.checked_mul(DomStyleCssomRoot::retained_owner_bytes())?)
            })
            .ok_or(DomError::RepresentationExhausted)?;
        self.dom.style_cssom_roots = replacement;
        op.custody_bytes = custody_bytes;
        Ok(())
    }
    /// Reacquires the exact retained owner for this reached request. No snapshot
    /// or reconstructed store substitutes for the live resource owner.
    pub fn cssom_root(&self, request: &DomStyleRequest) -> Result<DomStyleCssomRoot, DomError> {
        let op = self.dom.source_operation.as_ref().expect("live lease");
        let expected = op.request.as_ref().ok_or(DomError::StyleProtocol)?;
        if !Arc::ptr_eq(&expected.source, &request.source) || expected.action != request.action {
            return Err(DomError::StyleReceiptMismatch);
        }
        // This allocation-free access has no cancellation argument. It still
        // consumes the current operation's absolute source-work allowance.
        for root in &self.dom.style_cssom_roots {
            op.draft.source_step(&AtomicBool::new(false))?;
            if Some(root.root()) == expected.cssom_root() {
                return Ok(root.duplicate());
            }
        }
        Err(DomError::StyleReceiptMismatch)
    }
    /// Qualifies this actual external step with the consumer's root-to-CSSOM
    /// binding. It declares no readiness or owner association by itself.
    pub fn qualify_cssom(
        &mut self,
        request: &DomStyleRequest,
        root: NodeId,
        snapshot: &CssomSnapshot,
    ) -> Result<DomStyleRequest, DomError> {
        let retained = self.cssom_root(request)?;
        let locked = retained
            .lock()
            .map_err(DomStyleCssomRootError::into_dom_error)?;
        let actual = locked.snapshot();
        if actual.revision() != snapshot.revision() {
            return Err(DomError::StyleReceiptMismatch);
        }
        let op = self.dom.source_operation.as_mut().expect("live lease");
        let expected = op.request.as_mut().ok_or(DomError::StyleProtocol)?;
        if !Arc::ptr_eq(&expected.source, &request.source)
            || expected.action != request.action
            || expected.cssom_root() != Some(root)
            || op.receipt.is_some()
        {
            return Err(DomError::StyleReceiptMismatch);
        }
        if expected.action == DomStyleAction::Update(Action::CreateSheet) {
            if !matches!(
                expected.source.root_kind,
                DomStyleRootKind::Document | DomStyleRootKind::ShadowRoot
            ) {
                return Err(DomError::WrongNodeKind);
            }
        } else {
            snapshot
                .sheet(
                    &expected
                        .sheet
                        .as_ref()
                        .ok_or(DomError::StyleReceiptMismatch)?
                        .id,
                )
                .map_err(|_| DomError::StyleReceiptMismatch)?;
        }
        if op.owner_binding.is_some() && expected.cssom.as_ref() != Some(snapshot.revision()) {
            return Err(DomError::StyleReceiptMismatch);
        }
        expected.cssom = Some(snapshot.revision().clone());
        Ok(expected.clone())
    }
    pub fn bind_cssom_owner(
        &mut self,
        request: &DomStyleRequest,
        binding: DomStyleCssomOwnerBinding,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let op = self.dom.source_operation.as_mut().expect("live lease");
        let expected = op.request.as_ref().ok_or(DomError::StyleProtocol)?;
        if !matches!(
            expected.action,
            DomStyleAction::Update(Action::CreateSheet) | DomStyleAction::OwnerAttributes
        ) || !Arc::ptr_eq(&expected.source, &request.source)
            || !binding.matches(expected)
            || op.receipt.is_some()
        {
            return Err(DomError::StyleReceiptMismatch);
        }
        op.refresh_admission(cancel)?;
        op.draft.source_admit(binding.retained_bytes()?, cancel)?;
        for _ in binding
            .owner()
            .bytes()
            .chain(binding.version().identity.bytes())
        {
            op.draft.source_step(cancel)?;
        }
        op.owner_binding = Some(binding);
        Ok(())
    }
    #[expect(
        clippy::result_large_err,
        reason = "Return the actual non-Clone publication and snapshot allocation-free after invalid qualification; boxing adds a fallible allocation at recovery"
    )]
    pub fn accept_cssom_publication(
        &mut self,
        request: &DomStyleRequest,
        commit: CssomCommit,
        snapshot: CssomSnapshot,
        ticket: Option<&CssomSheetTicket>,
    ) -> Result<Option<DomStyleNotificationToken>, DomStyleCssomPublicationRejection> {
        let retained = match self.cssom_root(request) {
            Ok(root) => root,
            Err(error) => {
                return Err(DomStyleCssomPublicationRejection {
                    error,
                    commit,
                    snapshot,
                });
            }
        };
        let locked = match retained.lock() {
            Ok(locked) => locked,
            Err(error) => {
                return Err(DomStyleCssomPublicationRejection {
                    error: error.into_dom_error(),
                    commit,
                    snapshot,
                });
            }
        };
        if locked.snapshot().revision() != snapshot.revision() {
            return Err(DomStyleCssomPublicationRejection {
                error: DomError::StyleReceiptMismatch,
                commit,
                snapshot,
            });
        }
        // Acknowledgement cannot discard actual synchronous owner writes. The
        // caller retains both producer values and must service their effects.
        if !commit.owner_effects().is_empty() {
            return Err(DomStyleCssomPublicationRejection {
                error: DomError::StyleProtocol,
                commit,
                snapshot,
            });
        }
        let validated = self.validate_cssom_publication(request, &commit, &snapshot, ticket);
        let sheet = match validated {
            Ok(sheet) => sheet,
            Err(error) => {
                return Err(DomStyleCssomPublicationRejection {
                    error,
                    commit,
                    snapshot,
                });
            }
        };
        let op = self.dom.source_operation.as_mut().expect("live lease");
        let token = if request.action == DomStyleAction::Update(Action::CreateSheet) {
            Some(DomStyleNotificationToken {
                operation: op.id,
                association: op
                    .request
                    .as_ref()
                    .and_then(|r| r.association)
                    .expect("validated actual creation association"),
                complete: false,
            })
        } else {
            None
        };
        // Consuming retention precedes every cancellation poll/DOM allocation.
        op.receipt = Some(Receipt::Cssom {
            revision: snapshot.revision().clone(),
            sheet,
        });
        Ok(token)
    }
    fn validate_cssom_publication(
        &self,
        request: &DomStyleRequest,
        commit: &CssomCommit,
        snapshot: &CssomSnapshot,
        ticket: Option<&CssomSheetTicket>,
    ) -> Result<Option<surgeist_cssom::CssomSheetId>, DomError> {
        let op = self.dom.source_operation.as_ref().expect("live lease");
        let expected = op.request.as_ref().ok_or(DomError::StyleProtocol)?;
        if !Arc::ptr_eq(&expected.source, &request.source)
            || expected.action != request.action
            || expected.cssom != request.cssom
            || op.receipt.is_some()
        {
            return Err(DomError::StyleReceiptMismatch);
        }
        let (from, to) = match commit.publication() {
            CssomPublication::Unchanged { revision } => (revision, revision),
            CssomPublication::Changed(change) => (change.from(), change.to()),
        };
        if Some(from) != expected.cssom.as_ref() || to != snapshot.revision() {
            return Err(DomError::StyleReceiptMismatch);
        }
        let sheet = match expected.action {
            DomStyleAction::Update(Action::CreateSheet) => {
                expected.association.ok_or(DomError::StyleReceiptMismatch)?;
                let id = commit
                    .sheet(ticket.ok_or(DomError::StyleReceiptMismatch)?)
                    .map_err(|_| DomError::StyleReceiptMismatch)?;
                let actual = snapshot
                    .sheet(id)
                    .map_err(|_| DomError::StyleReceiptMismatch)?;
                let binding = op
                    .owner_binding
                    .as_ref()
                    .filter(|b| b.matches(expected))
                    .ok_or(DomError::StyleReceiptMismatch)?;
                let title = expected.source.sheet_title();
                if actual.inputs().owner.as_deref() != Some(binding.owner())
                    || &actual.inputs().version != binding.version()
                    || actual.inputs().title.as_slice() != title
                    // An untitled ordinary inline sheet has no preferred-set
                    // selection that could set its initially-unset flag.
                    || (title.is_empty() && actual.inputs().disabled)
                    || actual.inputs().base_url.is_some()
                    || actual.inputs().location.is_some()
                    || actual.inputs().constructor_document.is_some()
                    || !expected.source.matches_media(snapshot, actual.media())
                    || actual.inputs().constructed
                    || !actual.inputs().origin_clean
                    || actual.inputs().alternate
                    || actual.inputs().disallow_modification
                    || !actual
                        .authored()
                        .parse_source()
                        .and_then(|p| p.source().original_utf16())
                        .is_some_and(|s| s.as_slice() == expected.source.text.as_slice())
                {
                    return Err(DomError::StyleReceiptMismatch);
                }
                if snapshot.sheets().len() != expected.sheet_order().len()
                    || !snapshot.sheets().iter().zip(expected.sheet_order()).all(
                        |(actual, selected)| {
                            actual
                                == match selected {
                                    DomStyleSheetPlacement::Existing(existing) => existing,
                                    DomStyleSheetPlacement::ThisStyle => id,
                                }
                        },
                    )
                {
                    return Err(DomError::StyleReceiptMismatch);
                }
                Some(id.clone())
            }
            DomStyleAction::Update(Action::RemoveAssociatedSheet) => {
                let old = expected
                    .sheet
                    .as_ref()
                    .ok_or(DomError::StyleReceiptMismatch)?;
                let actual = snapshot
                    .sheet(&old.id)
                    .map_err(|_| DomError::StyleReceiptMismatch)?;
                if actual.inputs().owner.is_some()
                    || actual.owner_rule().is_some()
                    || snapshot.sheets().contains(&old.id)
                {
                    return Err(DomError::StyleReceiptMismatch);
                }
                None
            }
            DomStyleAction::SetDisabled(disabled) => {
                let old = expected
                    .sheet
                    .as_ref()
                    .ok_or(DomError::StyleReceiptMismatch)?;
                let actual = snapshot
                    .sheet(&old.id)
                    .map_err(|_| DomError::StyleReceiptMismatch)?;
                if actual.inputs().disabled != disabled || ticket.is_some() {
                    return Err(DomError::StyleReceiptMismatch);
                }
                Some(old.id.clone())
            }
            DomStyleAction::OwnerAttributes => {
                let old = expected
                    .sheet
                    .as_ref()
                    .ok_or(DomError::StyleReceiptMismatch)?;
                let actual = snapshot
                    .sheet(&old.id)
                    .map_err(|_| DomError::StyleReceiptMismatch)?;
                let binding = op
                    .owner_binding
                    .as_ref()
                    .filter(|b| b.matches(expected))
                    .ok_or(DomError::StyleReceiptMismatch)?;
                let title = expected.source.sheet_title();
                if actual.inputs().owner.as_deref() != Some(binding.owner())
                    || &actual.inputs().version != binding.version()
                    || actual.inputs().title.as_slice() != title
                    || !expected.source.matches_media(snapshot, actual.media())
                    || !snapshot.sheets().contains(&old.id)
                    || ticket.is_some()
                {
                    return Err(DomError::StyleReceiptMismatch);
                }
                Some(old.id.clone())
            }
            _ => return Err(DomError::StyleReceiptMismatch),
        };
        Ok(sheet)
    }
}
impl Operation {
    fn apply_receipt(&mut self, cancel: &AtomicBool) -> Result<(), DomError> {
        if matches!(self.receipt, Some(Receipt::CloseWatcher { .. })) {
            return self.apply_close_watcher_receipt(cancel);
        }
        if let Some(Receipt::DialogFocus {
            publication,
            applied,
        }) = &self.receipt
        {
            if *applied {
                return Ok(());
            }
            let request = publication.request.clone();
            self.apply_dialog_focus(&request, cancel)?;
            if let Some(Receipt::DialogFocus { applied, .. }) = &mut self.receipt {
                *applied = true;
            }
            return Ok(());
        }
        if let Some(Receipt::Watcher {
            request,
            watcher,
            applied,
        }) = &self.receipt
        {
            if *applied {
                return Ok(());
            }
            let request = request.clone();
            let watcher = *watcher;
            self.apply_watcher_receipt(request, watcher, cancel)?;
            if let Some(Receipt::Watcher { applied, .. }) = &mut self.receipt {
                *applied = true;
            }
            return Ok(());
        }
        let request = self.request.as_ref().ok_or(DomError::StyleProtocol)?;
        let mut frame = self.frame()?;
        if frame.cursor.action() != request.action {
            return Err(DomError::StyleProtocol);
        }
        let reply = match self.receipt.as_ref().expect("actual retained receipt") {
            Receipt::Watcher { .. }
            | Receipt::DialogFocus { .. }
            | Receipt::CloseWatcher { .. } => {
                unreachable!("external receipt handled above")
            }
            Receipt::Inline { observation, .. } => Reply::InlineAllowed(observation.allowed),
            Receipt::Cssom { revision, sheet } => {
                let old = self.draft.element(frame.key)?.style_state().expect("Style");
                let mut state = old.clone();
                if matches!(
                    request.action,
                    DomStyleAction::OwnerAttributes | DomStyleAction::SetDisabled(_)
                ) {
                    let associated = state.sheet.as_mut().ok_or(DomError::StyleReceiptMismatch)?;
                    associated.revision = revision.clone();
                } else if let Some(id) = sheet {
                    let association = request.association.ok_or(DomError::StyleReceiptMismatch)?;
                    if state.next_association.checked_add(1) != Some(association.serial) {
                        return Err(DomError::StyleReceiptMismatch);
                    }
                    state.next_association = association.serial;
                    state.sheet = Some(DomStyleSheet {
                        id: id.clone(),
                        revision: revision.clone(),
                        root: request.cssom_root().ok_or(DomError::StyleReceiptMismatch)?,
                        association: DomStyleAssociationId {
                            element: request.source.element,
                            serial: state.next_association,
                        },
                    });
                } else {
                    state.sheet = None;
                }
                self.draft.element_mut(frame.key).set_style_state(state);
                Reply::Applied
            }
        };
        frame
            .cursor
            .reply(reply)
            .map_err(|_| DomError::StyleProtocol)?;
        self.draft.source_step(cancel)?;
        let last = self.draft.effects.last_mut().expect("actual Style frame");
        last.effect = select::Effect::Style(frame);
        Ok(())
    }
}

struct SelectRoot {
    operation: crate::DomNativeTaskId,
    target: NodeId,
    finished: bool,
}
impl Operation {
    pub(in crate::store) fn allows_select_checkpoint(&self, serial: u64, target: NodeId) -> bool {
        self.select
            .as_ref()
            .is_some_and(|s| s.operation.serial == serial && s.target == target)
    }
}
impl Dom {
    fn validate_style_select(&self, task: &crate::DomSelectTask) -> Result<(), DomError> {
        let auth = &task.authorization;
        if auth.id.owner != self.owner {
            return Err(DomError::ForeignOwner);
        }
        if auth.status() != 0
            || !self
                .active_task
                .upgrade()
                .is_some_and(|a| Arc::ptr_eq(&a, auth))
            || !matches!(self.state.native_tasks.first(), Some(select::Task::Update {serial,target}) if *serial == auth.id.serial && *target == auth.target)
        {
            return Err(DomError::InvalidNativeTask);
        }
        Ok(())
    }
    pub fn begin_select_completion(
        &mut self,
        task: &crate::DomSelectTask,
        checkpoint: crate::DomNativeCheckpointLimits,
        limits: DomSourceLimits,
        cancel: &AtomicBool,
    ) -> Result<DomSourceLease<'_>, DomError> {
        self.require_source_entry()?;
        self.validate_style_select(task)?;
        let mut lease = self.begin_source_root(
            OperationRoot::SelectCompletion(task.select()),
            Arc::new(BatchIdentity),
            limits,
            cancel,
        )?;
        let op = lease.dom.source_operation.as_mut().expect("new operation");
        op.draft.select_scratch_limit = checkpoint.scratch;
        op.draft.select_work_limit = checkpoint.work;
        op.select = Some(SelectRoot {
            operation: task.operation(),
            target: task.select(),
            finished: false,
        });
        lease.select_authorization = Some(task.authorization.clone());
        Ok(lease)
    }
    pub fn resume_select_completion(
        &mut self,
        id: DomSourceOperationId,
        task: &crate::DomSelectTask,
        limits: DomSourceLimits,
        cancel: &AtomicBool,
    ) -> Result<DomSourceLease<'_>, DomError> {
        self.validate_style_select(task)?;
        let op = self
            .source_operation
            .as_ref()
            .filter(|o| o.id == id)
            .ok_or(DomError::InvalidSourceOperation)?;
        if !op.allows_select_checkpoint(task.operation().serial, task.select()) {
            return Err(DomError::InvalidNativeTask);
        }
        let mut lease = self.resume_source_permission(id, limits, cancel)?;
        lease.select_authorization = Some(task.authorization.clone());
        Ok(lease)
    }
}

impl Dom {
    pub fn begin_native_installation(
        &mut self,
        target: crate::DomNativeHtmlInstallTarget,
        source: &surgeist_html::HtmlFragment,
        budgets: crate::DomSourceNativeHtmlInstallLimits,
        cancel: &AtomicBool,
    ) -> Result<DomSourceLease<'_>, crate::DomNativeHtmlInstallError> {
        use crate::DomNativeHtmlInstallError as Error;
        let crate::DomNativeHtmlInstallTarget {
            basis,
            parent,
            before,
        } = target;
        let crate::DomSourceNativeHtmlInstallLimits {
            native: installation_limits,
            source: limits,
        } = budgets;
        self.require_source_entry().map_err(Error::Target)?;
        self.next_source_operation
            .checked_add(1)
            .ok_or(Error::Target(DomError::RepresentationExhausted))?;
        let original_limits = limits;
        let limits = self
            .source_child_limits(limits, cancel)
            .map_err(Error::Target)?;
        let nested = self
            .source_operation
            .as_ref()
            .is_some_and(|operation| operation.permits_reentry());
        if nested {
            self.push_source_observer(cancel).map_err(Error::Target)?;
        }
        let id = DomSourceOperationId {
            owner: self.owner,
            serial: self.next_source_operation,
        };
        let formed = (|| {
            let limits = if nested {
                self.source_rebase_limits(original_limits, cancel)
                    .map_err(Error::Target)?
            } else {
                limits
            };
            let mut staged = self.stage_native_html_installation(
                basis,
                parent,
                before,
                source,
                installation::Budgets {
                    native: installation_limits,
                    source: Some(limits),
                },
                cancel,
            )?;
            staged
                .continuation
                .prepare_source_closing(&staged.draft, cancel)?;
            let canonical_storage =
                storage::state(&self.state, || staged.draft.source_step(cancel))
                    .map_err(Error::Target)?;
            let canonical_aux_bytes = self
                .source_retained_stack_bytes()
                .map_err(Error::Target)?
                .checked_add(
                    self.source_dispatch_custody_bytes(|| staged.draft.source_step(cancel))
                        .map_err(Error::Target)?,
                )
                .ok_or(DomError::RepresentationExhausted)
                .map_err(Error::Target)?;
            let entry_scope = EntryScope::capture(self, Some(parent), &staged.draft, cancel)
                .map_err(Error::Target)?;
            let mut operation = Box::new(Operation {
                id,
                entry_scope,
                limits: original_limits,
                draft: dialog::OperationDraft::Segment(staged.draft),
                commands: OperationRoot::NativeInstallation,
                index: 0,
                started: false,
                published: false,
                event_exposed: false,
                event_work: std::sync::Weak::new(),
                dialog_queue: None,
                request_close: None,
                request_close_result: None,
                cancelled: false,
                phases: 0,
                headroom: None,
                command_admitted: false,
                outputs: 0,
                identities: 0,
                context: None,
                request: None,
                child: None,
                hosted_global: None,
                receipt: None,
                owner_binding: None,
                historical: Vec::new(),
                select: None,
                installation: Some(staged.continuation),
                canonical_storage,
                canonical_history_bytes: self.history_bytes,
                command_storage: storage::Payload::default(),
                custody_bytes: self.source_owner_custody_bytes().map_err(Error::Target)?,
                canonical_aux_bytes,
            });
            operation.refresh_admission(cancel).map_err(Error::Target)?;
            Ok::<_, Error>(operation)
        })();
        let operation = match formed {
            Ok(operation) => operation,
            Err(error) => {
                if nested {
                    self.source_operation = self.source_parents.pop();
                }
                return Err(error);
            }
        };
        self.next_source_operation += 1; // Checked before any nested parent transfer.
        self.source_operation = Some(operation);
        Ok(DomSourceLease {
            dom: self,
            id,
            complete: false,
            select_authorization: None,
            task_authorization: None,
        })
    }
}
impl Operation {
    fn installation_output(
        &mut self,
        dom: &Dom,
        cancel: &AtomicBool,
    ) -> Result<Option<NodeId>, DomError> {
        let Some(install) = &mut self.installation else {
            return Ok(None);
        };
        let formation = formation::Formation::new(&self.draft, cancel)?;
        let PublicationMap {
            slots,
            occupants,
            ids,
        } = self
            .draft
            .publication_map(dom, formation.as_ref(), cancel)?;
        let released =
            slots.capacity() * size_of::<Slot>() + occupants.capacity() * size_of::<Option<Key>>();
        drop(slots);
        drop(occupants);
        if let Some(budget) = &formation {
            budget.release_bytes(released);
        }
        debug_assert!(install.bindings.capacity() >= install.rows.len());
        // Each genuine binding is filled at most once. Source failure keeps
        // both partial progress and the already admitted immutable native bank.
        // No binding ID or native close work is fabricated or replayed.
        for index in install.bindings.len()..install.rows.len() {
            self.draft.source_step(cancel)?;
            let row = &install.rows[index];
            let id = match row.key {
                Key::Existing(slot) => NodeId {
                    owner: self.id.owner,
                    slot,
                    generation: self.draft.generations[slot],
                },
                Key::Created(index) => ids[index].ok_or(DomError::UnavailableCreation)?,
            };
            install.bindings.push(crate::DomNativeHtmlBinding {
                source: row.source.clone(),
                node: id,
            });
            #[cfg(test)]
            native_closing_observation::filled(install.bindings.len(), cancel);
        }
        let root = install
            .bindings
            .first()
            .ok_or(DomError::UnavailableCreation)?
            .node;
        let released = ids.capacity() * size_of::<Option<NodeId>>();
        drop(ids);
        if let Some(budget) = &formation {
            budget.release_bytes(released);
        }
        // Leave output and charge in their original owner even when a later
        // canonical publication refuses. Transfer follows successful commit.
        Ok(Some(root))
    }
}

#[cfg(test)]
mod native_closing_observation {
    use super::*;
    thread_local! { static CANCEL_AT: std::cell::Cell<Option<usize>> = const { std::cell::Cell::new(None) }; }
    pub fn cancel_after(count: usize) {
        CANCEL_AT.set(Some(count));
    }
    pub fn filled(count: usize, cancel: &AtomicBool) {
        if CANCEL_AT.get() == Some(count) {
            CANCEL_AT.set(None);
            cancel.store(true, Ordering::Release);
        }
    }
}

impl Dom {
    /// Execute the source post-event unblock against the target's CURRENT
    /// Document. No sheet equality test is made after legitimate event reentry.
    /// Failure leaves the token available; callers must not dispatch again.
    pub fn finish_style_notification(
        &mut self,
        token: &mut DomStyleNotificationToken,
        cancel: &AtomicBool,
    ) -> Result<DomStyleNotificationCompletion, DomError> {
        self.require_source_idle()?;
        check_cancel(cancel)?;
        if token.operation.owner != self.owner {
            return Err(DomError::ForeignOwner);
        }
        if token.complete {
            return Err(DomError::StyleProtocol);
        }
        let target = token.target();
        if self.state.node(self.owner, target).is_err() {
            token.complete = true;
            return Ok(DomStyleNotificationCompletion::TargetDestroyed);
        }
        let mut draft = Draft::new(self, Arc::new(crate::batch::BatchIdentity), cancel)?;
        let key = draft.resolve(target.into())?;
        draft
            .element(key)?
            .style_state()
            .ok_or(DomError::WrongNodeKind)?;
        // Native Style's parser-created first conjunct is false, independently
        // of media/environment. The HTML cursor's script-blocking removal is
        // therefore inapplicable; unconditional final unblock is still real.
        let document = draft.node(key)?.document;
        draft
            .node_mut(document)
            .base_document
            .as_mut()
            .ok_or(DomError::WrongNodeKind)?
            .render_blocking
            .retain(|k| *k != key);
        let prepared = draft.finish(self, cancel)?;
        let commit = self.publish(prepared, cancel)?;
        token.complete = true;
        Ok(DomStyleNotificationCompletion::Complete(commit))
    }
}

impl DomStyleSource {
    fn sheet_title(&self) -> &[u16] {
        if self.root_kind == DomStyleRootKind::Document {
            self.title.as_ref().map_or(&[], DocumentString::code_units)
        } else {
            &[]
        }
    }
    fn matches_media(
        &self,
        snapshot: &CssomSnapshot,
        media: &surgeist_cssom::CssomMediaListId,
    ) -> bool {
        // The actual producer retained the full original UTF16 parse input.
        // Grammar equivalence or serialization cannot establish this source join.
        snapshot
            .media(media)
            .ok()
            .and_then(|value| value.parse_source())
            .and_then(|source| source.source().original_utf16())
            .is_some_and(|input| {
                input.as_slice()
                    == self
                        .media
                        .as_ref()
                        .map_or(&[][..], DocumentString::code_units)
            })
    }
}

#[cfg(test)]
pub(in crate::store) mod accounting_tests {
    use super::*;
    fn limits(work: u64) -> DomSourceLimits {
        DomSourceLimits::new(16_000_000, 1_000_000, work, 64, 4096, 4096, 1)
    }
    fn xml_command(work_limit: u64) -> (Result<(), DomError>, u64) {
        let cancel = AtomicBool::new(false);
        let mut dom = Dom::new(DomLimits::new(32, 100_000, 64, 0, 0, 8, 100_000).unwrap()).unwrap();
        let mut setup = DomBatch::new();
        let associated = setup.create_document();
        let associated = dom
            .apply(setup, &cancel)
            .unwrap()
            .document(&associated)
            .unwrap();
        let before = dom.snapshot();
        let mut commands = DomBatch::new();
        let created =
            commands.create_xml_document(associated, crate::Namespace::svg(), "Root", None);
        let mut lease = match dom.begin_source_commands(commands, limits(work_limit), &cancel) {
            Ok(lease) => lease,
            Err(error) => return (Err(error), 0),
        };
        let work = lease
            .dom
            .source_operation
            .as_ref()
            .unwrap()
            .draft
            .source_work
            .clone();
        let result = (|| {
            let DomSourceProgress::Complete(done) = lease.advance(&cancel)? else {
                panic!("SVG root needs no service")
            };
            let document = done.commit().document(&created).unwrap();
            assert_eq!(
                lease
                    .snapshot()
                    .document(document)
                    .unwrap()
                    .metadata()
                    .content_type(),
                &DocumentString::from("image/svg+xml")
            );
            Ok(())
        })();
        if result.is_err() {
            assert_eq!(lease.snapshot().stamp(), before.stamp());
        }
        drop(lease);
        (result, work.load(Ordering::Relaxed))
    }
    #[test]
    fn xml_creator_source_work_is_cumulative_and_exact_budget_finishes_the_same_command() {
        let (result, spent) = xml_command(100_000_000);
        result.unwrap();
        assert!(spent > 1);
        let (exact, actual) = xml_command(spent);
        exact.unwrap();
        assert_eq!(actual, spent);
        let (short, actual) = xml_command(spent - 1);
        assert_eq!(
            short.unwrap_err(),
            DomError::SourceWorkLimit { limit: spent - 1 }
        );
        assert_eq!(actual, spent - 1);
    }
    #[test]
    fn xml_style_root_reserves_final_metadata_and_ticket_revision_before_first_exposure() {
        for (remaining, success) in [(6, false), (7, true)] {
            let cancel = AtomicBool::new(false);
            let mut dom =
                Dom::new(DomLimits::new(32, 100_000, 64, 0, 0, 8, 100_000).unwrap()).unwrap();
            let mut setup = DomBatch::new();
            let associated = setup.create_document();
            let doctype = setup.create_document_type(&associated, "source", "", "");
            setup.insert_before(&doctype, &associated, None);
            let created = dom.apply(setup, &cancel).unwrap();
            let associated = created.document(&associated).unwrap();
            let doctype = created.created(&doctype).unwrap().unwrap();
            dom.revision = DomRevision(u64::MAX - remaining);
            let before = dom.snapshot();
            let mut commands = DomBatch::new();
            commands.create_xml_document(
                associated,
                crate::Namespace::html(),
                "style",
                Some(doctype.into()),
            );
            let mut lease = dom
                .begin_source_commands(commands, limits(100_000_000), &cancel)
                .unwrap();
            let result = lease.advance(&cancel);
            if success {
                let DomSourceProgress::Request(DomSourceRequest::Style(request)) = result.unwrap()
                else {
                    panic!("actual inline policy")
                };
                assert_eq!(
                    request.action(),
                    DomStyleAction::Update(surgeist_html::HtmlStyleUpdateAction::CheckInlineCsp)
                );
                let metadata = lease.snapshot();
                assert_eq!(
                    metadata
                        .document(request.source().document())
                        .unwrap()
                        .metadata()
                        .content_type(),
                    &DocumentString::from("application/xml")
                );
            } else {
                assert_eq!(result.unwrap_err(), DomError::RevisionExhausted);
                assert_eq!(lease.snapshot().stamp(), before.stamp());
                assert_eq!(
                    lease.snapshot().node(doctype).unwrap().parent(),
                    Some(associated.node())
                );
                assert!(!lease.status().has_published());
            }
        }
    }
    fn fixture() -> (Dom, DomBatch) {
        let cancel = AtomicBool::new(false);
        let mut dom = Dom::new(DomLimits::new(64, 100_000, 64, 0, 0, 8, 100_000).unwrap()).unwrap();
        let mut setup = DomBatch::new();
        let document = setup.create_document();
        let container = setup.create_element_ns(&document, crate::Namespace::html(), "div");
        let text = setup.create_text(&document, "actual retained canonical payload".repeat(32));
        setup.insert_before(&text, &container, None);
        setup.insert_before(&container, &document, None);
        let commit = dom.apply(setup, &cancel).unwrap();
        let document = commit.document(&document).unwrap();
        let mut batch = DomBatch::new();
        batch.create_text(document, "unstarted creation");
        (dom, batch)
    }
    #[test]
    fn source_native_rows_keep_custody_through_abort_parking_cancellation_and_completion() {
        use surgeist_html::{HtmlConstructionLimits, HtmlFragment};
        let cancel = AtomicBool::new(false);
        let mut dom = Dom::new(DomLimits::new(32, 100_000, 64, 0, 0, 8, 100_000).unwrap()).unwrap();
        let mut setup = DomBatch::new();
        let document = setup.create_document();
        let parent = setup.create_element_ns(&document, crate::Namespace::html(), "div");
        setup.insert_before(&parent, &document, None);
        let created = dom.apply(setup, &cancel).unwrap();
        let document = created.document(&document).unwrap();
        let parent = created.created(&parent).unwrap().unwrap();
        let mut builder =
            HtmlFragment::builder(HtmlConstructionLimits::new(8, 8, 8, 256, 8)).unwrap();
        let style = builder
            .append_element(
                &builder.root(),
                crate::QualifiedName::new(crate::Namespace::html(), None, "style"),
                crate::OrderedAttributes::default(),
            )
            .unwrap();
        builder.append_text(&style, "a {}").unwrap();
        let source = builder.finish();
        let budgets = crate::DomSourceNativeHtmlInstallLimits::new(
            crate::DomNativeHtmlInstallLimits::new(8, 1_000_000, 1_000_000),
            limits(10_000_000),
        );
        let before = dom.snapshot();
        let lease = dom
            .begin_native_installation(
                crate::DomNativeHtmlInstallTarget::new(
                    before.document(document).unwrap().stamp(),
                    parent,
                    None,
                ),
                &source,
                budgets,
                &cancel,
            )
            .unwrap();
        let op = lease.dom.source_operation.as_ref().unwrap();
        let live = op.draft.source_formation_bytes.clone();
        let bytes = op.installation.as_ref().unwrap().rows.capacity()
            * size_of::<installation::Row>()
            + op.installation.as_ref().unwrap().bindings.capacity()
                * size_of::<crate::DomNativeHtmlBinding>();
        assert_eq!(live.load(Ordering::Relaxed), bytes);
        drop(lease); // An unexposed installation aborts and drops its actual bank.
        assert_eq!(live.load(Ordering::Relaxed), 0);
        assert_eq!(dom.snapshot().stamp(), before.stamp());
        assert!(dom.pending_source_edit().is_none());

        let mut lease = dom
            .begin_native_installation(
                crate::DomNativeHtmlInstallTarget::new(
                    before.document(document).unwrap().stamp(),
                    parent,
                    None,
                ),
                &source,
                budgets,
                &cancel,
            )
            .unwrap();
        let op = lease.dom.source_operation.as_ref().unwrap();
        let live = op.draft.source_formation_bytes.clone();
        let bytes = op.installation.as_ref().unwrap().rows.capacity()
            * size_of::<installation::Row>()
            + op.installation.as_ref().unwrap().bindings.capacity()
                * size_of::<crate::DomNativeHtmlBinding>();
        let request = match lease.advance(&cancel).unwrap() {
            DomSourceProgress::Request(DomSourceRequest::Style(request)) => request,
            other => panic!("real connected native Style CSP boundary, got {other:?}"),
        };
        assert_eq!(
            request.action(),
            DomStyleAction::Update(Action::CheckInlineCsp)
        );
        assert!(lease.status().has_published());
        let id = lease.operation();
        let published = lease.snapshot().stamp();
        drop(lease);
        assert_eq!(live.load(Ordering::Relaxed), bytes);
        cancel.store(true, Ordering::Release);
        assert!(matches!(
            dom.resume_source_edit(id, budgets.source(), &cancel),
            Err(DomError::Cancelled)
        ));
        assert_eq!(dom.snapshot().stamp(), published);
        assert_eq!(live.load(Ordering::Relaxed), bytes);
        cancel.store(false, Ordering::Release);
        let mut lease = dom
            .resume_source_edit(id, budgets.source(), &cancel)
            .unwrap();
        let resumed = match lease.advance(&cancel).unwrap() {
            DomSourceProgress::Request(DomSourceRequest::Style(request)) => request,
            other => panic!("same parked CSP request, got {other:?}"),
        };
        assert!(Arc::ptr_eq(&request.source, &resumed.source));
        lease.request_cancel();
        lease
            .accept_inline_csp(
                &resumed,
                &crate::DomStyleInlineCspObservation::evaluated(
                    resumed.source(),
                    Arc::new(Vec::new()),
                    1,
                    false,
                ),
                &cancel,
            )
            .unwrap();
        let result = match lease.advance(&cancel).unwrap() {
            DomSourceProgress::Complete(result) => result,
            other => panic!("denied native Style closes actual installation, got {other:?}"),
        };
        assert_eq!(result.native_installation().unwrap().bindings().len(), 3);
        assert!(result.cancellation_requested());
        drop(lease);
        assert_eq!(live.load(Ordering::Relaxed), 0);
        assert!(dom.pending_source_edit().is_none());
        assert_eq!(before.node(parent).unwrap().children().len(), 0);
        assert_eq!(dom.snapshot().node(parent).unwrap().children().len(), 1);
    }
    fn native_closing_budget(scratch: bool) {
        use crate::{
            DomNativeHtmlInstallError as InstallError, DomNativeHtmlInstallResource as Resource,
        };
        use surgeist_html::{HtmlConstructionLimits, HtmlFragment};
        let cancel = AtomicBool::new(false);
        let mut dom = Dom::new(DomLimits::new(32, 100_000, 64, 0, 0, 8, 100_000).unwrap()).unwrap();
        let mut setup = DomBatch::new();
        let document = setup.create_document();
        let parent = setup.create_element_ns(&document, crate::Namespace::html(), "div");
        setup.insert_before(&parent, &document, None);
        let created = dom.apply(setup, &cancel).unwrap();
        let document = created.document(&document).unwrap();
        let parent = created.created(&parent).unwrap().unwrap();
        let mut builder =
            HtmlFragment::builder(HtmlConstructionLimits::new(8, 8, 8, 256, 8)).unwrap();
        let style = builder
            .append_element(
                &builder.root(),
                crate::QualifiedName::new(crate::Namespace::html(), None, "style"),
                crate::OrderedAttributes::default(),
            )
            .unwrap();
        builder.append_text(&style, "a {}").unwrap();
        let source = builder.finish();
        let before = dom.snapshot();
        // Measure the unchanged native staging owner before closing capacity
        // preparation. This selects the same original immutable low allowance
        // after closing admission moves ahead of operation issuance.
        let probe = dom
            .stage_native_html_installation(
                before.document(document).unwrap().stamp(),
                parent,
                None,
                &source,
                installation::Budgets {
                    native: crate::DomNativeHtmlInstallLimits::new(8, 1_000_000, 1_000_000),
                    source: Some(limits(100_000_000)),
                },
                &cancel,
            )
            .unwrap();
        let installation = &probe.continuation;
        let work = installation.work;
        let closing_bytes =
            installation.bytes + installation.rows.len() * size_of::<crate::DomNativeHtmlBinding>();
        assert_eq!(installation.rows.len(), 3);
        drop(probe);
        let selected = if scratch {
            Resource::ScratchBytes
        } else {
            Resource::Work
        };
        let cap = if scratch { closing_bytes - 1 } else { work };
        let budgets = crate::DomSourceNativeHtmlInstallLimits::new(
            crate::DomNativeHtmlInstallLimits::new(
                8,
                if scratch { 1_000_000 } else { cap },
                if scratch { cap } else { 1_000_000 },
            ),
            limits(100_000_000),
        );
        let mut lease = match dom.begin_native_installation(
            crate::DomNativeHtmlInstallTarget::new(
                before.document(document).unwrap().stamp(),
                parent,
                None,
            ),
            &source,
            budgets,
            &cancel,
        ) {
            Ok(lease) => lease,
            Err(InstallError::ResourceLimit { resource, limit })
                if resource == selected && limit == cap =>
            {
                return;
            }
            Err(error) => panic!(
                "expected admitted staging or actual immutable closing-budget refusal: {error:?}"
            ),
        };
        let policy = match lease.advance(&cancel) {
            Ok(DomSourceProgress::Request(DomSourceRequest::Style(request))) => request,
            Err(DomError::SourceWorkLimit { limit }) if !scratch && limit == cap as u64 => {
                assert!(!lease.status().has_published());
                assert_eq!(lease.snapshot().stamp(), before.stamp());
                return;
            }
            Err(DomError::ResourceLimit {
                resource: DomResource::SourceRetainedBytes,
                limit,
            }) if scratch && limit == cap => {
                assert!(!lease.status().has_published());
                assert_eq!(lease.snapshot().stamp(), before.stamp());
                return;
            }
            other => panic!(
                "expected actual Style CSP or pre-publication immutable budget refusal: {other:?}"
            ),
        };
        assert_eq!(
            policy.action(),
            DomStyleAction::Update(Action::CheckInlineCsp)
        );
        lease
            .accept_inline_csp(
                &policy,
                &DomStyleInlineCspObservation::complete_empty(policy.source(), 1),
                &cancel,
            )
            .unwrap();
        let DomSourceProgress::Request(DomSourceRequest::Style(create)) =
            lease.advance(&cancel).unwrap()
        else {
            panic!("actual native Style creation")
        };
        assert_eq!(create.action(), DomStyleAction::Update(Action::CreateSheet));
        let root = commit_actual_empty_sheet(&mut lease, &create, &cancel);
        assert_eq!(root.snapshot().unwrap().sheets().len(), 1);
        let error = lease.advance(&cancel).unwrap_err();
        if scratch {
            assert_eq!(
                error,
                DomError::ResourceLimit {
                    resource: DomResource::SourceRetainedBytes,
                    limit: cap
                }
            );
        } else {
            assert_eq!(error, DomError::SourceWorkLimit { limit: cap as u64 });
        }
        assert!(
            !lease.status().has_published(),
            "immutable native {selected:?} closing budget stranded a mandatory tail after genuine CSSOM creation"
        );
    }
    #[test]
    fn source_native_closing_work_is_admitted_before_irreversible_cssom_creation() {
        native_closing_budget(false);
    }
    #[test]
    fn source_native_closing_scratch_is_admitted_before_irreversible_cssom_creation() {
        native_closing_budget(true);
    }
    #[test]
    fn source_native_partial_closing_bank_survives_cancel_park_and_retry_without_native_recharge() {
        use surgeist_html::{HtmlConstructionLimits, HtmlFragment};
        let cancel = AtomicBool::new(false);
        let mut dom = Dom::new(DomLimits::new(32, 100_000, 64, 0, 0, 8, 100_000).unwrap()).unwrap();
        let mut setup = DomBatch::new();
        let document = setup.create_document();
        let parent = setup.create_element_ns(&document, crate::Namespace::html(), "div");
        setup.insert_before(&parent, &document, None);
        let created = dom.apply(setup, &cancel).unwrap();
        let document = created.document(&document).unwrap();
        let parent = created.created(&parent).unwrap().unwrap();
        let mut builder =
            HtmlFragment::builder(HtmlConstructionLimits::new(8, 8, 8, 256, 8)).unwrap();
        let style = builder
            .append_element(
                &builder.root(),
                crate::QualifiedName::new(crate::Namespace::html(), None, "style"),
                crate::OrderedAttributes::default(),
            )
            .unwrap();
        builder.append_text(&style, "a {}").unwrap();
        let source = builder.finish();
        let bounds = crate::DomSourceNativeHtmlInstallLimits::new(
            crate::DomNativeHtmlInstallLimits::new(8, 1_000_000, 1_000_000),
            limits(100_000_000),
        );
        let before = dom.snapshot();
        let mut lease = dom
            .begin_native_installation(
                crate::DomNativeHtmlInstallTarget::new(
                    before.document(document).unwrap().stamp(),
                    parent,
                    None,
                ),
                &source,
                bounds,
                &cancel,
            )
            .unwrap();
        let operation = lease.operation();
        let installation = lease
            .dom
            .source_operation
            .as_ref()
            .unwrap()
            .installation
            .as_ref()
            .unwrap();
        let bank = installation.bindings.as_ptr();
        let capacity = installation.bindings.capacity();
        let native_work = installation.work;
        assert!(capacity >= 3);
        let live = lease
            .dom
            .source_operation
            .as_ref()
            .unwrap()
            .draft
            .source_formation_bytes
            .clone();
        let DomSourceProgress::Request(DomSourceRequest::Style(policy)) =
            lease.advance(&cancel).unwrap()
        else {
            panic!("actual CSP")
        };
        lease
            .accept_inline_csp(
                &policy,
                &DomStyleInlineCspObservation::complete_empty(policy.source(), 1),
                &cancel,
            )
            .unwrap();
        let DomSourceProgress::Request(DomSourceRequest::Style(create)) =
            lease.advance(&cancel).unwrap()
        else {
            panic!("actual sheet creation")
        };
        let root = commit_actual_empty_sheet(&mut lease, &create, &cancel);
        native_closing_observation::cancel_after(1);
        assert_eq!(lease.advance(&cancel).unwrap_err(), DomError::Cancelled);
        assert!(lease.status().has_published());
        let op = lease.dom.source_operation.as_ref().unwrap();
        let input = op.installation.as_ref().unwrap();
        assert_eq!(input.bindings.len(), 1);
        assert_eq!(input.bindings.as_ptr(), bank);
        assert_eq!(input.bindings.capacity(), capacity);
        assert_eq!(input.work, native_work);
        let first = input.bindings[0].node();
        let spent = op.draft.source_work.load(Ordering::Relaxed);
        drop(lease);
        assert!(
            live.load(Ordering::Relaxed) >= capacity * size_of::<crate::DomNativeHtmlBinding>()
        );
        assert!(matches!(
            dom.resume_source_edit(operation, bounds.source(), &cancel),
            Err(DomError::Cancelled)
        ));
        cancel.store(false, Ordering::Release);
        let mut lease = dom
            .resume_source_edit(operation, bounds.source(), &cancel)
            .unwrap();
        assert!(
            lease
                .dom
                .source_operation
                .as_ref()
                .unwrap()
                .draft
                .source_work
                .load(Ordering::Relaxed)
                >= spent
        );
        let DomSourceProgress::Complete(result) = lease.advance(&cancel).unwrap() else {
            panic!("same partial bank closes without CSSOM replay")
        };
        let installation = result.native_installation().unwrap();
        assert_eq!(installation.bindings().len(), 3);
        assert_eq!(installation.bindings().as_ptr(), bank);
        assert_eq!(installation.bindings()[0].node(), first);
        for binding in installation.bindings() {
            assert!(installation.snapshot().node(binding.node()).is_ok());
        }
        assert_eq!(root.snapshot().unwrap().sheets().len(), 1);
        drop(lease);
        assert_eq!(live.load(Ordering::Relaxed), 0);
        assert!(dom.pending_source_edit().is_none());
    }
    fn actual_input_alias(bytes: usize) -> Option<formation::mutation_observation::Growth> {
        let cancel = AtomicBool::new(false);
        let mut dom = Dom::new(DomLimits::new(16, 100_000, 32, 0, 0, 8, 100_000).unwrap()).unwrap();
        let mut setup = DomBatch::new();
        let document = setup.create_document();
        let input = setup.create_element_ns(&document, crate::Namespace::html(), "input");
        let created = dom.apply(setup, &cancel).unwrap();
        let input = created.created(&input).unwrap().unwrap();
        let before = dom.snapshot();
        let mut raw = Vec::with_capacity(65_536);
        raw.push(u16::from(b'a'));
        let mut batch = DomBatch::new();
        batch.set_input_value(input, DocumentString::from_utf16(raw));
        let bounds = DomSourceLimits::new(bytes, 1_000_000, 100_000_000, 64, 4096, 4096, 1);
        let Ok(mut lease) = dom.begin_source_commands(batch, bounds, &cancel) else {
            return None;
        };
        formation::mutation_observation::start(formation::mutation_observation::Site::InputAlias);
        let result = lease.advance(&cancel);
        let actual = formation::mutation_observation::finish();
        if result.is_err() {
            assert_eq!(lease.snapshot().stamp(), before.stamp());
        }
        actual
    }
    #[test]
    fn source_input_raw_alias_admits_actual_spare_backing_before_clone() {
        let high = actual_input_alias(16_000_000).expect("actual padded current Input alias");
        if let Some(actual) = actual_input_alias(high.required - 1) {
            assert!(
                actual.required <= actual.limit,
                "Input raw alias retained {} bytes under {}",
                actual.required,
                actual.limit
            );
        }
    }
    fn actual_character_data_creation(
        bytes: usize,
    ) -> Option<formation::mutation_observation::Growth> {
        let cancel = AtomicBool::new(false);
        let mut dom =
            Dom::new(DomLimits::new(16, 1_000_000, 32, 0, 0, 8, 1_000_000).unwrap()).unwrap();
        let mut setup = DomBatch::new();
        let document = setup.create_document();
        let text = setup.create_text(&document, "retained".repeat(10_000));
        let created = dom.apply(setup, &cancel).unwrap();
        let text = created.created(&text).unwrap().unwrap();
        let before = dom.snapshot();
        let mut batch = DomBatch::new();
        batch.replace_data(text, 1, 0, "new");
        let limit = DomSourceLimits::new(bytes, 10_000_000, 100_000_000, 64, 4096, 4096, 1);
        let Ok(mut lease) = dom.begin_source_commands(batch, limit, &cancel) else {
            return None;
        };
        formation::mutation_observation::start(formation::mutation_observation::Site::ReplaceData);
        let result = lease.advance(&cancel);
        let actual = formation::mutation_observation::finish();
        if result.is_err() {
            assert_eq!(lease.snapshot().stamp(), before.stamp());
        }
        actual
    }
    #[test]
    fn source_character_data_admits_actual_replacement_payload_before_materialization() {
        let high =
            actual_character_data_creation(16_000_000).expect("actual CharacterData replacement");
        if let Some(actual) = actual_character_data_creation(high.required - 1) {
            assert!(
                actual.required <= actual.limit,
                "CharacterData output retained {} bytes under {}",
                actual.required,
                actual.limit
            );
        }
    }
    fn actual_control_string_creation(
        bytes: usize,
        validity: bool,
    ) -> Option<formation::mutation_observation::Growth> {
        let cancel = AtomicBool::new(false);
        let mut dom =
            Dom::new(DomLimits::new(16, 1_000_000, 32, 0, 0, 8, 1_000_000).unwrap()).unwrap();
        let mut setup = DomBatch::new();
        let document = setup.create_document();
        let input = setup.create_element_ns(&document, crate::Namespace::html(), "input");
        let created = dom.apply(setup, &cancel).unwrap();
        let input = created.created(&input).unwrap().unwrap();
        let before = dom.snapshot();
        let mut batch = DomBatch::new();
        if validity {
            batch.set_custom_validity(input, "a\r\nb".repeat(20_000));
        } else {
            batch.set_input_value(input, "a\r\nb".repeat(20_000));
        }
        let limit = DomSourceLimits::new(bytes, 10_000_000, 100_000_000, 64, 4096, 4096, 1);
        let Ok(mut lease) = dom.begin_source_commands(batch, limit, &cancel) else {
            return None;
        };
        formation::mutation_observation::start(if validity {
            formation::mutation_observation::Site::CustomValidity
        } else {
            formation::mutation_observation::Site::InputString
        });
        let result = lease.advance(&cancel);
        let actual = formation::mutation_observation::finish();
        if result.is_err() {
            assert_eq!(lease.snapshot().stamp(), before.stamp());
        }
        actual
    }
    #[test]
    fn source_input_string_admits_actual_sanitized_payload_before_materialization() {
        let high =
            actual_control_string_creation(16_000_000, false).expect("actual Input String output");
        if let Some(actual) = actual_control_string_creation(high.required - 1, false) {
            assert!(
                actual.required <= actual.limit,
                "Input String output retained {} bytes under {}",
                actual.required,
                actual.limit
            );
        }
    }
    #[test]
    fn source_custom_validity_admits_actual_normalized_payload_before_materialization() {
        let high = actual_control_string_creation(16_000_000, true)
            .expect("actual custom validity output");
        if let Some(actual) = actual_control_string_creation(high.required - 1, true) {
            assert!(
                actual.required <= actual.limit,
                "custom validity output retained {} bytes under {}",
                actual.required,
                actual.limit
            );
        }
    }
    fn actual_children_growth(bytes: usize) -> Option<formation::mutation_observation::Growth> {
        let cancel = AtomicBool::new(false);
        let mut dom =
            Dom::new(DomLimits::new(64, 100_000, 128, 0, 0, 8, 100_000).unwrap()).unwrap();
        let mut setup = DomBatch::new();
        let document = setup.create_document();
        let parent = setup.create_element_ns(&document, crate::Namespace::html(), "div");
        let old = setup.create_text(&document, "old child");
        setup.insert_before(&old, &parent, None);
        setup.insert_before(&parent, &document, None);
        let fragment = setup.create_fragment(&document);
        for _ in 0..12 {
            let text = setup.create_text(&document, "new child");
            setup.insert_before(&text, &fragment, None);
        }
        let created = dom.apply(setup, &cancel).unwrap();
        let parent = created.created(&parent).unwrap().unwrap();
        let fragment = created.created(&fragment).unwrap().unwrap();
        let before = dom.snapshot();
        let mut batch = DomBatch::new();
        batch.insert_before(fragment, parent, None);
        let limit = DomSourceLimits::new(bytes, 1_000_000, 10_000_000, 64, 4096, 4096, 1);
        let Ok(mut lease) = dom.begin_source_commands(batch, limit, &cancel) else {
            return None;
        };
        formation::mutation_observation::start(formation::mutation_observation::Site::Children);
        let result = lease.advance(&cancel);
        let growth = formation::mutation_observation::finish();
        if result.is_err() {
            assert_eq!(lease.snapshot().stamp(), before.stamp());
        }
        drop(lease);
        assert!(dom.pending_source_edit().is_none());
        growth
    }
    fn actual_select_task_growth(bytes: usize) -> Option<formation::mutation_observation::Growth> {
        let cancel = AtomicBool::new(false);
        let domain = DomLimits::new(64, 100_000, 128, 0, 0, 8, 100_000)
            .unwrap()
            .with_select_processing(256, 1_000_000, 1_000_000)
            .unwrap();
        let mut dom = Dom::new(domain).unwrap();
        let mut setup = DomBatch::new();
        let document = setup.create_document();
        let select = setup.create_element_ns(&document, crate::Namespace::html(), "select");
        let option = setup.create_element_ns(&document, crate::Namespace::html(), "option");
        setup.set_attribute(&option, "selected", "");
        setup.insert_before(&option, &select, None);
        let created = dom.apply(setup, &cancel).unwrap();
        let option = created.created(&option).unwrap().unwrap();
        let select = created.created(&select).unwrap().unwrap();
        // Real queued updates remain in the canonical owner. This makes the
        // next replacement bank, rather than an earlier traversal, the decisive
        // actual storage interval; no fabricated private task rows.
        for _ in 0..64 {
            let mut prior = DomBatch::new();
            prior.detach(option);
            prior.insert_before(option, select, None);
            dom.apply(prior, &cancel).unwrap();
        }
        let before = dom.snapshot();
        let mut batch = DomBatch::new();
        batch.detach(option);
        let limit = DomSourceLimits::new(bytes, 1_000_000, 10_000_000, 64, 4096, 4096, 1);
        let Ok(mut lease) = dom.begin_source_commands(batch, limit, &cancel) else {
            return None;
        };
        formation::mutation_observation::start(formation::mutation_observation::Site::SelectTasks);
        let result = lease.advance(&cancel);
        let growth = formation::mutation_observation::finish();
        if result.is_err() {
            assert_eq!(lease.snapshot().stamp(), before.stamp());
        }
        drop(lease);
        assert!(dom.pending_source_edit().is_none());
        growth
    }
    fn actual_metadata_creation(bytes: usize) -> Option<formation::mutation_observation::Growth> {
        let cancel = AtomicBool::new(false);
        let mut dom =
            Dom::new(DomLimits::new(64, 100_000, 128, 0, 0, 8, 100_000).unwrap()).unwrap();
        let before = dom.snapshot();
        let mut batch = DomBatch::new();
        batch.create_document();
        let limit = DomSourceLimits::new(bytes, 1_000_000, 10_000_000, 64, 4096, 4096, 1);
        let Ok(mut lease) = dom.begin_source_commands(batch, limit, &cancel) else {
            return None;
        };
        formation::mutation_observation::start(formation::mutation_observation::Site::Metadata);
        let result = lease.advance(&cancel);
        let growth = formation::mutation_observation::finish();
        if result.is_err() {
            assert_eq!(lease.snapshot().stamp(), before.stamp());
        }
        drop(lease);
        assert!(dom.pending_source_edit().is_none());
        growth
    }
    fn actual_shared_input_family_copy(
        bytes: usize,
    ) -> Option<formation::mutation_observation::Growth> {
        let cancel = AtomicBool::new(false);
        let mut dom = Dom::new(DomLimits::new(16, 100_000, 64, 0, 0, 8, 100_000).unwrap()).unwrap();
        let mut setup = DomBatch::new();
        let document = setup.create_document();
        let input = setup.create_element_ns(&document, crate::Namespace::html(), "input");
        let created = dom.apply(setup, &cancel).unwrap();
        let input = created.created(&input).unwrap().unwrap();
        let before = dom.snapshot();
        let mut batch = DomBatch::new();
        batch.set_input_checked(input, true);
        let limits = DomSourceLimits::new(bytes, 1_000_000, 10_000_000, 64, 4096, 4096, 1);
        let Ok(mut lease) = dom.begin_source_commands(batch, limits, &cancel) else {
            return None;
        };
        formation::mutation_observation::start(formation::mutation_observation::Site::Family);
        let result = lease.advance(&cancel);
        let growth = formation::mutation_observation::finish();
        if result.is_err() {
            assert_eq!(lease.snapshot().stamp(), before.stamp());
        }
        growth
    }
    #[test]
    fn source_shared_input_family_admits_actual_copy_before_mutation() {
        let high =
            actual_shared_input_family_copy(16_000_000).expect("real shared Input family copy");
        assert!(
            actual_shared_input_family_copy(high.required).is_some(),
            "exact copy boundary must be reachable"
        );
        if let Some(actual) = actual_shared_input_family_copy(high.required - 1) {
            assert!(
                actual.required <= actual.limit,
                "shared Input family allocated {} bytes under {}",
                actual.required,
                actual.limit
            );
        }
    }
    fn actual_output_text_read(bytes: usize) -> Option<formation::mutation_observation::Growth> {
        let cancel = AtomicBool::new(false);
        let mut dom = Dom::new(DomLimits::new(32, 100_000, 64, 0, 0, 8, 100_000).unwrap()).unwrap();
        let mut setup = DomBatch::new();
        let document = setup.create_document();
        let output = setup.create_element_ns(&document, crate::Namespace::html(), "output");
        let text = setup.create_text(&document, "actual old output text".repeat(1024));
        setup.insert_before(&text, &output, None);
        let created = dom.apply(setup, &cancel).unwrap();
        let output = created.created(&output).unwrap().unwrap();
        let before = dom.snapshot();
        let mut batch = DomBatch::new();
        batch.set_output_value(output, "new");
        let limits = DomSourceLimits::new(bytes, 1_000_000, 10_000_000, 64, 4096, 4096, 1);
        let Ok(mut lease) = dom.begin_source_commands(batch, limits, &cancel) else {
            return None;
        };
        formation::mutation_observation::start(formation::mutation_observation::Site::Output);
        let result = lease.advance(&cancel);
        let growth = formation::mutation_observation::finish();
        if result.is_err() {
            assert_eq!(lease.snapshot().stamp(), before.stamp());
        }
        growth
    }
    #[test]
    fn source_output_text_admits_actual_materialization_before_retaining_read_frame() {
        let high = actual_output_text_read(16_000_000).expect("actual Output text materialization");
        assert!(
            actual_output_text_read(high.required).is_some(),
            "exact materialization boundary must be reachable"
        );
        if let Some(actual) = actual_output_text_read(high.required - 1) {
            assert!(
                actual.required <= actual.limit,
                "Output text materialized {} bytes under {}",
                actual.required,
                actual.limit
            );
        }
    }
    #[test]
    fn output_frame_payload_stays_admitted_through_checkpoint_parking_and_retry() {
        use surgeist_cssom::CssomEditGuard;
        let cancel = AtomicBool::new(false);
        let mut dom =
            Dom::new(DomLimits::new(64, 100_000, 128, 0, 0, 8, 100_000).unwrap()).unwrap();
        let mut setup = DomBatch::new();
        let document = setup.create_document();
        let output = setup.create_element_ns(&document, crate::Namespace::html(), "output");
        let style = setup.create_element_ns(&document, crate::Namespace::html(), "style");
        setup.insert_before(&style, &output, None);
        let old_text = "actual retained Output read".repeat(512);
        let text = setup.create_text(&document, old_text.clone());
        setup.insert_before(&text, &output, None);
        let created = dom.apply(setup, &cancel).unwrap();
        let document = created.document(&document).unwrap();
        let output = created.created(&output).unwrap().unwrap();
        let root = {
            let mut connect = DomBatch::new();
            connect.insert_before(output, document.node(), None);
            let mut lease = dom
                .begin_source_commands(connect, limits(10_000_000), &cancel)
                .unwrap();
            let DomSourceProgress::Request(DomSourceRequest::Style(policy)) =
                lease.advance(&cancel).unwrap()
            else {
                panic!("actual CSP boundary")
            };
            lease
                .accept_inline_csp(
                    &policy,
                    &DomStyleInlineCspObservation::complete_empty(policy.source(), 1),
                    &cancel,
                )
                .unwrap();
            let DomSourceProgress::Request(DomSourceRequest::Style(create)) =
                lease.advance(&cancel).unwrap()
            else {
                panic!("actual sheet creation")
            };
            let root = commit_actual_empty_sheet(&mut lease, &create, &cancel);
            assert!(matches!(
                lease.advance(&cancel).unwrap(),
                DomSourceProgress::Complete(_)
            ));
            root
        };
        let mut batch = DomBatch::new();
        batch.set_output_value(output, "new");
        let mut lease = dom
            .begin_source_commands(batch, limits(10_000_000), &cancel)
            .unwrap();
        let id = lease.operation();
        let DomSourceProgress::Request(DomSourceRequest::Style(remove)) =
            lease.advance(&cancel).unwrap()
        else {
            panic!("nested actual sheet removal")
        };
        assert_eq!(
            remove.action(),
            DomStyleAction::Update(surgeist_html::HtmlStyleUpdateAction::RemoveAssociatedSheet)
        );
        let parked_stamp = lease.snapshot().stamp();
        assert_eq!(
            lease
                .snapshot()
                .node(output)
                .unwrap()
                .as_element()
                .unwrap()
                .output_state()
                .unwrap()
                .default_value_override()
                .unwrap()
                .to_utf8()
                .unwrap(),
            old_text
        );
        // The two independently known live frame strings are supplied "new"
        // and the flattened old Text. The source input and canonical override
        // have their own graph occurrences; these counters cover the frame.
        let minimum_units = old_text.len() + 3;
        let minimum_bytes = 2 * size_of::<Vec<u16>>() + minimum_units * size_of::<u16>();
        let (live_bytes, live_units, frame_bytes, frame_units) = {
            let draft = &lease.dom.source_operation.as_ref().unwrap().draft;
            let live_bytes = draft.source_formation_bytes.clone();
            let live_units = draft.source_temporary_units.clone();
            let frame_bytes = live_bytes.load(Ordering::Relaxed);
            let frame_units = live_units.load(Ordering::Relaxed);
            assert!(frame_bytes >= minimum_bytes);
            assert!(frame_units >= minimum_units);
            let checkpoint = draft.source_checkpoint(0, 0, &cancel).unwrap();
            assert!(
                live_bytes.load(Ordering::Relaxed) >= frame_bytes + minimum_bytes,
                "checkpoint must admit its distinct retained frame occurrence"
            );
            drop(checkpoint);
            assert_eq!(live_bytes.load(Ordering::Relaxed), frame_bytes);
            assert_eq!(live_units.load(Ordering::Relaxed), frame_units);
            (live_bytes, live_units, frame_bytes, frame_units)
        };
        drop(lease);
        assert!(dom.pending_source_edit().is_some());
        assert_eq!(live_bytes.load(Ordering::Relaxed), frame_bytes);
        assert_eq!(live_units.load(Ordering::Relaxed), frame_units);
        let mut lease = dom
            .resume_source_edit(id, limits(10_000_000), &cancel)
            .unwrap();
        cancel.store(true, Ordering::Release);
        assert!(matches!(lease.advance(&cancel), Err(DomError::Cancelled)));
        assert_eq!(lease.snapshot().stamp(), parked_stamp);
        assert_eq!(live_bytes.load(Ordering::Relaxed), frame_bytes);
        assert_eq!(live_units.load(Ordering::Relaxed), frame_units);
        cancel.store(false, Ordering::Release);
        let remove = lease
            .qualify_cssom(&remove, root.root(), &root.snapshot().unwrap())
            .unwrap();
        let mut edit = root
            .batch(CssomEditGuard::from_snapshot(&root.snapshot().unwrap()))
            .unwrap();
        edit.remove_sheet(remove.sheet().unwrap().id()).unwrap();
        let publication = root.commit(&mut Some(edit)).unwrap();
        lease
            .accept_cssom_publication(&remove, publication, root.snapshot().unwrap(), None)
            .unwrap();
        assert!(matches!(
            lease.advance(&cancel).unwrap(),
            DomSourceProgress::Complete(_)
        ));
        drop(lease);
        assert!(dom.pending_source_edit().is_none());
        assert_eq!(live_bytes.load(Ordering::Relaxed), 0);
        assert_eq!(live_units.load(Ordering::Relaxed), 0);
        let snapshot = dom.snapshot();
        let children = snapshot.node(output).unwrap().children();
        assert_eq!(children.len(), 1);
        assert!(
            matches!(snapshot.node(children[0]).unwrap().kind(), DomNodeKind::Text(value) if value.to_utf8().unwrap() == "new")
        );
        assert_eq!(
            snapshot
                .node(output)
                .unwrap()
                .as_element()
                .unwrap()
                .output_state()
                .unwrap()
                .default_value_override()
                .unwrap()
                .to_utf8()
                .unwrap(),
            old_text
        );
        assert!(root.snapshot().unwrap().sheets().is_empty());
    }
    fn actual_textarea_api_read(bytes: usize) -> Option<formation::mutation_observation::Growth> {
        let cancel = AtomicBool::new(false);
        let mut dom = Dom::new(DomLimits::new(32, 100_000, 64, 0, 0, 8, 100_000).unwrap()).unwrap();
        let mut setup = DomBatch::new();
        let document = setup.create_document();
        let textarea = setup.create_element_ns(&document, crate::Namespace::html(), "textarea");
        setup.set_textarea_value(&textarea, "before\r\n".repeat(2048));
        let created = dom.apply(setup, &cancel).unwrap();
        let textarea = created.created(&textarea).unwrap().unwrap();
        let before = dom.snapshot();
        let mut batch = DomBatch::new();
        batch.set_textarea_value(textarea, "after\r\n".repeat(1024));
        let limit = DomSourceLimits::new(bytes, 1_000_000, 10_000_000, 64, 4096, 4096, 1);
        let Ok(mut lease) = dom.begin_source_commands(batch, limit, &cancel) else {
            return None;
        };
        formation::mutation_observation::start(formation::mutation_observation::Site::TextareaApi);
        let result = lease.advance(&cancel);
        let growth = formation::mutation_observation::finish();
        if result.is_err() {
            assert_eq!(lease.snapshot().stamp(), before.stamp());
        }
        growth
    }
    #[test]
    fn source_textarea_api_admits_actual_normalized_buffer_before_materialization() {
        let high = actual_textarea_api_read(16_000_000).expect("actual old API normalized buffer");
        assert!(
            actual_textarea_api_read(high.required).is_some(),
            "exact API materialization boundary must be reachable"
        );
        if let Some(actual) = actual_textarea_api_read(high.required - 1) {
            assert!(
                actual.required <= actual.limit,
                "Textarea API materialized {} bytes under {}",
                actual.required,
                actual.limit
            );
        }
    }
    #[test]
    fn source_metadata_defaults_admit_actual_payload_before_formation() {
        let high = actual_metadata_creation(16_000_000).expect("actual native metadata formation");
        assert!(high.required <= high.limit);
        let exact = actual_metadata_creation(high.required)
            .expect("exact allowance reaches native metadata formation");
        assert_eq!(exact.required, exact.limit);
        let short = actual_metadata_creation(high.required - 1);
        assert!(
            short.is_none(),
            "one-short allowance must refuse before actual metadata payload exists, observed {short:?}"
        );
    }
    #[test]
    fn source_select_task_growth_admits_actual_replacement_before_allocation() {
        let high = actual_select_task_growth(16_000_000)
            .expect("actual selected Option removal queues update");
        assert!(high.required <= high.limit);
        let exact = actual_select_task_growth(high.required)
            .expect("exact allowance reaches actual task bank growth");
        assert_eq!(exact.required, exact.limit);
        let short = actual_select_task_growth(high.required - 1);
        assert!(
            short.is_none(),
            "one-short allowance must refuse before actual task growth, observed {short:?}"
        );
    }
    #[test]
    fn source_children_growth_admits_actual_old_and_replacement_bank_before_allocation() {
        let high = actual_children_growth(16_000_000).expect("actual children capacity growth");
        assert!(high.required <= high.limit);
        let exact =
            actual_children_growth(high.required).expect("exact allowance reaches actual growth");
        assert_eq!(exact.required, exact.limit);
        let short = actual_children_growth(high.required - 1);
        assert!(
            short.is_none(),
            "one-short allowance must refuse before actual backing growth, observed {short:?}"
        );
    }
    #[test]
    fn multiple_style_mandatory_tail_cannot_publish_without_closing_revision_headroom() {
        use surgeist_css::{CssParserContext, CssUtf16InputLimits};
        use surgeist_cssom::{CssomEditGuard, CssomInputRole, CssomInputVersion, CssomSheetInputs};
        let cancel = AtomicBool::new(false);
        let mut dom = Dom::new(DomLimits::new(64, 100_000, 64, 0, 0, 8, 100_000).unwrap()).unwrap();
        let mut setup = DomBatch::new();
        let document = setup.create_document();
        let container = setup.create_element_ns(&document, crate::Namespace::html(), "div");
        for _ in 0..2 {
            let style = setup.create_element_ns(&document, crate::Namespace::html(), "style");
            setup.insert_before(&style, &container, None);
        }
        let commit = dom.apply(setup, &cancel).unwrap();
        let document = commit.document(&document).unwrap();
        let container = commit.created(&container).unwrap().unwrap();
        dom.revision = DomRevision(u64::MAX - 2);
        let before = dom.snapshot();
        let mut batch = DomBatch::new();
        batch.insert_before(container, document.node(), None);
        let small = DomSourceLimits::new(16_000_000, 1_000_000, 10_000_000, 2, 4096, 4096, 1);
        let mut lease = dom.begin_source_commands(batch, small, &cancel).unwrap();
        let operation = lease.operation();
        // Refusal at the original private boundary is the valid outcome. The
        // diagnostic branch below demonstrates the actual irreversible trap
        // if an implementation instead exposes the first reached request.
        let policy = match lease.advance(&cancel) {
            Err(DomError::RevisionExhausted | DomError::SourcePublicationLimit) => {
                assert!(!lease.status().has_published());
                assert_eq!(lease.snapshot().stamp(), before.stamp());
                return;
            }
            Ok(DomSourceProgress::Request(DomSourceRequest::Style(request))) => request,
            other => panic!("unexpected initial result: {other:?}"),
        };
        assert!(lease.status().has_published());
        lease
            .accept_inline_csp(
                &policy,
                &DomStyleInlineCspObservation::complete_empty(policy.source(), 1),
                &cancel,
            )
            .unwrap();
        let DomSourceProgress::Request(DomSourceRequest::Style(create)) =
            lease.advance(&cancel).unwrap()
        else {
            panic!("actual first creation")
        };
        let root = DomStyleCssomRoot::new(
            create.cssom_root().unwrap(),
            surgeist_cssom::CssomStore::new(Default::default(), Default::default()).unwrap(),
        );
        lease.bind_cssom_root(&create, &root, &cancel).unwrap();
        let create = lease
            .qualify_cssom(&create, root.root(), &root.snapshot().unwrap())
            .unwrap();
        let version = CssomInputVersion {
            role: CssomInputRole::Owner,
            identity: "near-counter-end".into(),
            revision: 1,
        };
        lease
            .bind_cssom_owner(
                &create,
                DomStyleCssomOwnerBinding::new(&create, version.clone(), "first-style".into())
                    .unwrap(),
                &cancel,
            )
            .unwrap();
        let context = CssParserContext::default();
        let body = context
            .parse_sheet_utf16_with_limits(
                create.source().text().clone(),
                CssUtf16InputLimits::default(),
                || Ok::<_, ()>(()),
            )
            .unwrap();
        let media = context
            .parse_media_query_list_utf16_with_limits(
                Arc::new(Vec::new()),
                CssUtf16InputLimits::default(),
                || Ok::<_, ()>(()),
            )
            .unwrap();
        let mut inputs = CssomSheetInputs::external(version, true);
        inputs.owner = Some("first-style".into());
        let mut edit = root
            .batch(CssomEditGuard::from_snapshot(&root.snapshot().unwrap()))
            .unwrap();
        let construction = edit
            .create_parsed_sheet_with_media_reports(body, media, inputs)
            .unwrap();
        let publication = root.commit(&mut Some(edit)).unwrap();
        let actual_sheet = publication.sheet(construction.ticket()).unwrap().clone();
        let notification = lease
            .accept_cssom_publication(
                &create,
                publication,
                root.snapshot().unwrap(),
                Some(construction.ticket()),
            )
            .unwrap();
        assert!(notification.is_some());
        assert_eq!(root.snapshot().unwrap().sheets(), &[actual_sheet]);
        assert_eq!(
            lease.advance(&cancel).unwrap_err(),
            DomError::SourcePublicationLimit
        );
        drop(lease);
        let larger = DomSourceLimits::new(16_000_000, 1_000_000, 10_000_000, 3, 4096, 4096, 1);
        assert!(matches!(
            dom.resume_source_edit(operation, larger, &cancel),
            Err(DomError::RevisionExhausted)
        ));
        assert!(dom.pending_source_edit().unwrap().has_published());
        panic!(
            "first publication stranded the original mandatory command after a genuine CSSOM commit; raising phases cannot create closing revisions"
        );
    }
    pub(in crate::store) fn commit_actual_empty_sheet(
        lease: &mut DomSourceLease<'_>,
        create: &DomStyleRequest,
        cancel: &AtomicBool,
    ) -> DomStyleCssomRoot {
        use surgeist_css::{CssParserContext, CssUtf16InputLimits};
        use surgeist_cssom::{CssomEditGuard, CssomInputRole, CssomInputVersion, CssomSheetInputs};
        let root = DomStyleCssomRoot::new(
            create.cssom_root().unwrap(),
            surgeist_cssom::CssomStore::new(Default::default(), Default::default()).unwrap(),
        );
        lease.bind_cssom_root(create, &root, cancel).unwrap();
        let create = lease
            .qualify_cssom(create, root.root(), &root.snapshot().unwrap())
            .unwrap();
        let version = CssomInputVersion {
            role: CssomInputRole::Owner,
            identity: "near-counter-end".into(),
            revision: 1,
        };
        lease
            .bind_cssom_owner(
                &create,
                DomStyleCssomOwnerBinding::new(&create, version.clone(), "first-style".into())
                    .unwrap(),
                cancel,
            )
            .unwrap();
        let context = CssParserContext::default();
        let body = context
            .parse_sheet_utf16_with_limits(
                create.source().text().clone(),
                CssUtf16InputLimits::default(),
                || Ok::<_, ()>(()),
            )
            .unwrap();
        let media = context
            .parse_media_query_list_utf16_with_limits(
                Arc::new(Vec::new()),
                CssUtf16InputLimits::default(),
                || Ok::<_, ()>(()),
            )
            .unwrap();
        let mut inputs = CssomSheetInputs::external(version, true);
        inputs.owner = Some("first-style".into());
        let mut edit = root
            .batch(CssomEditGuard::from_snapshot(&root.snapshot().unwrap()))
            .unwrap();
        let construction = edit
            .create_parsed_sheet_with_media_reports(body, media, inputs)
            .unwrap();
        let publication = root.commit(&mut Some(edit)).unwrap();
        let actual_sheet = publication.sheet(construction.ticket()).unwrap().clone();
        let notification = lease
            .accept_cssom_publication(
                &create,
                publication,
                root.snapshot().unwrap(),
                Some(construction.ticket()),
            )
            .unwrap();
        assert!(notification.is_some());
        assert_eq!(root.snapshot().unwrap().sheets(), &[actual_sheet]);
        root
    }
    fn actual_structural_and_control_reads(work: u64, cancelled: bool) -> (bool, u64) {
        let cancel = AtomicBool::new(false);
        let domain = DomLimits::new(256, 100_000, 256, 0, 0, 8, 100_000)
            .unwrap()
            .with_select_processing(256, 1_000_000, 1_000_000)
            .unwrap()
            .with_select_value_queries(
                surgeist_html_semantics::HtmlCaptureLimits::new(256, 512, 256, 100_000, 256, 0, 0),
                surgeist_html_semantics::HtmlSemanticLimits::new(
                    1_000_000, 1, 256, 256, 1_000_000, 256,
                ),
            );
        let mut dom = Dom::new(domain).unwrap();
        let mut setup = DomBatch::new();
        let document = setup.create_document();
        let other_document = setup.create_document();
        let root = setup.create_element_ns(&document, crate::Namespace::html(), "div");
        setup.insert_before(&root, &document, None);
        let form = setup.create_element_ns(&document, crate::Namespace::html(), "form");
        setup.set_attribute(&form, "id", "f".repeat(512));
        setup.insert_before(&form, &root, None);
        let input = setup.create_element_ns(&document, crate::Namespace::html(), "input");
        setup.insert_before(&input, &root, None);
        let select = setup.create_element_ns(&document, crate::Namespace::html(), "select");
        let option = setup.create_element_ns(&document, crate::Namespace::html(), "option");
        let label = setup.create_text(&document, "chosen");
        setup.insert_before(&label, &option, None);
        setup.insert_before(&option, &select, None);
        setup.insert_before(&select, &root, None);
        let moving = setup.create_element_ns(&document, crate::Namespace::html(), "div");
        let retired = setup.create_element_ns(&document, crate::Namespace::html(), "div");
        for _ in 0..32 {
            let child = setup.create_element_ns(&document, crate::Namespace::html(), "span");
            setup.insert_before(&child, &moving, None);
            let old = setup.create_text(&document, "retired");
            setup.insert_before(&old, &retired, None);
        }
        let created = dom.apply(setup, &cancel).unwrap();
        let other_document = created.document(&other_document).unwrap();
        let root = created.created(&root).unwrap().unwrap();
        let input = created.created(&input).unwrap().unwrap();
        let select = created.created(&select).unwrap().unwrap();
        let moving = created.created(&moving).unwrap().unwrap();
        let retired = created.created(&retired).unwrap().unwrap();
        let before = dom.snapshot();
        let mut commands = DomBatch::new();
        commands.set_attribute(input, "form", "f".repeat(512));
        commands.set_select_value(select, "chosen");
        commands.insert_before(moving, root, None);
        commands.adopt(moving, other_document);
        commands.destroy(retired);
        let mut lease = match dom.begin_source_commands(commands, limits(work), &cancel) {
            Ok(lease) => lease,
            Err(DomError::SourceWorkLimit { limit }) if limit == work => return (false, work),
            Err(error) => panic!("actual Source work setup: {error:?}"),
        };
        let meter = lease
            .dom
            .source_operation
            .as_ref()
            .unwrap()
            .draft
            .source_work
            .clone();
        let initial = meter.load(Ordering::Relaxed);
        cancel.store(cancelled, Ordering::Relaxed);
        let result = lease.advance(&cancel);
        let spent = meter.load(Ordering::Relaxed);
        match result {
            Ok(DomSourceProgress::Complete(_)) => {
                assert!(!cancelled);
                let snapshot = lease.snapshot();
                assert_eq!(
                    snapshot.node(moving).unwrap().owner_document(),
                    Some(other_document)
                );
                assert_eq!(snapshot.node(moving).unwrap().parent(), None);
                assert!(snapshot.node(retired).is_err());
                assert_eq!(
                    snapshot
                        .node(input)
                        .unwrap()
                        .as_element()
                        .unwrap()
                        .attribute(&"form".into())
                        .unwrap()
                        .len(),
                    512
                );
                (true, spent)
            }
            Err(DomError::SourceWorkLimit { limit }) if limit == work => {
                assert_eq!(lease.snapshot().stamp(), before.stamp());
                (false, spent)
            }
            Err(DomError::Cancelled) if cancelled => {
                assert_eq!(spent, initial);
                assert_eq!(lease.snapshot().stamp(), before.stamp());
                (false, spent)
            }
            other => panic!("actual Source structural/control result: {other:?}"),
        }
    }
    #[test]
    fn source_structural_control_and_task_reads_obey_exact_work_and_cancel_boundaries() {
        let (complete, exact) = actual_structural_and_control_reads(100_000_000, false);
        assert!(complete);
        assert!(exact > 512);
        assert_eq!(
            actual_structural_and_control_reads(exact, false),
            (true, exact)
        );
        assert_eq!(
            actual_structural_and_control_reads(exact - 1, false),
            (false, exact - 1)
        );
        assert!(!actual_structural_and_control_reads(exact, true).0);
    }

    #[test]
    fn mandatory_tail_issues_ordinals_only_on_exposure_and_resets_at_the_next_command() {
        let cancel = AtomicBool::new(false);
        let mut dom = Dom::new(DomLimits::new(64, 100_000, 64, 0, 0, 8, 100_000).unwrap()).unwrap();
        let mut setup = DomBatch::new();
        let document = setup.create_document();
        let parent = setup.create_element_ns(&document, crate::Namespace::html(), "div");
        setup.insert_before(&parent, &document, None);
        let first = setup.create_element_ns(&document, crate::Namespace::html(), "style");
        let second = setup.create_element_ns(&document, crate::Namespace::html(), "style");
        let created = dom.apply(setup, &cancel).unwrap();
        let parent = created.created(&parent).unwrap().unwrap();
        let first = created.created(&first).unwrap().unwrap();
        let second = created.created(&second).unwrap().unwrap();
        let before = dom.snapshot();
        let mut commands = DomBatch::new();
        commands.insert_before(first, parent, None);
        commands.insert_before(second, parent, None);
        let mut policy = limits(100_000_000);
        policy.outputs = 0;
        let mut lease = dom
            .begin_source_commands(commands, policy, &cancel)
            .unwrap();
        let operation = lease.operation();
        // Real fixed-counter boundary: one remaining ordinal can serve the
        // first command, and failed unpublished attempts must not consume it.
        lease
            .dom
            .source_operation
            .as_mut()
            .unwrap()
            .draft
            .source_invocation = u64::MAX - 1;
        let mut work = lease.status().spent_work();
        for _ in 0..2 {
            assert_eq!(
                lease.advance(&cancel).unwrap_err(),
                DomError::SourceOutputLimit
            );
            let op = lease.dom.source_operation.as_ref().unwrap();
            assert_eq!(op.draft.source_invocation, u64::MAX - 1);
            assert!(op.request.is_none());
            assert!(op.context.is_none());
            assert!(!op.published);
            assert!(!op.started);
            assert_eq!(lease.snapshot().stamp(), before.stamp());
            assert!(lease.status().spent_work() > work);
            work = lease.status().spent_work();
        }
        // Reborrow this actual permission owner so its public admission path
        // raises only the failed output budget without dropping the operation.
        let mut raised = lease
            .dom
            .resume_source_edit(operation, limits(100_000_000), &cancel)
            .unwrap();
        let DomSourceProgress::Request(DomSourceRequest::Style(csp)) =
            raised.advance(&cancel).unwrap()
        else {
            panic!("actual first CSP request")
        };
        assert_eq!(csp.source().ordinal(), u64::MAX - 1);
        {
            let op = raised.dom.source_operation.as_ref().unwrap();
            assert_eq!(op.draft.source_invocation, u64::MAX);
            assert_eq!(op.index, 0);
            assert!(op.command_admitted);
            let original = op.draft.source_limits.unwrap();
            let reserved = op.headroom.as_ref().unwrap().remaining_phases;
            let mut too_small = original;
            too_small.phases = op.phases + reserved - 1;
            assert!(matches!(
                raised.dom.resume_source_edit(operation, too_small, &cancel),
                Err(DomError::InvalidLimit(_))
            ));
            assert_eq!(
                raised
                    .dom
                    .source_operation
                    .as_ref()
                    .unwrap()
                    .draft
                    .source_limits,
                Some(original)
            );
        }
        let DomSourceProgress::Request(DomSourceRequest::Style(repeated)) =
            raised.advance(&cancel).unwrap()
        else {
            panic!("same exposed request")
        };
        assert_eq!(repeated.source().operation(), csp.source().operation());
        assert_eq!(repeated.source().ordinal(), csp.source().ordinal());
        assert_eq!(repeated.source().stamp(), csp.source().stamp());
        assert_eq!(repeated.source().element(), csp.source().element());
        assert_eq!(
            raised
                .dom
                .source_operation
                .as_ref()
                .unwrap()
                .draft
                .source_invocation,
            u64::MAX
        );
        raised
            .accept_inline_csp(
                &csp,
                &DomStyleInlineCspObservation::complete_empty(csp.source(), 1),
                &cancel,
            )
            .unwrap();
        let DomSourceProgress::Request(DomSourceRequest::Style(create)) =
            raised.advance(&cancel).unwrap()
        else {
            panic!("actual same-invocation create")
        };
        assert_eq!(create.source().ordinal(), csp.source().ordinal());
        let root = commit_actual_empty_sheet(&mut raised, &create, &cancel);
        let cssom_revision = root.snapshot().unwrap().revision().clone();
        assert_eq!(
            raised.advance(&cancel).unwrap_err(),
            DomError::RepresentationExhausted
        );
        let op = raised.dom.source_operation.as_ref().unwrap();
        assert_eq!(op.index, 1);
        assert!(!op.started);
        assert!(!op.command_admitted);
        assert!(op.request.is_none());
        assert!(op.context.is_none());
        assert_eq!(op.draft.source_invocation, u64::MAX);
        assert_eq!(
            raised.snapshot().node(first).unwrap().parent(),
            Some(parent)
        );
        assert_eq!(raised.snapshot().node(second).unwrap().parent(), None);
        assert_eq!(root.snapshot().unwrap().revision(), &cssom_revision);
        raised.request_cancel();
        assert!(matches!(
            raised.advance(&cancel).unwrap(),
            DomSourceProgress::Complete(_)
        ));
        assert_eq!(root.snapshot().unwrap().revision(), &cssom_revision);
        assert_eq!(root.snapshot().unwrap().sheets().len(), 1);
    }

    #[test]
    fn details_mandatory_tail_reserves_its_later_publication_identity_before_cssom_commit() {
        let cancel = AtomicBool::new(false);
        let domain = DomLimits::new(64, 100_000, 128, 0, 0, 8, 100_000)
            .unwrap()
            .with_details_toggle_intentions(64)
            .unwrap();
        let mut dom = Dom::new(domain).unwrap();
        let mut setup = DomBatch::new();
        let document = setup.create_document();
        let connected = setup.create_element_ns(&document, crate::Namespace::html(), "div");
        let peer = setup.create_element_ns(&document, crate::Namespace::html(), "details");
        setup.set_attribute(&peer, "name", "g");
        setup.set_attribute(&peer, "open", "");
        setup.insert_before(&peer, &connected, None);
        setup.insert_before(&connected, &document, None);
        let container = setup.create_element_ns(&document, crate::Namespace::html(), "div");
        let style = setup.create_element_ns(&document, crate::Namespace::html(), "style");
        setup.insert_before(&style, &container, None);
        let later = setup.create_element_ns(&document, crate::Namespace::html(), "details");
        setup.set_attribute(&later, "name", "g");
        setup.set_attribute(&later, "open", "");
        setup.insert_before(&later, &container, None);
        let created = dom.apply(setup, &cancel).unwrap();
        let connected = created.created(&connected).unwrap().unwrap();
        let container = created.created(&container).unwrap().unwrap();
        let peer = created.created(&peer).unwrap().unwrap();
        let later = created.created(&later).unwrap().unwrap();
        dom.take_details_toggle_intentions(64, &cancel).unwrap();
        assert_eq!(dom.pending_details_toggle_intentions(), 0);
        Arc::get_mut(&mut dom.state)
            .expect("exclusive actual state")
            .next_details_intention = u64::MAX;
        let before = dom.snapshot();
        assert_eq!(
            before
                .node(peer)
                .unwrap()
                .as_element()
                .unwrap()
                .details_state(),
            Some(surgeist_html::HtmlDetailsState::Open)
        );
        assert_eq!(
            before
                .node(later)
                .unwrap()
                .as_element()
                .unwrap()
                .details_state(),
            Some(surgeist_html::HtmlDetailsState::Open)
        );
        let mut batch = DomBatch::new();
        batch.insert_before(container, connected, None);
        let mut lease = dom
            .begin_source_commands(batch, limits(10_000_000), &cancel)
            .unwrap();
        let policy = match lease.advance(&cancel) {
            Err(DomError::RepresentationExhausted) => {
                assert!(!lease.status().has_published());
                assert_eq!(lease.snapshot().stamp(), before.stamp());
                return;
            }
            Ok(DomSourceProgress::Request(DomSourceRequest::Style(request))) => request,
            other => {
                panic!("expected pre-publication counter refusal or real CSP boundary: {other:?}")
            }
        };
        lease
            .accept_inline_csp(
                &policy,
                &DomStyleInlineCspObservation::complete_empty(policy.source(), 1),
                &cancel,
            )
            .unwrap();
        let DomSourceProgress::Request(DomSourceRequest::Style(create)) =
            lease.advance(&cancel).unwrap()
        else {
            panic!("actual creation")
        };
        let root = commit_actual_empty_sheet(&mut lease, &create, &cancel);
        assert_eq!(root.snapshot().unwrap().sheets().len(), 1);
        assert_eq!(
            lease.advance(&cancel).unwrap_err(),
            DomError::RepresentationExhausted
        );
        assert!(
            !lease.status().has_published(),
            "later Details identity exhaustion stranded a mandatory tail after a genuine CSSOM creation"
        );
    }
    #[test]
    fn select_mandatory_tail_reserves_new_same_target_identity_after_acknowledged_prefix() {
        use surgeist_cssom::CssomEditGuard;
        let cancel = AtomicBool::new(false);
        let domain = DomLimits::new(64, 100_000, 128, 0, 0, 8, 100_000)
            .unwrap()
            .with_select_processing(64, 1_000_000, 10_000_000)
            .unwrap();
        let mut dom = Dom::new(domain).unwrap();
        let mut setup = DomBatch::new();
        let document = setup.create_document();
        let select = setup.create_element_ns(&document, crate::Namespace::html(), "select");
        setup.set_attribute(&select, "multiple", "");
        let container = setup.create_element_ns(&document, crate::Namespace::html(), "div");
        let first = setup.create_element_ns(&document, crate::Namespace::html(), "option");
        setup.set_attribute(&first, "selected", "");
        setup.insert_before(&first, &container, None);
        let style = setup.create_element_ns(&document, crate::Namespace::html(), "style");
        setup.insert_before(&style, &container, None);
        let last = setup.create_element_ns(&document, crate::Namespace::html(), "option");
        setup.set_attribute(&last, "selected", "");
        setup.insert_before(&last, &container, None);
        setup.insert_before(&container, &select, None);
        let created = dom.apply(setup, &cancel).unwrap();
        let document = created.document(&document).unwrap();
        let select = created.created(&select).unwrap().unwrap();
        let container = created.created(&container).unwrap().unwrap();
        let first = created.created(&first).unwrap().unwrap();
        let last = created.created(&last).unwrap().unwrap();
        let root = {
            let mut connect = DomBatch::new();
            connect.insert_before(select, document.node(), None);
            let mut lease = dom
                .begin_source_commands(connect, limits(10_000_000), &cancel)
                .unwrap();
            let DomSourceProgress::Request(DomSourceRequest::Style(policy)) =
                lease.advance(&cancel).unwrap()
            else {
                panic!("real CSP")
            };
            lease
                .accept_inline_csp(
                    &policy,
                    &DomStyleInlineCspObservation::complete_empty(policy.source(), 1),
                    &cancel,
                )
                .unwrap();
            let DomSourceProgress::Request(DomSourceRequest::Style(create)) =
                lease.advance(&cancel).unwrap()
            else {
                panic!("real sheet creation")
            };
            let root = commit_actual_empty_sheet(&mut lease, &create, &cancel);
            assert!(matches!(
                lease.advance(&cancel).unwrap(),
                DomSourceProgress::Complete(_)
            ));
            root
        };
        assert_eq!(dom.pending_native_tasks(), 0);
        Arc::get_mut(&mut dom.state)
            .expect("exclusive actual state")
            .next_task = u64::MAX - 1;
        let before = dom.snapshot();
        for option in [first, last] {
            let node = before.node(option).unwrap();
            assert_eq!(
                node.as_option().unwrap().cached_select().unwrap(),
                Some(select)
            );
            assert!(
                node.as_element()
                    .unwrap()
                    .option_state()
                    .unwrap()
                    .flags()
                    .selectedness()
            );
        }
        let mut batch = DomBatch::new();
        batch.detach(container);
        let mut lease = dom
            .begin_source_commands(batch, limits(10_000_000), &cancel)
            .unwrap();
        let remove = match lease.advance(&cancel) {
            Err(DomError::RepresentationExhausted) => {
                assert!(!lease.status().has_published());
                assert_eq!(lease.snapshot().stamp(), before.stamp());
                return;
            }
            Ok(DomSourceProgress::Request(DomSourceRequest::Style(request))) => request,
            other => {
                panic!("expected pre-publication counter refusal or actual removal: {other:?}")
            }
        };
        assert_eq!(
            remove.action(),
            DomStyleAction::Update(surgeist_html::HtmlStyleUpdateAction::RemoveAssociatedSheet)
        );
        assert_eq!(lease.dom.state.next_task, u64::MAX);
        assert!(
            matches!(lease.dom.state.native_tasks.as_slice(), [select::Task::Update { serial, target }] if *serial == u64::MAX - 1 && *target == select)
        );
        let remove = lease
            .qualify_cssom(&remove, root.root(), &root.snapshot().unwrap())
            .unwrap();
        let mut edit = root
            .batch(CssomEditGuard::from_snapshot(&root.snapshot().unwrap()))
            .unwrap();
        edit.remove_sheet(remove.sheet().unwrap().id()).unwrap();
        let publication = root.commit(&mut Some(edit)).unwrap();
        assert!(
            lease
                .accept_cssom_publication(&remove, publication, root.snapshot().unwrap(), None)
                .unwrap()
                .is_none()
        );
        assert!(root.snapshot().unwrap().sheets().is_empty());
        assert_eq!(
            lease.advance(&cancel).unwrap_err(),
            DomError::RepresentationExhausted
        );
        assert!(
            !lease.status().has_published(),
            "new same-target Select task exhausted after a different task identity was issued and real CSSOM removal committed"
        );
    }
    #[test]
    fn failed_semantic_query_retains_child_spent_work_across_source_retry() {
        use surgeist_html_semantics::{HtmlCaptureLimits, HtmlSemanticLimits};
        let cancel = AtomicBool::new(false);
        let domain = DomLimits::new(32, 100_000, 64, 0, 0, 8, 100_000)
            .unwrap()
            .with_select_processing(16, 1_000_000, 1_000_000)
            .unwrap()
            .with_select_value_queries(
                HtmlCaptureLimits::new(32, 64, 32, 100_000, 32, 0, 0),
                HtmlSemanticLimits::new(1, 1, 128, 0, 1_000_000, 128),
            );
        let mut dom = Dom::new(domain).unwrap();
        let mut setup = DomBatch::new();
        let document = setup.create_document();
        let select = setup.create_element_ns(&document, crate::Namespace::html(), "select");
        let option = setup.create_element_ns(&document, crate::Namespace::html(), "option");
        setup.set_attribute(&option, "value", "actual option value");
        setup.insert_before(&option, &select, None);
        let created = dom.apply(setup, &cancel).unwrap();
        let select = created.created(&select).unwrap().unwrap();
        let before = dom.snapshot();
        let mut batch = DomBatch::new();
        batch.set_select_value(select, "different");
        let mut lease = dom
            .begin_source_commands(batch, limits(10_000_000), &cancel)
            .unwrap();
        let Err(DomError::SelectSemantic(first)) = lease.advance(&cancel) else {
            panic!("actual finite child query failure");
        };
        let spent = first.usage();
        assert!(spent.work() > 0);
        let actual = lease
            .dom
            .source_operation
            .as_ref()
            .unwrap()
            .draft
            .select_semantic_work
            .get();
        assert_eq!(
            actual,
            spent.work(),
            "actual failed producer work disappeared at Source checkpoint restore"
        );
        assert_eq!(
            lease
                .dom
                .source_operation
                .as_ref()
                .unwrap()
                .draft
                .select_semantic_bytes
                .get(),
            spent.allocated_bytes()
        );
        let Err(DomError::SelectSemantic(second)) = lease.advance(&cancel) else {
            panic!("same absolute allowance must remain exhausted");
        };
        assert_eq!(
            second.usage().work(),
            0,
            "retry cannot spend the original child work allowance again"
        );
        assert_eq!(
            lease
                .dom
                .source_operation
                .as_ref()
                .unwrap()
                .draft
                .select_semantic_work
                .get(),
            spent.work()
        );
        assert_eq!(lease.snapshot().stamp(), before.stamp());
        assert!(!lease.status().has_published());
    }
    #[test]
    fn unpublished_cancellation_cannot_bypass_its_spent_source_work_allowance() {
        let cancel = AtomicBool::new(false);
        let (mut measured, batch) = fixture();
        let first = measured
            .begin_source_commands(batch, limits(10_000_000), &cancel)
            .unwrap();
        let initial = first
            .dom
            .source_operation
            .as_ref()
            .unwrap()
            .draft
            .source_work
            .load(Ordering::Relaxed);
        drop(first);
        let (mut dom, batch) = fixture();
        let before = dom.snapshot();
        let mut lease = dom
            .begin_source_commands(batch, limits(initial), &cancel)
            .unwrap();
        lease.request_cancel();
        assert!(
            matches!(
                lease.advance(&cancel),
                Err(DomError::SourceWorkLimit { .. })
            ),
            "cancellation must admit its real result scan/formation before doing the work"
        );
        assert_eq!(lease.snapshot().stamp(), before.stamp());
    }
    #[test]
    fn source_operation_issuance_checks_exhaustion_and_never_reuses_an_issued_id() {
        let cancel = AtomicBool::new(false);
        let (mut dom, _) = fixture();
        dom.next_source_operation = u64::MAX;
        let before = dom.snapshot();
        assert!(matches!(
            dom.begin_source_commands(DomBatch::new(), limits(10_000_000), &cancel),
            Err(DomError::RepresentationExhausted)
        ));
        assert!(dom.source_operation.is_none());
        assert_eq!(dom.snapshot().stamp(), before.stamp());
        dom.next_source_operation = u64::MAX - 1;
        let lease = dom
            .begin_source_commands(DomBatch::new(), limits(10_000_000), &cancel)
            .unwrap();
        assert_eq!(lease.operation().serial, u64::MAX - 1);
        drop(lease);
        assert_eq!(dom.next_source_operation, u64::MAX);
        assert!(matches!(
            dom.begin_source_commands(DomBatch::new(), limits(10_000_000), &cancel),
            Err(DomError::RepresentationExhausted)
        ));
    }
    #[test]
    fn populated_candidate_retains_its_actual_text_backing_charge_until_discard() {
        let cancel = AtomicBool::new(false);
        let mut dom = Dom::new(DomLimits::new(16, 100_000, 32, 0, 0, 8, 100_000).unwrap()).unwrap();
        let mut setup = DomBatch::new();
        let document = setup.create_document();
        let text = "candidate payload".repeat(4096);
        let units = text.len();
        setup.create_text(&document, text);
        dom.apply(setup, &cancel).unwrap();
        let lease = dom
            .begin_source_commands(DomBatch::new(), limits(10_000_000), &cancel)
            .unwrap();
        let draft = &lease.dom.source_operation.as_ref().unwrap().draft;
        let prepared = draft.finish(lease.dom, &cancel).unwrap();
        // This is the actual populated candidate, not a requested final size.
        assert!(prepared.state.slots.iter().any(|slot| matches!(slot.node.as_ref().map(|n| &n.kind), Some(DomNodeKind::Text(value)) if value.len() == units)));
        assert!(
            draft.source_formation_bytes.load(Ordering::Relaxed)
                >= units * size_of::<u16>() + size_of::<Vec<u16>>(),
            "candidate backing occurrence must remain charged while Prepared owns it"
        );
        assert!(
            draft.source_temporary_units.load(Ordering::Relaxed) >= units,
            "candidate UTF16 occurrence must remain charged while Prepared owns it"
        );
        drop(prepared);
        assert_eq!(draft.source_formation_bytes.load(Ordering::Relaxed), 0);
        assert_eq!(draft.source_temporary_units.load(Ordering::Relaxed), 0);
    }
    #[test]
    fn candidate_third_payload_occurrence_admits_exact_utf16_and_refuses_one_short() {
        let cancel = AtomicBool::new(false);
        for short in [false, true] {
            let mut dom =
                Dom::new(DomLimits::new(16, 100_000, 32, 0, 0, 8, 100_000).unwrap()).unwrap();
            let mut setup = DomBatch::new();
            let document = setup.create_document();
            setup.create_text(&document, "candidate payload".repeat(4096));
            dom.apply(setup, &cancel).unwrap();
            // Independently enumerate this fixture's actual UTF16 backings:
            // three native Document strings and one detached Text. No facts,
            // attributes, form-name bank or imported history is present.
            let occurrence = dom
                .state
                .slots
                .iter()
                .filter_map(|slot| slot.node.as_ref())
                .map(|node| match &node.kind {
                    DomNodeKind::Document(metadata) => {
                        [metadata.content_type(), metadata.encoding(), metadata.url()]
                            .into_iter()
                            .map(|s| s.shared_code_units().capacity())
                            .sum::<usize>()
                    }
                    DomNodeKind::Text(value) => value.shared_code_units().capacity(),
                    _ => panic!("fixture has only a Document and Text"),
                })
                .sum::<usize>();
            let exact = occurrence * 3; // Original canonical + Draft + candidate.
            let limit = exact - usize::from(short);
            let before = dom.snapshot();
            let lease = dom
                .begin_source_commands(
                    DomBatch::new(),
                    DomSourceLimits::new(16_000_000, limit, 10_000_000, 64, 4096, 4096, 1),
                    &cancel,
                )
                .unwrap();
            let draft = &lease.dom.source_operation.as_ref().unwrap().draft;
            let result = draft.finish(lease.dom, &cancel);
            if short {
                assert!(
                    matches!(result, Err(DomError::ResourceLimit { resource: DomResource::SourceRetainedUnits, limit: actual }) if actual == limit)
                );
            } else {
                let prepared = result.unwrap();
                assert_eq!(
                    draft.source_temporary_units.load(Ordering::Relaxed),
                    occurrence
                );
                drop(prepared);
            }
            assert_eq!(draft.source_temporary_units.load(Ordering::Relaxed), 0);
            assert_eq!(draft.source_formation_bytes.load(Ordering::Relaxed), 0);
            assert_eq!(lease.snapshot().stamp(), before.stamp());
        }
    }
    fn candidate_occupied_boundary(
        bytes: usize,
    ) -> (
        Option<formation::candidate_observation::Boundary>,
        Result<(), DomError>,
    ) {
        let cancel = AtomicBool::new(false);
        let mut dom = Dom::new(DomLimits::new(16, 100_000, 32, 0, 0, 8, 100_000).unwrap()).unwrap();
        let mut setup = DomBatch::new();
        let document = setup.create_document();
        let mut padded = Vec::with_capacity(65_536);
        padded.push(0xd800);
        setup.create_text(&document, DocumentString::from_utf16(padded));
        dom.apply(setup, &cancel).unwrap();
        let before = dom.snapshot();
        let lease = dom
            .begin_source_commands(
                DomBatch::new(),
                DomSourceLimits::new(bytes, 1_000_000, 10_000_000, 64, 4096, 4096, 1),
                &cancel,
            )
            .unwrap();
        let draft = &lease.dom.source_operation.as_ref().unwrap().draft;
        formation::candidate_observation::start();
        let result = draft.finish(lease.dom, &cancel);
        let boundary = formation::candidate_observation::finish();
        let result = result.map(drop);
        assert_eq!(draft.source_formation_bytes.load(Ordering::Relaxed), 0);
        assert_eq!(draft.source_temporary_units.load(Ordering::Relaxed), 0);
        assert_eq!(lease.snapshot().stamp(), before.stamp());
        (boundary, result)
    }
    #[test]
    fn actual_populated_candidate_and_occupants_overlap_admits_exact_bytes_before_retention() {
        let (high, result) = candidate_occupied_boundary(16_000_000);
        result.unwrap();
        let boundary = high.unwrap();
        assert!(boundary.occupants > 0);
        assert_eq!(
            boundary.admitted, boundary.required,
            "independent live bank/backing enumeration equals the counter at actual populated overlap"
        );
        assert!(boundary.required <= boundary.limit);
        let (exact, _) = candidate_occupied_boundary(boundary.required);
        let exact = exact.expect("exact allowance reaches this real populated overlap, irrespective of later output formation");
        assert_eq!(exact.admitted, exact.required);
        assert_eq!(exact.required, exact.limit);
        let (short, result) = candidate_occupied_boundary(boundary.required - 1);
        assert!(
            short.is_none(),
            "one-short refuses before the candidate Text retains its spare-capacity backing"
        );
        assert!(
            matches!(result, Err(DomError::ResourceLimit { resource: DomResource::SourceRetainedBytes, limit }) if limit == boundary.required - 1)
        );
    }
    #[test]
    fn formed_source_input_holds_its_utf16_charge_while_candidate_remains_live() {
        let cancel = AtomicBool::new(false);
        let mut dom = Dom::new(DomLimits::new(32, 100_000, 64, 0, 0, 8, 100_000).unwrap()).unwrap();
        let mut setup = DomBatch::new();
        let document = setup.create_document();
        let style = setup.create_element_ns(&document, crate::Namespace::html(), "style");
        let text = setup.create_text(&document, "a { color: red }".repeat(2048));
        setup.insert_before(&text, &style, None);
        let commit = dom.apply(setup, &cancel).unwrap();
        let document = commit.document(&document).unwrap();
        let style = commit.created(&style).unwrap().unwrap();
        let mut batch = DomBatch::new();
        batch.insert_before(style, document, None);
        let lease = dom
            .begin_source_commands(batch, limits(10_000_000), &cancel)
            .unwrap();
        let actual_limits = lease.dom.limits;
        let op = lease.dom.source_operation.as_mut().unwrap();
        assert!(matches!(
            op.apply_command(actual_limits, &cancel),
            Err(DomError::StyleCompositionRequired(_))
        ));
        let frame = op.frame().unwrap();
        let op = lease.dom.source_operation.as_ref().unwrap();
        let prepared = op.draft.finish(lease.dom, &cancel).unwrap();
        let prior = op.draft.source_temporary_units.load(Ordering::Relaxed);
        let source = op
            .source(
                &prepared.state,
                &prepared.allocation_ids,
                DomStamp {
                    owner: lease.dom.owner,
                    revision: DomRevision(lease.dom.revision.0 + 1),
                },
                frame,
                prepared.formation.as_ref(),
                &cancel,
            )
            .unwrap();
        assert_eq!(
            op.draft.source_temporary_units.load(Ordering::Relaxed) - prior,
            source.retained_units().unwrap(),
            "actual assembled text/metadata/string capture stays live through the rest of candidate formation"
        );
        assert!(!source.text().is_empty());
        drop(source);
        drop(prepared);
        assert_eq!(op.draft.source_temporary_units.load(Ordering::Relaxed), 0);
        assert_eq!(op.draft.source_formation_bytes.load(Ordering::Relaxed), 0);
    }
    fn slot_operation_fixture() -> (Dom, DomBatch, NodeId, NodeId, NodeId) {
        let cancel = AtomicBool::new(false);
        let mut dom =
            Dom::new(DomLimits::new(64, 100_000, 128, 128, 128, 8, 100_000).unwrap()).unwrap();
        let mut b = DomBatch::new();
        let d = b.create_document();
        let host = b.create_element_ns(&d, crate::Namespace::html(), "div");
        b.insert_before(&host, &d, None);
        let root = b.attach_shadow(
            &host,
            crate::HtmlShadowRootProperties::new(
                crate::HtmlShadowRootMode::Open,
                crate::HtmlSlotAssignment::Manual,
                false,
                true,
                false,
            ),
        );
        let slot = b.create_element_ns(&d, crate::Namespace::html(), "slot");
        let x = b.create_text(&d, "old member");
        let y = b.create_text(&d, "new member");
        b.insert_before(&slot, &root, None);
        b.insert_before(&x, &host, None);
        b.insert_before(&y, &host, None);
        b.assign_slot(&slot, vec![(&x).into()]);
        let c = dom.apply(b, &cancel).unwrap();
        let slot = c.created(&slot).unwrap().unwrap();
        let x = c.created(&x).unwrap().unwrap();
        let y = c.created(&y).unwrap().unwrap();
        let (prepared, _) = dom
            .prepare_slot_checkpoint(crate::DomSlotQueryLimits::new(16, 16).unwrap(), &cancel)
            .unwrap();
        dom.publish(prepared, &cancel).unwrap();
        let mut batch = DomBatch::new();
        batch.assign_slot(slot, vec![y.into(), x.into(), y.into()]);
        (dom, batch, slot, x, y)
    }
    fn slot_operation_with_limits(limits: DomSourceLimits) -> bool {
        let (mut dom, batch, slot, x, y) = slot_operation_fixture();
        let before = dom.snapshot();
        let cancel = AtomicBool::new(false);
        let result = match dom.begin_source_commands(batch, limits, &cancel) {
            Err(error) => Err(error),
            Ok(mut lease) => match lease.advance(&cancel) {
                Ok(DomSourceProgress::Complete(done)) => {
                    assert!(!done.cancellation_requested());
                    Ok(())
                }
                Ok(DomSourceProgress::Request(DomSourceRequest::Style(_))) => {
                    panic!("Slot assignment has no external Style work")
                }
                Ok(DomSourceProgress::Request(DomSourceRequest::HostedChild(_))) => {
                    panic!("Slot assignment fixture has no navigable-container hooks")
                }
                Ok(DomSourceProgress::Request(
                    DomSourceRequest::Event(_)
                    | DomSourceRequest::Dialog(_)
                    | DomSourceRequest::CloseWatcher(_),
                )) => {
                    panic!("Slot assignment fixture has no eventful Dialog operation")
                }
                Err(error) => Err(error),
            },
        };
        assert!(
            dom.source_operation.is_none(),
            "unpublished permission drop discards only private work"
        );
        let current = dom.snapshot();
        match result {
            Ok(()) => {
                assert_eq!(current.slot(slot).unwrap().manual_assignments(), &[y, x]);
                assert_eq!(current.slot(slot).unwrap().assigned_nodes(), &[y, x]);
                assert_eq!(
                    current.node(y).unwrap().manual_assignment().unwrap(),
                    Some(slot)
                );
                assert_eq!(
                    current.node(y).unwrap().stored_assigned_slot().unwrap(),
                    Some(slot)
                );
                assert_eq!(before.slot(slot).unwrap().manual_assignments(), &[x]);
                assert_eq!(before.slot(slot).unwrap().assigned_nodes(), &[x]);
                true
            }
            Err(
                DomError::ResourceLimit {
                    resource: DomResource::SourceRetainedBytes | DomResource::SourceRetainedUnits,
                    ..
                }
                | DomError::SourceWorkLimit { .. },
            ) => {
                assert_eq!(current.stamp(), before.stamp());
                assert_eq!(current.slot(slot).unwrap().manual_assignments(), &[x]);
                assert_eq!(current.slot(slot).unwrap().assigned_nodes(), &[x]);
                assert_eq!(current.node(y).unwrap().manual_assignment().unwrap(), None);
                assert_eq!(
                    current.node(y).unwrap().stored_assigned_slot().unwrap(),
                    None
                );
                false
            }
            Err(error) => panic!("unexpected actual source refusal: {error:?}"),
        }
    }
    #[test]
    fn actual_slot_operation_admits_exact_bytes_units_and_work_and_rolls_back_one_short() {
        for dimension in 0..3 {
            let limits = |value: usize| {
                DomSourceLimits::new(
                    if dimension == 0 { value } else { 16_000_000 },
                    if dimension == 1 { value } else { 1_000_000 },
                    if dimension == 2 {
                        value as u64
                    } else {
                        10_000_000
                    },
                    64,
                    4096,
                    4096,
                    1,
                )
            };
            let mut low = 0;
            let mut high = [16_000_000, 1_000_000, 10_000_000][dimension];
            assert!(slot_operation_with_limits(limits(high)));
            while low < high {
                let middle = low + (high - low) / 2;
                if slot_operation_with_limits(limits(middle)) {
                    high = middle;
                } else {
                    low = middle + 1;
                }
            }
            assert!(low > 0);
            assert!(!slot_operation_with_limits(limits(low - 1)));
            assert!(slot_operation_with_limits(limits(low)));
        }
    }
}
