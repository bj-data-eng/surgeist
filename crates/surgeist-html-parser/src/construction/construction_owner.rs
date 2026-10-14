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
    pending_orders: [Option<ConstructionAllocation>; 2],
}

#[derive(Clone, Copy)]
enum Bank {
    Identity,
    Records,
    Children(usize),
    PendingOrder(usize),
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
        HtmlConstructionError::RepresentationExhausted => SessionError::Stop(
            control
                .representation::<()>(HtmlParseResource::ManagedCapacityBytes)
                .unwrap_err(),
        ),
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
        self.operation
            .as_mut()
            .expect("pending construction operation")
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
        let action = initialization
            .as_mut()
            .expect("initialization owner")
            .next_action()
            .map_err(|error| construction_error(control, error))?
            .expect("real identity action");
        let HtmlConstructionActionDescription::InitializeIdentity {
            payload_bytes,
            initialized_bytes,
            provider_calls,
        } = action.description()
        else {
            unreachable!("initialization descriptor")
        };
        control.prepare_construction_share(payload_bytes)?;
        control.charge(WorkCategory::ProviderCall, (provider_calls - 1) as u64)?;
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
            if let Some(receipt) = initialization
                .take()
                .expect("refused initialization owner")
                .abort()
            {
                control.retire_construction_allocation(
                    identity.take().expect("actual identity ticket"),
                    receipt,
                )?;
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
                while let Some(action) = retirement
                    .next_action()
                    .map_err(|error| construction_error(control, error))?
                {
                    let is_identity = matches!(action.description(),HtmlConstructionRetirementDescription::Allocation(value) if matches!(value.role(),HtmlConstructionAllocationRole::Identity));
                    if let Some(receipt) = action.perform() {
                        if is_identity {
                            control.retire_construction_allocation(
                                identity.take().expect("actual identity ticket"),
                                receipt,
                            )?;
                        } else {
                            assert_eq!(
                                receipt.allocation().payload_bytes(),
                                0,
                                "empty initialization record bank"
                            );
                        }
                    }
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
            pending_orders: [None, None],
        })
    }
    pub(super) fn len(&self) -> usize {
        self.storage().len()
    }
    pub(super) fn storage(&self) -> &HtmlConstructionStorage {
        self.owner
            .as_ref()
            .or_else(|| {
                self.attribute_edit
                    .as_ref()
                    .map(HtmlConstructionAttributeEdit::storage_view)
            })
            .expect("active shared construction storage")
    }
    pub(super) fn reserve_child_tickets(
        &mut self,
        control: &mut ParseControl,
        additional: usize,
    ) -> Result<(), SessionError> {
        control.reserve_vector(
            &mut self.children,
            &mut self.children_allocation,
            additional,
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
    pub(super) fn usage(&self) -> HtmlConstructionUsage {
        self.storage().usage()
    }
    pub(super) fn node_at(&self, position: usize) -> Option<HtmlConstructionNode<'_>> {
        self.storage().node_at(position)
    }
    fn allocation_slot(&mut self, bank: Bank) -> &mut Option<ConstructionAllocation> {
        match bank {
            Bank::Identity => &mut self.identity,
            Bank::Records => &mut self.records,
            Bank::Children(position) => &mut self.children[position],
            Bank::Wrapper => &mut self.wrapper,
            Bank::PendingOrder(slot) => &mut self.pending_orders[slot],
        }
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
            HtmlConstructionCommit::Changed {
                storage,
                created,
                transfers,
                mut retired_orders,
            } => {
                self.owner = Some(storage);
                let mut old: [Option<(usize, ConstructionAllocation)>; 2] = [None, None];
                for (slot, transfer) in transfers.into_iter().enumerate() {
                    if let Some(transfer) = transfer {
                        let position = transfer.position();
                        let pending = self.pending_orders[slot]
                            .take()
                            .expect("real pending order ticket");
                        let ticket = control.adopt_construction_order(pending, transfer);
                        old[slot] = Some((
                            position,
                            self.children[position]
                                .replace(ticket)
                                .expect("original child bank ticket"),
                        ));
                    }
                }
                while let Some(action) = retired_orders.next_action() {
                    let position = action
                        .allocation()
                        .child_position()
                        .expect("actual retired child bank");
                    let slot = old
                        .iter()
                        .position(|value| {
                            value
                                .as_ref()
                                .is_some_and(|(parent, _)| *parent == position)
                        })
                        .expect("same real replaced bank");
                    control.require_order_retirement(
                        &old[slot].as_ref().expect("retained old bank ticket").1,
                        action.allocation(),
                    )?;
                    control.retire_order_allocation(
                        old[slot].take().expect("retained old bank ticket").1,
                        action.perform(),
                    )?;
                }
                Ok(created)
            }
            HtmlConstructionCommit::Published { .. } => {
                unreachable!("mutation command cannot publish")
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
        let storage = self.owner.take().expect("retained construction owner");
        let before_edges = storage.usage().edges();
        Ok(PendingConstruction {
            operation: Some(
                storage
                    .prepare(command)
                    .unwrap_or_else(|_| unreachable!("pending operation retains all operands")),
            ),
            before_edges,
        })
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
                minimum_capacity,
                ..
            } => {
                control.charge(WorkCategory::RecordVisit, 2)?;
                let bank = if let Some(slot) = snapshot.pending_child_order_slot() {
                    Bank::PendingOrder(slot)
                } else {
                    snapshot
                        .child_position()
                        .map_or(Bank::Records, Bank::Children)
                };
                expected = minimum_capacity
                    .checked_mul(snapshot.element_size())
                    .ok_or_else(|| {
                        control
                            .representation::<()>(HtmlParseResource::ManagedCapacityBytes)
                            .unwrap_err()
                    })?;
                control.prepare_construction_reserve(
                    snapshot,
                    minimum_capacity,
                    self.allocation_slot(bank).as_ref(),
                )?;
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
                control.reserve_vector(&mut self.children, &mut self.children_allocation, 1)?;
                control.charge(WorkCategory::CopyByte, copied_bytes as u64)?;
                control.charge(WorkCategory::RecordWrite, record_writes as u64)?;
                control.charge(
                    WorkCategory::CopyByte,
                    (mem::size_of::<Vec<usize>>()
                        + mem::size_of::<Option<ConstructionAllocation>>())
                        as u64,
                )?;
                control.charge(WorkCategory::RecordWrite, 2)?;
                ticket_work(control)?;
                assert_eq!(
                    position,
                    self.children.len(),
                    "real child-bank registry append"
                );
                // The zero-capacity bank physically exists in the pending owner;
                // retain its ticket even if a later post-call checkpoint refuses.
                control.charge(WorkCategory::ProviderCall, provider_calls as u64)?;
                self.children.push(None);
                allocation = Some(Bank::Children(position));
            }
            HtmlConstructionActionDescription::CopyChildOrderSource {
                record_visits,
                copied_bytes,
            } => {
                control.charge(WorkCategory::RecordVisit, record_visits as u64)?;
                control.charge(WorkCategory::CopyByte, copied_bytes as u64)?;
            }
            HtmlConstructionActionDescription::InitializeChildOrder {
                provider_calls,
                slot,
                initialized_bytes,
                ..
            } => {
                control.charge(WorkCategory::CopyByte, initialized_bytes as u64)?;
                control.charge(WorkCategory::RecordWrite, 2)?;
                ticket_work(control)?;
                control.charge(WorkCategory::ProviderCall, provider_calls as u64)?;
                allocation = Some(Bank::PendingOrder(slot));
            }
            HtmlConstructionActionDescription::PushChildOrder {
                copied_bytes,
                record_writes,
                auxiliary_entries,
                ..
            } => {
                control.charge(WorkCategory::CopyByte, copied_bytes as u64)?;
                control.charge(WorkCategory::RecordWrite, record_writes as u64)?;
                control.acquire(
                    HtmlParseResource::AuxiliaryEntries,
                    auxiliary_entries as u64,
                )?;
            }
            HtmlConstructionActionDescription::PrepareChildOrderTransfers {
                record_writes,
                provider_calls,
                copied_bytes,
                ..
            } => {
                control.charge(WorkCategory::RecordWrite, record_writes as u64)?;
                control.charge(WorkCategory::ProviderCall, provider_calls as u64)?;
                control.charge(WorkCategory::CopyByte, copied_bytes as u64)?;
            }
            HtmlConstructionActionDescription::Preflight { usage } => {
                control.observe_tree_depth(usage.depth() as u64)?;
                control.check_acquire_many(requests)?;
            }
            HtmlConstructionActionDescription::Commit {
                depth_observation,
                record_writes,
                copied_bytes,
                initialized_bytes,
                usage,
                auxiliary_entries_released,
            } => {
                if let Some(depth) = depth_observation {
                    control.observe_tree_depth(depth as u64)?;
                }
                control.admit_atomic_layout(initialized_bytes)?;
                control.charge(WorkCategory::CopyByte, copied_bytes as u64)?;
                control.charge(WorkCategory::RecordWrite, record_writes as u64)?;
                before_commit(control)?;
                control.commit_construction_logical(
                    &self.control,
                    requests,
                    before_edges.saturating_sub(usage.edges()) as u64,
                    auxiliary_entries_released as u64,
                )?;
                final_action = true;
            }
            HtmlConstructionActionDescription::InitializeIdentity { .. } => {
                unreachable!("separate initialization owner")
            }
        }
        match action.perform() {
            Ok(observation) => {
                if let Some(bank) = allocation {
                    let snapshot = observation
                        .allocation()
                        .expect("described actual provider result");
                    control.observe_construction_allocation(
                        snapshot,
                        self.allocation_slot(bank),
                        expected,
                    )?;
                }
                observation.accept();
                Ok(final_action)
            }
            Err(failure) => {
                if let Some(bank) = allocation
                    && let Some(snapshot) = failure.allocation()
                {
                    control.record_construction_allocation(
                        snapshot,
                        self.allocation_slot(bank),
                        expected,
                    )?;
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
        self.restore_aborted(
            control,
            pending
                .operation
                .take()
                .expect("refused construction owner")
                .abort(),
        )
    }
    fn restore_aborted(
        &mut self,
        control: &mut ParseControl,
        aborted: HtmlConstructionAbort,
    ) -> Result<(), SessionError> {
        let (storage, _, retired, mut orders) = aborted.into_parts();
        while self.children.len() > storage.len() {
            if let Some(ticket) = self.children.pop().expect("uncommitted child bank row") {
                assert_eq!(
                    ticket.snapshot().payload_bytes(),
                    0,
                    "uncommitted child banks cannot reserve"
                );
            }
        }
        self.owner = Some(storage);
        while let Some(action) = orders.next_action() {
            let slot = action
                .allocation()
                .pending_child_order_slot()
                .expect("aborted actual pending order");
            control.require_order_retirement(
                self.pending_orders[slot]
                    .as_ref()
                    .expect("actual pending bank ticket"),
                action.allocation(),
            )?;
            control.retire_order_allocation(
                self.pending_orders[slot]
                    .take()
                    .expect("actual pending bank ticket"),
                action.perform(),
            )?;
        }
        if let Some(receipt) = retired {
            control.retire_construction_allocation(
                self.wrapper.take().expect("actual wrapper ticket"),
                receipt,
            )?;
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
        let ready = HtmlConstructionOperation::try_ready(&mut pending.operation)
            .unwrap_or_else(|_| unreachable!("all actual observations accepted"));
        self.adopt_mutation(control, ready.commit())
    }
    pub(super) fn continue_pending(
        &mut self,
        control: &mut ParseControl,
        pending: PendingConstruction,
        requests: &[(HtmlParseResource, u64)],
    ) -> Result<HtmlConstructionCommit, SessionError> {
        Ok(self.prepare_pending(control, pending, requests)?.commit())
    }
    pub(super) fn prepare_mutation(
        &mut self,
        control: &mut ParseControl,
        command: HtmlConstructionCommand,
    ) -> Result<HtmlConstructionReadyOperation, SessionError> {
        let pending = self.start(control, command)?;
        self.prepare_pending(control, pending, &[])
    }
    pub(super) fn abort_prepared(
        &mut self,
        control: &mut ParseControl,
        ready: HtmlConstructionReadyOperation,
    ) -> Result<(), SessionError> {
        self.restore_aborted(control, ready.abort())
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
        let ready = HtmlConstructionOperation::try_ready(&mut pending.operation)
            .unwrap_or_else(|_| unreachable!("all actual observations accepted"));
        Ok(ready)
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
            _ => unreachable!("publication command"),
        }
    }
    pub(super) fn require_recovery(
        &self,
        control: &mut ParseControl,
        lease: &HtmlConstructionLease,
    ) -> Result<(), SessionError> {
        control.require_construction_retirement(
            self.wrapper.as_ref().expect("actual frozen wrapper ticket"),
            lease.allocation(),
        )?;
        Ok(())
    }
    pub(super) fn recovered(
        &mut self,
        control: &mut ParseControl,
        recovery: HtmlConstructionRecovery,
    ) -> Result<(), SessionError> {
        let (storage, receipt) = recovery.into_parts();
        self.owner = Some(storage);
        control.retire_construction_allocation(
            self.wrapper.take().expect("actual frozen wrapper ticket"),
            receipt,
        )?;
        Ok(())
    }
    pub(super) fn retire(
        mut self,
        control: &mut ParseControl,
    ) -> Result<HtmlConstructionUsage, SessionError> {
        self.abort_attributes(control)?;
        let usage = self.usage();
        let storage = self
            .owner
            .take()
            .expect("unique construction owner before retirement");
        let mut retirement = storage.retire();
        while let Some(action) = retirement
            .next_action()
            .map_err(|error| construction_error(control, error))?
        {
            let bank = match action.description() {
                HtmlConstructionRetirementDescription::Payload { .. } => None,
                HtmlConstructionRetirementDescription::Allocation(snapshot) => {
                    let bank = if let Some(position) = snapshot.child_position() {
                        Bank::Children(position)
                    } else {
                        match snapshot.role() {
                            HtmlConstructionAllocationRole::Records => Bank::Records,
                            HtmlConstructionAllocationRole::Identity => Bank::Identity,
                            _ => unreachable!(
                                "recovered construction owns only record/children/identity banks"
                            ),
                        }
                    };
                    if let Some(ticket) = self.allocation_slot(bank).as_ref() {
                        control.require_construction_retirement(ticket, snapshot)?;
                    } else {
                        assert_eq!(snapshot.payload_bytes(), 0, "unallocated empty record bank");
                    }
                    Some(bank)
                }
            };
            if let Some(receipt) = action.perform() {
                if let Some(ticket) = self
                    .allocation_slot(bank.expect("actual allocation retirement"))
                    .take()
                {
                    control.retire_construction_allocation(ticket, receipt)?;
                } else {
                    assert_eq!(
                        receipt.allocation().payload_bytes(),
                        0,
                        "empty record bank has no physical refund"
                    );
                }
            }
        }
        control
            .release_vector(self.children, self.children_allocation)
            .map_err(|error| SessionError::Stop(error.into_parts().0))?;
        Ok(usage)
    }
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
