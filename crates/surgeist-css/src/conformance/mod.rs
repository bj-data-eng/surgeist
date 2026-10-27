//! The support catalogue and its lookups.
//!
//! The support catalogue records, for each declared production, which dated specification it
//! follows and whether this crate supports it completely, partially or only recognizes it.
//! [`feature_catalog`], [`feature_metadata`], [`specification_source`], [`conformance_exclusion`]
//! and [`property_support_metadata`] look entries up; none of them changes what a parse accepts.
//!
//! # Complete catalogue entries
//!
//! The generic rule, declaration, and list shells and the shared values
//! `syntax-token-stream`, `component-value`, `simple-block`, `function`,
//! `declaration-value`, `any-value`, `an-plus-b`, `unicode-range`,
//! `css-wide-keyword`, `custom-ident`, `ident`, `string`, `url`, and
//! `url-modifier` are Complete catalog entries, as are the Values 3 records for
//! `dimension`, `angle`, `angle-percentage`, `time-percentage`, `frequency`,
//! `frequency-percentage`, and `calc()`.
//!
//! The `ext.value.relative-color`, `ext.value.color-mix`, and
//! `ext.supports.selector` records are Partial with explicit subset and
//! remainder metadata. `@font-face` and `@font-feature-values` provide complete
//! selected authored rule construction and effective specified serialization.
//! The Fonts 4 index-source contradiction remains explicit in the reference.
//! [`feature_catalog`] owns its current cardinality, which is distinct from the
//! immutable official inventory of property units, normative legacy shorthands,
//! and non-property units; [`conformance_exclusions`] owns the exclusion rows.
//!
//! # Support metadata and application policy
//!
//! [`feature_catalog`] describes each declared conformance production as
//! [`CssSupportStatus::Complete`], [`CssSupportStatus::Partial`], or
//! [`CssSupportStatus::RecognizedUnsupported`]. Partial records state both their
//! accepted subset and scope not claimed as supported. A remainder may describe
//! known-valid unimplemented syntax or unresolved standard applicability; it does
//! not establish that every other form is valid CSS. A diagnostic-free use of a
//! partial production's accepted subset is still a clean parse.
//!
//! The source registry assigns every selected dated specification or preserved
//! repository baseline a stable [`CssSpecificationSourceId`], module, level, and
//! [`CssSpecificationTier`]. A tier classifies provenance only; it never implies
//! parser support. Each source has exactly one immutable specification URL or
//! repository provenance value. [`specification_source`], [`feature_metadata`],
//! and [`conformance_exclusion`] use exact, case-sensitive IDs without trimming
//! or aliasing.
//!
//! ```
//! use surgeist_css::conformance::{CssExclusionReason, CssFeatureKind, CssSpecificationTier, CssSupportStatus, conformance_exclusion, feature_metadata, specification_source};
//!
//! let color = specification_source("O-COLOR4").expect("dated Color 4 source");
//! assert_eq!(color.tier(), CssSpecificationTier::Snapshot2026Official);
//! assert!(specification_source("o-color4").is_none());
//!
//! let importance = feature_metadata("foundation.declaration.importance")
//!     .expect("atomic parser-facing record");
//! assert_eq!(importance.status(), CssSupportStatus::Complete);
//!
//! let generated = feature_metadata("official.selector.generated")
//!     .expect("atomic pseudo-element record");
//! assert_eq!(generated.kind(), CssFeatureKind::Selector);
//!
//! let processing = conformance_exclusion("excluded.O-IMAGES3.processing")
//!     .expect("official source exclusion");
//! assert_eq!(
//!     processing.reason(),
//!     CssExclusionReason::OutsideAuthoredSyntaxBoundary,
//! );
//! ```
//!
//! Every feature record is atomic and parser-facing, with one truthful support
//! status. Private reserved coverage slots describe later grammar boundaries only: they
//! are not feature records, carry no support status, and do not make their
//! spellings recognized. [`conformance_exclusions`] records informative,
//! superseded, and out-of-boundary official source items separately; exclusions
//! carry no support status and never change parser diagnostics. These metadata
//! and inventory boundaries do not change accepted CSS, retained syntax,
//! diagnostics, positions, spans, or recovery actions.

pub(crate) mod catalog;

#[doc(inline)]
pub use catalog::{
    CssConformanceExclusionId, CssConformanceSupersedingId, CssExclusionMetadata,
    CssExclusionReason, CssFeatureKind, CssFeatureMetadata, CssPropertySupportMetadata,
    CssSpecificationSource, CssSpecificationSourceId, CssSpecificationTier, CssSupportStatus,
    conformance_exclusion, conformance_exclusions, feature_catalog, feature_metadata,
    property_support_metadata, specification_source, specification_sources,
};
