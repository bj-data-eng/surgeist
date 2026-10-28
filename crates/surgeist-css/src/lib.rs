#![forbid(unsafe_code)]
#![deny(unfulfilled_lint_expectations)]
#![warn(missing_debug_implementations)]
#![warn(missing_docs)]
#![recursion_limit = "512"]
//! Browser-recovering CSS ingestion for Surgeist.
//!
//! `CssParser::parse_sheet` and `CssParser::parse_block_contents` parse UTF-8 input into CSS-owned
//! authored syntax plus every structured recovery diagnostic in source order.
//! Retained nodes are valid by construction. Unsupported or malformed source
//! units are recovered at their grammar boundary so later valid siblings remain
//! eligible; invalid authored nodes are never retained.
//!
//! A [`CssParseReport::is_clean`] result means exactly that the diagnostic slice
//! is empty. It is not a separate syntax-validity predicate, and callers must not
//! infer cleanliness from an empty retained sheet or declaration list.
//!
//! # Where things live
//!
//! The crate root holds the five items every task needs: [`CssParser`], the configured front
//! door; [`CssParserContext`] and [`CssParserMode`], the grammar context a declaration retains;
//! [`CssParseReport`], retained syntax plus ordered recovery diagnostics; and
//! [`CssValidationFailure`], the diagnostics of a rejected clean-report check. Every other
//! public item has exactly one path, in the module that owns its domain, and a root path to a
//! module item does not resolve.
//!
//! | Module | Domain |
//! | --- | --- |
//! | [`source`] | Source snapshots, coordinates and origins; byte decoding; UTF-16 preparation and limits; retention traversal. |
//! | [`diagnostic`] | Recovery diagnostics, errors with stable codes, and their fact views. |
//! | [`component`] | Component values, token views and their limits and errors. |
//! | [`numeric`] | Numbers, dimensions, calculations, units and range markers. |
//! | [`color`] | Authored color syntax, facts, and ordinary and controlled resolution. |
//! | [`selector`] | Selector syntax, namespace context and selector lists. |
//! | [`query`] | The generic condition model, `@when`, and the [`media`](query::media), [`supports`](query::supports) and [`container`](query::container) query grammars. |
//! | [`rule`] | Style sheets, rules and block fragments; at-rule modules for keyframes, `@page`, `@font-face`, `@counter-style`, `@font-feature-values`, `@font-palette-values` and `@color-profile`. |
//! | [`declaration`] | Declarations, custom properties, importance and the CSSOM declaration block. |
//! | [`property`] | Known properties, grammars, metadata and expansion. |
//! | [`values`] | Authored property values, one module per specification family. |
//! | [`serialization`] | Serialization limits and errors, rule-graph input and lexical serializers. |
//! | [`normalize`] | Normalized style sheets. |
//! | [`conformance`] | The specification source registry and support catalogue. |
//!
//! # Tasks
//!
//! Each task below is a compiled example against the current API.
//!
//! ## 1. Parse a style sheet
//!
//! [`CssParser::parse_sheet`] returns the retained sheet and one diagnostic per recovered unit.
//! Byte input is decoded first, and UTF-16 input is prepared under explicit limits.
//!
//! ```
//! use std::sync::Arc;
//! use surgeist_css::diagnostic::CssRecoveryAction;
//! use surgeist_css::source::{CssDecodedStylesheetInput, CssUtf16InputLimits};
//! use surgeist_css::CssParser;
//! use surgeist_css_parser::input::{DecodeHints, DecodeLimits};
//!
//! let parser = CssParser::default();
//! let report = parser.parse_sheet(".a { color: red } @unknown x; .b {}");
//! assert_eq!(report.syntax().rules().len(), 2);
//! assert_eq!(report.diagnostics()[0].action(), CssRecoveryAction::DropAtRule);
//!
//! // Bytes: select the encoding, decode, then parse the decoded input.
//! let hints = DecodeHints { environment_encoding: None, protocol_label: None };
//! let input = CssDecodedStylesheetInput::decode(
//!     b".a { color: red }",
//!     hints,
//!     DecodeLimits::default(),
//! )
//! .expect("decodable input");
//! assert!(parser.parse_sheet_bytes(&input).is_clean());
//!
//! // UTF-16: prepare under limits; `poll` may interrupt the preparation.
//! let units: Vec<u16> = ".a { color: red }".encode_utf16().collect();
//! let report = parser
//!     .parse_sheet_utf16(
//!         Arc::new(units),
//!         CssUtf16InputLimits::default(),
//!         || Ok::<(), std::convert::Infallible>(()),
//!     )
//!     .expect("prepared input");
//! assert!(report.is_clean());
//! ```
//!
//! ## 2. Parse declarations
//!
//! A style attribute or a `cssText` value is a declaration list:
//!
//! ```
//! use surgeist_css::declaration::CssImportance;
//! use surgeist_css::CssParser;
//!
//! let report = CssParser::default().parse_block_contents("color: red; width: 2px !important");
//! let width = &report.syntax()[1];
//! assert_eq!(width.importance(), CssImportance::Important);
//! ```
//!
//! ## 3. Parse and validate a property value
//!
//! [`CssParser::parse_property_value`] checks one value against one property's grammar.
//! [`CssParseReport::into_validation_result`] accepts exactly a clean report. Programmatic
//! callers build the same checked declaration from component values:
//!
//! ```
//! use surgeist_css::component::CssComponentValues;
//! use surgeist_css::declaration::{CssDeclaration, CssImportance};
//! use surgeist_css::property::{
//!     CssKnownProperty, CssPropertyNameRef, CssPropertyTarget, CssPropertyTextTarget,
//! };
//! use surgeist_css::CssParser;
//!
//! let parser = CssParser::default();
//! let name = CssPropertyNameRef::Known(CssKnownProperty::Width);
//! let report = parser.parse_property_value(
//!     "calc(100% - 12px)",
//!     CssPropertyTextTarget::Property(name),
//!     CssImportance::Normal,
//! );
//! let declaration = report
//!     .into_validation_result()
//!     .expect("a clean report")
//!     .expect("a declaration");
//!
//! // Checked components: substitution reentry and programmatic construction.
//! let values = CssComponentValues::parse(&parser, "calc(100% - 12px)").expect("component syntax");
//! let built = CssDeclaration::try_from_components(
//!     &parser,
//!     CssPropertyTarget::Property(name),
//!     values,
//!     CssImportance::Normal,
//! )
//! .expect("a value of the width grammar");
//! assert_eq!(built.known().map(|known| known.property()), Some(CssKnownProperty::Width));
//! # drop(declaration);
//! ```
//!
//! ## 4. Inspect a known declaration
//!
//! ```
//! use surgeist_css::declaration::CssImportance;
//! use surgeist_css::property::{CssKnownDeclaredValueRef, CssKnownPropertyValueRef};
//! use surgeist_css::values::box_model::CssSizeValue;
//! use surgeist_css::CssParser;
//!
//! let report = CssParser::default().parse_block_contents("width: calc(100% - 12px)");
//! let declaration = &report.syntax()[0];
//! assert_eq!(declaration.importance(), CssImportance::Normal);
//! match declaration.known().expect("known property").declared_value() {
//!     CssKnownDeclaredValueRef::Property(CssKnownPropertyValueRef::Width(width)) => {
//!         assert_eq!(width.as_css(), "calc(100% - 12px)");
//!         let _: &CssSizeValue = width.value();
//!     }
//!     _ => panic!("expected the width value"),
//! }
//! ```
//!
//! ## 5. Serialize, expand and normalize
//!
//! ```
//! use surgeist_css::property::{CssContributions, CssExpansion};
//! use surgeist_css::serialization::CssSerializationLimits;
//! use surgeist_css::CssParser;
//!
//! let parser = CssParser::default();
//! let report = parser.parse_block_contents("margin-block: 1px 2px");
//! let declaration = &report.syntax()[0];
//!
//! let text = declaration.serialize_specified(CssSerializationLimits::default()).unwrap();
//! assert_eq!(text, "margin-block: 1px 2px;");
//!
//! // Longhand contributions of the shorthand, without cascading them.
//! let CssExpansion::Contributions(CssContributions::Longhands(longhands)) =
//!     declaration.expand().unwrap()
//! else {
//!     panic!("a shorthand expands to longhands");
//! };
//! assert_eq!(longhands.items().len(), 2);
//!
//! let sheet = parser.parse_sheet(".a { margin-block: 1px 2px }");
//! let normalized = sheet.normalize(Default::default()).expect("supported expansion");
//! assert_eq!(normalized.syntax().items().len(), 2);
//! ```
//!
//! ## 6. Resolve colors
//!
//! [`color::parse_used_color`] resolves color text to a used color over explicit facts, and
//! [`color::CssColor::resolve`] resolves a checked authored color the same way. A caller that
//! services each fact request itself drives [`color::controlled::CssColorParse`] instead.
//!
//! ```
//! use surgeist_css::color::{
//!     CssAuthoredColorOrdinaryLimits, CssColorFactIdentity, CssColorFacts,
//!     CssColorGrammarRequest, CssColorInput, CssColorOrdinaryLimits, CssColorOrdinaryOutcome,
//!     CssColorPropertyFacts, CssColorSchemeFact, CssDeviceColorFact, parse_used_color,
//! };
//! use surgeist_css::property::CssKnownPropertyValueRef;
//! use surgeist_css::CssParser;
//!
//! let facts = CssColorFacts::new(
//!     CssColorFactIdentity::new(1, 1),
//!     CssColorPropertyFacts::InitialProperties,
//!     CssColorSchemeFact::Light,
//!     &[],
//!     &[],
//!     &[],
//!     CssDeviceColorFact::Unknown,
//! );
//! // The default ceilings cover inputs of up to 1,024 units and create no shared
//! // controller; pass explicit limits for longer inputs.
//! let limits = CssColorOrdinaryLimits::default();
//! let result = parse_used_color(
//!     CssColorInput::Utf8(b"red"),
//!     CssColorGrammarRequest::WholeColor,
//!     &facts,
//!     facts.identity(),
//!     limits,
//! )
//! .expect("within limits");
//! assert!(matches!(result.outcome(), CssColorOrdinaryOutcome::Used(_)));
//!
//! let report = CssParser::default().parse_block_contents("color: red");
//! let CssKnownPropertyValueRef::Color(color) =
//!     report.syntax()[0].known().unwrap().property_value().unwrap()
//! else {
//!     panic!("color");
//! };
//! let authored_limits = CssAuthoredColorOrdinaryLimits::default();
//! let authored = color.value().resolve(&facts, facts.identity(), authored_limits);
//! assert!(authored.is_ok());
//! ```
//!
//! ## 7. Inspect diagnostics and sources
//!
//! ```
//! use surgeist_css::CssParser;
//!
//! let report = CssParser::default().parse_sheet("@unknown x; .a { width: ; }");
//! for diagnostic in report.diagnostics() {
//!     let error = diagnostic.error();
//!     let (_code, _position) = (error.code(), error.position());
//!     let _expected = error.expectation(); // Option<CssGrammarExpectation>
//!     let _refused = error.is_resource_refusal(); // parser or component resource refusal
//!     let _span = diagnostic.span();
//! }
//! assert!(!report.is_clean());
//! ```
//!
//! ## 8. Query support metadata
//!
//! ```
//! use surgeist_css::conformance::{feature_metadata, property_support_metadata};
//! use surgeist_css::property::CssKnownProperty;
//!
//! assert!(feature_metadata("official.property.flex-flow").is_some());
//! assert!(property_support_metadata("width").is_some());
//! assert!(CssKnownProperty::Width.metadata().is_ok());
//! ```
//!
//! # Validating clean input
//!
//! [`CssParseReport::into_validation_result`] consumes a report and accepts exactly a clean
//! one; otherwise it preserves the complete non-empty diagnostic sequence in
//! [`CssValidationFailure`]. The step does not select a second grammar or change ordinary parsing.
//!
//! # API evolution
//!
//! Value enums and fact enums are `#[non_exhaustive]`, so a downstream match
//! needs a wildcard arm, including for the three-state
//! [`CssSupportStatus`](crate::conformance::CssSupportStatus). Error kinds are
//! private behind predicates and accessors. Three kinds of enum are exhaustive
//! on purpose:
//!
//! - The protocol enums of the controlled color parse in [`color::controlled`]:
//!   the step, action and outcome enums and the bank, fact-read and kernel
//!   enums an action names. Matching every variant is the compile-time proof
//!   that a consumer services every action.
//! - The admission-callback vocabularies
//!   [`CssSourceRetentionAction`](crate::source::CssSourceRetentionAction) and
//!   [`CssUtf16ValueAdmission`](crate::source::CssUtf16ValueAdmission): a
//!   callback that missed a charge kind would let work escape its bounded
//!   admission, so a new kind must fail to compile in every callback.
//! - [`CssImportance`](crate::declaration::CssImportance), because CSS Cascade
//!   defines exactly two importance states and cascade consumers match both to
//!   order precedence.
//!
//! This inspection model does not change parsing, recovery, or diagnostics.
//!
//! This match on the spec-closed importance states compiles without a wildcard:
//!
//! ```
//! use surgeist_css::declaration::CssImportance;
//!
//! fn describe(value: CssImportance) -> &'static str {
//!     match value {
//!         CssImportance::Normal => "normal",
//!         CssImportance::Important => "important",
//!     }
//! }
//! assert_eq!(describe(CssImportance::Important), "important");
//! ```
//!
//! Evolving enums intentionally require a wildcard in external matches. These
//! representative exhaustive matches therefore do not compile:
//!
//! ```compile_fail,E0004
//! use surgeist_css::conformance::CssSupportStatus;
//!
//! fn describe(value: CssSupportStatus) -> &'static str {
//!     match value {
//!         CssSupportStatus::Complete => "complete",
//!         CssSupportStatus::Partial => "partial",
//!         CssSupportStatus::RecognizedUnsupported => "recognized unsupported",
//!     }
//! }
//! ```
//!
//! ```compile_fail
//! use surgeist_css::query::media::CssMediaQueryModifier;
//!
//! fn describe(value: CssMediaQueryModifier) -> &'static str {
//!     match value {
//!         CssMediaQueryModifier::Not => "not",
//!         CssMediaQueryModifier::Only => "only",
//!     }
//! }
//! ```
//!
//! ```compile_fail
//! use surgeist_css::selector::CssSelectorCombinator;
//!
//! fn describe(value: CssSelectorCombinator) -> &'static str {
//!     match value {
//!         CssSelectorCombinator::Descendant => "descendant",
//!         CssSelectorCombinator::Child => "child",
//!         CssSelectorCombinator::NextSibling => "next",
//!         CssSelectorCombinator::SubsequentSibling => "subsequent",
//!     }
//! }
//! ```
//!
//!
//! ```compile_fail
//! use surgeist_css::values::animation::CssAnimationDirection;
//!
//! fn describe(value: CssAnimationDirection) -> &'static str {
//!     match value {
//!         CssAnimationDirection::Normal => "normal",
//!         CssAnimationDirection::Reverse => "reverse",
//!         CssAnimationDirection::Alternate => "alternate",
//!         CssAnimationDirection::AlternateReverse => "alternate-reverse",
//!     }
//! }
//! ```
//!
//! ```compile_fail
//! use surgeist_css::values::layout::CssGridAutoFlowAxis;
//!
//! fn describe(value: CssGridAutoFlowAxis) -> &'static str {
//!     match value {
//!         CssGridAutoFlowAxis::Row => "row",
//!         CssGridAutoFlowAxis::Column => "column",
//!     }
//! }
//! ```
//!
//! ```compile_fail,E0004
//! use surgeist_color::PredefinedColorSpace;
//! use surgeist_css::color::CssRelativeColorFunction;
//!
//! fn describe(value: CssRelativeColorFunction) -> &'static str {
//!     match value {
//!         CssRelativeColorFunction::Rgb => "rgb",
//!         CssRelativeColorFunction::Hsl => "hsl",
//!         CssRelativeColorFunction::Hwb => "hwb",
//!         CssRelativeColorFunction::Lab => "lab",
//!         CssRelativeColorFunction::Lch => "lch",
//!         CssRelativeColorFunction::Oklab => "oklab",
//!         CssRelativeColorFunction::Oklch => "oklch",
//!         CssRelativeColorFunction::Color(PredefinedColorSpace::Srgb) => "srgb",
//!         CssRelativeColorFunction::Color(_) => "other color space",
//!     }
//! }
//! ```
//!
//! # Boundary
//!
//! This crate owns authored CSS syntax, intrinsic grammar validation, recovery
//! boundaries, diagnostic provenance, support metadata and bounded source color
//! resolution over explicit supplied facts. Numerical color values, conversion,
//! interpolation and resolved serialization belong to [`surgeist_color`].
//! CSS does not apply cascade or inheritance; substitute custom properties;
//! validate computed post-substitution values; evaluate queries; match selectors;
//! resolve URLs, resources or general style units; perform layout, painting or
//! animation; or expose a mutable CSSOM.
//!
//! Public items live in their domain modules; the crate root re-exports only the
//! parser and its report types, so a root path to a module item does not resolve
//! (CF24):
//!
//! ```compile_fail,E0432
//! use surgeist_css::CssSheet;
//! ```

