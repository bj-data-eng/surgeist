//! Local attribute processing with explicit operands and caller admission.
//!
//! Classification and grammar remain HTML-owned. These operations establish no
//! captured-tree, resource or host evidence, and introduce no controller.

pub use crate::attributes::autocomplete::{
    AutocompleteMaterializationError, AutocompleteMaterializationFailure,
    AutocompleteMaterializationLimits, AutocompleteMaterializationUsage, AutocompleteMaterialized,
    autocomplete_materialization_work_bound,
};
pub use crate::attributes::bounded_rules::{
    HtmlAttributeKeywordRead, WritingSuggestionsProcessing, crossorigin_processing_with,
    input_readonly_applies, referrerpolicy_processing_with, stylesheet_link_context_work_bound,
};
pub use crate::attributes::hash_name::{HashNameScan, HashNameScanStep};
pub use crate::attributes::image_values::{
    EmbeddedDimensionApplicability, EmbeddedDimensionContext, EmbeddedDimensionEvidenceNeeded,
    EmbeddedDimensionParent, embedded_dimension_applicability,
};
pub use crate::attributes::navigable_target::TargetRecovery;
pub use crate::attributes::token_declarations::{IconSizeCursor, IconSizeShape};
