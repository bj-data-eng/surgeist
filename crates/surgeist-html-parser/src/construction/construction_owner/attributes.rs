//! Session composition of the exclusive HTML attribute owner and original tickets.
use super::*;
use surgeist_html::{
    bounded::construction::{
        HtmlConstructionAttributeCompletion, HtmlConstructionAttributeDescription,
        HtmlConstructionAttributeEdit, HtmlConstructionAttributeEditMode,
        HtmlConstructionAttributeRetirementDescription, HtmlConstructionAttributeSlot,
        HtmlConstructionReadyAttributeSlot,
    },
    values::Attribute,
};
#[cfg(test)]
mod tests;
use crate::parser::control::{SharedAllocation, VectorCapacity};
/// Attribute lists at least this long are edited through a table of their names, so that the
/// check for a repeated name costs work in proportion to the list instead of its square. A
/// shorter list compares every pair, which is cheaper than building the table.
pub(in crate::construction) const NAME_INDEX_MIN_ENTRIES: usize = 32;
/// Names that effects of the attribute being installed may add to the list: the table is sized
/// for them too, so that they are looked up in it like the rest.
pub(in crate::construction) const NAME_INDEX_SPARE_ENTRIES: usize = 8;

impl ConstructionOwner {
    /// Makes sure the table of attribute names can hold `entries` names, reserving its slots
    /// through the original controller. Nothing is built: an edit that borrows the table clears
    /// and fills it. A count too large to size a table leaves it as it is, and the edit then
    /// compares every pair.
    pub(in crate::construction) fn provide_name_index(
        &mut self,
        control: &mut ParseControl,
        entries: usize,
    ) -> Result<(), SessionError> {
        let Some(slots) = HtmlAttributeNameIndex::slots_required(entries) else {
            return Ok(());
        };
        if self
            .name_index
            .as_ref()
            .is_some_and(|index| index.capacity() >= slots)
        {
            return Ok(());
        }
        self.retire_name_index(control)?;
        let (mut vector, mut ticket) = control.new_vector::<HtmlNameIndexSlot>()?;
        if let Err(stop) = control.reserve_vector(&mut vector, &mut ticket, slots) {
            // The reserve may have changed the capacity before it stopped.
            release_vector(control, vector, ticket)?;
            return Err(stop.into());
        }
        self.name_index = Some(HtmlAttributeNameIndex::new(vector));
        self.name_index_slots = Some(ticket);
        Ok(())
    }
    /// Releases the table of attribute names, crediting its slots. Nothing happens while an edit
    /// has borrowed it or when there is none.
    pub(in crate::construction) fn retire_name_index(
        &mut self,
        control: &mut ParseControl,
    ) -> Result<(), SessionError> {
        let (Some(index), Some(ticket)) = (self.name_index.take(), self.name_index_slots.take())
        else {
            return Ok(());
        };
        release_vector(control, index.into_storage(), ticket)
    }
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
    /// Takes a lent edit back. A live edit already in the slot stays there and
    /// is never displaced: the registry row is latched, and the edit that has
    /// no place is ended so its witnesses and plan bank are still credited.
    pub(in crate::construction) fn put_attribute_edit(
        &mut self,
        control: &mut ParseControl,
        group: HtmlConstructionAttributeEdit,
    ) -> Result<(), SessionError> {
        if self.attribute_edit.is_some() {
            let invariant = latch_registry_row(control);
            // A refusal of the retirement is secondary to the invariant above.
            let _ = self.adopt_attribute_completion(control, group.abort());
            return Err(invariant);
        }
        self.attribute_edit = Some(group);
        Ok(())
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
        control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        control.charge(HtmlParseWorkCategory::ProviderCall, 1)?;
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
        let mut group = match self.name_index.take() {
            Some(index) => {
                HtmlConstructionAttributeEdit::begin_indexed(storage, handle, mode, index)
            }
            None => HtmlConstructionAttributeEdit::begin(storage, handle, mode),
        };
        let result = (|| {
            while let Some(action) = group
                .next_action()
                .map_err(|e| construction_error(control, e))?
            {
                let description = action.description();
                description
                    .charge(0, control)
                    .map_err(|error| charge_fault(control, error))?;
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
                        return Err(latch_owner_refusal(control));
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
        let Some(edit) = self.attribute_edit.take() else {
            return Err(latch_registry_row(control));
        };
        match edit.try_finish() {
            Ok(completion) => self.adopt_attribute_completion(control, completion),
            Err(refusal) => {
                // A refused finish hands the edit back, which stays where it was.
                let (edit, reason) = refusal.into_parts();
                self.attribute_edit = Some(edit);
                Err(construction_error(control, reason))
            }
        }
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
        // The table goes back to where `provide_name_index` and `retire_name_index` find it.
        let (storage, mut retirement, index) = completion.into_parts_with_index();
        if let Some(index) = index {
            self.name_index = Some(index);
        }
        self.owner = Some(storage);
        let mut refused = None;
        while let Some(action) = retirement.next_action() {
            match action.description() {
                HtmlConstructionAttributeRetirementDescription::NameWitness {
                    auxiliary_entries,
                    ..
                } => {
                    action.perform();
                    let cleaned = control.cleanup_auxiliary_entries(auxiliary_entries as u64);
                    #[cfg(test)]
                    inject_walk_refusal(control, WalkSite::AttributeWitness, &mut refused);
                    keep_first(&mut refused, cleaned);
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
                // The original reserve policy below admits the same actual
                // allocator provider once; it is not a second provider call.
                let prepaid = u64::from(matches!(
                    description,
                    HtmlConstructionAttributeDescription::ReservePlan { .. }
                ));
                description
                    .charge(prepaid, control)
                    .map_err(|error| charge_fault(control, error))?;
                let prior = description.prior_index();
                // A plan reservation doubles the plan, up to the candidate count it will hold, so
                // planning N names requests O(log N) reallocations. The capacity depends only on
                // the plan and the candidate count, never on a limit, and is admitted here,
                // before it is allocated: a limit that cannot hold it refuses the reservation.
                let (expected, reserved) =
                    if let HtmlConstructionAttributeDescription::ReservePlan {
                        allocation,
                        minimum_capacity,
                        ..
                    } = description
                    {
                        let capacity = allocation
                            .capacity()
                            .saturating_mul(2)
                            .min(names.len())
                            .max(minimum_capacity);
                        control.prepare_construction_reserve(
                            allocation,
                            capacity,
                            attribute_plan.as_ref(),
                        )?;
                        ticket_work(control)?;
                        let Some(expected) = capacity.checked_mul(allocation.element_size()) else {
                            return Err(control
                                .stop_representation(crate::HtmlParseResource::ManagedCapacityBytes)
                                .into());
                        };
                        (Some(expected), Some(capacity))
                    } else {
                        (None, None)
                    };
                let prior_name = match prior {
                    Some(prior) => match names.get(prior) {
                        Some(name) => Some(name.name.value.text()),
                        None => return Err(latch_registry_row(control)),
                    },
                    None => None,
                };
                let performed = match reserved {
                    Some(capacity) => action.perform_with_capacity(prior_name, capacity),
                    None => action.perform(prior_name),
                };
                match performed {
                    Ok(observation) => {
                        if let Some(expected) = expected {
                            let Some(snapshot) = observation.allocation() else {
                                return Err(latch_owner_refusal(control));
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
                        return Err(if error.is_allocation_failed() {
                            control
                                .construction_allocation_failed(expected.unwrap_or(0))
                                .into()
                        } else {
                            construction_error(control, error)
                        });
                    }
                }
            }
            let Some(candidate) = candidate.take() else {
                return Err(latch_registry_row(control));
            };
            candidate
                .try_finish()
                .map_err(|refusal| construction_error(control, refusal.into_parts().1))
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
        description
            .charge(0, control)
            .map_err(|error| charge_fault(control, error))?;
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
                        return Err(latch_owner_refusal(control));
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
                return Err(if error.is_allocation_failed() {
                    control
                        .construction_allocation_failed(expected.unwrap_or(0))
                        .into()
                } else {
                    construction_error(control, error)
                });
            }
        }
    }
    let Some(slot) = slot else {
        return Err(latch_registry_row(control));
    };
    slot.try_ready()
        .map_err(|refusal| construction_error(control, refusal.into_parts().1))
}
