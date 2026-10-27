//! The support catalogue and its lookups.

pub(crate) mod catalog;

#[doc(inline)]
pub use catalog::{
    CssConformanceExclusionId, CssConformanceSupersedingId, CssExclusionMetadata,
    CssExclusionReason, CssFeatureKind, CssFeatureMetadata, CssPropertySupportMetadata,
    CssSpecificationSource, CssSpecificationSourceId, CssSpecificationTier, CssSupportStatus,
    conformance_exclusion, conformance_exclusions, feature_catalog, feature_metadata,
    property_support_metadata, specification_source, specification_sources,
};
