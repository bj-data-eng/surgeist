//! Shadow roots, slots, the flat tree and Template contents.
//!
//! `DomShadowRoot` and `DomShadowRootInit` describe attachment and the checked root.
//! `DomSlot` reads a Slot's stored lists, which can differ from fresh queries;
//! `DomSlotNodes` is an owned query result, and `DomFlatTree` and `DomFlatRelation` read
//! flat-tree membership under `DomSlotQueryLimits`. `DomPreparedSlotAssignment` keeps an
//! assignment's operands and admission together until Source adopts or releases them.
//! `DomTemplate` and `DomTemplatePatch` read the separately owned Template contents.
//!
//! Manual intent, stored assignment and fresh Slot queries are separate;
//! `DomSlotNotifications` hands slotchange intentions to the explicit native checkpoint
//! in `task`, and no callback runs inside a transaction. `docs/shadow-and-template.md`
//! and `docs/slots-and-checkpoints.md` own the contracts.

pub use crate::internal::shadow::{DomShadowRootInit, DomShadowRootState};
pub use crate::internal::slot::{
    DomFlatRelation, DomFlatTree, DomSlot, DomSlotNodes, DomSlotNotificationTarget,
    DomSlotNotifications, DomSlotQueryLimits, DomSlotSignalCancellationReason,
};
pub use crate::internal::snapshot::DomShadowRoot;
pub use crate::internal::source::DomPreparedSlotAssignment;
pub use crate::internal::template::{DomTemplate, DomTemplateDocumentRole, DomTemplatePatch};
