//! One canonical retained synchronous source operation. The lease grants access;
//! all mandatory continuation and acknowledged external publication stays in Dom.
#[cfg(test)]
mod design_mode_tests;
mod dialog;
pub(crate) mod dialog_queue;
#[cfg(test)]
pub(in crate::store) mod empty_policy_tests;
mod failure;
mod focus;
mod inline;
pub(crate) mod meta;
use inline::{AdvanceError, Execution};
pub(crate) mod fullscreen;
#[cfg(test)]
mod fullscreen_tests;
pub(in crate::store) mod hosted;
#[cfg(test)]
mod pending_identity_tests;
mod request_close;
mod scope;
#[cfg(test)]
mod scope_tests;
mod show_modal;
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
    inline_source: Option<inline::SourceInline>,
    meta_policy: Option<meta::Policy>,
    pending_meta: Option<meta::Pending>,
    meta_receipt: Option<meta::SavedReceipt>,
    draft: dialog::OperationDraft,
    commands: OperationRoot,
    index: usize,
    started: bool,
    published: bool,
    event_exposed: bool,
    event_work: std::sync::Weak<crate::source::event::EventWork>,
    event_custody: Option<Arc<formation::EventWorkCustody>>,
    dialog_queue: Option<crate::DomElementTaskId>,
    request_close: Option<request_close::Call>,
    request_close_result: Option<DocumentString>,
    failure: Option<DomSourceCleanupCause>,
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
    command_storage: storage::Payload,
    custody_bytes: usize,
    custody_units: usize,
    canonical_aux_bytes: usize,
    pub(in crate::store) names: name_registry::Operation,
    #[cfg(test)]
    _measurement_lifetime: scale_metrics::SourceOwnerLifetime,
}
#[expect(
    clippy::large_enum_variant,
    reason = "Keep the sole caller-accounted Source receipt inline; boxing adds allocation and failure during published continuation"
)]
enum Receipt {
    CloseWatcher {
        publication: crate::DomCloseWatcherRequestPublication,
        applied: bool,
    },
    Focus {
        publication: crate::DomFocusPublication,
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
            && self.dom.source_operation.as_ref().is_some_and(|o| {
                !o.published
                    && !o.event_exposed
                    && o.observation().is_none()
                    && o.meta_policy.is_none()
            })
        {
            self.dom.source_operation = self.dom.source_parents.pop();
        }
        #[cfg(test)]
        self.dom.observe_source_lifecycle();
    }
}
impl Dom {
    /// Whether this canonical owner retains any actual mutable Style CSSOM root.
    /// Historical snapshots and parsed pending associations do not count.
    pub fn has_style_cssom_roots(&self) -> bool {
        !self.style_cssom_roots.is_empty()
    }
    /// Reissues the exact retained mutable owner of a live Document or
    /// ShadowRoot through its generation-qualified canonical slot index.
    pub fn style_cssom_root(&self, root: NodeId) -> Result<Option<DomStyleCssomRoot>, DomError> {
        if self.state.incomplete_source.is_some() {
            return Err(DomError::IncompleteSourceOperation);
        }
        let node = self.state.node(self.owner, root)?;
        if !matches!(
            node.kind(),
            DomNodeKind::Document(_) | DomNodeKind::ShadowRoot(_)
        ) {
            return Err(DomError::WrongNodeKind);
        }
        Ok(self.style_cssom_roots.get(root).map(|entry| {
            #[cfg(test)]
            crate::store::style_custody_tests::observe(
                crate::store::style_custody_tests::Site::Lookup,
            );
            entry.duplicate()
        }))
    }
    pub(in crate::store) fn source_owner_custody_bytes(&self) -> Result<usize, DomError> {
        let style = crate::store::style_custody::Bank::inline_bytes()
            .checked_add(size_of::<crate::store::pending_associations::Bank>())
            .ok_or(DomError::RepresentationExhausted)?;
        let fullscreen = self.fullscreen_custody_bytes()?;
        style
            .checked_add(crate::element_tasks::retained(&self.element_tasks)?)
            .and_then(|bytes| bytes.checked_add(fullscreen))
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
            formation::allocated_arc(BatchIdentity),
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
                    #[cfg(test)]
                    self.observe_source_lifecycle();
                    return Err(error);
                }
            }
        } else {
            limits
        };
        let formed =
            Draft::new_source_root(self, owner, &commands, commands.capacity(), limits, cancel)
                .and_then(|(draft, command)| {
                    let aux = self
                        .source_retained_stack_bytes()?
                        .checked_add(
                            self.source_dispatch_custody_bytes(|| draft.source_step(cancel))?,
                        )
                        .ok_or(DomError::RepresentationExhausted)?;
                    Ok((draft, command, aux))
                });
        let (draft, command_storage, canonical_aux_bytes) = match formed {
            Ok(formed) => formed,
            Err(error) => {
                if nested {
                    self.source_operation = self.source_parents.pop();
                }
                #[cfg(test)]
                self.observe_source_lifecycle();
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
                #[cfg(test)]
                self.observe_source_lifecycle();
                return Err(error);
            }
        };
        let mut op = formation::allocated_box(Operation {
            id,
            entry_scope,
            limits: original_limits,
            names: draft.names.clone(),
            draft: dialog::OperationDraft::Segment(draft),
            inline_source: None,
            meta_policy: None,
            pending_meta: None,
            meta_receipt: None,
            commands,
            index: 0,
            started: false,
            published: false,
            event_exposed: false,
            event_work: std::sync::Weak::new(),
            event_custody: None,
            dialog_queue: None,
            request_close: None,
            request_close_result: None,
            failure: None,
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
            command_storage,
            custody_bytes: 0,
            custody_units: self.fullscreen_custody_units()?,
            canonical_aux_bytes,
            #[cfg(test)]
            _measurement_lifetime: scale_metrics::SourceOwnerLifetime(id),
        });
        let admitted = (|| {
            op.custody_bytes = self.source_owner_custody_bytes()?;
            op.refresh_admission(cancel)
        })();
        if let Err(error) = admitted {
            if nested {
                self.source_operation = self.source_parents.pop();
            }
            #[cfg(test)]
            self.observe_source_lifecycle();
            return Err(error);
        }
        let next = self
            .next_source_operation
            .checked_add(1)
            .ok_or(DomError::RepresentationExhausted)?;
        self.source_operation = Some(op);
        #[cfg(test)]
        self.observe_source_lifecycle();
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
        let custody_units = self.fullscreen_custody_units()?;
        let protected = self.source_reserved_closing(cancel)?;
        let dispatch = &self.dispatch_custody;
        let op = self
            .source_operation
            .as_mut()
            .filter(|o| o.id == id)
            .ok_or(DomError::InvalidSourceOperation)?;
        op.custody_bytes = custody_bytes;
        op.custody_units = custody_units;
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
                        .ok_or(DomError::InvalidSourceOperation)?
                        .remaining_phases,
                )
                .and_then(|revision| revision.checked_add(self.hosted_revisions))
                .and_then(|revision| revision.checked_add(protected))
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
    DialogFocusing(crate::store::dialog::PendingFocusing),
    AutofocusInsertion(crate::store::autofocus::Frame),
    DesignMode(crate::store::design_mode::Pending),
}
struct PublishedSegment {
    commit: DomCommit,
    committed: transaction::PublicationCommitted,
    dialog: Option<crate::DomSourceDialogRequest>,
    focusing: Option<crate::DomSourceDialogFocusingRequest>,
    focus_delegate: Option<crate::DomSourceFocusDelegateRequest>,
    autofocus: Option<crate::DomSourceAutofocusInsertionRequest>,
    design_mode: Option<crate::DomSourceDesignModeRequest>,
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
        self.names.admit_limits(limits.names())?;
        if let Some(observation) = self.observation() {
            return observation.usage.admit_limits(limits);
        }
        let draft = &self.draft;
        admit(draft.ranges.len(), limits.ranges, DomResource::Ranges)?;
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
        draft.admit_dom_resources(limits)
    }
    pub(in crate::store) fn update_dom_limits(&mut self, limits: DomLimits) {
        self.names.replace_limits(limits.names());
        if self.observation().is_some() {
            return;
        }
        let draft = &mut self.draft;
        draft.dom_limits.event_recipients = limits.event_recipients;
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
            cleanup: self.failure,
            work: self.draft.source_work.load(Ordering::Relaxed),
            phases: self.phases,
            parent: None,
            phase: if self.failure.is_some() {
                crate::DomSourcePhase::PendingCleanup
            } else if self.draft.fullscreen_exit.is_some() {
                crate::DomSourcePhase::FullscreenExitHandoff
            } else if self.draft.pending_autofocus().is_some() {
                crate::DomSourcePhase::AutofocusInsertionService
            } else if matches!(self.draft.method.as_ref(), Some(crate::store::method::Frame::DesignMode(frame)) if frame.cursor.action() != surgeist_html::HtmlDesignModeAction::Complete)
            {
                crate::DomSourcePhase::DesignModeService
            } else if matches!(
                self.draft.method,
                Some(crate::store::method::Frame::Dialog(_))
            ) || self.draft.pending_dialog_hook().is_some()
            {
                crate::DomSourcePhase::DialogService
            } else {
                crate::DomSourcePhase::Staging
            },
        }
    }
    fn input_bytes(&self) -> Result<usize, DomError> {
        let mut bytes = self
            .command_storage
            .bytes
            .checked_add(self.custody_bytes)
            .and_then(|n| n.checked_add(self.canonical_aux_bytes))
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
        if let Some(binding) = self.owner_binding.as_ref().or_else(|| {
            self.meta_receipt
                .as_ref()
                .and_then(|saved| saved.binding.as_ref())
        }) {
            bytes = bytes
                .checked_add(binding.retained_bytes()?)
                .ok_or(DomError::RepresentationExhausted)?;
        }
        if let Some(Receipt::Inline {
            bytes: retained, ..
        }) = self
            .receipt
            .as_ref()
            .or_else(|| self.meta_receipt.as_ref().map(|saved| &saved.receipt))
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
        if let Some(order) = self
            .request
            .as_ref()
            .or_else(|| {
                self.meta_receipt
                    .as_ref()
                    .and_then(|saved| saved.request.as_ref())
            })
            .and_then(|r| r.sheet_order.as_ref())
        {
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
        if let Some(request) = self
            .meta_receipt
            .as_ref()
            .and_then(|saved| saved.request.as_ref())
            && !self
                .context
                .as_ref()
                .is_some_and(|(_, source)| Arc::ptr_eq(source, &request.source))
        {
            bytes = bytes
                .checked_add(request.source.retained_bytes()?)
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
            .command_storage
            .units
            .checked_add(self.custody_units)
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
        #[cfg(test)]
        if scale_metrics::ENABLED {
            let ledger = self.draft.custody_ledger();
            scale_metrics::select_source_ledger(
                ledger.measurement_identity(),
                ledger.total(),
                custody::Usage::new(self.draft.source_other_bytes, units),
            );
        }
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
        let mut reservation =
            self.draft
                .admit_source_command_headroom(dom, entry, self.phases, cancel)?;
        // A reread of the current segment cannot discard a tail already owed
        // by this original operation. The queue fence later refines the bound
        // after the authentic callback result and actual current guards.
        if let Some(owed) = &self.headroom {
            reservation.remaining_phases = reservation.remaining_phases.max(owed.remaining_phases);
        }
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
        allocated: &super::AllocationIds,
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
            Key::Existing(id) => id,
            Key::Created(i) => allocated.get(i).ok_or(DomError::UnavailableCreation)?,
        };
        let n = state.node(self.id.owner, id)?;
        let mut length = 0usize;
        for child in n.children() {
            self.draft.source_step(cancel)?;
            if let DomNodeKind::Text(s) = state.node(self.id.owner, *child)?.kind() {
                length = length
                    .checked_add(s.len())
                    .ok_or(DomError::RepresentationExhausted)?;
            }
        }
        let mut root = id;
        while let Some(parent) = state.node(self.id.owner, root)?.parent() {
            self.draft.source_step(cancel)?;
            root = parent;
        }
        let root_kind = match state.node(self.id.owner, root)?.kind() {
            DomNodeKind::Document(_) => DomStyleRootKind::Document,
            DomNodeKind::ShadowRoot(_) => DomStyleRootKind::ShadowRoot,
            _ => DomStyleRootKind::Detached,
        };
        let DomNodeKind::Document(metadata) =
            state.node(self.id.owner, n.document().node())?.kind()
        else {
            return Err(DomError::WrongDocument);
        };
        let DomNodeKind::Element(element) = n.kind() else {
            return Err(DomError::WrongNodeKind);
        };
        let base_url = self
            .draft
            .document_base_url(self.draft.node(frame.key)?.document(), cancel)?;
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
        #[cfg(test)]
        let mut copied = transaction::CopyObservation::default();
        for child in n.children() {
            self.draft.source_step(cancel)?;
            if let DomNodeKind::Text(s) = state.node(self.id.owner, *child)?.kind() {
                for unit in s.code_units() {
                    self.draft.source_step(cancel)?;
                    text.push(*unit);
                    #[cfg(test)]
                    copied.pushed::<u16>();
                }
            }
        }
        let source = formation::allocated_arc(DomStyleSource {
            operation: self.id,
            ordinal: match frame.invocation {
                Invocation::Unissued => self.draft.source_invocation,
                Invocation::Exposed(ordinal) => ordinal,
            },
            stamp,
            metadata: metadata.clone(),
            root_kind,
            element: id,
            document: n.document(),
            root,
            text: formation::allocated_arc(text),
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
            } else if let DomNodeKind::Element(element) = node.kind()
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
            if let Some(child) = node.children().first() {
                current = *child;
                continue;
            }
            loop {
                if current == source.root {
                    return Ok(());
                }
                let parent = state
                    .node(self.id.owner, current)?
                    .parent()
                    .ok_or(DomError::WrongNodeKind)?;
                let children = state.node(self.id.owner, parent)?.children();
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
        #[cfg(test)]
        let mut copied = transaction::CopyObservation::default();
        self.visit_sheet_order(state, source, cancel, |entry| {
            order.push(entry);
            #[cfg(test)]
            copied.pushed::<DomStyleSheetPlacement>();
            Ok(())
        })?;
        Ok(formation::allocated_arc(order))
    }
    fn publication(
        &mut self,
        dom: &mut Dom,
        incomplete: bool,
        target: PublicationTarget,
        checkpoint: Option<&checkpoint::Checkpoint>,
        cancel: &AtomicBool,
    ) -> Result<PublishedSegment, DomError> {
        self.publication_with_hosted(dom, incomplete, target, None, checkpoint, cancel)
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
        checkpoint: Option<&checkpoint::Checkpoint>,
        cancel: &AtomicBool,
    ) -> Result<PublishedSegment, crate::hosted::DomHostedError> {
        let segment = self.prepare_segment(dom, incomplete, target, hosted, checkpoint, cancel)?;
        if let Some(recording) = segment.prepared.inline_recording.as_ref() {
            recording.require_unreached(|| self.draft.source_step(cancel))?;
        }
        let permit = hosted
            .map(|(source, request, revision)| source.permit(request, revision, false, cancel))
            .transpose()?;
        let published = segment.publish(self, dom, checkpoint);
        if let Some(permit) = permit {
            permit.commit();
        }
        Ok(published)
    }
    fn prepare_segment(
        &mut self,
        dom: &mut Dom,
        incomplete: bool,
        target: PublicationTarget,
        hosted: Option<(
            &crate::DomHostedSource,
            &crate::hosted::Request,
            crate::DomHostedSourceRevision,
        )>,
        checkpoint: Option<&checkpoint::Checkpoint>,
        cancel: &AtomicBool,
    ) -> Result<SegmentPublication, crate::hosted::DomHostedError> {
        formation::SourceStorage::record_entry();
        let frame = match target {
            PublicationTarget::Style(frame) => Some(frame),
            _ => None,
        };
        // A proposed ordinal stays private through every fallible formation.
        // Only successful canonical publication/request exposure issues it.
        let invocation = frame.map(|frame| frame.invocation).or(match target {
            PublicationTarget::Dialog(pending) => pending.invocation,
            PublicationTarget::DialogFocusing(pending) => Some(pending.invocation),
            PublicationTarget::AutofocusInsertion(pending) => Some(pending.invocation),
            PublicationTarget::DesignMode(pending) => Some(pending.invocation),
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
        let mut prepared = self.draft.finish_segment(dom, cancel)?;
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
                Key::Existing(id) => *id,
                Key::Created(index) => prepared
                    .allocation_ids
                    .get(*index)
                    .ok_or(DomError::UnavailableCreation)?,
            });
            let globals_formation = prepared.formation.as_ref().map(formation::Formation::fork);
            let globals = crate::store::hosted::GlobalDocumentIndex::establish(
                dom.state.hosted_globals.as_deref(),
                *global,
                document,
                source,
                work,
                globals_formation.as_ref(),
                cancel,
            )?;
            prepared.set_hosted_globals(
                Some(globals),
                globals_formation,
                &dom.custody_registry,
                cancel,
            )?;
        }
        let changed = prepared.changed_from(&dom.state);
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
            let protected = dom.source_reserved_closing(cancel)?;
            dom.revision
                .0
                .checked_add(remaining)
                .and_then(|next| next.checked_add(dom.hosted_revisions))
                .and_then(|next| next.checked_add(protected))
                .ok_or(DomError::RevisionExhausted)?;
        }
        let outputs = self
            .outputs
            .checked_add(prepared.changes.len())
            .ok_or(DomError::RepresentationExhausted)?;
        if outputs
            .checked_add(self.meta_policy.as_ref().map_or(0, |p| p.outputs()))
            .and_then(|n| n.checked_add(usize::from(incomplete)))
            .is_none_or(|required| required > limits.outputs)
        {
            return Err(DomError::SourceOutputLimit.into());
        }
        let ids = prepared.allocation_ids.clone();
        let mut count = ids
            .len()
            .checked_sub(self.draft.phase_map.as_ref().expect("source map").ids.len())
            .ok_or(DomError::RepresentationExhausted)?;
        for change in &prepared.changes {
            self.draft.source_step(cancel)?;
            if matches!(change, DomChange::RangeCreated(_)) {
                count = count
                    .checked_add(1)
                    .ok_or(DomError::RepresentationExhausted)?;
            }
        }
        let mut identities = self
            .identities
            .checked_add(u64::try_from(count).map_err(|_| DomError::RepresentationExhausted)?)
            .ok_or(DomError::RepresentationExhausted)?;
        if identities > limits.identities {
            return Err(DomError::SourceIdentityLimit.into());
        }
        // The persistent assignment map preserves every actually published
        // creation, including one retired by a later phase. Public ticket rows
        // are output only at the final close, so materialize that output once.
        let mut historical = Vec::new();
        if !incomplete {
            formation::reserve(
                &mut historical,
                self.draft.tickets.len(),
                prepared.formation.as_ref(),
                cancel,
            )?;
            for key in &self.draft.tickets {
                self.draft.source_step(cancel)?;
                let Key::Created(k) = key else {
                    unreachable!("creation ticket")
                };
                historical.push(ids.get(*k));
            }
        }
        let dialog = if let PublicationTarget::Dialog(pending) = target {
            let subject = match pending.subject {
                Key::Existing(id) => id,
                Key::Created(index) => ids.get(index).ok_or(DomError::UnavailableCreation)?,
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
        let focusing = if let PublicationTarget::DialogFocusing(pending) = target
            && pending.delegate.is_none()
        {
            let subject = match pending.subject {
                Key::Existing(id) => id,
                Key::Created(index) => ids.get(index).ok_or(DomError::UnavailableCreation)?,
            };
            prepared.state.node(self.id.owner, subject)?;
            Some(crate::DomSourceDialogFocusingRequest {
                operation: self.id,
                subject,
                action: pending.action,
                occurrence: crate::source::watcher::HookOccurrence {
                    command: self.index,
                    ordinal: match pending.invocation {
                        Invocation::Unissued => self.draft.source_invocation,
                        Invocation::Exposed(ordinal) => ordinal,
                    },
                },
            })
        } else {
            None
        };
        let focus_delegate = if let PublicationTarget::DialogFocusing(pending) = target
            && let Some(delegate) = pending.delegate
        {
            let mapped = |key| -> Result<NodeId, DomError> {
                self.draft.source_step(cancel)?;
                let id = match key {
                    Key::Existing(id) => id,
                    Key::Created(index) => ids.get(index).ok_or(DomError::UnavailableCreation)?,
                };
                prepared.state.node(self.id.owner, id)?;
                Ok(id)
            };
            Some(crate::DomSourceFocusDelegateRequest {
                operation: self.id,
                subject: mapped(pending.subject)?,
                root: mapped(delegate.root)?,
                queried_node: mapped(delegate.queried_node)?,
                step: delegate.step,
                occurrence: crate::source::watcher::HookOccurrence {
                    command: self.index,
                    ordinal: match pending.invocation {
                        Invocation::Unissued => self.draft.source_invocation,
                        Invocation::Exposed(ordinal) => ordinal,
                    },
                },
            })
        } else {
            None
        };
        let autofocus = if let PublicationTarget::AutofocusInsertion(pending) = target {
            let subject = match pending.subject {
                Key::Existing(id) => id,
                Key::Created(index) => ids.get(index).ok_or(DomError::UnavailableCreation)?,
            };
            prepared.state.node(self.id.owner, subject)?;
            Some(crate::DomSourceAutofocusInsertionRequest {
                operation: self.id,
                subject,
                action: pending.cursor.action(),
                occurrence: crate::source::watcher::HookOccurrence {
                    command: self.index,
                    ordinal: match pending.invocation {
                        Invocation::Unissued => self.draft.source_invocation,
                        Invocation::Exposed(ordinal) => ordinal,
                    },
                },
            })
        } else {
            None
        };
        let design_mode = if let PublicationTarget::DesignMode(pending) = target {
            let mapped = |key| -> Result<NodeId, DomError> {
                self.draft.source_step(cancel)?;
                let id = match key {
                    Key::Existing(id) => id,
                    Key::Created(index) => ids.get(index).ok_or(DomError::UnavailableCreation)?,
                };
                prepared.state.node(self.id.owner, id)?;
                Ok(id)
            };
            let document = mapped(pending.document)?;
            if !matches!(
                prepared.state.node(self.id.owner, document)?.kind(),
                DomNodeKind::Document(_)
            ) {
                return Err(DomError::WrongNodeKind.into());
            }
            Some(crate::DomSourceDesignModeRequest {
                operation: self.id,
                document: DocumentId(document),
                step: pending.step,
                target: pending.target.map(mapped).transpose()?,
                range: pending.range,
                occurrence: crate::source::watcher::HookOccurrence {
                    command: self.index,
                    ordinal: match pending.invocation {
                        Invocation::Unissued => self.draft.source_invocation,
                        Invocation::Exposed(ordinal) => ordinal,
                    },
                },
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
            let state = self
                .draft
                .element(frame.key)?
                .language()
                .style_state()
                .expect("Style");
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
        // Newly formed source inputs already retain their byte/unit admission
        // in this candidate's Formation. Reused input is charged by context.
        self.draft.source_admit_units(0, cancel)?;
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
        let owner_custody = dom.source_owner_custody_bytes()?;
        let mut prepared = dom.prepare_publication(prepared, cancel)?;
        let rebase = self.draft.prepare_rebase(&mut prepared, cancel)?;
        let sharing = self
            .draft
            .prepare_sharing_publication(changed, checkpoint, cancel)?;
        Ok(SegmentPublication {
            prepared,
            rebase,
            sharing,
            owner_custody,
            issued_next,
            source,
            request,
            child,
            historical,
            outputs,
            identities,
            changed,
            dialog,
            focusing,
            focus_delegate,
            autofocus,
            design_mode,
        })
    }
}

// The original Source publication, prepared without callbacks or canonical writes.
// Its affine owner can join an admitted Runtime row before this exact commit.
struct SegmentPublication {
    prepared: PreparedPublication,
    rebase: transaction::PreparedRebase,
    sharing: Option<value_sharing::CloseReady>,
    owner_custody: usize,
    issued_next: Option<u64>,
    source: Option<Arc<DomStyleSource>>,
    request: Option<DomStyleRequest>,
    child: Option<hosted::Child>,
    historical: Vec<Option<NodeId>>,
    outputs: usize,
    identities: u64,
    changed: bool,
    dialog: Option<crate::DomSourceDialogRequest>,
    focusing: Option<crate::DomSourceDialogFocusingRequest>,
    focus_delegate: Option<crate::DomSourceFocusDelegateRequest>,
    autofocus: Option<crate::DomSourceAutofocusInsertionRequest>,
    design_mode: Option<crate::DomSourceDesignModeRequest>,
}
impl SegmentPublication {
    fn publish(
        self,
        op: &mut Operation,
        dom: &mut Dom,
        checkpoint: Option<&checkpoint::Checkpoint>,
    ) -> PublishedSegment {
        let Self {
            mut prepared,
            rebase,
            sharing,
            owner_custody,
            issued_next,
            source,
            request,
            child,
            historical,
            outputs,
            identities,
            changed,
            dialog,
            focusing,
            focus_delegate,
            autofocus,
            design_mode,
        } = self;
        op.draft
            .close_sharing_publication(&mut prepared, sharing, checkpoint);
        let crate::store::CommittedPublication { commit, committed } =
            dom.publish_prepared_with_proof(prepared);
        op.draft.commit_publication(rebase, &committed);
        op.draft.dispatch_retirements.clear();
        op.hosted_global.take();
        // Retirement only removes entries; capacity is retained and this cannot
        // overflow after the same table was admitted before publication.
        op.custody_bytes = owner_custody;
        // PreparedRebase installed only the actual phase's preadmitted queue
        // acknowledgment roots. Cancellation custody seals at the same fence.
        op.draft.acknowledge_cancelled_queues(&committed);
        op.draft.cancelled_images = None;
        if let Some(next) = issued_next {
            op.draft.source_invocation = next;
            if let Some(occurrence) = dialog.as_ref().and_then(|request| request.method()) {
                let Some(crate::store::method::Frame::Dialog(frame)) = op.draft.method.as_mut()
                else {
                    unreachable!("exposed actual Dialog");
                };
                frame.invocation = Invocation::Exposed(occurrence.ordinal);
            } else if let Some(ordinal) = focusing
                .as_ref()
                .map(|request| request.occurrence.ordinal)
                .or_else(|| {
                    focus_delegate
                        .as_ref()
                        .map(|request| request.occurrence.ordinal)
                })
            {
                let Some(crate::store::method::Frame::Dialog(frame)) = op.draft.method.as_mut()
                else {
                    unreachable!("called Dialog frame");
                };
                frame
                    .focusing
                    .as_mut()
                    .expect("actual focusing child")
                    .invocation = Invocation::Exposed(ordinal);
            } else if let Some(request) = &design_mode {
                let Some(crate::store::method::Frame::DesignMode(frame)) = op.draft.method.as_mut()
                else {
                    unreachable!("original designMode frame");
                };
                frame.invocation = Invocation::Exposed(request.occurrence.ordinal);
            } else {
                match &mut op
                    .draft
                    .effects
                    .last_mut()
                    .expect("exposed actual frame")
                    .effect
                {
                    select::Effect::AutofocusInsertion(actual) => {
                        actual.invocation = Invocation::Exposed(
                            autofocus
                                .as_ref()
                                .expect("issued insertion request")
                                .occurrence
                                .ordinal,
                        );
                    }
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
            op.context = Some((source.ordinal, source));
        }
        op.request = request;
        if let Some(child) = child {
            op.draft.source_invocation = child.request.occurrence;
            op.draft.hosted_pending = None;
            op.child = Some(child);
        }
        op.historical = historical;
        op.outputs = outputs;
        op.identities = identities;
        if changed && let Some(headroom) = &mut op.headroom {
            headroom.remaining_phases -= 1;
        }
        op.phases += u64::from(changed);
        op.published = true;
        op.meta_receipt = None;
        drop(op.draft.meta_recording.take());
        op.draft.meta_scope = None;
        PublishedSegment {
            commit,
            committed,
            dialog,
            focusing,
            focus_delegate,
            autofocus,
            design_mode,
        }
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
        match self.advance_execution(&mut Execution::disabled(), cancel) {
            Ok(progress) => Ok(progress),
            Err(AdvanceError::Dom(error)) => Err(error),
            Err(
                AdvanceError::Inline(_) | AdvanceError::Unwind(_) | AdvanceError::NativePrepared,
            ) => {
                unreachable!("no resolver in context-free advance")
            }
        }
    }
    fn advance_execution(
        &mut self,
        execution: &mut Execution<'_>,
        cancel: &AtomicBool,
    ) -> Result<DomSourceProgress, AdvanceError> {
        check_cancel(cancel)?;
        if self
            .dom
            .source_operation
            .as_ref()
            .is_some_and(|op| op.pending_meta.is_some())
        {
            return Err(DomError::Meta(crate::DomMetaError::DefaultStyleUnsupported).into());
        }
        if !self
            .dom
            .source_operation
            .as_ref()
            .is_some_and(|operation| operation.id == self.id)
        {
            return Err(DomError::InvalidSourceOperation.into());
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
            return Err(DomError::InvalidSourceOperation.into());
        }
        // Independent public capabilities may commit concurrently. Keep the
        // exact owner locked from receipt validation through canonical source
        // publication. An inline resolver consumes immutable input packets; it
        // does not grant event-observer reentry or access to this locked owner.
        if self
            .dom
            .source_operation
            .as_ref()
            .is_some_and(|op| op.observation().is_some())
        {
            return self.advance_dialog_observation(execution, cancel);
        }
        let root = self
            .dom
            .source_operation
            .as_ref()
            .filter(|_| execution.native.is_none())
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
                return Err(DomError::StyleReceiptMismatch.into());
            }
        }
        let custody = self.dom.source_owner_custody_bytes()?;
        let custody_units = self.dom.fullscreen_custody_units()?;
        let mut op = self
            .dom
            .source_operation
            .take()
            .ok_or(DomError::InvalidSourceOperation)?;
        op.custody_bytes = custody;
        op.custody_units = custody_units;
        let result = self.advance_operation(&mut op, execution, cancel);
        if self.complete || matches!(&result, Ok(DomSourceProgress::Complete(_))) {
            self.complete = true;
            drop(op);
            self.dom.source_operation = self.dom.source_parents.pop();
        } else {
            self.dom.source_operation = Some(op);
        }
        #[cfg(test)]
        self.dom.observe_source_lifecycle();
        result
    }
    fn advance_operation(
        &mut self,
        op: &mut Operation,
        execution: &mut Execution<'_>,
        cancel: &AtomicBool,
    ) -> Result<DomSourceProgress, AdvanceError> {
        op.prepare_meta_execution(execution, cancel)?;
        // A genuine observer may create the first binding after this Source
        // began. Refresh operational membership before any resumed mutation.
        op.refresh_inline_execution(self.dom, execution.resolver.is_some())?;
        if !op.event_exposed
            && let OperationRoot::DialogTask(task) = op.commands
        {
            op.refresh_admission(cancel)?;
            return op
                .observe_dialog_task(self.dom, task, cancel)
                .map_err(Into::into);
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
                        | Command::CreateRange { .. }
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
                created_ranges: Vec::new(),
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
                dialog_toggle_disposition: None,
                design_mode_range_disposition: None,
                fullscreen_exit_return: None,
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
        if op.draft.fullscreen_exit.is_some() {
            op.refresh_admission(cancel)?;
            let request = op.fullscreen_handoff_request(self.dom, cancel)?;
            op.event_exposed = true;
            return Ok(DomSourceProgress::Request(
                DomSourceRequest::FullscreenExitHandoff(request),
            ));
        }
        loop {
            // External task/runtime guards may have released storage since the
            // last receipt. Refresh before resumed methods or value formation.
            op.refresh_admission(cancel)?;
            if op.failure.is_some() && op.request_close.is_none() {
                return self.finish_source_failure(op, cancel).map_err(Into::into);
            }
            if op.request_close.is_some()
                && !op.request_close.as_ref().is_some_and(|call| call.nested)
                && let Some(progress) = self.advance_request_close(op, cancel)?
            {
                return Ok(progress);
            }
            op.draft.ensure_sharing(&self.dom.value_sharing, cancel)?;
            if op.published
                && !op.started
                && !op.cancelled
                && !op.command_admitted
                && op.index < op.commands.len()
            {
                op.admit_current_command(self.dom, cancel)?;
            }
            let input_bytes = op.input_bytes()?;
            let checkpoint = op.draft.source_checkpoint(input_bytes, 0, cancel)?;
            if op.meta_policy.is_some() {
                op.draft.meta_scope = Some((op.id, meta::SubjectScope::new(&checkpoint)));
            }
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
                } else if matches!(
                    op.draft.method,
                    Some(crate::store::method::Frame::FullscreenExit(_))
                ) {
                    let Command::ExitFullscreen { document } = &op.commands[op.index] else {
                        return Err(DomError::InvalidSourceOperation);
                    };
                    let document = op.draft.resolve(document.clone())?;
                    op.draft.exit_fullscreen_boundary(document, true, cancel)
                } else if let Some(crate::store::method::Frame::DesignMode(frame)) =
                    op.draft.method.as_ref()
                {
                    if frame.cursor.action() == surgeist_html::HtmlDesignModeAction::Complete {
                        Ok(())
                    } else {
                        let document = frame.document;
                        let Command::SetDesignMode { value, .. } = &op.commands[op.index] else {
                            return Err(DomError::InvalidSourceOperation);
                        };
                        op.draft.design_mode_boundary(document, value, cancel)
                    }
                } else if let Some(crate::store::method::Frame::Dialog(frame)) =
                    op.draft.method.as_ref()
                {
                    let subject = frame.subject;
                    let input = if op.request_close.as_ref().is_some_and(|call| call.nested) {
                        crate::store::dialog::MethodInput::Close(op.request_close_result.as_ref())
                    } else {
                        match &op.commands[op.index] {
                            Command::ShowModalDialog { .. } => {
                                crate::store::dialog::MethodInput::ShowModal
                            }
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
                        .dialog_method_boundary(subject, input, self.dom.limits, cancel)
                } else if op.started {
                    op.draft.run_effects(self.dom.limits, cancel)
                } else if op.index < op.commands.len() && !op.cancelled {
                    op.apply_command(self.dom.limits, cancel)
                } else {
                    Ok(())
                }
            })();
            match result {
                Err(DomError::FullscreenExitHandoffRequired) => {
                    if !op.command_admitted {
                        checkpoint.restore(&mut op.draft);
                        op.restore_external_receipt(external_applied);
                        op.started = started;
                        op.admit_current_command(self.dom, cancel)?;
                        continue;
                    }
                    let request = match op.fullscreen_handoff_request(self.dom, cancel) {
                        Ok(request) => request,
                        Err(error) => {
                            checkpoint.restore(&mut op.draft);
                            op.started = started;
                            return Err(error.into());
                        }
                    };
                    op.started = true;
                    // An exposed handoff is retained even before any tree write.
                    op.event_exposed = true;
                    drop(checkpoint);
                    return Ok(DomSourceProgress::Request(
                        DomSourceRequest::FullscreenExitHandoff(request),
                    ));
                }
                Err(DomError::DesignModeProcessingUnavailable { step }) => {
                    let pending = op
                        .draft
                        .pending_design_mode()
                        .filter(|pending| pending.step == step)
                        .ok_or(DomError::InvalidSourceOperation)?;
                    if !op.command_admitted {
                        checkpoint.restore(&mut op.draft);
                        op.restore_external_receipt(external_applied);
                        op.started = started;
                        op.admit_current_command(self.dom, cancel)?;
                        continue;
                    }
                    op.started = true;
                    let published = match op.publication_inline(
                        self.dom,
                        PublicationTarget::DesignMode(pending),
                        Some(meta::Disposition::DesignMode),
                        Some(&checkpoint),
                        execution,
                        cancel,
                    ) {
                        Ok(published) => published,
                        Err(error) => {
                            if matches!(error, AdvanceError::NativePrepared) {
                                op.park_meta_checkpoint(checkpoint);
                                return Err(error);
                            }
                            checkpoint.restore(&mut op.draft);
                            op.restore_external_receipt(external_applied);
                            op.started = started;
                            return Err(error);
                        }
                    };
                    drop(checkpoint);
                    return Ok(DomSourceProgress::Request(DomSourceRequest::DesignMode(
                        published
                            .design_mode
                            .expect("published original designMode requirement"),
                    )));
                }
                Err(DomError::AutofocusInsertionUnavailable { action }) => {
                    let pending = op
                        .draft
                        .pending_autofocus()
                        .filter(|p| p.cursor.action() == action)
                        .ok_or(DomError::InvalidSourceOperation)?;
                    if !op.command_admitted {
                        checkpoint.restore(&mut op.draft);
                        op.restore_external_receipt(external_applied);
                        op.started = started;
                        op.admit_current_command(self.dom, cancel)?;
                        continue;
                    }
                    op.started = true;
                    let published = match op.publication_inline(
                        self.dom,
                        PublicationTarget::AutofocusInsertion(pending),
                        Some(meta::Disposition::Autofocus),
                        Some(&checkpoint),
                        execution,
                        cancel,
                    ) {
                        Ok(published) => published,
                        Err(error) => {
                            if matches!(error, AdvanceError::NativePrepared) {
                                op.park_meta_checkpoint(checkpoint);
                                return Err(error);
                            }
                            checkpoint.restore(&mut op.draft);
                            op.restore_external_receipt(external_applied);
                            op.started = started;
                            return Err(error);
                        }
                    };
                    drop(checkpoint);
                    op.receipt = None;
                    return Ok(DomSourceProgress::Request(
                        DomSourceRequest::AutofocusInsertion(
                            published.autofocus.expect("formed insertion request"),
                        ),
                    ));
                }
                Err(DomError::DialogFocusingUnavailable { action }) => {
                    let pending = op
                        .draft
                        .pending_focusing()
                        .filter(|pending| pending.action == action)
                        .ok_or(DomError::InvalidSourceOperation)?;
                    if !op.command_admitted {
                        checkpoint.restore(&mut op.draft);
                        op.restore_external_receipt(external_applied);
                        op.started = started;
                        op.admit_current_command(self.dom, cancel)?;
                        continue;
                    }
                    op.started = true;
                    let published = match op.publication_inline(
                        self.dom,
                        PublicationTarget::DialogFocusing(pending),
                        Some(meta::Disposition::Focusing),
                        Some(&checkpoint),
                        execution,
                        cancel,
                    ) {
                        Ok(published) => published,
                        Err(error) => {
                            if matches!(error, AdvanceError::NativePrepared) {
                                op.park_meta_checkpoint(checkpoint);
                                return Err(error);
                            }
                            checkpoint.restore(&mut op.draft);
                            op.restore_external_receipt(external_applied);
                            op.started = started;
                            return Err(error);
                        }
                    };
                    drop(checkpoint);
                    op.receipt = None;
                    return Ok(DomSourceProgress::Request(match published.focus_delegate {
                        Some(request) => DomSourceRequest::FocusDelegate(request),
                        None => DomSourceRequest::DialogFocusing(
                            published
                                .focusing
                                .expect("actual called request formed before publication"),
                        ),
                    }));
                }
                Err(DomError::NeedsEventfulOperation {
                    subject: Some(subject),
                    action,
                }) => {
                    let observed = op.observe_dialog_method(self.dom, subject, action, cancel);
                    if observed.is_err() {
                        checkpoint.restore(&mut op.draft);
                        op.restore_external_receipt(external_applied);
                    }
                    return observed.map_err(Into::into);
                }
                Err(DomError::DialogProcessingUnavailable {
                    action: surgeist_html::HtmlDialogAction::AssertCloseWatcherPresent,
                }) if op.is_show_modal_method() => {
                    // AddOpen and every reached hook finish before this internal
                    // boundary. Rollback is allowed only before this publication.
                    if let Err(error) = op.publication_inline(
                        self.dom,
                        PublicationTarget::None,
                        None,
                        Some(&checkpoint),
                        execution,
                        cancel,
                    ) {
                        if matches!(error, AdvanceError::NativePrepared) {
                            op.park_meta_checkpoint(checkpoint);
                            return Err(error);
                        }
                        checkpoint.restore(&mut op.draft);
                        op.restore_external_receipt(external_applied);
                        op.started = started;
                        return Err(error);
                    }
                    drop(checkpoint);
                    op.started = true;
                    op.receipt = None;
                    op.observe_show_modal_assertion(self.dom, cancel)?;
                    continue;
                }
                Err(DomError::DialogProcessingUnavailable {
                    action:
                        surgeist_html::HtmlDialogAction::RequestWatcherClose {
                            require_history_action_activation,
                        },
                }) => {
                    let started_call = self.begin_request_close(
                        op,
                        require_history_action_activation,
                        &checkpoint,
                        cancel,
                    );
                    if let Err(error) = started_call {
                        checkpoint.restore(&mut op.draft);
                        op.restore_external_receipt(external_applied);
                        op.started = started;
                        return Err(error.into());
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
                    let published = match op.publication_inline(
                        self.dom,
                        PublicationTarget::Dialog(pending),
                        Some(meta::Disposition::Dialog),
                        Some(&checkpoint),
                        execution,
                        cancel,
                    ) {
                        Ok(published) => published,
                        Err(error) => {
                            if matches!(error, AdvanceError::NativePrepared) {
                                op.park_meta_checkpoint(checkpoint);
                                return Err(error);
                            }
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
                    if let Err(e) = op.publication_inline(
                        self.dom,
                        PublicationTarget::Style(frame),
                        Some(meta::Disposition::Style),
                        Some(&checkpoint),
                        execution,
                        cancel,
                    ) {
                        if matches!(e, AdvanceError::NativePrepared) {
                            op.park_meta_checkpoint(checkpoint);
                            return Err(e);
                        }
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
                    if let Err(error) = op.publication_inline(
                        self.dom,
                        PublicationTarget::None,
                        Some(meta::Disposition::Hosted),
                        Some(&checkpoint),
                        execution,
                        cancel,
                    ) {
                        if matches!(error, AdvanceError::NativePrepared) {
                            op.park_meta_checkpoint(checkpoint);
                            return Err(error);
                        }
                        checkpoint.restore(&mut op.draft);
                        op.restore_external_receipt(external_applied);
                        op.started = started;
                        return Err(error);
                    }
                    // The preceding external receipt is now canonical. Keep
                    // it on failed publication, but never replay it as child
                    // work after this successful phase transition.
                    op.receipt = None;
                    op.owner_binding = None;
                    let child = op.child.as_ref().ok_or(DomError::HostedSourceUnavailable)?;
                    return Ok(DomSourceProgress::Request(DomSourceRequest::HostedChild(
                        child.request.clone(),
                    )));
                }
                Err(e) => {
                    checkpoint.restore(&mut op.draft);
                    op.restore_external_receipt(external_applied);
                    op.started = started;
                    return Err(e.into());
                }
                Ok(()) => {
                    if op
                        .draft
                        .method
                        .as_ref()
                        .is_some_and(crate::store::method::Frame::is_running)
                    {
                        continue;
                    }
                    if op.request_close.as_ref().is_some_and(|call| call.nested) {
                        let call = op.request_close.as_mut().expect("nested original call");
                        call.nested = false;
                        call.cursor
                            .reply(surgeist_html::HtmlCloseWatcherRequestReply::Applied)
                            .map_err(DomError::CloseWatcherProtocol)?;
                        op.draft.source_set_method(
                            Some(crate::store::method::Frame::Dialog(
                                call.outer.before_focusing()?,
                            )),
                            cancel,
                        )?;
                        op.request_close_result = None;
                        op.receipt = None;
                        continue;
                    }
                    let completed_method = matches!(
                        op.draft.method,
                        Some(
                            crate::store::method::Frame::DesignMode(_)
                                | crate::store::method::Frame::FullscreenExit(_)
                        )
                    );
                    // A local setter has already finished its actual writes.
                    // Preserve that exact completed draft/result across final
                    // admission refusal, instead of restoring its entry cursor.
                    let checkpoint = if completed_method {
                        drop(checkpoint);
                        let input_bytes = op.input_bytes()?;
                        op.draft.source_checkpoint(input_bytes, 0, cancel)?
                    } else {
                        checkpoint
                    };
                    // The real current command has drained. Excess conservative
                    // capacity is released while one actual final close remains.
                    if let Some(headroom) = &mut op.headroom {
                        headroom.remaining_phases = headroom::FINAL_PUBLICATION;
                    }

                    if execution.native.is_some() {
                        op.save_meta_receipt();
                    }
                    if !matches!(
                        op.receipt,
                        Some(
                            Receipt::Watcher { .. }
                                | Receipt::Focus { .. }
                                | Receipt::CloseWatcher { .. }
                        )
                    ) {
                        op.receipt = None;
                        op.request = None;
                        op.owner_binding = None;
                    }
                    if op.index < op.commands.len() && (!op.cancelled || op.started) {
                        let next_index = op.index + 1;
                        if next_index < op.commands.len() && completed_method {
                            #[cfg(test)]
                            accounting_tests::cut_method_clear_work(&mut op.draft);
                            if let Err(error) = op.draft.source_set_method(None, cancel) {
                                checkpoint.restore(&mut op.draft);
                                op.restore_external_receipt(external_applied);
                                op.started = started;
                                op.index = command_index;
                                return Err(error.into());
                            }
                        }
                        // All fallible continuation cleanup precedes moving
                        // the nonjournaled Operation command cursor.
                        op.index = next_index;
                        op.started = false;
                        op.command_admitted = false;
                        op.context = None;
                    }
                    if op.index == op.commands.len() || op.cancelled {
                        // Complete fallible installation output before touching
                        // tentative Select finalization. These are distinct
                        // named roots, not two completion phases of one root.
                        let installation_output = op.installation_output(self.dom, cancel)?;
                        let select_finished = op.select.is_some() && (!op.cancelled || op.started);
                        if select_finished {
                            let select = op.select.as_ref().expect("actual Select root");
                            if !matches!(op.draft.native_tasks.first(), Some(select::Pending::Update {serial:Some(serial),target}) if serial==select.operation.serial && op.draft.resolve(select.target.into()).is_ok_and(|key|key==target))
                            {
                                return Err(DomError::InvalidNativeTask.into());
                            }
                            op.draft.queue_native_remove(0, 1, cancel)?;
                        }
                        // FIFO removal is tentative until every final output
                        // and canonical publication succeeds. A refusal restores
                        // the Draft checkpoint; no independent finished flag may
                        // outlive that rollback and suppress removal on retry.
                        let commit = match op.publication_inline(
                            self.dom,
                            PublicationTarget::None,
                            Some(meta::Disposition::Complete {
                                installation: installation_output,
                                select: select_finished,
                            }),
                            Some(&checkpoint),
                            execution,
                            cancel,
                        ) {
                            Ok(published) => published.commit,
                            Err(error) => {
                                if matches!(error, AdvanceError::NativePrepared) {
                                    op.park_meta_checkpoint(checkpoint);
                                    return Err(error);
                                }
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
                        if select_finished {
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
                            fullscreen_exit_return: match op.draft.method.as_ref() {
                                Some(crate::store::method::Frame::FullscreenExit(result)) => {
                                    *result
                                }
                                _ => None,
                            },
                            design_mode_range_disposition: match op.draft.method.as_ref() {
                                Some(crate::store::method::Frame::DesignMode(frame)) => frame.range,
                                _ => None,
                            },
                            // A task enters this Segment only after its actual
                            // accepted observation rebases through tracker clear.
                            // Retired targets and pre-dispatch cancellation finish
                            // directly from Observed, without reaching this path.
                            dialog_toggle_disposition: match op.commands {
                                OperationRoot::DialogTask(crate::DomDialogTaskInfo::Toggle(_)) => {
                                    Some(crate::DomDialogToggleTaskDisposition::TrackerCleared)
                                }
                                _ => None,
                            },
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
        let mut bytes = source_bytes
            .checked_add(observation.policies.retained_bytes()?)
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
        if let Some(existing) = self.dom.style_cssom_roots.get(root.root()) {
            #[cfg(test)]
            crate::store::style_custody_tests::observe(
                crate::store::style_custody_tests::Site::BindingLookup,
            );
            op.draft.source_step(cancel)?;
            return if existing.same_owner(root) {
                Ok(())
            } else {
                Err(DomError::StyleReceiptMismatch)
            };
        }
        let document = self.dom.state.node(self.dom.owner, root.root())?.document();
        root.snapshot()
            .map_err(DomStyleCssomRootError::into_dom_error)?;
        let replacement = self.dom.style_cssom_roots.prepare_insert(
            root,
            document,
            |usage| op.draft.admit_custody_usage(usage, cancel),
            || op.draft.source_step(cancel),
        )?;
        // The shared registry already retains and admits every old/new path.
        // Transfer the actual owner only after the complete candidate succeeds.
        self.dom.style_cssom_roots = replacement;
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
        // This allocation-free access still consumes the current operation's
        // original work allowance for its exact selected owner.
        op.draft.source_step(&AtomicBool::new(false))?;
        self.dom
            .style_cssom_roots
            .get(
                expected
                    .cssom_root()
                    .ok_or(DomError::StyleReceiptMismatch)?,
            )
            .map(|root| {
                #[cfg(test)]
                crate::store::style_custody_tests::observe(
                    crate::store::style_custody_tests::Site::LeaseLookup,
                );
                root.duplicate()
            })
            .ok_or(DomError::StyleReceiptMismatch)
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
                origin: Some(crate::DomStyleNotificationOrigin::NativeSource(op.id)),
                parsed: None,
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
        if let Some(Receipt::Focus {
            publication,
            applied,
        }) = &self.receipt
        {
            if *applied {
                return Ok(());
            }
            let request = publication.request.clone();
            self.apply_focus(&request, cancel)?;
            if let Some(Receipt::Focus { applied, .. }) = &mut self.receipt {
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
            Receipt::Watcher { .. } | Receipt::Focus { .. } | Receipt::CloseWatcher { .. } => {
                unreachable!("external receipt handled above")
            }
            Receipt::Inline { observation, .. } => Reply::InlineAllowed(observation.allowed),
            Receipt::Cssom { revision, sheet } => {
                let mut state = self.draft.style_state_mut(frame.key, cancel)?;
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
                state.commit(cancel)?;
                Reply::Applied
            }
        };
        frame
            .cursor
            .reply(reply)
            .map_err(|_| DomError::StyleProtocol)?;
        self.draft.source_set_style_frame(frame, cancel)
    }
}

struct SelectRoot {
    operation: crate::DomNativeTaskId,
    target: NodeId,
}
impl Operation {
    pub(in crate::store) fn allows_select_checkpoint(&self, serial: u64, target: NodeId) -> bool {
        self.select
            .as_ref()
            .is_some_and(|s| s.operation.serial == serial && s.target == target)
    }
}
impl Dom {
    pub fn begin_select_completion(
        &mut self,
        task: &crate::DomSelectTask,
        checkpoint: crate::DomNativeCheckpointLimits,
        limits: DomSourceLimits,
        cancel: &AtomicBool,
    ) -> Result<crate::DomSelectCompletionStart<'_>, DomError> {
        if let select::CompletionAdmission::Skipped(completion) =
            self.admit_select_completion(task, checkpoint, cancel)?
        {
            return Ok(crate::DomSelectCompletionStart::Skipped(completion));
        }
        let mut lease = self.begin_source_root(
            OperationRoot::SelectCompletion(task.select()),
            formation::allocated_arc(BatchIdentity),
            limits,
            cancel,
        )?;
        let op = lease.dom.source_operation.as_mut().expect("new operation");
        op.draft.select_scratch_limit = checkpoint.scratch;
        op.draft.select_work_limit = checkpoint.work;
        op.select = Some(SelectRoot {
            operation: task.operation(),
            target: task.select(),
        });
        lease.select_authorization = Some(task.authorization.clone());
        Ok(crate::DomSelectCompletionStart::Source(lease))
    }
    pub fn resume_select_completion(
        &mut self,
        id: DomSourceOperationId,
        task: &crate::DomSelectTask,
        limits: DomSourceLimits,
        cancel: &AtomicBool,
    ) -> Result<DomSourceLease<'_>, DomError> {
        self.validate_pending_select_task(task)?;
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
            let mut operation = formation::allocated_box(Operation {
                id,
                entry_scope,
                limits: original_limits,
                names: staged.draft.names.clone(),
                draft: dialog::OperationDraft::Segment(staged.draft),
                inline_source: None,
                meta_policy: None,
                pending_meta: None,
                meta_receipt: None,
                commands: OperationRoot::NativeInstallation,
                index: 0,
                started: false,
                published: false,
                event_exposed: false,
                event_work: std::sync::Weak::new(),
                event_custody: None,
                dialog_queue: None,
                request_close: None,
                request_close_result: None,
                failure: None,
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
                command_storage: storage::Payload::default(),
                custody_bytes: self.source_owner_custody_bytes().map_err(Error::Target)?,
                custody_units: self.fullscreen_custody_units().map_err(Error::Target)?,
                canonical_aux_bytes,
                #[cfg(test)]
                _measurement_lifetime: scale_metrics::SourceOwnerLifetime(id),
            });
            operation.refresh_admission(cancel).map_err(Error::Target)?;
            Ok::<_, Error>(operation)
        })();
        let operation = match formed {
            Ok(operation) => operation,
            Err(error) => {
                if nested {
                    self.source_operation = self.source_parents.pop();
                    #[cfg(test)]
                    self.observe_source_lifecycle();
                }
                return Err(error);
            }
        };
        self.next_source_operation += 1; // Checked before any nested parent transfer.
        self.source_operation = Some(operation);
        #[cfg(test)]
        self.observe_source_lifecycle();
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
        let ids = self.draft.allocation_ids(dom, formation.as_ref(), cancel)?;
        debug_assert!(install.bindings.capacity() >= install.rows.len());
        // Each genuine binding is filled at most once. Source failure keeps
        // both partial progress and the already admitted immutable native bank.
        // No binding ID or native close work is fabricated or replayed.
        for index in install.bindings.len()..install.rows.len() {
            self.draft.source_step(cancel)?;
            let row = &install.rows[index];
            let id = match row.key {
                Key::Existing(id) => id,
                Key::Created(index) => ids.get(index).ok_or(DomError::UnavailableCreation)?,
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
        drop(ids);
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
    /// `dispatch_work` admits one caller-owned header advance, including the
    /// final end check. It is per-call, finite, and zero is exhausted. Original
    /// activated-entry scans retain their cumulative processing work on retry.
    pub fn finish_style_notification(
        &mut self,
        token: &mut DomStyleNotificationToken,
        delivery: crate::DomStyleNotificationDelivery<'_>,
        dispatch_work: u64,
        cancel: &AtomicBool,
    ) -> Result<DomStyleNotificationCompletion, DomError> {
        self.require_source_idle()?;
        check_cancel(cancel)?;
        if token.owner() != self.owner {
            return Err(DomError::ForeignOwner);
        }
        if token.complete {
            return Err(DomError::StyleProtocol);
        }
        let parsed = matches!(
            token.origin(),
            crate::DomStyleNotificationOrigin::ParsedOccurrence { .. }
        );
        match (&delivery, parsed) {
            (crate::DomStyleNotificationDelivery::Native, false) => {}
            (crate::DomStyleNotificationDelivery::Parsed(input), true) if input.is_some() => {}
            _ => return Err(DomError::StyleProtocol),
        }
        let target = token.target();
        if self.state.node(self.owner, target).is_err() {
            if parsed {
                return Err(DomError::StaleNode);
            }
            token.complete = true;
            return Ok(DomStyleNotificationCompletion::TargetDestroyed);
        }
        // Every actual header advance, including end, consumes caller work.
        // Original activated-inventory reads keep their own cumulative Control.
        let mut cursor = self.parsed_html_dispatch_cursor();
        let mut spent = 0u64;
        loop {
            if spent == dispatch_work {
                return Err(DomError::WorkLimit {
                    resource: DomResource::ParsedAssociationDispatchWork,
                    limit: dispatch_work,
                });
            }
            spent += 1;
            let Some(entry) = cursor.advance(cancel)? else {
                break;
            };
            if entry.requirement(cancel)?.is_some() {
                return Err(DomError::ParsedAssociationSynchronousPending {
                    pending: entry.contribution(),
                });
            }
        }
        if matches!(
            token.origin(),
            crate::DomStyleNotificationOrigin::ParsedOccurrence { .. }
        ) {
            let crate::DomStyleNotificationDelivery::Parsed(input) = delivery else {
                unreachable!()
            };
            return self.finish_parsed_style_notification(token, input, cancel);
        }
        let mut draft = Draft::new(
            self,
            formation::allocated_arc(crate::batch::BatchIdentity),
            cancel,
        )?;
        let key = draft.resolve(target.into())?;
        draft
            .element(key)?
            .language()
            .style_state()
            .ok_or(DomError::WrongNodeKind)?;
        // Native Style's parser-created first conjunct is false, independently
        // of media/environment. The HTML cursor's script-blocking removal is
        // therefore inapplicable; unconditional final unblock is still real.
        let document = draft.node(key)?.document();
        draft.relations_retain(
            document,
            transaction::RelationField::RenderBlocking,
            |candidate| candidate != key,
            cancel,
        )?;
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
    use std::cell::Cell;
    thread_local! {
        static CUT_METHOD_CLEAR_WORK: Cell<bool> = const { Cell::new(false) };
    }
    pub(super) fn cut_method_clear_work(draft: &mut Draft) {
        if CUT_METHOD_CLEAR_WORK.replace(false) {
            // Deterministically reach the original real gate. This changes
            // only the finite policy, never the gate's returned result.
            draft.source_limits.as_mut().unwrap().work = draft.source_work.load(Ordering::Relaxed);
        }
    }

    #[test]
    fn completed_method_clear_refusal_keeps_next_command_for_same_operation_retry() {
        let cancel = AtomicBool::new(false);
        let mut dom = Dom::new(
            DomLimits::new(
                16,
                100_000,
                64,
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
        let target = setup.create_element_ns(&document, crate::Namespace::html(), "div");
        let done = dom.apply(setup, &cancel).unwrap();
        let document = done.document(&document).unwrap();
        let target = done.created(&target).unwrap().unwrap();
        let mut commands = DomBatch::new();
        commands.set_design_mode(document, "off");
        commands.set_attribute(target, "data-next", "once");
        let mut lease = dom
            .begin_source_commands(commands, limits(10_000_000), &cancel)
            .unwrap();
        let operation = lease.operation();
        CUT_METHOD_CLEAR_WORK.set(true);
        assert!(matches!(
            lease.advance(&cancel),
            Err(DomError::SourceWorkLimit { .. })
        ));
        let spent = lease.status().spent_work();
        let original_work = lease
            .dom
            .source_operation
            .as_ref()
            .unwrap()
            .draft
            .source_work
            .source_arc()
            .clone();
        let op = lease.dom.source_operation.as_ref().unwrap();
        assert_eq!(op.index, 0);
        assert!(matches!(
            op.draft.method,
            Some(method::Frame::DesignMode(_))
        ));
        assert!(
            lease
                .snapshot()
                .node(target)
                .unwrap()
                .as_element()
                .unwrap()
                .attribute(&"data-next".into())
                .is_none()
        );
        let mut lease = lease
            .dom
            .resume_source_edit(operation, limits(10_000_000), &cancel)
            .unwrap();
        assert_eq!(lease.operation(), operation);
        assert!(lease.status().spent_work() >= spent);
        assert!(Arc::ptr_eq(
            &original_work,
            lease
                .dom
                .source_operation
                .as_ref()
                .unwrap()
                .draft
                .source_work
                .source_arc()
        ));
        let DomSourceProgress::Complete(done) = lease.advance(&cancel).unwrap() else {
            panic!("actual completion after policy raise")
        };
        assert_eq!(
            done.commit
                .changes()
                .changes()
                .iter()
                .filter(|change| matches!(change, DomChange::Attributes(id) if *id==target))
                .count(),
            1
        );
        assert_eq!(
            lease
                .snapshot()
                .node(target)
                .unwrap()
                .as_element()
                .unwrap()
                .attribute(&"data-next".into()),
            Some(&DocumentString::from("once"))
        );
    }
    fn limits(work: u64) -> DomSourceLimits {
        DomSourceLimits::new(16_000_000, 1_000_000, work, 64, 4096, 4096, 1)
    }
    #[test]
    fn xml_creator_source_work_is_cumulative_and_exact_remaining_budget_refuses_without_reissue() {
        let cancel = AtomicBool::new(false);
        let mut dom = Dom::new(
            DomLimits::new(
                32,
                100_000,
                64,
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
        let associated = setup.create_document();
        let done = dom.apply(setup, &cancel).unwrap();
        let associated = done.document(&associated).unwrap();
        let before = dom.snapshot();
        let mut commands = DomBatch::new();
        let created =
            commands.create_xml_document(associated, crate::Namespace::svg(), "Root", None);
        let mut lease = dom
            .begin_source_commands(commands, limits(100_000_000), &cancel)
            .unwrap();
        let operation = lease.operation();
        let spent = lease.status().spent_work();
        let work = lease
            .dom
            .source_operation
            .as_ref()
            .unwrap()
            .draft
            .source_work
            .source_arc()
            .clone();
        // Use this original owner's actual spent frontier. A separately built
        // owner can have a different private registry path and work receipt.
        lease
            .dom
            .source_operation
            .as_mut()
            .unwrap()
            .draft
            .source_limits
            .as_mut()
            .unwrap()
            .work = spent;
        assert_eq!(
            lease.advance(&cancel).unwrap_err(),
            DomError::SourceWorkLimit { limit: spent }
        );
        assert_eq!(lease.status().spent_work(), spent);
        assert_eq!(lease.snapshot().stamp(), before.stamp());
        let mut lease = lease
            .dom
            .resume_source_edit(operation, limits(100_000_000), &cancel)
            .unwrap();
        assert_eq!(lease.operation(), operation);
        assert!(lease.status().spent_work() >= spent);
        assert!(Arc::ptr_eq(
            &work,
            lease
                .dom
                .source_operation
                .as_ref()
                .unwrap()
                .draft
                .source_work
                .source_arc()
        ));

        let DomSourceProgress::Complete(done) = lease.advance(&cancel).unwrap() else {
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
        assert!(work.load(Ordering::Relaxed) > spent);
    }
    #[test]
    fn xml_style_root_reserves_final_metadata_and_ticket_revision_before_first_exposure() {
        for (remaining, success) in [(6, false), (7, true)] {
            let cancel = AtomicBool::new(false);
            let mut dom = Dom::new(
                DomLimits::new(
                    32,
                    100_000,
                    64,
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
        let mut dom = Dom::new(
            DomLimits::new(
                64,
                100_000,
                64,
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
        let mut dom = Dom::new(
            DomLimits::new(
                32,
                100_000,
                64,
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
                    formation::allocated_arc(Vec::new()),
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
        let mut dom = Dom::new(
            DomLimits::new(
                32,
                100_000,
                64,
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
        let mut dom = Dom::new(
            DomLimits::new(
                32,
                100_000,
                64,
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
        let mut dom = Dom::new(
            DomLimits::new(
                16,
                1_000_000,
                32,
                0,
                0,
                8,
                1_000_000,
                crate::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
            )
            .unwrap(),
        )
        .unwrap();
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
        let mut dom = Dom::new(
            DomLimits::new(
                16,
                1_000_000,
                32,
                0,
                0,
                8,
                1_000_000,
                crate::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
            )
            .unwrap(),
        )
        .unwrap();
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
        let mut dom = Dom::new(
            DomLimits::new(
                64,
                100_000,
                128,
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
        let domain = DomLimits::new(
            64,
            100_000,
            128,
            0,
            0,
            8,
            100_000,
            crate::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
        )
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
        let mut dom = Dom::new(
            DomLimits::new(
                64,
                100_000,
                128,
                0,
                0,
                8,
                100_000,
                crate::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
            )
            .unwrap(),
        )
        .unwrap();
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
    #[test]
    fn changed_input_field_keeps_unchanged_backing_and_prices_only_the_selected_native_body() {
        let cancel = AtomicBool::new(false);
        let mut dom = Dom::new(
            DomLimits::new(
                16,
                100_000,
                64,
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
        let input = setup.create_element_ns(&document, crate::Namespace::html(), "input");
        setup.set_input_value(&input, "stored unchanged input");
        let done = dom.apply(setup, &cancel).unwrap();
        let input = done.created(&input).unwrap().unwrap();
        let before = dom.snapshot();
        let original = before
            .node(input)
            .unwrap()
            .as_element()
            .unwrap()
            .input_state()
            .unwrap();
        assert!(!original.flags().checkedness());
        let original_buffer = original.current_value().shared_code_units().clone();
        assert!(matches!(
            original.image_history(),
            surgeist_html::HtmlInputImageHistory::NeverEntered
        ));
        // Independent raw capacity oracle for this selected Input body. Name,
        // attributes and every other feature remain original child aliases.
        let native_units = original.current_value().shared_code_units().capacity()
            + original
                .custom_validity_message()
                .shared_code_units()
                .capacity();
        assert!(native_units > 0);
        let original_element = before.node(input).unwrap().as_element().unwrap();
        let original_native = original_element.input_payload().unwrap().clone();
        let mut commands = DomBatch::new();
        commands.set_input_checked(input, true);
        let lease = dom
            .begin_source_commands(commands, limits(10_000_000), &cancel)
            .unwrap();
        let actual_limits = lease.dom.limits;
        let op = lease.dom.source_operation.as_mut().unwrap();
        let baseline = op.draft.custody_ledger().total().units();
        op.draft
            .apply(&op.commands[0], actual_limits, &cancel)
            .unwrap();
        let current = op
            .draft
            .element(Key::Existing(input))
            .unwrap()
            .language()
            .input_state()
            .unwrap();
        assert!(current.flags().checkedness());
        assert!(Arc::ptr_eq(
            &original_buffer,
            current.current_value().shared_code_units()
        ));
        let current_element = op.draft.element(Key::Existing(input)).unwrap().language();
        assert!(Arc::ptr_eq(
            original_element.name_payload(),
            current_element.name_payload()
        ));
        assert!(original_element.same_attribute_storage(current_element));
        let changed_native = current_element.input_payload().unwrap().clone();
        assert!(!Arc::ptr_eq(&original_native, &changed_native));
        assert_eq!(
            op.draft.custody_ledger().total().units(),
            baseline + native_units
        );
        let draft = &lease.dom.source_operation.as_ref().unwrap().draft;
        let prepared = draft.finish_segment(lease.dom, &cancel).unwrap();
        let DomNodeKind::Element(element) =
            prepared.state.node(lease.dom.owner, input).unwrap().kind()
        else {
            panic!("Input candidate")
        };
        assert!(element.input_state().unwrap().flags().checkedness());
        assert!(Arc::ptr_eq(
            &original_buffer,
            element
                .input_state()
                .unwrap()
                .current_value()
                .shared_code_units()
        ));
        assert!(Arc::ptr_eq(
            &changed_native,
            element.input_payload().unwrap()
        ));
        assert!(Arc::ptr_eq(
            original_element.name_payload(),
            element.name_payload()
        ));
        assert!(original_element.same_attribute_storage(element));
        // Publication keeps the already sealed selected native identity. It
        // creates no second occurrence of the unchanged strings.
        assert_eq!(
            draft.custody_ledger().total().units(),
            baseline + native_units
        );
        drop(prepared);
        assert_eq!(
            draft.custody_ledger().total().units(),
            baseline + native_units
        );
        assert!(
            !before
                .node(input)
                .unwrap()
                .as_element()
                .unwrap()
                .input_state()
                .unwrap()
                .flags()
                .checkedness()
        );
    }
    fn actual_output_text_read(bytes: usize) -> Option<formation::mutation_observation::Growth> {
        let cancel = AtomicBool::new(false);
        let mut dom = Dom::new(
            DomLimits::new(
                32,
                100_000,
                64,
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
        if bytes == 16_000_000 {
            assert!(
                matches!(result, Ok(DomSourceProgress::Complete(_))),
                "generous original Output edit completes: {result:?}"
            );
            let snapshot = lease.snapshot();
            let node = snapshot.node(output).unwrap();
            assert_eq!(
                node.as_element()
                    .unwrap()
                    .output_state()
                    .unwrap()
                    .default_value_override()
                    .unwrap()
                    .to_utf8()
                    .unwrap(),
                "actual old output text".repeat(1024)
            );
            assert_eq!(node.children().len(), 1);
            assert!(
                matches!(snapshot.node(node.children()[0]).unwrap().kind(), DomNodeKind::Text(value) if value.to_utf8().unwrap() == "new")
            );
        } else if result.is_err() {
            assert_eq!(lease.snapshot().stamp(), before.stamp());
        }
        growth
    }
    #[test]
    fn source_output_text_admits_actual_materialization_before_retaining_read_frame() {
        let high = actual_output_text_read(16_000_000).expect("actual Output text materialization");
        // Each attempt owns different private registry identities. It may
        // refuse earlier; every reached materialization must fit its actual
        // caller envelope. Same-owner candidate and payload tests separately
        // assert exact and one-short admission boundaries.
        for limit in [high.required, high.required - 1] {
            if let Some(actual) = actual_output_text_read(limit) {
                assert!(
                    actual.required <= actual.limit,
                    "Output text materialized {} bytes under {}",
                    actual.required,
                    actual.limit
                );
            }
        }
    }
    #[test]
    fn final_ticket_output_preserves_an_earlier_published_then_retired_creation() {
        let cancel = AtomicBool::new(false);
        let mut dom = Dom::new(
            DomLimits::new(
                64,
                100_000,
                128,
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
        let done = dom.apply(setup, &cancel).unwrap();
        let document = done.document(&document).unwrap();
        let mut batch = DomBatch::new();
        let transient = batch.create_text(document, "published transient");
        let never_published = batch.create_text(document, "retired before any phase");
        batch.destroy(&never_published);
        let style = batch.create_element_ns(document, crate::Namespace::html(), "style");
        let text = batch.create_text(document, "a {}");
        batch.insert_before(&text, &style, None);
        batch.insert_before(&style, document, None);
        batch.destroy(&transient);
        let mut lease = dom
            .begin_source_commands(batch, limits(10_000_000), &cancel)
            .unwrap();
        let DomSourceProgress::Request(DomSourceRequest::Style(policy)) =
            lease.advance(&cancel).unwrap()
        else {
            panic!("actual connected Style CSP publication")
        };
        let published = {
            let draft = &lease.dom.source_operation.as_ref().unwrap().draft;
            let Key::Created(index) = draft.tickets[transient.index] else {
                panic!("original creation ticket")
            };
            draft.phase_map.as_ref().unwrap().ids.get(index).unwrap()
        };
        assert_eq!(
            lease.snapshot().node(published).unwrap().data(),
            Some(&DocumentString::from("published transient"))
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
            panic!("actual sheet creation publication")
        };
        let _root = commit_actual_empty_sheet(&mut lease, &create, &cancel);
        let DomSourceProgress::Complete(done) = lease.advance(&cancel).unwrap() else {
            panic!("actual final close")
        };
        assert_eq!(done.published[transient.index], Some(published));
        assert_eq!(done.published[never_published.index], None);
        assert!(lease.snapshot().node(published).is_err());
    }

    #[test]
    fn output_frame_payload_stays_admitted_through_checkpoint_parking_and_retry() {
        use surgeist_cssom::CssomEditGuard;
        let cancel = AtomicBool::new(false);
        let mut dom = Dom::new(
            DomLimits::new(
                64,
                100_000,
                128,
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
            let draft = &mut lease.dom.source_operation.as_mut().unwrap().draft;
            let live_bytes = draft.source_formation_bytes.clone();
            let live_units = draft.source_temporary_units.clone();
            let frame_bytes = live_bytes.load(Ordering::Relaxed);
            let frame_units = live_units.load(Ordering::Relaxed);
            assert!(frame_bytes >= minimum_bytes);
            assert!(frame_units >= minimum_units);
            let controlled = draft.custody_ledger().total();
            let checkpoint = draft.source_checkpoint(0, 0, &cancel).unwrap();
            assert_eq!(live_bytes.load(Ordering::Relaxed), frame_bytes);
            assert_eq!(live_units.load(Ordering::Relaxed), frame_units);
            assert!(
                draft.custody_ledger().total().bytes()
                    >= controlled.bytes() + size_of::<super::checkpoint::Checkpoint>(),
                "a mark owns its control header and shares the unchanged frame"
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
        let mut dom = Dom::new(
            DomLimits::new(
                32,
                100_000,
                64,
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
        if bytes == 16_000_000 {
            assert!(
                matches!(result, Ok(DomSourceProgress::Complete(_))),
                "generous original Textarea edit completes: {result:?}"
            );
            assert_eq!(
                lease
                    .snapshot()
                    .node(textarea)
                    .unwrap()
                    .as_element()
                    .unwrap()
                    .textarea_state()
                    .unwrap()
                    .raw_value(),
                &DocumentString::from("after\r\n".repeat(1024))
            );
            assert_eq!(
                before
                    .node(textarea)
                    .unwrap()
                    .as_element()
                    .unwrap()
                    .textarea_state()
                    .unwrap()
                    .raw_value(),
                &DocumentString::from("before\r\n".repeat(2048))
            );
        } else if result.is_err() {
            assert_eq!(lease.snapshot().stamp(), before.stamp());
        }
        growth
    }
    #[test]
    fn source_textarea_api_admits_actual_normalized_buffer_before_materialization() {
        let high = actual_textarea_api_read(16_000_000).expect("actual old API normalized buffer");
        // Private registry paths belong to each actual owner. An attempt can
        // refuse before this selected buffer; every reached buffer must still
        // fit that owner's true envelope. Same-owner candidate and factory
        // tests independently cover exact byte and UTF16 limits.
        for limit in [high.required, high.required - 1] {
            if let Some(actual) = actual_textarea_api_read(limit) {
                assert!(
                    actual.required <= actual.limit,
                    "Textarea API materialized {} bytes under {}",
                    actual.required,
                    actual.limit
                );
            }
        }
    }
    #[test]
    fn source_metadata_defaults_admit_actual_payload_before_formation() {
        let high = actual_metadata_creation(16_000_000).expect("actual native metadata formation");
        assert!(high.required <= high.limit);
        // Each independently constructed owner has its own materialization
        // IDs and registry paths. Check the actual envelope of each reached
        // allocation; earlier refusal is checked for snapshot rollback by the
        // owning fixture. Exact gates use one original Source below.
        for bytes in [high.required, high.required - 1] {
            if let Some(actual) = actual_metadata_creation(bytes) {
                assert!(
                    actual.required <= actual.limit,
                    "actual retained overlap {} exceeds its own allowance {}",
                    actual.required,
                    actual.limit
                );
            }
        }
    }
    #[test]
    fn source_select_task_growth_admits_actual_replacement_before_allocation() {
        let high = actual_select_task_growth(16_000_000)
            .expect("actual selected Option removal queues update");
        assert!(high.required <= high.limit);
        // Each independently constructed owner has its own materialization
        // IDs and registry paths. Check the actual envelope of each reached
        // allocation; earlier refusal is checked for snapshot rollback by the
        // owning fixture. Exact gates use one original Source below.
        for bytes in [high.required, high.required - 1] {
            if let Some(actual) = actual_select_task_growth(bytes) {
                assert!(
                    actual.required <= actual.limit,
                    "actual retained overlap {} exceeds its own allowance {}",
                    actual.required,
                    actual.limit
                );
            }
        }
    }
    #[test]
    fn source_children_growth_admits_actual_old_and_replacement_bank_before_allocation() {
        let high = actual_children_growth(16_000_000).expect("actual children capacity growth");
        assert!(high.required <= high.limit);
        // Each independently constructed owner has its own materialization
        // IDs and registry paths. Check the actual envelope of each reached
        // allocation; earlier refusal is checked for snapshot rollback by the
        // owning fixture. Exact gates use one original Source below.
        for bytes in [high.required, high.required - 1] {
            if let Some(actual) = actual_children_growth(bytes) {
                assert!(
                    actual.required <= actual.limit,
                    "actual retained overlap {} exceeds its own allowance {}",
                    actual.required,
                    actual.limit
                );
            }
        }
    }
    #[test]
    fn multiple_style_mandatory_tail_cannot_publish_without_closing_revision_headroom() {
        use surgeist_css::{CssParserContext, CssUtf16InputLimits};
        use surgeist_cssom::{CssomEditGuard, CssomInputRole, CssomInputVersion, CssomSheetInputs};
        let cancel = AtomicBool::new(false);
        let mut dom = Dom::new(
            DomLimits::new(
                64,
                100_000,
                64,
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
                formation::allocated_arc(Vec::new()),
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
                formation::allocated_arc(Vec::new()),
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
    fn actual_structural_and_control_reads(
        work: u64,
        cancelled: bool,
        remaining: Option<u64>,
    ) -> (bool, u64) {
        let cancel = AtomicBool::new(false);
        let domain = DomLimits::new(
            256,
            100_000,
            256,
            0,
            0,
            8,
            100_000,
            crate::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
        )
        .unwrap()
        .with_select_processing(256, 1_000_000, 1_000_000)
        .unwrap()
        .with_select_value_queries(
            surgeist_html_semantics::HtmlCaptureLimits::new(
                256, 512, 256, 100_000, 256, 0, 0, 16_777_216,
            ),
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
        // This semantic/work fixture has many simultaneous staged records.
        // Its byte envelope is deliberately generous; separate same-owner
        // candidate tests assert exact byte and UTF16 admission boundaries.
        let mut policy = limits(work);
        policy.retained_bytes = 128_000_000;
        let mut lease = match dom.begin_source_commands(commands, policy, &cancel) {
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
            .source_arc()
            .clone();
        let initial = meter.load(Ordering::Relaxed);
        let effective_work = remaining.map_or(work, |remaining| initial + remaining);
        lease
            .dom
            .source_operation
            .as_mut()
            .unwrap()
            .draft
            .source_limits
            .as_mut()
            .unwrap()
            .work = effective_work;
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
            Err(DomError::SourceWorkLimit { limit }) if limit == effective_work => {
                assert_eq!(spent, effective_work);
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
        let (complete, spent) = actual_structural_and_control_reads(100_000_000, false, None);
        assert!(complete);
        assert!(spent > 512);
        for remaining in [0, 1] {
            assert!(!actual_structural_and_control_reads(100_000_000, false, Some(remaining)).0);
        }
        assert!(!actual_structural_and_control_reads(100_000_000, true, None).0);
    }

    #[test]
    fn mandatory_tail_issues_ordinals_only_on_exposure_and_resets_at_the_next_command() {
        let cancel = AtomicBool::new(false);
        let mut dom = Dom::new(
            DomLimits::new(
                64,
                100_000,
                64,
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
        let domain = DomLimits::new(
            64,
            100_000,
            128,
            0,
            0,
            8,
            100_000,
            crate::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
        )
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
        let domain = DomLimits::new(
            64,
            100_000,
            128,
            0,
            0,
            8,
            100_000,
            crate::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
        )
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
        let domain = DomLimits::new(
            32,
            100_000,
            64,
            0,
            0,
            8,
            100_000,
            crate::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
        )
        .unwrap()
        .with_select_processing(16, 1_000_000, 1_000_000)
        .unwrap()
        .with_select_value_queries(
            HtmlCaptureLimits::new(32, 64, 32, 100_000, 32, 0, 0, 16_777_216),
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
        let (mut dom, batch) = fixture();
        let before = dom.snapshot();
        let mut lease = dom
            .begin_source_commands(batch, limits(10_000_000), &cancel)
            .unwrap();
        let initial = lease.status().spent_work();
        // The exact spent frontier belongs to this actual owner. A separately
        // constructed registry can require different private index paths.
        lease
            .dom
            .source_operation
            .as_mut()
            .unwrap()
            .draft
            .source_limits
            .as_mut()
            .unwrap()
            .work = initial;
        lease.request_cancel();
        assert_eq!(
            lease.advance(&cancel).unwrap_err(),
            DomError::SourceWorkLimit { limit: initial },
            "cancellation admits its actual result scan before executing it"
        );
        assert_eq!(lease.status().spent_work(), initial);
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
    fn unchanged_padded_text_retains_its_original_record_and_payload_in_a_candidate() {
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
        let document = setup.create_document();
        let mut padded = Vec::with_capacity(65_536);
        padded.push(0xd800);
        let text = setup.create_text(&document, DocumentString::from_utf16(padded));
        let done = dom.apply(setup, &cancel).unwrap();
        let text = done.created(&text).unwrap().unwrap();
        let lease = dom
            .begin_source_commands(DomBatch::new(), limits(10_000_000), &cancel)
            .unwrap();
        let draft = &lease.dom.source_operation.as_ref().unwrap().draft;
        let units = draft.custody_ledger().total().units();
        let original = dom_slot(&lease.dom.state, text);
        let prepared = draft.finish_segment(lease.dom, &cancel).unwrap();
        assert!(Arc::ptr_eq(
            original,
            prepared.state.slots.get_shared(text.slot).unwrap()
        ));
        assert_eq!(draft.custody_ledger().total().units(), units);
        assert_eq!(draft.source_temporary_units.load(Ordering::Relaxed), 0);
        let DomNodeKind::Text(value) = prepared.state.slots[text.slot]
            .node
            .as_ref()
            .unwrap()
            .kind()
        else {
            panic!("original Text")
        };
        assert_eq!(value.code_units(), &[0xd800]);
        assert_eq!(value.shared_code_units().capacity(), 65_536);
        drop(prepared);
        assert_eq!(draft.custody_ledger().total().units(), units);
    }

    fn dom_slot(state: &State, id: NodeId) -> &Arc<Slot> {
        state.slots.get_shared(id.slot).unwrap()
    }

    #[test]
    fn same_source_candidate_root_body_admits_exact_bytes_and_refuses_one_short() {
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
        let document = setup.create_document();
        let mut padded = Vec::with_capacity(65_536);
        padded.push(0xd800);
        let text = setup.create_text(&document, DocumentString::from_utf16(padded));
        let done = dom.apply(setup, &cancel).unwrap();
        let text = done.created(&text).unwrap().unwrap();
        let lease = dom
            .begin_source_commands(DomBatch::new(), limits(10_000_000), &cancel)
            .unwrap();
        assert!(lease.dom.state.signal_slots.is_empty());
        assert!(lease.dom.state.native_tasks.is_empty());
        assert!(lease.dom.state.details_intentions.is_empty());
        let before = lease.snapshot();
        let original = dom_slot(&lease.dom.state, text).clone();
        let op = lease.dom.source_operation.as_mut().unwrap();
        op.draft.custody_registry().reconcile_releases();
        let baseline = op.draft.custody_ledger().total().bytes() + op.draft.source_other_bytes;
        // This materialized State has three absent canonical queue backings.
        // The payload bank and its padded Text remain original Arc aliases.
        let candidate = size_of::<State>();
        let exact = baseline + candidate;
        op.draft.source_limits.as_mut().unwrap().retained_bytes = exact - 1;
        let draft = &lease.dom.source_operation.as_ref().unwrap().draft;
        assert!(
            matches!(draft.finish_segment(lease.dom, &cancel), Err(DomError::ResourceLimit {resource:DomResource::SourceRetainedBytes,limit}) if limit==exact-1)
        );
        assert_eq!(lease.snapshot().stamp(), before.stamp());
        assert!(Arc::ptr_eq(&original, dom_slot(&lease.dom.state, text)));
        lease
            .dom
            .source_operation
            .as_mut()
            .unwrap()
            .draft
            .source_limits
            .as_mut()
            .unwrap()
            .retained_bytes = exact;
        let draft = &lease.dom.source_operation.as_ref().unwrap().draft;
        let prepared = draft.finish_segment(lease.dom, &cancel).unwrap();
        assert_eq!(
            draft.custody_ledger().total().bytes() + draft.source_other_bytes,
            exact
        );
        assert!(Arc::ptr_eq(&original, dom_slot(&prepared.state, text)));
        drop(prepared);
        assert_eq!(
            draft.custody_ledger().total().bytes() + draft.source_other_bytes,
            baseline
        );
    }

    #[test]
    fn changed_text_candidate_reuses_its_payload_with_exact_and_one_short_union_admission() {
        let cancel = AtomicBool::new(false);
        for short in [false, true] {
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
            let document = setup.create_document();
            let text = setup.create_text(&document, "original");
            let done = dom.apply(setup, &cancel).unwrap();
            let text = done.created(&text).unwrap().unwrap();
            let original_units = dom
                .state
                .slots
                .values()
                .filter_map(|slot| slot.node.as_ref())
                .map(|node| match node.kind() {
                    DomNodeKind::Document(metadata) => {
                        [metadata.content_type(), metadata.encoding(), metadata.url()]
                            .into_iter()
                            .map(|s| s.shared_code_units().capacity())
                            .sum::<usize>()
                    }
                    DomNodeKind::Text(value) => value.shared_code_units().capacity(),
                    _ => panic!("Document and Text fixture"),
                })
                .sum::<usize>();
            let mut batch = DomBatch::new();
            batch.replace_data(text, 0, usize::MAX, "replacement");
            let lease = dom
                .begin_source_commands(batch, limits(10_000_000), &cancel)
                .unwrap();
            let actual_limits = lease.dom.limits;
            let op = lease.dom.source_operation.as_mut().unwrap();
            op.draft
                .apply(&op.commands[0], actual_limits, &cancel)
                .unwrap();
            let Command::ReplaceData { data, .. } = &op.commands[0] else {
                panic!("original replacement command")
            };
            let input_units = data.shared_code_units().capacity();
            let field = op.draft.node(Key::Existing(text)).unwrap();
            let transaction::KindRead::Text(value) = field.kind() else {
                panic!("actual changed Text field")
            };
            let field_units = value.shared_code_units().capacity();
            assert_eq!(
                value.code_units(),
                DocumentString::from("replacement").code_units()
            );
            let DraftNode::Text(staged) = op.draft.records_for(Key::Existing(text)) else {
                panic!("actual typed Text owner")
            };
            let staged_data = staged.value().data.value.clone();
            // Original canonical payload + command input + one newly sealed
            // CharacterData target. Publication aliases that target rather
            // than forming a second whole-kind string occurrence.
            let exact = original_units + input_units + field_units;
            let limit = exact - usize::from(short);
            op.draft.source_limits.as_mut().unwrap().retained_units = limit;
            let draft = &lease.dom.source_operation.as_ref().unwrap().draft;
            let before = lease.snapshot();
            let result = draft.finish_segment(lease.dom, &cancel);
            if short {
                assert!(
                    matches!(result, Err(DomError::ResourceLimit {resource:DomResource::SourceRetainedUnits,limit:actual}) if actual==limit)
                );
            } else {
                let prepared = result.unwrap();
                assert_eq!(
                    draft.custody_ledger().total().units() + draft.source_other_units,
                    exact
                );
                assert!(!Arc::ptr_eq(
                    dom_slot(&lease.dom.state, text),
                    prepared.state.slots.get_shared(text.slot).unwrap()
                ));
                let Node::Text(published) = prepared.state.node(lease.dom.owner, text).unwrap()
                else {
                    panic!("published typed Text owner")
                };
                assert!(Arc::ptr_eq(&staged_data, &published.value.data));
                assert_eq!(
                    published.value.data.value.value.code_units(),
                    DocumentString::from("replacement").code_units()
                );
                drop(prepared);
                assert_eq!(
                    draft.custody_ledger().total().units() + draft.source_other_units,
                    exact
                );
            }
            assert_eq!(lease.snapshot().stamp(), before.stamp());
            assert_eq!(
                lease.snapshot().node(text).unwrap().data(),
                Some(&DocumentString::from("original"))
            );
        }
    }
    #[test]
    fn formed_source_input_holds_its_utf16_charge_while_candidate_remains_live() {
        let cancel = AtomicBool::new(false);
        let mut dom = Dom::new(
            DomLimits::new(
                32,
                100_000,
                64,
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
        let prepared = op.draft.finish_segment(lease.dom, &cancel).unwrap();
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
        let mut dom = Dom::new(
            DomLimits::new(
                64,
                100_000,
                128,
                128,
                128,
                8,
                100_000,
                crate::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
            )
            .unwrap(),
        )
        .unwrap();
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
    #[derive(Clone, Copy)]
    enum SlotAdmissionCut {
        BytesAtBaseline,
        UnitsExact,
        UnitsOneShort,
        WorkAtSpent,
    }
    fn slot_operation_admission_case(cut: Option<SlotAdmissionCut>) -> bool {
        let (mut dom, batch, slot, x, y) = slot_operation_fixture();
        let before = dom.snapshot();
        let cancel = AtomicBool::new(false);
        let result = match dom.begin_source_commands(batch, limits(10_000_000), &cancel) {
            Err(error) => Err(error),
            Ok(mut lease) => {
                let draft = &mut lease.dom.source_operation.as_mut().unwrap().draft;
                let controlled = draft.custody_ledger().total();
                let bytes = controlled.bytes() + draft.source_other_bytes;
                let units = controlled.units() + draft.source_other_units;
                let spent = draft.source_work.load(Ordering::Relaxed);
                let ledger = draft.custody_ledger().clone();
                if let Some(dimension) = cut {
                    let policy = draft.source_limits.as_mut().unwrap();
                    match dimension {
                        SlotAdmissionCut::BytesAtBaseline => policy.retained_bytes = bytes,
                        SlotAdmissionCut::UnitsExact => policy.retained_units = units,
                        SlotAdmissionCut::UnitsOneShort => {
                            policy.retained_units = units
                                .checked_sub(1)
                                .expect("nonempty original Text and Element names")
                        }
                        SlotAdmissionCut::WorkAtSpent => policy.work = spent,
                    }
                }
                match lease.advance(&cancel) {
                    Ok(DomSourceProgress::Complete(done)) => {
                        assert!(!done.cancellation_requested());
                        if matches!(cut, Some(SlotAdmissionCut::UnitsExact)) {
                            // Reference-only edits retain the original string
                            // targets; the new topology/relation groups own zero
                            // UTF16 units. Existing baseline is configuration,
                            // not a newly priced materialization receipt.
                            assert_eq!(ledger.total().units(), controlled.units());
                        }
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
                        | DomSourceRequest::CloseWatcher(_)
                        | DomSourceRequest::DialogFocusing(_)
                        | DomSourceRequest::FocusDelegate(_)
                        | DomSourceRequest::DesignMode(_)
                        | DomSourceRequest::FullscreenExitHandoff(_)
                        | DomSourceRequest::AutofocusInsertion(_),
                    )) => {
                        panic!("Slot assignment fixture has no eventful Dialog operation")
                    }
                    Err(error) => {
                        match cut {
                            Some(SlotAdmissionCut::BytesAtBaseline) => assert!(
                                matches!(error, DomError::ResourceLimit {resource: DomResource::SourceRetainedBytes,limit} if limit==bytes)
                            ),
                            Some(SlotAdmissionCut::UnitsOneShort) => assert!(
                                matches!(error, DomError::ResourceLimit {resource: DomResource::SourceRetainedUnits,limit} if limit==units-1)
                            ),
                            Some(SlotAdmissionCut::WorkAtSpent) => {
                                assert_eq!(error, DomError::SourceWorkLimit { limit: spent })
                            }
                            Some(SlotAdmissionCut::UnitsExact) => panic!(
                                "reference-only Slot edit exceeded its unchanged UTF16 union: {error:?}"
                            ),
                            None => {}
                        }
                        Err(error)
                    }
                }
            }
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
    fn actual_slot_operation_checks_each_original_admission_domain_and_rolls_back_refusal() {
        assert!(slot_operation_admission_case(None));
        assert!(slot_operation_admission_case(Some(
            SlotAdmissionCut::UnitsExact
        )));
        for dimension in [
            SlotAdmissionCut::BytesAtBaseline,
            SlotAdmissionCut::UnitsOneShort,
            SlotAdmissionCut::WorkAtSpent,
        ] {
            assert!(!slot_operation_admission_case(Some(dimension)));
        }
    }
}
