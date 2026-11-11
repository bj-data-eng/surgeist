use super::{
    AllocationGeneration, FrameIdentity, ManagerIdentity, ResourceAccountingFault,
    ResourceCacheKey, ResourceIdentity,
    entries::ResourceEntries,
    gaussian::{GaussianKernelBufferLimits, GaussianKernelPlan},
    lock_state,
    manager::{
        IdleReuse, ResourceEntry, ResourceEntryState, ResourceManagerState, ResourcePayload,
    },
};
use crate::{
    BackendErrorCode, Error, Result,
    accounting::{
        Account, AccountedVec, CpuAccount, Generation, GenerationClass, RenderAccountingError,
        RenderCpuScope, RenderResource, RenderWork, StorageAccount,
    },
    backend::DeviceCapabilities,
    image::ResolvedMaskUploadDescriptor,
    texture::EffectTextureDescriptor,
    vello_engine::{VelloAtlasOutcome, VelloBufferKey, VelloImageKey},
};
use std::{
    fmt,
    sync::{Arc, Mutex},
};
use wgpu::util::DeviceExt;

/// Identity pair validated together for every acquisition into one active frame.
struct FrameLeaseOwner<'a> {
    manager_identity: &'a ManagerIdentity,
    frame: FrameIdentity,
}

enum ResolutionProofMode {
    CompleteFrame,
    ProvidedLeases,
}

/// Mandatory real cleanup transactions run even after the first health fault.
fn cleanup_phase<T>(cpu: &CpuAccount, run: impl FnOnce() -> T) -> T {
    cpu.maintenance_phase(run)
}

