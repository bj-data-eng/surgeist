//! Select, option and selectedcontent state and the select task protocol.
//!
//! `DomSelect`, `DomOption` and `DomSelectedcontent` are the checked views, and
//! `DomSelectState` and `DomOptionState` read the native Select scalars and an Option's
//! selectedness and dirty flags. `DomSelectTask`, `DomSelectCompletionStart` and the
//! related completion types carry an issued Select intention through the native
//! checkpoint; canonical FIFO work stays pending when every clone of a task's permission
//! is dropped.
//!
//! `docs/forms-and-select.md` owns the contract.

pub use crate::internal::select::{
    DomOption, DomOptionState, DomSelect, DomSelectCompletionStart, DomSelectOptionBefore,
    DomSelectState, DomSelectTask, DomSelectTaskCancellationReason, DomSelectTaskCompletion,
    DomSelectTaskOutcome, DomSelectedcontent,
};
