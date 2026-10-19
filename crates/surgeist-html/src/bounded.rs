//! Controlled HTML-owned operations, driven by a caller's admission policy.
//!
//! Actions describe their work before reading their operands. This module owns
//! value invariants; callers retain cancellation, work and allocation accounting.

mod attribute_rules;
mod local;
mod media;
mod microdata;
mod numeric;
mod rel;
mod time_element;
pub use attribute_rules::*;
pub use local::*;
pub use media::*;
pub use microdata::*;
pub use numeric::*;
pub use rel::*;
pub use time_element::*;

pub use crate::child_navigable::{frame_iframe_after_ancestors_with, frame_iframe_src_with};

pub use crate::fragment_initialization::{
    HtmlFragmentNameTest, fragment_initial_modes, stack_reset_insertion_mode,
};
pub use crate::integration_points::{
    HtmlIntegrationPointComparison, HtmlIntegrationPointRule, HtmlMathmlTextPointTest,
    html_integration_point_encoding_attribute, html_integration_point_rule,
    mathml_text_integration_point,
};
pub use crate::values::bounded::{
    HtmlAttributeStorage, HtmlAttributeValidation, HtmlAttributeValidationAction,
    HtmlAttributeValidationError, HtmlAttributeValidationLayout, HtmlValidatedAttributes,
};
pub use crate::values::sharing::{
    HtmlAttributePool, HtmlAttributePoolCandidate, HtmlAttributePoolOperation,
    HtmlAttributePoolResult, HtmlStringPool, HtmlStringPoolCandidate, HtmlStringPoolOperation,
    HtmlStringPoolResult, HtmlValuePoolAction, HtmlValuePoolActionKind, HtmlValuePoolChange,
    HtmlValuePoolError, HtmlValuePoolLayout, HtmlValuePoolMark, HtmlValuePoolOccurrence,
    HtmlValuePoolProgress, HtmlValuePoolRoot, HtmlValuePoolSeal, HtmlValuePoolUsage,
    HtmlValuePoolWork,
};

pub use crate::construction::bounded::*;

pub use crate::form_association::native_form_association_with;
pub use crate::form_owner::{
    HtmlFormOwnerLookup, HtmlFormOwnerResetAction, HtmlFormOwnerResetCursor,
    HtmlFormOwnerResetCursorError, HtmlFormOwnerResetDecision, HtmlFormOwnerResetReply,
    HtmlFormOwnerResetTrigger, form_owner_lookup, form_owner_reset_trigger,
};
pub use crate::native_initialization::{
    HtmlElementNameRead, element_classification_with, native_element_initialization_with,
};

pub use crate::script_preparation::{
    HtmlScriptPreparationAction, HtmlScriptPreparationCursor, HtmlScriptPreparationError,
    HtmlScriptPreparationReply, HtmlScriptPreparationResult,
};

pub use crate::select::{
    HtmlOptionDisabledAncestor, HtmlOptionSelectAncestorAction, HtmlOptionSelectAncestors,
    HtmlSelectAction, HtmlSelectCursor, HtmlSelectCursorError, HtmlSelectFirstSelectedAction,
    HtmlSelectFirstSelectedCursor, HtmlSelectFirstSelection, HtmlSelectOperation,
    HtmlSelectOptionRule, HtmlSelectReply, HtmlSelectSetterAction, HtmlSelectSetterCursor,
    HtmlSelectSetterReply, HtmlSelectedcontentAncestorAction, HtmlSelectedcontentAncestors,
    HtmlSelectedcontentUpdate, option_disabled_ancestor, select_display_size_default,
    select_option_rule, selectedcontent_update,
};

pub use crate::slot_assign::{
    HtmlSlotAssignAction, HtmlSlotAssignCursor, HtmlSlotAssignCursorError, HtmlSlotAssignReply,
};

pub use crate::input_range::{
    HtmlInputRangeNumber, HtmlInputRangeReplacement, HtmlInputRangeSanitizationAction,
    HtmlInputRangeSanitizationCursor, HtmlInputRangeSanitizationReply,
    HtmlInputRangeSanitizationReplyError, HtmlInputRangeValue,
};