impl ResourceManagerState {
    fn ensure_frame_resolution_ready<'a>(
        &mut self,
        manager_identity: &ManagerIdentity,
        frame: FrameIdentity,
        tokens: impl Iterator<Item = &'a ResourceLeaseToken>,
    ) -> Result<()> {
        self.ensure_accounting_exact(None)?;
        self.prepare_resolution_proof(
            manager_identity,
            frame,
            tokens,
            ResolutionProofMode::CompleteFrame,
        )
    }
    fn prepare_resolution_proof<'a>(
        &mut self,
        manager_identity: &ManagerIdentity,
        frame: FrameIdentity,
        tokens: impl Iterator<Item = &'a ResourceLeaseToken>,
        mode: ResolutionProofMode,
    ) -> Result<()> {
        let unavailable_frame = || {
            Error::new(
                BackendErrorCode::RenderFailed,
                "resource frame is unavailable for clean accounting resolution",
            )
        };
        if self.identity != *manager_identity {
            return Err(unavailable_frame());
        }
        let scratch = self
            .active_frames
            .get_mut(&frame)
            .ok_or_else(unavailable_frame)?;
        let complete_frame = matches!(mode, ResolutionProofMode::CompleteFrame);
        scratch.indices.clear();
        if complete_frame {
            for (identity, entry) in self.entries.iter() {
                self.cpu
                    .at(RenderCpuScope::Cleanup)
                    .maintenance(RenderWork::ResourceProbes, 1);
                if entry.state == (ResourceEntryState::Leased { frame }) && entry.resolved {
                    scratch.push_index(0, *identity)?;
                }
            }
        }
        for token in tokens {
            Self::validate_lease_identity(&self.identity, manager_identity, frame, token)?;
            Self::validate_lease_entry(&self.entries, &self.generation, frame, token, None)?;
            if scratch.indices.len() == scratch.indices.capacity() {
                return Err(Self::invalid_lease(
                    token.resource_identity,
                    "must occur at most once in the active frame",
                ));
            }
            scratch.push_index(0, token.resource_identity)?;
        }
        {
            let cpu = self.cpu.at(RenderCpuScope::Cleanup);
            scratch.indices.sort_unstable_by(|a, b| {
                cpu.maintenance(RenderWork::ResourceProbes, 1);
                a.1.cmp(&b.1)
            });
            for pair in scratch.indices.windows(2) {
                cpu.maintenance(RenderWork::ResourceProbes, 1);
                if pair[0].1 == pair[1].1 {
                    return Err(Self::invalid_lease(
                        pair[0].1,
                        "must occur at most once in one clean resolution",
                    ));
                }
            }
        }
        if complete_frame {
            let mut covered = scratch.indices.iter().map(|(_, id)| *id);
            for (identity, entry) in self.entries.iter() {
                self.cpu
                    .at(RenderCpuScope::Cleanup)
                    .maintenance(RenderWork::ResourceProbes, 1);
                if entry.state == (ResourceEntryState::Leased { frame })
                    && covered.next() != Some(*identity)
                {
                    return Err(Error::new(
                        BackendErrorCode::RenderFailed,
                        "clean resource resolution must own every exact lease in its frame",
                    ));
                }
            }
            if covered.next().is_some() {
                return Err(Error::new(
                    BackendErrorCode::RenderFailed,
                    "clean resource resolution contains an unavailable lease",
                ));
            }
        }
        Ok(())
    }

    fn invalid_lease(resource: ResourceIdentity, reason: &'static str) -> Error {
        Error::invalid_value("resource lease", resource.get(), reason)
    }

    fn acquire_with_payload(
        &mut self,
        owner: FrameLeaseOwner<'_>,
        key: ResourceCacheKey,
        byte_len: u64,
        idle_reuse: IdleReuse,
        observer: Option<&Account>,
        create_payload: impl FnOnce(&Generation) -> Result<ResourcePayload>,
    ) -> Result<ResourceLease> {
        let FrameLeaseOwner {
            manager_identity,
            frame,
        } = owner;
        self.cpu.ensure_healthy()?;
        self.ensure_accounting_exact(observer)?;
        if self.identity != *manager_identity || !self.active_frames.contains_key(&frame) {
            return Err(Error::invalid_value(
                "resource frame",
                frame.0,
                "must belong to an active frame of this resource manager",
            ));
        }
        if byte_len == 0 {
            return Err(Error::invalid_value(
                "resource byte length",
                byte_len,
                "must be greater than zero",
            ));
        }

        let mut reusable = None;
        if idle_reuse == IdleReuse::Allowed {
            for (identity, entry) in self.entries.iter() {
                self.generation
                    .probe(entry.key.generation_class(), observer)?;
                if let ResourceEntryState::Idle { last_used_frame } = entry.state
                    && entry.key.matches_in(&key, observer)?
                    && entry.byte_len == byte_len
                {
                    let candidate = (last_used_frame, *identity);
                    if reusable.is_none_or(|selected| candidate < selected) {
                        reusable = Some(candidate);
                    }
                }
            }
        }
        let reusable = reusable.map(|(_, identity)| identity);

        let (resource, allocation_generation, payload) = if let Some(resource) = reusable {
            self.generation.probe_cache(observer)?;
            self.generation.reuse(key.generation_class())?;
            let entry = match Self::selected_entry_mut(&mut self.entries, resource) {
                Ok(entry) => entry,
                Err(fault) => return self.fail_accounting(fault),
            };
            entry.registration.set_idle(false)?;
            entry.resolved = false;
            entry.provisional = false;
            entry.state = ResourceEntryState::Leased { frame };
            (resource, entry.allocation_generation, entry.payload.clone())
        } else {
            let next_resource = self.next_resource.checked_add(1).ok_or_else(|| {
                Error::invalid_value(
                    "resource identity",
                    self.next_resource,
                    "must have remaining identity space",
                )
            })?;
            self.generation
                .preview_cache_allocations([byte_len], observer)
                .map_err(Self::allocation_preflight_error)?;
            let required = self
                .entries
                .len()
                .checked_add(1)
                .ok_or(RenderAccountingError::Invariant)?;
            for scratch in self.active_frames.values_mut() {
                scratch.reserve(required)?;
            }
            self.entries.reserve(1, &self.cpu)?;
            let resource = ResourceIdentity(next_resource);
            let slot = self.entries.append_slot(resource)?;
            #[cfg(test)]
            {
                self.payload_creation_attempts = self.payload_creation_attempts.saturating_add(1);
            }
            let payload = create_payload(&self.generation)?;
            if !payload.matches_key(&key) {
                return Err(Error::invalid_value(
                    "resource payload",
                    payload.label(),
                    "must match its exact resource cache-key namespace",
                ));
            }
            let allocation_generation = AllocationGeneration(1);
            let registration = self
                .generation
                .cache_registration(key.generation_class(), byte_len)?;
            slot.write(ResourceEntry {
                key,
                allocation_generation,
                byte_len,
                state: ResourceEntryState::Leased { frame },
                payload: payload.clone(),
                registration,
                resolved: false,
                provisional: true,
            });
            self.next_resource = next_resource;
            (resource, allocation_generation, payload)
        };

        Ok(ResourceLease::new(
            manager_identity.clone(),
            frame,
            resource,
            allocation_generation,
            payload,
        ))
    }

    #[cfg(test)]
    pub(super) fn acquire(
        &mut self,
        manager_identity: &ManagerIdentity,
        frame: FrameIdentity,
        key: ResourceCacheKey,
        byte_len: u64,
        observer: Option<&Account>,
    ) -> Result<ResourceLease> {
        if !key.accepts_modeled_payload() {
            return Err(Error::invalid_value(
                "modeled resource key",
                format!("{key:?}"),
                "must use a Vello atlas or effect texture namespace",
            ));
        }
        self.acquire_with_payload(
            FrameLeaseOwner {
                manager_identity,
                frame,
            },
            key.clone(),
            byte_len,
            IdleReuse::Allowed,
            observer,
            |generation| {
                Ok(ResourcePayload::Modeled {
                    _allocation: generation.allocation(
                        key.generation_class(),
                        byte_len,
                        observer,
                        || (),
                    )?,
                })
            },
        )
    }

    fn validate_lease(
        &self,
        manager_identity: &ManagerIdentity,
        frame: FrameIdentity,
        token: &ResourceLeaseToken,
        observer: Option<&Account>,
    ) -> Result<()> {
        Self::validate_lease_identity(&self.identity, manager_identity, frame, token)?;
        if !self.active_frames.contains_key(&frame) {
            return Err(Self::invalid_lease(
                token.resource_identity,
                "must belong to this active resource frame",
            ));
        }
        Self::validate_lease_entry(&self.entries, &self.generation, frame, token, observer)
    }

    fn validate_lease_identity(
        identity: &ManagerIdentity,
        manager_identity: &ManagerIdentity,
        frame: FrameIdentity,
        token: &ResourceLeaseToken,
    ) -> Result<()> {
        if identity != manager_identity || &token.manager_identity != identity {
            return Err(Self::invalid_lease(
                token.resource_identity,
                "must belong to this resource manager",
            ));
        }
        if token.frame_identity != frame {
            return Err(Self::invalid_lease(
                token.resource_identity,
                "must belong to this active resource frame",
            ));
        }
        Ok(())
    }

    /// Checks the entry after the caller admits the frame and qualifies the token identity.
    fn validate_lease_entry(
        entries: &ResourceEntries,
        generation: &Generation,
        frame: FrameIdentity,
        token: &ResourceLeaseToken,
        observer: Option<&Account>,
    ) -> Result<()> {
        generation.probe_cache(observer)?;
        let Some(entry) = entries.get(&token.resource_identity) else {
            return Err(Self::invalid_lease(
                token.resource_identity,
                "must name a current resource identity",
            ));
        };
        if entry.resolved {
            return Err(Self::invalid_lease(
                token.resource_identity,
                "must not have already been resolved by this frame",
            ));
        }
        if entry.allocation_generation != token.allocation_generation {
            return Err(Self::invalid_lease(
                token.resource_identity,
                "must name the current allocation generation",
            ));
        }
        if entry.state != (ResourceEntryState::Leased { frame }) {
            return Err(Self::invalid_lease(
                token.resource_identity,
                "must name a resource leased by this frame",
            ));
        }
        Ok(())
    }

    pub(super) fn release(
        &mut self,
        manager_identity: &ManagerIdentity,
        frame: FrameIdentity,
        token: ResourceLeaseToken,
    ) -> Result<()> {
        self.validate_lease(manager_identity, frame, &token, None)?;
        self.generation.probe_cache(None)?;
        let entry = match Self::selected_entry_mut(&mut self.entries, token.resource_identity) {
            Ok(entry) => entry,
            Err(fault) => return self.fail_accounting(fault),
        };
        entry.resolved = true;
        self.generation.lease_return(entry.key.generation_class());
        Ok(())
    }

    fn resolve_leases_atomically(
        &mut self,
        manager_identity: &ManagerIdentity,
        frame: FrameIdentity,
        leases: &[&ResourceLease],
    ) -> Result<()> {
        self.prepare_resolution_proof(
            manager_identity,
            frame,
            leases.iter().map(|lease| &lease.token),
            ResolutionProofMode::ProvidedLeases,
        )?;
        for lease in leases {
            self.generation.probe_cache(None)?;
            let entry =
                match Self::selected_entry_mut(&mut self.entries, lease.token.resource_identity) {
                    Ok(entry) => entry,
                    Err(fault) => return self.fail_accounting(fault),
                };
            entry.resolved = true;
            self.generation.lease_return(entry.key.generation_class());
        }
        Ok(())
    }

    #[cfg(test)]
    pub(super) fn replace(
        &mut self,
        manager_identity: &ManagerIdentity,
        frame: FrameIdentity,
        token: ResourceLeaseToken,
        key: ResourceCacheKey,
        byte_len: u64,
        observer: Option<&Account>,
    ) -> Result<ResourceLease> {
        self.ensure_accounting_healthy()?;
        self.validate_lease(manager_identity, frame, &token, observer)?;
        if !key.accepts_modeled_payload() {
            return Err(Error::invalid_value(
                "modeled resource key",
                format!("{key:?}"),
                "must use a Vello atlas or effect texture namespace",
            ));
        }
        if byte_len == 0 {
            return Err(Error::invalid_value(
                "resource byte length",
                byte_len,
                "must be greater than zero",
            ));
        }
        let entry = self
            .entries
            .get(&token.resource_identity)
            .expect("a validated resource lease must remain registered");
        let replaced_byte_len = entry.byte_len;
        let allocation_generation =
            AllocationGeneration(entry.allocation_generation.0.checked_add(1).ok_or_else(
                || {
                    Error::invalid_value(
                        "resource allocation generation",
                        entry.allocation_generation.0,
                        "must have remaining generation space",
                    )
                },
            )?);
        let retained_bytes = self.generation.registered_cache_bytes()?;
        let Some(retained_without_replaced) = retained_bytes.checked_sub(replaced_byte_len) else {
            return self.fail_accounting(ResourceAccountingFault::RetainedByteUnderflow {
                retained_bytes,
                discarded_entry_bytes: replaced_byte_len,
            });
        };
        let Some(registered_entry_bytes) = self.checked_registered_entry_bytes(observer)? else {
            return self.fail_accounting(ResourceAccountingFault::SurvivingEntryByteTotalOverflow);
        };
        if retained_bytes != registered_entry_bytes {
            return self.fail_accounting(ResourceAccountingFault::RetainedByteMismatch {
                retained_bytes,
                registered_entry_bytes,
            });
        }
        let Some(retained_bytes) = retained_without_replaced.checked_add(byte_len) else {
            return self.fail_accounting(ResourceAccountingFault::SurvivingEntryByteTotalOverflow);
        };
        let Some(registered_without_replaced) =
            registered_entry_bytes.checked_sub(replaced_byte_len)
        else {
            return self.fail_accounting(ResourceAccountingFault::RetainedByteUnderflow {
                retained_bytes: registered_entry_bytes,
                discarded_entry_bytes: replaced_byte_len,
            });
        };
        let Some(replaced_entry_bytes) = registered_without_replaced.checked_add(byte_len) else {
            return self.fail_accounting(ResourceAccountingFault::SurvivingEntryByteTotalOverflow);
        };
        if retained_bytes != replaced_entry_bytes {
            return self.fail_accounting(ResourceAccountingFault::RetainedByteMismatch {
                retained_bytes,
                registered_entry_bytes: replaced_entry_bytes,
            });
        }

        let payload = ResourcePayload::Modeled {
            _allocation: self.generation.allocation(
                key.generation_class(),
                byte_len,
                None,
                || (),
            )?,
        };
        let registration = self
            .generation
            .cache_registration(key.generation_class(), byte_len)?;
        let entry = self
            .entries
            .get_mut(&token.resource_identity)
            .expect("a validated resource lease must remain registered");
        entry.key = key;
        entry.allocation_generation = allocation_generation;
        entry.byte_len = byte_len;
        entry.payload = payload.clone();
        entry.registration = registration;
        Ok(ResourceLease::new(
            manager_identity.clone(),
            frame,
            token.resource_identity,
            allocation_generation,
            payload,
        ))
    }

    fn accounting_after_removal(&mut self, retained_before: u64, removed_bytes: u64) -> Result<()> {
        if self.accounting_fault.is_some() {
            return Ok(());
        }
        if retained_before.checked_sub(removed_bytes).is_none() {
            return self.fail_accounting(ResourceAccountingFault::RetainedByteUnderflow {
                retained_bytes: retained_before,
                discarded_entry_bytes: removed_bytes,
            });
        }
        self.ensure_accounting_exact(None)
    }

    fn discard(
        &mut self,
        manager_identity: &ManagerIdentity,
        frame: FrameIdentity,
        token: ResourceLeaseToken,
    ) -> Result<()> {
        self.validate_lease(manager_identity, frame, &token, None)?;
        let cpu_was_healthy = self.cpu.ensure_healthy().is_ok();
        let generation_health_before = self.generation.ensure_healthy();
        let retained_before = self.generation.registered_cache_bytes()?;
        self.generation.probe_cache(None)?;
        let removed_bytes = match Self::remove_selected_entry(
            &self.generation,
            &mut self.entries,
            token.resource_identity,
        ) {
            Ok(bytes) => bytes,
            Err(fault) => return self.fail_accounting(fault),
        };
        if self.accounting_fault.is_some() {
            return Ok(());
        }
        generation_health_before?;
        self.accounting_after_removal(retained_before, removed_bytes)?;
        if cpu_was_healthy {
            self.cpu.ensure_healthy()?;
        }
        Ok(())
    }

    fn return_entry_to_idle(
        generation: &Generation,
        _resource_identity: ResourceIdentity,
        entry: &mut ResourceEntry,
        frame: FrameIdentity,
    ) -> Result<()> {
        entry.state = ResourceEntryState::Idle {
            last_used_frame: frame,
        };
        let transition = entry.registration.set_idle(true);
        if !entry.resolved {
            generation.lease_return(entry.key.generation_class());
        }
        entry.resolved = false;
        entry.provisional = false;
        transition
    }

    fn cleanup_frame(
        &mut self,
        manager_identity: &ManagerIdentity,
        frame: FrameIdentity,
    ) -> FrameCleanup {
        if self.identity != *manager_identity {
            return FrameCleanup::default();
        }
        let Some(scratch) = self.active_frames.remove(&frame) else {
            return FrameCleanup::default();
        };
        let mut transition_failed = false;
        for (identity, entry) in self.entries.iter_mut() {
            let _ = self.generation.probe(entry.key.generation_class(), None);
            if entry.state == (ResourceEntryState::Leased { frame })
                && Self::return_entry_to_idle(&self.generation, *identity, entry, frame).is_err()
            {
                transition_failed = true;
            }
        }
        if transition_failed {
            self.record_accounting_fault(ResourceAccountingFault::GenerationAccountingFailure);
        }
        self.trim_idle(scratch)
    }

    fn abort_provisional_frame(
        &mut self,
        manager_identity: &ManagerIdentity,
        frame: FrameIdentity,
    ) -> FrameCleanup {
        if self.identity != *manager_identity || !self.active_frames.contains_key(&frame) {
            return FrameCleanup::default();
        }
        let mut transition_failed = false;
        for (identity, entry) in self.entries.iter_mut() {
            let _ = self.generation.probe(entry.key.generation_class(), None);
            if entry.state == (ResourceEntryState::Leased { frame })
                && !entry.provisional
                && Self::return_entry_to_idle(&self.generation, *identity, entry, frame).is_err()
            {
                transition_failed = true;
            }
        }
        if transition_failed {
            self.record_accounting_fault(ResourceAccountingFault::GenerationAccountingFailure);
        }
        self.discard_frame(manager_identity, frame)
    }

    fn discard_frame(
        &mut self,
        manager_identity: &ManagerIdentity,
        frame: FrameIdentity,
    ) -> FrameCleanup {
        if self.identity != *manager_identity {
            return FrameCleanup::default();
        }
        let Some(mut scratch) = self.active_frames.remove(&frame) else {
            return FrameCleanup::default();
        };
        scratch.indices.clear();
        for (identity, entry) in self.entries.iter() {
            self.cpu
                .at(RenderCpuScope::Cleanup)
                .maintenance(RenderWork::ResourceProbes, 1);
            if entry.state == (ResourceEntryState::Leased { frame })
                && scratch.push_index(0, *identity).is_err()
            {
                self.accounting_fault
                    .get_or_insert(ResourceAccountingFault::GenerationAccountingFailure);
                break;
            }
        }
        let FrameScratch {
            indices,
            evicted,
            cpu,
        } = scratch;
        let mut cleanup = FrameCleanup::with_reserved(evicted, cpu);
        for &(_, identity) in indices.iter() {
            let generation_health_before = self.generation.ensure_healthy();
            let retained_before = self.generation.registered_cache_bytes();
            let _ = self.generation.probe_cache(None);
            let removed_bytes =
                match Self::remove_selected_entry(&self.generation, &mut self.entries, identity) {
                    Ok(bytes) => bytes,
                    Err(fault) => {
                        if generation_health_before.is_err() || retained_before.is_err() {
                            self.record_accounting_fault(
                                ResourceAccountingFault::GenerationAccountingFailure,
                            );
                        }
                        self.record_accounting_fault(fault);
                        continue;
                    }
                };
            if cleanup.push_reserved(identity).is_err() {
                self.record_accounting_fault(ResourceAccountingFault::GenerationAccountingFailure);
            }
            match (generation_health_before, retained_before) {
                (Ok(()), Ok(bytes)) => {
                    let _ = self.accounting_after_removal(bytes, removed_bytes);
                }
                _ => self
                    .record_accounting_fault(ResourceAccountingFault::GenerationAccountingFailure),
            }
        }
        drop(indices);
        self.complete_cleanup(cleanup)
    }

    fn complete_cleanup(&mut self, mut cleanup: FrameCleanup) -> FrameCleanup {
        if let Some(fault) = self.accounting_fault {
            cleanup.retention = ResourceRetentionOutcome::AccountingFault { fault };
            return cleanup;
        }
        if self.ensure_accounting_exact(None).is_err() {
            let fault = self
                .accounting_fault
                .unwrap_or(ResourceAccountingFault::GenerationAccountingFailure);
            self.record_accounting_fault(fault);
            cleanup.retention = ResourceRetentionOutcome::AccountingFault { fault };
            return cleanup;
        }
        let Some((retained_count, retained_byte_len)) = self.checked_idle_summary() else {
            let fault = ResourceAccountingFault::SurvivingEntryByteTotalOverflow;
            self.record_accounting_fault(fault);
            cleanup.retention = ResourceRetentionOutcome::AccountingFault { fault };
            return cleanup;
        };
        cleanup.retention = if cleanup.evicted_resources.is_empty() {
            if retained_count == 0 {
                ResourceRetentionOutcome::NoIdleResources
            } else {
                ResourceRetentionOutcome::RetainedReusable {
                    resource_count: retained_count,
                    byte_len: retained_byte_len,
                }
            }
        } else {
            ResourceRetentionOutcome::Trimmed {
                released_count: cleanup.evicted_resources.len(),
                retained_count,
                retained_byte_len,
            }
        };
        cleanup
    }

    pub(super) fn trim_idle(&mut self, mut scratch: FrameScratch) -> FrameCleanup {
        scratch.indices.clear();
        if self.ensure_accounting_exact(None).is_err() {
            return self
                .complete_cleanup(FrameCleanup::with_reserved(scratch.evicted, scratch.cpu));
        }
        let mut probe_failed = false;
        for (identity, entry) in self.entries.iter() {
            if self
                .generation
                .probe(entry.key.generation_class(), None)
                .is_err()
            {
                probe_failed = true;
                break;
            }
            if let ResourceEntryState::Idle { last_used_frame } = entry.state
                && scratch.push_index(last_used_frame.0, *identity).is_err()
            {
                self.accounting_fault
                    .get_or_insert(ResourceAccountingFault::GenerationAccountingFailure);
                break;
            }
        }
        if probe_failed {
            return self
                .complete_cleanup(FrameCleanup::with_reserved(scratch.evicted, scratch.cpu));
        }
        let cpu = scratch.cpu.clone();
        scratch.indices.sort_unstable_by(|a, b| {
            cpu.maintenance(RenderWork::ResourceProbes, 1);
            a.cmp(b)
        });
        let FrameScratch {
            indices,
            evicted,
            cpu,
        } = scratch;
        let mut cleanup = FrameCleanup::with_reserved(evicted, cpu);
        for &(_, identity) in indices.iter() {
            match self.generation.cache_bytes() {
                Ok(bytes) if bytes <= self.budget.bytes() => break,
                Ok(_) => {}
                Err(_) => {
                    self.record_accounting_fault(
                        ResourceAccountingFault::GenerationAccountingFailure,
                    );
                    break;
                }
            }
            let _ = self.generation.probe_cache(None);
            if let Err(fault) =
                Self::remove_selected_entry(&self.generation, &mut self.entries, identity)
            {
                self.record_accounting_fault(fault);
                continue;
            }
            if cleanup.push_reserved(identity).is_err() {
                self.record_accounting_fault(ResourceAccountingFault::GenerationAccountingFailure);
            }
        }
        drop(indices);
        self.complete_cleanup(cleanup)
    }
}

