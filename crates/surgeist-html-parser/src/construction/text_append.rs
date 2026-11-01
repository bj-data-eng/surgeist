//! Real append ownership: one native backing, original registry and controller.
use super::*;
use crate::bounded::{
    HtmlConstructionTextAppend as Append, HtmlConstructionTextAppendDescription as Description,
    HtmlConstructionTextAppendWork as Work,
};
use construction_owner::construction_error;

fn admit_work(
    control: &mut ParseControl,
    work: Work,
    wrapper_prepared: bool,
) -> Result<(), SessionError> {
    control.admit_atomic_layout(work.atomic_layout_bytes)?;
    control.charge(WorkCategory::RecordVisit, work.record_visits as u64)?;
    let Some(provider_calls) = work
        .provider_calls
        .checked_sub(usize::from(wrapper_prepared))
    else {
        return Err(latch_registry_row(control));
    };
    control.charge(WorkCategory::ProviderCall, provider_calls as u64)?;
    control.charge(WorkCategory::InspectByte, work.inspected_bytes as u64)?;
    control.charge(WorkCategory::CopyByte, work.copied_bytes as u64)?;
    control.charge(WorkCategory::RecordWrite, work.record_writes as u64)?;
    Ok(())
}

/// One actual stopped pending allocation, kept until explicit owner cleanup.
pub(super) struct FailedTextAppend {
    value: DocumentString,
    allocation: DocumentStringAllocation,
}
impl FailedTextAppend {
    /// A terminal owner holding a ticket, for tests that need the slot taken.
    #[cfg(test)]
    pub(super) fn held(value: DocumentString, allocation: DocumentStringAllocation) -> Self {
        Self { value, allocation }
    }
    pub(super) fn release(self, control: &mut ParseControl) -> Result<(), SessionError> {
        drop(self.value);
        Ok(control.release_document_string(self.allocation)?)
    }
}

pub(super) struct RegisteredTextAppend {
    pub(super) parent: usize,
    pub(super) index: usize,
    pub(super) old_slot: usize,
    pub(super) incoming_slot: usize,
    pub(super) incoming: DocumentString,
    pub(super) dispose_old: bool,
}

/// What a committed append still holds before the registry row that adopts it
/// exists: the new backing's ticket, the displaced original's ticket and the
/// displaced value. Whatever is left after a refusal is released, so nothing
/// is dropped uncredited.
struct CommittedTickets {
    pending: Option<DocumentStringAllocation>,
    original: Option<DocumentStringAllocation>,
    discarded: Option<DocumentString>,
}
impl CommittedTickets {
    /// Releases every remaining ticket, remembering the first refusal.
    fn release(self, control: &mut ParseControl) -> Option<SessionError> {
        let Self {
            pending,
            original,
            discarded,
        } = self;
        drop(discarded);
        let mut refused = None;
        for ticket in [pending, original].into_iter().flatten() {
            keep_first(&mut refused, control.release_document_string(ticket));
        }
        refused
    }
}

impl SessionStorage {
    /// Takes the original string's ticket out of its row for the append, leaving
    /// the row retired or, when it has origins, historical. A refusal leaves
    /// the row as it was.
    fn take_original_ticket(
        &mut self,
        control: &mut ParseControl,
        old_slot: usize,
    ) -> Result<DocumentStringAllocation, SessionError> {
        let Some(row) = self.payloads.values.get_mut(old_slot) else {
            return Err(latch_registry_row(control));
        };
        let RetainedPayload::String {
            value, allocation, ..
        } = &mut *row
        else {
            return Err(SessionError::WrongToken);
        };
        control.suspend_text_append_ticket(allocation, value)?;
        match mem::replace(row, RetainedPayload::Retired) {
            RetainedPayload::String {
                value,
                allocation,
                origins,
                ..
            } => {
                drop(value);
                if let Some(origins) = origins {
                    *row = RetainedPayload::HistoricalString { origins };
                }
                Ok(allocation)
            }
            other => {
                *row = other;
                Err(latch_registry_row(control))
            }
        }
    }

