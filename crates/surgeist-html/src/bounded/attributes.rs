//! Local attribute processing with explicit operands and caller admission.
//!
//! Classification and grammar remain HTML-owned. These operations establish no
//! captured-tree, resource or host evidence, and introduce no controller.

pub use crate::internal::attributes::autocomplete::{
    HtmlAutocompleteControl, HtmlAutocompleteIdl, HtmlAutocompleteMaterializationError,
    HtmlAutocompleteMaterializationFailure, HtmlAutocompleteMaterializationLimits,
    HtmlAutocompleteMaterializationUsage, HtmlAutocompleteMaterialized,
    autocomplete_materialization_work_bound, control_autocomplete_idl_with,
};
pub use crate::internal::attributes::bounded_rules::{
    HtmlAttributeKeywordRead, HtmlButtonSubmitterFact, HtmlHiddenSetterAction,
    HtmlHiddenSetterInput, HtmlIntegerAttributeRead, HtmlWritingSuggestionsProcessing,
    button_submitter_context_with, button_type_keyword_with, closedby_processing_with,
    contenteditable_processing_with, contenteditable_setter_state_with,
    crossorigin_processing_with, dir_keyword_with, enterkeyhint_keyword_with,
    fetchpriority_processing_with, form_autocomplete_keyword_with, form_enctype_keyword_with,
    form_enctype_override_keyword_with, form_method_keyword_with,
    form_method_override_keyword_with, heading_offset_reflection_with, heading_offset_setter_value,
    hidden_processing_with, hidden_setter_action_with, input_list_applies, input_readonly_applies,
    input_submitter_context, inputmode_keyword_with, popover_keyword_with,
    referrerpolicy_processing_with, stylesheet_link_context_work_bound,
    template_shadow_root_mode_keyword_with, template_shadow_root_slot_assignment_keyword_with,
    translate_processing_with,
};
pub use crate::internal::attributes::command::{
    button_command_readback_with, popover_target_action_with,
};
pub use crate::internal::attributes::control_length::{
    HtmlControlDimensionRule, HtmlControlIntegerIndexSize, control_length_reflection_with,
    control_length_setter_value,
};
pub use crate::internal::attributes::hash_name::{HtmlHashNameScan, HtmlHashNameScanStep};
pub use crate::internal::attributes::image_values::{
    HtmlEmbeddedDimensionApplicability, HtmlEmbeddedDimensionApplicabilityContext,
    HtmlEmbeddedDimensionEvidenceNeeded, HtmlEmbeddedDimensionParent,
    embedded_dimension_applicability,
};
pub use crate::internal::attributes::link_destination::link_as_keyword_with;
pub use crate::internal::attributes::meta_pragma::meta_pragma_state_with;
pub use crate::internal::attributes::meta_processing::{
    HtmlMetaRefreshAction, HtmlMetaRefreshSyntaxTarget, meta_language_token_with,
    meta_referrer_policy_with, meta_refresh_syntax_with,
};
pub use crate::internal::attributes::navigable_target::HtmlTargetRecovery;
pub use crate::internal::attributes::scalar::table_span::HtmlTableSpanRule;
pub use crate::internal::attributes::table_header_scope::bounded_scope::{
    table_cell_scope_keyword_with, th_scope_state, th_scope_work_bound,
};
pub use crate::internal::attributes::token_declarations::{HtmlIconSizeCursor, HtmlIconSizeShape};