pub(super) struct ResourceLeaseToken {
    pub(super) manager_identity: ManagerIdentity,
    pub(super) frame_identity: FrameIdentity,
    pub(super) resource_identity: ResourceIdentity,
    pub(super) allocation_generation: AllocationGeneration,
}

/// A production lease is intentionally neither `Clone` nor `Copy`; every
/// resolving operation consumes it.
#[must_use = "resource leases must be resolved by their owning frame scope"]
pub(crate) struct ResourceLease {
    pub(super) token: ResourceLeaseToken,
    payload: ResourcePayload,
}

impl ResourceLease {
    fn new(
        manager_identity: ManagerIdentity,
        frame_identity: FrameIdentity,
        resource_identity: ResourceIdentity,
        allocation_generation: AllocationGeneration,
        payload: ResourcePayload,
    ) -> Self {
        Self {
            token: ResourceLeaseToken {
                manager_identity,
                frame_identity,
                resource_identity,
                allocation_generation,
            },
            payload,
        }
    }

    pub(crate) const fn resource_identity(&self) -> ResourceIdentity {
        self.token.resource_identity
    }
}

impl fmt::Debug for ResourceLease {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ResourceLease")
            .field("manager_identity", &self.token.manager_identity)
            .field("frame_identity", &self.token.frame_identity)
            .field("resource_identity", &self.token.resource_identity)
            .field("allocation_generation", &self.token.allocation_generation)
            .field("payload", &self.payload.label())
            .finish()
    }
}

