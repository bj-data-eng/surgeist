//! Private accounting owner for parsing. Counts describe admitted physical work
//! and owned storage, not CPU instructions, elapsed time, allocator metadata or RSS.

use crate::{HtmlSource, HtmlSourceCoordinates, HtmlSourceId};
use std::{
    mem::size_of,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

const RESOURCE_COUNT: usize = 19;
const WORK_COUNT: usize = 8;
pub(crate) const WORK_QUANTUM: usize = 256;

/// Declaration order is the precedence for simultaneous resource failures.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(usize)]
pub(crate) enum Resource {
    SourceUnits,
    BufferedUtf16Units,
    TokenAttributes,
    AuxiliaryEntries,
    ManagedNodes,
    ManagedEdges,
    ManagedAttributes,
    ManagedUtf16Units,
    SourceMapEntries,
    SourceMapPayloadUnits,
    Diagnostics,
    DiagnosticPayloadUnits,
    ContextEffects,
    OpenElements,
    TreeDepth,
    ManagedCapacityBytes,
    PeakTransientBytes,
    AtomicLayoutBytes,
    WorkCredits,
}

const RESOURCES: [Resource; RESOURCE_COUNT] = [
    Resource::SourceUnits,
    Resource::BufferedUtf16Units,
    Resource::TokenAttributes,
    Resource::AuxiliaryEntries,
    Resource::ManagedNodes,
    Resource::ManagedEdges,
    Resource::ManagedAttributes,
    Resource::ManagedUtf16Units,
    Resource::SourceMapEntries,
    Resource::SourceMapPayloadUnits,
    Resource::Diagnostics,
    Resource::DiagnosticPayloadUnits,
    Resource::ContextEffects,
    Resource::OpenElements,
    Resource::TreeDepth,
    Resource::ManagedCapacityBytes,
    Resource::PeakTransientBytes,
    Resource::AtomicLayoutBytes,
    Resource::WorkCredits,
];

impl Resource {
    fn storage(self) -> bool {
        (self as usize) > 0 && (self as usize) < 14
    }
}

/// No implicit defaults or unlimited sentinel. Zero is an ordinary finite cap.
#[derive(Clone, Copy, Debug)]
pub(crate) struct ParseLimits {
    pub(crate) source_units: u64,
    pub(crate) buffered_utf16_units: u64,
    pub(crate) token_attributes: u64,
    pub(crate) auxiliary_entries: u64,
    pub(crate) managed_nodes: u64,
    pub(crate) managed_edges: u64,
    pub(crate) managed_attributes: u64,
    pub(crate) managed_utf16_units: u64,
    pub(crate) source_map_entries: u64,
    pub(crate) source_map_payload_units: u64,
    pub(crate) diagnostics: u64,
    pub(crate) diagnostic_payload_units: u64,
    pub(crate) context_effects: u64,
    pub(crate) open_elements: u64,
    pub(crate) tree_depth: u64,
    pub(crate) managed_capacity_bytes: u64,
    pub(crate) peak_transient_bytes: u64,
    pub(crate) atomic_layout_bytes: u64,
    pub(crate) work_credits: u64,
}

impl ParseLimits {
    fn validate(self) -> Result<[u64; RESOURCE_COUNT], LimitConstructionError> {
        let values = self.values();
        for resource in RESOURCES {
            let maximum = match resource {
                Resource::WorkCredits => u64::MAX,
                Resource::BufferedUtf16Units
                | Resource::ManagedUtf16Units
                | Resource::SourceMapPayloadUnits
                | Resource::DiagnosticPayloadUnits => (isize::MAX as u64) / 2,
                Resource::ManagedCapacityBytes | Resource::PeakTransientBytes => usize::MAX as u64,
                _ => isize::MAX as u64,
            };
            let provided = values[resource as usize];
            if provided > maximum {
                return Err(LimitConstructionError {
                    resource,
                    provided,
                    maximum,
                });
            }
        }
        Ok(values)
    }
    fn values(self) -> [u64; RESOURCE_COUNT] {
        [
            self.source_units,
            self.buffered_utf16_units,
            self.token_attributes,
            self.auxiliary_entries,
            self.managed_nodes,
            self.managed_edges,
            self.managed_attributes,
            self.managed_utf16_units,
            self.source_map_entries,
            self.source_map_payload_units,
            self.diagnostics,
            self.diagnostic_payload_units,
            self.context_effects,
            self.open_elements,
            self.tree_depth,
            self.managed_capacity_bytes,
            self.peak_transient_bytes,
            self.atomic_layout_bytes,
            self.work_credits,
        ]
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct LimitConstructionError {
    pub(crate) resource: Resource,
    pub(crate) provided: u64,
    pub(crate) maximum: u64,
}

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

#[derive(Clone, Debug)]
pub(crate) struct Cancellation(Arc<AtomicBool>);
impl Cancellation {
    pub(crate) fn new() -> Self {
        Self(Arc::new(AtomicBool::new(false)))
    }
    pub(crate) fn cancel(&self) {
        self.0.store(true, Ordering::Release);
    }
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
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ParseStop {
    Cancelled,
    LimitExceeded {
        resource: Resource,
        limit: u64,
        current: u64,
        requested_delta: u64,
    },
    RepresentationExhausted {
        resource: Resource,
    },
    AllocationFailed {
        resource: Resource,
        layout_bytes: u64,
    },
    InternalInvariant(Invariant),
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
    pub(crate) fn resource(&self, resource: Resource) -> ResourceUsage {
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
    cancellation: Cancellation,
    terminal: Option<ParseStop>,
    source: Option<(HtmlSourceId, HtmlSourceCoordinates)>,
    identity: Arc<()>,
}

#[derive(Clone)]
pub(crate) struct ControlIdentity(Arc<()>);

impl ParseControl {
    pub(crate) fn new(
        limits: ParseLimits,
        cancellation: Cancellation,
    ) -> Result<Self, LimitConstructionError> {
        let limits = limits.validate()?;
        Ok(Self {
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
            source: None,
            identity: Arc::new(()),
        })
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
    pub(crate) fn fail_invariant(&mut self, invariant: Invariant) -> ParseStop {
        if let Err(reason) = self.checkpoint() {
            return reason;
        }
        *self
            .terminal
            .get_or_insert(ParseStop::InternalInvariant(invariant))
    }
    fn fail<T>(&mut self, reason: ParseStop) -> Result<T, ParseStop> {
        let first = *self.terminal.get_or_insert(reason);
        Err(first)
    }
    pub(crate) fn checkpoint(&mut self) -> Result<(), ParseStop> {
        if let Some(reason) = self.terminal {
            return Err(reason);
        }
        if self.cancellation.0.load(Ordering::Acquire) {
            return self.fail(ParseStop::Cancelled);
        }
        Ok(())
    }
    fn representation<T>(&mut self, resource: Resource) -> Result<T, ParseStop> {
        self.fail(ParseStop::RepresentationExhausted { resource })
    }
    fn check_value(&mut self, resource: Resource, value: u64, delta: u64) -> Result<(), ParseStop> {
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
    fn set(&mut self, resource: Resource, value: u64) {
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
            return self.representation(Resource::SourceUnits);
        };
        self.check_value(Resource::SourceUnits, units, units)?;
        self.set(Resource::SourceUnits, units);
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
                return self.representation(Resource::WorkCredits);
            };
            work[category as usize] = next;
            increment = total;
        }
        let Some(total) = self
            .usage
            .resource(Resource::WorkCredits)
            .current
            .checked_add(increment)
        else {
            return self.representation(Resource::WorkCredits);
        };
        self.check_value(Resource::WorkCredits, total, increment)?;
        self.usage.work = work;
        self.set(Resource::WorkCredits, total);
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
        requests: &[(Resource, u64)],
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
    pub(crate) fn acquire(&mut self, resource: Resource, units: u64) -> Result<(), ParseStop> {
        self.acquire_many(&[(resource, units)])
    }
    /// Coupled logical allocations admit atomically in resource precedence order.
    pub(crate) fn acquire_many(&mut self, requests: &[(Resource, u64)]) -> Result<(), ParseStop> {
        let values = self.plan_storage(requests)?;
        for resource in RESOURCES {
            if resource.storage() {
                self.set(resource, values[resource as usize]);
            }
        }
        Ok(())
    }
    pub(crate) fn release(&mut self, resource: Resource, units: u64) -> Result<(), ParseStop> {
        self.checkpoint()?;
        if !resource.storage() {
            return self.fail(ParseStop::InternalInvariant(Invariant::NonStorageResource));
        }
        if resource == Resource::ManagedNodes {
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
            Resource::TreeDepth,
            depth,
            depth.saturating_sub(self.usage.resource(Resource::TreeDepth).current),
        )?;
        self.set(Resource::TreeDepth, depth);
        Ok(())
    }
    fn reserve<T>(&mut self, vector: &mut Vec<T>, plan: ReservationPlan) -> Result<(), ParseStop> {
        self.checkpoint()?;
        let current = self.usage.resource(Resource::ManagedCapacityBytes).current;
        let Some(without_old) = current.checked_sub(plan.old_bytes) else {
            return self.fail(ParseStop::InternalInvariant(Invariant::CapacityMismatch));
        };
        let (Some(new_total), Some(overlap)) = (
            without_old.checked_add(plan.new_bytes),
            current.checked_add(plan.new_bytes),
        ) else {
            return self.representation(Resource::ManagedCapacityBytes);
        };
        self.check_value(
            Resource::ManagedCapacityBytes,
            new_total,
            plan.new_bytes - plan.old_bytes,
        )?;
        self.check_value(Resource::PeakTransientBytes, overlap, plan.new_bytes)?;
        self.check_value(Resource::AtomicLayoutBytes, plan.new_bytes, plan.new_bytes)?;
        self.set(Resource::PeakTransientBytes, overlap);
        self.set(Resource::AtomicLayoutBytes, plan.new_bytes);
        self.usage.largest_atomic_layout = self.usage.largest_atomic_layout.max(plan.new_bytes);
        if vector.try_reserve_exact(plan.additional).is_err() {
            self.set(Resource::PeakTransientBytes, current);
            return self.fail(ParseStop::AllocationFailed {
                resource: Resource::ManagedCapacityBytes,
                layout_bytes: plan.new_bytes,
            });
        }
        let Some(actual) = vector
            .capacity()
            .checked_mul(size_of::<T>())
            .and_then(|bytes| u64::try_from(bytes).ok())
        else {
            return self.representation(Resource::ManagedCapacityBytes);
        };
        let Some(actual_total) = without_old.checked_add(actual) else {
            return self.representation(Resource::ManagedCapacityBytes);
        };
        self.usage.actual_capacity_bytes = actual_total;
        self.usage.actual_capacity_high_water =
            self.usage.actual_capacity_high_water.max(actual_total);
        self.set(Resource::ManagedCapacityBytes, actual_total);
        let Some(actual_overlap) = current.checked_add(actual) else {
            return self.representation(Resource::PeakTransientBytes);
        };
        self.set(Resource::PeakTransientBytes, actual_overlap);
        self.set(Resource::PeakTransientBytes, actual_total);
        if actual > plan.new_bytes {
            return self.fail(ParseStop::LimitExceeded {
                resource: Resource::ManagedCapacityBytes,
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
        resource: Resource,
        units: u64,
        bytes: u64,
    ) -> Result<(), ParseStop> {
        let (Some(logical), Some(capacity)) = (
            self.usage.resource(resource).current.checked_sub(units),
            self.usage
                .resource(Resource::ManagedCapacityBytes)
                .current
                .checked_sub(bytes),
        ) else {
            return self.fail(ParseStop::InternalInvariant(Invariant::ReleaseUnderflow));
        };
        self.set(resource, logical);
        self.set(Resource::ManagedCapacityBytes, capacity);
        self.set(Resource::PeakTransientBytes, capacity);
        self.usage.actual_capacity_bytes = capacity;
        Ok(())
    }
    /// Admits only the known fixed Arc payload. Arc headers, allocator metadata,
    /// and its infallible setup are provider limitations, not measured capacity.
    /// The consumer polls after the actual shared-wrapper construction.
    pub(crate) fn admit_document_string_wrapper(
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
        };
        Ok((buffer.into_vec(), allocation))
    }
    fn admit_document_string_payload(&mut self, resource: Resource) -> Result<u64, ParseStop> {
        self.checkpoint()?;
        if resource != Resource::BufferedUtf16Units {
            return self.fail(ParseStop::InternalInvariant(Invariant::NonStorageResource));
        }
        let bytes = size_of::<Vec<u16>>() as u64;
        self.charge_many(&[
            (WorkCategory::ReserveByte, bytes),
            (WorkCategory::ProviderCall, 1),
        ])?;
        let current = self.usage.resource(Resource::ManagedCapacityBytes).current;
        let Some(total) = current.checked_add(bytes) else {
            return self.representation(Resource::ManagedCapacityBytes);
        };
        self.check_value(Resource::ManagedCapacityBytes, total, bytes)?;
        self.check_value(Resource::PeakTransientBytes, total, bytes)?;
        self.check_value(Resource::AtomicLayoutBytes, bytes, bytes)?;
        self.set(Resource::ManagedCapacityBytes, total);
        self.set(Resource::PeakTransientBytes, total);
        self.set(Resource::AtomicLayoutBytes, bytes);
        self.usage.largest_atomic_layout = self.usage.largest_atomic_layout.max(bytes);
        // This total includes known fixed payloads, never opaque provider headers.
        self.usage.actual_capacity_bytes = total;
        self.usage.actual_capacity_high_water = self.usage.actual_capacity_high_water.max(total);
        Ok(bytes)
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
            return self.representation(Resource::ManagedCapacityBytes);
        };
        self.cleanup_storage(Resource::BufferedUtf16Units, allocation.units, bytes)
    }
}

/// Non-cloneable accounting ticket for the transferred payload. It must remain
/// alongside the frozen text until the execution owner releases that allocation.
pub(crate) struct DocumentStringAllocation {
    owner: Arc<()>,
    units: u64,
    vector_capacity: usize,
    wrapper_bytes: u64,
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
                    resource: Resource::ManagedCapacityBytes,
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
                    resource: Resource::ManagedCapacityBytes,
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
    resource: Resource,
    owner: Option<Arc<()>>,
}
impl<T> OwnedVec<T> {
    pub(crate) fn new(resource: Resource) -> Result<Self, ParseStop> {
        if !resource.storage() || resource == Resource::ManagedNodes {
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
                None => return control.representation(Resource::ManagedCapacityBytes),
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
            control.check_value(Resource::AtomicLayoutBytes, bytes, bytes)?;
            control.set(Resource::AtomicLayoutBytes, bytes);
            control.usage.largest_atomic_layout = control.usage.largest_atomic_layout.max(bytes);
        }
        control.acquire(self.resource, 1)?;
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
                control.check_value(Resource::AtomicLayoutBytes, bytes, bytes)?;
                control.set(Resource::AtomicLayoutBytes, bytes);
                control.usage.largest_atomic_layout =
                    control.usage.largest_atomic_layout.max(bytes);
            }
            control.release(self.resource, 1)?;
            drop(self.vector.pop());
        }
        Ok(())
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
            return control.representation(Resource::ManagedCapacityBytes);
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
                control.check_value(Resource::AtomicLayoutBytes, bytes, bytes)?;
                control.set(Resource::AtomicLayoutBytes, bytes);
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
    use super::*;
    use crate::DocumentString;

    fn limits() -> ParseLimits {
        ParseLimits {
            source_units: 1_000,
            buffered_utf16_units: 1_000,
            token_attributes: 1_000,
            auxiliary_entries: 1_000,
            managed_nodes: 1_000,
            managed_edges: 1_000,
            managed_attributes: 1_000,
            managed_utf16_units: 1_000,
            source_map_entries: 1_000,
            source_map_payload_units: 1_000,
            diagnostics: 1_000,
            diagnostic_payload_units: 1_000,
            context_effects: 1_000,
            open_elements: 1_000,
            tree_depth: 1_000,
            managed_capacity_bytes: 16_000,
            peak_transient_bytes: 32_000,
            atomic_layout_bytes: 16_000,
            work_credits: 100_000,
        }
    }
    fn control() -> ParseControl {
        ParseControl::new(limits(), Cancellation::new()).unwrap()
    }

    #[test]
    fn all_limits_have_independent_slots() {
        let limits = ParseLimits {
            source_units: 1,
            buffered_utf16_units: 2,
            token_attributes: 3,
            auxiliary_entries: 4,
            managed_nodes: 5,
            managed_edges: 6,
            managed_attributes: 7,
            managed_utf16_units: 8,
            source_map_entries: 9,
            source_map_payload_units: 10,
            diagnostics: 11,
            diagnostic_payload_units: 12,
            context_effects: 13,
            open_elements: 14,
            tree_depth: 15,
            managed_capacity_bytes: 16,
            peak_transient_bytes: 17,
            atomic_layout_bytes: 18,
            work_credits: 19,
        };
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
        assert_eq!(control.usage().resource(Resource::SourceUnits).current, 5);
        assert_eq!(
            control
                .usage()
                .resource(Resource::ManagedCapacityBytes)
                .current,
            0
        );
        let source = HtmlSource::utf16(DocumentString::from_utf16([0xD800, 0x61]));
        let mut control = super::tests::control();
        control.admit_source(&source).unwrap();
        assert_eq!(control.usage().resource(Resource::SourceUnits).current, 2);
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
        limits.source_units = 0;
        let mut control = ParseControl::new(limits, Cancellation::new()).unwrap();
        control.admit_source(&HtmlSource::utf8("")).unwrap();
        let mut control = ParseControl::new(limits, Cancellation::new()).unwrap();
        assert_eq!(
            control.admit_source(&HtmlSource::utf8("a")),
            Err(ParseStop::LimitExceeded {
                resource: Resource::SourceUnits,
                limit: 0,
                current: 0,
                requested_delta: 1
            })
        );
    }
    #[test]
    fn cancellation_is_sticky_across_threads() {
        let cancellation = Cancellation::new();
        let other = cancellation.clone();
        std::thread::spawn(move || {
            other.cancel();
            other.cancel();
        })
        .join()
        .unwrap();
        let mut control = ParseControl::new(limits(), cancellation).unwrap();
        assert_eq!(
            control.charge(WorkCategory::StateStep, 1),
            Err(ParseStop::Cancelled)
        );
        assert_eq!(control.usage().resource(Resource::WorkCredits).current, 0);
    }
    #[test]
    fn observed_cancel_precedes_limit_and_first_stop_never_changes() {
        let cancellation = Cancellation::new();
        cancellation.cancel();
        let mut limits = limits();
        limits.work_credits = 0;
        let mut control = ParseControl::new(limits, cancellation).unwrap();
        assert_eq!(
            control.charge(WorkCategory::StateStep, 1),
            Err(ParseStop::Cancelled)
        );
        assert_eq!(
            control.acquire(Resource::Diagnostics, u64::MAX),
            Err(ParseStop::Cancelled)
        );
        assert_eq!(control.stop(), Some(ParseStop::Cancelled));
    }
    #[test]
    fn cap_failure_precedes_later_cancel() {
        let cancellation = Cancellation::new();
        let mut limits = limits();
        limits.work_credits = 0;
        let mut control = ParseControl::new(limits, cancellation.clone()).unwrap();
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
        assert_eq!(control.usage().resource(Resource::WorkCredits).current, 16);
    }
    #[test]
    fn rejected_work_does_not_change_any_work_counter() {
        let mut limits = limits();
        limits.work_credits = 3;
        let mut control = ParseControl::new(limits, Cancellation::new()).unwrap();
        control.charge(WorkCategory::StateStep, 2).unwrap();
        assert_eq!(
            control.charge(WorkCategory::InputUnitRead, 2),
            Err(ParseStop::LimitExceeded {
                resource: Resource::WorkCredits,
                limit: 3,
                current: 2,
                requested_delta: 2
            })
        );
        assert_eq!(control.usage().work(WorkCategory::InputUnitRead), 0);
        assert_eq!(control.usage().resource(Resource::WorkCredits).current, 2);
    }
    #[test]
    fn arithmetic_overflow_is_terminal_and_atomic() {
        let mut limits = limits();
        limits.work_credits = u64::MAX;
        let mut control = ParseControl::new(limits, Cancellation::new()).unwrap();
        control.charge(WorkCategory::StateStep, u64::MAX).unwrap();
        assert_eq!(
            control.charge(WorkCategory::RecordVisit, 1),
            Err(ParseStop::RepresentationExhausted {
                resource: Resource::WorkCredits
            })
        );
        assert_eq!(control.usage().work(WorkCategory::RecordVisit), 0);
    }
    #[test]
    fn coupled_resources_fail_in_declared_order_without_partial_admission() {
        let mut limits = limits();
        limits.diagnostics = 0;
        limits.buffered_utf16_units = 0;
        let mut control = ParseControl::new(limits, Cancellation::new()).unwrap();
        assert_eq!(
            control.acquire_many(&[
                (Resource::Diagnostics, 1),
                (Resource::BufferedUtf16Units, 1)
            ]),
            Err(ParseStop::LimitExceeded {
                resource: Resource::BufferedUtf16Units,
                limit: 0,
                current: 0,
                requested_delta: 1
            })
        );
        assert_eq!(control.usage().resource(Resource::Diagnostics).current, 0);
    }
    #[test]
    fn diagnostic_exhaustion_is_a_terminal_error() {
        let mut limits = limits();
        limits.diagnostics = 1;
        let mut control = ParseControl::new(limits, Cancellation::new()).unwrap();
        control.acquire(Resource::Diagnostics, 1).unwrap();
        assert!(matches!(
            control.acquire(Resource::Diagnostics, 1),
            Err(ParseStop::LimitExceeded {
                resource: Resource::Diagnostics,
                ..
            })
        ));
        assert!(control.checkpoint().is_err());
    }
    #[test]
    fn release_retains_high_water_and_managed_nodes_are_not_refunded() {
        let mut control = control();
        control.acquire(Resource::OpenElements, 7).unwrap();
        control.release(Resource::OpenElements, 5).unwrap();
        assert_eq!(
            control.usage().resource(Resource::OpenElements),
            ResourceUsage {
                current: 2,
                high_water: 7
            }
        );
        control.acquire(Resource::ManagedNodes, 1).unwrap();
        assert_eq!(
            control.release(Resource::ManagedNodes, 1),
            Err(ParseStop::InternalInvariant(Invariant::NonReclaimableNodes))
        );
    }
    #[test]
    fn release_underflow_cannot_wrap() {
        let mut control = control();
        assert_eq!(
            control.release(Resource::ManagedEdges, 1),
            Err(ParseStop::InternalInvariant(Invariant::ReleaseUnderflow))
        );
        assert_eq!(control.usage().resource(Resource::ManagedEdges).current, 0);
    }
    #[test]
    fn ancestry_depth_is_independent_of_open_stack() {
        let mut control = control();
        control.acquire(Resource::OpenElements, 2).unwrap();
        control.observe_tree_depth(12).unwrap();
        control.observe_tree_depth(3).unwrap();
        assert_eq!(
            control.usage().resource(Resource::TreeDepth),
            ResourceUsage {
                current: 3,
                high_water: 12
            }
        );
        assert_eq!(control.usage().resource(Resource::OpenElements).current, 2);
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
                resource: Resource::ManagedCapacityBytes
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
        limits.managed_capacity_bytes = 1;
        let mut control = ParseControl::new(limits, Cancellation::new()).unwrap();
        let mut buffer = OwnedVec::<u16>::new(Resource::BufferedUtf16Units).unwrap();
        assert!(matches!(
            buffer.try_push(&mut control, 0xD800),
            Err(ParseStop::LimitExceeded {
                resource: Resource::ManagedCapacityBytes,
                ..
            })
        ));
        assert_eq!(buffer.len(), 0);
        assert_eq!(buffer.capacity(), 0);
        assert_eq!(
            control
                .usage()
                .resource(Resource::BufferedUtf16Units)
                .current,
            0
        );
    }
    #[test]
    fn actual_capacity_and_overlap_are_recorded_and_cleanup_after_stop_works() {
        let mut control = control();
        let mut buffer = OwnedVec::<u16>::new(Resource::BufferedUtf16Units).unwrap();
        buffer
            .try_extend_from_slice(&mut control, &[0xD800, 0, 0xDC00])
            .unwrap();
        assert_eq!(buffer.as_slice(), &[0xD800, 0, 0xDC00]);
        assert_eq!(control.usage().actual_capacity_bytes, 6);
        buffer.try_push(&mut control, 0x61).unwrap();
        assert_eq!(
            control
                .usage()
                .resource(Resource::PeakTransientBytes)
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
            control.usage().resource(Resource::BufferedUtf16Units),
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
        let mut buffer = OwnedVec::<u16>::new(Resource::BufferedUtf16Units).unwrap();
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
        for resource in [Resource::PeakTransientBytes, Resource::AtomicLayoutBytes] {
            let mut limits = limits();
            if resource == Resource::PeakTransientBytes {
                limits.peak_transient_bytes = 1;
            } else {
                limits.atomic_layout_bytes = 1;
            }
            let mut control = ParseControl::new(limits, Cancellation::new()).unwrap();
            let mut buffer = OwnedVec::<u16>::new(Resource::BufferedUtf16Units).unwrap();
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
        assert_eq!(admitted.usage().resource(Resource::SourceUnits).current, 1);
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
        let mut buffer = OwnedVec::<u16>::new(Resource::BufferedUtf16Units).unwrap();
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
            .cleanup_storage(Resource::BufferedUtf16Units, 1, 2)
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
        let mut buffer = OwnedVec::<u16>::new(Resource::BufferedUtf16Units).unwrap();
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
        limits.managed_capacity_bytes = usize::MAX as u64;
        limits.peak_transient_bytes = usize::MAX as u64;
        limits.atomic_layout_bytes = isize::MAX as u64;
        limits.work_credits = u64::MAX;
        let mut control = ParseControl::new(limits, Cancellation::new()).unwrap();
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
                resource: Resource::ManagedCapacityBytes,
                layout_bytes: isize::MAX as u64,
            })
        );
        assert_eq!(vector.capacity(), 0);
        assert_eq!(control.usage().actual_capacity_bytes, 0);
    }
    #[test]
    fn construction_rejects_unrepresentable_units_and_layout_in_resource_order() {
        let mut limits = limits();
        limits.buffered_utf16_units = isize::MAX as u64;
        limits.atomic_layout_bytes = u64::MAX;
        assert!(
            matches!(ParseControl::new(limits, Cancellation::new()), Err(LimitConstructionError {
            resource: Resource::BufferedUtf16Units, provided, maximum,
        }) if provided == isize::MAX as u64 && maximum == isize::MAX as u64 / 2)
        );
        limits.buffered_utf16_units = 0;
        assert!(
            matches!(ParseControl::new(limits, Cancellation::new()), Err(LimitConstructionError {
            resource: Resource::AtomicLayoutBytes, maximum, ..
        }) if maximum == isize::MAX as u64)
        );
    }
    #[test]
    fn maximal_finite_work_and_valid_zero_caps_are_constructible() {
        let mut limits = limits();
        limits.work_credits = u64::MAX;
        limits.buffered_utf16_units = 0;
        limits.source_units = 0;
        let mut control = ParseControl::new(limits, Cancellation::new()).unwrap();
        control.charge(WorkCategory::StateStep, u64::MAX).unwrap();
        assert_eq!(
            control.usage().resource(Resource::WorkCredits).current,
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
            OwnedVec::<u16>::new(Resource::SourceUnits),
            Err(ParseStop::InternalInvariant(Invariant::NonStorageResource))
        ));
        assert!(matches!(
            OwnedVec::<()>::new(Resource::BufferedUtf16Units),
            Err(ParseStop::InternalInvariant(Invariant::ZeroSizedStorage))
        ));
        let mut control = control();
        assert_eq!(
            control.acquire(Resource::WorkCredits, 1),
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
        let mut first_buffer = OwnedVec::<u16>::new(Resource::BufferedUtf16Units).unwrap();
        first_buffer.try_push(&mut first, 4).unwrap();
        let (vector, ticket) = first.admit_document_string_wrapper(first_buffer).unwrap();
        let mut second_buffer = OwnedVec::<u16>::new(Resource::BufferedUtf16Units).unwrap();
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
        let mut buffer = OwnedVec::<u16>::new(Resource::BufferedUtf16Units).unwrap();
        buffer
            .try_extend_from_slice(&mut control, &[1, 2, 3])
            .unwrap();
        let previous = control.usage().resource(Resource::WorkCredits).current;
        let copy = control.usage().work(WorkCategory::CopyByte);
        buffer.clear(&mut control).unwrap();
        assert_eq!(
            control.usage().resource(Resource::WorkCredits).current,
            previous + 12
        );
        assert_eq!(control.usage().work(WorkCategory::RecordVisit), 3);
        assert_eq!(control.usage().work(WorkCategory::RecordWrite), 3);
        assert_eq!(control.usage().work(WorkCategory::CopyByte), copy + 6);
        assert_eq!(
            control.usage().resource(Resource::BufferedUtf16Units),
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
        let mut buffer = OwnedVec::<u16>::new(Resource::BufferedUtf16Units).unwrap();
        buffer
            .try_extend_from_slice(&mut control, &[1, 2, 3])
            .unwrap();
        let previous = control.usage().resource(Resource::WorkCredits).current;
        control.limits[Resource::WorkCredits as usize] = previous + 4;
        let stop = buffer.clear(&mut control).unwrap_err();
        assert_eq!(buffer.as_slice(), &[1, 2]);
        assert_eq!(
            control
                .usage()
                .resource(Resource::BufferedUtf16Units)
                .current,
            2
        );
        let work = control.usage().resource(Resource::WorkCredits).current;
        buffer.release(&mut control).unwrap();
        assert_eq!(control.stop(), Some(stop));
        assert_eq!(
            control.usage().resource(Resource::WorkCredits).current,
            work
        );
        assert_eq!(control.usage().actual_capacity_bytes, 0);
    }
    #[test]
    fn work_limit_precedes_storage_limit_for_a_growth_operation() {
        let mut limits = limits();
        limits.work_credits = 3;
        limits.buffered_utf16_units = 0;
        let mut control = ParseControl::new(limits, Cancellation::new()).unwrap();
        let mut buffer = OwnedVec::<u16>::new(Resource::BufferedUtf16Units).unwrap();
        assert!(matches!(
            buffer.try_push(&mut control, 1),
            Err(ParseStop::LimitExceeded {
                resource: Resource::WorkCredits,
                ..
            })
        ));
        assert_eq!(
            control
                .usage()
                .resource(Resource::BufferedUtf16Units)
                .current,
            0
        );
    }
}
