//! Event parents, recipients, dispatch custody and the Source event firing protocol.
//!
//! `DomEventParent` and `DomEventParentQuery` read the current event-parent topology,
//! while the runtime keeps path construction and invocation. `DomEventRecipient`,
//! `DomEventRecipientTarget` and `DomEventRecipientLimits` give a Load or Error listener
//! a positive recipient capability, `DomEventRegistryAssociation` and
//! `DomEventRegistryPreparation` tie the one live listener-registry owner to a DOM, and
//! `DomDispatchCustody` and `DomDispatchCustodyLimits` hold affine custody of exact
//! invocation targets (zero capacity by default). The `DomSourceEvent*` types are the
//! Source continuation through which an operation fires an event and receives its
//! outcome.
//!
//! `docs/dialogs-and-focus.md` (qualified dispatch reads) and `docs/source-and-style.md`
//! (dispatch target custody) own the contracts.

pub use crate::internal::dispatch::{
    DomDispatchAssociation, DomDispatchCustody, DomDispatchCustodyLimits,
};
pub use crate::internal::event::{
    DomEventParent, DomEventParentQuery, DomEventPayloadRetention, DomEventRecipient,
    DomEventRecipientLimits, DomEventRecipientTarget, DomEventRegistryAssociation,
    DomEventRegistryPreparation,
};
pub use crate::internal::source::{
    DomSourceAdvance, DomSourceEventClockFailure, DomSourceEventClockMode,
    DomSourceEventDispatchGuard, DomSourceEventDispatchPermit, DomSourceEventExecutor,
    DomSourceEventFiring, DomSourceEventId, DomSourceEventKind, DomSourceEventOutcome,
    DomSourceEventPayloadRetention, DomSourceEventPublication, DomSourceEventRequest,
    DomSourceEventRetention, DomSourceEventTime, DomSourceToggleEvent,
};
pub use crate::internal::store::source::operation::DomSourceEventFiringLoan;
