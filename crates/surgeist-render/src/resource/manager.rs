use super::{
    active_frames::ActiveFrames,
    convolution::{ConvolutionWeightsKey, ConvolutionWeightsPlan},
    entries::ResourceEntries,
    gaussian::{GaussianKernelKey, GaussianKernelPlan},
};
use crate::{
    BackendErrorCode, Error, RenderAccountingError, RenderQuantity, RenderRefusalReason,
    RenderResource, ResourceCacheBudget, Result,
    accounting::{
        Account, AccountedVec, CacheRegistration, CpuAccount, Generation, GenerationClass,
        OwnedGpu, StorageAccount,
    },
    image::{ResolvedMaskUploadDescriptor, ResolvedMaskUploadKey},
    texture::{EffectTextureDescriptor, EffectTextureKey},
    vello_engine::{VelloAtlasOutcome, VelloBufferKey, VelloImageKey},
};
use std::{fmt, sync::Arc};

#[derive(Clone)]
pub(crate) struct ManagerIdentity(Arc<()>);

impl ManagerIdentity {
    pub(super) fn new() -> Self {
        Self(Arc::new(()))
    }
}

#[cfg(test)]
mod registry_tests;

impl PartialEq for ManagerIdentity {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for ManagerIdentity {}

impl fmt::Debug for ManagerIdentity {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("ManagerIdentity(..)")
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct FrameIdentity(pub(super) u64);

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct ResourceIdentity(pub(super) u64);

impl ResourceIdentity {
    pub(crate) const fn get(self) -> u64 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct AllocationGeneration(pub(super) u64);

/// One non-interchangeable namespace for every resource role entering the
/// per-device manager.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(crate) enum ResourceCacheKey {
    #[cfg(test)]
    VelloAtlas,
    VelloBuffer(VelloBufferKey),
    VelloImage(VelloImageKey),
    EffectTexture(EffectTextureKey),
    ResolvedMaskUpload(ResolvedMaskUploadKey),
    GaussianKernelBuffer(GaussianKernelKey),
    ConvolutionWeightsBuffer(ConvolutionWeightsKey),
}

impl ResourceCacheKey {
    pub(super) fn matches_in(&self, other: &Self, observer: Option<&Account>) -> Result<bool> {
        match (self, other) {
            (Self::ResolvedMaskUpload(a), Self::ResolvedMaskUpload(b)) => a.matches_in(b, observer),
            (Self::ConvolutionWeightsBuffer(a), Self::ConvolutionWeightsBuffer(b)) => {
                a.matches_in(b, observer)
            }
            _ => Ok(self == other),
        }
    }
    pub(super) const fn generation_class(&self) -> GenerationClass {
        match self {
            #[cfg(test)]
            Self::VelloAtlas => GenerationClass::CacheVello,
            Self::VelloBuffer(_) | Self::VelloImage(_) => GenerationClass::CacheVello,
            Self::EffectTexture(_) => GenerationClass::CacheEffects,
            Self::ResolvedMaskUpload(_) => GenerationClass::CacheMasks,
            Self::GaussianKernelBuffer(_) | Self::ConvolutionWeightsBuffer(_) => {
                GenerationClass::CacheKernels
            }
        }
    }

    pub(super) const fn is_vello_atlas(&self) -> bool {
        matches!(self, Self::VelloImage(key) if key.is_persistent_atlas())
    }

    const fn accepts_graph_preparation(&self) -> bool {
        matches!(
            self,
            Self::EffectTexture(_)
                | Self::ResolvedMaskUpload(_)
                | Self::GaussianKernelBuffer(_)
                | Self::ConvolutionWeightsBuffer(_)
        )
    }
}

/// Immutable concrete-allocation facts used to preflight a complete graph
/// before opening its frame resource scope.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ResourceAllocationPreflight {
    key: ResourceCacheKey,
    byte_len: u64,
}

impl ResourceAllocationPreflight {
    pub(crate) fn effect_texture(descriptor: EffectTextureDescriptor) -> Result<Self> {
        Ok(Self {
            key: ResourceCacheKey::EffectTexture(descriptor.cache_key()),
            byte_len: descriptor.checked_byte_len()?,
        })
    }

    pub(crate) fn resolved_mask(
        descriptor: &ResolvedMaskUploadDescriptor,
        account: &Account,
    ) -> Result<Option<Self>> {
        let size = descriptor.physical_size();
        if size.width() == 0 || size.height() == 0 {
            return Ok(None);
        }
        descriptor.validate_upload_byte_len(descriptor.bytes().len())?;
        Ok(Some(Self {
            key: ResourceCacheKey::ResolvedMaskUpload(descriptor.cache_key_in(account)?),
            byte_len: descriptor.byte_len(),
        }))
    }

