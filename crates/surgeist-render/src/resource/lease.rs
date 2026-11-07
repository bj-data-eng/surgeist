use super::{
    AllocationGeneration, FrameIdentity, ManagerIdentity, ResourceAccountingFault,
    ResourceCacheKey, ResourceIdentity,
    gaussian::{GaussianKernelBufferLimits, GaussianKernelPlan},
    lock_state,
    manager::{
        IdleReuse, ResourceEntry, ResourceEntryState, ResourceManagerState, ResourcePayload,
    },
};
use crate::{
    BackendErrorCode, Error, Result,
    accounting::{Generation, GenerationClass},
    backend::DeviceCapabilities,
    image::ResolvedMaskUploadDescriptor,
    texture::EffectTextureDescriptor,
    vello_engine::{VelloAtlasOutcome, VelloBufferKey, VelloImageKey},
};
use std::{
    collections::BTreeSet,
    fmt,
    sync::{Arc, Mutex},
};
use wgpu::util::DeviceExt;

impl ResourceManagerState {
    fn ensure_frame_resolution_ready(
        &mut self,
        manager_identity: &ManagerIdentity,
        frame: FrameIdentity,
        tokens: &[&ResourceLeaseToken],
    ) -> Result<()> {
        self.ensure_accounting_exact()?;
        if self.identity != *manager_identity || !self.active_frames.contains(&frame) {
            return Err(Error::new(
                BackendErrorCode::RenderFailed,
                "resource frame is unavailable for clean accounting resolution",
            ));
        }

        let mut covered = self
            .resolved_leases
            .iter()
            .filter_map(|(resolved_frame, resource)| {
                (*resolved_frame == frame).then_some(*resource)
            })
            .collect::<BTreeSet<_>>();
        for token in tokens {
            if !covered.insert(token.resource_identity) {
                return Err(Self::invalid_lease(
                    token.resource_identity,
                    "must occur exactly once in one clean frame resolution",
                ));
            }
            self.validate_lease(manager_identity, frame, token)?;
        }
        let leased = self
            .entries
            .iter()
            .filter_map(|(identity, entry)| {
                (entry.state == (ResourceEntryState::Leased { frame })).then_some(*identity)
            })
            .collect::<BTreeSet<_>>();
        if covered != leased {
            return Err(Error::new(
                BackendErrorCode::RenderFailed,
                "clean resource resolution must own every exact lease in its frame",
            ));
        }
        Ok(())
    }

    fn invalid_lease(resource: ResourceIdentity, reason: &'static str) -> Error {
        Error::invalid_value("resource lease", resource.get(), reason)
    }

