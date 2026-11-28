//! Queue the hosted removals reached by one private Form reset body.

use super::*;
use crate::internal::{
    hosted::{
        DomHostedChildRemovalHandoff, DomHostedCreation, DomHostedDestructionInvocation,
        DomHostedError, DomHostedGlobalId, DomHostedNavigableId, DomHostedQueuedHistory,
        DomHostedResource, DomHostedSource, Request, RequestKey, Retained, Work, admit,
        removal_issuance::Funding,
    },
    store::hosted as canonical,
};

pub(super) struct Occurrence {
    pub(super) container: NodeId,
    pub(super) ordinal: u64,
    pub(super) hook: hosted::ChildHook,
}

struct Removal {
    creation: DomHostedCreation,
    parent: DomHostedNavigableId,
    funding: Option<RemovalWork>,
}

struct RemovalWork {
    work: Arc<Work>,
    _source: Option<checkpoint::Charge>,
}

pub(super) struct ResetBody {
    start: Option<checkpoint::Checkpoint>,
    form: Key,
    failure: Option<DomError>,
    pub(super) stylesheets: super::form_stylesheets::Stylesheets,
}

impl ResetBody {
    pub(super) fn start(
        draft: &mut Draft,
        form: Key,
        cancel: &AtomicBool,
    ) -> Result<Self, DomError> {
        Ok(Self {
            start: Some(draft.source_checkpoint(0, 0, cancel)?),
            form,
            failure: None,
            stylesheets: super::form_stylesheets::Stylesheets::default(),
        })
    }
}

#[derive(Default)]
pub(super) struct Removals {
    entries: Vec<Removal>,
    queue: Option<Queue>,
    request: Option<DomSourceHostedRemovalsRequest>,
    entries_charge: Option<checkpoint::Charge>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Queue {
    global: DomHostedGlobalId,
    traversable: DomHostedNavigableId,
}

/// The exact queue destination and number of child removals reached by an edit.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DomSourceHostedRemovalsRequest {
    operation: DomSourceOperationId,
    queue: Queue,
    count: usize,
}

impl DomSourceHostedRemovalsRequest {
    /// The original parent Global and initiating task loop.
    pub const fn parent_global(self) -> DomHostedGlobalId {
        self.queue.global
    }

    /// The traversable that owns the original history queue.
    pub const fn traversable(self) -> DomHostedNavigableId {
        self.queue.traversable
    }

    /// The number of removals, in canonical algorithm order.
    pub const fn count(self) -> usize {
        self.count
    }
}

/// Original history and destruction handoffs, in the reached removal order.
pub struct DomQueuedSourceHostedRemovals {
    handoffs: std::vec::IntoIter<DomHostedChildRemovalHandoff>,
    _charge: Retained,
}

impl Iterator for DomQueuedSourceHostedRemovals {
    type Item = DomHostedChildRemovalHandoff;

    fn next(&mut self) -> Option<Self::Item> {
        self.handoffs.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.handoffs.size_hint()
    }
}

impl ExactSizeIterator for DomQueuedSourceHostedRemovals {}

pub(super) fn unjoined_form_service(error: &DomError) -> bool {
    matches!(
        error,
        DomError::StyleCompositionRequired(_)
            | DomError::StyleOwnerAttributesCompositionRequired
            | DomError::StyleDisabledCompositionRequired
            | DomError::StyleProcessingUnavailable
            | DomError::HostedSourceUnavailable
            | DomError::ObjectHostedTransitionUnavailable
            | DomError::FocusedAreaDesignationUnavailable { .. }
            | DomError::ScriptPreparationRequired
            | DomError::MetaCspProcessingRequired
            | DomError::FullscreenExitHandoffRequired
            | DomError::DesignModeProcessingUnavailable { .. }
            | DomError::AutofocusInsertionUnavailable { .. }
            | DomError::DialogFocusingUnavailable { .. }
            | DomError::NeedsEventfulOperation { .. }
            | DomError::DialogProcessingUnavailable { .. }
    )
}

