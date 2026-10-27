//! Known properties, their grammars, metadata, expansion and values.
//!
//! A [`CssKnownProperty`] names one property this crate recognizes and has a [`CssPropertyGrammar`].
//! The module also holds property metadata (inheritance, initial value, shorthand membership),
//! the typed value of each property ([`CssPropertyValue`]) and the expansion of a declaration into
//! longhand contributions, which selects no cascade winner.
//!
//! # Declaration inspection and API evolution
//!
//! Parsing and `CssDeclaration::try_from_components` construct [`CssKnownDeclaration`] through the same
//! property grammar. Its fields are private, and [`CssKnownDeclaration::property`](crate::property::CssKnownDeclaration::property) derives
//! identity from the active coupled value, so callers cannot create a property/value mismatch.
//! [`CssKnownDeclaration::declared_value`](crate::property::CssKnownDeclaration::declared_value) returns exactly one of the
//! [`CssKnownDeclaredValueRef::Property`], [`CssKnownDeclaredValueRef::Global`],
//! or [`CssKnownDeclaredValueRef::SubstitutionDependent`] branches. The
//! [`CssKnownDeclaration::property_value`](crate::property::CssKnownDeclaration::property_value), [`CssKnownDeclaration::global`](crate::property::CssKnownDeclaration::global), and
//! [`CssKnownDeclaration::substitution_dependent`](crate::property::CssKnownDeclaration::substitution_dependent) convenience accessors are
//! mutually exclusive views of those same branches.
//!
//! The property branch borrows a non-exhaustive [`CssKnownPropertyValueRef`].
//! Match the variant that names the property and retain a wildcard for future
//! variants:
//!
//! ```
//! use surgeist_css::values::box_model::{CssBoxSize, CssSizeValue};
//! use surgeist_css::declaration::CssImportance;
//! use surgeist_css::property::{CssKnownDeclaredValueRef, CssKnownPropertyValueRef};
//! use surgeist_css::CssParser;
//!
//! let report = CssParser::default().parse_block_contents("width: calc(100% - 12px) !important");
//! let declaration = &report.syntax()[0];
//! assert_eq!(declaration.importance(), CssImportance::Important);
//! let known = declaration.known().expect("known declaration");
//!
//! match known.declared_value() {
//!     CssKnownDeclaredValueRef::Property(property) => match property {
//!         CssKnownPropertyValueRef::Width(width) => {
//!             assert_eq!(width.as_css(), "calc(100% - 12px)");
//!             let CssSizeValue::BoxSize(CssBoxSize::LengthPercentage(length)) = width.value() else {
//!                 panic!("expected a length-percentage");
//!             };
//!             assert!(length.calculation().is_some());
//!         }
//!         _ => panic!("expected width"),
//!     },
//!     CssKnownDeclaredValueRef::Global(_)
//!     | CssKnownDeclaredValueRef::SubstitutionDependent(_) => {
//!         panic!("expected an ordinary property value")
//!     }
//!     _ => panic!("future declared-value branch"),
//! }
//! ```
//!
//! Each ordinary property variant borrows one private-field [`CssPropertyValue`] holding the
//! authored declaration and its grammar-checked semantic value. The property identity belongs to
//! the [`CssKnownPropertyValueRef`] variant, so one generic type serves every property whose
//! grammar produces the same value. `as_css()` returns the exact authored
//! ordinary slice, excluding boundary trivia and terminal importance. Checked
//! construction uses token-preserving serialization; original or programmatic
//! provenance remains available through [`CssDeclaration::value_components`](crate::declaration::CssDeclaration::value_components).
//! Its one semantic accessor is `value()`.
//! The `all` property accepts only global or substitution-dependent values and
//! has no ordinary value.
//!
//! # Residual official properties and legacy orientation
//!
//! Complete typed authored grammars cover thirteen CSS2 residual
//! properties; Writing Modes 3 text combination, orientation, and bidi properties; UI3
//! outline-offset and UI4 caret-color and resize; Containment 2 `contain`; Transforms 1
//! `transform-box`; and Compositing 1 blend and isolation properties. These values retain
//! authored syntax and parser coordinates without applying cascade, layout, pagination,
//! painting, containment semantics, blending, hit testing, or writing-mode resolution.
//!
//! [`CssParserContext::with_svg_glyph_orientation_vertical`](crate::CssParserContext::with_svg_glyph_orientation_vertical) explicitly selects
//! the independent frozen SVG authored definition. Its Auto/Angle/implied-degree
//! Number body retains actual document mode and presentation-attribute admission
//! through originals, pending reentry and one independent terminal contribution.
//! Explicit finite grammar handles keep the Writing Modes meaning. Fixed
//! attribute-value fronts use Normal importance; root owns markup binding.
//! Specified Unitless output becomes degrees using shared numeric precision,
//! without computed quadrant rounding. See the crate's reference for source
//! reconciliation, canonical reparse policy and unresolved normative clauses.
//!
//! Ordinary `glyph-orientation-vertical` is an explicit restricted legacy shorthand that maps to a
//! parser-produced [`CssKnownProperty::TextOrientation`] value. It is not a name-equivalent
//! schema alias: [`CssKnownProperty::aliases`](crate::property::CssKnownProperty::aliases) remains empty for `TextOrientation`, while
//! [`feature_metadata`](crate::conformance::feature_metadata) exposes its distinct [`CssFeatureKind::PropertyAlias`](crate::conformance::CssFeatureKind::PropertyAlias) record. That
//! record is partial: the admitted literal mapping is supported, while numeric-terminal
//! spelling and math applicability remain unresolved by the selected standards. The shared
//! box-edge and blend-mode productions have independent complete records.
//!
//! ```
//! use surgeist_css::values::background::CssBlendMode;
//! use surgeist_css::conformance::{CssFeatureKind, CssSupportStatus, feature_metadata};
//! use surgeist_css::property::{CssKnownProperty, CssKnownPropertyValueRef};
//! use surgeist_css::CssParser;
//!
//! let report = CssParser::default().parse_block_contents(concat!(
//!     "border-spacing: 2px 3px; ",
//!     "glyph-orientation-vertical: 90; ",
//!     "background-blend-mode: multiply, luminosity",
//! ));
//! assert!(report.is_clean());
//! assert_eq!(
//!     report.syntax()[1].known().expect("legacy shorthand").property(),
//!     CssKnownProperty::TextOrientation,
//! );
//! let CssKnownPropertyValueRef::BackgroundBlendMode(blending) = report.syntax()[2]
//!     .known().expect("known blending property")
//!     .property_value().expect("ordinary value")
//! else { panic!("expected background blend modes") };
//! assert_eq!(
//!     blending.value().modes(),
//!     &[CssBlendMode::Multiply, CssBlendMode::Luminosity],
//! );
//! let alias = feature_metadata("official.property-alias.glyph-orientation-vertical")
//!     .expect("legacy alias metadata");
//! assert_eq!(alias.kind(), CssFeatureKind::PropertyAlias);
//! assert_eq!(alias.status(), CssSupportStatus::Partial);
//! ```