    /// Registers a committed append: the surviving ticket is restored to the
    /// committed text and moves into the row reserved for it. Everything that
    /// can refuse is checked while the tickets are still in `tickets`, so a
    /// refusal leaves them there for the caller to release.
    fn adopt_committed_append(
        &mut self,
        control: &mut ParseControl,
        tickets: &mut CommittedTickets,
        adoption: Adoption,
    ) -> Result<(), SessionError> {
        let Adoption {
            index,
            old_slot,
            slot,
            origins,
            dispose_old,
        } = adoption;
        let Some(slot) = slot else {
            return Err(latch_registry_row(control));
        };
        if self.payloads.values.get(slot).is_none() {
            return Err(latch_registry_row(control));
        }
        // A reused original has no new backing and displaces nothing.
        let reused = tickets.pending.is_none();
        if reused && tickets.discarded.is_some() {
            return Err(latch_registry_row(control));
        }
        // A preserved original no longer names the committed text.
        if !dispose_old {
            self.bind_string(old_slot, StringLocation::Detached)
                .map_err(|_| latch_registry_row(control))?;
        }
        let Some(HtmlNodeKind::Text(committed)) =
            self.construction.node_at(index).map(|record| record.kind())
        else {
            return Err(latch_registry_row(control));
        };
        let Some(survivor) = (if reused {
            tickets.original.as_mut()
        } else {
            tickets.pending.as_mut()
        }) else {
            return Err(latch_registry_row(control));
        };
        control.restore_text_append_ticket(survivor, committed)?;
        let value = committed.clone();
        drop(tickets.discarded.take());
        let Some(allocation) = (if reused {
            tickets.original.take()
        } else {
            tickets.pending.take()
        }) else {
            return Err(latch_registry_row(control));
        };
        let Some(row) = self.payloads.values.get_mut(slot) else {
            // Checked above; the ticket goes back rather than being dropped.
            if reused {
                tickets.original = Some(allocation);
            } else {
                tickets.pending = Some(allocation);
            }
            return Err(latch_registry_row(control));
        };
        *row = RetainedPayload::String {
            value,
            allocation,
            origins,
            location: StringLocation::Text(index),
        };
        // The row owns the surviving ticket from here on.
        if let Some(old) = tickets.original.take() {
            control.release_document_string(old)?;
        }
        self.payloads
            .join_text(index, slot)
            .map_err(|_| latch_registry_row(control))
    }

    /// Hands back what a stopped append holds: the pending buffer becomes the
    /// terminal owner, the original string's ticket returns to its row.
    /// A refusal is latched and reported in place of `error`; the tickets it
    /// concerns stay with their owners.
    fn settle_stopped_append(
        &mut self,
        control: &mut ParseControl,
        stopped: StoppedAppend,
        error: SessionError,
    ) -> SessionError {
        let StoppedAppend {
            index,
            old_slot,
            pending,
            original,
            discarded,
        } = stopped;
        let mut refused: Option<SessionError> = None;
        match (pending, discarded) {
            (Some(mut allocation), Some(value)) => {
                keep_first(
                    &mut refused,
                    control.retain_failed_text_append_ticket(&mut allocation, &value),
                );
                if self.failed_text_append.is_none() {
                    self.failed_text_append = Some(FailedTextAppend { value, allocation });
                } else {
                    // The terminal owner is taken: this buffer is released
                    // here instead of replacing it.
                    keep_first(&mut refused, Err(latch_registry_row(control)));
                    drop(value);
                    keep_first(&mut refused, control.release_document_string(allocation));
                }
            }
            // The failure came before the backing existed: only the ticket.
            (Some(allocation), None) => {
                keep_first(&mut refused, control.release_document_string(allocation));
            }
            (None, Some(value)) => {
                keep_first(&mut refused, Err(latch_registry_row(control)));
                drop(value);
            }
            (None, None) => {}
        }
        if let Some(mut allocation) = original {
            let restored = match self.construction.node_at(index).map(|record| record.kind()) {
                Some(HtmlNodeKind::Text(value)) => {
                    keep_first(
                        &mut refused,
                        control.restore_text_append_ticket(&mut allocation, value),
                    );
                    match self.payloads.values.get_mut(old_slot) {
                        Some(
                            row @ (RetainedPayload::HistoricalString { .. }
                            | RetainedPayload::Retired),
                        ) => {
                            let value = value.clone();
                            let origins = match mem::replace(row, RetainedPayload::Retired) {
                                RetainedPayload::HistoricalString { origins } => Some(origins),
                                _ => None,
                            };
                            *row = RetainedPayload::String {
                                value,
                                allocation,
                                origins,
                                location: StringLocation::Text(index),
                            };
                            None
                        }
                        _ => Some(allocation),
                    }
                }
                _ => Some(allocation),
            };
            if let Some(allocation) = restored {
                // No row can take the ticket back; it is credited here.
                keep_first(&mut refused, Err(latch_registry_row(control)));
                keep_first(&mut refused, control.release_document_string(allocation));
            }
        }
        refused.unwrap_or(error)
    }

