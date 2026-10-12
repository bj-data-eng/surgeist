//! Private accounting owner for parsing. Counts describe admitted physical work
//! and owned storage, not CPU instructions, elapsed time, allocator metadata or RSS.

use crate::settings::{RESOURCE_COUNT, RESOURCES};
use crate::{
    HtmlParseCancellation, HtmlParseLimits, HtmlParseResource, HtmlSource, HtmlSourceCoordinates,
    HtmlSourceId,
};
use std::{
    marker::PhantomData,
    mem::size_of,
    sync::{Arc, Weak},
};

const WORK_COUNT: usize = 8;
pub(crate) const WORK_QUANTUM: usize = 256;

/// Each category has weight one; units are physical bytes or individual events.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(usize)]
pub(crate) enum WorkCategory {
    InputUnitRead,
    InspectByte,
    CopyByte,
    StateStep,
    RecordVisit,
    RecordWrite,
    ReserveByte,
    ProviderCall,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Invariant {
    ForeignSource,
    NonStorageResource,
    ReleaseUnderflow,
    NonReclaimableNodes,
    CapacityMismatch,
    ZeroSizedStorage,
    MissingSource,
    InputCursorState,
    ForeignControl,
    ForeignEvidence,
    CleanupWhileRunning,
    SharedAllocationAliased,
    PendingFormEffects,
    IncompleteFormAssociationCreation,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ParseStop {
    Cancelled,
    LimitExceeded {
        resource: HtmlParseResource,
        limit: u64,
        current: u64,
        requested_delta: u64,
    },
    RepresentationExhausted {
        resource: HtmlParseResource,
    },
    AllocationFailed {
        resource: HtmlParseResource,
        layout_bytes: u64,
    },
    InternalInvariant(Invariant),
    UnsupportedStandaloneState(StandaloneUnsupported),
}
/// Actual selected closed-environment execution boundaries, not limit errors.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum StandaloneUnsupported {
    Provenance,
    ScriptDocument,
    ScriptPause,
    PendingScript,
    InvalidNesting,
    Completion,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct ResourceUsage {
    pub(crate) current: u64,
    pub(crate) high_water: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ParseUsage {
    resources: [ResourceUsage; RESOURCE_COUNT],
    work: [u64; WORK_COUNT],
    /// Actual capacities can exceed requested layouts. This is observed after
    /// allocation; no promise is made about preventing allocator overshoot.
    pub(crate) actual_capacity_bytes: u64,
    pub(crate) actual_capacity_high_water: u64,
    /// Largest indivisible reserve/copy submission (not a time bound).
    pub(crate) largest_atomic_layout: u64,
}
impl ParseUsage {
    pub(crate) fn resource(&self, resource: HtmlParseResource) -> ResourceUsage {
        self.resources[resource as usize]
    }
    pub(crate) fn work(&self, category: WorkCategory) -> u64 {
        self.work[category as usize]
    }
}

/// Exclusively mutable owner. Only the cancellation handle can be cloned.
pub(crate) struct ParseControl {
    limits: [u64; RESOURCE_COUNT],
    usage: ParseUsage,
    cancellation: HtmlParseCancellation,
    terminal: Option<ParseStop>,
    // Fixed first-stop observation, captured before any producer cleanup.
    first_stop_usage: Option<ParseUsage>,
    source: Option<(HtmlSourceId, HtmlSourceCoordinates)>,
    identity: Arc<()>,
    #[cfg(test)]
    owned_spans: OwnedSpanObservation,
}

/// Fixed-size diagnostic metadata for actual execution between real polls.
/// It neither admits work/storage nor participates in parser decisions. Only
/// the observed append and registry operations are instrumented here; this is
/// not an observation of every parser instruction or provider implementation.
#[cfg(test)]
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct OwnedSpanObservation {
    pub(crate) polls: u64,
    pub(crate) append_calls: u64,
    pub(crate) append_slots: usize,
    pub(crate) largest_append_copy_bytes: usize,
    pub(crate) registry_iterations: usize,
    pub(crate) largest_registry_iterations_between_polls: usize,
    pub(crate) largest_registry_credits_between_polls: usize,
    registry_iterations_since_poll: usize,
    registry_credits_since_poll: usize,
}

#[derive(Clone)]
pub(crate) struct ControlIdentity(Arc<()>);

impl ParseControl {
    pub(crate) fn new(limits: HtmlParseLimits, cancellation: HtmlParseCancellation) -> Self {
        let limits = limits.values();
        Self {
            limits,
            usage: ParseUsage {
                resources: [ResourceUsage::default(); RESOURCE_COUNT],
                work: [0; WORK_COUNT],
                actual_capacity_bytes: 0,
                actual_capacity_high_water: 0,
                largest_atomic_layout: 0,
            },
            cancellation,
            terminal: None,
            first_stop_usage: None,
            source: None,
            identity: Arc::new(()),
            #[cfg(test)]
            owned_spans: OwnedSpanObservation::default(),
        }
    }
    pub(crate) fn usage(&self) -> &ParseUsage {
        &self.usage
    }
    pub(crate) fn identity(&self) -> ControlIdentity {
        ControlIdentity(self.identity.clone())
    }
    pub(crate) fn require_identity(&mut self, identity: &ControlIdentity) -> Result<(), ParseStop> {
        self.checkpoint()?;
        if Arc::ptr_eq(&identity.0, &self.identity) {
            Ok(())
        } else {
            self.fail(ParseStop::InternalInvariant(Invariant::ForeignControl))
        }
    }
    pub(crate) fn stop(&self) -> Option<ParseStop> {
        self.terminal
    }
    pub(crate) fn first_stop_usage(&self) -> Option<&ParseUsage> {
        self.first_stop_usage.as_ref()
    }
    /// Fixed identity readback, including after stop. The actual entry copies
    /// the fresh immutable request's capability into this sole controller.
    pub(crate) fn owns_request(&self, cancellation: &HtmlParseCancellation) -> bool {
        self.cancellation.same_handle(cancellation)
    }
    /// Fixed stopped-prefix observation. None means source admission never
    /// installed a binding; it is not an invented zero-length source.
    pub(crate) fn stopped_source_binding(
        &self,
    ) -> Result<Option<&(HtmlSourceId, HtmlSourceCoordinates)>, ParseStop> {
        if self.terminal.is_none() {
            return Err(ParseStop::InternalInvariant(Invariant::CleanupWhileRunning));
        }
        Ok(self.source.as_ref())
    }
    /// Cleanup-only move of the actual optional binding, including a prefix
    /// stopped before source admission. The caller preflights its owning slots.
    pub(crate) fn take_stopped_optional_source_binding(
        &mut self,
    ) -> Result<Option<(HtmlSourceId, HtmlSourceCoordinates)>, ParseStop> {
        self.stopped_source_binding()?;
        Ok(self.source.take())
    }
    /// A stopped, cleaned consumer moves the already-admitted source binding.
    /// Refusal leaves it intact; no polling, charging or handle clone follows
    /// stop. The caller retains this controller for diagnostic ticket disposal.
    pub(crate) fn take_stopped_source_binding(
        &mut self,
    ) -> Result<(HtmlSourceId, HtmlSourceCoordinates), ParseStop> {
        if self.terminal.is_none() {
            return Err(ParseStop::InternalInvariant(Invariant::CleanupWhileRunning));
        }
        self.source
            .take()
            .ok_or(ParseStop::InternalInvariant(Invariant::MissingSource))
    }
    fn latch_stop(&mut self, reason: ParseStop) -> ParseStop {
        if let Some(first) = self.terminal {
            return first;
        }
        // No allocation or useful progression: the fixed control layout owns
        // this observation. Denied requests remain in the stop, not in usage.
        self.first_stop_usage = Some(self.usage.clone());
        self.terminal = Some(reason);
        reason
    }
    pub(crate) fn fail_invariant(&mut self, invariant: Invariant) -> ParseStop {
        if let Err(reason) = self.checkpoint() {
            return reason;
        }
        self.latch_stop(ParseStop::InternalInvariant(invariant))
    }
    pub(crate) fn stop_unsupported_standalone(
        &mut self,
        reason: StandaloneUnsupported,
    ) -> ParseStop {
        if let Err(first) = self.checkpoint() {
            return first;
        }
        self.latch_stop(ParseStop::UnsupportedStandaloneState(reason))
    }
    fn fail<T>(&mut self, reason: ParseStop) -> Result<T, ParseStop> {
        let first = self.latch_stop(reason);
        Err(first)
    }
    pub(crate) fn checkpoint(&mut self) -> Result<(), ParseStop> {
        if let Some(reason) = self.terminal {
            return Err(reason);
        }
        #[cfg(test)]
        {
            self.owned_spans.polls = self.owned_spans.polls.saturating_add(1);
            self.owned_spans.registry_iterations_since_poll = 0;
            self.owned_spans.registry_credits_since_poll = 0;
        }
        if self.cancellation.is_cancelled() {
            return self.fail(ParseStop::Cancelled);
        }
        Ok(())
    }
    #[cfg(test)]
    pub(crate) fn clear_owned_span_observation(&mut self) {
        self.owned_spans = OwnedSpanObservation::default();
    }
    #[cfg(test)]
    pub(crate) fn owned_span_observation(&self) -> OwnedSpanObservation {
        self.owned_spans
    }
    #[cfg(test)]
    fn observe_attribute_append(&mut self, slots: usize) {
        // The actual fixed pop/push transfers each slot twice. Observe its
        // performed copy span, not an unrelated prepaid whole-vector total.
        let bytes = slots.saturating_mul(2 * size_of::<crate::Attribute>());
        self.owned_spans.append_calls = self.owned_spans.append_calls.saturating_add(1);
        self.owned_spans.append_slots = self.owned_spans.append_slots.saturating_add(slots);
        self.owned_spans.largest_append_copy_bytes =
            self.owned_spans.largest_append_copy_bytes.max(bytes);
    }
    #[cfg(test)]
    pub(crate) fn observe_registry_iteration(&mut self) {
        let observed = &mut self.owned_spans;
        observed.registry_iterations = observed.registry_iterations.saturating_add(1);
        observed.registry_iterations_since_poll =
            observed.registry_iterations_since_poll.saturating_add(1);
        observed.largest_registry_iterations_between_polls = observed
            .largest_registry_iterations_between_polls
            .max(observed.registry_iterations_since_poll);
        self.observe_registry_credits(1);
    }
    #[cfg(test)]
    pub(crate) fn observe_registry_location_write(&mut self, bytes: usize) {
        // Actual read and write of one location, plus its record-write event.
        self.observe_registry_credits(bytes.saturating_mul(2).saturating_add(1));
    }
    #[cfg(test)]
    fn observe_registry_credits(&mut self, credits: usize) {
        let observed = &mut self.owned_spans;
        observed.registry_credits_since_poll =
            observed.registry_credits_since_poll.saturating_add(credits);
        observed.largest_registry_credits_between_polls = observed
            .largest_registry_credits_between_polls
            .max(observed.registry_credits_since_poll);
    }
    /// Latches a checked-arithmetic failure observed after the caller's initial
    /// checkpoint. No extra poll can replace that already observed first stop.
    pub(crate) fn representation<T>(
        &mut self,
        resource: HtmlParseResource,
    ) -> Result<T, ParseStop> {
        self.fail(ParseStop::RepresentationExhausted { resource })
    }
    /// Admits one known indivisible owned-record layout before its move/copy.
    /// The caller separately precharges actual bytes and record writes. This
    /// observes layout only; it creates no allocation or cumulative work credit.
    pub(crate) fn admit_atomic_layout(&mut self, bytes: usize) -> Result<(), ParseStop> {
        self.checkpoint()?;
        if bytes > isize::MAX as usize {
            return self.representation(HtmlParseResource::AtomicLayoutBytes);
        }
        let Ok(bytes) = u64::try_from(bytes) else {
            return self.representation(HtmlParseResource::AtomicLayoutBytes);
        };
        self.check_value(HtmlParseResource::AtomicLayoutBytes, bytes, bytes)?;
        self.set(HtmlParseResource::AtomicLayoutBytes, bytes);
        self.usage.largest_atomic_layout = self.usage.largest_atomic_layout.max(bytes);
        Ok(())
    }
    fn check_value(
        &mut self,
        resource: HtmlParseResource,
        value: u64,
        delta: u64,
    ) -> Result<(), ParseStop> {
        let limit = self.limits[resource as usize];
        if value > limit {
            return self.fail(ParseStop::LimitExceeded {
                resource,
                limit,
                current: self.usage.resource(resource).current,
                requested_delta: delta,
            });
        }
        Ok(())
    }
    fn set(&mut self, resource: HtmlParseResource, value: u64) {
        let usage = &mut self.usage.resources[resource as usize];
        usage.current = value;
        usage.high_water = usage.high_water.max(value);
    }
    pub(crate) fn admit_source(&mut self, source: &HtmlSource) -> Result<(), ParseStop> {
        self.checkpoint()?;
        self.charge(WorkCategory::RecordVisit, 1)?;
        if let Some((identity, coordinates)) = &self.source {
            return if identity == source.identity() && *coordinates == source.coordinates() {
                Ok(())
            } else {
                self.fail(ParseStop::InternalInvariant(Invariant::ForeignSource))
            };
        }
        let Ok(units) = u64::try_from(source.len()) else {
            return self.representation(HtmlParseResource::SourceUnits);
        };
        self.check_value(HtmlParseResource::SourceUnits, units, units)?;
        self.set(HtmlParseResource::SourceUnits, units);
        self.source = Some((source.identity().clone(), source.coordinates()));
        Ok(())
    }
    pub(crate) fn require_source(&mut self, source: &HtmlSource) -> Result<(), ParseStop> {
        self.checkpoint()?;
        match &self.source {
            Some((identity, coordinates))
                if identity == source.identity() && *coordinates == source.coordinates() =>
            {
                Ok(())
            }
            Some(_) => self.fail(ParseStop::InternalInvariant(Invariant::ForeignSource)),
            None => self.fail(ParseStop::InternalInvariant(Invariant::MissingSource)),
        }
    }
    pub(crate) fn require_source_range(
        &mut self,
        range: &crate::HtmlSourceRange,
    ) -> Result<(), ParseStop> {
        self.charge_many(&[
            (WorkCategory::RecordVisit, 1),
            (WorkCategory::ProviderCall, 1),
        ])?;
        match &self.source {
            Some((identity, coordinates))
                if identity == range.source() && *coordinates == range.coordinates() =>
            {
                Ok(())
            }
            Some(_) => self.fail(ParseStop::InternalInvariant(Invariant::ForeignSource)),
            None => self.fail(ParseStop::InternalInvariant(Invariant::MissingSource)),
        }
    }
    pub(crate) fn charge(&mut self, category: WorkCategory, units: u64) -> Result<(), ParseStop> {
        self.charge_many(&[(category, units)])
    }
    fn charge_many(&mut self, charges: &[(WorkCategory, u64)]) -> Result<(), ParseStop> {
        self.checkpoint()?;
        let mut work = self.usage.work;
        let mut increment = 0_u64;
        for &(category, units) in charges {
            let (Some(next), Some(total)) = (
                work[category as usize].checked_add(units),
                increment.checked_add(units),
            ) else {
                return self.representation(HtmlParseResource::WorkCredits);
            };
            work[category as usize] = next;
            increment = total;
        }
        let Some(total) = self
            .usage
            .resource(HtmlParseResource::WorkCredits)
            .current
            .checked_add(increment)
        else {
            return self.representation(HtmlParseResource::WorkCredits);
        };
        self.check_value(HtmlParseResource::WorkCredits, total, increment)?;
        self.usage.work = work;
        self.set(HtmlParseResource::WorkCredits, total);
        Ok(())
    }
    /// Owned scans/loops must execute only the returned span before calling again.
    /// Allocations are separately precharged indivisible provider submissions.
    pub(crate) fn work_batch(
        &mut self,
        category: WorkCategory,
        remaining: usize,
    ) -> Result<usize, ParseStop> {
        let batch = remaining.min(WORK_QUANTUM);
        self.charge(category, batch as u64)?;
        Ok(batch)
    }
    fn plan_storage(
        &mut self,
        requests: &[(HtmlParseResource, u64)],
    ) -> Result<[u64; RESOURCE_COUNT], ParseStop> {
        self.checkpoint()?;
        let mut values = self.usage.resources.map(|usage| usage.current);
        let mut deltas = [0_u64; RESOURCE_COUNT];
        for &(resource, delta) in requests {
            if !resource.storage() {
                return self.fail(ParseStop::InternalInvariant(Invariant::NonStorageResource));
            }
            let (Some(value), Some(sum)) = (
                values[resource as usize].checked_add(delta),
                deltas[resource as usize].checked_add(delta),
            ) else {
                return self.representation(resource);
            };
            values[resource as usize] = value;
            deltas[resource as usize] = sum;
        }
        for resource in RESOURCES {
            if resource.storage() {
                self.check_value(
                    resource,
                    values[resource as usize],
                    deltas[resource as usize],
                )?;
            }
        }
        Ok(values)
    }
    pub(crate) fn acquire(
        &mut self,
        resource: HtmlParseResource,
        units: u64,
    ) -> Result<(), ParseStop> {
        self.acquire_many(&[(resource, units)])
    }
    /// Coupled logical allocations admit atomically in resource precedence order.
    pub(crate) fn acquire_many(
        &mut self,
        requests: &[(HtmlParseResource, u64)],
    ) -> Result<(), ParseStop> {
        let values = self.plan_storage(requests)?;
        for resource in RESOURCES {
            if resource.storage() {
                self.set(resource, values[resource as usize]);
            }
        }
        Ok(())
    }
    /// Nonmutating logical preflight before actual storage reservations. The
    /// session rechecks with `acquire_many` immediately before graph mutation;
    /// no logical acquisition may intervene between that commit and mutation.
    pub(crate) fn check_acquire_many(
        &mut self,
        requests: &[(HtmlParseResource, u64)],
    ) -> Result<(), ParseStop> {
        self.plan_storage(requests).map(|_| ())
    }
    /// Transfers admitted replacement child-order slots out of scratch storage
    /// and commits their actual ordinary-edge delta together. The construction
    /// owner precharges the bounded swaps/backlinks/copies before this final
    /// checkpoint, then changes topology immediately after success, without an
    /// intervening allocation, poll or fallible operation. Rejection retains all
    /// scratch and edge admission for terminal cleanup of the unchanged graph.
    pub(crate) fn commit_child_orders(
        &mut self,
        owner: &ControlIdentity,
        scratch_entries: u64,
        edge_add: u64,
        edge_remove: u64,
    ) -> Result<(), ParseStop> {
        self.require_identity(owner)?;
        let Some(auxiliary) = self
            .usage
            .resource(HtmlParseResource::AuxiliaryEntries)
            .current
            .checked_sub(scratch_entries)
        else {
            return self.fail(ParseStop::InternalInvariant(Invariant::ReleaseUnderflow));
        };
        let Some(retained_edges) = self
            .usage
            .resource(HtmlParseResource::ManagedEdges)
            .current
            .checked_sub(edge_remove)
        else {
            return self.fail(ParseStop::InternalInvariant(Invariant::ReleaseUnderflow));
        };
        let Some(edges) = retained_edges.checked_add(edge_add) else {
            return self.representation(HtmlParseResource::ManagedEdges);
        };
        self.check_value(HtmlParseResource::ManagedEdges, edges, edge_add)?;
        self.set(HtmlParseResource::AuxiliaryEntries, auxiliary);
        self.set(HtmlParseResource::ManagedEdges, edges);
        Ok(())
    }
    /// Produces the actual empty vector and its unique accounting ticket.
    /// This creates no allocation or logical node/edge/storage admission.
    pub(crate) fn new_vector<T>(&mut self) -> Result<(Vec<T>, VectorCapacity<T>), ParseStop> {
        self.checkpoint()?;
        if size_of::<T>() == 0 {
            return self.fail(ParseStop::InternalInvariant(Invariant::ZeroSizedStorage));
        }
        let Some(bytes) = size_of::<Vec<T>>()
            .checked_add(size_of::<VectorCapacity<T>>())
            .and_then(|bytes| u64::try_from(bytes).ok())
        else {
            return self.representation(HtmlParseResource::AtomicLayoutBytes);
        };
        self.charge_many(&[
            (WorkCategory::RecordWrite, 2),
            (WorkCategory::CopyByte, bytes),
        ])?;
        Ok((
            Vec::new(),
            VectorCapacity {
                owner: self.identity.clone(),
                capacity: 0,
                element: PhantomData,
            },
        ))
    }
    /// Reserves the consumer's actual vector. Only the construction owner may
    /// hold its mutable storage; engine operations receive no vector escape.
    /// Logical admission and actual record copies/writes remain caller duties.
    pub(crate) fn reserve_vector<T>(
        &mut self,
        vector: &mut Vec<T>,
        allocation: &mut VectorCapacity<T>,
        additional: usize,
    ) -> Result<(), ParseStop> {
        self.checkpoint()?;
        if !Arc::ptr_eq(&allocation.owner, &self.identity) {
            return self.fail(ParseStop::InternalInvariant(Invariant::ForeignControl));
        }
        self.charge(WorkCategory::RecordVisit, 2)?;
        if allocation.capacity != vector.capacity() {
            return self.fail(ParseStop::InternalInvariant(Invariant::CapacityMismatch));
        }
        let plan = match ReservationPlan::for_vec(vector, additional) {
            Ok(Some(plan)) => plan,
            Ok(None) => return Ok(()),
            Err(stop) => return self.fail(stop),
        };
        let Some(bytes) = plan.old_bytes.checked_add(plan.new_bytes) else {
            return self.representation(HtmlParseResource::ManagedCapacityBytes);
        };
        self.charge_many(&[
            (WorkCategory::ReserveByte, bytes),
            (WorkCategory::ProviderCall, 1),
            (WorkCategory::RecordWrite, 1),
            (WorkCategory::CopyByte, size_of::<usize>() as u64),
        ])?;
        let result = self.reserve(vector, plan);
        // Successful reserve can change capacity before a post-call stop. That
        // physical ownership must remain releasable even when logical work ends.
        Self::record_vector_capacity(vector, allocation, result)
    }
    fn record_vector_capacity<T>(
        vector: &Vec<T>,
        allocation: &mut VectorCapacity<T>,
        result: Result<(), ParseStop>,
    ) -> Result<(), ParseStop> {
        allocation.capacity = vector.capacity();
        result
    }
    /// The only Weak reference permitted here is this actual wrapper ticket.
    /// Its temporary removal and restoration are prepaid, even if reserve
    /// latches a terminal failure. Opaque Arc headers stay outside byte counts.
    fn prepare_attribute_edit(
        &mut self,
        values: &Arc<Vec<crate::Attribute>>,
        shared: &SharedAllocation<Vec<crate::Attribute>>,
        vector: &VectorCapacity<crate::Attribute>,
    ) -> Result<(), ParseStop> {
        self.charge_many(&[
            (WorkCategory::RecordVisit, 3),
            (WorkCategory::ProviderCall, 9),
            (
                WorkCategory::CopyByte,
                (2 * size_of::<Weak<Vec<crate::Attribute>>>()) as u64,
            ),
            (WorkCategory::RecordWrite, 2),
        ])?;
        if !Arc::ptr_eq(&shared.owner, &self.identity)
            || !Arc::ptr_eq(&vector.owner, &self.identity)
        {
            return self.fail(ParseStop::InternalInvariant(Invariant::ForeignControl));
        }
        if shared.shared.as_ptr() != Arc::as_ptr(values)
            || shared.bytes != size_of::<Vec<crate::Attribute>>() as u64
            || vector.capacity != values.capacity()
        {
            return self.fail(ParseStop::InternalInvariant(Invariant::CapacityMismatch));
        }
        if Arc::strong_count(values) != 1 || Arc::weak_count(values) != 1 {
            return self.fail(ParseStop::InternalInvariant(
                Invariant::SharedAllocationAliased,
            ));
        }
        Ok(())
    }
    pub(crate) fn reserve_attribute_storage(
        &mut self,
        values: &mut Arc<Vec<crate::Attribute>>,
        shared: &mut SharedAllocation<Vec<crate::Attribute>>,
        vector: &mut VectorCapacity<crate::Attribute>,
        additional: usize,
    ) -> Result<(), ParseStop> {
        self.prepare_attribute_edit(values, shared, vector)?;
        drop(std::mem::replace(&mut shared.shared, Weak::new()));
        let result = self.reserve_vector(
            Arc::get_mut(values).expect("only ticket Weak was removed"),
            vector,
            additional,
        );
        shared.shared = Arc::downgrade(values);
        result
    }
    /// Moves one actual last staged slot into already reserved storage. The
    /// caller prepays its fixed progress write and commits it immediately after
    /// success. Repeated slots must call this producer again, observing a poll.
    pub(crate) fn append_attribute_slot(
        &mut self,
        values: &mut Arc<Vec<crate::Attribute>>,
        shared: &mut SharedAllocation<Vec<crate::Attribute>>,
        vector: &VectorCapacity<crate::Attribute>,
        incoming: &mut Vec<crate::Attribute>,
        incoming_allocation: &VectorCapacity<crate::Attribute>,
    ) -> Result<(), ParseStop> {
        self.prepare_attribute_edit(values, shared, vector)?;
        self.charge_many(&[
            (WorkCategory::ProviderCall, 1),
            (WorkCategory::RecordVisit, 2),
        ])?;
        if !Arc::ptr_eq(&incoming_allocation.owner, &self.identity) {
            return self.fail(ParseStop::InternalInvariant(Invariant::ForeignControl));
        }
        if incoming.capacity() != incoming_allocation.capacity {
            return self.fail(ParseStop::InternalInvariant(Invariant::CapacityMismatch));
        }
        if incoming.is_empty() {
            return self.fail(ParseStop::InternalInvariant(Invariant::CapacityMismatch));
        }
        let Some(len) = values.len().checked_add(1) else {
            return self.representation(HtmlParseResource::ManagedAttributes);
        };
        if len > values.capacity() {
            return self.fail(ParseStop::InternalInvariant(Invariant::CapacityMismatch));
        }
        let bytes = 2 * size_of::<crate::Attribute>();
        self.admit_atomic_layout(bytes)?;
        self.charge_many(&[
            (WorkCategory::CopyByte, bytes as u64),
            (WorkCategory::RecordWrite, 3),
        ])?;
        drop(std::mem::replace(&mut shared.shared, Weak::new()));
        #[cfg(test)]
        self.observe_attribute_append(1);
        Arc::get_mut(values)
            .expect("only ticket Weak was removed")
            .push(incoming.pop().expect("checked real last slot"));
        shared.shared = Arc::downgrade(values);
        Ok(())
    }
    /// Replaces one actual initialized slot without growing or changing the
    /// logical attribute count. All rejection precedes ownership transfer.
    pub(crate) fn replace_attribute_slot(
        &mut self,
        values: &mut Arc<Vec<crate::Attribute>>,
        shared: &mut SharedAllocation<Vec<crate::Attribute>>,
        vector: &VectorCapacity<crate::Attribute>,
        index: usize,
        incoming: &mut Option<crate::Attribute>,
    ) -> Result<crate::Attribute, ParseStop> {
        self.prepare_attribute_edit(values, shared, vector)?;
        self.charge(WorkCategory::RecordVisit, 2)?;
        if index >= values.len() || incoming.is_none() {
            return self.fail(ParseStop::InternalInvariant(Invariant::CapacityMismatch));
        }
        let bytes = 3 * size_of::<crate::Attribute>();
        self.admit_atomic_layout(bytes)?;
        self.charge_many(&[
            (WorkCategory::CopyByte, bytes as u64),
            (WorkCategory::RecordWrite, 3),
        ])?;
        drop(std::mem::replace(&mut shared.shared, Weak::new()));
        let old = std::mem::replace(
            &mut Arc::get_mut(values).expect("only ticket Weak was removed")[index],
            incoming.take().expect("checked actual incoming slot"),
        );
        shared.shared = Arc::downgrade(values);
        Ok(old)
    }
    /// Consumes real storage plus its ticket, returning both on invalid owner
    /// or capacity. Cleanup needs no budget, and releases capacity only: managed
    /// nodes, logical edges/slots, cumulative work and high-water stay admitted.
    /// The session separately extracts/releases nested allocation tickets first.
    pub(crate) fn release_vector<T>(
        &mut self,
        vector: Vec<T>,
        allocation: VectorCapacity<T>,
    ) -> Result<(), VectorReleaseError<T>> {
        let release = (|| {
            if !Arc::ptr_eq(&allocation.owner, &self.identity) {
                return self.fail(ParseStop::InternalInvariant(Invariant::ForeignControl));
            }
            if allocation.capacity != vector.capacity() {
                return self.fail(ParseStop::InternalInvariant(Invariant::CapacityMismatch));
            }
            let Some(bytes) = allocation
                .capacity
                .checked_mul(size_of::<T>())
                .and_then(|bytes| u64::try_from(bytes).ok())
            else {
                return self.representation(HtmlParseResource::ManagedCapacityBytes);
            };
            self.cleanup_capacity(bytes)
        })();
        match release {
            Ok(()) => {
                drop(vector);
                Ok(())
            }
            Err(stop) => Err(VectorReleaseError {
                stop,
                vector,
                allocation,
            }),
        }
    }
    fn cleanup_capacity(&mut self, bytes: u64) -> Result<(), ParseStop> {
        let Some(capacity) = self
            .usage
            .resource(HtmlParseResource::ManagedCapacityBytes)
            .current
            .checked_sub(bytes)
        else {
            return self.fail(ParseStop::InternalInvariant(Invariant::ReleaseUnderflow));
        };
        self.set(HtmlParseResource::ManagedCapacityBytes, capacity);
        self.set(HtmlParseResource::PeakTransientBytes, capacity);
        self.usage.actual_capacity_bytes = capacity;
        Ok(())
    }
    /// Builds an actual fixed shared payload, then polls before returning it.
    /// Nested vectors keep their own tickets. Arc headers/allocator metadata and
    /// infallible Arc setup are opaque provider limits; zero-sized identity is
    /// valid and still charges the provider call. Failure returns owned payload.
    pub(crate) fn share_owned<T>(
        &mut self,
        payload: T,
    ) -> Result<(Arc<T>, SharedAllocation<T>), (ParseStop, T)> {
        let admission = (|| {
            self.checkpoint()?;
            let Ok(bytes) = u64::try_from(size_of::<T>()) else {
                return self.representation(HtmlParseResource::AtomicLayoutBytes);
            };
            self.charge_many(&[
                (WorkCategory::ReserveByte, bytes),
                (WorkCategory::ProviderCall, 1),
                (WorkCategory::CopyByte, bytes),
                (WorkCategory::RecordWrite, 1),
            ])?;
            let Some(total) = self
                .usage
                .resource(HtmlParseResource::ManagedCapacityBytes)
                .current
                .checked_add(bytes)
            else {
                return self.representation(HtmlParseResource::ManagedCapacityBytes);
            };
            self.check_value(HtmlParseResource::ManagedCapacityBytes, total, bytes)?;
            self.check_value(HtmlParseResource::PeakTransientBytes, total, bytes)?;
            self.check_value(HtmlParseResource::AtomicLayoutBytes, bytes, bytes)?;
            self.set(HtmlParseResource::ManagedCapacityBytes, total);
            self.set(HtmlParseResource::PeakTransientBytes, total);
            self.set(HtmlParseResource::AtomicLayoutBytes, bytes);
            self.usage.largest_atomic_layout = self.usage.largest_atomic_layout.max(bytes);
            self.usage.actual_capacity_bytes = total;
            self.usage.actual_capacity_high_water =
                self.usage.actual_capacity_high_water.max(total);
            Ok(bytes)
        })();
        let bytes = match admission {
            Ok(bytes) => bytes,
            Err(stop) => return Err((stop, payload)),
        };
        let shared = Arc::new(payload);
        let allocation = SharedAllocation {
            owner: self.identity.clone(),
            shared: Arc::downgrade(&shared),
            bytes,
            element: PhantomData,
        };
        self.finish_shared(shared, allocation)
    }
    fn finish_shared<T>(
        &mut self,
        shared: Arc<T>,
        allocation: SharedAllocation<T>,
    ) -> Result<(Arc<T>, SharedAllocation<T>), (ParseStop, T)> {
        match self.checkpoint() {
            Ok(()) => Ok((shared, allocation)),
            Err(stop) => {
                // This private transition receives the sole newly created Arc;
                // it has not escaped and no callback can have cloned it.
                let payload = Arc::into_inner(shared)
                    .expect("new shared payload has exactly one strong owner");
                // Admission above proved ownership and available capacity. No
                // other storage mutation occurs before this cleanup transition.
                self.cleanup_capacity(allocation.bytes)
                    .expect("new shared payload remains admitted");
                Err((stop, payload))
            }
        }
    }
    /// Recovers the unique payload and releases only its known shared layout.
    /// Aliased/foreign failures return the actual Arc and ticket unchanged, so
    /// the original owner may retry cleanup after outstanding handles are gone.
    pub(crate) fn release_shared<T>(
        &mut self,
        shared: Arc<T>,
        allocation: SharedAllocation<T>,
    ) -> Result<T, SharedReleaseError<T>> {
        if !Arc::ptr_eq(&allocation.owner, &self.identity) {
            let stop = self
                .fail::<()>(ParseStop::InternalInvariant(Invariant::ForeignControl))
                .unwrap_err();
            return Err(SharedReleaseError {
                stop,
                shared,
                allocation,
            });
        }
        if !std::ptr::eq(Arc::as_ptr(&shared), allocation.shared.as_ptr()) {
            let stop = self
                .fail::<()>(ParseStop::InternalInvariant(Invariant::CapacityMismatch))
                .unwrap_err();
            return Err(SharedReleaseError {
                stop,
                shared,
                allocation,
            });
        }
        if self
            .usage
            .resource(HtmlParseResource::ManagedCapacityBytes)
            .current
            < allocation.bytes
        {
            let stop = self
                .fail::<()>(ParseStop::InternalInvariant(Invariant::ReleaseUnderflow))
                .unwrap_err();
            return Err(SharedReleaseError {
                stop,
                shared,
                allocation,
            });
        }
        match Arc::try_unwrap(shared) {
            Ok(payload) => {
                self.cleanup_capacity(allocation.bytes)
                    .expect("shared payload capacity checked before recovery");
                Ok(payload)
            }
            Err(shared) => {
                let stop = self
                    .fail::<()>(ParseStop::InternalInvariant(
                        Invariant::SharedAllocationAliased,
                    ))
                    .unwrap_err();
                Err(SharedReleaseError {
                    stop,
                    shared,
                    allocation,
                })
            }
        }
    }
    /// Drops one actual zero-sized shared payload and its accounting ticket.
    /// Other strong owners may retain the opaque Arc header; this operation
    /// makes no deallocation claim and refunds no capacity. Invalid tickets
    /// return the actual Arc and ticket for cleanup by their original owner.
    pub(crate) fn release_zero_sized_shared<T>(
        &mut self,
        shared: Arc<T>,
        allocation: SharedAllocation<T>,
    ) -> Result<(), SharedReleaseError<T>> {
        let invariant = if !Arc::ptr_eq(&allocation.owner, &self.identity) {
            Some(Invariant::ForeignControl)
        } else if !std::ptr::eq(Arc::as_ptr(&shared), allocation.shared.as_ptr())
            || size_of::<T>() != 0
            || allocation.bytes != 0
        {
            Some(Invariant::CapacityMismatch)
        } else {
            None
        };
        if let Some(invariant) = invariant {
            let stop = self
                .fail::<()>(ParseStop::InternalInvariant(invariant))
                .unwrap_err();
            return Err(SharedReleaseError {
                stop,
                shared,
                allocation,
            });
        }
        drop(shared);
        drop(allocation);
        Ok(())
    }
    pub(crate) fn release(
        &mut self,
        resource: HtmlParseResource,
        units: u64,
    ) -> Result<(), ParseStop> {
        self.checkpoint()?;
        if !resource.storage() {
            return self.fail(ParseStop::InternalInvariant(Invariant::NonStorageResource));
        }
        if resource == HtmlParseResource::ManagedNodes {
            return self.fail(ParseStop::InternalInvariant(Invariant::NonReclaimableNodes));
        }
        let Some(value) = self.usage.resource(resource).current.checked_sub(units) else {
            return self.fail(ParseStop::InternalInvariant(Invariant::ReleaseUnderflow));
        };
        self.set(resource, value);
        Ok(())
    }
    /// Releases actually deallocated non-node logical slots after first-stop.
    /// No successful engine transition may use this cleanup-only path.
    pub(crate) fn cleanup_release(
        &mut self,
        identity: &ControlIdentity,
        resource: HtmlParseResource,
        units: u64,
    ) -> Result<(), ParseStop> {
        if self.stop().is_none() {
            return Err(self.fail_invariant(Invariant::CleanupWhileRunning));
        }
        if !Arc::ptr_eq(&identity.0, &self.identity) {
            return self.fail(ParseStop::InternalInvariant(Invariant::ForeignControl));
        }
        if !resource.storage() {
            return self.fail(ParseStop::InternalInvariant(Invariant::NonStorageResource));
        }
        if resource == HtmlParseResource::ManagedNodes {
            return self.fail(ParseStop::InternalInvariant(Invariant::NonReclaimableNodes));
        }
        let Some(value) = self.usage.resource(resource).current.checked_sub(units) else {
            return self.fail(ParseStop::InternalInvariant(Invariant::ReleaseUnderflow));
        };
        self.set(resource, value);
        Ok(())
    }
    pub(crate) fn observe_tree_depth(&mut self, depth: u64) -> Result<(), ParseStop> {
        self.checkpoint()?;
        self.check_value(
            HtmlParseResource::TreeDepth,
            depth,
            depth.saturating_sub(self.usage.resource(HtmlParseResource::TreeDepth).current),
        )?;
        self.set(HtmlParseResource::TreeDepth, depth);
        Ok(())
    }
    fn reserve<T>(&mut self, vector: &mut Vec<T>, plan: ReservationPlan) -> Result<(), ParseStop> {
        self.checkpoint()?;
        let current = self
            .usage
            .resource(HtmlParseResource::ManagedCapacityBytes)
            .current;
        let Some(without_old) = current.checked_sub(plan.old_bytes) else {
            return self.fail(ParseStop::InternalInvariant(Invariant::CapacityMismatch));
        };
        let (Some(new_total), Some(overlap)) = (
            without_old.checked_add(plan.new_bytes),
            current.checked_add(plan.new_bytes),
        ) else {
            return self.representation(HtmlParseResource::ManagedCapacityBytes);
        };
        self.check_value(
            HtmlParseResource::ManagedCapacityBytes,
            new_total,
            plan.new_bytes - plan.old_bytes,
        )?;
        self.check_value(
            HtmlParseResource::PeakTransientBytes,
            overlap,
            plan.new_bytes,
        )?;
        self.check_value(
            HtmlParseResource::AtomicLayoutBytes,
            plan.new_bytes,
            plan.new_bytes,
        )?;
        self.set(HtmlParseResource::PeakTransientBytes, overlap);
        self.set(HtmlParseResource::AtomicLayoutBytes, plan.new_bytes);
        self.usage.largest_atomic_layout = self.usage.largest_atomic_layout.max(plan.new_bytes);
        if vector.try_reserve_exact(plan.additional).is_err() {
            self.set(HtmlParseResource::PeakTransientBytes, current);
            return self.fail(ParseStop::AllocationFailed {
                resource: HtmlParseResource::ManagedCapacityBytes,
                layout_bytes: plan.new_bytes,
            });
        }
        let Some(actual) = vector
            .capacity()
            .checked_mul(size_of::<T>())
            .and_then(|bytes| u64::try_from(bytes).ok())
        else {
            return self.representation(HtmlParseResource::ManagedCapacityBytes);
        };
        let Some(actual_total) = without_old.checked_add(actual) else {
            return self.representation(HtmlParseResource::ManagedCapacityBytes);
        };
        self.usage.actual_capacity_bytes = actual_total;
        self.usage.actual_capacity_high_water =
            self.usage.actual_capacity_high_water.max(actual_total);
        self.set(HtmlParseResource::ManagedCapacityBytes, actual_total);
        let Some(actual_overlap) = current.checked_add(actual) else {
            return self.representation(HtmlParseResource::PeakTransientBytes);
        };
        self.set(HtmlParseResource::PeakTransientBytes, actual_overlap);
        self.set(HtmlParseResource::PeakTransientBytes, actual_total);
        if actual > plan.new_bytes {
            return self.fail(ParseStop::LimitExceeded {
                resource: HtmlParseResource::ManagedCapacityBytes,
                limit: plan.new_bytes,
                current: plan.old_bytes,
                requested_delta: actual - plan.old_bytes,
            });
        }
        self.checkpoint()
    }
    /// Cleanup is permitted after a terminal stop; it refunds no admitted work or
    /// high-water observation. The vector owner supplies its exact retained size.
    fn cleanup_storage(
        &mut self,
        resource: HtmlParseResource,
        units: u64,
        bytes: u64,
    ) -> Result<(), ParseStop> {
        let (Some(logical), Some(capacity)) = (
            self.usage.resource(resource).current.checked_sub(units),
            self.usage
                .resource(HtmlParseResource::ManagedCapacityBytes)
                .current
                .checked_sub(bytes),
        ) else {
            return self.fail(ParseStop::InternalInvariant(Invariant::ReleaseUnderflow));
        };
        self.set(resource, logical);
        self.set(HtmlParseResource::ManagedCapacityBytes, capacity);
        self.set(HtmlParseResource::PeakTransientBytes, capacity);
        self.usage.actual_capacity_bytes = capacity;
        Ok(())
    }
    /// Admits only the known fixed Arc payload. Arc headers, allocator metadata,
    /// and its infallible setup are provider limitations, not measured capacity.
    /// The consumer polls after the actual shared-wrapper construction.
    fn admit_document_string_wrapper(
        &mut self,
        mut buffer: OwnedVec<u16>,
    ) -> Result<(Vec<u16>, DocumentStringAllocation), ParseStop> {
        let admission = buffer
            .require_owner(self)
            .and_then(|()| self.admit_document_string_payload(buffer.resource));
        let bytes = match admission {
            Ok(bytes) => bytes,
            Err(reason) => {
                if buffer
                    .owner
                    .as_ref()
                    .is_none_or(|owner| Arc::ptr_eq(owner, &self.identity))
                {
                    buffer.release(self)?;
                }
                return Err(reason);
            }
        };
        let allocation = DocumentStringAllocation {
            owner: self.identity.clone(),
            units: buffer.len() as u64,
            vector_capacity: buffer.capacity(),
            wrapper_bytes: bytes,
            resource: HtmlParseResource::BufferedUtf16Units,
            data_pointer: buffer.as_slice().as_ptr() as usize,
            backing: None,
        };
        Ok((buffer.into_vec(), allocation))
    }
    fn admit_document_string_payload(
        &mut self,
        resource: HtmlParseResource,
    ) -> Result<u64, ParseStop> {
        self.checkpoint()?;
        if resource != HtmlParseResource::BufferedUtf16Units {
            return self.fail(ParseStop::InternalInvariant(Invariant::NonStorageResource));
        }
        let bytes = size_of::<Vec<u16>>() as u64;
        self.charge_many(&[
            (WorkCategory::ReserveByte, bytes),
            (WorkCategory::ProviderCall, 1),
        ])?;
        let current = self
            .usage
            .resource(HtmlParseResource::ManagedCapacityBytes)
            .current;
        let Some(total) = current.checked_add(bytes) else {
            return self.representation(HtmlParseResource::ManagedCapacityBytes);
        };
        self.check_value(HtmlParseResource::ManagedCapacityBytes, total, bytes)?;
        self.check_value(HtmlParseResource::PeakTransientBytes, total, bytes)?;
        self.check_value(HtmlParseResource::AtomicLayoutBytes, bytes, bytes)?;
        self.set(HtmlParseResource::ManagedCapacityBytes, total);
        self.set(HtmlParseResource::PeakTransientBytes, total);
        self.set(HtmlParseResource::AtomicLayoutBytes, bytes);
        self.usage.largest_atomic_layout = self.usage.largest_atomic_layout.max(bytes);
        // This total includes known fixed payloads, never opaque provider headers.
        self.usage.actual_capacity_bytes = total;
        self.usage.actual_capacity_high_water = self.usage.actual_capacity_high_water.max(total);
        Ok(bytes)
    }
    /// Owns actual wrapper construction, including the empty-vector case where
    /// a dangling data pointer cannot by itself prove backing provenance.
    /// Arc's fixed known Vec payload is admitted; opaque headers, allocator
    /// bookkeeping and infallible std Arc setup remain explicit limitations.
    pub(crate) fn freeze_document_string(
        &mut self,
        buffer: OwnedVec<u16>,
    ) -> Result<(crate::DocumentString, DocumentStringAllocation), ParseStop> {
        let (units, mut allocation) = self.admit_document_string_wrapper(buffer)?;
        let text = crate::DocumentString::from_utf16(units);
        if let Err(stop) = self.bind_document_string(&mut allocation, &text) {
            drop(text);
            self.release_document_string(allocation)?;
            return Err(stop);
        }
        Ok((text, allocation))
    }
    /// Binds the actual immutable wrapper to the vector admitted by the sole
    /// producer. Pointer equality observes identity only; no pointer is read.
    fn bind_document_string(
        &mut self,
        allocation: &mut DocumentStringAllocation,
        value: &crate::DocumentString,
    ) -> Result<(), ParseStop> {
        self.checkpoint()?;
        self.charge_many(&[
            (WorkCategory::RecordVisit, 1),
            (WorkCategory::ProviderCall, 1),
        ])?;
        if !Arc::ptr_eq(&allocation.owner, &self.identity) {
            return self.fail(ParseStop::InternalInvariant(Invariant::ForeignControl));
        }
        if allocation.backing.is_some()
            || value.len() as u64 != allocation.units
            || value.backing().capacity() != allocation.vector_capacity
            || value.code_units().as_ptr() as usize != allocation.data_pointer
        {
            return self.fail(ParseStop::InternalInvariant(Invariant::CapacityMismatch));
        }
        allocation.backing = Some(Arc::downgrade(value.backing()));
        Ok(())
    }
    pub(crate) fn require_document_string(
        &mut self,
        allocation: &DocumentStringAllocation,
        value: &crate::DocumentString,
    ) -> Result<(), ParseStop> {
        self.checkpoint()?;
        self.charge(WorkCategory::RecordVisit, 1)?;
        self.validate_document_string(allocation, value)
    }
    /// Disposal-only inspection of already-admitted backing. Terminal cleanup
    /// may validate ownership before taking a caller's retained payload; it
    /// cannot resume parsing, acquire storage, or refund execution observations.
    pub(crate) fn require_document_string_for_disposal(
        &mut self,
        allocation: &DocumentStringAllocation,
        value: &crate::DocumentString,
    ) -> Result<(), ParseStop> {
        if self.stop().is_none() {
            return self.require_document_string(allocation, value);
        }
        self.validate_document_string(allocation, value)
    }
    fn validate_document_string(
        &mut self,
        allocation: &DocumentStringAllocation,
        value: &crate::DocumentString,
    ) -> Result<(), ParseStop> {
        if !Arc::ptr_eq(&allocation.owner, &self.identity) {
            return self.fail(ParseStop::InternalInvariant(Invariant::ForeignControl));
        }
        if allocation
            .backing
            .as_ref()
            .is_none_or(|backing| backing.as_ptr() != Arc::as_ptr(value.backing()))
            || value.len() as u64 != allocation.units
            || value.backing().capacity() != allocation.vector_capacity
        {
            return self.fail(ParseStop::InternalInvariant(Invariant::CapacityMismatch));
        }
        Ok(())
    }
    /// One atomic logical domain transition; real capacity and the immutable
    /// backing are unchanged. A failed cap or poll leaves both currents and the
    /// ticket phase unchanged.
    pub(crate) fn move_document_string_to_managed(
        &mut self,
        allocation: &mut DocumentStringAllocation,
        value: &crate::DocumentString,
    ) -> Result<(), ParseStop> {
        self.require_document_string(allocation, value)?;
        if allocation.resource != HtmlParseResource::BufferedUtf16Units {
            return self.fail(ParseStop::InternalInvariant(Invariant::NonStorageResource));
        }
        let buffered = self
            .usage
            .resource(HtmlParseResource::BufferedUtf16Units)
            .current;
        let managed = self
            .usage
            .resource(HtmlParseResource::ManagedUtf16Units)
            .current;
        let Some(buffered) = buffered.checked_sub(allocation.units) else {
            return self.fail(ParseStop::InternalInvariant(Invariant::ReleaseUnderflow));
        };
        let Some(managed) = managed.checked_add(allocation.units) else {
            return self.representation(HtmlParseResource::ManagedUtf16Units);
        };
        self.check_value(
            HtmlParseResource::ManagedUtf16Units,
            managed,
            allocation.units,
        )?;
        self.set(HtmlParseResource::BufferedUtf16Units, buffered);
        self.set(HtmlParseResource::ManagedUtf16Units, managed);
        allocation.resource = HtmlParseResource::ManagedUtf16Units;
        Ok(())
    }

    pub(crate) fn release_document_string(
        &mut self,
        allocation: DocumentStringAllocation,
    ) -> Result<(), ParseStop> {
        if !Arc::ptr_eq(&allocation.owner, &self.identity) {
            return self.fail(ParseStop::InternalInvariant(Invariant::ForeignControl));
        }
        let Some(bytes) = allocation
            .vector_capacity
            .checked_mul(size_of::<u16>())
            .and_then(|bytes| u64::try_from(bytes).ok())
            .and_then(|bytes| bytes.checked_add(allocation.wrapper_bytes))
        else {
            return self.representation(HtmlParseResource::ManagedCapacityBytes);
        };
        if allocation
            .backing
            .as_ref()
            .is_some_and(|backing| backing.strong_count() != 0)
        {
            return self.fail(ParseStop::InternalInvariant(
                Invariant::SharedAllocationAliased,
            ));
        }
        self.cleanup_storage(allocation.resource, allocation.units, bytes)
    }
}

/// Unique ticket for an actual consumer-owned vector. It cannot register an
/// arbitrary preallocated vector or be cloned to refund the same capacity twice.
#[derive(Debug)]
pub(crate) struct VectorCapacity<T> {
    owner: Arc<()>,
    capacity: usize,
    element: PhantomData<T>,
}
impl<T> VectorCapacity<T> {
    pub(crate) fn capacity(&self) -> usize {
        self.capacity
    }
}

#[derive(Debug)]
pub(crate) struct VectorReleaseError<T> {
    stop: ParseStop,
    vector: Vec<T>,
    allocation: VectorCapacity<T>,
}
impl<T> VectorReleaseError<T> {
    pub(crate) fn into_parts(self) -> (ParseStop, Vec<T>, VectorCapacity<T>) {
        (self.stop, self.vector, self.allocation)
    }
}

/// Known fixed payload only, excluding opaque shared headers and allocator
/// bookkeeping. Nested dynamic payloads retain distinct vector/text tickets.
#[derive(Debug)]
pub(crate) struct SharedAllocation<T> {
    owner: Arc<()>,
    shared: Weak<T>,
    bytes: u64,
    element: PhantomData<T>,
}
impl<T> SharedAllocation<T> {
    pub(crate) fn payload_bytes(&self) -> u64 {
        self.bytes
    }
}

#[derive(Debug)]
pub(crate) struct SharedReleaseError<T> {
    stop: ParseStop,
    shared: Arc<T>,
    allocation: SharedAllocation<T>,
}
impl<T> SharedReleaseError<T> {
    pub(crate) fn into_parts(self) -> (ParseStop, Arc<T>, SharedAllocation<T>) {
        (self.stop, self.shared, self.allocation)
    }
}

/// Non-cloneable accounting ticket for the transferred payload. It must remain
/// alongside the frozen text until the execution owner releases that allocation.
pub(crate) struct DocumentStringAllocation {
    owner: Arc<()>,
    units: u64,
    vector_capacity: usize,
    wrapper_bytes: u64,
    resource: HtmlParseResource,
    data_pointer: usize,
    backing: Option<Weak<Vec<u16>>>,
}
impl DocumentStringAllocation {
    pub(crate) fn units(&self) -> u64 {
        self.units
    }
    pub(crate) fn vector_capacity(&self) -> usize {
        self.vector_capacity
    }
    pub(crate) fn wrapper_bytes(&self) -> u64 {
        self.wrapper_bytes
    }
}

/// Checked layout only; it neither allocates nor admits logical storage.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ReservationPlan {
    additional: usize,
    old_bytes: u64,
    new_bytes: u64,
}
impl ReservationPlan {
    fn for_vec<T>(vector: &Vec<T>, additional: usize) -> Result<Option<Self>, ParseStop> {
        if size_of::<T>() == 0 {
            return Err(ParseStop::InternalInvariant(Invariant::ZeroSizedStorage));
        }
        let length =
            vector
                .len()
                .checked_add(additional)
                .ok_or(ParseStop::RepresentationExhausted {
                    resource: HtmlParseResource::ManagedCapacityBytes,
                })?;
        if length <= vector.capacity() {
            return Ok(None);
        }
        let layout = |slots: usize| {
            slots
                .checked_mul(size_of::<T>())
                .filter(|bytes| *bytes <= isize::MAX as usize)
                .and_then(|bytes| u64::try_from(bytes).ok())
                .ok_or(ParseStop::RepresentationExhausted {
                    resource: HtmlParseResource::ManagedCapacityBytes,
                })
        };
        Ok(Some(Self {
            additional,
            old_bytes: layout(vector.capacity())?,
            new_bytes: layout(length)?,
        }))
    }
}

/// Private variable storage. No mutable Vec escape can bypass fallible growth.
/// Elements must be flat owned records; recursive provider drop is not bounded
/// by this byte ledger. `into_vec` transfers admitted storage without a copy.
pub(crate) struct OwnedVec<T> {
    vector: Vec<T>,
    resource: HtmlParseResource,
    owner: Option<Arc<()>>,
}
impl<T> OwnedVec<T> {
    pub(crate) fn new(resource: HtmlParseResource) -> Result<Self, ParseStop> {
        if !resource.storage() || resource == HtmlParseResource::ManagedNodes {
            return Err(ParseStop::InternalInvariant(Invariant::NonStorageResource));
        }
        if size_of::<T>() == 0 {
            return Err(ParseStop::InternalInvariant(Invariant::ZeroSizedStorage));
        }
        Ok(Self {
            vector: Vec::new(),
            resource,
            owner: None,
        })
    }
    pub(crate) fn as_slice(&self) -> &[T] {
        &self.vector
    }
    pub(crate) fn len(&self) -> usize {
        self.vector.len()
    }
    pub(crate) fn capacity(&self) -> usize {
        self.vector.capacity()
    }
    /// Admits one indexed record visit, including an out-of-bounds attempt.
    /// The borrow grants no payload-work or nested-storage admission: the caller
    /// must charge every field inspection, write and copy before performing it,
    /// and admit/release any nested payload and allocation tickets separately.
    pub(crate) fn get_mut(
        &mut self,
        index: usize,
        control: &mut ParseControl,
    ) -> Result<Option<&mut T>, ParseStop> {
        self.require_owner(control)?;
        control.charge(WorkCategory::RecordVisit, 1)?;
        Ok(self.vector.get_mut(index))
    }
    /// Swaps two actual admitted fixed slots. No payload clone, allocation,
    /// logical-slot change, or mutable-vector escape is involved.
    pub(crate) fn swap_slots(
        &mut self,
        left: usize,
        right: usize,
        control: &mut ParseControl,
    ) -> Result<bool, ParseStop> {
        self.require_owner(control)?;
        control.charge(WorkCategory::RecordVisit, 2)?;
        if left >= self.vector.len() || right >= self.vector.len() {
            return Ok(false);
        }
        if left == right {
            return Ok(true);
        }
        let Some(bytes) = size_of::<T>().checked_mul(2) else {
            return control.representation(HtmlParseResource::AtomicLayoutBytes);
        };
        control.admit_atomic_layout(bytes)?;
        control.charge_many(&[
            (WorkCategory::CopyByte, bytes as u64),
            (WorkCategory::RecordWrite, 2),
        ])?;
        self.vector.swap(left, right);
        Ok(true)
    }
    /// Removes one admitted slot and transfers its value without releasing its
    /// nested payload tickets. The caller owns their separate release/transfer.
    /// Capacity and all high-water observations are retained. An empty attempt
    /// visits only the container; it charges no write or payload copy.
    pub(crate) fn try_pop(&mut self, control: &mut ParseControl) -> Result<Option<T>, ParseStop> {
        self.require_owner(control)?;
        if self.vector.is_empty() {
            control.charge(WorkCategory::RecordVisit, 1)?;
            return Ok(None);
        }
        let Ok(bytes) = u64::try_from(size_of::<T>()) else {
            return control.representation(HtmlParseResource::AtomicLayoutBytes);
        };
        control.charge_many(&[
            (WorkCategory::RecordVisit, 1),
            (WorkCategory::RecordWrite, 1),
            (WorkCategory::CopyByte, bytes),
        ])?;
        if size_of::<T>() > WORK_QUANTUM - 2 {
            control.check_value(HtmlParseResource::AtomicLayoutBytes, bytes, bytes)?;
            control.set(HtmlParseResource::AtomicLayoutBytes, bytes);
            control.usage.largest_atomic_layout = control.usage.largest_atomic_layout.max(bytes);
        }
        control.release(self.resource, 1)?;
        Ok(self.vector.pop())
    }
    /// Failure cleanup only: a first stop must already be latched. The returned
    /// value may only be deallocated or have its nested tickets released; this
    /// is not successful tokenizer progress. Checks the same owner and logical
    /// slot underflow, retaining first-stop, capacity, work and high-water use.
    /// Cleanup is deliberately uncharged so exhaustion cannot strand tickets.
    pub(crate) fn cleanup_pop(
        &mut self,
        control: &mut ParseControl,
    ) -> Result<Option<T>, ParseStop> {
        if control.stop().is_none() {
            return Err(control.fail_invariant(Invariant::CleanupWhileRunning));
        }
        if self
            .owner
            .as_ref()
            .is_some_and(|owner| !Arc::ptr_eq(owner, &control.identity))
        {
            return control.fail(ParseStop::InternalInvariant(Invariant::ForeignControl));
        }
        if self.vector.is_empty() {
            return Ok(None);
        }
        control.cleanup_storage(self.resource, 1, 0)?;
        Ok(self.vector.pop())
    }
    pub(crate) fn require_owner(&mut self, control: &mut ParseControl) -> Result<(), ParseStop> {
        control.checkpoint()?;
        if let Some(owner) = &self.owner {
            if !Arc::ptr_eq(owner, &control.identity) {
                return control.fail(ParseStop::InternalInvariant(Invariant::ForeignControl));
            }
        } else {
            self.owner = Some(control.identity.clone());
        }
        Ok(())
    }
    /// Read-only owner preflight for actual payload disposal, including after
    /// exhaustion. An unbound empty vector has no admitted storage owner.
    pub(crate) fn require_disposal_owner(
        &self,
        control: &mut ParseControl,
    ) -> Result<(), ParseStop> {
        if control.stop().is_none() {
            control.checkpoint()?;
            control.charge(WorkCategory::RecordVisit, 1)?;
        }
        if let Some(owner) = &self.owner {
            if control.stop().is_none() {
                control.charge(WorkCategory::ProviderCall, 1)?;
            }
            if !Arc::ptr_eq(owner, &control.identity) {
                return control.fail(ParseStop::InternalInvariant(Invariant::ForeignControl));
            }
        }
        Ok(())
    }
    fn prepare(
        &mut self,
        control: &mut ParseControl,
        additional: usize,
        copy_bytes: u64,
        writes: u64,
    ) -> Result<(), ParseStop> {
        self.require_owner(control)?;
        let plan = match ReservationPlan::for_vec(&self.vector, additional) {
            Ok(plan) => plan,
            Err(reason) => return control.fail(reason),
        };
        let Ok(units) = u64::try_from(additional) else {
            return control.representation(self.resource);
        };
        let reserve_bytes = match plan {
            Some(plan) => match plan.old_bytes.checked_add(plan.new_bytes) {
                Some(bytes) => bytes,
                None => return control.representation(HtmlParseResource::ManagedCapacityBytes),
            },
            None => 0,
        };
        control.charge_many(&[
            (WorkCategory::CopyByte, copy_bytes),
            (WorkCategory::RecordWrite, writes),
            (WorkCategory::ReserveByte, reserve_bytes),
            (WorkCategory::ProviderCall, u64::from(plan.is_some())),
        ])?;
        control.plan_storage(&[(self.resource, units)])?;
        if let Some(plan) = plan {
            control.reserve(&mut self.vector, plan)?;
        }
        Ok(())
    }
    pub(crate) fn try_push(
        &mut self,
        control: &mut ParseControl,
        value: T,
    ) -> Result<(), ParseStop> {
        control.checkpoint()?;
        let bytes = size_of::<T>() as u64;
        self.prepare(control, 1, bytes, 1)?;
        if size_of::<T>() >= WORK_QUANTUM {
            control.check_value(HtmlParseResource::AtomicLayoutBytes, bytes, bytes)?;
            control.set(HtmlParseResource::AtomicLayoutBytes, bytes);
            control.usage.largest_atomic_layout = control.usage.largest_atomic_layout.max(bytes);
        }
        control.acquire(self.resource, 1)?;
        self.vector.push(value);
        Ok(())
    }
    /// One consumed evidence commit. Both capacities, logical slots and fixed
    /// writes are admitted before either vector's committed length changes.
    pub(crate) fn try_push_pair<U>(
        &mut self,
        other: &mut OwnedVec<U>,
        control: &mut ParseControl,
        value: T,
        other_value: U,
    ) -> Result<(), ParseStop> {
        let bytes = size_of::<T>()
            .checked_add(size_of::<U>())
            .ok_or_else(|| control.fail_invariant(Invariant::CapacityMismatch))?;
        control.admit_atomic_layout(bytes)?;
        self.prepare(control, 1, size_of::<T>() as u64, 1)?;
        other.prepare(control, 1, size_of::<U>() as u64, 1)?;
        control.acquire_many(&[(self.resource, 1), (other.resource, 1)])?;
        // No fallible operation follows admission. Record is published last.
        other.vector.push(other_value);
        self.vector.push(value);
        Ok(())
    }
    pub(crate) fn clear(&mut self, control: &mut ParseControl) -> Result<(), ParseStop> {
        control.checkpoint()?;
        if self
            .owner
            .as_ref()
            .is_some_and(|owner| !Arc::ptr_eq(owner, &control.identity))
        {
            return control.fail(ParseStop::InternalInvariant(Invariant::ForeignControl));
        }
        while !self.vector.is_empty() {
            let bytes = size_of::<T>() as u64;
            control.charge_many(&[
                (WorkCategory::RecordVisit, 1),
                (WorkCategory::RecordWrite, 1),
                (WorkCategory::CopyByte, bytes),
            ])?;
            if size_of::<T>() + 2 > WORK_QUANTUM {
                control.check_value(HtmlParseResource::AtomicLayoutBytes, bytes, bytes)?;
                control.set(HtmlParseResource::AtomicLayoutBytes, bytes);
                control.usage.largest_atomic_layout =
                    control.usage.largest_atomic_layout.max(bytes);
            }
            control.release(self.resource, 1)?;
            drop(self.vector.pop());
        }
        Ok(())
    }
    /// Uses normal push admission, but returns the original value on every
    /// precommit failure so its nested allocation tickets remain recoverable.
    /// The caller must release those tickets through their original owner; this
    /// does not refund admitted work or capacity and never resumes failed work.
    pub(crate) fn try_push_recover(
        &mut self,
        control: &mut ParseControl,
        value: T,
    ) -> Result<(), (ParseStop, T)> {
        let admission = (|| {
            control.checkpoint()?;
            let bytes = size_of::<T>() as u64;
            self.prepare(control, 1, bytes, 1)?;
            if size_of::<T>() >= WORK_QUANTUM {
                control.check_value(HtmlParseResource::AtomicLayoutBytes, bytes, bytes)?;
                control.set(HtmlParseResource::AtomicLayoutBytes, bytes);
                control.usage.largest_atomic_layout =
                    control.usage.largest_atomic_layout.max(bytes);
            }
            control.acquire(self.resource, 1)
        })();
        match admission {
            Ok(()) => {
                self.vector.push(value);
                Ok(())
            }
            Err(stop) => Err((stop, value)),
        }
    }
    pub(crate) fn release(self, control: &mut ParseControl) -> Result<(), ParseStop> {
        if self
            .owner
            .as_ref()
            .is_some_and(|owner| !Arc::ptr_eq(owner, &control.identity))
        {
            return control.fail(ParseStop::InternalInvariant(Invariant::ForeignControl));
        }
        let Some(bytes) = self
            .vector
            .capacity()
            .checked_mul(size_of::<T>())
            .and_then(|bytes| u64::try_from(bytes).ok())
        else {
            return control.representation(HtmlParseResource::ManagedCapacityBytes);
        };
        control.cleanup_storage(self.resource, self.vector.len() as u64, bytes)?;
        drop(self);
        Ok(())
    }
    fn into_vec(self) -> Vec<T> {
        self.vector
    }
}
impl<T: Copy> OwnedVec<T> {
    /// Every copied segment polls and has at most 256 bytes, except a single
    /// larger fixed record admitted under AtomicLayoutBytes.
    pub(crate) fn try_extend_from_slice(
        &mut self,
        control: &mut ParseControl,
        values: &[T],
    ) -> Result<(), ParseStop> {
        control.checkpoint()?;
        self.prepare(control, values.len(), 0, 0)?;
        let stride = (WORK_QUANTUM / size_of::<T>()).max(1);
        for chunk in values.chunks(stride) {
            let bytes = std::mem::size_of_val(chunk) as u64;
            control.charge(WorkCategory::CopyByte, bytes)?;
            if bytes > WORK_QUANTUM as u64 {
                control.check_value(HtmlParseResource::AtomicLayoutBytes, bytes, bytes)?;
                control.set(HtmlParseResource::AtomicLayoutBytes, bytes);
                control.usage.largest_atomic_layout =
                    control.usage.largest_atomic_layout.max(bytes);
            }
            control.acquire(self.resource, chunk.len() as u64)?;
            self.vector.extend_from_slice(chunk);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn fixed_slot_swap_charges_two_real_slot_moves_and_writes_without_storage_growth() {
        let mut c = control();
        let mut slots = OwnedVec::new(HtmlParseResource::AuxiliaryEntries).unwrap();
        slots.try_push(&mut c, 11u16).unwrap();
        slots.try_push(&mut c, 22u16).unwrap();
        let before = c.usage().clone();
        let capacity = slots.capacity();
        assert!(slots.swap_slots(0, 1, &mut c).unwrap());
        assert_eq!(slots.as_slice(), &[22, 11]);
        assert_eq!(slots.capacity(), capacity);
        assert_eq!(
            c.usage().work(WorkCategory::RecordVisit) - before.work(WorkCategory::RecordVisit),
            2
        );
        assert_eq!(
            c.usage().work(WorkCategory::CopyByte) - before.work(WorkCategory::CopyByte),
            4
        );
        assert_eq!(
            c.usage().work(WorkCategory::RecordWrite) - before.work(WorkCategory::RecordWrite),
            2
        );
        assert_eq!(
            c.usage().work(WorkCategory::ReserveByte),
            before.work(WorkCategory::ReserveByte)
        );
        assert_eq!(
            c.usage().resource(HtmlParseResource::AuxiliaryEntries),
            before.resource(HtmlParseResource::AuxiliaryEntries)
        );
        let before = c.usage().clone();
        assert!(slots.swap_slots(0, 0, &mut c).unwrap());
        assert!(!slots.swap_slots(0, 2, &mut c).unwrap());
        assert_eq!(
            c.usage().work(WorkCategory::RecordVisit) - before.work(WorkCategory::RecordVisit),
            4
        );
        assert_eq!(
            c.usage().work(WorkCategory::CopyByte),
            before.work(WorkCategory::CopyByte)
        );
        assert_eq!(
            c.usage().work(WorkCategory::RecordWrite),
            before.work(WorkCategory::RecordWrite)
        );
        slots.release(&mut c).unwrap();
    }

    #[test]
    fn fixed_slot_swap_exact_and_one_short_work_preserve_atomic_mutation() {
        for short in [false, true] {
            let selected = limits();
            let mut c = ParseControl::new(selected, HtmlParseCancellation::new());
            let mut slots = OwnedVec::new(HtmlParseResource::AuxiliaryEntries).unwrap();
            slots.try_push(&mut c, 11u16).unwrap();
            slots.try_push(&mut c, 22u16).unwrap();
            // Two visits + two u16 moves + two writes; initial admitted
            // allocation already establishes the real two-slot atomic layout.
            let remaining = selected.work_credits()
                - c.usage().resource(HtmlParseResource::WorkCredits).current;
            c.charge(WorkCategory::StateStep, remaining - 8 + u64::from(short))
                .unwrap();
            let result = slots.swap_slots(0, 1, &mut c);
            if short {
                assert!(matches!(
                    result,
                    Err(ParseStop::LimitExceeded {
                        resource: HtmlParseResource::WorkCredits,
                        ..
                    })
                ));
                assert_eq!(slots.as_slice(), &[11, 22]);
            } else {
                assert_eq!(result, Ok(true));
                assert_eq!(slots.as_slice(), &[22, 11]);
            }
            slots.release(&mut c).unwrap();
            assert_eq!(
                c.usage()
                    .resource(HtmlParseResource::AuxiliaryEntries)
                    .current,
                0
            );
        }
    }

    #[test]
    fn fixed_slot_swap_rejects_foreign_owner_and_cancellation_before_mutation() {
        let cancel = HtmlParseCancellation::new();
        let mut c = ParseControl::new(limits(), cancel.clone());
        let mut slots = OwnedVec::new(HtmlParseResource::AuxiliaryEntries).unwrap();
        slots.try_push(&mut c, 11u16).unwrap();
        slots.try_push(&mut c, 22u16).unwrap();
        let mut foreign = control();
        assert_eq!(
            slots.swap_slots(0, 1, &mut foreign),
            Err(ParseStop::InternalInvariant(Invariant::ForeignControl))
        );
        assert_eq!(slots.as_slice(), &[11, 22]);
        cancel.cancel();
        assert_eq!(slots.swap_slots(0, 1, &mut c), Err(ParseStop::Cancelled));
        assert_eq!(slots.as_slice(), &[11, 22]);
        slots.release(&mut c).unwrap();
        assert_eq!(
            c.usage()
                .resource(HtmlParseResource::AuxiliaryEntries)
                .current,
            0
        );
    }

    #[test]
    fn unsupported_standalone_state_is_terminal_without_refunding_admitted_storage_or_work() {
        let mut c = control();
        let mut storage = OwnedVec::new(HtmlParseResource::AuxiliaryEntries).unwrap();
        storage.try_push(&mut c, 7u16).unwrap();
        let before = c.usage().clone();
        let stop = ParseStop::UnsupportedStandaloneState(StandaloneUnsupported::Provenance);
        assert_eq!(
            c.stop_unsupported_standalone(StandaloneUnsupported::Provenance),
            stop
        );
        assert_eq!(c.usage(), &before);
        assert_eq!(
            c.stop_unsupported_standalone(StandaloneUnsupported::PendingScript),
            stop
        );
        assert_eq!(c.charge(WorkCategory::StateStep, 1), Err(stop));
        assert_eq!(storage.try_push(&mut c, 9), Err(stop));
        assert_eq!(storage.cleanup_pop(&mut c).unwrap(), Some(7));
        storage.release(&mut c).unwrap();
        assert_eq!(
            c.usage()
                .resource(HtmlParseResource::AuxiliaryEntries)
                .current,
            0
        );
        assert_eq!(
            c.usage()
                .resource(HtmlParseResource::ManagedCapacityBytes)
                .current,
            0
        );
        assert_eq!(
            c.usage().resource(HtmlParseResource::WorkCredits),
            before.resource(HtmlParseResource::WorkCredits)
        );
        assert_eq!(c.stop(), Some(stop));
    }

    #[test]
    fn an_observed_cancellation_precedes_later_unsupported_state() {
        let cancellation = HtmlParseCancellation::new();
        let mut c = ParseControl::new(limits(), cancellation.clone());
        cancellation.cancel();
        assert_eq!(
            c.stop_unsupported_standalone(StandaloneUnsupported::InvalidNesting),
            ParseStop::Cancelled
        );
        assert_eq!(
            c.stop_unsupported_standalone(StandaloneUnsupported::ScriptPause),
            ParseStop::Cancelled
        );
        assert_eq!(c.stop(), Some(ParseStop::Cancelled));
    }
    use super::*;

    #[test]
    fn optional_source_binding_move_refuses_running_control_and_preserves_stopped_usage() {
        let source = HtmlSource::utf8("x");
        let request = crate::HtmlDocumentParseRequest::offline(
            source.clone(),
            crate::parser::tree_builder::tests::limits(100),
        );
        let mut c = ParseControl::new(*request.limits(), request.cancellation().clone());
        c.admit_source(request.source()).unwrap();
        assert_eq!(
            c.take_stopped_optional_source_binding(),
            Err(ParseStop::InternalInvariant(Invariant::CleanupWhileRunning))
        );
        assert!(c.stop().is_none());
        c.require_source(&source).unwrap();
        request.cancellation().cancel();
        assert_eq!(c.checkpoint(), Err(ParseStop::Cancelled));
        let first = c.first_stop_usage().unwrap().clone();
        let current = c.usage().clone();
        assert_eq!(
            c.stopped_source_binding().unwrap().unwrap().0,
            *source.identity()
        );
        let binding = c.take_stopped_optional_source_binding().unwrap().unwrap();
        assert_eq!(&binding.0, source.identity());
        assert_eq!(binding.1, source.coordinates());
        assert!(c.stopped_source_binding().unwrap().is_none());
        assert_eq!(c.usage(), &current);
        assert_eq!(c.first_stop_usage(), Some(&first));
    }
    use crate::DocumentString;

    #[test]
    fn pending_form_effect_stop_is_canonical_and_cannot_replace_first_failure() {
        for invariant in [
            Invariant::PendingFormEffects,
            Invariant::IncompleteFormAssociationCreation,
        ] {
            let mut c = ParseControl::new(limits(), HtmlParseCancellation::new());
            let stop = ParseStop::InternalInvariant(invariant);
            assert_eq!(c.fail_invariant(invariant), stop);
            assert_eq!(c.charge(WorkCategory::RecordWrite, 1), Err(stop));
            assert_eq!(c.fail_invariant(Invariant::ForeignSource), stop);
            assert_eq!(c.checkpoint(), Err(stop));
        }
        let cancellation = HtmlParseCancellation::new();
        let mut cancelled = ParseControl::new(limits(), cancellation.clone());
        cancellation.cancel();
        assert_eq!(
            cancelled.fail_invariant(Invariant::PendingFormEffects),
            ParseStop::Cancelled
        );
        assert_eq!(cancelled.stop(), Some(ParseStop::Cancelled));
    }

    #[test]
    fn first_stop_snapshot_excludes_denied_growth_and_survives_terminal_refund() {
        let caps = limits()
            .into_builder()
            .auxiliary_entries(5)
            .finish()
            .unwrap();
        let mut c = ParseControl::new(caps, HtmlParseCancellation::new());
        assert!(c.first_stop_usage().is_none());
        c.acquire(HtmlParseResource::AuxiliaryEntries, 4).unwrap();
        c.charge(WorkCategory::RecordWrite, 3).unwrap();
        let before = c.usage().clone();
        let stop = c
            .acquire(HtmlParseResource::AuxiliaryEntries, 2)
            .unwrap_err();
        assert_eq!(
            stop,
            ParseStop::LimitExceeded {
                resource: HtmlParseResource::AuxiliaryEntries,
                limit: 5,
                current: 4,
                requested_delta: 2,
            }
        );
        assert_eq!(c.first_stop_usage(), Some(&before));
        let owner = c.identity();
        c.cleanup_release(&owner, HtmlParseResource::AuxiliaryEntries, 4)
            .unwrap();
        assert_eq!(
            c.usage()
                .resource(HtmlParseResource::AuxiliaryEntries)
                .current,
            0
        );
        assert_eq!(
            c.first_stop_usage()
                .unwrap()
                .resource(HtmlParseResource::AuxiliaryEntries)
                .current,
            4
        );
        c.cancellation.cancel();
        assert_eq!(c.checkpoint(), Err(stop));
        assert_eq!(c.fail_invariant(Invariant::ForeignControl), stop);
        assert_eq!(c.first_stop_usage(), Some(&before));
    }
    #[test]
    fn cancellation_snapshot_retains_real_vector_capacity_before_disposal() {
        let cancellation = HtmlParseCancellation::new();
        let mut c = ParseControl::new(limits(), cancellation.clone());
        let mut values = OwnedVec::new(HtmlParseResource::AuxiliaryEntries).unwrap();
        values.try_push(&mut c, 7_u16).unwrap();
        let before = c.usage().clone();
        assert_eq!(
            before.resource(HtmlParseResource::AuxiliaryEntries).current,
            1
        );
        assert!(before.actual_capacity_bytes >= size_of::<u16>() as u64);
        cancellation.cancel();
        assert_eq!(c.checkpoint(), Err(ParseStop::Cancelled));
        assert_eq!(c.first_stop_usage(), Some(&before));
        values.release(&mut c).unwrap();
        assert_eq!(c.usage().actual_capacity_bytes, 0);
        assert_eq!(
            c.usage()
                .resource(HtmlParseResource::AuxiliaryEntries)
                .current,
            0
        );
        assert_eq!(c.first_stop_usage(), Some(&before));
        assert_eq!(
            c.usage().actual_capacity_high_water,
            before.actual_capacity_high_water
        );
        assert_eq!(
            c.stop_unsupported_standalone(StandaloneUnsupported::Provenance),
            ParseStop::Cancelled
        );
        assert_eq!(c.first_stop_usage(), Some(&before));
    }
    fn limits() -> HtmlParseLimits {
        HtmlParseLimits::builder()
            .source_units(1_000)
            .buffered_utf16_units(1_000)
            .token_attributes(1_000)
            .auxiliary_entries(1_000)
            .managed_nodes(1_000)
            .managed_edges(1_000)
            .managed_attributes(1_000)
            .managed_utf16_units(1_000)
            .source_map_entries(1_000)
            .source_map_payload_units(1_000)
            .diagnostics(1_000)
            .diagnostic_payload_units(1_000)
            .context_effects(1_000)
            .open_elements(1_000)
            .tree_depth(1_000)
            .managed_capacity_bytes(16_000)
            .peak_transient_bytes(32_000)
            .atomic_layout_bytes(16_000)
            .work_credits(100_000)
            .finish()
            .unwrap()
    }
    fn control() -> ParseControl {
        ParseControl::new(limits(), HtmlParseCancellation::new())
    }

    fn attribute(local: &str) -> crate::Attribute {
        crate::Attribute::new(
            crate::QualifiedName::new(crate::Namespace::none(), None, local),
            "v",
        )
    }
    #[test]
    fn actual_attribute_edit_restores_ticket_weak_on_reserve_stop_and_retains_contents() {
        let mut c = control();
        let (mut vector, mut capacity) = c.new_vector().unwrap();
        c.reserve_vector(&mut vector, &mut capacity, 1).unwrap();
        vector.push(attribute("a"));
        let (mut backing, mut shared) = c.share_owned(vector).unwrap();
        c.limits[HtmlParseResource::ManagedCapacityBytes as usize] =
            c.usage().actual_capacity_bytes;
        let before = c.usage().actual_capacity_bytes;
        assert!(matches!(
            c.reserve_attribute_storage(&mut backing, &mut shared, &mut capacity, 100),
            Err(ParseStop::LimitExceeded {
                resource: HtmlParseResource::ManagedCapacityBytes,
                ..
            })
        ));
        assert_eq!(backing.len(), 1);
        assert_eq!(backing[0].name().local_name().to_utf8().unwrap(), "a");
        assert_eq!(Arc::weak_count(&backing), 1);
        assert_eq!(c.usage().actual_capacity_bytes, before);
        let stop = c.stop();
        let vector = c.release_shared(backing, shared).unwrap();
        c.release_vector(vector, capacity).unwrap();
        assert_eq!(c.usage().actual_capacity_bytes, 0);
        assert_eq!(c.stop(), stop);
    }
    #[test]
    fn actual_attribute_edit_rejects_external_strong_and_weak_owners_without_dropping_tickets() {
        for weak in [false, true] {
            let mut c = control();
            let (vector, mut capacity) = c.new_vector().unwrap();
            let (mut backing, mut shared) = c.share_owned(vector).unwrap();
            let strong = (!weak).then(|| backing.clone());
            let external_weak = weak.then(|| Arc::downgrade(&backing));
            assert_eq!(
                c.reserve_attribute_storage(&mut backing, &mut shared, &mut capacity, 1),
                Err(ParseStop::InternalInvariant(
                    Invariant::SharedAllocationAliased
                ))
            );
            assert!(backing.is_empty());
            drop(strong);
            drop(external_weak);
            assert_eq!(Arc::weak_count(&backing), 1);
            let vector = c.release_shared(backing, shared).unwrap();
            c.release_vector(vector, capacity).unwrap();
            assert_eq!(c.usage().actual_capacity_bytes, 0);
        }
    }

    #[test]
    fn actual_attribute_replacement_qualifies_owner_aliases_and_cancellation_before_take() {
        for case in 0..4 {
            let cancellation = HtmlParseCancellation::new();
            let mut c = ParseControl::new(limits(), cancellation.clone());
            let (mut values, mut vector) = c.new_vector().unwrap();
            c.reserve_vector(&mut values, &mut vector, 1).unwrap();
            values.push(attribute("old"));
            let (mut backing, mut shared) = c.share_owned(values).unwrap();
            let strong = (case == 1).then(|| backing.clone());
            let weak = (case == 2).then(|| Arc::downgrade(&backing));
            if case == 3 {
                cancellation.cancel();
            }
            let mut incoming = Some(attribute("new"));
            let mut foreign = control();
            let active = if case == 0 { &mut foreign } else { &mut c };
            let stop = active
                .replace_attribute_slot(&mut backing, &mut shared, &vector, 0, &mut incoming)
                .unwrap_err();
            assert_eq!(
                stop,
                if case == 0 {
                    ParseStop::InternalInvariant(Invariant::ForeignControl)
                } else if case == 3 {
                    ParseStop::Cancelled
                } else {
                    ParseStop::InternalInvariant(Invariant::SharedAllocationAliased)
                }
            );
            assert_eq!(backing[0].name().local_name().to_utf8().unwrap(), "old");
            assert_eq!(
                incoming
                    .as_ref()
                    .unwrap()
                    .name()
                    .local_name()
                    .to_utf8()
                    .unwrap(),
                "new"
            );
            drop(strong);
            drop(weak);
            assert_eq!(Arc::weak_count(&backing), 1);
            let values = c.release_shared(backing, shared).unwrap();
            c.release_vector(values, vector).unwrap();
            assert_eq!(c.usage().actual_capacity_bytes, 0);
            if case == 0 {
                assert!(c.stop().is_none());
                assert_eq!(foreign.stop(), Some(stop));
            } else {
                assert_eq!(c.stop(), Some(stop));
            }
        }
    }

    #[test]
    fn hidden_input_metadata_slots_are_admitted_before_take_and_bound_only_to_actual_nodes() {
        use crate::construction::parser_session::{SessionError, release_token_kind};
        use crate::parser::tokenizer::{
            states::State,
            tests::LexerFixture,
            token::{Event, TokenKind},
        };
        use crate::parser::tree_builder::StandaloneDocumentDriver;
        // One node-document binding precedes the unchanged four Hidden
        // metadata slots. Four extra slots stop before lexical take; five
        // reach the first lexical String-slot admission after take.
        for additional in [4, 5] {
            let request = crate::HtmlDocumentParseRequest::offline(
                crate::HtmlSource::utf8("<html><head></head><input type=hidden>"),
                limits()
                    .into_builder()
                    .atomic_layout_bytes(1_000_000)
                    .managed_capacity_bytes(1_000_000)
                    .peak_transient_bytes(2_000_000)
                    .work_credits(10_000_000)
                    .finish()
                    .unwrap(),
            );
            let mut driver = Some(crate::parser::tree_builder::entry::initialized_driver(
                &request,
            ));
            for _ in 0..128 {
                if driver.as_mut().unwrap().standalone_parts().is_some() {
                    break;
                }
                driver.as_mut().unwrap().step().unwrap();
            }
            let (mut c, mut session, capability) =
                StandaloneDocumentDriver::take_standalone_fixture(&mut driver).unwrap();
            let mut lexer = LexerFixture::new(request.source(), State::Data, None, &mut c).unwrap();
            let (mut input, token_source) = loop {
                let event = lexer.next(&mut c, false).unwrap().unwrap();
                match event {
                    Event::Token(event) => {
                        let token = event.into_token(&mut c).unwrap();
                        if matches!(&token.kind,TokenKind::Tag(tag) if tag.name.value.text().code_units()==[105,110,112,117,116])
                        {
                            let TokenKind::Tag(tag) = token.kind else {
                                unreachable!()
                            };
                            break (Some(tag), token.source);
                        }
                        release_token_kind(&mut Some(token.kind), &mut c).unwrap();
                    }
                    event => event.release(&mut c).unwrap(),
                }
            };
            let parent = session.root(&mut c).unwrap();
            let pointer = input
                .as_ref()
                .unwrap()
                .name
                .value
                .text()
                .code_units()
                .as_ptr();
            let nodes = c.usage().resource(HtmlParseResource::ManagedNodes).current;
            let slots = c
                .usage()
                .resource(HtmlParseResource::AuxiliaryEntries)
                .current;
            c.limits[HtmlParseResource::AuxiliaryEntries as usize] = slots + additional;
            assert!(matches!(
                session.allocate_hidden_input_for_creation(
                    &mut c,
                    &mut input,
                    crate::construction::parser_session::CreationLocation {
                        intended_parent: &parent,
                        current_form: None,
                        context: crate::construction::parser_session::CreationContext::Ordinary
                    },
                    &token_source,
                    &capability
                ),
                Err(SessionError::Stop(ParseStop::LimitExceeded {
                    resource: HtmlParseResource::AuxiliaryEntries,
                    ..
                }))
            ));
            if additional == 4 {
                assert_eq!(
                    input
                        .as_ref()
                        .unwrap()
                        .name
                        .value
                        .text()
                        .code_units()
                        .as_ptr(),
                    pointer
                );
            } else {
                assert!(input.is_none());
            }
            assert_eq!(
                c.usage().resource(HtmlParseResource::ManagedNodes).current,
                nodes
            );
            assert_eq!(
                c.usage()
                    .resource(HtmlParseResource::AuxiliaryEntries)
                    .current,
                slots + additional
            );
            let first = c.stop();
            if let Some(tag) = input.take() {
                release_token_kind(&mut Some(TokenKind::Tag(tag)), &mut c).unwrap();
            }
            lexer.release(&mut c).unwrap();
            session.release(&mut c).unwrap();
            assert_eq!(c.stop(), first);
            for resource in [
                HtmlParseResource::BufferedUtf16Units,
                HtmlParseResource::ManagedUtf16Units,
                HtmlParseResource::TokenAttributes,
                HtmlParseResource::AuxiliaryEntries,
                HtmlParseResource::SourceMapEntries,
                HtmlParseResource::ManagedCapacityBytes,
            ] {
                assert_eq!(c.usage().resource(resource).current, 0);
            }
        }
    }

    #[test]
    fn actual_attribute_replacement_counts_real_moves_without_growth() {
        let mut c = control();
        let (mut values, mut capacity) = c.new_vector().unwrap();
        c.reserve_vector(&mut values, &mut capacity, 1).unwrap();
        values.push(attribute("old"));
        let (mut backing, mut shared) = c.share_owned(values).unwrap();
        let mut incoming = Some(attribute("new"));
        let before = c.usage().clone();
        let old = c
            .replace_attribute_slot(&mut backing, &mut shared, &capacity, 0, &mut incoming)
            .unwrap();
        assert!(incoming.is_none());
        assert_eq!(old.name().local_name().to_utf8().unwrap(), "old");
        assert_eq!(backing[0].name().local_name().to_utf8().unwrap(), "new");
        assert_eq!(
            c.usage().actual_capacity_bytes,
            before.actual_capacity_bytes
        );
        assert_eq!(
            c.usage().work(WorkCategory::RecordVisit) - before.work(WorkCategory::RecordVisit),
            5
        );
        assert_eq!(
            c.usage().work(WorkCategory::ProviderCall) - before.work(WorkCategory::ProviderCall),
            9
        );
        assert_eq!(
            c.usage().work(WorkCategory::RecordWrite) - before.work(WorkCategory::RecordWrite),
            5
        );
        assert_eq!(
            c.usage().work(WorkCategory::CopyByte) - before.work(WorkCategory::CopyByte),
            (2 * size_of::<Weak<Vec<crate::Attribute>>>() + 3 * size_of::<crate::Attribute>())
                as u64
        );
        assert_eq!(Arc::weak_count(&backing), 1);
        drop(old);
        let values = c.release_shared(backing, shared).unwrap();
        c.release_vector(values, capacity).unwrap();
        assert_eq!(c.usage().actual_capacity_bytes, 0);
    }
    #[test]
    fn actual_attribute_replacement_atomic_and_work_caps_preserve_incoming_and_ticket() {
        for atomic in [true, false] {
            for short in [true, false] {
                let mut c = control();
                let (mut values, mut capacity) = c.new_vector().unwrap();
                c.reserve_vector(&mut values, &mut capacity, 1).unwrap();
                values.push(attribute("old"));
                let (mut backing, mut shared) = c.share_owned(values).unwrap();
                let mut incoming = Some(attribute("new"));
                if atomic {
                    c.limits[HtmlParseResource::AtomicLayoutBytes as usize] =
                        (3 * size_of::<crate::Attribute>()) as u64 - u64::from(short);
                } else {
                    // prepare: 3 visits+9 providers+2 Weak moves+2 writes.
                    // slot: 2 visits+3 Attribute moves+3 writes.
                    let work = 5
                        + 9
                        + 5
                        + 2 * size_of::<Weak<Vec<crate::Attribute>>>()
                        + 3 * size_of::<crate::Attribute>();
                    c.limits[HtmlParseResource::WorkCredits as usize] =
                        c.usage().resource(HtmlParseResource::WorkCredits).current + work as u64
                            - u64::from(short);
                }
                let result = c.replace_attribute_slot(
                    &mut backing,
                    &mut shared,
                    &capacity,
                    0,
                    &mut incoming,
                );
                if short {
                    assert!(
                        matches!(result, Err(ParseStop::LimitExceeded { resource, .. }) if resource == if atomic {HtmlParseResource::AtomicLayoutBytes} else {HtmlParseResource::WorkCredits})
                    );
                    assert!(incoming.is_some());
                    assert_eq!(backing[0].name().local_name().to_utf8().unwrap(), "old");
                } else {
                    drop(result.unwrap());
                    assert!(incoming.is_none());
                }
                assert_eq!(Arc::weak_count(&backing), 1);
                let stop = c.stop();
                let values = c.release_shared(backing, shared).unwrap();
                c.release_vector(values, capacity).unwrap();
                assert_eq!(c.stop(), stop);
                assert_eq!(c.usage().actual_capacity_bytes, 0);
            }
        }
    }

    #[test]
    fn actual_attribute_append_moves_slots_without_capacity_or_logical_growth() {
        let mut c = control();
        let (mut old, mut old_capacity) = c.new_vector().unwrap();
        c.reserve_vector(&mut old, &mut old_capacity, 1).unwrap();
        old.push(attribute("old"));
        let (mut backing, mut shared) = c.share_owned(old).unwrap();
        let (mut incoming, mut incoming_capacity) = c.new_vector().unwrap();
        c.reserve_vector(&mut incoming, &mut incoming_capacity, 2)
            .unwrap();
        incoming.push(attribute("second"));
        incoming.push(attribute("first"));
        c.acquire(HtmlParseResource::ManagedAttributes, 3).unwrap();
        c.reserve_attribute_storage(&mut backing, &mut shared, &mut old_capacity, 2)
            .unwrap();
        let before = c.usage().actual_capacity_bytes;
        let copies = c.usage().work(WorkCategory::CopyByte);
        for _ in 0..2 {
            c.append_attribute_slot(
                &mut backing,
                &mut shared,
                &old_capacity,
                &mut incoming,
                &incoming_capacity,
            )
            .unwrap();
        }
        assert_eq!(
            backing
                .iter()
                .map(|a| a.name().local_name().to_utf8().unwrap())
                .collect::<Vec<_>>(),
            ["old", "first", "second"]
        );
        assert!(incoming.is_empty());
        assert_eq!(c.usage().actual_capacity_bytes, before);
        assert_eq!(
            c.usage()
                .resource(HtmlParseResource::ManagedAttributes)
                .current,
            3
        );
        assert_eq!(
            c.usage().work(WorkCategory::CopyByte) - copies,
            (4 * size_of::<Weak<Vec<crate::Attribute>>>() + 4 * size_of::<crate::Attribute>())
                as u64
        );
        assert_eq!(Arc::weak_count(&backing), 1);
        let vector = c.release_shared(backing, shared).unwrap();
        c.release_vector(vector, old_capacity).unwrap();
        c.release_vector(incoming, incoming_capacity).unwrap();
        c.release(HtmlParseResource::ManagedAttributes, 3).unwrap();
        assert_eq!(c.usage().actual_capacity_bytes, 0);
    }
    #[test]
    fn actual_attribute_append_rejects_foreign_stage_and_work_cap_before_any_slot_move() {
        for foreign in [false, true] {
            let mut c = control();
            let mut other = control();
            let (mut old, mut old_capacity) = c.new_vector().unwrap();
            c.reserve_vector(&mut old, &mut old_capacity, 4).unwrap();
            old.push(attribute("old"));
            let (mut backing, mut shared) = c.share_owned(old).unwrap();
            let producer = if foreign { &mut other } else { &mut c };
            let (mut incoming, mut incoming_capacity) = producer.new_vector().unwrap();
            producer
                .reserve_vector(&mut incoming, &mut incoming_capacity, 1)
                .unwrap();
            incoming.push(attribute("new"));
            if !foreign {
                c.limits[HtmlParseResource::WorkCredits as usize] =
                    c.usage().resource(HtmlParseResource::WorkCredits).current + 1;
            }
            assert!(
                c.append_attribute_slot(
                    &mut backing,
                    &mut shared,
                    &old_capacity,
                    &mut incoming,
                    &incoming_capacity
                )
                .is_err()
            );
            assert_eq!(backing.len(), 1);
            assert_eq!(incoming.len(), 1);
            assert_eq!(Arc::weak_count(&backing), 1);
            let vector = c.release_shared(backing, shared).unwrap();
            c.release_vector(vector, old_capacity).unwrap();
            if foreign {
                other.release_vector(incoming, incoming_capacity).unwrap();
            } else {
                c.release_vector(incoming, incoming_capacity).unwrap();
            }
            assert_eq!(c.usage().actual_capacity_bytes, 0);
        }
    }

    fn bound_string(
        control: &mut ParseControl,
        units: &[u16],
    ) -> (DocumentString, DocumentStringAllocation) {
        let mut buffer = OwnedVec::new(HtmlParseResource::BufferedUtf16Units).unwrap();
        buffer.try_extend_from_slice(control, units).unwrap();
        let (units, mut ticket) = control.admit_document_string_wrapper(buffer).unwrap();
        let text = DocumentString::from_utf16(units);
        control.bind_document_string(&mut ticket, &text).unwrap();
        (text, ticket)
    }
    #[test]
    fn exact_bound_backing_moves_domains_without_capacity_or_second_copy() {
        let mut c = control();
        let (text, mut ticket) = bound_string(&mut c, &[0xd800, 0xd83d, 0xde00, 0]);
        let capacity = c.usage().actual_capacity_bytes;
        let copies = c.usage().work(WorkCategory::CopyByte);
        c.move_document_string_to_managed(&mut ticket, &text)
            .unwrap();
        assert_eq!(
            c.usage()
                .resource(HtmlParseResource::BufferedUtf16Units)
                .current,
            0
        );
        assert_eq!(
            c.usage()
                .resource(HtmlParseResource::ManagedUtf16Units)
                .current,
            4
        );
        assert_eq!(c.usage().actual_capacity_bytes, capacity);
        assert_eq!(c.usage().work(WorkCategory::CopyByte), copies);
        assert_eq!(text.code_units(), &[0xd800, 0xd83d, 0xde00, 0]);
        drop(text);
        c.release_document_string(ticket).unwrap();
        assert_eq!(c.usage().actual_capacity_bytes, 0);
        assert_eq!(
            c.usage()
                .resource(HtmlParseResource::ManagedUtf16Units)
                .current,
            0
        );
    }
    #[test]
    fn one_short_managed_domain_cap_preserves_buffered_ticket_and_both_currents() {
        let caps = limits()
            .into_builder()
            .managed_utf16_units(1)
            .finish()
            .unwrap();
        let mut c = ParseControl::new(caps, HtmlParseCancellation::new());
        let (text, mut ticket) = bound_string(&mut c, &[0x61, 0x62]);
        let capacity = c.usage().actual_capacity_bytes;
        assert_eq!(
            c.move_document_string_to_managed(&mut ticket, &text),
            Err(ParseStop::LimitExceeded {
                resource: HtmlParseResource::ManagedUtf16Units,
                limit: 1,
                current: 0,
                requested_delta: 2,
            })
        );
        assert_eq!(ticket.resource, HtmlParseResource::BufferedUtf16Units);
        assert_eq!(
            c.usage()
                .resource(HtmlParseResource::BufferedUtf16Units)
                .current,
            2
        );
        assert_eq!(
            c.usage()
                .resource(HtmlParseResource::ManagedUtf16Units)
                .current,
            0
        );
        assert_eq!(c.usage().actual_capacity_bytes, capacity);
        let stop = c.stop();
        drop(text);
        c.release_document_string(ticket).unwrap();
        assert_eq!(c.stop(), stop);
        assert_eq!(c.usage().actual_capacity_bytes, 0);
    }
    #[test]
    fn matching_units_wrong_backing_and_foreign_owner_cannot_move_ticket() {
        let mut c = control();
        let (text, mut ticket) = bound_string(&mut c, &[0x61]);
        let mut foreign = control();
        assert_eq!(
            foreign.move_document_string_to_managed(&mut ticket, &text),
            Err(ParseStop::InternalInvariant(Invariant::ForeignControl))
        );
        assert_eq!(
            c.usage()
                .resource(HtmlParseResource::BufferedUtf16Units)
                .current,
            1
        );
        let other = DocumentString::from_utf16(vec![0x61]);
        assert_eq!(
            c.require_document_string(&ticket, &other),
            Err(ParseStop::InternalInvariant(Invariant::CapacityMismatch))
        );
        drop(text);
        c.release_document_string(ticket).unwrap();
        assert_eq!(c.usage().actual_capacity_bytes, 0);
    }
    #[test]
    fn duplicate_transfer_and_cancellation_reject_atomically_and_managed_cleanup_is_terminal() {
        let cancel = HtmlParseCancellation::new();
        let mut c = ParseControl::new(limits(), cancel.clone());
        let (text, mut ticket) = bound_string(&mut c, &[0x61]);
        c.move_document_string_to_managed(&mut ticket, &text)
            .unwrap();
        assert_eq!(
            c.move_document_string_to_managed(&mut ticket, &text),
            Err(ParseStop::InternalInvariant(Invariant::NonStorageResource))
        );
        assert_eq!(
            c.usage()
                .resource(HtmlParseResource::ManagedUtf16Units)
                .current,
            1
        );
        let work = c.usage().work;
        drop(text);
        c.release_document_string(ticket).unwrap();
        assert_eq!(c.usage().work, work);
        assert_eq!(
            c.usage()
                .resource(HtmlParseResource::ManagedUtf16Units)
                .current,
            0
        );

        let mut c = ParseControl::new(limits(), cancel.clone());
        let (text, mut ticket) = bound_string(&mut c, &[0x62]);
        cancel.cancel();
        assert_eq!(
            c.move_document_string_to_managed(&mut ticket, &text),
            Err(ParseStop::Cancelled)
        );
        assert_eq!(ticket.resource, HtmlParseResource::BufferedUtf16Units);
        assert_eq!(
            c.usage()
                .resource(HtmlParseResource::BufferedUtf16Units)
                .current,
            1
        );
        drop(text);
        c.release_document_string(ticket).unwrap();
        assert_eq!(c.stop(), Some(ParseStop::Cancelled));
        assert_eq!(c.usage().actual_capacity_bytes, 0);
    }

    #[test]
    fn all_limits_have_independent_slots() {
        let limits = HtmlParseLimits::builder()
            .source_units(1)
            .buffered_utf16_units(2)
            .token_attributes(3)
            .auxiliary_entries(4)
            .managed_nodes(5)
            .managed_edges(6)
            .managed_attributes(7)
            .managed_utf16_units(8)
            .source_map_entries(9)
            .source_map_payload_units(10)
            .diagnostics(11)
            .diagnostic_payload_units(12)
            .context_effects(13)
            .open_elements(14)
            .tree_depth(15)
            .managed_capacity_bytes(16)
            .peak_transient_bytes(17)
            .atomic_layout_bytes(18)
            .work_credits(19)
            .finish()
            .unwrap();
        assert_eq!(
            limits.values(),
            [
                1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19
            ]
        );
    }
    #[test]
    fn source_counts_supplied_basis_once() {
        let source = HtmlSource::utf8("a😀");
        let mut control = control();
        control.admit_source(&source).unwrap();
        control.admit_source(&source.clone()).unwrap();
        assert_eq!(
            control
                .usage()
                .resource(HtmlParseResource::SourceUnits)
                .current,
            5
        );
        assert_eq!(
            control
                .usage()
                .resource(HtmlParseResource::ManagedCapacityBytes)
                .current,
            0
        );
        let source = HtmlSource::utf16(DocumentString::from_utf16([0xD800, 0x61]));
        let mut control = super::tests::control();
        control.admit_source(&source).unwrap();
        assert_eq!(
            control
                .usage()
                .resource(HtmlParseResource::SourceUnits)
                .current,
            2
        );
    }
    #[test]
    fn equal_text_different_identity_is_rejected() {
        let mut control = control();
        control.admit_source(&HtmlSource::utf8("x")).unwrap();
        assert_eq!(
            control.admit_source(&HtmlSource::utf8("x")),
            Err(ParseStop::InternalInvariant(Invariant::ForeignSource))
        );
    }
    #[test]
    fn zero_source_cap_admits_only_empty() {
        let mut limits = limits();
        limits = limits.into_builder().source_units(0).finish().unwrap();
        let mut control = ParseControl::new(limits, HtmlParseCancellation::new());
        control.admit_source(&HtmlSource::utf8("")).unwrap();
        let mut control = ParseControl::new(limits, HtmlParseCancellation::new());
        assert_eq!(
            control.admit_source(&HtmlSource::utf8("a")),
            Err(ParseStop::LimitExceeded {
                resource: HtmlParseResource::SourceUnits,
                limit: 0,
                current: 0,
                requested_delta: 1
            })
        );
    }
    #[test]
    fn cancellation_is_sticky_across_threads() {
        let cancellation = HtmlParseCancellation::new();
        let other = cancellation.clone();
        std::thread::spawn(move || {
            other.cancel();
            other.cancel();
        })
        .join()
        .unwrap();
        let mut control = ParseControl::new(limits(), cancellation);
        assert_eq!(
            control.charge(WorkCategory::StateStep, 1),
            Err(ParseStop::Cancelled)
        );
        assert_eq!(
            control
                .usage()
                .resource(HtmlParseResource::WorkCredits)
                .current,
            0
        );
    }
    #[test]
    fn observed_cancel_precedes_limit_and_first_stop_never_changes() {
        let cancellation = HtmlParseCancellation::new();
        cancellation.cancel();
        let mut limits = limits();
        limits = limits.into_builder().work_credits(0).finish().unwrap();
        let mut control = ParseControl::new(limits, cancellation);
        assert_eq!(
            control.charge(WorkCategory::StateStep, 1),
            Err(ParseStop::Cancelled)
        );
        assert_eq!(
            control.acquire(HtmlParseResource::Diagnostics, u64::MAX),
            Err(ParseStop::Cancelled)
        );
        assert_eq!(control.stop(), Some(ParseStop::Cancelled));
    }
    #[test]
    fn cap_failure_precedes_later_cancel() {
        let cancellation = HtmlParseCancellation::new();
        let mut limits = limits();
        limits = limits.into_builder().work_credits(0).finish().unwrap();
        let mut control = ParseControl::new(limits, cancellation.clone());
        let first = control.charge(WorkCategory::ProviderCall, 1).unwrap_err();
        cancellation.cancel();
        assert_eq!(control.checkpoint(), Err(first));
    }
    #[test]
    fn all_work_categories_accumulate_without_refunds() {
        let mut control = control();
        for category in [
            WorkCategory::InputUnitRead,
            WorkCategory::InspectByte,
            WorkCategory::CopyByte,
            WorkCategory::StateStep,
            WorkCategory::RecordVisit,
            WorkCategory::RecordWrite,
            WorkCategory::ReserveByte,
            WorkCategory::ProviderCall,
        ] {
            control.charge(category, 2).unwrap();
            assert_eq!(control.usage().work(category), 2);
        }
        assert_eq!(
            control
                .usage()
                .resource(HtmlParseResource::WorkCredits)
                .current,
            16
        );
    }
    #[test]
    fn rejected_work_does_not_change_any_work_counter() {
        let mut limits = limits();
        limits = limits.into_builder().work_credits(3).finish().unwrap();
        let mut control = ParseControl::new(limits, HtmlParseCancellation::new());
        control.charge(WorkCategory::StateStep, 2).unwrap();
        assert_eq!(
            control.charge(WorkCategory::InputUnitRead, 2),
            Err(ParseStop::LimitExceeded {
                resource: HtmlParseResource::WorkCredits,
                limit: 3,
                current: 2,
                requested_delta: 2
            })
        );
        assert_eq!(control.usage().work(WorkCategory::InputUnitRead), 0);
        assert_eq!(
            control
                .usage()
                .resource(HtmlParseResource::WorkCredits)
                .current,
            2
        );
    }
    #[test]
    fn arithmetic_overflow_is_terminal_and_atomic() {
        let mut limits = limits();
        limits = limits
            .into_builder()
            .work_credits(u64::MAX)
            .finish()
            .unwrap();
        let mut control = ParseControl::new(limits, HtmlParseCancellation::new());
        control.charge(WorkCategory::StateStep, u64::MAX).unwrap();
        assert_eq!(
            control.charge(WorkCategory::RecordVisit, 1),
            Err(ParseStop::RepresentationExhausted {
                resource: HtmlParseResource::WorkCredits
            })
        );
        assert_eq!(control.usage().work(WorkCategory::RecordVisit), 0);
    }
    #[test]
    fn coupled_resources_fail_in_declared_order_without_partial_admission() {
        let mut limits = limits();
        limits = limits.into_builder().diagnostics(0).finish().unwrap();
        limits = limits
            .into_builder()
            .buffered_utf16_units(0)
            .finish()
            .unwrap();
        let mut control = ParseControl::new(limits, HtmlParseCancellation::new());
        assert_eq!(
            control.acquire_many(&[
                (HtmlParseResource::Diagnostics, 1),
                (HtmlParseResource::BufferedUtf16Units, 1)
            ]),
            Err(ParseStop::LimitExceeded {
                resource: HtmlParseResource::BufferedUtf16Units,
                limit: 0,
                current: 0,
                requested_delta: 1
            })
        );
        assert_eq!(
            control
                .usage()
                .resource(HtmlParseResource::Diagnostics)
                .current,
            0
        );
    }
    #[test]
    fn diagnostic_exhaustion_is_a_terminal_error() {
        let mut limits = limits();
        limits = limits.into_builder().diagnostics(1).finish().unwrap();
        let mut control = ParseControl::new(limits, HtmlParseCancellation::new());
        control.acquire(HtmlParseResource::Diagnostics, 1).unwrap();
        assert!(matches!(
            control.acquire(HtmlParseResource::Diagnostics, 1),
            Err(ParseStop::LimitExceeded {
                resource: HtmlParseResource::Diagnostics,
                ..
            })
        ));
        assert!(control.checkpoint().is_err());
    }
    #[test]
    fn release_retains_high_water_and_managed_nodes_are_not_refunded() {
        let mut control = control();
        control.acquire(HtmlParseResource::OpenElements, 7).unwrap();
        control.release(HtmlParseResource::OpenElements, 5).unwrap();
        assert_eq!(
            control.usage().resource(HtmlParseResource::OpenElements),
            ResourceUsage {
                current: 2,
                high_water: 7
            }
        );
        control.acquire(HtmlParseResource::ManagedNodes, 1).unwrap();
        assert_eq!(
            control.release(HtmlParseResource::ManagedNodes, 1),
            Err(ParseStop::InternalInvariant(Invariant::NonReclaimableNodes))
        );
    }
    #[test]
    fn release_underflow_cannot_wrap() {
        let mut control = control();
        assert_eq!(
            control.release(HtmlParseResource::ManagedEdges, 1),
            Err(ParseStop::InternalInvariant(Invariant::ReleaseUnderflow))
        );
        assert_eq!(
            control
                .usage()
                .resource(HtmlParseResource::ManagedEdges)
                .current,
            0
        );
    }
    #[test]
    fn ancestry_depth_is_independent_of_open_stack() {
        let mut control = control();
        control.acquire(HtmlParseResource::OpenElements, 2).unwrap();
        control.observe_tree_depth(12).unwrap();
        control.observe_tree_depth(3).unwrap();
        assert_eq!(
            control.usage().resource(HtmlParseResource::TreeDepth),
            ResourceUsage {
                current: 3,
                high_water: 12
            }
        );
        assert_eq!(
            control
                .usage()
                .resource(HtmlParseResource::OpenElements)
                .current,
            2
        );
    }
    #[test]
    fn batches_bound_owned_work_and_poll_even_empty() {
        let mut control = control();
        assert_eq!(
            control
                .work_batch(WorkCategory::InspectByte, 1_000)
                .unwrap(),
            256
        );
        assert_eq!(control.work_batch(WorkCategory::InspectByte, 3).unwrap(), 3);
        control.cancellation.cancel();
        assert_eq!(
            control.work_batch(WorkCategory::InspectByte, 0),
            Err(ParseStop::Cancelled)
        );
    }
    #[test]
    fn reservation_layout_rejects_unrepresentable_growth_without_allocator() {
        assert_eq!(
            ReservationPlan::for_vec(&Vec::<u16>::new(), usize::MAX),
            Err(ParseStop::RepresentationExhausted {
                resource: HtmlParseResource::ManagedCapacityBytes
            })
        );
        assert_eq!(
            ReservationPlan::for_vec(&Vec::<()>::new(), 1),
            Err(ParseStop::InternalInvariant(Invariant::ZeroSizedStorage))
        );
    }
    #[test]
    fn capacity_limit_rejects_before_logical_write() {
        let mut limits = limits();
        limits = limits
            .into_builder()
            .managed_capacity_bytes(1)
            .finish()
            .unwrap();
        let mut control = ParseControl::new(limits, HtmlParseCancellation::new());
        let mut buffer = OwnedVec::<u16>::new(HtmlParseResource::BufferedUtf16Units).unwrap();
        assert!(matches!(
            buffer.try_push(&mut control, 0xD800),
            Err(ParseStop::LimitExceeded {
                resource: HtmlParseResource::ManagedCapacityBytes,
                ..
            })
        ));
        assert_eq!(buffer.len(), 0);
        assert_eq!(buffer.capacity(), 0);
        assert_eq!(
            control
                .usage()
                .resource(HtmlParseResource::BufferedUtf16Units)
                .current,
            0
        );
    }
    #[test]
    fn actual_capacity_and_overlap_are_recorded_and_cleanup_after_stop_works() {
        let mut control = control();
        let mut buffer = OwnedVec::<u16>::new(HtmlParseResource::BufferedUtf16Units).unwrap();
        buffer
            .try_extend_from_slice(&mut control, &[0xD800, 0, 0xDC00])
            .unwrap();
        assert_eq!(buffer.as_slice(), &[0xD800, 0, 0xDC00]);
        assert_eq!(control.usage().actual_capacity_bytes, 6);
        buffer.try_push(&mut control, 0x61).unwrap();
        assert_eq!(
            control
                .usage()
                .resource(HtmlParseResource::PeakTransientBytes)
                .high_water,
            14
        );
        assert_eq!(control.usage().actual_capacity_bytes, 8);
        assert_eq!(control.usage().largest_atomic_layout, 8);
        control.cancellation.cancel();
        control.checkpoint().unwrap_err();
        buffer.release(&mut control).unwrap();
        assert_eq!(control.usage().actual_capacity_bytes, 0);
        assert_eq!(control.usage().actual_capacity_high_water, 8);
        assert_eq!(
            control
                .usage()
                .resource(HtmlParseResource::BufferedUtf16Units),
            ResourceUsage {
                current: 0,
                high_water: 4
            }
        );
        assert_eq!(control.stop(), Some(ParseStop::Cancelled));
    }
    #[test]
    fn clear_retains_capacity_and_freeze_transfer_retains_accounting() {
        let mut control = control();
        let mut buffer = OwnedVec::<u16>::new(HtmlParseResource::BufferedUtf16Units).unwrap();
        buffer.try_push(&mut control, 7).unwrap();
        buffer.clear(&mut control).unwrap();
        assert_eq!(buffer.len(), 0);
        assert_eq!(buffer.capacity(), 1);
        buffer.try_push(&mut control, 9).unwrap();
        let vector = buffer.into_vec();
        assert_eq!(vector, [9]);
        assert_eq!(control.usage().actual_capacity_bytes, 2);
        assert_eq!(control.usage().work(WorkCategory::ProviderCall), 1);
    }
    #[test]
    fn transient_and_atomic_caps_are_independent() {
        for resource in [
            HtmlParseResource::PeakTransientBytes,
            HtmlParseResource::AtomicLayoutBytes,
        ] {
            let mut limits = limits();
            if resource == HtmlParseResource::PeakTransientBytes {
                limits = limits
                    .into_builder()
                    .peak_transient_bytes(1)
                    .finish()
                    .unwrap();
            } else {
                limits = limits
                    .into_builder()
                    .atomic_layout_bytes(1)
                    .finish()
                    .unwrap();
            }
            let mut control = ParseControl::new(limits, HtmlParseCancellation::new());
            let mut buffer = OwnedVec::<u16>::new(HtmlParseResource::BufferedUtf16Units).unwrap();
            assert!(
                matches!(buffer.try_push(&mut control, 1), Err(ParseStop::LimitExceeded { resource: failed, .. }) if failed == resource)
            );
            assert_eq!(buffer.len(), 0);
            assert_eq!(buffer.capacity(), 0);
        }
    }
    #[test]
    fn verification_requires_admission_and_keeps_source_identity() {
        let source = HtmlSource::utf8("a");
        let mut missing = control();
        assert_eq!(
            missing.require_source(&source),
            Err(ParseStop::InternalInvariant(Invariant::MissingSource))
        );
        let mut admitted = control();
        admitted.admit_source(&source).unwrap();
        admitted.require_source(&source.clone()).unwrap();
        assert_eq!(
            admitted.require_source(&HtmlSource::utf8("a")),
            Err(ParseStop::InternalInvariant(Invariant::ForeignSource))
        );
        assert_eq!(
            admitted
                .usage()
                .resource(HtmlParseResource::SourceUnits)
                .current,
            1
        );
    }
    #[test]
    fn cursor_invariant_uses_the_same_first_terminal_owner() {
        let mut control = control();
        assert_eq!(
            control.fail_invariant(Invariant::InputCursorState),
            ParseStop::InternalInvariant(Invariant::InputCursorState)
        );
        control.cancellation.cancel();
        assert_eq!(
            control.fail_invariant(Invariant::ForeignSource),
            ParseStop::InternalInvariant(Invariant::InputCursorState)
        );
    }
    #[test]
    fn storage_cannot_be_mutated_with_a_different_control() {
        let mut owner = control();
        let mut other = control();
        let mut buffer = OwnedVec::<u16>::new(HtmlParseResource::BufferedUtf16Units).unwrap();
        buffer.try_push(&mut owner, 9).unwrap();
        assert_eq!(
            buffer.try_push(&mut other, 10),
            Err(ParseStop::InternalInvariant(Invariant::ForeignControl))
        );
        assert_eq!(buffer.as_slice(), &[9]);
        buffer.release(&mut owner).unwrap();
    }
    #[test]
    fn cleanup_underflow_is_checked_even_after_a_stop() {
        let mut control = control();
        let reason = control
            .cleanup_storage(HtmlParseResource::BufferedUtf16Units, 1, 2)
            .unwrap_err();
        assert_eq!(
            reason,
            ParseStop::InternalInvariant(Invariant::ReleaseUnderflow)
        );
        assert_eq!(control.usage().actual_capacity_bytes, 0);
    }
    #[test]
    fn fixed_shared_payload_is_admitted_without_variable_copy() {
        let mut control = control();
        let mut buffer = OwnedVec::<u16>::new(HtmlParseResource::BufferedUtf16Units).unwrap();
        buffer.try_push(&mut control, 0xD800).unwrap();
        let copied = control.usage().work(WorkCategory::CopyByte);
        let (vector, allocation) = control.admit_document_string_wrapper(buffer).unwrap();
        let wrapper = allocation.wrapper_bytes();
        assert_eq!(allocation.units(), 1);
        assert_eq!(allocation.vector_capacity(), 1);
        let text = DocumentString::from_utf16(vector);
        control.checkpoint().unwrap();
        assert_eq!(text.code_units(), &[0xD800]);
        assert_eq!(wrapper, size_of::<Vec<u16>>() as u64);
        assert_eq!(control.usage().actual_capacity_bytes, 2 + wrapper);
        assert_eq!(control.usage().work(WorkCategory::CopyByte), copied);
        control.cancellation.cancel();
        control.checkpoint().unwrap_err();
        drop(text);
        control.release_document_string(allocation).unwrap();
        assert_eq!(control.usage().actual_capacity_bytes, 0);
    }
    #[test]
    fn allocation_failure_is_fallible_and_does_not_admit_payload() {
        let mut limits = limits();
        limits = limits
            .into_builder()
            .managed_capacity_bytes(usize::MAX as u64)
            .finish()
            .unwrap();
        limits = limits
            .into_builder()
            .peak_transient_bytes(usize::MAX as u64)
            .finish()
            .unwrap();
        limits = limits
            .into_builder()
            .atomic_layout_bytes(isize::MAX as u64)
            .finish()
            .unwrap();
        limits = limits
            .into_builder()
            .work_credits(u64::MAX)
            .finish()
            .unwrap();
        let mut control = ParseControl::new(limits, HtmlParseCancellation::new());
        let mut vector = Vec::<u8>::new();
        let plan = ReservationPlan::for_vec(&vector, isize::MAX as usize)
            .unwrap()
            .unwrap();
        control
            .charge(WorkCategory::ReserveByte, isize::MAX as u64)
            .unwrap();
        control.charge(WorkCategory::ProviderCall, 1).unwrap();
        assert_eq!(
            control.reserve(&mut vector, plan),
            Err(ParseStop::AllocationFailed {
                resource: HtmlParseResource::ManagedCapacityBytes,
                layout_bytes: isize::MAX as u64,
            })
        );
        assert_eq!(vector.capacity(), 0);
        assert_eq!(control.usage().actual_capacity_bytes, 0);
    }
    #[test]
    fn maximal_finite_work_and_valid_zero_caps_are_constructible() {
        let mut limits = limits();
        limits = limits
            .into_builder()
            .work_credits(u64::MAX)
            .finish()
            .unwrap();
        limits = limits
            .into_builder()
            .buffered_utf16_units(0)
            .finish()
            .unwrap();
        limits = limits.into_builder().source_units(0).finish().unwrap();
        let mut control = ParseControl::new(limits, HtmlParseCancellation::new());
        control.charge(WorkCategory::StateStep, u64::MAX).unwrap();
        assert_eq!(
            control
                .usage()
                .resource(HtmlParseResource::WorkCredits)
                .current,
            u64::MAX
        );
    }
    #[test]
    fn source_admission_is_one_metadata_visit_without_input_scanning() {
        let mut control = control();
        control.admit_source(&HtmlSource::utf8("abcdef")).unwrap();
        assert_eq!(control.usage().work(WorkCategory::RecordVisit), 1);
        assert_eq!(control.usage().work(WorkCategory::InputUnitRead), 0);
        assert_eq!(control.usage().work(WorkCategory::InspectByte), 0);
    }
    #[test]
    fn every_storage_cap_rejects_its_own_growth() {
        for resource in RESOURCES.into_iter().filter(|resource| resource.storage()) {
            let mut control = control();
            control.limits[resource as usize] = 0;
            assert_eq!(
                control.acquire(resource, 1),
                Err(ParseStop::LimitExceeded {
                    resource,
                    limit: 0,
                    current: 0,
                    requested_delta: 1,
                })
            );
            assert_eq!(control.usage().resource(resource).current, 0);
        }
    }
    #[test]
    fn non_storage_and_zero_sized_owned_vectors_are_rejected() {
        assert!(matches!(
            OwnedVec::<u16>::new(HtmlParseResource::SourceUnits),
            Err(ParseStop::InternalInvariant(Invariant::NonStorageResource))
        ));
        assert!(matches!(
            OwnedVec::<()>::new(HtmlParseResource::BufferedUtf16Units),
            Err(ParseStop::InternalInvariant(Invariant::ZeroSizedStorage))
        ));
        let mut control = control();
        assert_eq!(
            control.acquire(HtmlParseResource::WorkCredits, 1),
            Err(ParseStop::InternalInvariant(Invariant::NonStorageResource))
        );
    }
    #[test]
    fn control_identity_rejects_another_execution_even_for_the_same_source() {
        let source = HtmlSource::utf8("x");
        let mut first = control();
        let mut second = control();
        first.admit_source(&source).unwrap();
        second.admit_source(&source).unwrap();
        let identity = first.identity();
        first.require_identity(&identity).unwrap();
        assert_eq!(
            second.require_identity(&identity),
            Err(ParseStop::InternalInvariant(Invariant::ForeignControl))
        );
    }
    #[test]
    fn frozen_allocation_ticket_cannot_release_a_different_owner() {
        let mut first = control();
        let mut second = control();
        let mut first_buffer = OwnedVec::<u16>::new(HtmlParseResource::BufferedUtf16Units).unwrap();
        first_buffer.try_push(&mut first, 4).unwrap();
        let (vector, ticket) = first.admit_document_string_wrapper(first_buffer).unwrap();
        let mut second_buffer =
            OwnedVec::<u16>::new(HtmlParseResource::BufferedUtf16Units).unwrap();
        second_buffer.try_push(&mut second, 4).unwrap();
        let (_, second_ticket) = second.admit_document_string_wrapper(second_buffer).unwrap();
        let previous = second.usage().clone();
        assert_eq!(
            second.release_document_string(ticket),
            Err(ParseStop::InternalInvariant(Invariant::ForeignControl))
        );
        assert_eq!(second.usage(), &previous);
        second.release_document_string(second_ticket).unwrap();
        drop(vector);
    }
    #[test]
    fn active_clear_charges_every_popped_slot_and_never_refunds_work() {
        let mut control = control();
        let mut buffer = OwnedVec::<u16>::new(HtmlParseResource::BufferedUtf16Units).unwrap();
        buffer
            .try_extend_from_slice(&mut control, &[1, 2, 3])
            .unwrap();
        let previous = control
            .usage()
            .resource(HtmlParseResource::WorkCredits)
            .current;
        let copy = control.usage().work(WorkCategory::CopyByte);
        buffer.clear(&mut control).unwrap();
        assert_eq!(
            control
                .usage()
                .resource(HtmlParseResource::WorkCredits)
                .current,
            previous + 12
        );
        assert_eq!(control.usage().work(WorkCategory::RecordVisit), 3);
        assert_eq!(control.usage().work(WorkCategory::RecordWrite), 3);
        assert_eq!(control.usage().work(WorkCategory::CopyByte), copy + 6);
        assert_eq!(
            control
                .usage()
                .resource(HtmlParseResource::BufferedUtf16Units),
            ResourceUsage {
                current: 0,
                high_water: 3
            }
        );
        assert_eq!(buffer.capacity(), 3);
    }
    #[test]
    fn failed_clear_leaves_remaining_entries_for_terminal_cleanup() {
        let mut control = control();
        let mut buffer = OwnedVec::<u16>::new(HtmlParseResource::BufferedUtf16Units).unwrap();
        buffer
            .try_extend_from_slice(&mut control, &[1, 2, 3])
            .unwrap();
        let previous = control
            .usage()
            .resource(HtmlParseResource::WorkCredits)
            .current;
        control.limits[HtmlParseResource::WorkCredits as usize] = previous + 4;
        let stop = buffer.clear(&mut control).unwrap_err();
        assert_eq!(buffer.as_slice(), &[1, 2]);
        assert_eq!(
            control
                .usage()
                .resource(HtmlParseResource::BufferedUtf16Units)
                .current,
            2
        );
        let work = control
            .usage()
            .resource(HtmlParseResource::WorkCredits)
            .current;
        buffer.release(&mut control).unwrap();
        assert_eq!(control.stop(), Some(stop));
        assert_eq!(
            control
                .usage()
                .resource(HtmlParseResource::WorkCredits)
                .current,
            work
        );
        assert_eq!(control.usage().actual_capacity_bytes, 0);
    }
    #[test]
    fn work_limit_precedes_storage_limit_for_a_growth_operation() {
        let mut limits = limits();
        limits = limits.into_builder().work_credits(3).finish().unwrap();
        limits = limits
            .into_builder()
            .buffered_utf16_units(0)
            .finish()
            .unwrap();
        let mut control = ParseControl::new(limits, HtmlParseCancellation::new());
        let mut buffer = OwnedVec::<u16>::new(HtmlParseResource::BufferedUtf16Units).unwrap();
        assert!(matches!(
            buffer.try_push(&mut control, 1),
            Err(ParseStop::LimitExceeded {
                resource: HtmlParseResource::WorkCredits,
                ..
            })
        ));
        assert_eq!(
            control
                .usage()
                .resource(HtmlParseResource::BufferedUtf16Units)
                .current,
            0
        );
    }
    #[test]
    fn indexed_access_preserves_bounds_and_charges_only_its_record_visit() {
        let mut control = control();
        let mut entries = OwnedVec::<u16>::new(HtmlParseResource::AuxiliaryEntries).unwrap();
        assert_eq!(entries.get_mut(usize::MAX, &mut control).unwrap(), None);
        entries.try_push(&mut control, 7).unwrap();
        let before = control.usage().clone();
        let entry = entries.get_mut(0, &mut control).unwrap().unwrap();
        control.charge(WorkCategory::InspectByte, 2).unwrap();
        assert_eq!(*entry, 7);
        control
            .charge_many(&[(WorkCategory::RecordWrite, 1), (WorkCategory::CopyByte, 2)])
            .unwrap();
        *entry = 9;
        assert_eq!(entries.as_slice(), &[9]);
        assert_eq!(
            control.usage().work(WorkCategory::RecordVisit),
            before.work(WorkCategory::RecordVisit) + 1
        );
        assert_eq!(
            control.usage().work(WorkCategory::InspectByte),
            before.work(WorkCategory::InspectByte) + 2
        );
        assert_eq!(
            control.usage().work(WorkCategory::CopyByte),
            before.work(WorkCategory::CopyByte) + 2
        );
        assert_eq!(
            control
                .usage()
                .resource(HtmlParseResource::AuxiliaryEntries),
            before.resource(HtmlParseResource::AuxiliaryEntries)
        );
        assert_eq!(entries.get_mut(1, &mut control).unwrap(), None);
        assert_eq!(
            control.usage().work(WorkCategory::RecordVisit),
            before.work(WorkCategory::RecordVisit) + 2
        );
    }
    #[test]
    fn indexed_access_rejects_foreign_owner_before_a_visit() {
        let mut owner = control();
        let mut foreign = control();
        let mut entries = OwnedVec::<u16>::new(HtmlParseResource::AuxiliaryEntries).unwrap();
        entries.try_push(&mut owner, 7).unwrap();
        let before = foreign.usage().clone();
        assert_eq!(
            entries.get_mut(0, &mut foreign),
            Err(ParseStop::InternalInvariant(Invariant::ForeignControl))
        );
        assert_eq!(foreign.usage(), &before);
        assert_eq!(entries.as_slice(), &[7]);
        entries.release(&mut owner).unwrap();
    }
    #[test]
    fn cancellation_prevents_indexed_borrow_and_normal_pop() {
        let mut control = control();
        let mut entries = OwnedVec::<u16>::new(HtmlParseResource::AuxiliaryEntries).unwrap();
        entries.try_push(&mut control, 7).unwrap();
        let before = control.usage().clone();
        control.cancellation.cancel();
        assert_eq!(entries.get_mut(0, &mut control), Err(ParseStop::Cancelled));
        assert_eq!(entries.try_pop(&mut control), Err(ParseStop::Cancelled));
        assert_eq!(control.usage(), &before);
        assert_eq!(entries.as_slice(), &[7]);
        entries.release(&mut control).unwrap();
    }
    #[test]
    fn indexed_borrow_work_cap_is_checked_before_exposing_the_slot() {
        let mut limits = limits();
        limits = limits.into_builder().work_credits(6).finish().unwrap();
        let mut control = ParseControl::new(limits, HtmlParseCancellation::new());
        let mut entries = OwnedVec::<u16>::new(HtmlParseResource::AuxiliaryEntries).unwrap();
        entries.try_push(&mut control, 7).unwrap();
        assert_eq!(
            entries.get_mut(0, &mut control),
            Err(ParseStop::LimitExceeded {
                resource: HtmlParseResource::WorkCredits,
                limit: 6,
                current: 6,
                requested_delta: 1,
            })
        );
        assert_eq!(control.usage().work(WorkCategory::RecordVisit), 0);
        assert_eq!(entries.as_slice(), &[7]);
        entries.release(&mut control).unwrap();
    }
    #[test]
    fn normal_pop_empty_attempt_visits_without_write_copy_or_allocation() {
        let mut control = control();
        let mut entries = OwnedVec::<u16>::new(HtmlParseResource::AuxiliaryEntries).unwrap();
        assert_eq!(entries.try_pop(&mut control).unwrap(), None);
        assert_eq!(
            control
                .usage()
                .resource(HtmlParseResource::WorkCredits)
                .current,
            1
        );
        assert_eq!(control.usage().work(WorkCategory::RecordVisit), 1);
        assert_eq!(control.usage().work(WorkCategory::RecordWrite), 0);
        assert_eq!(control.usage().work(WorkCategory::CopyByte), 0);
        assert_eq!(control.usage().actual_capacity_bytes, 0);
    }
    #[test]
    fn normal_pop_transfers_last_slot_and_retains_capacity_and_high_water() {
        let mut control = control();
        let mut entries = OwnedVec::<u16>::new(HtmlParseResource::AuxiliaryEntries).unwrap();
        entries
            .try_extend_from_slice(&mut control, &[3, 7])
            .unwrap();
        let before = control.usage().clone();
        let capacity = entries.capacity();
        assert_eq!(entries.try_pop(&mut control).unwrap(), Some(7));
        assert_eq!(entries.as_slice(), &[3]);
        assert_eq!(entries.capacity(), capacity);
        assert_eq!(
            control
                .usage()
                .resource(HtmlParseResource::AuxiliaryEntries),
            ResourceUsage {
                current: 1,
                high_water: 2
            }
        );
        assert_eq!(
            control
                .usage()
                .resource(HtmlParseResource::WorkCredits)
                .current,
            before.resource(HtmlParseResource::WorkCredits).current + 4
        );
        assert_eq!(
            control.usage().work(WorkCategory::RecordVisit),
            before.work(WorkCategory::RecordVisit) + 1
        );
        assert_eq!(
            control.usage().work(WorkCategory::RecordWrite),
            before.work(WorkCategory::RecordWrite) + 1
        );
        assert_eq!(
            control.usage().work(WorkCategory::CopyByte),
            before.work(WorkCategory::CopyByte) + 2
        );
        assert_eq!(
            control.usage().actual_capacity_bytes,
            before.actual_capacity_bytes
        );
        entries.release(&mut control).unwrap();
        assert_eq!(control.usage().actual_capacity_bytes, 0);
        assert_eq!(
            control
                .usage()
                .resource(HtmlParseResource::AuxiliaryEntries)
                .high_water,
            2
        );
    }
    #[test]
    fn normal_pop_foreign_owner_and_work_cap_leave_the_value_in_place() {
        let mut limits = limits();
        limits = limits.into_builder().work_credits(6).finish().unwrap();
        let mut owner = ParseControl::new(limits, HtmlParseCancellation::new());
        let mut foreign = control();
        let mut entries = OwnedVec::<u16>::new(HtmlParseResource::AuxiliaryEntries).unwrap();
        entries.try_push(&mut owner, 7).unwrap();
        assert_eq!(
            entries.try_pop(&mut foreign),
            Err(ParseStop::InternalInvariant(Invariant::ForeignControl))
        );
        assert_eq!(
            foreign
                .usage()
                .resource(HtmlParseResource::WorkCredits)
                .current,
            0
        );
        assert_eq!(
            entries.try_pop(&mut owner),
            Err(ParseStop::LimitExceeded {
                resource: HtmlParseResource::WorkCredits,
                limit: 6,
                current: 6,
                requested_delta: 4,
            })
        );
        assert_eq!(entries.as_slice(), &[7]);
        assert_eq!(
            owner
                .usage()
                .resource(HtmlParseResource::AuxiliaryEntries)
                .current,
            1
        );
        entries.release(&mut owner).unwrap();
    }
    #[test]
    fn normal_pop_large_fixed_record_preserves_its_admitted_atomic_layout() {
        let mut control = control();
        let mut entries = OwnedVec::<[u8; 512]>::new(HtmlParseResource::AuxiliaryEntries).unwrap();
        entries.try_push(&mut control, [7; 512]).unwrap();
        let before = control
            .usage()
            .resource(HtmlParseResource::WorkCredits)
            .current;
        assert_eq!(entries.try_pop(&mut control).unwrap(), Some([7; 512]));
        assert_eq!(
            control
                .usage()
                .resource(HtmlParseResource::WorkCredits)
                .current,
            before + 514
        );
        assert_eq!(control.usage().largest_atomic_layout, 512);
        assert_eq!(
            control
                .usage()
                .resource(HtmlParseResource::AtomicLayoutBytes)
                .current,
            512
        );
        assert_eq!(entries.capacity(), 1);
        entries.release(&mut control).unwrap();
    }
    #[test]
    fn active_cleanup_pop_is_rejected_before_removal() {
        let mut control = control();
        let mut entries = OwnedVec::<u16>::new(HtmlParseResource::AuxiliaryEntries).unwrap();
        entries.try_push(&mut control, 7).unwrap();
        let before = control.usage().clone();
        assert_eq!(
            entries.cleanup_pop(&mut control),
            Err(ParseStop::InternalInvariant(Invariant::CleanupWhileRunning))
        );
        assert_eq!(entries.as_slice(), &[7]);
        assert_eq!(control.usage(), &before);
        assert_eq!(
            entries.try_pop(&mut control),
            Err(ParseStop::InternalInvariant(Invariant::CleanupWhileRunning))
        );
        entries.release(&mut control).unwrap();
    }
    #[test]
    fn terminal_cleanup_transfers_nested_tickets_without_work_refund_or_resumption() {
        let mut control = control();
        let mut text = OwnedVec::<u16>::new(HtmlParseResource::BufferedUtf16Units).unwrap();
        text.try_push(&mut control, 0xD800).unwrap();
        let (text, allocation) = control.admit_document_string_wrapper(text).unwrap();
        let text = DocumentString::from_utf16(text);
        let mut entries = OwnedVec::new(HtmlParseResource::TokenAttributes).unwrap();
        entries.try_push(&mut control, (text, allocation)).unwrap();
        control.cancellation.cancel();
        let stop = control.checkpoint().unwrap_err();
        let before = control.usage().clone();
        let (text, allocation) = entries.cleanup_pop(&mut control).unwrap().unwrap();
        assert_eq!(text.code_units(), &[0xD800]);
        assert_eq!(
            control.usage().resource(HtmlParseResource::TokenAttributes),
            ResourceUsage {
                current: 0,
                high_water: 1
            }
        );
        assert_eq!(
            control
                .usage()
                .resource(HtmlParseResource::BufferedUtf16Units)
                .current,
            1
        );
        assert_eq!(
            control.usage().resource(HtmlParseResource::WorkCredits),
            before.resource(HtmlParseResource::WorkCredits)
        );
        assert_eq!(control.usage().work, before.work);
        assert_eq!(
            control.usage().actual_capacity_bytes,
            before.actual_capacity_bytes
        );
        drop(text);
        control.release_document_string(allocation).unwrap();
        assert!(entries.cleanup_pop(&mut control).unwrap().is_none());
        assert!(matches!(entries.get_mut(0, &mut control), Err(reason) if reason == stop));
        assert!(matches!(entries.try_pop(&mut control), Err(reason) if reason == stop));
        entries.release(&mut control).unwrap();
        assert_eq!(control.usage().actual_capacity_bytes, 0);
        assert_eq!(control.stop(), Some(stop));
        assert_eq!(
            control.usage().resource(HtmlParseResource::WorkCredits),
            before.resource(HtmlParseResource::WorkCredits)
        );
    }
    #[test]
    fn cleanup_pop_rejects_foreign_owner_without_changing_the_first_stop() {
        let mut owner = control();
        let mut foreign = control();
        let mut entries = OwnedVec::<u16>::new(HtmlParseResource::AuxiliaryEntries).unwrap();
        entries.try_push(&mut owner, 7).unwrap();
        foreign.cancellation.cancel();
        let stop = foreign.checkpoint().unwrap_err();
        let before = foreign.usage().clone();
        assert_eq!(entries.cleanup_pop(&mut foreign), Err(stop));
        assert_eq!(entries.as_slice(), &[7]);
        assert_eq!(foreign.usage(), &before);
        assert_eq!(foreign.stop(), Some(stop));
        owner.cancellation.cancel();
        owner.checkpoint().unwrap_err();
        assert_eq!(entries.cleanup_pop(&mut owner).unwrap(), Some(7));
        entries.release(&mut owner).unwrap();
    }
    #[test]
    fn recovering_push_has_the_same_success_accounting_as_normal_push() {
        let mut normal = control();
        let mut recovering = control();
        let mut normal_entries = OwnedVec::<u16>::new(HtmlParseResource::AuxiliaryEntries).unwrap();
        let mut recovering_entries =
            OwnedVec::<u16>::new(HtmlParseResource::AuxiliaryEntries).unwrap();
        normal_entries.try_push(&mut normal, 7).unwrap();
        assert!(
            recovering_entries
                .try_push_recover(&mut recovering, 7)
                .is_ok()
        );
        assert_eq!(normal_entries.as_slice(), recovering_entries.as_slice());
        assert_eq!(normal.usage(), recovering.usage());
        normal_entries.release(&mut normal).unwrap();
        recovering_entries.release(&mut recovering).unwrap();
    }
    #[test]
    fn recovering_push_returns_nested_tickets_on_slot_cap_failure() {
        let mut limits = limits();
        limits = limits.into_builder().token_attributes(0).finish().unwrap();
        let mut control = ParseControl::new(limits, HtmlParseCancellation::new());
        let mut units = OwnedVec::<u16>::new(HtmlParseResource::BufferedUtf16Units).unwrap();
        units.try_push(&mut control, 0xD800).unwrap();
        let (units, allocation) = control.admit_document_string_wrapper(units).unwrap();
        let text = DocumentString::from_utf16(units);
        let mut entries = OwnedVec::new(HtmlParseResource::TokenAttributes).unwrap();
        let (stop, (text, allocation)) = entries
            .try_push_recover(&mut control, (text, allocation))
            .unwrap_err();
        assert_eq!(
            stop,
            ParseStop::LimitExceeded {
                resource: HtmlParseResource::TokenAttributes,
                limit: 0,
                current: 0,
                requested_delta: 1
            }
        );
        assert_eq!(text.code_units(), &[0xD800]);
        assert_eq!(entries.len(), 0);
        assert_eq!(entries.capacity(), 0);
        assert_eq!(
            control
                .usage()
                .resource(HtmlParseResource::BufferedUtf16Units)
                .current,
            1
        );
        let work = control
            .usage()
            .resource(HtmlParseResource::WorkCredits)
            .current;
        drop(text);
        control.release_document_string(allocation).unwrap();
        entries.release(&mut control).unwrap();
        assert_eq!(control.usage().actual_capacity_bytes, 0);
        assert_eq!(control.stop(), Some(stop));
        assert_eq!(
            control
                .usage()
                .resource(HtmlParseResource::WorkCredits)
                .current,
            work
        );
    }
    #[test]
    fn recovering_push_returns_the_value_on_cancel_foreign_owner_and_work_failure() {
        let mut owner = control();
        let mut foreign = control();
        let mut entries = OwnedVec::<u16>::new(HtmlParseResource::AuxiliaryEntries).unwrap();
        entries.try_push(&mut owner, 7).unwrap();
        assert_eq!(
            entries.try_push_recover(&mut foreign, 11),
            Err((ParseStop::InternalInvariant(Invariant::ForeignControl), 11))
        );
        owner.cancellation.cancel();
        assert_eq!(
            entries.try_push_recover(&mut owner, 13),
            Err((ParseStop::Cancelled, 13))
        );
        assert_eq!(entries.as_slice(), &[7]);
        entries.release(&mut owner).unwrap();
        let mut limits = limits();
        limits = limits.into_builder().work_credits(0).finish().unwrap();
        let mut control = ParseControl::new(limits, HtmlParseCancellation::new());
        let mut entries = OwnedVec::<u16>::new(HtmlParseResource::AuxiliaryEntries).unwrap();
        assert_eq!(
            entries.try_push_recover(&mut control, 17),
            Err((
                ParseStop::LimitExceeded {
                    resource: HtmlParseResource::WorkCredits,
                    limit: 0,
                    current: 0,
                    requested_delta: 6
                },
                17
            ))
        );
        assert_eq!(entries.len(), 0);
        assert_eq!(entries.capacity(), 0);
        assert_eq!(
            control
                .usage()
                .resource(HtmlParseResource::WorkCredits)
                .current,
            0
        );
        entries.release(&mut control).unwrap();
    }
    #[test]
    fn normal_pop_transfers_nested_tickets_without_releasing_their_payload() {
        let mut control = control();
        let mut units = OwnedVec::<u16>::new(HtmlParseResource::BufferedUtf16Units).unwrap();
        units.try_push(&mut control, 0xD800).unwrap();
        let (units, allocation) = control.admit_document_string_wrapper(units).unwrap();
        let mut entries = OwnedVec::new(HtmlParseResource::TokenAttributes).unwrap();
        entries
            .try_push(
                &mut control,
                (DocumentString::from_utf16(units), allocation),
            )
            .unwrap();
        let bytes = control.usage().actual_capacity_bytes;
        let (text, allocation) = entries.try_pop(&mut control).unwrap().unwrap();
        assert_eq!(text.code_units(), &[0xD800]);
        assert_eq!(
            control
                .usage()
                .resource(HtmlParseResource::TokenAttributes)
                .current,
            0
        );
        assert_eq!(
            control
                .usage()
                .resource(HtmlParseResource::BufferedUtf16Units)
                .current,
            1
        );
        assert_eq!(control.usage().actual_capacity_bytes, bytes);
        drop(text);
        control.release_document_string(allocation).unwrap();
        entries.release(&mut control).unwrap();
        assert_eq!(control.usage().actual_capacity_bytes, 0);
    }
    #[test]
    fn actual_vector_reservation_tracks_growth_overlap_and_within_capacity_reuse() {
        let mut control = control();
        let (mut vector, mut ticket) = control.new_vector::<u32>().unwrap();
        assert_eq!(ticket.capacity(), 0);
        assert_eq!(vector.capacity(), 0);
        control
            .check_acquire_many(&[(HtmlParseResource::ManagedNodes, 2)])
            .unwrap();
        assert_eq!(
            control
                .usage()
                .resource(HtmlParseResource::ManagedNodes)
                .current,
            0
        );
        control.reserve_vector(&mut vector, &mut ticket, 2).unwrap();
        assert_eq!(vector.capacity(), 2);
        assert_eq!(ticket.capacity(), 2);
        assert_eq!(control.usage().actual_capacity_bytes, 8);
        control
            .acquire_many(&[(HtmlParseResource::ManagedNodes, 2)])
            .unwrap();
        control
            .charge_many(&[(WorkCategory::RecordWrite, 2), (WorkCategory::CopyByte, 8)])
            .unwrap();
        vector.push(10);
        vector.push(20);
        control.reserve_vector(&mut vector, &mut ticket, 1).unwrap();
        assert_eq!(ticket.capacity(), 3);
        assert_eq!(vector.capacity(), 3);
        assert_eq!(vector, [10, 20]);
        assert_eq!(control.usage().actual_capacity_bytes, 12);
        assert_eq!(
            control
                .usage()
                .resource(HtmlParseResource::PeakTransientBytes)
                .high_water,
            20
        );
        assert_eq!(control.usage().work(WorkCategory::ReserveByte), 28);
        let calls = control.usage().work(WorkCategory::ProviderCall);
        control.reserve_vector(&mut vector, &mut ticket, 1).unwrap();
        assert_eq!(control.usage().work(WorkCategory::ProviderCall), calls);
        control.cancellation.cancel();
        let stop = control.checkpoint().unwrap_err();
        let work = control.usage().resource(HtmlParseResource::WorkCredits);
        control.release_vector(vector, ticket).unwrap();
        assert_eq!(control.usage().actual_capacity_bytes, 0);
        assert_eq!(
            control.usage().resource(HtmlParseResource::ManagedNodes),
            ResourceUsage {
                current: 2,
                high_water: 2
            }
        );
        assert_eq!(
            control.usage().resource(HtmlParseResource::WorkCredits),
            work
        );
        assert_eq!(control.stop(), Some(stop));
    }
    #[test]
    fn actual_vector_ticket_rejects_foreign_owner_and_recovers_owned_storage() {
        let mut owner = control();
        let mut foreign = control();
        let (mut vector, mut ticket) = owner.new_vector::<u32>().unwrap();
        owner.reserve_vector(&mut vector, &mut ticket, 2).unwrap();
        let before = foreign.usage().clone();
        assert_eq!(
            foreign.reserve_vector(&mut vector, &mut ticket, 3),
            Err(ParseStop::InternalInvariant(Invariant::ForeignControl))
        );
        assert_eq!(foreign.usage(), &before);
        assert_eq!(ticket.capacity(), 2);
        let (stop, vector, ticket) = foreign
            .release_vector(vector, ticket)
            .unwrap_err()
            .into_parts();
        assert_eq!(
            stop,
            ParseStop::InternalInvariant(Invariant::ForeignControl)
        );
        assert_eq!(vector.capacity(), 2);
        assert_eq!(ticket.capacity(), 2);
        owner.release_vector(vector, ticket).unwrap();
        assert_eq!(owner.usage().actual_capacity_bytes, 0);
    }
    #[test]
    fn actual_vector_ticket_checks_exact_capacity_before_growth_and_cleanup() {
        let mut control = control();
        let (mut first, mut first_ticket) = control.new_vector::<u32>().unwrap();
        let (mut second, mut second_ticket) = control.new_vector::<u32>().unwrap();
        control
            .reserve_vector(&mut first, &mut first_ticket, 1)
            .unwrap();
        control
            .reserve_vector(&mut second, &mut second_ticket, 2)
            .unwrap();
        let calls = control.usage().work(WorkCategory::ProviderCall);
        assert_eq!(
            control.reserve_vector(&mut second, &mut first_ticket, 3),
            Err(ParseStop::InternalInvariant(Invariant::CapacityMismatch))
        );
        assert_eq!(control.usage().work(WorkCategory::ProviderCall), calls);
        assert_eq!(second.capacity(), 2);
        assert_eq!(first_ticket.capacity(), 1);
        let (_, second, first_ticket) = control
            .release_vector(second, first_ticket)
            .unwrap_err()
            .into_parts();
        assert_eq!(control.usage().actual_capacity_bytes, 12);
        control.release_vector(first, first_ticket).unwrap();
        control.release_vector(second, second_ticket).unwrap();
        assert_eq!(control.usage().actual_capacity_bytes, 0);
    }
    #[test]
    fn actual_vector_cancel_and_capacity_failure_do_not_grow_or_admit_nodes() {
        let mut limits = limits();
        limits = limits
            .into_builder()
            .managed_capacity_bytes(3)
            .finish()
            .unwrap();
        let mut control = ParseControl::new(limits, HtmlParseCancellation::new());
        let (mut vector, mut ticket) = control.new_vector::<u32>().unwrap();
        assert_eq!(
            control.reserve_vector(&mut vector, &mut ticket, 1),
            Err(ParseStop::LimitExceeded {
                resource: HtmlParseResource::ManagedCapacityBytes,
                limit: 3,
                current: 0,
                requested_delta: 4,
            })
        );
        assert_eq!(ticket.capacity(), 0);
        assert_eq!(vector.capacity(), 0);
        assert_eq!(
            control
                .usage()
                .resource(HtmlParseResource::ManagedNodes)
                .current,
            0
        );
        control.release_vector(vector, ticket).unwrap();
        let mut control = super::tests::control();
        let (mut vector, mut ticket) = control.new_vector::<u32>().unwrap();
        control.cancellation.cancel();
        let before = control.usage().clone();
        assert_eq!(
            control.reserve_vector(&mut vector, &mut ticket, 1),
            Err(ParseStop::Cancelled)
        );
        assert_eq!(control.usage(), &before);
        assert_eq!(ticket.capacity(), 0);
        control.release_vector(vector, ticket).unwrap();
    }
    #[test]
    fn actual_allocation_failure_preserves_zero_capacity_ticket_for_cleanup() {
        let mut limits = limits();
        limits = limits
            .into_builder()
            .managed_capacity_bytes(usize::MAX as u64)
            .finish()
            .unwrap();
        limits = limits
            .into_builder()
            .peak_transient_bytes(usize::MAX as u64)
            .finish()
            .unwrap();
        limits = limits
            .into_builder()
            .atomic_layout_bytes(isize::MAX as u64)
            .finish()
            .unwrap();
        limits = limits
            .into_builder()
            .work_credits(u64::MAX)
            .finish()
            .unwrap();
        let mut control = ParseControl::new(limits, HtmlParseCancellation::new());
        let (mut vector, mut ticket) = control.new_vector::<u8>().unwrap();
        assert_eq!(
            control.reserve_vector(&mut vector, &mut ticket, isize::MAX as usize),
            Err(ParseStop::AllocationFailed {
                resource: HtmlParseResource::ManagedCapacityBytes,
                layout_bytes: isize::MAX as u64,
            })
        );
        assert_eq!(ticket.capacity(), 0);
        assert_eq!(vector.capacity(), 0);
        control.release_vector(vector, ticket).unwrap();
        assert_eq!(control.usage().actual_capacity_bytes, 0);
    }
    #[test]
    fn ticket_refresh_records_actual_growth_even_when_post_reserve_phase_stops() {
        let mut control = control();
        let (mut vector, mut ticket) = control.new_vector::<u32>().unwrap();
        // Execute the real reservation phase, then deterministically cancel at
        // its post-call boundary before the production ticket-refresh phase.
        let plan = ReservationPlan::for_vec(&vector, 2).unwrap().unwrap();
        control
            .charge_many(&[
                (WorkCategory::ReserveByte, 8),
                (WorkCategory::ProviderCall, 1),
            ])
            .unwrap();
        control.reserve(&mut vector, plan).unwrap();
        control.cancellation.cancel();
        let stop = control.checkpoint().unwrap_err();
        assert_eq!(
            ParseControl::record_vector_capacity(&vector, &mut ticket, Err(stop)),
            Err(stop)
        );
        assert_eq!(ticket.capacity(), 2);
        assert_eq!(control.usage().actual_capacity_bytes, 8);
        control.release_vector(vector, ticket).unwrap();
        assert_eq!(control.usage().actual_capacity_bytes, 0);
        assert_eq!(control.stop(), Some(stop));
    }
    #[test]
    fn logical_preflight_does_not_commit_and_final_admission_rechecks_the_boundary() {
        let mut limits = limits();
        limits = limits.into_builder().managed_nodes(1).finish().unwrap();
        let mut control = ParseControl::new(limits, HtmlParseCancellation::new());
        control
            .check_acquire_many(&[
                (HtmlParseResource::ManagedNodes, 1),
                (HtmlParseResource::ManagedEdges, 1),
            ])
            .unwrap();
        assert_eq!(
            control
                .usage()
                .resource(HtmlParseResource::ManagedNodes)
                .current,
            0
        );
        control.acquire(HtmlParseResource::ManagedNodes, 1).unwrap();
        assert!(matches!(
            control.acquire_many(&[
                (HtmlParseResource::ManagedNodes, 1),
                (HtmlParseResource::ManagedEdges, 1)
            ]),
            Err(ParseStop::LimitExceeded {
                resource: HtmlParseResource::ManagedNodes,
                ..
            })
        ));
        assert_eq!(
            control
                .usage()
                .resource(HtmlParseResource::ManagedEdges)
                .current,
            0
        );
    }
    #[test]
    fn fixed_shared_payload_moves_actual_vector_and_tickets_without_copying_backing() {
        let mut control = control();
        let (mut vector, mut ticket) = control.new_vector::<u32>().unwrap();
        control.reserve_vector(&mut vector, &mut ticket, 2).unwrap();
        control.acquire(HtmlParseResource::ManagedNodes, 2).unwrap();
        control
            .charge_many(&[(WorkCategory::CopyByte, 8), (WorkCategory::RecordWrite, 2)])
            .unwrap();
        vector.push(10);
        vector.push(20);
        let pointer = vector.as_ptr();
        let (shared, allocation) = control.share_owned(vector).unwrap();
        assert_eq!(shared.as_ptr(), pointer);
        assert_eq!(shared.as_slice(), &[10, 20]);
        assert_eq!(allocation.payload_bytes(), size_of::<Vec<u32>>() as u64);
        assert_eq!(
            control.usage().actual_capacity_bytes,
            8 + size_of::<Vec<u32>>() as u64
        );
        control.cancellation.cancel();
        let stop = control.checkpoint().unwrap_err();
        let vector = control.release_shared(shared, allocation).unwrap();
        assert_eq!(vector.as_ptr(), pointer);
        assert_eq!(vector, [10, 20]);
        control.release_vector(vector, ticket).unwrap();
        assert_eq!(control.usage().actual_capacity_bytes, 0);
        assert_eq!(
            control
                .usage()
                .resource(HtmlParseResource::ManagedNodes)
                .current,
            2
        );
        assert_eq!(control.stop(), Some(stop));
    }
    #[test]
    fn zero_sized_shared_identity_is_real_but_headers_are_not_fabricated_capacity() {
        let mut control = control();
        let (shared, ticket) = control.share_owned(()).unwrap();
        assert_eq!(Arc::strong_count(&shared), 1);
        assert_eq!(ticket.payload_bytes(), 0);
        assert_eq!(control.usage().actual_capacity_bytes, 0);
        assert_eq!(control.usage().work(WorkCategory::ProviderCall), 1);
        assert_eq!(control.release_shared(shared, ticket).unwrap(), ());
        assert!(matches!(
            control.new_vector::<()>(),
            Err(ParseStop::InternalInvariant(Invariant::ZeroSizedStorage))
        ));
    }
    #[test]
    fn shared_admission_cap_or_cancel_failure_returns_the_original_payload() {
        let mut limits = limits();
        limits = limits
            .into_builder()
            .managed_capacity_bytes(3)
            .finish()
            .unwrap();
        let mut control = ParseControl::new(limits, HtmlParseCancellation::new());
        assert_eq!(
            control.share_owned(77_u32).unwrap_err(),
            (
                ParseStop::LimitExceeded {
                    resource: HtmlParseResource::ManagedCapacityBytes,
                    limit: 3,
                    current: 0,
                    requested_delta: 4,
                },
                77
            )
        );
        assert_eq!(control.usage().actual_capacity_bytes, 0);
        let mut control = super::tests::control();
        control.cancellation.cancel();
        assert_eq!(
            control.share_owned(91_u32).unwrap_err(),
            (ParseStop::Cancelled, 91)
        );
        assert_eq!(
            control
                .usage()
                .resource(HtmlParseResource::WorkCredits)
                .current,
            0
        );
    }
    #[test]
    fn shared_post_construction_checkpoint_recovers_actual_owned_payload() {
        let mut control = control();
        let (shared, ticket) = control.share_owned(77_u32).unwrap();
        let work = control.usage().resource(HtmlParseResource::WorkCredits);
        control.cancellation.cancel();
        assert_eq!(
            control.finish_shared(shared, ticket).unwrap_err(),
            (ParseStop::Cancelled, 77)
        );
        assert_eq!(control.usage().actual_capacity_bytes, 0);
        assert_eq!(control.usage().actual_capacity_high_water, 4);
        assert_eq!(
            control.usage().resource(HtmlParseResource::WorkCredits),
            work
        );
    }
    #[test]
    fn shared_foreign_alias_and_wrong_payload_rejections_preserve_actual_ownership() {
        let mut owner = control();
        let mut foreign = control();
        let (shared, ticket) = owner.share_owned(77_u32).unwrap();
        let (stop, shared, ticket) = foreign
            .release_shared(shared, ticket)
            .unwrap_err()
            .into_parts();
        assert_eq!(
            stop,
            ParseStop::InternalInvariant(Invariant::ForeignControl)
        );
        assert_eq!(owner.usage().actual_capacity_bytes, 4);
        let other_handle = shared.clone();
        let (stop, shared, ticket) = owner
            .release_shared(shared, ticket)
            .unwrap_err()
            .into_parts();
        assert_eq!(
            stop,
            ParseStop::InternalInvariant(Invariant::SharedAllocationAliased)
        );
        assert_eq!(owner.usage().actual_capacity_bytes, 4);
        drop(other_handle);
        assert_eq!(owner.release_shared(shared, ticket).unwrap(), 77);
        let mut control = control();
        let (first, first_ticket) = control.share_owned(10_u32).unwrap();
        let (second, second_ticket) = control.share_owned(20_u32).unwrap();
        let (_, second, first_ticket) = control
            .release_shared(second, first_ticket)
            .unwrap_err()
            .into_parts();
        assert_eq!(control.usage().actual_capacity_bytes, 8);
        assert_eq!(control.release_shared(first, first_ticket).unwrap(), 10);
        assert_eq!(control.release_shared(second, second_ticket).unwrap(), 20);
        assert_eq!(control.usage().actual_capacity_bytes, 0);
    }
    #[test]
    fn terminal_logical_cleanup_requires_same_owner_and_never_refunds_nodes_or_work() {
        let mut control = control();
        let identity = control.identity();
        let foreign_identity = super::tests::control().identity();
        control
            .acquire_many(&[
                (HtmlParseResource::ManagedNodes, 2),
                (HtmlParseResource::AuxiliaryEntries, 3),
                (HtmlParseResource::ManagedEdges, 1),
            ])
            .unwrap();
        control.charge(WorkCategory::StateStep, 5).unwrap();
        control.cancellation.cancel();
        let stop = control.checkpoint().unwrap_err();
        let before = control.usage().clone();
        assert_eq!(
            control.cleanup_release(&foreign_identity, HtmlParseResource::AuxiliaryEntries, 1),
            Err(stop)
        );
        assert_eq!(
            control.cleanup_release(&identity, HtmlParseResource::AuxiliaryEntries, 4),
            Err(stop)
        );
        assert_eq!(
            control.cleanup_release(&identity, HtmlParseResource::ManagedNodes, 1),
            Err(stop)
        );
        assert_eq!(
            control.cleanup_release(&identity, HtmlParseResource::WorkCredits, 1),
            Err(stop)
        );
        assert_eq!(control.usage(), &before);
        control
            .cleanup_release(&identity, HtmlParseResource::AuxiliaryEntries, 3)
            .unwrap();
        control
            .cleanup_release(&identity, HtmlParseResource::ManagedEdges, 1)
            .unwrap();
        assert_eq!(
            control
                .usage()
                .resource(HtmlParseResource::AuxiliaryEntries),
            ResourceUsage {
                current: 0,
                high_water: 3
            }
        );
        assert_eq!(
            control.usage().resource(HtmlParseResource::ManagedEdges),
            ResourceUsage {
                current: 0,
                high_water: 1
            }
        );
        assert_eq!(
            control.usage().resource(HtmlParseResource::ManagedNodes),
            before.resource(HtmlParseResource::ManagedNodes)
        );
        assert_eq!(
            control.usage().resource(HtmlParseResource::WorkCredits),
            before.resource(HtmlParseResource::WorkCredits)
        );
        assert_eq!(control.stop(), Some(stop));
    }
    #[test]
    fn active_logical_cleanup_is_rejected_without_changing_admitted_counts() {
        let mut control = control();
        let identity = control.identity();
        control
            .acquire(HtmlParseResource::AuxiliaryEntries, 1)
            .unwrap();
        let before = control.usage().clone();
        assert_eq!(
            control.cleanup_release(&identity, HtmlParseResource::AuxiliaryEntries, 1),
            Err(ParseStop::InternalInvariant(Invariant::CleanupWhileRunning))
        );
        assert_eq!(control.usage(), &before);
        control
            .cleanup_release(&identity, HtmlParseResource::AuxiliaryEntries, 1)
            .unwrap();
    }
    #[test]
    fn fixed_record_layout_observation_is_separate_from_work_and_storage() {
        let mut control = control();
        let before = control.usage().clone();
        control.admit_atomic_layout(512).unwrap();
        control.admit_atomic_layout(128).unwrap();
        assert_eq!(
            control
                .usage()
                .resource(HtmlParseResource::AtomicLayoutBytes),
            ResourceUsage {
                current: 128,
                high_water: 512,
            }
        );
        assert_eq!(control.usage().largest_atomic_layout, 512);
        assert_eq!(control.usage().work, before.work);
        assert_eq!(control.usage().actual_capacity_bytes, 0);
        assert_eq!(control.usage().actual_capacity_high_water, 0);
        for resource in RESOURCES {
            if resource != HtmlParseResource::AtomicLayoutBytes {
                assert_eq!(
                    control.usage().resource(resource),
                    before.resource(resource)
                );
            }
        }
    }
    #[test]
    fn fixed_record_layout_rejection_and_cancellation_observe_no_layout() {
        let mut caps = limits();
        caps = caps
            .into_builder()
            .atomic_layout_bytes(511)
            .finish()
            .unwrap();
        let mut owner = ParseControl::new(caps, HtmlParseCancellation::new());
        let before = owner.usage().clone();
        let stop = owner.admit_atomic_layout(512).unwrap_err();
        assert_eq!(
            stop,
            ParseStop::LimitExceeded {
                resource: HtmlParseResource::AtomicLayoutBytes,
                limit: 511,
                current: 0,
                requested_delta: 512,
            }
        );
        owner.cancellation.cancel();
        assert_eq!(owner.admit_atomic_layout(0), Err(stop));
        assert_eq!(owner.usage(), &before);

        let mut owner = control();
        owner.cancellation.cancel();
        let before = owner.usage().clone();
        assert_eq!(owner.admit_atomic_layout(1), Err(ParseStop::Cancelled));
        assert_eq!(owner.usage(), &before);
    }
    #[test]
    fn fixed_record_layout_checks_representable_span_and_zero_layout() {
        let mut owner = control();
        owner.admit_atomic_layout(0).unwrap();
        let before = owner.usage().clone();
        assert_eq!(
            owner.admit_atomic_layout(usize::MAX),
            Err(ParseStop::RepresentationExhausted {
                resource: HtmlParseResource::AtomicLayoutBytes,
            })
        );
        assert_eq!(owner.usage(), &before);
    }
    #[test]
    fn checked_arithmetic_failure_latches_observation_and_preserves_first_stop() {
        let mut owner = control();
        owner.checkpoint().unwrap();
        let before = owner.usage().clone();
        // HtmlParseCancellation arrives after the arithmetic operation's initial poll.
        // The already observed failure remains the first terminal transition.
        owner.cancellation.cancel();
        let stop = ParseStop::RepresentationExhausted {
            resource: HtmlParseResource::ManagedUtf16Units,
        };
        assert_eq!(
            owner.representation::<()>(HtmlParseResource::ManagedUtf16Units),
            Err(stop)
        );
        assert_eq!(owner.checkpoint(), Err(stop));
        assert_eq!(
            owner.representation::<()>(HtmlParseResource::ManagedEdges),
            Err(stop)
        );
        assert_eq!(owner.usage(), &before);

        let mut owner = control();
        owner.cancellation.cancel();
        assert_eq!(owner.checkpoint(), Err(ParseStop::Cancelled));
        assert_eq!(
            owner.representation::<()>(HtmlParseResource::ManagedUtf16Units),
            Err(ParseStop::Cancelled)
        );
    }
    #[test]
    fn zero_sized_shared_cleanup_allows_actual_alias_without_capacity_refund() {
        let mut owner = control();
        let (shared, allocation) = owner.share_owned(()).unwrap();
        let alias = shared.clone();
        assert_eq!(Arc::strong_count(&alias), 2);
        owner.cancellation.cancel();
        let stop = owner.checkpoint().unwrap_err();
        let before = owner.usage().clone();
        owner.release_zero_sized_shared(shared, allocation).unwrap();
        assert_eq!(Arc::strong_count(&alias), 1);
        assert_eq!(owner.usage(), &before);
        assert_eq!(owner.stop(), Some(stop));
    }
    #[test]
    fn zero_sized_shared_cleanup_recovers_foreign_or_mismatched_actual_ownership() {
        let mut owner = control();
        let mut foreign = control();
        let (first, first_ticket) = owner.share_owned(()).unwrap();
        let (stop, first, first_ticket) = foreign
            .release_zero_sized_shared(first, first_ticket)
            .unwrap_err()
            .into_parts();
        assert_eq!(
            stop,
            ParseStop::InternalInvariant(Invariant::ForeignControl)
        );
        assert_eq!(owner.stop(), None);
        let (second, second_ticket) = owner.share_owned(()).unwrap();
        let before = owner.usage().clone();
        let (stop, second, first_ticket) = owner
            .release_zero_sized_shared(second, first_ticket)
            .unwrap_err()
            .into_parts();
        assert_eq!(
            stop,
            ParseStop::InternalInvariant(Invariant::CapacityMismatch)
        );
        assert_eq!(owner.usage(), &before);
        owner
            .release_zero_sized_shared(first, first_ticket)
            .unwrap();
        owner
            .release_zero_sized_shared(second, second_ticket)
            .unwrap();
        assert_eq!(owner.usage(), &before);
    }
    #[test]
    fn zero_sized_cleanup_rejects_nonzero_payload_without_refunding_its_layout() {
        let mut owner = control();
        let (shared, allocation) = owner.share_owned(77_u32).unwrap();
        let before = owner.usage().clone();
        let (stop, shared, allocation) = owner
            .release_zero_sized_shared(shared, allocation)
            .unwrap_err()
            .into_parts();
        assert_eq!(
            stop,
            ParseStop::InternalInvariant(Invariant::CapacityMismatch)
        );
        assert_eq!(owner.usage(), &before);
        assert_eq!(owner.release_shared(shared, allocation).unwrap(), 77);
        assert_eq!(owner.usage().actual_capacity_bytes, 0);
        assert_eq!(owner.usage().actual_capacity_high_water, 4);
        assert_eq!(owner.usage().work, before.work);
        assert_eq!(owner.stop(), Some(stop));
    }
    #[test]
    fn child_order_commit_transfers_scratch_and_edge_delta_without_other_refunds() {
        let mut owner = control();
        let identity = owner.identity();
        owner
            .acquire_many(&[
                (HtmlParseResource::AuxiliaryEntries, 3),
                (HtmlParseResource::ManagedEdges, 2),
                (HtmlParseResource::ManagedNodes, 4),
            ])
            .unwrap();
        owner
            .charge_many(&[(WorkCategory::RecordWrite, 3), (WorkCategory::CopyByte, 24)])
            .unwrap();
        let before = owner.usage().clone();
        owner.commit_child_orders(&identity, 3, 2, 1).unwrap();
        assert_eq!(
            owner.usage().resource(HtmlParseResource::AuxiliaryEntries),
            ResourceUsage {
                current: 0,
                high_water: 3,
            }
        );
        assert_eq!(
            owner.usage().resource(HtmlParseResource::ManagedEdges),
            ResourceUsage {
                current: 3,
                high_water: 3,
            }
        );
        for resource in RESOURCES {
            if resource != HtmlParseResource::AuxiliaryEntries
                && resource != HtmlParseResource::ManagedEdges
            {
                assert_eq!(owner.usage().resource(resource), before.resource(resource));
            }
        }
        assert_eq!(owner.usage().work, before.work);
        assert_eq!(owner.stop(), None);
    }
    #[test]
    fn child_order_same_parent_move_can_commit_at_edge_cap_without_transient_growth() {
        let mut caps = limits();
        caps = caps.into_builder().managed_edges(2).finish().unwrap();
        let mut owner = ParseControl::new(caps, HtmlParseCancellation::new());
        let identity = owner.identity();
        owner
            .acquire_many(&[
                (HtmlParseResource::AuxiliaryEntries, 2),
                (HtmlParseResource::ManagedEdges, 2),
            ])
            .unwrap();
        owner.commit_child_orders(&identity, 2, 1, 1).unwrap();
        assert_eq!(
            owner
                .usage()
                .resource(HtmlParseResource::AuxiliaryEntries)
                .current,
            0
        );
        assert_eq!(
            owner.usage().resource(HtmlParseResource::ManagedEdges),
            ResourceUsage {
                current: 2,
                high_water: 2,
            }
        );
        let before = owner.usage().clone();
        owner.commit_child_orders(&identity, 0, 0, 0).unwrap();
        assert_eq!(owner.usage(), &before);
    }
    #[test]
    fn child_order_cap_failure_retains_scratch_edges_and_first_stop() {
        let mut caps = limits();
        caps = caps.into_builder().managed_edges(1).finish().unwrap();
        let mut owner = ParseControl::new(caps, HtmlParseCancellation::new());
        let identity = owner.identity();
        owner
            .acquire_many(&[
                (HtmlParseResource::AuxiliaryEntries, 2),
                (HtmlParseResource::ManagedEdges, 1),
            ])
            .unwrap();
        let before = owner.usage().clone();
        let stop = ParseStop::LimitExceeded {
            resource: HtmlParseResource::ManagedEdges,
            limit: 1,
            current: 1,
            requested_delta: 1,
        };
        assert_eq!(owner.commit_child_orders(&identity, 2, 1, 0), Err(stop));
        assert_eq!(owner.usage(), &before);
        owner.cancellation.cancel();
        assert_eq!(owner.commit_child_orders(&identity, 0, 0, 0), Err(stop));
        assert_eq!(owner.usage(), &before);
        owner
            .cleanup_release(&identity, HtmlParseResource::AuxiliaryEntries, 2)
            .unwrap();
        assert_eq!(
            owner
                .usage()
                .resource(HtmlParseResource::AuxiliaryEntries)
                .current,
            0
        );
        assert_eq!(
            owner
                .usage()
                .resource(HtmlParseResource::AuxiliaryEntries)
                .high_water,
            2
        );
        assert_eq!(
            owner.usage().resource(HtmlParseResource::ManagedEdges),
            before.resource(HtmlParseResource::ManagedEdges)
        );
        assert_eq!(owner.stop(), Some(stop));
    }
    #[test]
    fn child_order_foreign_owner_rejects_before_any_counter_commit() {
        let mut owner = control();
        let foreign_identity = control().identity();
        owner
            .acquire_many(&[
                (HtmlParseResource::AuxiliaryEntries, 2),
                (HtmlParseResource::ManagedEdges, 1),
            ])
            .unwrap();
        let before = owner.usage().clone();
        assert_eq!(
            owner.commit_child_orders(&foreign_identity, 2, 0, 1),
            Err(ParseStop::InternalInvariant(Invariant::ForeignControl))
        );
        assert_eq!(owner.usage(), &before);
    }
    #[test]
    fn child_order_underflow_rejects_all_counters_before_edge_arithmetic() {
        for (scratch, removal) in [(3, 0), (2, 2)] {
            let mut owner = control();
            let identity = owner.identity();
            owner
                .acquire_many(&[
                    (HtmlParseResource::AuxiliaryEntries, 2),
                    (HtmlParseResource::ManagedEdges, 1),
                ])
                .unwrap();
            let before = owner.usage().clone();
            assert_eq!(
                owner.commit_child_orders(&identity, scratch, u64::MAX, removal),
                Err(ParseStop::InternalInvariant(Invariant::ReleaseUnderflow))
            );
            assert_eq!(owner.usage(), &before);
        }
    }
    #[test]
    fn child_order_edge_addition_overflow_retains_scratch_and_existing_edges() {
        let mut owner = control();
        let identity = owner.identity();
        owner
            .acquire_many(&[
                (HtmlParseResource::AuxiliaryEntries, 2),
                (HtmlParseResource::ManagedEdges, 1),
            ])
            .unwrap();
        let before = owner.usage().clone();
        assert_eq!(
            owner.commit_child_orders(&identity, 2, u64::MAX, 0),
            Err(ParseStop::RepresentationExhausted {
                resource: HtmlParseResource::ManagedEdges,
            })
        );
        assert_eq!(owner.usage(), &before);
    }
    #[test]
    fn child_order_final_checkpoint_cancellation_precedes_validation_even_for_empty_commit() {
        for (scratch, addition, removal) in [(0, 0, 0), (3, u64::MAX, 2)] {
            let mut owner = control();
            let identity = owner.identity();
            owner
                .acquire_many(&[
                    (HtmlParseResource::AuxiliaryEntries, 2),
                    (HtmlParseResource::ManagedEdges, 1),
                ])
                .unwrap();
            owner.cancellation.cancel();
            let before = owner.usage().clone();
            assert_eq!(
                owner.commit_child_orders(&identity, scratch, addition, removal),
                Err(ParseStop::Cancelled)
            );
            assert_eq!(owner.usage(), &before);
            assert_eq!(
                owner.commit_child_orders(&identity, 2, 0, 1),
                Err(ParseStop::Cancelled)
            );
            assert_eq!(owner.usage(), &before);
        }
    }
}
