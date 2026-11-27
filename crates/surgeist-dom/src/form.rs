//! Form-associated element state: form, button, fieldset, input, output and textarea.
//!
//! `DomForm`, `DomButton`, `DomFieldset`, `DomInput`, `DomOutput` and `DomTextarea` are
//! checked views at one immutable revision; all but the Form have a `*State` value for
//! the retained native state. `DomTextareaReadLimits` bounds Textarea string reads.
//! Select and its options live in `form::select`.
//! `DomHtmlFormControlsCollection` retains the live controls of one exact Form.
//!
//! Current and default values stay distinct for Input, Output and Textarea: an Output
//! keeps one nullable default-value override, and a Textarea's `defaultValue` remains its
//! immediate child Text. `docs/forms-and-select.md`, `docs/input.md` and
//! `docs/text-controls.md` own the contracts.

pub mod select;

pub use crate::internal::button::{DomButton, DomButtonState};
pub use crate::internal::fieldset::{DomFieldset, DomFieldsetState};
pub use crate::internal::form::DomForm;
pub use crate::internal::input::{DomInput, DomInputState};
pub use crate::internal::output::{DomOutput, DomOutputState};
pub use crate::internal::store::{
    DomFormNamedValue, DomHtmlFormControlsCollection, DomHtmlFormControlsCollectionRead,
    DomRadioNodeList, DomRadioNodeListRead,
};
pub use crate::internal::textarea::{DomTextarea, DomTextareaReadLimits, DomTextareaState};