/// Reserved before native creation, shared through the manager's active-frame
/// registry so another nested scope cannot invalidate an older scope's bound.
pub(super) struct FrameScratch {
    pub(super) indices: AccountedVec<(u64, ResourceIdentity)>,
    evicted: AccountedVec<ResourceIdentity>,
    cpu: CpuAccount,
}
impl FrameScratch {
    pub(super) fn new(cpu: &CpuAccount, count: usize) -> Result<Self> {
        let cpu = cpu.at(RenderCpuScope::Cleanup);
        let storage = StorageAccount::Renderer(cpu.clone());
        Ok(Self {
            indices: AccountedVec::new_in(&storage, count, RenderResource::ScratchCpuBytes)?,
            evicted: AccountedVec::new_in(&storage, count, RenderResource::ScratchCpuBytes)?,
            cpu,
        })
    }
    fn reserve(&mut self, count: usize) -> Result<()> {
        self.indices
            .grow(count.saturating_sub(self.indices.len()))?;
        self.evicted
            .grow(count.saturating_sub(self.evicted.len()))?;
        Ok(())
    }
    pub(super) fn push_index(&mut self, order: u64, identity: ResourceIdentity) -> Result<()> {
        if self.indices.len() == self.indices.capacity() {
            self.cpu.note_fault(RenderAccountingError::Invariant);
            return Err(RenderAccountingError::Invariant.into());
        }
        self.cpu.maintenance(RenderWork::ResourceProbes, 1);
        self.indices
            .insert_slot(self.indices.len())?
            .write((order, identity));
        Ok(())
    }
}

#[must_use = "frame resource scopes clean outstanding leases when resolved or dropped"]
pub(crate) struct FrameResourceScope {
    pub(super) state: Arc<Mutex<ResourceManagerState>>,
    pub(super) manager_identity: ManagerIdentity,
    pub(super) frame: FrameIdentity,
    generation: Generation,
    pub(super) observer: Option<Account>,
    pending_drop_disposition: Option<FrameResourceDisposition>,
}

#[derive(Clone, Copy)]
enum FrameResourceDisposition {
    ReleaseReusable,
    AbortProvisional,
    Discard,
}

impl FrameResourceScope {
    pub(super) fn new(
        state: Arc<Mutex<ResourceManagerState>>,
        manager_identity: ManagerIdentity,
        frame: FrameIdentity,
        generation: Generation,
        observer: Option<Account>,
    ) -> Self {
        Self {
            state,
            manager_identity,
            frame,
            generation,
            observer,
            pending_drop_disposition: Some(FrameResourceDisposition::ReleaseReusable),
        }
    }