impl Operation {
    pub(super) fn stop_form_body(
        &mut self,
        error: DomError,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let body = self
            .reset_body
            .as_mut()
            .ok_or(DomError::InvalidSourceOperation)?;
        body.failure = Some(error);
        self.close_stopped_form_body(cancel)
    }

    pub(super) fn close_stopped_form_body(&mut self, cancel: &AtomicBool) -> Result<(), DomError> {
        let Some(body) = self.reset_body.as_mut() else {
            return Ok(());
        };
        if !self.cancelled && body.failure.is_none() {
            return Ok(());
        }
        if let Some(checkpoint) = body.start.take() {
            checkpoint.restore(&mut self.draft);
            self.hosted_removals = Removals::default();
            body.stylesheets = super::form_stylesheets::Stylesheets::default();
        }
        let form = body.form;
        let failure = body.failure.clone();
        let input_bytes = self.input_bytes()?;
        let checkpoint = self.draft.source_checkpoint(input_bytes, 0, cancel)?;
        let cleanup = (|| {
            self.draft.stage_form_reset_tail(form, false, cancel)?;
            if let Some(error) = failure {
                self.draft.source_set_method(
                    Some(crate::internal::store::method::Frame::CompoundFailure(
                        error,
                    )),
                    cancel,
                )?;
            }
            Ok(())
        })();
        if cleanup.is_ok() {
            self.reset_body = None;
        } else {
            checkpoint.restore(&mut self.draft);
        }
        cleanup
    }

    pub(super) fn collect_form_removal(
        &mut self,
        dom: &Dom,
        cancel: &AtomicBool,
    ) -> Result<bool, DomError> {
        if self.reset_body.is_none() {
            return Ok(false);
        }
        let Some(
            hook @ hosted::ChildHook::Remove {
                key: Key::Existing(container),
                navigable,
            },
        ) = self.draft.hosted_pending
        else {
            return Err(DomError::HostedSourceUnavailable);
        };
        let collected = u64::try_from(self.hosted_removals.entries.len())
            .map_err(|_| DomError::RepresentationExhausted)?;
        let ordinal = self
            .draft
            .source_invocation
            .checked_add(collected)
            .and_then(|last| last.checked_add(1))
            .ok_or(DomError::RepresentationExhausted)?;
        let formation = formation::Formation::new(&self.draft, cancel)?;
        let request = hosted::Child::qualify(
            self,
            &dom.state,
            DomStamp {
                owner: dom.owner,
                revision: dom.revision,
            },
            Occurrence {
                container,
                ordinal,
                hook,
            },
            formation.as_ref(),
            cancel,
        )?;
        let queue = Queue {
            global: request.parent_global(),
            traversable: request.traversable(),
        };
        if self
            .hosted_removals
            .queue
            .is_some_and(|other| other != queue)
        {
            return Err(DomError::HostedSourceUnavailable);
        }
        let graph = self
            .draft
            .hosted_history
            .as_deref()
            .ok_or(DomError::HostedSourceUnavailable)?;
        let state = graph
            .navigable_with(navigable, || {
                self.draft.source_step(cancel).map_err(DomHostedError::from)
            })
            .map_err(canonical_error)?;
        if state.parent != Some(request.parent_navigable())
            || state.traversable != queue.traversable
        {
            return Err(DomError::HostedSourceUnavailable);
        }
        let document = graph
            .entry_with(state.active, || {
                self.draft.source_step(cancel).map_err(DomHostedError::from)
            })
            .map_err(canonical_error)?
            .document
            .ok_or(DomError::HostedSourceUnavailable)?;
        let creation = DomHostedCreation {
            document,
            host: state.host,
            context: state.context,
            global: state.global,
            entry: state.active,
            navigable,
        };
        let removals = &mut self.hosted_removals;
        formation::reserve(&mut removals.entries, 1, formation.as_ref(), cancel)?;
        if let Some(formation) = formation {
            let charge = formation.into_charge();
            match &mut removals.entries_charge {
                Some(retained) => retained.merge(charge),
                slot => *slot = Some(charge),
            }
        }
        removals.entries.push(Removal {
            creation,
            parent: request.parent_navigable(),
            funding: None,
        });
        removals.queue = Some(queue);
        self.draft.hosted_pending = None;
        Ok(true)
    }