    pub(crate) fn convolution_weights(plan: &ConvolutionWeightsPlan) -> Result<Self> {
        plan.validate_upload_byte_len(plan.upload_bytes().len())?;
        Ok(Self {
            key: ResourceCacheKey::ConvolutionWeightsBuffer(plan.key()),
            byte_len: plan.byte_len(),
        })
    }

    pub(crate) fn gaussian_kernel(plan: &GaussianKernelPlan) -> Result<Self> {
        plan.validate_upload_byte_len(plan.upload_bytes().len())?;
        Ok(Self {
            key: ResourceCacheKey::GaussianKernelBuffer(plan.key()),
            byte_len: plan.byte_len(),
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ResourceAccountingFault {
    SelectedResourceMissing {
        resource: ResourceIdentity,
    },
    SelectedResourceAlreadyReserved {
        resource: ResourceIdentity,
    },
    RetainedByteUnderflow {
        retained_bytes: u64,
        discarded_entry_bytes: u64,
    },
    SurvivingEntryByteTotalOverflow,
    RetainedByteMismatch {
        retained_bytes: u64,
        registered_entry_bytes: u64,
    },
    GenerationAccountingFailure,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum ResourceEntryState {
    Idle { last_used_frame: FrameIdentity },
    Leased { frame: FrameIdentity },
}

#[derive(Clone)]
pub(super) enum ResourcePayload {
    #[cfg(test)]
    Modeled {
        _allocation: OwnedGpu<()>,
    },
    VelloBuffer {
        buffer: OwnedGpu<wgpu::Buffer>,
    },
    VelloImage {
        texture: OwnedGpu<wgpu::Texture>,
        view: OwnedGpu<wgpu::TextureView>,
    },
    EffectTexture {
        texture: OwnedGpu<wgpu::Texture>,
        view: OwnedGpu<wgpu::TextureView>,
    },
    ResolvedMaskUpload {
        texture: OwnedGpu<wgpu::Texture>,
        view: OwnedGpu<wgpu::TextureView>,
    },
    GaussianKernelBuffer {
        buffer: OwnedGpu<wgpu::Buffer>,
    },
    ConvolutionWeightsBuffer {
        buffer: OwnedGpu<wgpu::Buffer>,
    },
}

impl ResourcePayload {
    pub(super) const fn matches_key(&self, key: &ResourceCacheKey) -> bool {
        match (self, key) {
            #[cfg(test)]
            (
                Self::Modeled { .. },
                ResourceCacheKey::VelloAtlas | ResourceCacheKey::EffectTexture(_),
            ) => true,
            (Self::VelloBuffer { .. }, ResourceCacheKey::VelloBuffer(_))
            | (Self::VelloImage { .. }, ResourceCacheKey::VelloImage(_))
            | (Self::EffectTexture { .. }, ResourceCacheKey::EffectTexture(_))
            | (Self::ResolvedMaskUpload { .. }, ResourceCacheKey::ResolvedMaskUpload(_))
            | (Self::GaussianKernelBuffer { .. }, ResourceCacheKey::GaussianKernelBuffer(_))
            | (
                Self::ConvolutionWeightsBuffer { .. },
                ResourceCacheKey::ConvolutionWeightsBuffer(_),
            ) => true,
            _ => false,
        }
    }

    pub(super) const fn label(&self) -> &'static str {
        match self {
            #[cfg(test)]
            Self::Modeled { .. } => "Modeled",
            Self::VelloBuffer { .. } => "VelloBuffer",
            Self::VelloImage { .. } => "VelloImage",
            Self::EffectTexture { .. } => "EffectTexture",
            Self::ResolvedMaskUpload { .. } => "ResolvedMaskUpload",
            Self::GaussianKernelBuffer { .. } => "GaussianKernelBuffer",
            Self::ConvolutionWeightsBuffer { .. } => "ConvolutionWeightsBuffer",
        }
    }
}

pub(super) struct ResourceEntry {
    pub(super) key: ResourceCacheKey,
    pub(super) allocation_generation: AllocationGeneration,
    pub(super) byte_len: u64,
    pub(super) state: ResourceEntryState,
    pub(super) payload: ResourcePayload,
    pub(super) registration: CacheRegistration,
    pub(super) resolved: bool,
    pub(super) provisional: bool,
}

pub(super) struct ResourceManagerState {
    pub(super) generation: Generation,
    pub(super) cpu: CpuAccount,
    pub(super) identity: ManagerIdentity,
    pub(super) budget: ResourceCacheBudget,
    pub(super) next_resource: u64,
    pub(super) accounting_fault: Option<ResourceAccountingFault>,
    pub(super) active_frames: ActiveFrames,
    pub(super) entries: ResourceEntries,
    pub(super) pending_vello_atlas_recovery: Option<VelloAtlasOutcome>,
    #[cfg(test)]
    pub(super) payload_creation_attempts: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum IdleReuse {
    Allowed,
    Fresh,
}

impl ResourceManagerState {
    const ACCOUNTING_FAULT_MESSAGE: &'static str =
        "resource manager is unavailable after a retained-byte accounting invariant failure";

    pub(super) fn accounting_fault_error() -> Error {
        Error::new(
            BackendErrorCode::RenderFailed,
            Self::ACCOUNTING_FAULT_MESSAGE,
        )
    }

    pub(super) fn allocation_preflight_error(error: Error) -> Error {
        match error.accounting_diagnostic() {
            Some(RenderAccountingError::GenerationRefused {
                quantity: RenderQuantity::Resource(RenderResource::GpuBytes),
                current,
                amount,
                reason: RenderRefusalReason::Overflow,
            }) => Error::invalid_value(
                "retained resource byte length",
                format!("{current} + {amount}"),
                "must fit in u64",
            ),
            _ => error,
        }
    }

    pub(super) fn ensure_accounting_healthy(&self) -> Result<()> {
        if self.accounting_fault.is_some() {
            return Err(Self::accounting_fault_error());
        }
        self.generation.ensure_healthy()
    }

    pub(super) fn record_accounting_fault(&mut self, fault: ResourceAccountingFault) {
        if self.accounting_fault.is_none() {
            self.accounting_fault = Some(fault);
        }
    }

    pub(super) fn fail_accounting<T>(&mut self, fault: ResourceAccountingFault) -> Result<T> {
        self.record_accounting_fault(fault);
        Err(Self::accounting_fault_error())
    }

    pub(super) fn remove_selected_entry(
        generation: &Generation,
        entries: &mut ResourceEntries,
        identity: ResourceIdentity,
    ) -> std::result::Result<u64, ResourceAccountingFault> {
        let entry = entries
            .remove(&identity)
            .ok_or(ResourceAccountingFault::SelectedResourceMissing { resource: identity })?;
        let removed_bytes = entry.byte_len;
        generation.eviction(entry.key.generation_class());
        drop(entry);
        Ok(removed_bytes)
    }

    pub(super) fn selected_entry_mut(
        entries: &mut ResourceEntries,
        identity: ResourceIdentity,
    ) -> std::result::Result<&mut ResourceEntry, ResourceAccountingFault> {
        entries
            .get_mut(&identity)
            .ok_or(ResourceAccountingFault::SelectedResourceMissing { resource: identity })
    }

    fn idle_reservation_index(
        reserved: &[ResourceIdentity],
        identity: ResourceIdentity,
    ) -> std::result::Result<usize, ResourceAccountingFault> {
        match reserved.binary_search(&identity) {
            Err(index) => Ok(index),
            Ok(_) => {
                Err(ResourceAccountingFault::SelectedResourceAlreadyReserved { resource: identity })
            }
        }
    }

    pub(super) fn checked_registered_entry_bytes(
        &self,
        observer: Option<&Account>,
    ) -> Result<Option<u64>> {
        let mut bytes = 0_u64;
        for entry in self.entries.values() {
            self.generation
                .probe(entry.key.generation_class(), observer)?;
            let Some(next) = bytes.checked_add(entry.byte_len) else {
                return Ok(None);
            };
            bytes = next;
        }
        Ok(Some(bytes))
    }

    pub(super) fn checked_idle_summary(&self) -> Option<(usize, u64)> {
        let retained_count = self
            .entries
            .values()
            .filter(|entry| {
                // This mandatory summary cannot spend the caller's allowance.
                // The lifetime observer records any first fault and continues.
                let _ = self.generation.probe(entry.key.generation_class(), None);
                matches!(entry.state, ResourceEntryState::Idle { .. })
            })
            .count();
        let retained_byte_len = self
            .entries
            .values()
            .filter_map(|entry| {
                let _ = self.generation.probe(entry.key.generation_class(), None);
                match entry.state {
                    ResourceEntryState::Idle { .. } => Some(entry.byte_len),
                    ResourceEntryState::Leased { .. } => None,
                }
            })
            .try_fold(0_u64, u64::checked_add)?;
        Some((retained_count, retained_byte_len))
    }

    pub(super) fn ensure_accounting_exact(&mut self, observer: Option<&Account>) -> Result<()> {
        self.ensure_accounting_healthy()?;
        let Some(registered_entry_bytes) = self.checked_registered_entry_bytes(observer)? else {
            return self.fail_accounting(ResourceAccountingFault::SurvivingEntryByteTotalOverflow);
        };
        let retained_bytes = self.generation.registered_cache_bytes()?;
        if retained_bytes != registered_entry_bytes {
            return self.fail_accounting(ResourceAccountingFault::RetainedByteMismatch {
                retained_bytes,
                registered_entry_bytes,
            });
        }
        Ok(())
    }

    pub(super) fn preflight_graph_acquisitions(
        &mut self,
        requests: &[ResourceAllocationPreflight],
        observer: Option<&Account>,
    ) -> Result<()> {
        self.cpu.ensure_healthy()?;
        self.ensure_accounting_exact(observer)?;
        let mut selected_idle = AccountedVec::<ResourceIdentity>::new_in(
            &StorageAccount::Renderer(self.cpu.clone()),
            requests.len(),
            RenderResource::ScratchCpuBytes,
        )?;
        let mut allocation_bytes = AccountedVec::new_in(
            &StorageAccount::Renderer(self.cpu.clone()),
            requests.len(),
            RenderResource::ScratchCpuBytes,
        )?;
        let mut next_resource = self.next_resource;

        for request in requests {
            if !request.key.accepts_graph_preparation() {
                return Err(Error::invalid_value(
                    "graph resource preflight key",
                    format!("{:?}", request.key),
                    "must use an effect texture, resolved mask, Gaussian kernel, or convolution weights namespace",
                ));
            }
            if request.byte_len == 0 {
                return Err(Error::invalid_value(
                    "graph resource preflight byte length",
                    request.byte_len,
                    "must be greater than zero",
                ));
            }

            let mut reusable = None;
            for (identity, entry) in self.entries.iter() {
                self.generation
                    .probe(entry.key.generation_class(), observer)?;
                if let ResourceEntryState::Idle { last_used_frame } = entry.state
                    && entry.key.matches_in(&request.key, observer)?
                    && entry.byte_len == request.byte_len
                {
                    self.generation.probe_cache(observer)?;
                    if selected_idle.binary_search(identity).is_ok() {
                        continue;
                    }
                    let candidate = (last_used_frame, *identity);
                    if reusable.is_none_or(|selected| candidate < selected) {
                        reusable = Some(candidate);
                    }
                }
            }
            if let Some((_, identity)) = reusable {
                self.generation.probe_cache(observer)?;
                let index = match Self::idle_reservation_index(&selected_idle, identity) {
                    Ok(index) => index,
                    Err(fault) => return self.fail_accounting(fault),
                };
                selected_idle.insert_slot(index)?.write(identity);
                continue;
            }

            next_resource = next_resource.checked_add(1).ok_or_else(|| {
                Error::invalid_value(
                    "resource identity",
                    next_resource,
                    "must have remaining identity space for every prepared allocation",
                )
            })?;
            allocation_bytes.push((request.key.generation_class(), request.byte_len))?;
        }

        self.generation
            .preview_classified_cache_allocations(allocation_bytes.iter().copied(), observer)
            .map_err(Self::allocation_preflight_error)?;
        self.entries.reserve(allocation_bytes.len(), &self.cpu)
    }

    pub(super) fn record_vello_atlas_recovery(&mut self, outcome: VelloAtlasOutcome) {
        self.pending_vello_atlas_recovery =
            VelloAtlasOutcome::merge_pending_recovery(self.pending_vello_atlas_recovery, outcome);
    }

    pub(super) fn consume_vello_atlas_recovery(&mut self) {
        let _ = self.pending_vello_atlas_recovery.take();
    }

    pub(super) fn retire_idle_vello_atlases(&mut self, frame: FrameIdentity) -> Result<()> {
        self.ensure_accounting_exact(None)?;
        let scratch = self
            .active_frames
            .get_mut(&frame)
            .ok_or(RenderAccountingError::Invariant)?;
        scratch.indices.clear();
        for (identity, entry) in self.entries.iter() {
            self.generation.probe(entry.key.generation_class(), None)?;
            if entry.key.is_vello_atlas() && matches!(entry.state, ResourceEntryState::Idle { .. })
            {
                scratch.push_index(entry.byte_len, *identity)?;
            }
        }
        for &(_, identity) in scratch.indices.iter() {
            self.generation.probe_cache(None)?;
            if let Err(fault) =
                Self::remove_selected_entry(&self.generation, &mut self.entries, identity)
            {
                scratch.indices.clear();
                return self.fail_accounting(fault);
            }
        }
        scratch.indices.clear();
        self.ensure_accounting_exact(None)
    }
}
