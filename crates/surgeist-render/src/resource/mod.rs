mod active_frames;
mod convolution;
mod entries;
mod gaussian;
mod lease;
mod manager;
#[cfg(test)]
mod test_support;

pub(crate) use convolution::ConvolutionWeightsPlan;
pub(crate) use gaussian::{
    GaussianKernelBufferLimits, GaussianKernelGeometry, GaussianKernelKey, GaussianKernelPlan,
    GaussianKernelSamplingForm,
};
#[cfg(test)]
pub(crate) use lease::ResourceRetentionOutcome;
pub(crate) use lease::{FrameCleanup, FrameResourceScope, ResourceLease};
pub(crate) use manager::{
    AllocationGeneration, FrameIdentity, ManagerIdentity, ResourceAccountingFault,
    ResourceAllocationPreflight, ResourceCacheKey, ResourceIdentity,
};
#[cfg(test)]
pub(crate) use test_support::{ResourceLeaseTokenForTest, ResourceManagerObservationForTest};

use super::{Error, ResourceCacheBudget, Result};
use crate::accounting::{Account, CpuAccount, Generation};
#[cfg(test)]
use crate::accounting::{RenderAccountingOptions, RenderCpuScope, RendererCpu};
use manager::ResourceManagerState;
use std::sync::{Arc, Mutex, MutexGuard};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum WorkingFormat {
    HighPrecision,
    ReducedPrecision,
}

impl WorkingFormat {
    pub(crate) const fn texture_format(self) -> wgpu::TextureFormat {
        match self {
            Self::HighPrecision => wgpu::TextureFormat::Rgba16Float,
            Self::ReducedPrecision => wgpu::TextureFormat::Rgba8Unorm,
        }
    }

    pub(crate) const fn required_usages(self) -> wgpu::TextureUsages {
        let _ = self;
        wgpu::TextureUsages::RENDER_ATTACHMENT
            .union(wgpu::TextureUsages::TEXTURE_BINDING)
            .union(wgpu::TextureUsages::COPY_SRC)
            .union(wgpu::TextureUsages::COPY_DST)
    }

    pub(crate) const fn required_format_features(self) -> wgpu::TextureFormatFeatureFlags {
        let _ = self;
        wgpu::TextureFormatFeatureFlags::FILTERABLE
    }

    pub(crate) fn is_supported_by(self, features: wgpu::TextureFormatFeatures) -> bool {
        features.allowed_usages.contains(self.required_usages())
            && features.flags.contains(self.required_format_features())
    }
}

/// The sole private owner of resource identity, lease state, accounting, and
/// deterministic idle retention for one device generation.
pub(crate) struct ResourceManager {
    state: Arc<Mutex<ResourceManagerState>>,
    generation: Generation,
}

impl ResourceManager {
    pub(crate) fn with_generation(
        budget: ResourceCacheBudget,
        generation: Generation,
        cpu: CpuAccount,
    ) -> Self {
        Self {
            state: Arc::new(Mutex::new(ResourceManagerState {
                generation: generation.clone(),
                cpu,
                identity: ManagerIdentity::new(),
                budget,
                next_resource: 0,
                accounting_fault: None,
                active_frames: active_frames::ActiveFrames::default(),
                entries: entries::ResourceEntries::default(),
                pending_vello_atlas_recovery: None,
                #[cfg(test)]
                payload_creation_attempts: 0,
            })),
            generation,
        }
    }

    pub(crate) fn generation(&self) -> &Generation {
        &self.generation
    }

    #[cfg(test)]
    pub(crate) fn new(budget: ResourceCacheBudget) -> Self {
        Self::with_generation(
            budget,
            Generation::new(RenderAccountingOptions::default()).expect("valid fixture generation"),
            RendererCpu::new(RenderAccountingOptions::default())
                .expect("valid fixture CPU owner")
                .at(RenderCpuScope::DeviceBacking),
        )
    }

    pub(crate) fn begin_frame(&self, observer: Option<&Account>) -> Result<FrameResourceScope> {
        let mut state = lock_state(&self.state);
        state.cpu.ensure_healthy()?;
        state.ensure_accounting_exact(observer)?;
        let ResourceManagerState {
            active_frames,
            entries,
            cpu,
            ..
        } = &mut *state;
        let frame = active_frames.open(entries.len(), cpu)?;
        Ok(FrameResourceScope::new(
            Arc::clone(&self.state),
            state.identity.clone(),
            frame,
            self.generation.clone(),
            observer.cloned(),
        ))
    }

    pub(crate) fn preflight_graph_acquisitions(
        &self,
        requests: &[ResourceAllocationPreflight],
        observer: Option<&Account>,
    ) -> Result<()> {
        lock_state(&self.state).preflight_graph_acquisitions(requests, observer)
    }
}

#[cfg(test)]
impl Default for ResourceManager {
    fn default() -> Self {
        Self::new(ResourceCacheBudget::DEFAULT)
    }
}

fn lock_state(state: &Arc<Mutex<ResourceManagerState>>) -> MutexGuard<'_, ResourceManagerState> {
    state
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}