    pub(super) fn form_publication_request(
        &mut self,
    ) -> Result<Option<DomSourceRequest>, DomError> {
        if self.hosted_removals.queue.is_none()
            && self
                .reset_body
                .as_ref()
                .is_none_or(|body| body.stylesheets.is_empty())
        {
            return Ok(None);
        }
        if let Some(task) = self
            .draft
            .final_element_tasks
            .as_ref()
            .and_then(|tasks| tasks.first())
        {
            return Err(task.kind.unavailable());
        }
        if self.meta_reached() {
            return Err(DomError::Meta(
                crate::internal::meta::DomMetaError::DefaultStyleUnsupported,
            ));
        }
        if let Some(queue) = self.hosted_removals.queue {
            let request = DomSourceHostedRemovalsRequest {
                operation: self.id,
                queue,
                count: self.hosted_removals.entries.len(),
            };
            self.hosted_removals.request = Some(request);
            return Ok(Some(DomSourceRequest::HostedRemovals(request)));
        }
        Ok(self
            .form_stylesheets_request()
            .map(DomSourceRequest::FormStylesheets))
    }

    pub(super) fn pending_hosted_removals(&self) -> Option<DomSourceHostedRemovalsRequest> {
        self.hosted_removals.request
    }
}

fn canonical_error(error: DomHostedError) -> DomError {
    match error {
        DomHostedError::Dom(error) => error,
        _ => DomError::HostedSourceUnavailable,
    }
}

struct RemovalRows {
    history: canonical::operations::Transfer,
    destruction: canonical::destruction::RemovalDestruction,
}

struct RetainedRows<T> {
    values: Vec<T>,
    retained: Retained,
}

fn reserve_rows<T>(
    count: usize,
    source: &DomHostedSource,
    scratch: &mut usize,
    formation: Option<&formation::Formation>,
    cancel: &AtomicBool,
) -> Result<RetainedRows<T>, DomHostedError> {
    let requested = count
        .checked_mul(size_of::<T>())
        .ok_or(DomHostedError::RepresentationExhausted)?;
    add_scratch(scratch, requested, source)?;
    if let Some(formation) = formation {
        formation.retain_bytes(requested, cancel)?;
    }
    let mut retained = source.retain(requested, cancel)?;
    let mut rows = Vec::new();
    rows.try_reserve_exact(count)
        .map_err(|_| DomHostedError::AllocationFailed)?;
    let extra = (rows.capacity() - count)
        .checked_mul(size_of::<T>())
        .ok_or(DomHostedError::RepresentationExhausted)?;
    add_scratch(scratch, extra, source)?;
    if let Some(formation) = formation {
        formation.retain_bytes(extra, cancel)?;
    }
    retained.append(source.retain(extra, cancel)?)?;
    Ok(RetainedRows {
        values: rows,
        retained,
    })
}

fn add_scratch(
    scratch: &mut usize,
    bytes: usize,
    source: &DomHostedSource,
) -> Result<(), DomHostedError> {
    *scratch = scratch
        .checked_add(bytes)
        .ok_or(DomHostedError::RepresentationExhausted)?;
    admit(
        *scratch,
        source.limits().scratch_bytes(),
        DomHostedResource::ScratchBytes,
    )
}

