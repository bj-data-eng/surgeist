//! Session composition of the exclusive HTML attribute owner and original tickets.
use super::*;
#[cfg(test)]
mod tests;
use crate::parser::control::{SharedAllocation, VectorCapacity};
impl ConstructionOwner {
    /// Lends the active edit to the caller, who returns it with
    /// [`Self::put_attribute_edit`]. The edit holds the storage meanwhile, so
    /// a second edit cannot begin and nothing else can occupy the slot.
    pub(in crate::construction) fn take_attribute_edit(
        &mut self,
        control: &mut ParseControl,
    ) -> Result<HtmlConstructionAttributeEdit, SessionError> {
        self.attribute_edit
            .take()
            .ok_or_else(|| latch_registry_row(control))
    }
    pub(in crate::construction) fn put_attribute_edit(
        &mut self,
        group: HtmlConstructionAttributeEdit,
    ) {
        self.attribute_edit = Some(group);
    }
    pub(in crate::construction) fn has_attribute_edit(&self) -> bool {
        self.attribute_edit.is_some()
    }
    pub(in crate::construction) fn attribute_edit(
        &mut self,
        control: &mut ParseControl,
    ) -> Result<&mut HtmlConstructionAttributeEdit, SessionError> {
        self.attribute_edit
            .as_mut()
            .ok_or_else(|| latch_registry_row(control))
    }
    pub(in crate::construction) fn begin_attributes(
        &mut self,
        control: &mut ParseControl,
        target: usize,
        mode: HtmlConstructionAttributeEditMode,
    ) -> Result<(), SessionError> {
        control.require_identity(&self.control)?;
        // Only one construction writer: a second edit would need the storage
        // the first one holds. Nothing is taken before this is checked.
        if self.attribute_edit.is_some() {
            return Err(latch_registry_row(control));
        }
        control.charge(WorkCategory::RecordVisit, 1)?;
        control.charge(WorkCategory::ProviderCall, 1)?;
        let handle = self
            .storage(control)?
            .node_at(target)
            .ok_or(SessionError::ForeignNode)?
            .handle();
        // Inline owner/phase/counter initialization is control bookkeeping;
        // real backing, plan, witness and slot work is admitted by each action.
        let Some(storage) = self.owner.take() else {
            return Err(latch_registry_row(control));
        };
        let mut group = match HtmlConstructionAttributeEdit::begin(storage, handle, mode) {
            Ok(group) => group,
            Err(rejection) => {
                // A refusal returns the unchanged storage.
                let (storage, _, _, reason) = rejection.into_parts();
                self.owner = Some(storage);
                return Err(construction_error(control, reason));
            }
        };
        let result = (|| {
            while let Some(action) = group
                .next_action()
                .map_err(|e| construction_error(control, e))?
            {
                let description = action.description();
                control.admit_attribute_work(description.work())?;
                let report = matches!(
                    description,
                    HtmlConstructionAttributeDescription::PrepareAllocationReport(_)
                );
                if report {
                    ticket_work(control)?;
                }
                let observation = action
                    .perform()
                    .map_err(|e| construction_error(control, e))?;
                if report {
                    let Some(snapshot) = observation.allocation() else {
                        return Err(latch_registry_row(control));
                    };
                    control.observe_construction_allocation(
                        snapshot,
                        &mut self.attribute_plan,
                        0,
                    )?;
                }
                observation.accept();
            }
            Ok(())
        })();
        self.attribute_edit = Some(group);
        #[cfg(test)]
        if std::mem::take(&mut self.hide_next_attribute_edit) {
            self.hidden_attribute_edit = self.attribute_edit.take();
        }
        if result.is_err() {
            self.abort_attributes(control)?;
        }
        result
    }
    pub(in crate::construction) fn finish_attributes(
        &mut self,
        control: &mut ParseControl,
    ) -> Result<(), SessionError> {
        let completion = HtmlConstructionAttributeEdit::try_finish(&mut self.attribute_edit)
            .map_err(|e| construction_error(control, e))?;
        self.adopt_attribute_completion(control, completion)
    }
    pub(in crate::construction) fn abort_attributes(
        &mut self,
        control: &mut ParseControl,
    ) -> Result<(), SessionError> {
        if let Some(group) = self.attribute_edit.take() {
            self.adopt_attribute_completion(control, group.abort())?;
        }
        Ok(())
    }
    /// Takes the storage back and retires the edit's witnesses and plan bank.
    /// A refusal is remembered and the retirement continues, since it only
    /// advances by performing its actions; the first refusal is returned.
    fn adopt_attribute_completion(
        &mut self,
        control: &mut ParseControl,
        completion: HtmlConstructionAttributeCompletion,
    ) -> Result<(), SessionError> {
        let (storage, mut retirement) = completion.into_parts();
        self.owner = Some(storage);
        let mut refused = None;
        while let Some(action) = retirement.next_action() {
            match action.description() {
                HtmlConstructionAttributeRetirementDescription::NameWitness {
                    auxiliary_entries,
                    ..
                } => {
                    action.perform();
                    keep_first(
                        &mut refused,
                        control.cleanup_auxiliary_entries(auxiliary_entries as u64),
                    );
                }
                HtmlConstructionAttributeRetirementDescription::Allocation => {
                    let ticket = self.attribute_plan.take();
                    let checked = match (ticket.as_ref(), action.allocation()) {
                        (Some(ticket), Some(snapshot)) => control
                            .require_construction_retirement(ticket, snapshot)
                            .map_err(SessionError::from),
                        _ => Err(latch_registry_row(control)),
                    };
                    let receipt = action.perform();
                    match (checked, ticket, receipt) {
                        (Ok(()), Some(ticket), Some(receipt)) => keep_first(
                            &mut refused,
                            control.retire_construction_allocation(ticket, receipt),
                        ),
                        (checked, ticket, _) => {
                            // The ticket stays registered rather than being
                            // dropped uncredited.
                            self.attribute_plan = ticket;
                            let impossible =
                                checked.err().unwrap_or_else(|| latch_registry_row(control));
                            keep_first(&mut refused, Err(impossible));
                        }
                    }
                }
            }
        }
        refused.map_or(Ok(()), Err)
    }
    pub(in crate::construction) fn plan_attribute_candidate(
        &mut self,
        control: &mut ParseControl,
        index: usize,
        names: &[crate::parser::tokenizer::token::Attribute],
    ) -> Result<bool, SessionError> {
        let Self {
            attribute_edit,
            attribute_plan,
            ..
        } = self;
        let Some(group) = attribute_edit.as_mut() else {
            return Err(latch_registry_row(control));
        };
        let Some(name) = names.get(index) else {
            return Err(latch_registry_row(control));
        };
        let mut candidate = Some(
            group
                .candidate(index, name.name.value.text())
                .map_err(|e| construction_error(control, e))?,
        );
        (|| {
            loop {
                let Some(pending) = candidate.as_mut() else {
                    return Err(latch_registry_row(control));
                };
                let Some(action) = pending
                    .next_action()
                    .map_err(|e| construction_error(control, e))?
                else {
                    break;
                };
                let description = action.description();
                let mut work = description.work();
                // The original reserve policy below admits the same actual
                // allocator provider once; it is not a second provider call.
                if matches!(
                    description,
                    HtmlConstructionAttributeDescription::ReservePlan { .. }
                ) {
                    let Some(calls) = work.provider_calls.checked_sub(1) else {
                        return Err(latch_registry_row(control));
                    };
                    work.provider_calls = calls;
                }
                control.admit_attribute_work(work)?;
                let prior = description.prior_index();
                let expected = if let HtmlConstructionAttributeDescription::ReservePlan {
                    allocation,
                    minimum_capacity,
                    ..
                } = description
                {
                    control.prepare_construction_reserve(
                        allocation,
                        minimum_capacity,
                        attribute_plan.as_ref(),
                    )?;
                    ticket_work(control)?;
                    let Some(expected) = minimum_capacity.checked_mul(allocation.element_size())
                    else {
                        return Err(control
                            .stop_representation(crate::HtmlParseResource::ManagedCapacityBytes)
                            .into());
                    };
                    Some(expected)
                } else {
                    None
                };
                let prior_name = match prior {
                    Some(prior) => match names.get(prior) {
                        Some(name) => Some(name.name.value.text()),
                        None => return Err(latch_registry_row(control)),
                    },
                    None => None,
                };
                match action.perform(prior_name) {
                    Ok(observation) => {
                        if let Some(expected) = expected {
                            let Some(snapshot) = observation.allocation() else {
                                return Err(latch_registry_row(control));
                            };
                            control.observe_construction_allocation(
                                snapshot,
                                attribute_plan,
                                expected,
                            )?;
                        }
                        observation.accept();
                    }
                    Err(error) => {
                        if let Some(expected) = expected {
                            let Some(failed) = candidate.as_ref() else {
                                return Err(latch_registry_row(control));
                            };
                            control.record_construction_allocation(
                                failed.allocation(),
                                attribute_plan,
                                expected,
                            )?;
                        }
                        return Err(if error == HtmlConstructionError::AllocationFailed {
                            control
                                .construction_allocation_failed(expected.unwrap_or(0))
                                .into()
                        } else {
                            construction_error(control, error)
                        });
                    }
                }
            }
            HtmlConstructionAttributeCandidate::try_finish(&mut candidate)
                .map_err(|e| construction_error(control, e))
        })()
    }
}
/// The returned ready owner keeps the same group and incoming slot exclusively
/// borrowed; the caller commits or drops it before restoring its original Weak.
pub(in crate::construction) fn prepare_attribute_slot<'a>(
    control: &mut ParseControl,
    slot: HtmlConstructionAttributeSlot<'a>,
    shared: &mut SharedAllocation<Vec<Attribute>>,
    allocation: &mut VectorCapacity<Attribute>,
) -> Result<HtmlConstructionReadyAttributeSlot<'a>, SessionError> {
    let backing = slot
        .shared_attributes()
        .map_err(|e| construction_error(control, e))?;
    control.prepare_controlled_attribute_edit(backing, shared, allocation)?;
    let mut slot = Some(slot);
    loop {
        let Some(pending) = slot.as_mut() else {
            return Err(latch_registry_row(control));
        };
        let Some(action) = pending
            .next_action()
            .map_err(|e| construction_error(control, e))?
        else {
            break;
        };
        let description = action.description();
        control.admit_attribute_work(description.work())?;
        let unique = matches!(
            description,
            HtmlConstructionAttributeDescription::UniqueBacking(_)
        );
        let expected = if let HtmlConstructionAttributeDescription::ReserveAttributes {
            before,
            minimum_capacity,
            ..
        } = description
        {
            Some(control.prepare_controlled_attribute_reserve(
                before,
                minimum_capacity,
                allocation,
            )?)
        } else {
            None
        };
        if unique || expected.is_some() {
            control.suspend_attribute_weak(shared);
        }
        match action.perform() {
            Ok(observation) => {
                if let Some(expected) = expected {
                    let Some(capacity) = observation.capacity() else {
                        return Err(latch_registry_row(control));
                    };
                    control.record_controlled_attribute_capacity(capacity, allocation, expected)?;
                    let backing = observation
                        .shared_attributes()
                        .map_err(|e| construction_error(control, e))?;
                    control.restore_attribute_weak(backing, shared);
                    control.checkpoint()?;
                }
                observation.accept();
                if unique {
                    break;
                }
            }
            Err(error) => {
                if let Some(expected) = expected {
                    let Some(failed) = slot.as_ref() else {
                        return Err(latch_registry_row(control));
                    };
                    control.record_controlled_attribute_capacity(
                        failed.capacity(),
                        allocation,
                        expected,
                    )?;
                }
                return Err(if error == HtmlConstructionError::AllocationFailed {
                    control
                        .construction_allocation_failed(expected.unwrap_or(0))
                        .into()
                } else {
                    construction_error(control, error)
                });
            }
        }
    }
    HtmlConstructionAttributeSlot::try_ready(&mut slot).map_err(|e| construction_error(control, e))
}
