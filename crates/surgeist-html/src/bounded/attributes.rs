//! Local attribute processing with explicit operands and caller admission.
//!
//! Classification and grammar remain HTML-owned. These operations establish no
//! captured-tree, resource or host evidence, and introduce no controller.

pub use crate::internal::attributes::autocomplete::{
    HtmlAutocompleteMaterializationError, HtmlAutocompleteMaterializationFailure,
    HtmlAutocompleteMaterializationLimits, HtmlAutocompleteMaterializationUsage,
    HtmlAutocompleteMaterialized, autocomplete_materialization_work_bound,
};
pub use crate::internal::attributes::bounded_rules::{
    HtmlAttributeKeywordRead, HtmlButtonSubmitterFact, HtmlIntegerAttributeRead,
    HtmlWritingSuggestionsProcessing, button_submitter_context_with, crossorigin_processing_with,
    dir_keyword_with, heading_offset_reflection_with, heading_offset_setter_value,
    input_list_applies, input_readonly_applies, input_submitter_context, popover_keyword_with,
    referrerpolicy_processing_with, stylesheet_link_context_work_bound,
    template_shadow_root_mode_keyword_with, template_shadow_root_slot_assignment_keyword_with,
};
pub use crate::internal::attributes::hash_name::{HtmlHashNameScan, HtmlHashNameScanStep};
pub use crate::internal::attributes::image_values::{
    HtmlEmbeddedDimensionApplicability, HtmlEmbeddedDimensionApplicabilityContext,
    HtmlEmbeddedDimensionEvidenceNeeded, HtmlEmbeddedDimensionParent,
    embedded_dimension_applicability,
};
pub use crate::internal::attributes::meta_pragma::meta_pragma_state_with;
pub use crate::internal::attributes::meta_processing::{
    HtmlMetaRefreshAction, HtmlMetaRefreshSyntaxTarget, meta_language_token_with,
    meta_referrer_policy_with, meta_refresh_syntax_with,
};
pub use crate::internal::attributes::navigable_target::HtmlTargetRecovery;
pub use crate::internal::attributes::scalar::table_span::HtmlTableSpanRule;
pub use crate::internal::attributes::table_header_scope::bounded_scope::{
    th_scope_state, th_scope_work_bound,
};
pub use crate::internal::attributes::token_declarations::{HtmlIconSizeCursor, HtmlIconSizeShape};