    fn lease_owner(&self) -> FrameLeaseOwner<'_> {
        FrameLeaseOwner {
            manager_identity: &self.manager_identity,
            frame: self.frame,
        }
    }

    pub(crate) fn generation(&self) -> &Generation {
        &self.generation
    }

    pub(crate) fn storage(&self) -> StorageAccount {
        StorageAccount::Renderer(lock_state(&self.state).cpu.at(RenderCpuScope::FrameBacking))
    }
    pub(crate) fn probe(&self) -> Result<()> {
        self.generation.probe_cache(self.observer.as_ref())
    }

    fn validated_payload<'scope>(
        &'scope self,
        lease: &'scope ResourceLease,
    ) -> Result<&'scope ResourcePayload> {
        lock_state(&self.state).validate_lease(
            &self.manager_identity,
            self.frame,
            &lease.token,
            self.observer.as_ref(),
        )?;
        Ok(&lease.payload)
    }

    pub(crate) fn vello_buffer<'scope>(
        &'scope self,
        lease: &'scope ResourceLease,
    ) -> Result<&'scope wgpu::Buffer> {
        match self.validated_payload(lease)? {
            ResourcePayload::VelloBuffer { buffer } => Ok(buffer),
            _ => Err(Error::invalid_value(
                "resource lease payload",
                lease.resource_identity().get(),
                "must contain an internal Vello buffer",
            )),
        }
    }

    pub(crate) fn vello_image<'scope>(
        &'scope self,
        lease: &'scope ResourceLease,
    ) -> Result<(&'scope wgpu::Texture, &'scope wgpu::TextureView)> {
        match self.validated_payload(lease)? {
            ResourcePayload::VelloImage { texture, view } => Ok((texture, view)),
            _ => Err(Error::invalid_value(
                "resource lease payload",
                lease.resource_identity().get(),
                "must contain an internal Vello image",
            )),
        }
    }

    pub(crate) fn acquire_vello_buffer(
        &mut self,
        device: &wgpu::Device,
        key: VelloBufferKey,
    ) -> Result<ResourceLease> {
        self.acquire_vello_buffer_with_reuse(device, key, IdleReuse::Fresh)
    }

    pub(crate) fn acquire_reusable_vello_buffer(
        &mut self,
        device: &wgpu::Device,
        key: VelloBufferKey,
    ) -> Result<ResourceLease> {
        self.acquire_vello_buffer_with_reuse(device, key, IdleReuse::Allowed)
    }

    fn acquire_vello_buffer_with_reuse(
        &mut self,
        device: &wgpu::Device,
        key: VelloBufferKey,
        idle_reuse: IdleReuse,
    ) -> Result<ResourceLease> {
        let byte_len = key.byte_len();
        lock_state(&self.state).acquire_with_payload(
            self.lease_owner(),
            ResourceCacheKey::VelloBuffer(key),
            byte_len,
            idle_reuse,
            self.observer.as_ref(),
            |generation| {
                let buffer = generation.allocation(
                    GenerationClass::CacheVello,
                    byte_len,
                    self.observer.as_ref(),
                    || {
                        device.create_buffer(&wgpu::BufferDescriptor {
                            label: Some("Surgeist internal Vello buffer"),
                            size: byte_len,
                            usage: key.usage(),
                            mapped_at_creation: false,
                        })
                    },
                )?;
                Ok(ResourcePayload::VelloBuffer { buffer })
            },
        )
    }

    pub(crate) fn acquire_vello_image(
        &mut self,
        device: &wgpu::Device,
        key: VelloImageKey,
        byte_len: u64,
    ) -> Result<ResourceLease> {
        self.acquire_vello_image_with_reuse(device, key, byte_len, IdleReuse::Fresh)
    }

    pub(crate) fn acquire_reusable_vello_image(
        &mut self,
        device: &wgpu::Device,
        key: VelloImageKey,
        byte_len: u64,
    ) -> Result<ResourceLease> {
        self.acquire_vello_image_with_reuse(device, key, byte_len, IdleReuse::Allowed)
    }

    fn acquire_vello_image_with_reuse(
        &mut self,
        device: &wgpu::Device,
        key: VelloImageKey,
        byte_len: u64,
        idle_reuse: IdleReuse,
    ) -> Result<ResourceLease> {
        lock_state(&self.state).acquire_with_payload(
            self.lease_owner(),
            ResourceCacheKey::VelloImage(key),
            byte_len,
            idle_reuse,
            self.observer.as_ref(),
            |generation| {
                let format = key.texture_format();
                let extent = key.extent();
                let texture = generation.allocation(
                    GenerationClass::CacheVello,
                    byte_len,
                    self.observer.as_ref(),
                    || {
                        device.create_texture(&wgpu::TextureDescriptor {
                            label: Some("Surgeist internal Vello image"),
                            size: wgpu::Extent3d {
                                width: extent.width(),
                                height: extent.height(),
                                depth_or_array_layers: 1,
                            },
                            mip_level_count: 1,
                            sample_count: 1,
                            dimension: wgpu::TextureDimension::D2,
                            format,
                            usage: key.usage(),
                            view_formats: &[],
                        })
                    },
                )?;
                let view = texture.create_view(
                    &wgpu::TextureViewDescriptor {
                        label: Some("Surgeist internal Vello image view"),
                        format: Some(format),
                        dimension: Some(wgpu::TextureViewDimension::D2),
                        usage: None,
                        aspect: wgpu::TextureAspect::All,
                        base_mip_level: 0,
                        mip_level_count: None,
                        base_array_layer: 0,
                        array_layer_count: None,
                    },
                    self.observer.as_ref(),
                )?;
                Ok(ResourcePayload::VelloImage { texture, view })
            },
        )
    }

    pub(crate) fn effect_texture<'scope>(
        &'scope self,
        lease: &'scope ResourceLease,
    ) -> Result<(&'scope wgpu::Texture, &'scope wgpu::TextureView)> {
        match self.validated_payload(lease)? {
            ResourcePayload::EffectTexture { texture, view } => Ok((texture, view)),
            _ => Err(Error::invalid_value(
                "resource lease payload",
                lease.resource_identity().get(),
                "must contain an effect texture",
            )),
        }
    }

    pub(crate) fn resolved_mask_texture<'scope>(
        &'scope self,
        lease: &'scope ResourceLease,
    ) -> Result<(&'scope wgpu::Texture, &'scope wgpu::TextureView)> {
        match self.validated_payload(lease)? {
            ResourcePayload::ResolvedMaskUpload { texture, view } => Ok((texture, view)),
            _ => Err(Error::invalid_value(
                "resource lease payload",
                lease.resource_identity().get(),
                "must contain a resolved-mask texture",
            )),
        }
    }

    pub(crate) fn gaussian_kernel_buffer<'scope>(
        &'scope self,
        lease: &'scope ResourceLease,
    ) -> Result<&'scope wgpu::Buffer> {
        match self.validated_payload(lease)? {
            ResourcePayload::GaussianKernelBuffer { buffer } => Ok(buffer),
            _ => Err(Error::invalid_value(
                "resource lease payload",
                lease.resource_identity().get(),
                "must contain a Gaussian kernel buffer",
            )),
        }
    }

    pub(crate) fn acquire_effect_texture(
        &mut self,
        device: &wgpu::Device,
        capabilities: &DeviceCapabilities,
        descriptor: EffectTextureDescriptor,
    ) -> Result<ResourceLease> {
        capabilities.validate_effect_texture_allocation(
            descriptor.physical_size(),
            descriptor.working_format(),
            descriptor.texture_format(),
            descriptor.usage(),
        )?;
        let byte_len = descriptor.checked_byte_len()?;
        let key = ResourceCacheKey::EffectTexture(descriptor.cache_key());
        lock_state(&self.state).acquire_with_payload(
            self.lease_owner(),
            key,
            byte_len,
            IdleReuse::Allowed,
            self.observer.as_ref(),
            |generation| {
                let texture = generation.allocation(
                    GenerationClass::CacheEffects,
                    byte_len,
                    self.observer.as_ref(),
                    || {
                        device.create_texture(&wgpu::TextureDescriptor {
                            label: Some(descriptor.label()),
                            size: wgpu::Extent3d {
                                width: descriptor.physical_size().width(),
                                height: descriptor.physical_size().height(),
                                depth_or_array_layers: 1,
                            },
                            mip_level_count: 1,
                            sample_count: 1,
                            dimension: wgpu::TextureDimension::D2,
                            format: descriptor.texture_format(),
                            usage: descriptor.usage(),
                            view_formats: &[],
                        })
                    },
                )?;
                let view = texture.create_view(
                    &wgpu::TextureViewDescriptor::default(),
                    self.observer.as_ref(),
                )?;
                Ok(ResourcePayload::EffectTexture { texture, view })
            },
        )
    }

    pub(crate) fn acquire_resolved_mask_upload(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        capabilities: &DeviceCapabilities,
        descriptor: &ResolvedMaskUploadDescriptor,
    ) -> Result<ResourceLease> {
        let physical_size = descriptor.physical_size();
        if physical_size.width() == 0 || physical_size.height() == 0 {
            return Err(Error::invalid_value(
                "resolved mask upload extent",
                format!("{}x{}", physical_size.width(), physical_size.height()),
                "must have positive width and height before GPU allocation",
            ));
        }
        descriptor.validate_upload_byte_len(descriptor.bytes().len())?;
        let usage = wgpu::TextureUsages::TEXTURE_BINDING.union(wgpu::TextureUsages::COPY_DST);
        capabilities.validate_effect_texture_allocation(
            physical_size,
            None,
            wgpu::TextureFormat::Rgba8Unorm,
            usage,
        )?;
        let key = ResourceCacheKey::ResolvedMaskUpload(
            descriptor.cache_key_with_cpu(&lock_state(&self.state).cpu, self.observer.as_ref())?,
        );
        lock_state(&self.state).acquire_with_payload(
            self.lease_owner(),
            key,
            descriptor.byte_len(),
            IdleReuse::Allowed,
            self.observer.as_ref(),
            |generation| {
                let texture = generation.allocation(
                    GenerationClass::CacheMasks,
                    descriptor.byte_len(),
                    self.observer.as_ref(),
                    || {
                        device.create_texture(&wgpu::TextureDescriptor {
                            label: Some("Surgeist retained resolved mask upload"),
                            size: wgpu::Extent3d {
                                width: physical_size.width(),
                                height: physical_size.height(),
                                depth_or_array_layers: 1,
                            },
                            mip_level_count: 1,
                            sample_count: 1,
                            dimension: wgpu::TextureDimension::D2,
                            format: wgpu::TextureFormat::Rgba8Unorm,
                            usage,
                            view_formats: &[],
                        })
                    },
                )?;
                generation.upload(
                    GenerationClass::CacheMasks,
                    self.observer.as_ref(),
                    descriptor.bytes().len(),
                )?;
                queue.write_texture(
                    wgpu::TexelCopyTextureInfo {
                        texture: &texture,
                        mip_level: 0,
                        origin: wgpu::Origin3d::ZERO,
                        aspect: wgpu::TextureAspect::All,
                    },
                    descriptor.bytes(),
                    wgpu::TexelCopyBufferLayout {
                        offset: 0,
                        bytes_per_row: Some(descriptor.row_bytes()),
                        rows_per_image: None,
                    },
                    wgpu::Extent3d {
                        width: physical_size.width(),
                        height: physical_size.height(),
                        depth_or_array_layers: 1,
                    },
                );
                let view = texture.create_view(
                    &wgpu::TextureViewDescriptor::default(),
                    self.observer.as_ref(),
                )?;
                Ok(ResourcePayload::ResolvedMaskUpload { texture, view })
            },
        )
    }

    pub(crate) fn acquire_gaussian_kernel_buffer(
        &mut self,
        device: &wgpu::Device,
        plan: &GaussianKernelPlan,
    ) -> Result<ResourceLease> {
        plan.validate_upload_byte_len(plan.upload_bytes().len())?;
        plan.validate_buffer_limits(GaussianKernelBufferLimits::from_device_limits(
            &device.limits(),
        ))?;
        let byte_len = plan.byte_len();
        let key = ResourceCacheKey::GaussianKernelBuffer(plan.key());
        lock_state(&self.state).acquire_with_payload(
            self.lease_owner(),
            key,
            byte_len,
            IdleReuse::Allowed,
            self.observer.as_ref(),
            |generation| {
                let buffer = generation.allocation_with_upload(
                    GenerationClass::CacheKernels,
                    byte_len,
                    plan.upload_bytes().len(),
                    self.observer.as_ref(),
                    || {
                        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some("Surgeist immutable Gaussian kernel buffer"),
                            contents: plan.upload_bytes(),
                            usage: wgpu::BufferUsages::STORAGE,
                        })
                    },
                )?;
                Ok(ResourcePayload::GaussianKernelBuffer { buffer })
            },
        )
    }

    pub(crate) fn release(&mut self, lease: ResourceLease) -> Result<()> {
        lock_state(&self.state).release(&self.manager_identity, self.frame, lease.token)
    }

    pub(crate) fn resolve_leases_atomically(&mut self, leases: &[&ResourceLease]) -> Result<()> {
        let mut state = lock_state(&self.state);
        let cpu = state.cpu.at(RenderCpuScope::Cleanup);
        cleanup_phase(&cpu, || {
            state.resolve_leases_atomically(&self.manager_identity, self.frame, leases)
        })
    }

    pub(crate) fn record_vello_atlas_recovery(&mut self, outcome: VelloAtlasOutcome) {
        lock_state(&self.state).record_vello_atlas_recovery(outcome);
    }

    pub(crate) fn consume_vello_atlas_recovery(&mut self) {
        lock_state(&self.state).consume_vello_atlas_recovery();
    }

    pub(crate) fn retire_idle_vello_atlases(&mut self) -> Result<()> {
        let mut state = lock_state(&self.state);
        let cpu = state.cpu.at(RenderCpuScope::Cleanup);
        cleanup_phase(&cpu, || state.retire_idle_vello_atlases(self.frame))
    }

    /// Removes the exact validated lease even when accounting becomes faulted.
    /// A newly detected fault returns a bounded error after removal; cleanup
    /// under an existing fault succeeds without replacing its first diagnostic.
    pub(crate) fn discard(&mut self, lease: ResourceLease) -> Result<()> {
        lock_state(&self.state).discard(&self.manager_identity, self.frame, lease.token)
    }

    pub(crate) fn discard_on_drop(&mut self) {
        if self.pending_drop_disposition.is_some() {
            self.pending_drop_disposition = Some(FrameResourceDisposition::Discard);
        }
    }

    pub(crate) fn abort_provisional_on_drop(&mut self) {
        if self.pending_drop_disposition.is_some() {
            self.pending_drop_disposition = Some(FrameResourceDisposition::AbortProvisional);
        }
    }

    pub(crate) fn finish(mut self) -> FrameCleanup {
        self.resolve(FrameResourceDisposition::ReleaseReusable)
    }

    pub(crate) fn ensure_commit_ready(&self, leases: &[&ResourceLease]) -> Result<()> {
        self.ensure_commit_ready_iter(leases.iter().copied())
    }
    pub(crate) fn ensure_commit_ready_iter<'a>(
        &self,
        leases: impl Iterator<Item = &'a ResourceLease>,
    ) -> Result<()> {
        let mut state = lock_state(&self.state);
        let cpu = state.cpu.at(RenderCpuScope::Cleanup);
        cleanup_phase(&cpu, || {
            state.ensure_frame_resolution_ready(
                &self.manager_identity,
                self.frame,
                leases.map(|lease| &lease.token),
            )
        })
    }

    pub(crate) fn finish_checked(mut self) -> Result<FrameCleanup> {
        self.ensure_commit_ready(&[])?;
        self.resolve(FrameResourceDisposition::ReleaseReusable)
            .into_accounting_result()
    }

    fn resolve(&mut self, disposition: FrameResourceDisposition) -> FrameCleanup {
        if self.pending_drop_disposition.take().is_none() {
            return FrameCleanup::default();
        }
        let mut state = lock_state(&self.state);
        let cpu = state.cpu.at(RenderCpuScope::Cleanup);
        cleanup_phase(&cpu, || match disposition {
            FrameResourceDisposition::ReleaseReusable => {
                state.cleanup_frame(&self.manager_identity, self.frame)
            }
            FrameResourceDisposition::AbortProvisional => {
                state.abort_provisional_frame(&self.manager_identity, self.frame)
            }
            FrameResourceDisposition::Discard => {
                state.discard_frame(&self.manager_identity, self.frame)
            }
        })
    }
}