    pub(super) fn append_registered_text(
        &mut self,
        control: &mut ParseControl,
        append: RegisteredTextAppend,
    ) -> Result<TextInsertion, SessionError> {
        let RegisteredTextAppend {
            parent,
            index,
            old_slot,
            incoming_slot,
            incoming,
            dispose_old,
        } = append;
        control.charge(WorkCategory::RecordVisit, 2)?;
        let HtmlNodeKind::Text(old) = self
            .construction
            .node_at(index)
            .ok_or(SessionError::ForeignNode)?
            .kind()
        else {
            return Err(SessionError::WrongToken);
        };
        let offset = old.len();
        let added = incoming.len();
        // Two live vectors' lengths: their sum is bounded by the address space.
        let total = offset
            .checked_add(added)
            .ok_or_else(|| latch_registry_row(control))?;
        let node = self.node(control, index)?;
        // Actual registry row/ticket restoration on both outcomes is prepaid.
        // Moving the exclusive owner is bounded inline control metadata.
        payload_write::<RetainedPayload>(control, 2)?;
        payload_write::<Option<FailedTextAppend>>(control, 1)?;
        payload_write::<Option<usize>>(control, 1)?;
        payload_write::<StringLocation>(control, 1)?;
        // At most registry-strong drop, incoming drop, discarded-original
        // drop, and the final registry clone; Weak work is ticket-owned.
        control.charge(WorkCategory::ProviderCall, 4)?;
        // Prepay the actual two native-view headers used during commit/recovery.
        control.charge(WorkCategory::RecordVisit, 2)?;
        control.charge(
            WorkCategory::CopyByte,
            const { (2 * mem::size_of::<DocumentString>()) as u64 },
        )?;
        // The exclusive owner is taken before anything changes, and goes back
        // with the refusal of anything that follows until the append begins.
        let Some(storage) = self.construction.owner.take() else {
            return Err(latch_registry_row(control));
        };
        let mut original_allocation = if dispose_old {
            match self.take_original_ticket(control, old_slot) {
                Ok(ticket) => Some(ticket),
                Err(error) => {
                    self.construction.owner = Some(storage);
                    return Err(error);
                }
            }
        } else {
            None
        };
        let mut operation = Some(Append::begin(storage, node.handle, incoming));
        let mut pending_allocation = None;
        let mut merged_group = None;
        let mut merged_slot = None;
        let mut children_changed = false;
        let mut result = (|| {
            loop {
                // Same fixed descriptor/header admission as the generic
                // PendingConstruction driver: phase, target and Vec headers.
                control.charge(WorkCategory::RecordVisit, 3)?;
                let Some(owner) = operation.as_mut() else {
                    return Err(latch_registry_row(control));
                };
                let preserve = owner.preserves_original();
                let Some(action) = owner
                    .next_action()
                    .map_err(|error| construction_error(control, error))?
                else {
                    break;
                };
                let description = action.description();
                let mut minimum = None;
                let mut expected_capacity = None;
                match description {
                    Description::InitializeBacking(_) => {
                        // The wrapper preparation charges its actual Arc provider.
                        admit_work(control, description.work(), true)?;
                        pending_allocation = Some(control.prepare_text_append_allocation()?);
                    }
                    Description::Reserve {
                        before,
                        required_capacity,
                        ..
                    } => {
                        let Some(allocation) = (if preserve {
                            pending_allocation.as_ref()
                        } else {
                            original_allocation.as_ref()
                        }) else {
                            return Err(latch_registry_row(control));
                        };
                        let target = control.prepare_text_append_reserve(
                            allocation,
                            before,
                            required_capacity,
                        )?;
                        minimum = Some(target);
                        expected_capacity = Some(target);
                        admit_work(control, description.work(), false)?;
                    }
                    Description::CopyPrefix { units, .. } | Description::CopyTail { units, .. } => {
                        admit_work(control, description.work(), false)?;
                        let Some(allocation) = (if preserve {
                            pending_allocation.as_mut()
                        } else {
                            original_allocation.as_mut()
                        }) else {
                            return Err(latch_registry_row(control));
                        };
                        control.prepare_text_append_units(allocation, units)?;
                    }
                    Description::Preflight { .. } => {
                        merged_group = self.merge_field_groups(
                            control,
                            old_slot,
                            incoming_slot,
                            offset,
                            total,
                        )?;
                        self.prepare_payload_slot_owned(control)?;
                        merged_slot = Some(self.payloads.values.len());
                        self.payloads.values.push(RetainedPayload::Retired);
                        let Some(view) = action.storage_view() else {
                            return Err(latch_registry_row(control));
                        };
                        children_changed = self.textarea_slot(control, parent)?.is_some()
                            || Self::html_local_in(control, view, parent, "script")?
                            || Self::html_local_in(control, view, parent, "style")?;
                        if children_changed {
                            payload_write::<Option<FormEffects>>(control, 1)?;
                        }
                        payload_write::<HtmlParseResource>(control, 1)?;
                        admit_work(control, description.work(), false)?;
                    }
                    _ => admit_work(control, description.work(), false)?,
                }
                let performed = match &description {
                    // A test makes the backing fail: it was never created, so
                    // the action goes unperformed and the operation reports it.
                    #[cfg(test)]
                    Description::InitializeBacking(_) if control.take_text_append_backing_fault() => {
                        Err(HtmlConstructionError::AllocationFailed)
                    }
                    _ => action.perform(minimum),
                };
                match performed {
                    Ok(observation) => {
                        if matches!(
                            description,
                            Description::InitializeBacking(_) | Description::Reserve { .. }
                        ) {
                            let Some(allocation) = (if observation.preserves_original() {
                                pending_allocation.as_mut()
                            } else {
                                original_allocation.as_mut()
                            }) else {
                                return Err(latch_registry_row(control));
                            };
                            let Some(capacity) = observation.capacity() else {
                                return Err(latch_registry_row(control));
                            };
                            control.record_text_append_capacity(
                                allocation,
                                capacity,
                                expected_capacity.unwrap_or(0),
                            )?;
                            if matches!(description, Description::InitializeBacking(_)) {
                                let Some(backing) = observation.backing() else {
                                    return Err(latch_registry_row(control));
                                };
                                control.bind_text_append_pending(allocation, backing)?;
                            }
                        }
                        #[cfg(test)]
                        if matches!(description, Description::CopyTail { .. })
                            && let Some(cancellation) = self.cancel_after_text_append_chunk.take()
                        {
                            cancellation.cancel();
                        }
                        control.checkpoint()?;
                        observation.accept();
                    }
                    Err(error) => {
                        if let Some(expected) = expected_capacity {
                            let Some(owner) = operation.as_ref() else {
                                return Err(latch_registry_row(control));
                            };
                            let Some(allocation) = (if preserve {
                                pending_allocation.as_mut()
                            } else {
                                original_allocation.as_mut()
                            }) else {
                                return Err(latch_registry_row(control));
                            };
                            let Some(capacity) = owner.capacity() else {
                                return Err(latch_registry_row(control));
                            };
                            control.record_text_append_capacity(allocation, capacity, expected)?;
                        }
                        return Err(match error {
                            HtmlConstructionError::AllocationFailed => control
                                .construction_allocation_failed(
                                    expected_capacity
                                        .unwrap_or(0)
                                        .saturating_mul(mem::size_of::<u16>()),
                                )
                                .into(),
                            error => construction_error(control, error),
                        });
                    }
                }
            }
            let Some(allocation) = pending_allocation.as_mut().or(original_allocation.as_mut())
            else {
                return Err(latch_registry_row(control));
            };
            control.commit_text_append_domain(allocation)?;
            // Final fallible retirement preparation; tail remains unobservable
            // under the exclusive owner and is truncated on refusal.
            self.retire_text_append_group(control, old_slot, index)?;
            Ok(())
        })();
        let ready = match &result {
            Ok(()) => match Append::try_ready(&mut operation) {
                Ok(ready) => Some(ready),
                Err(error) => {
                    // Not ready after all: abort it like any other failure.
                    result = Err(construction_error(control, error));
                    None
                }
            },
            Err(_) => None,
        };
        let completion = match ready {
            Some(ready) => ready.commit(),
            None => {
                let Some(operation) = operation.take() else {
                    // The exclusive owner went with the operation; the tickets
                    // are all that is left to hand back.
                    let error = latch_registry_row(control);
                    let tickets = CommittedTickets {
                        pending: pending_allocation,
                        original: original_allocation,
                        discarded: None,
                    };
                    tickets.release(control);
                    return Err(error);
                };
                operation.abort()
            }
        };
        let (storage, handle, incoming, discarded) = completion.into_parts();
        self.construction.owner = Some(storage);
        drop(incoming);
        match result {
            Ok(()) => {
                let mut tickets = CommittedTickets {
                    pending: pending_allocation,
                    original: original_allocation,
                    discarded,
                };
                let adoption = Adoption {
                    index,
                    old_slot,
                    slot: merged_slot,
                    origins: merged_group,
                    dispose_old,
                };
                if let Err(error) = self.adopt_committed_append(control, &mut tickets, adoption) {
                    // The latched refusal is reported; whatever the row did
                    // not take is credited rather than dropped.
                    tickets.release(control);
                    return Err(error);
                }
                if children_changed {
                    self.effects.form = Some(FormEffects {
                        root: parent,
                        old_parent: None,
                        new_parent: Some(parent),
                        stage: FormEffectStage::InsertionChildren,
                        insert: false,
                        cause: InsertionCause::NonParser,
                        next: None,
                        pass: FormEffectPass::Local,
                    });
                    self.drain_form_effects(control)?;
                }
                Ok(TextInsertion {
                    _node: ParserNode { handle, index },
                    _offset: offset,
                })
            }
            Err(error) => Err(self.settle_stopped_append(
                control,
                StoppedAppend {
                    index,
                    old_slot,
                    pending: pending_allocation,
                    original: original_allocation,
                    discarded,
                },
                error,
            )),
        }
    }
}

/// Where a committed append registers its string.
struct Adoption {
    index: usize,
    old_slot: usize,
    slot: Option<usize>,
    origins: Option<provenance::OriginGroupRef>,
    dispose_old: bool,
}

/// What a stopped append hands back.
struct StoppedAppend {
    index: usize,
    old_slot: usize,
    pending: Option<DocumentStringAllocation>,
    original: Option<DocumentStringAllocation>,
    discarded: Option<DocumentString>,
}
