// Copyright 2023 the Vello Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Modified for surgeist-render; see NOTICE-VELLO.md for provenance and adaptations.

use std::collections::{HashMap, HashSet};

use crate::{
    BackendErrorCode, Error, PhysicalSize, Result,
    gpu_transaction::VelloResourceCommitProof,
    resource::{FrameCleanup, FrameResourceScope, ResourceLease, ResourceManager},
};

use super::encoder::ActiveVelloEncodingScope;
use super::{
    BufferHandle, BufferIntent, BufferRole, ImageHandle, ImageIntent, ImageRetention, ImageRole,
    RasterImageFormat, ResourceIntent, ResourceReference,
};

#[cfg(test)]
use super::RecordingBuilder;

#[cfg(test)]
use crate::resource::ResourceAccountingFault;

struct ManagedBuffer {
    lease: ResourceLease,
    byte_len: u64,
}

struct ManagedImage {
    lease: ResourceLease,
    extent: PhysicalSize,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) struct VelloBufferKey {
    role: BufferRole,
    byte_len: u64,
    usage: wgpu::BufferUsages,
}

impl VelloBufferKey {
    fn from_intent(intent: &BufferIntent) -> Self {
        Self {
            role: intent.role,
            byte_len: intent.byte_len,
            usage: buffer_usage(intent.role),
        }
    }

    pub(crate) const fn byte_len(self) -> u64 {
        self.byte_len
    }

    pub(crate) const fn usage(self) -> wgpu::BufferUsages {
        self.usage
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) struct VelloImageKey {
    role: ImageRole,
    extent: PhysicalSize,
    format: RasterImageFormat,
    retention: ImageRetention,
    usage: wgpu::TextureUsages,
}

impl VelloImageKey {
    fn from_intent(intent: &ImageIntent) -> Self {
        Self {
            role: intent.role,
            extent: intent.extent,
            format: intent.format,
            retention: intent.retention,
            usage: image_usage(intent.role),
        }
    }

    pub(crate) const fn extent(self) -> PhysicalSize {
        self.extent
    }

    pub(crate) const fn texture_format(self) -> wgpu::TextureFormat {
        texture_format(self.format)
    }

    pub(crate) const fn usage(self) -> wgpu::TextureUsages {
        self.usage
    }