impl DomSourceLease<'_> {
    /// Publish the original reset and transfer its reached child removals in
    /// order. The hosted owner reserves all queue destinations before calling.
    pub fn publish_hosted_child_removals(
        &mut self,
        source: &mut DomHostedSource,
        request: DomSourceHostedRemovalsRequest,
        cancel: &AtomicBool,
    ) -> Result<
        (DomSourceCompletion, DomQueuedSourceHostedRemovals),
        super::form_stylesheets::DomFormResetPublicationError,
    > {
        self.dom.validate_hosted_source(source)?;
        let mut operation = self
            .dom
            .source_operation
            .take()
            .ok_or(DomHostedError::WrongPhase)?;
        if self.complete
            || request.operation != self.id
            || operation.cancelled
            || operation.pending_hosted_removals() != Some(request)
        {
            self.dom.source_operation = Some(operation);
            return Err(DomHostedError::WrongPhase.into());
        }
        let result = operation.publish_form_removals(self.dom, source, request, cancel);
        if result.is_ok() {
            self.complete = true;
            self.dom.source_operation = self.dom.source_parents.pop();
            #[cfg(test)]
            self.dom.observe_source_lifecycle();
        } else {
            self.dom.source_operation = Some(operation);
        }
        result
    }
}

impl Operation {
    fn publish_form_removals(
        &mut self,
        dom: &mut Dom,
        source: &DomHostedSource,
        request: DomSourceHostedRemovalsRequest,
        cancel: &AtomicBool,
    ) -> Result<
        (DomSourceCompletion, DomQueuedSourceHostedRemovals),
        super::form_stylesheets::DomFormResetPublicationError,
    > {
        self.refresh_admission(cancel)?;
        let input_bytes = self.input_bytes()?;
        let checkpoint = self.draft.source_checkpoint(input_bytes, 0, cancel)?;
        let result = self.with_form_stylesheets(dom, cancel, |op, dom| {
            op.join_form_removals(dom, source, request, cancel)
                .map_err(Into::into)
        });
        if result.is_err() {
            checkpoint.restore(&mut self.draft);
        }
        result
    }