impl Drop for FrameResourceScope {
    fn drop(&mut self) {
        if let Some(disposition) = self.pending_drop_disposition {
            let _ = self.resolve(disposition);
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) enum ResourceRetentionOutcome {
    #[default]
    NoIdleResources,
    RetainedReusable {
        resource_count: usize,
        byte_len: u64,
    },
    Trimmed {
        released_count: usize,
        retained_count: usize,
        retained_byte_len: u64,
    },
    AccountingFault {
        fault: ResourceAccountingFault,
    },
}

/// An empty cleanup has no allocation; an admitted cleanup keeps its
/// vector and credit coupled, including through mandatory infallible cleanup.
#[derive(Default)]
pub(super) enum CleanupBuffer {
    #[default]
    Empty,
    Accounted(AccountedVec<ResourceIdentity>),
}
impl CleanupBuffer {
    fn as_slice(&self) -> &[ResourceIdentity] {
        match self {
            Self::Empty => &[],
            Self::Accounted(values) => values.as_slice(),
        }
    }
    fn capacity(&self) -> usize {
        match self {
            Self::Empty => 0,
            Self::Accounted(values) => values.capacity(),
        }
    }
    fn clear(&mut self) {
        if let Self::Accounted(values) = self {
            values.clear();
        }
    }
    fn append_from(&mut self, source: &mut Self) -> Result<()> {
        match self {
            Self::Empty if source.is_empty() => Ok(()),
            Self::Empty => Err(crate::accounting::StorageContractError::ReservedCapacity {
                needed: source.len(),
                capacity: 0,
            }
            .into()),
            Self::Accounted(values) => {
                values.copy_slot(source.as_slice())?.write();
                source.clear();
                Ok(())
            }
        }
    }
    fn rotate_right(&mut self, count: usize) -> Result<()> {
        match self {
            Self::Empty if count == 0 => Ok(()),
            Self::Empty => Err(crate::accounting::StorageContractError::InvalidPosition {
                index: count,
                len: 0,
            }
            .into()),
            Self::Accounted(values) => {
                if count > values.len() {
                    return Err(crate::accounting::StorageContractError::InvalidPosition {
                        index: count,
                        len: values.len(),
                    }
                    .into());
                }
                values.rotate_right(count);
                Ok(())
            }
        }
    }
}
impl std::ops::Deref for CleanupBuffer {
    type Target = [ResourceIdentity];
    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}
#[derive(Default)]
pub(crate) struct FrameCleanup {
    pub(super) evicted_resources: CleanupBuffer,
    cpu: Option<CpuAccount>,
    pub(super) retention: ResourceRetentionOutcome,
}
// AccountedVec already disposes its actual vector and credit together under the
// original Cleanup lifetime scope. An outer disposal callback would time it twice.
impl fmt::Debug for FrameCleanup {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FrameCleanup")
            .field("evicted_resources", &self.evicted_resources.as_slice())
            .field("retention", &self.retention)
            .finish()
    }
}
impl PartialEq for FrameCleanup {
    fn eq(&self, other: &Self) -> bool {
        self.evicted_resources.as_slice() == other.evicted_resources.as_slice()
            && self.retention == other.retention
    }
}
impl Eq for FrameCleanup {}
impl FrameCleanup {
    fn with_reserved(values: AccountedVec<ResourceIdentity>, cpu: CpuAccount) -> Self {
        Self {
            evicted_resources: CleanupBuffer::Accounted(values),
            cpu: Some(cpu),
            retention: ResourceRetentionOutcome::default(),
        }
    }
    fn capacity_error(&self) -> Error {
        if let Some(cpu) = &self.cpu {
            cpu.note_fault(RenderAccountingError::Invariant);
        }
        RenderAccountingError::Invariant.into()
    }
    fn push_reserved(&mut self, identity: ResourceIdentity) -> Result<()> {
        if self.evicted_resources.len() == self.evicted_resources.capacity() {
            return Err(self.capacity_error());
        }
        match &mut self.evicted_resources {
            CleanupBuffer::Empty => Err(self.capacity_error()),
            CleanupBuffer::Accounted(values) => {
                let slot = values.append_slot()?;
                if let Some(cpu) = &self.cpu {
                    cpu.maintenance(RenderWork::ResourceProbes, 1);
                }
                slot.write(identity);
                Ok(())
            }
        }
    }
    pub(crate) fn followed_by(self, later: Self) -> Result<Self> {
        let cpu = self.cpu.as_ref().or(later.cpu.as_ref()).cloned();
        match cpu {
            Some(cpu) => cleanup_phase(&cpu, || self.merge_reserved(later)),
            None => self.merge_reserved(later),
        }
    }
    fn merge_reserved(mut self, mut later: Self) -> Result<Self> {
        let count = self
            .evicted_resources
            .len()
            .checked_add(later.evicted_resources.len())
            .ok_or_else(|| self.capacity_error())?;
        let retention = match (self.retention, later.retention) {
            (ResourceRetentionOutcome::AccountingFault { fault }, _)
            | (_, ResourceRetentionOutcome::AccountingFault { fault }) => {
                ResourceRetentionOutcome::AccountingFault { fault }
            }
            (_, later) => later,
        };
        if count <= self.evicted_resources.capacity() {
            if !later.evicted_resources.is_empty()
                && let Some(cpu) = &self.cpu
            {
                cpu.maintenance(
                    RenderWork::ResourceProbes,
                    later.evicted_resources.len() as u64,
                );
            }
            self.evicted_resources
                .append_from(&mut later.evicted_resources)?;
            self.retention = retention;
            Ok(self)
        } else if count <= later.evicted_resources.capacity() {
            let old_count = self.evicted_resources.len();
            if old_count != 0
                && let Some(cpu) = &later.cpu
            {
                cpu.maintenance(RenderWork::ResourceProbes, old_count as u64);
            }
            later
                .evicted_resources
                .append_from(&mut self.evicted_resources)?;
            later.evicted_resources.rotate_right(old_count)?;
            later.retention = retention;
            Ok(later)
        } else {
            Err(self.capacity_error())
        }
    }