    fn acquire_with_payload(
        &mut self,
        manager_identity: &ManagerIdentity,
        frame: FrameIdentity,
        key: ResourceCacheKey,
        byte_len: u64,
        idle_reuse: IdleReuse,
        create_payload: impl FnOnce(&Generation) -> Result<ResourcePayload>,
    ) -> Result<ResourceLease> {
        self.ensure_accounting_exact()?;
        if self.identity != *manager_identity || !self.active_frames.contains(&frame) {
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
            for (identity, entry) in &self.entries {
                self.generation.probe(entry.key.generation_class())?;
                if let ResourceEntryState::Idle { last_used_frame } = entry.state
                    && entry.key == key
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
            self.generation.reuse(key.generation_class())?;
            let entry = self
                .entries
                .get_mut(&resource)
                .expect("the selected idle resource must remain registered");
            debug_assert!(entry.payload.matches_key(&key));
            entry.registration.set_idle(false)?;
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
                .preview_cache_allocations([byte_len])
                .map_err(Self::allocation_preflight_error)?;
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
            let resource = ResourceIdentity(next_resource);
            let allocation_generation = AllocationGeneration(1);
            let registration = self
                .generation
                .cache_registration(key.generation_class(), byte_len)?;
            self.next_resource = next_resource;
            self.entries.insert(
                resource,
                ResourceEntry {
                    key,
                    allocation_generation,
                    byte_len,
                    state: ResourceEntryState::Leased { frame },
                    payload: payload.clone(),
                    registration,
                },
            );
            self.provisional_allocations.insert((frame, resource));
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
    ) -> Result<ResourceLease> {
        if !key.accepts_modeled_payload() {
            return Err(Error::invalid_value(
                "modeled resource key",
                format!("{key:?}"),
                "must use a Vello atlas or effect texture namespace",
            ));
        }
        self.acquire_with_payload(
            manager_identity,
            frame,
            key.clone(),
            byte_len,
            IdleReuse::Allowed,
            |generation| {
                Ok(ResourcePayload::Modeled {
                    _allocation: generation.allocation(key.generation_class(), byte_len, || ())?,
                })
            },
        )
    }

    fn validate_lease(
        &self,
        manager_identity: &ManagerIdentity,
        frame: FrameIdentity,
        token: &ResourceLeaseToken,
    ) -> Result<()> {
        if self.identity != *manager_identity || token.manager_identity != self.identity {
            return Err(Self::invalid_lease(
                token.resource_identity,
                "must belong to this resource manager",
            ));
        }
        if token.frame_identity != frame || !self.active_frames.contains(&frame) {
            return Err(Self::invalid_lease(
                token.resource_identity,
                "must belong to this active resource frame",
            ));
        }
        if self
            .resolved_leases
            .contains(&(frame, token.resource_identity))
        {
            return Err(Self::invalid_lease(
                token.resource_identity,
                "must not have already been resolved by this frame",
            ));
        }
        let Some(entry) = self.entries.get(&token.resource_identity) else {
            return Err(Self::invalid_lease(
                token.resource_identity,
                "must name a current resource identity",
            ));
        };
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
        self.validate_lease(manager_identity, frame, &token)?;
        self.resolved_leases
            .insert((frame, token.resource_identity));
        let entry = &self.entries[&token.resource_identity];
        self.generation.lease_return(entry.key.generation_class());
        Ok(())
    }

    fn resolve_leases_atomically(
        &mut self,
        manager_identity: &ManagerIdentity,
        frame: FrameIdentity,
        tokens: &[&ResourceLeaseToken],
    ) -> Result<()> {
        let mut resources = BTreeSet::new();
        for token in tokens {
            if !resources.insert(token.resource_identity) {
                return Err(Self::invalid_lease(
                    token.resource_identity,
                    "must occur at most once in one atomic release",
                ));
            }
            self.validate_lease(manager_identity, frame, token)?;
        }

        for token in tokens {
            self.resolved_leases
                .insert((frame, token.resource_identity));
            let entry = &self.entries[&token.resource_identity];
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
    ) -> Result<ResourceLease> {
        self.ensure_accounting_healthy()?;
        self.validate_lease(manager_identity, frame, &token)?;
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
        let Some(registered_entry_bytes) = self.checked_registered_entry_bytes() else {
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
            _allocation: self
                .generation
                .allocation(key.generation_class(), byte_len, || ())?,
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
        self.ensure_accounting_exact()
    }

    fn discard(
        &mut self,
        manager_identity: &ManagerIdentity,
        frame: FrameIdentity,
        token: ResourceLeaseToken,
    ) -> Result<()> {
        self.validate_lease(manager_identity, frame, &token)?;
        let generation_health_before = self.generation.ensure_healthy();
        let retained_before = self.generation.registered_cache_bytes()?;
        let entry = self
            .entries
            .remove(&token.resource_identity)
            .expect("a validated resource lease must remain registered");
        let removed_bytes = entry.byte_len;
        self.generation.eviction(entry.key.generation_class());
        drop(entry);
        if self.accounting_fault.is_some() {
            return Ok(());
        }
        generation_health_before?;
        self.accounting_after_removal(retained_before, removed_bytes)
    }

    fn return_entry_to_idle(
        generation: &Generation,
        resolved_leases: &mut BTreeSet<(FrameIdentity, ResourceIdentity)>,
        resource_identity: ResourceIdentity,
        entry: &mut ResourceEntry,
        frame: FrameIdentity,
    ) -> Result<()> {
        entry.state = ResourceEntryState::Idle {
            last_used_frame: frame,
        };
        let transition = entry.registration.set_idle(true);
        if !resolved_leases.remove(&(frame, resource_identity)) {
            generation.lease_return(entry.key.generation_class());
        }
        transition
    }

    fn cleanup_frame(
        &mut self,
        manager_identity: &ManagerIdentity,
        frame: FrameIdentity,
    ) -> FrameCleanup {
        if self.identity != *manager_identity || !self.active_frames.remove(&frame) {
            return FrameCleanup::default();
        }
        let mut transition_failed = false;
        for (identity, entry) in &mut self.entries {
            if entry.state == (ResourceEntryState::Leased { frame })
                && Self::return_entry_to_idle(
                    &self.generation,
                    &mut self.resolved_leases,
                    *identity,
                    entry,
                    frame,
                )
                .is_err()
            {
                transition_failed = true;
            }
        }
        self.provisional_allocations
            .retain(|(owner, _)| *owner != frame);
        if transition_failed {
            self.record_accounting_fault(ResourceAccountingFault::GenerationAccountingFailure);
        }
        self.trim_idle()
    }

    fn abort_provisional_frame(
        &mut self,
        manager_identity: &ManagerIdentity,
        frame: FrameIdentity,
    ) -> FrameCleanup {
        if self.identity != *manager_identity || !self.active_frames.contains(&frame) {
            return FrameCleanup::default();
        }
        let mut transition_failed = false;
        for (identity, entry) in &mut self.entries {
            if entry.state == (ResourceEntryState::Leased { frame })
                && !self.provisional_allocations.contains(&(frame, *identity))
                && Self::return_entry_to_idle(
                    &self.generation,
                    &mut self.resolved_leases,
                    *identity,
                    entry,
                    frame,
                )
                .is_err()
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
        if self.identity != *manager_identity || !self.active_frames.remove(&frame) {
            return FrameCleanup::default();
        }
        let discarded = self
            .entries
            .iter()
            .filter_map(|(identity, entry)| {
                (entry.state == (ResourceEntryState::Leased { frame })).then_some(*identity)
            })
            .collect::<Vec<_>>();
        let mut cleanup = FrameCleanup::default();
        for identity in discarded {
            let generation_health_before = self.generation.ensure_healthy();
            let retained_before = self.generation.registered_cache_bytes();
            let entry = self
                .entries
                .remove(&identity)
                .expect("selected frame resource must remain registered");
            let removed_bytes = entry.byte_len;
            self.generation.eviction(entry.key.generation_class());
            drop(entry);
            cleanup.evicted_resources.push(identity);
            match (generation_health_before, retained_before) {
                (Ok(()), Ok(bytes)) => {
                    let _ = self.accounting_after_removal(bytes, removed_bytes);
                }
                _ => self
                    .record_accounting_fault(ResourceAccountingFault::GenerationAccountingFailure),
            }
        }
        self.resolved_leases.retain(|(owner, _)| *owner != frame);
        self.provisional_allocations
            .retain(|(owner, _)| *owner != frame);
        self.complete_cleanup(cleanup)
    }

    fn complete_cleanup(&mut self, mut cleanup: FrameCleanup) -> FrameCleanup {
        if let Some(fault) = self.accounting_fault {
            cleanup.retention = ResourceRetentionOutcome::AccountingFault { fault };
            return cleanup;
        }
        if self.ensure_accounting_exact().is_err() {
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

    pub(super) fn trim_idle(&mut self) -> FrameCleanup {
        if self.ensure_accounting_exact().is_err() {
            return self.complete_cleanup(FrameCleanup::default());
        }
        let mut idle = Vec::new();
        for (identity, entry) in &self.entries {
            if self.generation.probe(entry.key.generation_class()).is_err() {
                return self.complete_cleanup(FrameCleanup::default());
            }
            if let ResourceEntryState::Idle { last_used_frame } = entry.state {
                idle.push((last_used_frame, *identity));
            }
        }
        idle.sort_unstable();
        let mut cleanup = FrameCleanup::default();
        for (_, identity) in idle {
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
            let entry = self
                .entries
                .remove(&identity)
                .expect("selected idle resource must remain registered");
            self.generation.eviction(entry.key.generation_class());
            // Physical custody may survive in a native/lease clone; only idle
            // entries are eligible even when the physical total remains high.
            drop(entry);
            cleanup.evicted_resources.push(identity);
        }
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

#[must_use = "frame resource scopes clean outstanding leases when resolved or dropped"]
pub(crate) struct FrameResourceScope {
    pub(super) state: Arc<Mutex<ResourceManagerState>>,
    pub(super) manager_identity: ManagerIdentity,
    pub(super) frame: FrameIdentity,
    generation: Generation,
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
    ) -> Self {
        Self {
            state,
            manager_identity,
            frame,
            generation,
            pending_drop_disposition: Some(FrameResourceDisposition::ReleaseReusable),
        }
    }

    pub(crate) fn generation(&self) -> &Generation {
        &self.generation
    }

    fn validated_payload<'scope>(
        &'scope self,
        lease: &'scope ResourceLease,
    ) -> Result<&'scope ResourcePayload> {
        lock_state(&self.state).validate_lease(&self.manager_identity, self.frame, &lease.token)?;
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
            &self.manager_identity,
            self.frame,
            ResourceCacheKey::VelloBuffer(key),
            byte_len,
            idle_reuse,
            |generation| {
                let buffer =
                    generation.allocation(GenerationClass::CacheVello, byte_len, || {
                        device.create_buffer(&wgpu::BufferDescriptor {
                            label: Some("Surgeist internal Vello buffer"),
                            size: byte_len,
                            usage: key.usage(),
                            mapped_at_creation: false,
                        })
                    })?;
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
            &self.manager_identity,
            self.frame,
            ResourceCacheKey::VelloImage(key),
            byte_len,
            idle_reuse,
            |generation| {
                let format = key.texture_format();
                let extent = key.extent();
                let texture =
                    generation.allocation(GenerationClass::CacheVello, byte_len, || {
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
                    })?;
                let view = texture.create_view(&wgpu::TextureViewDescriptor {
                    label: Some("Surgeist internal Vello image view"),
                    format: Some(format),
                    dimension: Some(wgpu::TextureViewDimension::D2),
                    usage: None,
                    aspect: wgpu::TextureAspect::All,
                    base_mip_level: 0,
                    mip_level_count: None,
                    base_array_layer: 0,
                    array_layer_count: None,
                })?;
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
            &self.manager_identity,
            self.frame,
            key,
            byte_len,
            IdleReuse::Allowed,
            |generation| {
                let texture =
                    generation.allocation(GenerationClass::CacheEffects, byte_len, || {
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
                    })?;
                let view = texture.create_view(&wgpu::TextureViewDescriptor::default())?;
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
        let key = ResourceCacheKey::ResolvedMaskUpload(descriptor.cache_key());
        lock_state(&self.state).acquire_with_payload(
            &self.manager_identity,
            self.frame,
            key,
            descriptor.byte_len(),
            IdleReuse::Allowed,
            |generation| {
                let texture = generation.allocation(
                    GenerationClass::CacheMasks,
                    descriptor.byte_len(),
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
                let view = texture.create_view(&wgpu::TextureViewDescriptor::default())?;
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
            &self.manager_identity,
            self.frame,
            key,
            byte_len,
            IdleReuse::Allowed,
            |generation| {
                let buffer =
                    generation.allocation(GenerationClass::CacheKernels, byte_len, || {
                        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some("Surgeist immutable Gaussian kernel buffer"),
                            contents: plan.upload_bytes(),
                            usage: wgpu::BufferUsages::STORAGE,
                        })
                    })?;
                Ok(ResourcePayload::GaussianKernelBuffer { buffer })
            },
        )
    }

    pub(crate) fn release(&mut self, lease: ResourceLease) -> Result<()> {
        lock_state(&self.state).release(&self.manager_identity, self.frame, lease.token)
    }

    pub(crate) fn resolve_leases_atomically(&mut self, leases: &[&ResourceLease]) -> Result<()> {
        let tokens = leases.iter().map(|lease| &lease.token).collect::<Vec<_>>();
        lock_state(&self.state).resolve_leases_atomically(
            &self.manager_identity,
            self.frame,
            &tokens,
        )
    }

    pub(crate) fn record_vello_atlas_recovery(&mut self, outcome: VelloAtlasOutcome) {
        lock_state(&self.state).record_vello_atlas_recovery(outcome);
    }

    pub(crate) fn consume_vello_atlas_recovery(&mut self) {
        lock_state(&self.state).consume_vello_atlas_recovery();
    }

    pub(crate) fn retire_idle_vello_atlases(&mut self) -> Result<()> {
        lock_state(&self.state).retire_idle_vello_atlases()
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
        let tokens = leases.iter().map(|lease| &lease.token).collect::<Vec<_>>();
        lock_state(&self.state).ensure_frame_resolution_ready(
            &self.manager_identity,
            self.frame,
            &tokens,
        )
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
        match disposition {
            FrameResourceDisposition::ReleaseReusable => {
                state.cleanup_frame(&self.manager_identity, self.frame)
            }
            FrameResourceDisposition::AbortProvisional => {
                state.abort_provisional_frame(&self.manager_identity, self.frame)
            }
            FrameResourceDisposition::Discard => {
                state.discard_frame(&self.manager_identity, self.frame)
            }
        }
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

#[derive(Debug, Default, Eq, PartialEq)]
pub(crate) struct FrameCleanup {
    pub(super) evicted_resources: Vec<ResourceIdentity>,
    pub(super) retention: ResourceRetentionOutcome,
}

impl FrameCleanup {
    pub(crate) fn followed_by(mut self, mut later: Self) -> Self {
        self.evicted_resources.append(&mut later.evicted_resources);
        self.retention = match (self.retention, later.retention) {
            (ResourceRetentionOutcome::AccountingFault { fault }, _) => {
                ResourceRetentionOutcome::AccountingFault { fault }
            }
            (_, ResourceRetentionOutcome::AccountingFault { fault }) => {
                ResourceRetentionOutcome::AccountingFault { fault }
            }
            (_, later) => later,
        };
        self
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
    fn cache_candidate_scans_charge_each_inspected_entry_to_its_own_class() {
        let manager = ResourceManager::default();
        let mut initial = manager.begin_frame().unwrap();
        let atlas = initial.acquire(ResourceCacheKey::VelloAtlas, 4).unwrap();
        let effect = initial.acquire(effect_key(1), 4).unwrap();
        initial.release(atlas).unwrap();
        initial.release(effect).unwrap();
        let _ = initial.finish();

        // Allocation examines the existing atlas once. Idle trimming examines
        // both entries, even though the default target retains both.
        let probes = |class| {
            manager
                .generation()
                .class_usage(class)
                .unwrap()
                .work(RenderWork::ResourceProbes)
        };
        assert_eq!(probes(GenerationClass::CacheVello), 2);
        assert_eq!(probes(GenerationClass::CacheEffects), 1);

        let request = crate::resource::ResourceAllocationPreflight::effect_texture(
            EffectTextureDescriptor::try_capture(
                PhysicalSize::new(1, 1),
                wgpu::TextureUsages::TEXTURE_BINDING,
            )
            .unwrap(),
        )
        .unwrap();
        manager
            .preflight_graph_acquisitions(&[request.clone(), request])
            .unwrap();
        // Each independent request examines both candidates; selection of one
        // idle identity does not remove it from the next request's scan.
        assert_eq!(probes(GenerationClass::CacheVello), 4);
        assert_eq!(probes(GenerationClass::CacheEffects), 3);

        let mut reuse = manager.begin_frame().unwrap();
        let effect = reuse.acquire(effect_key(1), 4).unwrap();
        assert_eq!(probes(GenerationClass::CacheVello), 5);
        assert_eq!(probes(GenerationClass::CacheEffects), 4);
        reuse.release(effect).unwrap();
        let _ = reuse.finish();
        assert_eq!(probes(GenerationClass::CacheVello), 6);
        assert_eq!(probes(GenerationClass::CacheEffects), 5);

        let mut retirement = manager.begin_frame().unwrap();
        retirement.retire_idle_vello_atlases().unwrap();
        assert_eq!(probes(GenerationClass::CacheVello), 7);
        assert_eq!(probes(GenerationClass::CacheEffects), 6);
    }

    #[test]
    fn graph_allocation_batch_preview_refuses_overflow_without_custody_or_peak_changes() {
        let manager = ResourceManager::default();
        let mut frame = manager.begin_frame().unwrap();
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
            .preflight_graph_acquisitions(&[request.clone(), request])
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
        let mut first = manager.begin_frame().unwrap();
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

        let mut second = manager.begin_frame().unwrap();
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
        let mut first = manager.begin_frame().unwrap();
        let removed = first.acquire(effect_key(2), 8).unwrap();
        let surviving_native = removed.payload.clone();
        first.discard(removed).unwrap();
        let _ = first.finish();
        assert_eq!(manager.generation().registered_cache_bytes().unwrap(), 0);
        assert_eq!(manager.generation().cache_bytes().unwrap(), 8);

        let mut second = manager.begin_frame().unwrap();
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
        let mut first = manager.begin_frame().unwrap();
        let mut second = manager.begin_frame().unwrap();
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
            .allocation(GenerationClass::PassBuffers, 16, || ())
            .unwrap();
        let mut frame = manager.begin_frame().unwrap();
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
        let frame = manager.begin_frame().unwrap();
        let result = lock_state(&frame.state).acquire_with_payload(
            &frame.manager_identity,
            frame.frame,
            effect_key(1),
            4,
            IdleReuse::Fresh,
            |generation| {
                let _native = generation.allocation(GenerationClass::CacheEffects, 4, || ())?;
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
        let mut frame = manager.begin_frame().unwrap();
        let lease = frame.acquire(effect_key(1), 4).unwrap();
        manager.generation().retire();
        assert_eq!(manager.generation().cache_bytes().unwrap(), 0);
        assert_eq!(manager.generation().registered_cache_bytes().unwrap(), 0);
        assert!(manager.begin_frame().is_err());
        frame.discard(lease).unwrap_err();
        drop(frame);
        manager.generation().retire();
        assert_eq!(manager.generation().cache_bytes().unwrap(), 0);
        assert_eq!(manager.generation().registered_cache_bytes().unwrap(), 0);
    }
}