    pub(crate) const fn is_persistent_atlas(self) -> bool {
        matches!(
            (self.role, self.retention),
            (ImageRole::ImageAtlas, ImageRetention::PersistentImageAtlas)
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum VelloAtlasOutcome {
    NoAtlas,
    #[cfg(test)]
    Retain,
    MarkDirty,
    Recreate,
}

impl VelloAtlasOutcome {
    pub(crate) const fn merge_pending_recovery(
        pending: Option<Self>,
        latest: Self,
    ) -> Option<Self> {
        match (pending, latest) {
            (Some(Self::Recreate), _) | (Some(Self::MarkDirty), Self::Recreate) => {
                Some(Self::Recreate)
            }
            (Some(Self::MarkDirty), _) => Some(Self::MarkDirty),
            #[cfg(test)]
            (Some(Self::NoAtlas | Self::Retain) | None, Self::MarkDirty | Self::Recreate) => {
                Some(latest)
            }
            #[cfg(not(test))]
            (Some(Self::NoAtlas) | None, Self::MarkDirty | Self::Recreate) => Some(latest),
            #[cfg(test)]
            (Some(Self::NoAtlas | Self::Retain) | None, Self::NoAtlas | Self::Retain) => None,
            #[cfg(not(test))]
            (Some(Self::NoAtlas) | None, Self::NoAtlas) => None,
        }
    }
}

#[derive(Clone, Copy)]
enum PendingPersistentAtlas {
    NoAtlas,
    NewlyAllocated(ImageHandle),
}

impl PendingPersistentAtlas {
    const fn is_present(self) -> bool {
        !matches!(self, Self::NoAtlas)
    }

    const fn resource(self) -> Option<ImageHandle> {
        match self {
            Self::NoAtlas => None,
            Self::NewlyAllocated(resource) => Some(resource),
        }
    }

    #[cfg(test)]
    const fn commit_outcome(self) -> VelloAtlasOutcome {
        match self {
            Self::NoAtlas => VelloAtlasOutcome::NoAtlas,
            Self::NewlyAllocated(_) => VelloAtlasOutcome::Retain,
        }
    }

    const fn abort_outcome(self) -> VelloAtlasOutcome {
        match self {
            Self::NoAtlas => VelloAtlasOutcome::NoAtlas,
            Self::NewlyAllocated(_) => VelloAtlasOutcome::Recreate,
        }
    }
}

struct PendingVelloResources {
    frame_scope: Option<FrameResourceScope>,
    clean_retention: CleanVelloResourceRetention,
    buffers: HashMap<BufferHandle, ManagedBuffer>,
    images: HashMap<ImageHandle, ManagedImage>,
    persistent_image_atlas: PendingPersistentAtlas,
    released_buffers: HashSet<BufferHandle>,
    released_images: HashSet<ImageHandle>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum CleanVelloResourceRetention {
    DirectAtlasOnly,
    ReusableGraphFrame,
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum VelloResourceAllocationRoleForTest {
    InternalVelloRasterBuffer,
    InternalVelloRasterImage,
}

#[cfg(test)]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct VelloResourceAllocationSummaryForTest {
    requested_roles: Vec<VelloResourceAllocationRoleForTest>,
    allocated_roles: Vec<VelloResourceAllocationRoleForTest>,
}

#[cfg(test)]
impl VelloResourceAllocationSummaryForTest {
    fn role_for_intent(intent: &ResourceIntent) -> VelloResourceAllocationRoleForTest {
        match intent {
            ResourceIntent::Buffer(_) => {
                VelloResourceAllocationRoleForTest::InternalVelloRasterBuffer
            }
            ResourceIntent::Image(_) => {
                VelloResourceAllocationRoleForTest::InternalVelloRasterImage
            }
        }
    }

    fn record_request(&mut self, intent: &ResourceIntent) {
        self.requested_roles.push(Self::role_for_intent(intent));
    }

    fn record_allocation(&mut self, intent: &ResourceIntent) {
        self.allocated_roles.push(Self::role_for_intent(intent));
    }

    fn extend(&mut self, other: Self) {
        self.requested_roles.extend(other.requested_roles);
        self.allocated_roles.extend(other.allocated_roles);
    }

    fn role_count(
        roles: &[VelloResourceAllocationRoleForTest],
        role: VelloResourceAllocationRoleForTest,
    ) -> usize {
        roles.iter().filter(|actual| **actual == role).count()
    }

    pub(crate) fn internal_vello_raster_buffer_requests_for_test(&self) -> usize {
        Self::role_count(
            &self.requested_roles,
            VelloResourceAllocationRoleForTest::InternalVelloRasterBuffer,
        )
    }

    pub(crate) fn internal_vello_raster_buffer_allocations_for_test(&self) -> usize {
        Self::role_count(
            &self.allocated_roles,
            VelloResourceAllocationRoleForTest::InternalVelloRasterBuffer,
        )
    }

    pub(crate) fn internal_vello_raster_image_requests_for_test(&self) -> usize {
        Self::role_count(
            &self.requested_roles,
            VelloResourceAllocationRoleForTest::InternalVelloRasterImage,
        )
    }

    pub(crate) fn internal_vello_raster_image_allocations_for_test(&self) -> usize {
        Self::role_count(
            &self.allocated_roles,
            VelloResourceAllocationRoleForTest::InternalVelloRasterImage,
        )
    }
}

#[must_use]
pub(crate) struct VelloResourceLease {
    pending: PendingVelloResources,
    #[cfg(test)]
    allocation_summary: VelloResourceAllocationSummaryForTest,
}

#[must_use = "scope-clean Vello resource leases must be committed or aborted"]
pub(crate) struct ScopeResolvedVelloResourceLease {
    lease: VelloResourceLease,
}

/// Owns every non-clone lease encoded under one checked Vello scope.
#[must_use = "active Vello resource aggregates must be scope-resolved or aborted"]
pub(crate) struct VelloResourceLeaseAggregate {
    leases: Option<crate::accounting::AccountedVec<VelloResourceLease>>,
}

#[must_use = "scope-clean Vello resource aggregates must be committed or aborted"]
pub(crate) struct ScopeResolvedVelloResourceLeaseAggregate {
    leases: Option<crate::accounting::AccountedVec<VelloResourceLease>>,
}

#[must_use]
pub(crate) struct CommittedVelloResources {
    frame_cleanup: FrameCleanup,
    #[cfg(test)]
    atlas_outcome: VelloAtlasOutcome,
}

/// Keeps a scope-clean resource lease uncertain until the owning GPU transaction succeeds.
#[must_use = "pending Vello resources must be committed by their transaction or aborted on drop"]
pub(crate) struct PendingVelloResourceCommit {
    leases: Option<crate::accounting::AccountedVec<VelloResourceLease>>,
}

/// Sealed one-shot state proving that every pending Vello frame currently has
/// exact accounting and owns its complete clean disposition.
#[must_use = "accounting-ready Vello resources must be committed or aborted on drop"]
pub(crate) struct AccountingReadyVelloResourceCommit {
    pending: PendingVelloResourceCommit,
}

impl PendingVelloResourceCommit {
    pub(crate) fn new(mut lease: ScopeResolvedVelloResourceLease) -> Result<Self> {
        let storage = lease.lease.pending.frame_scope()?.storage();
        let mut leases = crate::accounting::AccountedVec::new_in(
            &storage,
            1,
            crate::RenderResource::ScratchCpuBytes,
        )?;
        lease.consume_pending_atlas_recovery();
        leases.push(lease.lease)?;
        Ok(Self {
            leases: Some(leases),
        })
    }
    pub(crate) fn from_aggregate(mut aggregate: ScopeResolvedVelloResourceLeaseAggregate) -> Self {
        let mut leases = aggregate.leases.take().expect("owned clean aggregate");
        for lease in leases.iter_mut() {
            lease.pending.consume_pending_atlas_recovery();
        }
        Self {
            leases: Some(leases),
        }
    }

    #[cfg(test)]
    pub(crate) fn allocation_summary_for_test(&self) -> VelloResourceAllocationSummaryForTest {
        self.leases
            .as_ref()
            .expect("owned pending leases")
            .iter()
            .fold(
                VelloResourceAllocationSummaryForTest::default(),
                |mut summary, lease| {
                    summary.extend(lease.allocation_summary.clone());
                    summary
                },
            )
    }

    #[cfg(test)]
    pub(crate) fn lease_count_for_test(&self) -> usize {
        self.leases.as_ref().expect("owned pending leases").len()
    }

    #[cfg(test)]
    pub(crate) fn poison_retained_byte_accounting_for_test(&self) -> ResourceAccountingFault {
        self.leases
            .as_ref()
            .expect("owned pending leases")
            .first()
            .expect("the accounting-poison fixture requires one pending Vello resource lease")
            .pending
            .frame_scope()
            .expect("the accounting-poison fixture requires an unresolved Vello frame")
            .poison_retained_byte_accounting_for_test()
    }

    fn ensure_commit_ready(&self) -> Result<()> {
        for lease in self.leases.as_ref().expect("owned pending leases").iter() {
            lease.pending.ensure_commit_ready()?;
        }
        Ok(())
    }

    pub(crate) fn into_accounting_ready(self) -> Result<AccountingReadyVelloResourceCommit> {
        self.ensure_commit_ready()?;
        Ok(AccountingReadyVelloResourceCommit { pending: self })
    }
}

impl AccountingReadyVelloResourceCommit {
    pub(crate) fn ensure_commit_ready(&self) -> Result<()> {
        self.pending.ensure_commit_ready()
    }

    pub(crate) fn commit(mut self, _proof: VelloResourceCommitProof) -> Result<FrameCleanup> {
        self.ensure_commit_ready()?;
        let mut cleanup = FrameCleanup::default();
        let leases = self.pending.leases.take().expect("owned pending leases");
        for lease in leases {
            cleanup =
                cleanup.followed_by(lease.into_committed_resources()?.into_frame_cleanup())?;
        }
        cleanup.into_accounting_result()
    }
}

impl Drop for PendingVelloResourceCommit {
    fn drop(&mut self) {
        if let Some(leases) = self.leases.take() {
            let _ = abort_vello_leases(leases);
        }
    }
}

#[must_use]
#[derive(Debug)]
pub(crate) struct AbortedVelloResources {
    #[cfg(test)]
    discarded_resource_count: usize,
    #[cfg(test)]
    atlas_outcome: VelloAtlasOutcome,
}

impl VelloResourceLease {
    #[cfg(test)]
    pub(super) fn allocate(
        scope: &ActiveVelloEncodingScope<'_>,
        manager: &ResourceManager,
        intents: &[ResourceIntent],
        observer: Option<&crate::accounting::Account>,
    ) -> Result<Self> {
        Self::allocate_with_retention(
            scope,
            manager,
            intents,
            CleanVelloResourceRetention::DirectAtlasOnly,
            observer,
        )
    }

    pub(super) fn allocate_with_retention(
        scope: &ActiveVelloEncodingScope<'_>,
        manager: &ResourceManager,
        intents: &[ResourceIntent],
        clean_retention: CleanVelloResourceRetention,
        observer: Option<&crate::accounting::Account>,
    ) -> Result<Self> {
        let device = scope.device();
        preflight_resource_intents(&device.limits(), intents, manager.generation(), observer)?;
        let mut pending = PendingVelloResources {
            frame_scope: Some(manager.begin_frame(observer)?),
            clean_retention,
            buffers: HashMap::new(),
            images: HashMap::new(),
            persistent_image_atlas: PendingPersistentAtlas::NoAtlas,
            released_buffers: HashSet::new(),
            released_images: HashSet::new(),
        };
        #[cfg(test)]
        let mut allocation_summary = VelloResourceAllocationSummaryForTest::default();

        for intent in intents {
            #[cfg(test)]
            allocation_summary.record_request(intent);
            match intent {
                ResourceIntent::Buffer(buffer) => allocate_buffer(device, &mut pending, buffer)?,
                ResourceIntent::Image(image) => allocate_image(device, &mut pending, image)?,
            }
            #[cfg(test)]
            allocation_summary.record_allocation(intent);
        }

        Ok(Self {
            pending,
            #[cfg(test)]
            allocation_summary,
        })
    }

    pub(super) fn buffer(&self, handle: BufferHandle) -> Result<&wgpu::Buffer> {
        let managed = self.live_buffer(handle)?;
        self.pending.frame_scope()?.vello_buffer(&managed.lease)
    }

    pub(super) fn buffer_for_upload(
        &self,
        handle: BufferHandle,
        byte_len: usize,
    ) -> Result<&wgpu::Buffer> {
        let allocated = self.live_buffer(handle)?;
        let byte_len = u64::try_from(byte_len).map_err(|_| {
            render_failed("internal Vello buffer upload length does not fit the GPU address space")
        })?;
        if byte_len > allocated.byte_len {
            return Err(render_failed(
                "internal Vello buffer upload exceeds its prepared allocation",
            ));
        }
        self.pending.frame_scope()?.vello_buffer(&allocated.lease)
    }

    pub(super) fn indirect_buffer(
        &self,
        handle: BufferHandle,
        offset: u64,
    ) -> Result<&wgpu::Buffer> {
        if !offset.is_multiple_of(u64::from(std::mem::size_of::<u32>() as u32)) {
            return Err(render_failed(
                "internal Vello indirect dispatch offset is not aligned",
            ));
        }
        let allocated = self.live_buffer(handle)?;
        let required_end = offset.checked_add(3 * u64::from(std::mem::size_of::<u32>() as u32));
        if required_end.is_none_or(|end| end > allocated.byte_len) {
            return Err(render_failed(
                "internal Vello indirect dispatch exceeds its prepared allocation",
            ));
        }
        self.pending.frame_scope()?.vello_buffer(&allocated.lease)
    }

    pub(super) fn image_texture(&self, handle: ImageHandle) -> Result<&wgpu::Texture> {
        let managed = self.live_image(handle)?;
        self.pending
            .frame_scope()?
            .vello_image(&managed.lease)
            .map(|(texture, _)| texture)
    }

    pub(super) fn image_view(&self, handle: ImageHandle) -> Result<&wgpu::TextureView> {
        let managed = self.live_image(handle)?;
        self.pending
            .frame_scope()?
            .vello_image(&managed.lease)
            .map(|(_, view)| view)
    }

    pub(super) fn image_extent(&self, handle: ImageHandle) -> Result<PhysicalSize> {
        self.live_image(handle).map(|allocated| allocated.extent)
    }

    pub(super) fn record_release(&mut self, reference: ResourceReference) -> Result<()> {
        match reference {
            ResourceReference::Buffer(handle) => {
                self.pending.frame_scope()?.probe()?;
                if !self.pending.buffers.contains_key(&handle) {
                    return Err(render_failed(
                        "internal Vello recording releases an unknown buffer",
                    ));
                }
                self.pending.frame_scope()?.probe()?;
                if !self.pending.released_buffers.insert(handle) {
                    return Err(render_failed(
                        "internal Vello recording releases a buffer more than once",
                    ));
                }
            }
            ResourceReference::Image(handle) => {
                self.pending.frame_scope()?.probe()?;
                if !self.pending.images.contains_key(&handle) {
                    return Err(render_failed(
                        "internal Vello recording releases an unknown image",
                    ));
                }
                self.pending.frame_scope()?.probe()?;
                if !self.pending.released_images.insert(handle) {
                    return Err(render_failed(
                        "internal Vello recording releases an image more than once",
                    ));
                }
            }
        }
        Ok(())
    }

    pub(super) fn after_clean_scope(self) -> ScopeResolvedVelloResourceLease {
        ScopeResolvedVelloResourceLease { lease: self }
    }

    fn into_committed_resources(mut self) -> Result<CommittedVelloResources> {
        self.pending.commit()
    }

    pub(crate) fn abort(mut self) -> AbortedVelloResources {
        self.pending.abort()
    }

    fn live_buffer(&self, handle: BufferHandle) -> Result<&ManagedBuffer> {
        self.pending.frame_scope()?.probe()?;
        if self.pending.released_buffers.contains(&handle) {
            return Err(render_failed(
                "internal Vello recording uses a released buffer",
            ));
        }
        self.pending.frame_scope()?.probe()?;
        self.pending
            .buffers
            .get(&handle)
            .ok_or_else(|| render_failed("internal Vello recording uses an unknown buffer"))
    }

    fn live_image(&self, handle: ImageHandle) -> Result<&ManagedImage> {
        self.pending.frame_scope()?.probe()?;
        if self.pending.released_images.contains(&handle) {
            return Err(render_failed(
                "internal Vello recording uses a released image",
            ));
        }
        self.pending.frame_scope()?.probe()?;
        self.pending
            .images
            .get(&handle)
            .ok_or_else(|| render_failed("internal Vello recording uses an unknown image"))
    }
}

impl VelloResourceLeaseAggregate {
    pub(crate) fn new(account: &crate::accounting::Account, capacity: usize) -> Result<Self> {
        Ok(Self {
            leases: Some(crate::accounting::AccountedVec::new(
                account,
                capacity,
                crate::RenderResource::ScratchCpuBytes,
            )?),
        })
    }
    pub(crate) fn push(&mut self, lease: VelloResourceLease) -> Result<()> {
        self.leases
            .as_mut()
            .expect("owned active aggregate")
            .push(lease)
    }
    pub(crate) fn after_clean_scope(mut self) -> ScopeResolvedVelloResourceLeaseAggregate {
        ScopeResolvedVelloResourceLeaseAggregate {
            leases: self.leases.take(),
        }
    }
    pub(crate) fn abort(mut self) -> AbortedVelloResources {
        abort_vello_leases(self.leases.take().expect("owned active aggregate"))
    }
}
impl Drop for VelloResourceLeaseAggregate {
    fn drop(&mut self) {
        if let Some(leases) = self.leases.take() {
            let _ = abort_vello_leases(leases);
        }
    }
}
impl Drop for ScopeResolvedVelloResourceLeaseAggregate {
    fn drop(&mut self) {
        if let Some(leases) = self.leases.take() {
            let _ = abort_vello_leases(leases);
        }
    }
}
fn abort_vello_leases(
    leases: crate::accounting::AccountedVec<VelloResourceLease>,
) -> AbortedVelloResources {
    leases.into_iter().fold(
        AbortedVelloResources::without_resources(),
        |aborted, lease| aborted.merge(lease.abort()),
    )
}

impl ScopeResolvedVelloResourceLease {
    #[cfg(test)]
    fn commit(self) -> Result<CommittedVelloResources> {
        self.lease.into_committed_resources()
    }

    #[cfg(test)]
    pub(crate) fn abort(self) -> AbortedVelloResources {
        self.lease.abort()
    }

    fn consume_pending_atlas_recovery(&mut self) {
        self.lease.pending.consume_pending_atlas_recovery();
    }
}

#[cfg(test)]
pub(crate) async fn over_limit_buffer_preflight_for_test(device: &wgpu::Device) -> Result<()> {
    let requested_size = device
        .limits()
        .max_buffer_size
        .checked_add(1)
        .ok_or_else(|| {
            render_failed("test device cannot represent an over-limit buffer request")
        })?;
    let account = crate::accounting::Account::for_test();
    let mut recording = super::RecordingBuilder::new(&account)?;
    let _buffer = recording.new_buffer(super::BufferRole::Scene, requested_size)?;
    let (_recording, intents) = recording.finish();
    let manager = ResourceManager::default();
    let scope = ActiveVelloEncodingScope::begin(device);
    let allocation = VelloResourceLease::allocate(&scope, &manager, &intents, None);
    let allocation_result = match allocation {
        Ok(lease) => {
            let _aborted = lease.abort();
            Ok(())
        }
        Err(error) => Err(error),
    };
    scope.finish(&account).await?;
    allocation_result
}

#[cfg(test)]
pub(crate) fn commit_scope_resolved_for_test(
    lease: ScopeResolvedVelloResourceLease,
) -> Result<VelloAtlasOutcome> {
    lease.commit().map(|committed| committed.atlas_outcome())
}

#[cfg(test)]
pub(crate) async fn no_atlas_commit_outcome_for_test(
    device: &wgpu::Device,
) -> Result<VelloAtlasOutcome> {
    let account = crate::accounting::Account::for_test();
    let intents = no_atlas_resource_intents_for_test()?;
    let manager = ResourceManager::default();
    let scope = ActiveVelloEncodingScope::begin(device);
    let allocation = VelloResourceLease::allocate(&scope, &manager, &intents, None);
    match allocation {
        Ok(lease) => match scope.finish_with_lease(lease, &account).await {
            Ok(lease) => {
                let committed = lease.commit()?;
                Ok(committed.atlas_outcome())
            }
            Err(failure) => Err(failure.into_error_and_aborted_resources().0),
        },
        Err(error) => {
            scope.finish(&account).await?;
            Err(error)
        }
    }
}

#[cfg(test)]
pub(crate) async fn no_atlas_abort_outcome_for_test(
    device: &wgpu::Device,
) -> Result<VelloAtlasOutcome> {
    let account = crate::accounting::Account::for_test();
    let intents = no_atlas_resource_intents_for_test()?;
    let manager = ResourceManager::default();
    let scope = ActiveVelloEncodingScope::begin(device);
    let allocation = VelloResourceLease::allocate(&scope, &manager, &intents, None);
    let outcome = allocation.map(|lease| lease.abort().into_atlas_outcome());
    scope.finish(&account).await?;
    outcome
}

#[cfg(test)]
fn no_atlas_resource_intents_for_test() -> Result<crate::accounting::AccountedVec<ResourceIntent>> {
    let account = crate::accounting::Account::for_test();
    let mut recording = RecordingBuilder::new(&account)?;
    let _buffer = recording.new_buffer(BufferRole::Scene, 4)?;
    let (_recording, intents) = recording.finish();
    Ok(intents)
}

impl AbortedVelloResources {
    pub(super) fn without_resources() -> Self {
        Self {
            #[cfg(test)]
            discarded_resource_count: 0,
            #[cfg(test)]
            atlas_outcome: VelloAtlasOutcome::NoAtlas,
        }
    }

    fn merge(self, _other: Self) -> Self {
        Self {
            #[cfg(test)]
            discarded_resource_count: self
                .discarded_resource_count
                .saturating_add(_other.discarded_resource_count),
            #[cfg(test)]
            atlas_outcome: match (self.atlas_outcome, _other.atlas_outcome) {
                (VelloAtlasOutcome::Recreate, _) | (_, VelloAtlasOutcome::Recreate) => {
                    VelloAtlasOutcome::Recreate
                }
                (VelloAtlasOutcome::MarkDirty, _) | (_, VelloAtlasOutcome::MarkDirty) => {
                    VelloAtlasOutcome::MarkDirty
                }
                #[cfg(test)]
                (VelloAtlasOutcome::Retain, _) | (_, VelloAtlasOutcome::Retain) => {
                    VelloAtlasOutcome::Retain
                }
                (VelloAtlasOutcome::NoAtlas, VelloAtlasOutcome::NoAtlas) => {
                    VelloAtlasOutcome::NoAtlas
                }
            },
        }
    }
}

#[cfg(test)]
impl AbortedVelloResources {
    pub(crate) const fn discarded_resource_count_for_test(&self) -> usize {
        self.discarded_resource_count
    }
}

#[cfg(test)]
impl AbortedVelloResources {
    pub(crate) fn into_atlas_outcome(self) -> VelloAtlasOutcome {
        self.atlas_outcome
    }
}

impl CommittedVelloResources {
    pub(crate) fn into_frame_cleanup(self) -> FrameCleanup {
        self.frame_cleanup
    }

    #[cfg(test)]
    pub(crate) const fn atlas_outcome(&self) -> VelloAtlasOutcome {
        self.atlas_outcome
    }
}

impl PendingVelloResources {
    fn frame_scope(&self) -> Result<&FrameResourceScope> {
        self.frame_scope
            .as_ref()
            .ok_or_else(|| render_failed("internal Vello resource frame was already resolved"))
    }

    fn frame_scope_mut(&mut self) -> Result<&mut FrameResourceScope> {
        self.frame_scope
            .as_mut()
            .ok_or_else(|| render_failed("internal Vello resource frame was already resolved"))
    }

    fn consume_pending_atlas_recovery(&mut self) {
        if let Some(scope) = self.frame_scope.as_mut() {
            scope.consume_vello_atlas_recovery();
        }
    }

    fn ensure_commit_ready(&self) -> Result<()> {
        let frame_scope = self.frame_scope()?;
        let leases = self
            .buffers
            .values()
            .map(|managed| &managed.lease)
            .chain(self.images.values().map(|managed| &managed.lease));
        frame_scope.ensure_commit_ready_iter(leases)
    }

    fn commit(&mut self) -> Result<CommittedVelloResources> {
        self.ensure_commit_ready()?;
        let atlas_handle = self.persistent_image_atlas.resource();
        #[cfg(test)]
        let atlas_outcome = self.persistent_image_atlas.commit_outcome();
        let mut frame_scope = self
            .frame_scope
            .take()
            .expect("a pending Vello commit must own its resource frame");

        for (_, managed) in self.buffers.drain() {
            match self.clean_retention {
                CleanVelloResourceRetention::DirectAtlasOnly => {
                    frame_scope.discard(managed.lease)?;
                }
                CleanVelloResourceRetention::ReusableGraphFrame => {
                    frame_scope.release(managed.lease)?;
                }
            }
        }

        let mut retained_atlas = None;
        for (handle, managed) in self.images.drain() {
            if Some(handle) == atlas_handle {
                retained_atlas = Some(managed.lease);
            } else {
                match self.clean_retention {
                    CleanVelloResourceRetention::DirectAtlasOnly => {
                        frame_scope.discard(managed.lease)?;
                    }
                    CleanVelloResourceRetention::ReusableGraphFrame => {
                        frame_scope.release(managed.lease)?;
                    }
                }
            }
        }
        if let Some(atlas) = retained_atlas {
            if self.clean_retention == CleanVelloResourceRetention::DirectAtlasOnly {
                frame_scope.retire_idle_vello_atlases()?;
            }
            frame_scope.release(atlas)?;
        }
        let frame_cleanup = frame_scope.finish_checked()?;

        Ok(CommittedVelloResources {
            frame_cleanup,
            #[cfg(test)]
            atlas_outcome,
        })
    }

    fn abort(&mut self) -> AbortedVelloResources {
        #[cfg(test)]
        let discarded_resource_count = self.buffers.len().saturating_add(self.images.len());
        let atlas_outcome = self.persistent_image_atlas.abort_outcome();
        if let Some(mut frame_scope) = self.frame_scope.take() {
            for (_, managed) in self.buffers.drain() {
                let _ = frame_scope.discard(managed.lease);
            }
            for (_, managed) in self.images.drain() {
                let _ = frame_scope.discard(managed.lease);
            }
            frame_scope.record_vello_atlas_recovery(atlas_outcome);
            let _ = frame_scope.finish();
        }
        AbortedVelloResources {
            #[cfg(test)]
            discarded_resource_count,
            #[cfg(test)]
            atlas_outcome,
        }
    }
}

impl Drop for PendingVelloResources {
    fn drop(&mut self) {
        if self.frame_scope.is_some() {
            let _ = self.abort();
        }
    }
}

fn preflight_resource_intents(
    limits: &wgpu::Limits,
    intents: &[ResourceIntent],
    generation: &crate::accounting::Generation,
    observer: Option<&crate::accounting::Account>,
) -> Result<()> {
    // Keep every synchronous intent failure before allocation so an error cannot lose
    // fresh-atlas provenance from a partially built lease.
    let mut buffer_handles = HashSet::new();
    let mut image_handles = HashSet::new();
    let mut has_persistent_image_atlas = false;
    for intent in intents {
        match intent {
            ResourceIntent::Buffer(buffer) => {
                preflight_buffer(limits, buffer)?;
                generation.probe_cache(observer)?;
                if !buffer_handles.insert(buffer.resource) {
                    return Err(render_failed(
                        "internal Vello resource allocation repeats a buffer identity",
                    ));
                }
            }
            ResourceIntent::Image(image) => {
                preflight_image(limits, image)?;
                generation.probe_cache(observer)?;
                if !image_handles.insert(image.resource) {
                    return Err(render_failed(
                        "internal Vello resource allocation repeats an image identity",
                    ));
                }
                if image.retention == ImageRetention::PersistentImageAtlas {
                    if has_persistent_image_atlas {
                        return Err(render_failed(
                            "internal Vello resource allocation repeats the persistent image atlas",
                        ));
                    }
                    has_persistent_image_atlas = true;
                }
            }
        }
    }
    Ok(())
}

fn preflight_buffer(limits: &wgpu::Limits, intent: &BufferIntent) -> Result<()> {
    if intent.byte_len == 0 {
        return Err(render_failed(
            "internal Vello resource allocation received an empty buffer",
        ));
    }
    if intent.byte_len > limits.max_buffer_size {
        return Err(render_failed(
            "internal Vello buffer exceeds the device limit before allocation",
        ));
    }
    let binding_limit = match intent.role {
        BufferRole::Config => limits.max_uniform_buffer_binding_size,
        _ => limits.max_storage_buffer_binding_size,
    };
    if intent.byte_len > binding_limit {
        return Err(render_failed(
            "internal Vello buffer exceeds its device binding-class limit before allocation",
        ));
    }
    Ok(())
}

fn preflight_image(limits: &wgpu::Limits, intent: &ImageIntent) -> Result<()> {
    if intent.extent.width() == 0 || intent.extent.height() == 0 {
        return Err(render_failed(
            "internal Vello resource allocation received an empty image",
        ));
    }
    let max_extent = limits.max_texture_dimension_2d;
    if intent.extent.width() > max_extent || intent.extent.height() > max_extent {
        return Err(render_failed(
            "internal Vello image exceeds the device 2D texture limit before allocation",
        ));
    }
    Ok(())
}

fn allocate_buffer(
    device: &wgpu::Device,
    pending: &mut PendingVelloResources,
    intent: &BufferIntent,
) -> Result<()> {
    if intent.byte_len == 0 {
        return Err(render_failed(
            "internal Vello resource allocation received an empty buffer",
        ));
    }
    pending.frame_scope()?.probe()?;
    if pending.buffers.contains_key(&intent.resource) {
        return Err(render_failed(
            "internal Vello resource allocation repeats a buffer identity",
        ));
    }
    let key = VelloBufferKey::from_intent(intent);
    let retention = pending.clean_retention;
    let lease = match retention {
        CleanVelloResourceRetention::DirectAtlasOnly => pending
            .frame_scope_mut()?
            .acquire_vello_buffer(device, key)?,
        CleanVelloResourceRetention::ReusableGraphFrame => pending
            .frame_scope_mut()?
            .acquire_reusable_vello_buffer(device, key)?,
    };
    pending.buffers.insert(
        intent.resource,
        ManagedBuffer {
            lease,
            byte_len: intent.byte_len,
        },
    );
    Ok(())
}

fn allocate_image(
    device: &wgpu::Device,
    pending: &mut PendingVelloResources,
    intent: &ImageIntent,
) -> Result<()> {
    if intent.extent.width() == 0 || intent.extent.height() == 0 {
        return Err(render_failed(
            "internal Vello resource allocation received an empty image",
        ));
    }
    pending.frame_scope()?.probe()?;
    if pending.images.contains_key(&intent.resource) {
        return Err(render_failed(
            "internal Vello resource allocation repeats an image identity",
        ));
    }
    if intent.retention == ImageRetention::PersistentImageAtlas
        && pending.persistent_image_atlas.is_present()
    {
        return Err(render_failed(
            "internal Vello resource allocation repeats the persistent image atlas",
        ));
    }

    let key = VelloImageKey::from_intent(intent);
    let byte_len = image_byte_len(intent.extent, intent.format)?;
    let retention = pending.clean_retention;
    let lease = match retention {
        CleanVelloResourceRetention::DirectAtlasOnly => pending
            .frame_scope_mut()?
            .acquire_vello_image(device, key, byte_len)?,
        CleanVelloResourceRetention::ReusableGraphFrame => pending
            .frame_scope_mut()?
            .acquire_reusable_vello_image(device, key, byte_len)?,
    };
    if intent.retention == ImageRetention::PersistentImageAtlas {
        pending.persistent_image_atlas = PendingPersistentAtlas::NewlyAllocated(intent.resource);
    }
    pending.images.insert(
        intent.resource,
        ManagedImage {
            lease,
            extent: intent.extent,
        },
    );
    Ok(())
}

fn image_byte_len(extent: PhysicalSize, format: RasterImageFormat) -> Result<u64> {
    let bytes_per_pixel = match format {
        RasterImageFormat::Rgba8Unorm => 4_u64,
    };
    u64::from(extent.width())
        .checked_mul(u64::from(extent.height()))
        .and_then(|pixels| pixels.checked_mul(bytes_per_pixel))
        .ok_or_else(|| {
            render_failed("internal Vello image byte length overflows the GPU address space")
        })
}

const fn texture_format(format: RasterImageFormat) -> wgpu::TextureFormat {
    match format {
        RasterImageFormat::Rgba8Unorm => wgpu::TextureFormat::Rgba8Unorm,
    }
}

const fn buffer_usage(role: BufferRole) -> wgpu::BufferUsages {
    match role {
        BufferRole::Config => wgpu::BufferUsages::UNIFORM.union(wgpu::BufferUsages::COPY_DST),
        _ => wgpu::BufferUsages::STORAGE
            .union(wgpu::BufferUsages::COPY_DST)
            .union(wgpu::BufferUsages::COPY_SRC)
            .union(wgpu::BufferUsages::INDIRECT),
    }
}

const fn image_usage(role: ImageRole) -> wgpu::TextureUsages {
    let usage = wgpu::TextureUsages::TEXTURE_BINDING.union(wgpu::TextureUsages::COPY_DST);
    match role {
        ImageRole::ImageAtlas => usage.union(wgpu::TextureUsages::STORAGE_BINDING),
        ImageRole::GradientRamp => usage,
    }
}

fn render_failed(message: &'static str) -> Error {
    Error::new(BackendErrorCode::RenderFailed, message)
}

#[cfg(test)]
mod tests {
    use super::VelloAtlasOutcome;

    #[test]
    fn pending_atlas_recovery_merge_preserves_the_stronger_outcome() {
        use VelloAtlasOutcome::{MarkDirty, NoAtlas, Recreate, Retain};

        assert_eq!(
            VelloAtlasOutcome::merge_pending_recovery(Some(MarkDirty), Recreate),
            Some(Recreate)
        );
        assert_eq!(
            VelloAtlasOutcome::merge_pending_recovery(Some(Recreate), MarkDirty),
            Some(Recreate)
        );
        assert_eq!(
            VelloAtlasOutcome::merge_pending_recovery(Some(Recreate), NoAtlas),
            Some(Recreate)
        );
        assert_eq!(
            VelloAtlasOutcome::merge_pending_recovery(Some(MarkDirty), Retain),
            Some(MarkDirty)
        );
    }

    #[test]
    fn aborting_allocated_empty_lease_backing_times_its_actual_disposal() {
        use super::{VelloResourceLease, abort_vello_leases};
        use crate::accounting::{
            AccountedVec, RenderAccountingOptions, RenderClock, RenderCpuScope, RenderProfile,
            RenderResource, RendererCpu, StorageAccount,
        };
        use std::sync::atomic::{AtomicU64, Ordering};
        use std::time::Duration;

        static TICKS: AtomicU64 = AtomicU64::new(0);
        fn advancing_clock() -> Duration {
            Duration::from_nanos(TICKS.fetch_add(1, Ordering::Relaxed))
        }

        TICKS.store(0, Ordering::Relaxed);
        let cpu = RendererCpu::new(
            RenderAccountingOptions::default()
                .with_clock(RenderClock::Host(advancing_clock))
                .with_profile(RenderProfile::Summary { timing: true }),
        )
        .unwrap();
        let storage = StorageAccount::Renderer(cpu.at(RenderCpuScope::FrameBacking));
        let baseline = cpu
            .snapshot()
            .unwrap()
            .usage()
            .live(RenderResource::ScratchCpuBytes);
        let leases = AccountedVec::<VelloResourceLease>::new_in(
            &storage,
            3,
            RenderResource::ScratchCpuBytes,
        )
        .unwrap();
        assert!(leases.is_empty());
        assert!(leases.capacity() >= 3);
        let backing_bytes = leases.capacity() * std::mem::size_of::<VelloResourceLease>();
        assert!(backing_bytes > 0);

        // Snapshot after allocation isolates the abort's real destruction from
        // allocation timing. The host clock remains monotonic throughout.
        let before = cpu.snapshot().unwrap();
        assert_eq!(
            before.usage().live(RenderResource::ScratchCpuBytes),
            baseline + backing_bytes as u64,
        );
        let before_elapsed = before.scope_elapsed(RenderCpuScope::FrameBacking).unwrap();
        let aborted = abort_vello_leases(leases);
        let after = cpu.snapshot().unwrap();
        assert_eq!(aborted.discarded_resource_count, 0);
        assert_eq!(aborted.atlas_outcome, VelloAtlasOutcome::NoAtlas);
        assert_eq!(
            after.usage().live(RenderResource::ScratchCpuBytes),
            baseline
        );
        assert_eq!(after.fault(), None);
        let after_elapsed = after.scope_elapsed(RenderCpuScope::FrameBacking).unwrap();
        assert!(
            after_elapsed > before_elapsed,
            "destruction of admitted abort-vector backing must add CPU phase time: before={before_elapsed:?}, after={after_elapsed:?}",
        );
    }

    #[test]
    fn empty_lease_abort_preserves_work_and_records_one_backing_disposal() {
        use super::{VelloResourceLease, abort_vello_leases};
        use crate::accounting::{
            AccountedVec, RenderAccountingOptions, RenderClock, RenderCpuScope, RenderProfile,
            RenderResource, RenderWork, RendererCpu, StorageAccount,
        };
        use std::sync::atomic::{AtomicU64, Ordering};
        use std::time::Duration;
        use surgeist_ledger::Dimension;

        static TICKS: AtomicU64 = AtomicU64::new(0);
        fn clock() -> Duration {
            Duration::from_nanos(TICKS.fetch_add(1, Ordering::Relaxed))
        }
        fn forbidden_clock() -> Duration {
            panic!("off and untimed empty-lease disposal must not read the clock")
        }
        for profile in [
            RenderProfile::Off,
            RenderProfile::Summary { timing: false },
            RenderProfile::Summary { timing: true },
        ] {
            for capacity in [0, 3] {
                let timed = matches!(profile, RenderProfile::Summary { timing: true });
                let cpu = RendererCpu::new(
                    RenderAccountingOptions::new()
                        .with_profile(profile)
                        .with_clock(RenderClock::Host(if timed {
                            clock
                        } else {
                            forbidden_clock
                        })),
                )
                .unwrap();
                let storage = StorageAccount::Renderer(cpu.at(RenderCpuScope::FrameBacking));
                let baseline = cpu
                    .snapshot()
                    .unwrap()
                    .usage()
                    .live(RenderResource::ScratchCpuBytes);
                let leases = AccountedVec::<VelloResourceLease>::new_in(
                    &storage,
                    capacity,
                    RenderResource::ScratchCpuBytes,
                )
                .unwrap();
                let bytes = (leases.capacity() * std::mem::size_of::<VelloResourceLease>()) as u64;
                assert!(leases.is_empty());
                let before = cpu.snapshot().unwrap();
                assert_eq!(
                    before.usage().live(RenderResource::ScratchCpuBytes),
                    baseline + bytes
                );
                let aborted = abort_vello_leases(leases);
                assert_eq!(aborted.discarded_resource_count, 0);
                assert_eq!(aborted.atlas_outcome, VelloAtlasOutcome::NoAtlas);
                let after = cpu.snapshot().unwrap();
                assert_eq!(
                    after.usage().live(RenderResource::ScratchCpuBytes),
                    baseline
                );
                assert_eq!(
                    after.usage().peak(RenderResource::ScratchCpuBytes),
                    before.usage().peak(RenderResource::ScratchCpuBytes)
                );
                assert_eq!(after.fault(), None);
                for &work in RenderWork::ALL {
                    assert_eq!(after.usage().work(work), before.usage().work(work));
                }
                let recorded = u64::from(!matches!(profile, RenderProfile::Off));
                assert_eq!(
                    after.scope_count(RenderCpuScope::FrameBacking)
                        - before.scope_count(RenderCpuScope::FrameBacking),
                    recorded
                );
                if timed {
                    assert_eq!(
                        after.scope_elapsed(RenderCpuScope::FrameBacking).unwrap()
                            - before.scope_elapsed(RenderCpuScope::FrameBacking).unwrap(),
                        Duration::from_nanos(1)
                    );
                } else {
                    assert_eq!(after.scope_elapsed(RenderCpuScope::FrameBacking), None);
                }
            }
        }
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
#[path = "resources/collection_tests.rs"]
mod collection_tests;

#[cfg(all(test, not(target_arch = "wasm32")))]
mod gradient_extent_tests {
    use super::*;

    #[test]
    fn expanded_gradient_extent_refuses_before_any_resource_allocation() {
        let mut renderer = pollster::block_on(crate::Renderer::new(crate::Options::default()))
            .expect("host adapter for actual device-limit preflight");
        let (device, _) = renderer.default_wgpu_device_queue().unwrap();
        let height = device
            .limits()
            .max_texture_dimension_2d
            .checked_add(1)
            .expect("host texture limit permits one larger descriptor");
        let account = crate::accounting::Account::for_test();
        let mut recording = RecordingBuilder::new(&account).unwrap();
        // A valid request precedes the oversized gradient image. The complete
        // intent list must be admitted before even that first buffer allocates.
        recording.new_buffer(BufferRole::Scene, 4).unwrap();
        recording
            .new_transient_image(ImageRole::GradientRamp, PhysicalSize::new(512, height))
            .unwrap();
        let (_recording, intents) = recording.finish();
        let manager = ResourceManager::default();
        let before = manager.stats();
        let scope = ActiveVelloEncodingScope::begin(device);
        let error = match VelloResourceLease::allocate(&scope, &manager, &intents, None) {
            Ok(lease) => {
                let _aborted = lease.abort();
                panic!("an oversized endpoint descriptor texture must refuse")
            }
            Err(error) => error,
        };
        assert_eq!(error.code(), crate::ErrorCode::RenderFailed);
        assert!(error.message().contains("device 2D texture limit"));
        assert_eq!(manager.stats(), before);
        assert_eq!(manager.live_count(), 0);
        pollster::block_on(scope.finish(&account)).unwrap();
    }
}