pub(crate) mod expansion;
pub(crate) mod properties;
pub(crate) mod property_name;
pub(crate) mod property_value;
pub(crate) mod svg_glyph;
pub(crate) mod syntax;

#[doc(inline)]
pub use expansion::{
    CssContributionValueRef, CssContributions, CssCustomPropertyContribution, CssExpansion,
    CssExpansionError, CssFourSideShorthandMetadata, CssInitialValueRef, CssLonghandContribution,
    CssLonghandContributions, CssLonghandInitialValue, CssLonghandMetadata, CssLonghandProperty,
    CssLonghandValue, CssLonghandValueRef, CssPendingSubstitution, CssPropertyKindRef,
    CssPropertyMetadata, CssPropertyMetadataError, CssShorthandMetadata,
    CssSvgGlyphOrientationVerticalContribution, CssUniversalReset, CssUniversalResetMetadata,
    CssUserAgentInitial,
};
#[doc(inline)]
pub use properties::{
    CssKnownDeclaration, CssKnownDeclaredValueRef, CssKnownProperty, CssKnownPropertyValueRef,
    CssPropertyGrammar, CssPropertyValue,
};
#[doc(inline)]
pub use property_name::CssPropertyName;
#[doc(inline)]
pub use property_value::{CssPropertyTarget, CssPropertyTextTarget, CssPropertyValueError};
#[doc(inline)]
pub use svg_glyph::{
    CssSvgGlyphOrientationVerticalDeclaration, CssSvgGlyphOrientationVerticalMetadata,
    CssSvgGlyphOrientationVerticalValue,
};
#[doc(inline)]
pub use syntax::{
    CssAllDeclaredValue, CssCustomPropertyMetadata, CssGuaranteedInvalidInitial, CssPropertyNameRef,
};
