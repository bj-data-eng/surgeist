//! Drives HTML's sole record owner through this session's original controller.
use super::*;
use crate::bounded::*;
use crate::parser::control::ConstructionAllocation;
mod attributes;
#[cfg(test)]
mod tests;
pub(super) use attributes::prepare_attribute_slot;

pub(super) struct ConstructionOwner {
    control: ControlIdentity,
    pub(super) owner: Option<HtmlConstructionStorage>,
    attribute_edit: Option<HtmlConstructionAttributeEdit>,
    attribute_plan: Option<ConstructionAllocation>,
    identity: Option<ConstructionAllocation>,
    records: Option<ConstructionAllocation>,
    children: Vec<Option<ConstructionAllocation>>,
    children_allocation: VectorCapacity<Option<ConstructionAllocation>>,
    wrapper: Option<ConstructionAllocation>,
    /// Hides the attribute edit the next `begin_attributes` starts, as if its
    /// custody slot had been emptied behind the owner's back.
    #[cfg(test)]
    hide_next_attribute_edit: bool,
    #[cfg(test)]
    hidden_attribute_edit: Option<HtmlConstructionAttributeEdit>,
}

#[derive(Clone, Copy)]
enum Bank {
    Identity,
    Records,
    Children(usize),
    Wrapper,
}

