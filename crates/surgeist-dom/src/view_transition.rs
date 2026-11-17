//! View Transition structural state, pseudo-elements, rules and sheets.
//!
//! `DomViewTransition`, `DomViewTransitionId` and `DomViewTransitionRecord` read admitted
//! transitions and participant records, `DomViewTransitionPseudo` and its children read
//! generated pseudo relations, and `DomViewTransitionRules`, `DomViewTransitionSheet` and
//! the `DomPreparedViewTransition*` types carry the CSSOM associations.
//! `CreatedViewTransition` and `CreatedViewTransitionRecord` are batch-local creation
//! tickets.
//!
//! The capability is explicitly initialized and starts with zero capacity
//! (`DomLimits::with_view_transitions`). It stores structural state only; the runtime
//! owns rendering resources. `docs/view-transitions.md` owns the contract.

pub use crate::internal::view_transition::{
    CreatedViewTransition, CreatedViewTransitionRecord, DomDocumentViewTransitions,
    DomPreparedViewTransitionRules, DomPreparedViewTransitionSheet, DomViewTransition,
    DomViewTransitionBranches, DomViewTransitionCaptureLimits, DomViewTransitionCssomError,
    DomViewTransitionId, DomViewTransitionImageRules, DomViewTransitionPseudo,
    DomViewTransitionPseudoChildren, DomViewTransitionPseudoId, DomViewTransitionPseudoKind,
    DomViewTransitionRecord, DomViewTransitionRecordId, DomViewTransitionRuleAssociations,
    DomViewTransitionRules, DomViewTransitionSheet, DomViewTransitionSubject,
    ViewTransitionOperand, ViewTransitionRecordOperand,
};