pub mod color;
pub mod component;
pub mod conformance;
pub mod declaration;
pub mod diagnostic;
pub mod normalize;
pub mod numeric;
pub mod property;
pub mod query;
pub mod rule;
pub mod selector;
pub mod serialization;
pub mod source;
pub mod values;

mod parser;
mod parser_context;
mod report;
mod specified_inverse;
mod syntax_consumption;
#[cfg(test)]
mod test_support;
#[cfg(test)]
mod tests;
mod tokenization;
mod validation;

#[doc(inline)]
pub use parser_context::{CssParser, CssParserContext, CssParserMode};
#[doc(inline)]
pub use report::{CssParseReport, CssValidationFailure};

/// Crate-issued seals for the public numeric traits.
///
/// The traits are nominally public, so markers defined in any module can
/// implement them, and this module is crate-private, so no other crate can
/// name or implement them.
pub(crate) mod sealed {
    /// The crate-issued identity of one numeric root marker.
    #[derive(Clone, Copy, Debug)]
    pub struct Root(pub(crate) crate::numeric::value::RootKind);

    /// Seals `CssNumericRoot`; only CSS implements it.
    pub trait NumericRoot {
        /// The root this marker names.
        const ROOT: Root;
    }

    /// Parses a real-brace block as one grammar; only CSS implements it.
    pub trait BlockBody: Sized {
        /// Parses `source` as this body's block fragment.
        #[track_caller]
        fn parse_block(
            parser: &crate::CssParser,
            source: &str,
            context: &crate::rule::CssBlockContext<'_>,
        ) -> crate::CssParseReport<Option<crate::rule::CssBlockFragment<Self>>>;
    }

    /// The range check behind `CssRange<T>`; only CSS implements it.
    pub trait RangeBounds<T: crate::numeric::CssNumericRoot> {
        /// Whether an exact literal of root `T` lies in this range, compared
        /// exactly, without binary64 conversion.
        fn admits(literal: &crate::numeric::CssNumericLiteral<T>) -> bool;
    }

    /// The unit vocabulary of one dimension root; only CSS implements it.
    pub trait DimensionUnit: Copy {
        /// Selects this vocabulary's unit from a decoded registry unit.
        fn from_numeric(unit: crate::numeric::CssNumericUnit) -> Option<Self>;
        /// Returns this unit's registry identity.
        fn into_numeric(self) -> crate::numeric::CssNumericUnit;
    }
}

/// Shared ceiling for authored rules and owned component-value trees.
const STRUCTURAL_NESTING_LIMIT: u32 = 256;