pub(super) fn construction_error(
    control: &mut ParseControl,
    error: HtmlConstructionError,
) -> SessionError {
    match error {
        HtmlConstructionError::ForeignBuilder | HtmlConstructionError::UnknownNode => {
            SessionError::ForeignNode
        }
        HtmlConstructionError::RootMove => SessionError::RootMove,
        HtmlConstructionError::AssociationMove => SessionError::AssociationMove,
        HtmlConstructionError::ParentCannotHaveChildren => SessionError::NotContainer,
        HtmlConstructionError::TemplateContentsRequired => SessionError::TemplateContentsRequired,
        HtmlConstructionError::WrongSibling | HtmlConstructionError::ParentChildConflict => {
            SessionError::WrongSibling
        }
        HtmlConstructionError::Cycle => SessionError::Cycle,
        HtmlConstructionError::SharedAllocationAliased => SessionError::Stop(
            control.fail_invariant(crate::parser::control::Invariant::SharedAllocationAliased),
        ),
        HtmlConstructionError::RepresentationExhausted => {
            SessionError::Stop(control.stop_representation(HtmlParseResource::ManagedCapacityBytes))
        }
        _ => SessionError::Stop(
            control.fail_invariant(crate::parser::control::Invariant::CapacityMismatch),
        ),
    }
}
pub(super) struct PendingConstruction {
    operation: Option<HtmlConstructionOperation>,
    before_edges: usize,
}
impl PendingConstruction {
    pub(super) fn action(
        &mut self,
        control: &mut ParseControl,
    ) -> Result<Option<HtmlConstructionAction<'_>>, SessionError> {
        control.charge(WorkCategory::RecordVisit, 3)?;
        let Some(operation) = self.operation.as_mut() else {
            return Err(latch_registry_row(control));
        };
        operation
            .next_action()
            .map_err(|error| construction_error(control, error))
    }
    pub(super) fn before_edges(&self) -> usize {
        self.before_edges
    }
}
fn ticket_work(control: &mut ParseControl) -> Result<(), SessionError> {
    control.charge(WorkCategory::ProviderCall, 2)?;
    control.charge(
        WorkCategory::CopyByte,
        mem::size_of::<ConstructionAllocation>() as u64,
    )?;
    control.charge(WorkCategory::RecordWrite, 1)?;
    Ok(())
}
impl ConstructionOwner {
    pub(super) fn new(control: &mut ParseControl) -> Result<Self, SessionError> {
        control.charge(WorkCategory::ProviderCall, 1)?;
        let mut initialization = Some(HtmlConstructionStorage::begin());
        let Some(owner) = initialization.as_mut() else {
            return Err(latch_registry_row(control));
        };
        let Some(action) = owner
            .next_action()
            .map_err(|error| construction_error(control, error))?
        else {
            return Err(latch_registry_row(control));
        };
        let HtmlConstructionActionDescription::InitializeIdentity {
            payload_bytes,
            initialized_bytes,
            provider_calls,
        } = action.description()
        else {
            return Err(latch_registry_row(control));
        };
        control.prepare_construction_share(payload_bytes)?;
        let Some(further_calls) = provider_calls.checked_sub(1) else {
            return Err(latch_registry_row(control));
        };
        control.charge(WorkCategory::ProviderCall, further_calls as u64)?;
        control.admit_atomic_layout(initialized_bytes)?;
        control.charge(WorkCategory::CopyByte, initialized_bytes as u64)?;
        ticket_work(control)?;
        let observation = action.perform();
        let mut identity = None;
        if let Err(stop) = control.observe_construction_allocation(
            observation.allocation(),
            &mut identity,
            payload_bytes,
        ) {
            drop(observation);
            if let Some(receipt) = initialization.take().and_then(|owner| owner.abort()) {
                let Some(ticket) = identity.take() else {
                    return Err(latch_registry_row(control));
                };
                control.retire_construction_allocation(ticket, receipt)?;
            }
            return Err(stop.into());
        }
        observation.accept();
        let owner = HtmlConstructionInitialization::try_ready(&mut initialization)
            .map_err(|error| construction_error(control, error))?
            .commit();
        let (children, children_allocation) = match control.new_vector() {
            Ok(value) => value,
            Err(stop) => {
                let mut retirement = owner.retire();
                // Every bank is retired even after an impossible one: the
                // walk only advances by performing its actions.
                let mut impossible = false;
                while let Some(action) = retirement
                    .next_action()
                    .map_err(|error| construction_error(control, error))?
                {
                    let is_identity = matches!(action.description(),HtmlConstructionRetirementDescription::Allocation(value) if matches!(value.role(),HtmlConstructionAllocationRole::Identity));
                    if let Some(receipt) = action.perform() {
                        if is_identity {
                            match identity.take() {
                                Some(ticket) => {
                                    control.retire_construction_allocation(ticket, receipt)?;
                                }
                                None => impossible = true,
                            }
                        } else if receipt.allocation().payload_bytes() != 0 {
                            // An empty initialization record bank holds nothing.
                            impossible = true;
                        }
                    }
                }
                if impossible {
                    return Err(latch_registry_row(control));
                }
                return Err(stop.into());
            }
        };
        Ok(Self {
            control: control.identity(),
            owner: Some(owner),
            attribute_edit: None,
            attribute_plan: None,
            identity,
            records: None,
            children,
            children_allocation,
            wrapper: None,
            #[cfg(test)]
            hide_next_attribute_edit: false,
            #[cfg(test)]
            hidden_attribute_edit: None,
        })
    }
    pub(super) fn len(&self, control: &mut ParseControl) -> Result<usize, SessionError> {
        Ok(self.storage(control)?.len())
    }
    /// The storage while it is at rest or lent to an attribute edit. It is
    /// absent only while a pending operation or a publication holds it, and a
    /// caller that reads it then has met an impossible custody state.
    fn resident(&self) -> Option<&HtmlConstructionStorage> {
        self.owner.as_ref().or_else(|| {
            self.attribute_edit
                .as_ref()
                .map(HtmlConstructionAttributeEdit::storage_view)
        })
    }
    pub(super) fn storage(
        &self,
        control: &mut ParseControl,
    ) -> Result<&HtmlConstructionStorage, SessionError> {
        self.resident().ok_or_else(|| latch_registry_row(control))
    }
    pub(super) fn reserve_child_tickets(
        &mut self,
        control: &mut ParseControl,
        additional: usize,
    ) -> Result<(), SessionError> {
        control.reserve_vector_geometric(
            &mut self.children,
            &mut self.children_allocation,
            additional,
        )?;
        Ok(())
    }
    /// Reserves HTML-owned record storage for `additional` new records with
    /// geometric spare capacity, so a document of N records costs O(N) bytes of
    /// reservation work instead of O(N^2).
    pub(super) fn reserve_records(
        &mut self,
        control: &mut ParseControl,
        additional: usize,
    ) -> Result<(), SessionError> {
        let len = self.len(control)?;
        let additional =
            control.geometric_construction_additional(self.records.as_ref(), len, additional)?;
        self.apply(
            control,
            HtmlConstructionCommand::ReserveRecords { additional },
            &[],
        )?;
        Ok(())
    }
    #[cfg(test)]
    pub(super) fn actual_record_layout(&self) -> (usize, usize) {
        let snapshot = self
            .records
            .as_ref()
            .expect("created actual record bank")
            .snapshot();
        (snapshot.capacity(), snapshot.element_size())
    }
    pub(super) fn usage(
        &self,
        control: &mut ParseControl,
    ) -> Result<HtmlConstructionUsage, SessionError> {
        Ok(self.storage(control)?.usage())
    }
    /// The record at `position`; storage that is not at rest holds no record
    /// a caller may name.
    pub(super) fn node_at(&self, position: usize) -> Option<HtmlConstructionNode<'_>> {
        self.resident()?.node_at(position)
    }
    fn allocation_slot(
        &mut self,
        control: &mut ParseControl,
        bank: Bank,
    ) -> Result<&mut Option<ConstructionAllocation>, SessionError> {
        match bank {
            Bank::Identity => Some(&mut self.identity),
            Bank::Records => Some(&mut self.records),
            Bank::Children(position) => self.children.get_mut(position),
            Bank::Wrapper => Some(&mut self.wrapper),
        }
        .ok_or_else(|| latch_registry_row(control))
    }
    pub(super) fn apply(
        &mut self,
        control: &mut ParseControl,
        command: HtmlConstructionCommand,
        requests: &[(HtmlParseResource, u64)],
    ) -> Result<Option<HtmlBuilderNode>, SessionError> {
        let committed = self.run(control, command, requests)?;
        self.adopt_mutation(control, committed)
    }
    pub(super) fn adopt_mutation(
        &mut self,
        control: &mut ParseControl,
        committed: HtmlConstructionCommit,
    ) -> Result<Option<HtmlBuilderNode>, SessionError> {
        match committed {
            HtmlConstructionCommit::Changed { storage, created } => {
                self.owner = Some(storage);
                Ok(created)
            }
            HtmlConstructionCommit::Published { result, lease } => {
                // A mutation never publishes. The published storage is taken
                // back through its lease, so the owner keeps every record.
                let invariant = latch_registry_row(control);
                let mut result = Some(result);
                let mut lease = Some(lease);
                if let Ok(recovery) = HtmlConstructionLease::try_recover(&mut result, &mut lease) {
                    // A refused recovery cannot displace the invariant above,
                    // which is already the reported first stop.
                    let _ = self.recovered(control, recovery);
                }
                Err(invariant)
            }
        }
    }
    pub(super) fn start(
        &mut self,
        control: &mut ParseControl,
        command: HtmlConstructionCommand,
    ) -> Result<PendingConstruction, SessionError> {
        control.charge(WorkCategory::RecordVisit, 1)?;
        // Moving bounded inline control metadata is not a payload copy in the
        // parser work model. Actual retained bank/slot writes and providers are
        // admitted by the operation's actions.
        let Some(storage) = self.owner.take() else {
            return Err(latch_registry_row(control));
        };
        let before_edges = storage.usage().edges();
        match storage.prepare(command) {
            Ok(operation) => Ok(PendingConstruction {
                operation: Some(operation),
                before_edges,
            }),
            Err(rejection) => {
                // A refusal returns the unchanged storage.
                let (storage, _, reason) = rejection.into_parts();
                self.owner = Some(storage);
                Err(construction_error(control, reason))
            }
        }
    }
    pub(super) fn perform_action(
        &mut self,
        control: &mut ParseControl,
        action: HtmlConstructionAction<'_>,
        requests: &[(HtmlParseResource, u64)],
        before_edges: usize,
    ) -> Result<bool, SessionError> {
        self.perform_action_with_commit(control, action, requests, before_edges, |_| Ok(()))
    }
    pub(super) fn perform_action_with_commit(
        &mut self,
        control: &mut ParseControl,
        action: HtmlConstructionAction<'_>,
        requests: &[(HtmlParseResource, u64)],
        before_edges: usize,
        before_commit: impl FnOnce(&mut ParseControl) -> Result<(), SessionError>,
    ) -> Result<bool, SessionError> {
        let mut allocation = None;
        let mut expected = 0;
        let mut reserved_capacity = None;
        let mut final_action = false;
        match action.description() {
            HtmlConstructionActionDescription::Inspect {
                record_visits,
                inspected_bytes,
            } => {
                control.charge(WorkCategory::RecordVisit, record_visits as u64)?;
                control.charge(WorkCategory::InspectByte, inspected_bytes as u64)?;
            }
            HtmlConstructionActionDescription::Compare { inspected_bytes } => {
                control.charge(WorkCategory::InspectByte, inspected_bytes as u64)?
            }
            HtmlConstructionActionDescription::PrepareAllocationReport {
                record_visits,
                provider_calls,
                copied_bytes,
            } => {
                control.charge(WorkCategory::RecordVisit, record_visits as u64)?;
                control.charge(WorkCategory::ProviderCall, provider_calls as u64)?;
                control.charge(WorkCategory::CopyByte, copied_bytes as u64)?;
            }
            HtmlConstructionActionDescription::Reserve {
                report_provider_calls,
                allocation: snapshot,
                required_len,
                minimum_capacity,
                growth,
            } => {
                control.charge(WorkCategory::RecordVisit, 2)?;
                let bank = snapshot
                    .child_position()
                    .map_or(Bank::Records, Bank::Children);
                // A geometric bank grows by the parser's own amortized plan,
                // which never reserves spare capacity the limits cannot hold.
                let capacity = match growth {
                    HtmlConstructionGrowth::Exact => minimum_capacity,
                    HtmlConstructionGrowth::Geometric => {
                        let slot = self.allocation_slot(control, bank)?;
                        let additional = control.geometric_construction_additional(
                            slot.as_ref(),
                            snapshot.initialized_len(),
                            required_len.saturating_sub(snapshot.initialized_len()),
                        )?;
                        snapshot
                            .initialized_len()
                            .checked_add(additional)
                            .map(|planned| planned.max(minimum_capacity))
                            .ok_or_else(|| {
                                control.stop_representation(HtmlParseResource::ManagedCapacityBytes)
                            })?
                    }
                };
                reserved_capacity =
                    matches!(growth, HtmlConstructionGrowth::Geometric).then_some(capacity);
                expected = capacity
                    .checked_mul(snapshot.element_size())
                    .ok_or_else(|| {
                        control.stop_representation(HtmlParseResource::ManagedCapacityBytes)
                    })?;
                let slot = self.allocation_slot(control, bank)?;
                control.prepare_construction_reserve(snapshot, capacity, slot.as_ref())?;
                ticket_work(control)?;
                control.charge(WorkCategory::ProviderCall, report_provider_calls as u64)?;
                allocation = Some(bank);
            }
            HtmlConstructionActionDescription::Share {
                report_provider_calls,
                allocation: snapshot,
            } => {
                expected = snapshot.element_size();
                control.prepare_construction_share(expected)?;
                ticket_work(control)?;
                control.charge(WorkCategory::ProviderCall, report_provider_calls as u64)?;
                allocation = Some(Bank::Wrapper);
            }
            HtmlConstructionActionDescription::ShareReferences {
                provider_calls,
                copied_bytes,
            } => {
                control.charge(WorkCategory::ProviderCall, provider_calls as u64)?;
                control.charge(WorkCategory::CopyByte, copied_bytes as u64)?;
            }
            HtmlConstructionActionDescription::InitializeChildren {
                provider_calls,
                copied_bytes,
                record_writes,
                position,
                ..
            } => {
                control.reserve_vector_geometric(
                    &mut self.children,
                    &mut self.children_allocation,
                    1,
                )?;
                control.charge(WorkCategory::CopyByte, copied_bytes as u64)?;
                control.charge(WorkCategory::RecordWrite, record_writes as u64)?;
                control.charge(
                    WorkCategory::CopyByte,
                    const {
                        (mem::size_of::<Vec<usize>>()
                            + mem::size_of::<Option<ConstructionAllocation>>())
                            as u64
                    },
                )?;
                control.charge(WorkCategory::RecordWrite, 2)?;
                ticket_work(control)?;
                // The real child-bank registry append lands at the next row.
                if position != self.children.len() {
                    return Err(latch_registry_row(control));
                }
                // The zero-capacity bank physically exists in the pending owner;
                // retain its ticket even if a later post-call checkpoint refuses.
                control.charge(WorkCategory::ProviderCall, provider_calls as u64)?;
                self.children.push(None);
                allocation = Some(Bank::Children(position));
            }
            HtmlConstructionActionDescription::Preflight { usage } => {
                control.observe_tree_depth(usage.depth() as u64)?;
                control.check_acquire_many(requests)?;
            }
            HtmlConstructionActionDescription::Commit {
                depth_observation,
                record_visits,
                record_writes,
                copied_bytes,
                initialized_bytes,
                usage,
            } => {
                if let Some(depth) = depth_observation {
                    control.observe_tree_depth(depth as u64)?;
                }
                control.admit_atomic_layout(initialized_bytes)?;
                control.charge(WorkCategory::CopyByte, copied_bytes as u64)?;
                control.charge(WorkCategory::RecordVisit, record_visits as u64)?;
                control.charge(WorkCategory::RecordWrite, record_writes as u64)?;
                before_commit(control)?;
                control.commit_construction_logical(
                    &self.control,
                    requests,
                    before_edges.saturating_sub(usage.edges()) as u64,
                )?;
                final_action = true;
            }
            HtmlConstructionActionDescription::InitializeIdentity { .. } => {
                // Identity belongs to the separate initialization owner.
                return Err(latch_registry_row(control));
            }
        }
        let performed = match reserved_capacity {
            Some(capacity) => action.perform_with_capacity(capacity),
            None => action.perform(),
        };
        match performed {
            Ok(observation) => {
                if let Some(bank) = allocation {
                    let Some(snapshot) = observation.allocation() else {
                        return Err(latch_registry_row(control));
                    };
                    let slot = self.allocation_slot(control, bank)?;
                    control.observe_construction_allocation(snapshot, slot, expected)?;
                }
                observation.accept();
                Ok(final_action)
            }
            Err(failure) => {
                if let Some(bank) = allocation
                    && let Some(snapshot) = failure.allocation()
                {
                    let slot = self.allocation_slot(control, bank)?;
                    control.record_construction_allocation(snapshot, slot, expected)?;
                }
                let error = failure.into_parts().0;
                Err(if error == HtmlConstructionError::AllocationFailed {
                    SessionError::Stop(control.construction_allocation_failed(expected))
                } else {
                    construction_error(control, error)
                })
            }
        }
    }
    pub(super) fn abort_pending(
        &mut self,
        control: &mut ParseControl,
        mut pending: PendingConstruction,
    ) -> Result<(), SessionError> {
        // An operation that is gone took its storage with it; there is none
        // to put back, and the invariant is all that can be reported.
        let Some(operation) = pending.operation.take() else {
            return Err(latch_registry_row(control));
        };
        self.restore_aborted(control, operation.abort())
    }
    fn restore_aborted(
        &mut self,
        control: &mut ParseControl,
        aborted: HtmlConstructionAbort,
    ) -> Result<(), SessionError> {
        let (storage, _, retired) = aborted.into_parts();
        let mut impossible = false;
        while self.children.len() > storage.len() {
            match self.children.pop() {
                Some(Some(ticket)) if ticket.snapshot().payload_bytes() != 0 => {
                    // Uncommitted child banks cannot have reserved; the ticket
                    // stays registered rather than being dropped uncredited.
                    self.children.push(Some(ticket));
                    impossible = true;
                    break;
                }
                Some(_) => {}
                None => {
                    impossible = true;
                    break;
                }
            }
        }
        self.owner = Some(storage);
        if let Some(receipt) = retired {
            match self.wrapper.take() {
                Some(ticket) => control.retire_construction_allocation(ticket, receipt)?,
                None => impossible = true,
            }
        }
        if impossible {
            return Err(latch_registry_row(control));
        }
        Ok(())
    }
    pub(super) fn run(
        &mut self,
        control: &mut ParseControl,
        command: HtmlConstructionCommand,
        requests: &[(HtmlParseResource, u64)],
    ) -> Result<HtmlConstructionCommit, SessionError> {
        let pending = self.start(control, command)?;
        self.continue_pending(control, pending, requests)
    }
    pub(super) fn finish_pending(
        &mut self,
        control: &mut ParseControl,
        mut pending: PendingConstruction,
    ) -> Result<Option<HtmlBuilderNode>, SessionError> {
        let ready = self.ready_pending(control, &mut pending)?;
        self.adopt_mutation(control, ready.commit())
    }
    /// The ready operation once every observation was accepted. An operation
    /// that is not ready is aborted, which returns its storage.
    fn ready_pending(
        &mut self,
        control: &mut ParseControl,
        pending: &mut PendingConstruction,
    ) -> Result<HtmlConstructionReadyOperation, SessionError> {
        match HtmlConstructionOperation::try_ready(&mut pending.operation) {
            Ok(ready) => Ok(ready),
            Err(error) => {
                let error = construction_error(control, error);
                if let Some(operation) = pending.operation.take() {
                    self.restore_aborted(control, operation.abort())?;
                }
                Err(error)
            }
        }
    }
    pub(super) fn continue_pending(
        &mut self,
        control: &mut ParseControl,
        pending: PendingConstruction,
        requests: &[(HtmlParseResource, u64)],
    ) -> Result<HtmlConstructionCommit, SessionError> {
        Ok(self.prepare_pending(control, pending, requests)?.commit())
    }
    fn prepare_pending(
        &mut self,
        control: &mut ParseControl,
        mut pending: PendingConstruction,
        requests: &[(HtmlParseResource, u64)],
    ) -> Result<HtmlConstructionReadyOperation, SessionError> {
        let result = (|| {
            loop {
                let before_edges = pending.before_edges();
                let Some(action) = pending.action(control)? else {
                    break;
                };
                if self.perform_action(control, action, requests, before_edges)? {
                    break;
                }
            }
            Ok::<_, SessionError>(())
        })();
        if let Err(error) = result {
            self.abort_pending(control, pending)?;
            return Err(error);
        }
        self.ready_pending(control, &mut pending)
    }
}

