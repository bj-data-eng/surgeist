//! Affine results from the original Window manager, consumed by Source receipts.
use crate::{DomCloseWatcherId, DomSourceDialogRequest};
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct HookOccurrence {
    pub(crate) command: usize,
    pub(crate) ordinal: u64,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DialogOccurrence {
    Hook(HookOccurrence),
    Method(HookOccurrence),
}
/// Exact source-qualified manager effect. The trusted owner performs the real
/// callback-free registration/removal before publishing this capability.
#[derive(Debug)]
pub struct DomDialogWatcherPreparation {
    pub(crate) request: DomSourceDialogRequest,
    pub(crate) watcher: DomCloseWatcherId,
}
impl DomDialogWatcherPreparation {
    pub const fn watcher(&self) -> DomCloseWatcherId {
        self.watcher
    }
    pub fn request(&self) -> &DomSourceDialogRequest {
        &self.request
    }
    /// Attest the prepared actual Window-manager effect, without a DOM fence.
    pub fn publish(self) -> DomDialogWatcherPublication {
        DomDialogWatcherPublication {
            request: self.request,
            watcher: self.watcher,
        }
    }
}
/// Move-only actual manager effect retained until exact Source acceptance.
#[derive(Debug)]
pub struct DomDialogWatcherPublication {
    pub(crate) request: DomSourceDialogRequest,
    pub(crate) watcher: DomCloseWatcherId,
}