    pub(crate) fn into_accounting_result(self) -> Result<Self> {
        if matches!(
            self.retention,
            ResourceRetentionOutcome::AccountingFault { .. }
        ) {
            return Err(ResourceManagerState::accounting_fault_error());
        }
        Ok(self)
    }
}

#[cfg(test)]
mod proof_tests;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        PhysicalSize, RenderResource, RenderWork, ResourceCacheBudget, resource::ResourceManager,
    };

    fn effect_key(width: u32) -> ResourceCacheKey {
        ResourceCacheKey::EffectTexture(
            EffectTextureDescriptor::try_capture(
                PhysicalSize::new(width, 1),
                wgpu::TextureUsages::TEXTURE_BINDING,
            )
            .unwrap()
            .cache_key(),
        )
    }

    #[test]
    fn frame_discard_continues_after_the_first_retained_byte_fault() {
        use crate::accounting::{RenderAccountingOptions, RenderProfile, RendererCpu};

        let options =
            RenderAccountingOptions::new().with_profile(RenderProfile::Summary { timing: false });
        let cpu = RendererCpu::new(options).unwrap();
        let manager = ResourceManager::with_generation(
            ResourceCacheBudget::DEFAULT,
            Generation::new(options).unwrap(),
            cpu.at(RenderCpuScope::DeviceBacking),
        );
        let mut discarded = manager.begin_frame(None).unwrap();
        let first = discarded.acquire(ResourceCacheKey::VelloAtlas, 4).unwrap();
        let mut unrelated = manager.begin_frame(None).unwrap();
        let survivor = unrelated.acquire(ResourceCacheKey::VelloAtlas, 4).unwrap();
        let last = discarded.acquire(ResourceCacheKey::VelloAtlas, 4).unwrap();
        let generation = manager.generation();
        let before = generation.class_usage(GenerationClass::CacheVello).unwrap();
        let probes = generation.cache_work(RenderWork::ResourceProbes).unwrap();
        let cleanup_probes = || {
            cpu.snapshot()
                .unwrap()
                .scope_usage(RenderCpuScope::Cleanup)
                .unwrap()
                .work(RenderWork::ResourceProbes)
        };
        let cpu_before = cleanup_probes();
        manager.inject_retained_byte_mismatch_before_discard_for_test();

        let cleanup = discarded.resolve(FrameResourceDisposition::Discard);
        assert_eq!(
            cleanup.evicted_resources(),
            &[first.resource_identity(), last.resource_identity()]
        );
        let fault = ResourceAccountingFault::RetainedByteMismatch {
            retained_bytes: 7,
            registered_entry_bytes: 8,
        };
        assert_eq!(
            cleanup.retention(),
            ResourceRetentionOutcome::AccountingFault { fault }
        );
        let after = manager.observation_for_test();
        assert_eq!(after.accounting_fault_for_test(), Some(fault));
        assert_eq!(
            after.entry_identities_for_test(),
            &[survivor.resource_identity()]
        );
        assert_eq!(after.accounted_entry_bytes, Some(4));
        assert_eq!((after.active_frame_count, after.leased_count), (1, 1));
        assert_eq!(generation.registered_cache_bytes().unwrap(), 3);
        assert_eq!(generation.idle_cache_bytes().unwrap(), 0);
        assert_eq!(generation.cache_bytes().unwrap(), 12);
        drop((first, last));
        assert_eq!(generation.cache_bytes().unwrap(), 4);
        let after = generation.class_usage(GenerationClass::CacheVello).unwrap();
        assert_eq!(
            after.work(RenderWork::ResourceEvictions) - before.work(RenderWork::ResourceEvictions),
            2
        );
        assert_eq!(
            after.work(RenderWork::ResourceLeaseReturns),
            before.work(RenderWork::ResourceLeaseReturns)
        );
        // Only the first removal reconciles two remaining entries. Both removals
        // separately probe the cache, then publish their identities in order.
        assert_eq!(
            after.work(RenderWork::ResourceProbes) - before.work(RenderWork::ResourceProbes),
            2
        );
        assert_eq!(
            generation.cache_work(RenderWork::ResourceProbes).unwrap() - probes,
            4
        );
        // Three selection visits, two scratch insertions and two result appends.
        assert_eq!(cleanup_probes() - cpu_before, 7);

        unrelated.release(survivor).unwrap();
        assert_eq!(
            unrelated.finish().retention(),
            ResourceRetentionOutcome::AccountingFault { fault }
        );
    }

    #[test]
    fn idle_trim_accounts_for_only_the_returned_entries_and_keeps_the_active_survivor() {
        let manager = ResourceManager::new(ResourceCacheBudget::DISABLED);
        let mut finishing = manager.begin_frame(None).unwrap();
        let vello = finishing.acquire(ResourceCacheKey::VelloAtlas, 4).unwrap();
        let effect = finishing.acquire(effect_key(1), 4).unwrap();
        let mut unrelated = manager.begin_frame(None).unwrap();
        let survivor = unrelated.acquire(effect_key(1), 4).unwrap();
        let expected = [vello.resource_identity(), effect.resource_identity()];
        drop((vello, effect));
        let generation = manager.generation();
        let probes = generation.cache_work(RenderWork::ResourceProbes).unwrap();
        let vello_before = generation.class_usage(GenerationClass::CacheVello).unwrap();
        let effects_before = generation
            .class_usage(GenerationClass::CacheEffects)
            .unwrap();
        assert_eq!(generation.cache_bytes().unwrap(), 12);
        assert_eq!(generation.registered_cache_bytes().unwrap(), 12);
        assert_eq!(generation.idle_cache_bytes().unwrap(), 0);

        let cleanup = finishing.finish();
        assert_eq!(cleanup.evicted_resources(), &expected);
        assert_eq!(
            cleanup.retention(),
            ResourceRetentionOutcome::Trimmed {
                released_count: 2,
                retained_count: 0,
                retained_byte_len: 0
            }
        );
        let after = manager.observation_for_test();
        assert_eq!(
            after.entry_identities_for_test(),
            &[survivor.resource_identity()]
        );
        assert_eq!(after.accounted_entry_bytes, Some(4));
        assert_eq!((after.active_frame_count, after.leased_count), (1, 1));
        assert_eq!(after.accounting_fault_for_test(), None);
        assert_eq!(generation.cache_bytes().unwrap(), 4);
        assert_eq!(generation.registered_cache_bytes().unwrap(), 4);
        assert_eq!(generation.idle_cache_bytes().unwrap(), 0);
        // Return, pre-trim audit and selection inspect every entry. The final
        // audit and both summary walks each inspect the remaining effect.
        for (class, before, class_probes) in [
            (GenerationClass::CacheVello, vello_before, 3),
            (GenerationClass::CacheEffects, effects_before, 9),
        ] {
            let after = generation.class_usage(class).unwrap();
            assert_eq!(
                after.work(RenderWork::ResourceProbes) - before.work(RenderWork::ResourceProbes),
                class_probes
            );
            assert_eq!(
                after.work(RenderWork::ResourceEvictions)
                    - before.work(RenderWork::ResourceEvictions),
                1
            );
            assert_eq!(
                after.work(RenderWork::ResourceLeaseReturns)
                    - before.work(RenderWork::ResourceLeaseReturns),
                1
            );
        }
        assert_eq!(
            generation.cache_work(RenderWork::ResourceProbes).unwrap() - probes,
            3 + 9 + 2
        );
        let survivor_id = survivor.resource_identity();
        unrelated.release(survivor).unwrap();
        assert_eq!(unrelated.finish().evicted_resources(), &[survivor_id]);
        assert_eq!(generation.cache_bytes().unwrap(), 0);
    }

    #[test]
    fn cache_candidate_scans_charge_each_inspected_entry_to_its_own_class() {
        let manager = ResourceManager::default();
        let mut initial = manager.begin_frame(None).unwrap();
        let atlas = initial.acquire(ResourceCacheKey::VelloAtlas, 4).unwrap();
        let effect = initial.acquire(effect_key(1), 4).unwrap();
        initial.release(atlas).unwrap();
        initial.release(effect).unwrap();
        let _ = initial.finish();

        // The second allocation audits and scans the atlas (two visits).
        // Cleanup visits each entry six times: lease return, pre-trim audit,
        // idle selection, final audit, idle count and idle-byte summary.
        let probes = |class| {
            manager
                .generation()
                .class_usage(class)
                .unwrap()
                .work(RenderWork::ResourceProbes)
        };
        assert_eq!(probes(GenerationClass::CacheVello), 8);
        assert_eq!(probes(GenerationClass::CacheEffects), 6);

        let request = crate::resource::ResourceAllocationPreflight::effect_texture(
            EffectTextureDescriptor::try_capture(
                PhysicalSize::new(1, 1),
                wgpu::TextureUsages::TEXTURE_BINDING,
            )
            .unwrap(),
        )
        .unwrap();
        manager
            .preflight_graph_acquisitions(&[request.clone(), request], None)
            .unwrap();
        // Preflight first audits both entries, then each independent request
        // scans both candidates: three more visits per class. Selecting one
        // idle identity does not remove it from the next request's scan.
        assert_eq!(probes(GenerationClass::CacheVello), 11);
        assert_eq!(probes(GenerationClass::CacheEffects), 9);

        let mut reuse = manager.begin_frame(None).unwrap();
        let effect = reuse.acquire(effect_key(1), 4).unwrap();
        // Begin-frame audit, acquisition audit and candidate selection each
        // visit both entries. The later selected lookup has the cache scope.
        assert_eq!(probes(GenerationClass::CacheVello), 14);
        assert_eq!(probes(GenerationClass::CacheEffects), 12);
        reuse.release(effect).unwrap();
        let _ = reuse.finish();
        // Another six cleanup visits per entry.
        assert_eq!(probes(GenerationClass::CacheVello), 20);
        assert_eq!(probes(GenerationClass::CacheEffects), 18);

        let mut retirement = manager.begin_frame(None).unwrap();
        retirement.retire_idle_vello_atlases().unwrap();
        // Begin and retirement audits, atlas selection and the final audit
        // inspect both. The modeled VelloAtlas key is not a native persistent
        // image-atlas key, so this selection removes neither entry.
        assert_eq!(probes(GenerationClass::CacheVello), 24);
        assert_eq!(probes(GenerationClass::CacheEffects), 22);
    }

    #[test]
    fn graph_allocation_batch_preview_refuses_overflow_without_custody_or_peak_changes() {
        let manager = ResourceManager::default();
        let mut frame = manager.begin_frame(None).unwrap();
        let maximum = frame
            .acquire(ResourceCacheKey::VelloAtlas, u64::MAX - 4)
            .unwrap();
        let request = crate::resource::ResourceAllocationPreflight::effect_texture(
            EffectTextureDescriptor::try_capture(
                PhysicalSize::new(1, 1),
                wgpu::TextureUsages::TEXTURE_BINDING,
            )
            .unwrap(),
        )
        .unwrap();
        let error = manager
            .preflight_graph_acquisitions(&[request.clone(), request], None)
            .expect_err("two four-byte requests exceed the four remaining byte units");
        assert_eq!(error.code(), crate::ErrorCode::InvalidInput);
        assert_eq!(
            error
                .invalid_value_diagnostic()
                .map(crate::InvalidValue::field),
            Some("retained resource byte length")
        );
        assert_eq!(manager.generation().cache_bytes().unwrap(), u64::MAX - 4);
        assert_eq!(
            manager
                .generation()
                .usage()
                .unwrap()
                .peak(RenderResource::GpuBytes),
            u64::MAX - 4
        );
        assert_eq!(
            manager
                .generation()
                .cache_work(RenderWork::ResourceAllocations)
                .unwrap(),
            1
        );
        assert_eq!(
            manager
                .generation()
                .cache_work(RenderWork::ResourceAllocationRequests)
                .unwrap(),
            1
        );
        assert_eq!(manager.observation_for_test().entry_count, 1);
        assert_eq!(manager.observation_for_test().next_resource, 1);
        assert_eq!(manager.observation_for_test().payload_creation_attempts, 1);
        manager.generation().ensure_healthy().unwrap();
        frame.release(maximum).unwrap();
        let _ = frame.finish();
        assert_eq!(manager.generation().cache_bytes().unwrap(), 0);
    }

    #[test]
    fn managed_registration_and_idle_classification_follow_exact_lease_boundaries() {
        let manager = ResourceManager::new(ResourceCacheBudget::new(8));
        let generation = manager.generation();
        let interval = generation.begin_interval().unwrap();
        let mut first = manager.begin_frame(None).unwrap();
        let allocation = first.acquire(effect_key(2), 8).unwrap();
        let identity = allocation.resource_identity();
        assert_eq!(generation.cache_bytes().unwrap(), 8);
        assert_eq!(generation.registered_cache_bytes().unwrap(), 8);
        assert_eq!(generation.idle_cache_bytes().unwrap(), 0);
        first.release(allocation).unwrap();
        assert_eq!(generation.idle_cache_bytes().unwrap(), 0);
        assert_eq!(
            generation
                .cache_work(RenderWork::ResourceLeaseReturns)
                .unwrap(),
            1
        );
        let _ = first.finish();
        assert_eq!(generation.idle_cache_bytes().unwrap(), 8);

        let mut second = manager.begin_frame(None).unwrap();
        let reused = second.acquire(effect_key(2), 8).unwrap();
        assert_eq!(reused.resource_identity(), identity);
        assert_eq!(generation.idle_cache_bytes().unwrap(), 0);
        second.release(reused).unwrap();
        let _ = second.finish();
        let interval = interval.finish().unwrap();
        assert_eq!(interval.cache_allocations(), 1);
        assert_eq!(interval.cache_reuses(), 1);
        assert_eq!(interval.idle_cache_bytes(), 8);
        assert_eq!(generation.cache_bytes().unwrap(), 8);
    }

    #[test]
    fn removed_entry_clone_keeps_physical_custody_and_drives_idle_eviction() {
        let manager = ResourceManager::new(ResourceCacheBudget::new(4));
        let mut first = manager.begin_frame(None).unwrap();
        let removed = first.acquire(effect_key(2), 8).unwrap();
        let surviving_native = removed.payload.clone();
        first.discard(removed).unwrap();
        let _ = first.finish();
        assert_eq!(manager.generation().registered_cache_bytes().unwrap(), 0);
        assert_eq!(manager.generation().cache_bytes().unwrap(), 8);

        let mut second = manager.begin_frame(None).unwrap();
        let idle_candidate = second.acquire(effect_key(1), 4).unwrap();
        let identity = idle_candidate.resource_identity();
        second.release(idle_candidate).unwrap();
        let cleanup = second.finish();
        assert_eq!(cleanup.evicted_resources(), &[identity]);
        assert_eq!(manager.generation().registered_cache_bytes().unwrap(), 0);
        assert_eq!(manager.generation().idle_cache_bytes().unwrap(), 0);
        assert_eq!(manager.generation().cache_bytes().unwrap(), 8);
        drop(surviving_native);
        assert_eq!(manager.generation().cache_bytes().unwrap(), 0);
        manager.generation().ensure_healthy().unwrap();
    }

    #[test]
    fn simultaneous_private_scopes_evict_only_idle_entries_above_target() {
        let manager = ResourceManager::new(ResourceCacheBudget::new(4));
        let interval = manager.generation().begin_interval().unwrap();
        let mut first = manager.begin_frame(None).unwrap();
        let mut second = manager.begin_frame(None).unwrap();
        let smaller = first.acquire(effect_key(1), 4).unwrap();
        let larger = second.acquire(effect_key(2), 8).unwrap();
        let smaller_id = smaller.resource_identity();
        let larger_id = larger.resource_identity();
        assert_eq!(manager.generation().cache_bytes().unwrap(), 12);
        assert_eq!(manager.generation().idle_cache_bytes().unwrap(), 0);
        first.release(smaller).unwrap();
        assert_eq!(first.finish().evicted_resources(), &[smaller_id]);
        assert_eq!(manager.generation().cache_bytes().unwrap(), 8);
        assert_eq!(manager.live_count(), 1);
        second.release(larger).unwrap();
        assert_eq!(second.finish().evicted_resources(), &[larger_id]);
        let interval = interval.finish().unwrap();
        assert_eq!(interval.cache_allocations(), 2);
        assert_eq!(interval.cache_reuses(), 0);
        assert_eq!(interval.idle_cache_bytes(), 0);
        assert_eq!(manager.generation().cache_bytes().unwrap(), 0);
    }

    #[test]
    fn noncache_physical_bytes_do_not_evict_an_in_target_cache_entry() {
        let manager = ResourceManager::new(ResourceCacheBudget::new(4));
        let noncache = manager
            .generation()
            .allocation(GenerationClass::PassBuffers, 16, None, || ())
            .unwrap();
        let mut frame = manager.begin_frame(None).unwrap();
        let cached = frame.acquire(effect_key(1), 4).unwrap();
        frame.release(cached).unwrap();
        assert!(frame.finish().evicted_resources().is_empty());
        assert_eq!(manager.generation().cache_bytes().unwrap(), 4);
        assert_eq!(
            manager
                .generation()
                .usage()
                .unwrap()
                .live(RenderResource::GpuBytes),
            20
        );
        drop(noncache);
        assert_eq!(manager.generation().cache_bytes().unwrap(), 4);
        assert_eq!(
            manager
                .generation()
                .usage()
                .unwrap()
                .live(RenderResource::GpuBytes),
            4
        );
    }

    #[test]
    fn failed_provisional_payload_releases_physical_custody_without_registering_entry() {
        let manager = ResourceManager::default();
        let frame = manager.begin_frame(None).unwrap();
        let result = lock_state(&frame.state).acquire_with_payload(
            frame.lease_owner(),
            effect_key(1),
            4,
            IdleReuse::Fresh,
            None,
            |generation| {
                let _native =
                    generation.allocation(GenerationClass::CacheEffects, 4, None, || ())?;
                Err(Error::invalid_value(
                    "fixture payload",
                    "failure",
                    "must abort registration",
                ))
            },
        );
        assert!(result.is_err());
        assert_eq!(manager.generation().cache_bytes().unwrap(), 0);
        assert_eq!(manager.generation().registered_cache_bytes().unwrap(), 0);
        assert_eq!(
            manager
                .generation()
                .cache_work(RenderWork::ResourceAllocations)
                .unwrap(),
            1
        );
        assert_eq!(manager.observation_for_test().entry_count, 0);
        assert_eq!(manager.observation_for_test().next_resource, 0);
        assert_eq!(manager.observation_for_test().payload_creation_attempts, 1);
        drop(frame);
        manager.generation().ensure_healthy().unwrap();
    }

    #[test]
    fn retirement_with_a_surviving_cache_lease_releases_custody_once() {
        let manager = ResourceManager::new(ResourceCacheBudget::DISABLED);
        let mut frame = manager.begin_frame(None).unwrap();
        let lease = frame.acquire(effect_key(1), 4).unwrap();
        manager.generation().retire();
        assert_eq!(manager.generation().cache_bytes().unwrap(), 0);
        assert_eq!(manager.generation().registered_cache_bytes().unwrap(), 0);
        assert!(manager.begin_frame(None).is_err());
        frame.discard(lease).unwrap_err();
        drop(frame);
        manager.generation().retire();
        assert_eq!(manager.generation().cache_bytes().unwrap(), 0);
        assert_eq!(manager.generation().registered_cache_bytes().unwrap(), 0);
    }
}