    fn join_form_removals(
        &mut self,
        dom: &mut Dom,
        source: &DomHostedSource,
        request: DomSourceHostedRemovalsRequest,
        cancel: &AtomicBool,
    ) -> Result<(DomSourceCompletion, DomQueuedSourceHostedRemovals), DomHostedError> {
        let mut scratch = 0;
        let staging = formation::Formation::new(&self.draft, cancel)?;
        let mut funding = reserve_rows(
            request.count,
            source,
            &mut scratch,
            staging.as_ref(),
            cancel,
        )?;
        for entry in &mut self.hosted_removals.entries {
            let work = match &entry.funding {
                Some(funding) => funding.work.clone(),
                None => {
                    let storage = formation::Formation::new(&self.draft, cancel)?;
                    if let Some(storage) = &storage {
                        storage
                            .retain_bytes(size_of::<Work>() + 2 * size_of::<AtomicU64>(), cancel)?;
                    }
                    let work = source.work(cancel)?;
                    entry.funding = Some(RemovalWork {
                        work: work.clone(),
                        _source: storage.map(formation::Formation::into_charge),
                    });
                    work
                }
            };
            source.refresh_private_removal_work(&work)?;
            work.step(cancel)?;
            let existing_cleanup = dom.removal_cleanup(source, entry.creation, &work, cancel)?;
            self.draft.stage_hosted_child_unlink(
                source,
                &work,
                entry.creation.navigable,
                entry.parent,
                scratch,
                cancel,
            )?;
            funding.values.push(Funding {
                work,
                existing_cleanup,
            });
        }
        let segment = self.prepare_segment(dom, false, PublicationTarget::None, None, cancel)?;
        if let Some(recording) = segment.prepared.inline_recording.as_ref() {
            recording.require_unreached(|| self.draft.source_step(cancel))?;
        }
        let issuance = source.prepare_child_removal_issuance(
            &funding.values,
            scratch,
            staging.as_ref(),
            cancel,
        )?;
        add_scratch(&mut scratch, issuance.scratch_bytes(), source)?;
        let mut rows = reserve_rows(
            request.count,
            source,
            &mut scratch,
            staging.as_ref(),
            cancel,
        )?;
        let mut handoffs = reserve_rows(
            request.count,
            source,
            &mut scratch,
            staging.as_ref(),
            cancel,
        )?;
        let mut cleanup_count = 0u64;
        for ((entry, funding), ids) in self
            .hosted_removals
            .entries
            .iter()
            .zip(&funding.values)
            .zip(issuance.ids())
        {
            let creation = entry.creation;
            let work = &funding.work;
            let destruction = dom.prepare_removal_destruction(
                source,
                canonical::destruction::RemovalTarget {
                    creation,
                    traversable: request.queue.traversable,
                    storage: canonical::destruction::RemovalStorage {
                        scratch_base: scratch,
                        formation: staging.as_ref(),
                    },
                },
                funding,
                *ids,
                cancel,
            )?;
            add_scratch(&mut scratch, destruction.backing, source)?;
            cleanup_count += u64::from(destruction.cleanup.is_some());
            let transaction::KindRead::Document(metadata) = self
                .draft
                .node(Key::Existing(creation.document.node()))?
                .kind()
            else {
                return Err(DomError::WrongDocument.into());
            };
            let retained = source.retain(size_of::<canonical::Operation>(), cancel)?;
            let bank = dom.hosted_operations.prepare(
                ids.history.slot,
                work,
                source,
                scratch,
                staging.as_ref(),
                canonical::operations::BodyCredit::Owned(retained),
                canonical::operations::Projection::default(),
                cancel,
            )?;
            add_scratch(&mut scratch, bank.backing_bytes()?, source)?;
            let (bank, retained) = bank.into_owned();
            let continuation = canonical::queue::Continuation::removal(
                Request {
                    key: RequestKey {
                        operation: ids.history,
                        phase: 1,
                    },
                    host: creation.host,
                    context: creation.context,
                    work: work.clone(),
                    parent: None,
                },
                creation,
                request.queue.traversable,
            );
            let history = canonical::operations::Transfer::new(
                bank,
                canonical::Operation {
                    origin: metadata.origin().clone(),
                    policy: None,
                    phase: canonical::CreationPhase::Queued(continuation),
                    _retained: retained,
                },
            );
            rows.values.push(RemovalRows {
                history,
                destruction,
            });
            handoffs.values.push(DomHostedChildRemovalHandoff {
                invocation: DomHostedDestructionInvocation {
                    operation: ids.invocation,
                    navigable: creation.navigable,
                },
                history: DomHostedQueuedHistory {
                    operation: ids.history,
                    traversable: request.queue.traversable,
                },
            });
        }
        let hosted_revisions = dom
            .hosted_revisions
            .checked_add(cleanup_count)
            .ok_or(DomError::RevisionExhausted)?;
        let protected = dom.source_reserved_closing(cancel)?;
        dom.revision
            .0
            .checked_add(u64::from(segment.changed))
            .and_then(|revision| revision.checked_add(hosted_revisions))
            .and_then(|revision| revision.checked_add(protected))
            .ok_or(DomError::RevisionExhausted)?;
        let permit = issuance.authorize(source, cancel)?;
        let commit = segment.publish(self, dom).commit;
        permit.commit();
        dom.hosted_revisions = hosted_revisions;
        for rows in rows.values {
            drop(dom.hosted_operations.insert_prepared(rows.history, false));
            drop(
                dom.hosted_operations
                    .insert_prepared(rows.destruction.invocation, false),
            );
            if let Some(cleanup) = rows.destruction.cleanup {
                drop(dom.hosted_operations.insert_prepared(cleanup, false));
            }
        }
        Ok((
            self.complete_form_reset(commit),
            DomQueuedSourceHostedRemovals {
                handoffs: handoffs.values.into_iter(),
                _charge: handoffs.retained,
            },
        ))
    }
}
