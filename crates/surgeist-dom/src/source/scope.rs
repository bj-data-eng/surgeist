//! Historical entry association; observations grant no execution authority.
use crate::{DomHostedGlobalId, DomHostedHostId};
/// The original Source entry's retained execution association.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DomSourceExecutionEntryScope {
    /// A fresh native entry may still receive its one explicit Host binding.
    NativeEntry,
    /// Native entry advanced without an explicit Host; resume cannot bind it.
    Unbound,
    /// Original creation Global, even after adoption, inactivity or retirement.
    Hosted {
        original_global: DomHostedGlobalId,
        original_host: Option<DomHostedHostId>,
    },
    /// Exact explicit Host selected before the first native entry advance.
    ExplicitHost(DomHostedHostId),
}
