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
    control.charge(
        WorkCategory::ProviderCall,
        (work.provider_calls - usize::from(wrapper_prepared)) as u64,
    )?;
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
    pub(super) fn release(self, control: &mut ParseControl) {
        drop(self.value);
        control
            .release_document_string(self.allocation)
            .expect("actual stopped append ticket");
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

impl SessionStorage {
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
            (2 * mem::size_of::<DocumentString>()) as u64,
        )?;
        let mut original_allocation = if dispose_old {
            let RetainedPayload::String {
                value, allocation, ..
            } = &mut self.payloads.values[old_slot]
            else {
                return Err(SessionError::WrongToken);
            };
            control.suspend_text_append_ticket(allocation, value)?;
            let RetainedPayload::String {
                value,
                allocation,
                origins,
                ..
            } = mem::replace(
                &mut self.payloads.values[old_slot],
                RetainedPayload::Retired,
            )
            else {
                unreachable!()
            };
            drop(value);
            if let Some(origins) = origins {
                self.payloads.values[old_slot] = RetainedPayload::HistoricalString { origins };
            }
            Some(allocation)
        } else {
            None
        };
        let mut operation = Some(Append::begin(
            self.construction
                .owner
                .take()
                .expect("exclusive construction owner"),
            node.handle,
            incoming,
        ));
        let mut pending_allocation = None;
        let mut merged_group = None;
        let mut merged_slot = None;
        let mut children_changed = false;
        let result = (|| {
            loop {
                // Same fixed descriptor/header admission as the generic
                // PendingConstruction driver: phase, target and Vec headers.
                control.charge(WorkCategory::RecordVisit, 3)?;
                let preserve = operation
                    .as_ref()
                    .expect("append owner")
                    .preserves_original();
                let Some(action) = operation
                    .as_mut()
                    .expect("append owner")
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
                        let allocation = if preserve {
                            pending_allocation.as_ref()
                        } else {
                            original_allocation.as_ref()
                        }
                        .expect("actual append allocation");
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
                        let allocation = if preserve {
                            pending_allocation.as_mut()
                        } else {
                            original_allocation.as_mut()
                        }
                        .expect("actual append allocation");
                        control.prepare_text_append_units(allocation, units)?;
                    }
                    Description::Preflight { .. } => {
                        merged_group = self.merge_field_groups(
                            control,
                            old_slot,
                            incoming_slot,
                            offset,
                            offset + added,
                        )?;
                        self.prepare_payload_slot_owned(control)?;
                        merged_slot = Some(self.payloads.values.len());
                        self.payloads.values.push(RetainedPayload::Retired);
                        children_changed = self.textarea_slot(control, parent)?.is_some()
                            || Self::html_local_in(
                                control,
                                action.storage_view().expect("actual pre-tail storage"),
                                parent,
                                "script",
                            )?
                            || Self::html_local_in(
                                control,
                                action.storage_view().expect("actual pre-tail storage"),
                                parent,
                                "style",
                            )?;
                        if children_changed {
                            payload_write::<Option<FormEffects>>(control, 1)?;
                        }
                        payload_write::<HtmlParseResource>(control, 1)?;
                        admit_work(control, description.work(), false)?;
                    }
                    _ => admit_work(control, description.work(), false)?,
                }
                let performed = action.perform(minimum);
                match performed {
                    Ok(observation) => {
                        if matches!(
                            description,
                            Description::InitializeBacking(_) | Description::Reserve { .. }
                        ) {
                            let allocation = if observation.preserves_original() {
                                pending_allocation.as_mut()
                            } else {
                                original_allocation.as_mut()
                            }
                            .expect("actual append ticket");
                            control.record_text_append_capacity(
                                allocation,
                                observation.capacity().expect("actual backing"),
                                expected_capacity.unwrap_or(0),
                            )?;
                            if matches!(description, Description::InitializeBacking(_)) {
                                control.bind_text_append_pending(
                                    allocation,
                                    observation.backing().expect("real pending backing"),
                                )?;
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
                            let owner = operation.as_ref().expect("failed append owner");
                            let allocation = if preserve {
                                pending_allocation.as_mut()
                            } else {
                                original_allocation.as_mut()
                            }
                            .expect("actual append ticket");
                            control.record_text_append_capacity(
                                allocation,
                                owner.capacity().expect("retained actual bank"),
                                expected,
                            )?;
                        }
                        return Err(match error {
                            HtmlConstructionError::AllocationFailed => control
                                .construction_allocation_failed(
                                    expected_capacity.unwrap_or(0) * mem::size_of::<u16>(),
                                )
                                .into(),
                            error => construction_error(control, error),
                        });
                    }
                }
            }
            let allocation = pending_allocation
                .as_mut()
                .or(original_allocation.as_mut())
                .expect("completed append allocation");
            control.commit_text_append_domain(allocation)?;
            // Final fallible retirement preparation; tail remains unobservable
            // under the exclusive owner and is truncated on refusal.
            self.retire_text_append_group(control, old_slot, index)?;
            Ok(())
        })();
        let completion = match result {
            Ok(()) => Append::try_ready(&mut operation)
                .expect("all append observations accepted")
                .commit(),
            Err(_) => operation.take().expect("failed append owner").abort(),
        };
        let (storage, handle, incoming, discarded) = completion.into_parts();
        self.construction.owner = Some(storage);
        drop(incoming);
        match result {
            Ok(()) => {
                let allocation = if let Some(mut allocation) = pending_allocation.take() {
                    let value = match self
                        .construction
                        .node_at(index)
                        .expect("committed text")
                        .kind()
                    {
                        HtmlNodeKind::Text(value) => value,
                        _ => unreachable!(),
                    };
                    control.restore_text_append_ticket(&mut allocation, value);
                    drop(discarded);
                    if let Some(old) = original_allocation.take() {
                        control.release_document_string(old)?;
                    }
                    allocation
                } else {
                    assert!(discarded.is_none());
                    let mut allocation =
                        original_allocation.take().expect("reused original ticket");
                    let value = match self
                        .construction
                        .node_at(index)
                        .expect("committed text")
                        .kind()
                    {
                        HtmlNodeKind::Text(value) => value,
                        _ => unreachable!(),
                    };
                    control.restore_text_append_ticket(&mut allocation, value);
                    allocation
                };
                let value = match self
                    .construction
                    .node_at(index)
                    .expect("committed text")
                    .kind()
                {
                    HtmlNodeKind::Text(value) => value.clone(),
                    _ => unreachable!(),
                };
                let slot = merged_slot.expect("admitted initialized append destination");
                if !dispose_old {
                    self.bind_string(old_slot, StringLocation::Detached)?;
                }
                self.payloads.values[slot] = RetainedPayload::String {
                    value,
                    allocation,
                    origins: merged_group,
                    location: StringLocation::Text(index),
                };
                self.payloads.join_text(index, slot);
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
            Err(error) => {
                if let Some(mut allocation) = pending_allocation {
                    let value = discarded.expect("actual initialized pending append backing");
                    control.retain_failed_text_append_ticket(&mut allocation, &value);
                    assert!(
                        self.failed_text_append.is_none(),
                        "one terminal append owner"
                    );
                    self.failed_text_append = Some(FailedTextAppend { value, allocation });
                } else {
                    assert!(discarded.is_none());
                }
                if let Some(mut allocation) = original_allocation {
                    let value = match self
                        .construction
                        .node_at(index)
                        .expect("restored text")
                        .kind()
                    {
                        HtmlNodeKind::Text(value) => value,
                        _ => unreachable!(),
                    };
                    control.restore_text_append_ticket(&mut allocation, value);
                    let value = value.clone();
                    let origins = match mem::replace(
                        &mut self.payloads.values[old_slot],
                        RetainedPayload::Retired,
                    ) {
                        RetainedPayload::HistoricalString { origins } => Some(origins),
                        RetainedPayload::Retired => None,
                        _ => unreachable!(),
                    };
                    self.payloads.values[old_slot] = RetainedPayload::String {
                        value,
                        allocation,
                        origins,
                        location: StringLocation::Text(index),
                    };
                }
                Err(error)
            }
        }
    }
}