impl ConstructionOwner {
    pub(super) fn publish(
        &mut self,
        control: &mut ParseControl,
        root: HtmlBuilderNode,
    ) -> Result<(HtmlConstructionResult, HtmlConstructionLease), SessionError> {
        match self.run(control, HtmlConstructionCommand::Publish { root }, &[])? {
            HtmlConstructionCommit::Published { result, lease } => Ok((result, lease)),
            HtmlConstructionCommit::Changed { storage, .. } => {
                // Publication never leaves the storage changed in place; what
                // came back is the owner's storage and goes back to it.
                self.owner = Some(storage);
                Err(latch_registry_row(control))
            }
        }
    }
    pub(super) fn require_recovery(
        &self,
        control: &mut ParseControl,
        lease: &HtmlConstructionLease,
    ) -> Result<(), SessionError> {
        let Some(wrapper) = self.wrapper.as_ref() else {
            return Err(latch_registry_row(control));
        };
        control.require_construction_retirement(wrapper, lease.allocation())?;
        Ok(())
    }
    pub(super) fn recovered(
        &mut self,
        control: &mut ParseControl,
        recovery: HtmlConstructionRecovery,
    ) -> Result<(), SessionError> {
        let (storage, receipt) = recovery.into_parts();
        self.owner = Some(storage);
        // The storage is back before anything can refuse, so the owner can
        // still be retired whatever this reports.
        let Some(ticket) = self.wrapper.take() else {
            return Err(latch_registry_row(control));
        };
        control.retire_construction_allocation(ticket, receipt)?;
        #[cfg(test)]
        if control.take_construction_recovery_fault() {
            return Err(latch_registry_row(control));
        }
        Ok(())
    }
    /// Retires every bank this owner holds. A refusal is remembered and the
    /// walk continues, so each remaining bank is still credited and its ticket
    /// released; the first refusal is returned at the end.
    pub(super) fn retire(
        mut self,
        control: &mut ParseControl,
    ) -> Result<HtmlConstructionUsage, SessionError> {
        let mut refused = None;
        keep_first(&mut refused, self.abort_attributes(control));
        let Some(storage) = self.owner.take() else {
            // The storage is lent to something that was not ended, so no bank
            // can be retired here; the tickets below still go back to the
            // controller with the children registry.
            keep_first(&mut refused, Err(latch_registry_row(control)));
            keep_first(
                &mut refused,
                control
                    .release_vector(self.children, self.children_allocation)
                    .map_err(|error| SessionError::Stop(error.into_parts().0)),
            );
            return Err(refused.unwrap_or_else(|| latch_registry_row(control)));
        };
        let usage = storage.usage();
        let mut retirement = storage.retire();
        loop {
            let action = match retirement.next_action() {
                Ok(Some(action)) => action,
                Ok(None) => break,
                Err(error) => {
                    keep_first(&mut refused, Err(construction_error(control, error)));
                    break;
                }
            };
            let credit = match action.description() {
                HtmlConstructionRetirementDescription::Payload { .. } => Credit::None,
                HtmlConstructionRetirementDescription::Allocation(snapshot) => {
                    let bank = if let Some(position) = snapshot.child_position() {
                        Some(Bank::Children(position))
                    } else {
                        match snapshot.role() {
                            HtmlConstructionAllocationRole::Records => Some(Bank::Records),
                            HtmlConstructionAllocationRole::Identity => Some(Bank::Identity),
                            // A recovered owner holds only record, children
                            // and identity banks.
                            _ => None,
                        }
                    };
                    match bank {
                        None => {
                            keep_first(&mut refused, Err(latch_registry_row(control)));
                            Credit::Refused
                        }
                        Some(bank) => match self.allocation_slot(control, bank) {
                            Err(error) => {
                                keep_first(&mut refused, Err(error));
                                Credit::Refused
                            }
                            Ok(slot) => match slot.as_ref() {
                                Some(ticket) => {
                                    match control.require_construction_retirement(ticket, snapshot)
                                    {
                                        Ok(()) => Credit::Bank(bank),
                                        Err(stop) => {
                                            keep_first(&mut refused, Err(stop));
                                            Credit::Refused
                                        }
                                    }
                                }
                                // An unallocated record bank holds nothing.
                                None if snapshot.payload_bytes() == 0 => Credit::Empty,
                                None => {
                                    keep_first(&mut refused, Err(latch_registry_row(control)));
                                    Credit::Refused
                                }
                            },
                        },
                    }
                }
            };
            // The walk advances only by performing its actions.
            let Some(receipt) = action.perform() else {
                continue;
            };
            match credit {
                Credit::Bank(bank) => match self.allocation_slot(control, bank) {
                    Ok(slot) => match slot.take() {
                        Some(ticket) => keep_first(
                            &mut refused,
                            control.retire_construction_allocation(ticket, receipt),
                        ),
                        None => keep_first(&mut refused, Err(latch_registry_row(control))),
                    },
                    Err(error) => keep_first(&mut refused, Err(error)),
                },
                // An empty record bank has no physical refund.
                Credit::Empty if receipt.allocation().payload_bytes() != 0 => {
                    keep_first(&mut refused, Err(latch_registry_row(control)));
                }
                Credit::Empty | Credit::None | Credit::Refused => {}
            }
        }
        keep_first(
            &mut refused,
            control
                .release_vector(self.children, self.children_allocation)
                .map_err(|error| SessionError::Stop(error.into_parts().0)),
        );
        #[cfg(test)]
        if control.take_construction_retire_fault() {
            keep_first(&mut refused, Err(latch_registry_row(control)));
        }
        refused.map_or(Ok(usage), Err)
    }
}

