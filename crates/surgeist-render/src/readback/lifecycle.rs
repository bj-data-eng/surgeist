use crate::accounting::{Account, OwnedGpu, ReadbackCleanupTail};
use std::sync::{Arc, Mutex};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum ReadbackStagingDisposition {
    Idle,
    MapPending,
    MappedActive,
    Released,
}

pub(super) struct ReadbackStagingMapState {
    disposition: Mutex<ReadbackStagingDisposition>,
}
impl ReadbackStagingMapState {
    pub(super) fn completed(&self, succeeded: bool) {
        let mut disposition = self
            .disposition
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        // The once-owned callback is the only physical completion producer.
        // A late callback cannot resurrect locally released staging.
        if *disposition == ReadbackStagingDisposition::MapPending {
            *disposition = if succeeded {
                ReadbackStagingDisposition::MappedActive
            } else {
                ReadbackStagingDisposition::Idle
            };
        }
    }
    fn release(&self) -> bool {
        let previous = std::mem::replace(
            &mut *self
                .disposition
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
            ReadbackStagingDisposition::Released,
        );
        matches!(
            previous,
            ReadbackStagingDisposition::MapPending | ReadbackStagingDisposition::MappedActive
        )
    }
    #[cfg(all(test, not(target_arch = "wasm32")))]
    pub(super) fn disposition_for_test(&self) -> ReadbackStagingDisposition {
        *self
            .disposition
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

/// Mandatory native staging drops before its accounting phase tail.
/// The callback's independent clone can retain shared custody afterward.
pub(super) struct ReadbackStaging {
    pub(super) buffer: OwnedGpu<wgpu::Buffer>,
    #[cfg(test)]
    pub(super) probe: super::test_support::BufferDropProbe,
    cleanup: ReadbackCleanupTail,
    map: Arc<ReadbackStagingMapState>,
}
impl ReadbackStaging {
    pub(super) fn new(buffer: OwnedGpu<wgpu::Buffer>, account: &Account) -> Self {
        Self {
            buffer,
            #[cfg(test)]
            probe: super::test_support::BufferDropProbe::default(),
            map: Arc::new(ReadbackStagingMapState {
                disposition: Mutex::new(ReadbackStagingDisposition::Idle),
            }),
            cleanup: account.clone().readback_cleanup_tail(),
        }
    }
    pub(super) fn account(&self) -> &Account {
        self.cleanup.account()
    }
    pub(super) fn register(&self) -> Arc<ReadbackStagingMapState> {
        // Consuming registration calls this once, before map_async can deliver.
        *self
            .map
            .disposition
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) =
            ReadbackStagingDisposition::MapPending;
        self.map.clone()
    }
    #[cfg(all(test, not(target_arch = "wasm32")))]
    pub(super) fn observation(&self) -> Arc<ReadbackStagingMapState> {
        self.map.clone()
    }
}
impl Drop for ReadbackStaging {
    fn drop(&mut self) {
        self.cleanup.begin();
        if self.map.release() {
            self.buffer.unmap();
            #[cfg(test)]
            if let Some(observation) = &self.probe.0 {
                observation
                    .record(super::test_support::ReadbackCleanupEventForTest::StagingUnmapped);
            }
        }
        // Rust destroys buffer (native value then hold handle), the cleanup
        // tail, then unrelated map metadata. No resource is extracted/replaced.
    }
}
