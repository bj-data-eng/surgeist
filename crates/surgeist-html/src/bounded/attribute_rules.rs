//! Local attribute processing with explicit operands and caller admission.
//!
//! Classification and grammar remain HTML-owned. These operations establish no
//! captured-tree, resource or host evidence, and introduce no controller.

pub use crate::internal::attributes::autocomplete::{
    AutocompleteMaterializationError, AutocompleteMaterializationFailure,
    AutocompleteMaterializationLimits, AutocompleteMaterializationUsage, AutocompleteMaterialized,
    autocomplete_materialization_work_bound,
};
pub use crate::internal::attributes::bounded_rules::{
    HtmlAttributeKeywordRead, WritingSuggestionsProcessing, crossorigin_processing_with,
    input_readonly_applies, referrerpolicy_processing_with, stylesheet_link_context_work_bound,
};
pub use crate::internal::attributes::bounded_scope::{th_scope_state, th_scope_work_bound};
pub use crate::internal::attributes::hash_name::{HashNameScan, HashNameScanStep};
pub use crate::internal::attributes::image_values::{
    EmbeddedDimensionApplicability, EmbeddedDimensionContext, EmbeddedDimensionEvidenceNeeded,
    EmbeddedDimensionParent, embedded_dimension_applicability,
};
pub use crate::internal::attributes::meta_pragma_state_with;
pub use crate::internal::attributes::meta_processing::{
    HtmlMetaRefreshAction, HtmlMetaRefreshSyntaxTarget, meta_language_token_with,
    meta_referrer_policy_with, meta_refresh_syntax_with,
};
pub use crate::internal::attributes::navigable_target::TargetRecovery;
pub use crate::internal::attributes::table_span::HtmlTableSpanRule;
pub use crate::internal::attributes::token_declarations::{IconSizeCursor, IconSizeShape};