/// What a retired action's receipt is credited against.
enum Credit {
    /// A payload action: nothing to credit.
    None,
    /// An unallocated bank: its receipt must hold nothing.
    Empty,
    /// The ticket of this bank.
    Bank(Bank),
    /// The action was refused and already recorded; it is only performed.
    Refused,
}

/// Borrows the same storage before or after publication; it owns no records or identity.
#[derive(Clone, Copy)]
pub(super) enum ConstructionView<'a> {
    Active(&'a HtmlConstructionStorage),
    Frozen(&'a HtmlConstructionResult),
}
impl<'a> ConstructionView<'a> {
    pub(super) fn node_at(self, position: usize) -> Option<HtmlConstructionNode<'a>> {
        match self {
            Self::Active(value) => value.node_at(position),
            Self::Frozen(value) => value.node_at(position),
        }
    }
    pub(super) fn len(self) -> usize {
        match self {
            Self::Active(value) => value.len(),
            Self::Frozen(value) => value.len(),
        }
    }
}

#[cfg(test)]
impl ConstructionOwner {
    /// The storage at rest, for tests that hold the owner in a known state.
    pub(super) fn test_storage(&self) -> &HtmlConstructionStorage {
        self.resident().expect("storage at rest")
    }
    /// Makes the next attribute edit begin with its custody slot emptied; the
    /// hidden edit is returned by [`Self::restore_hidden_attribute_edit`].
    pub(super) fn hide_next_attribute_edit(&mut self) {
        self.hide_next_attribute_edit = true;
    }
    pub(super) fn restore_hidden_attribute_edit(&mut self) {
        self.attribute_edit = self.hidden_attribute_edit.take();
    }
}
