//! Declarations, declaration lists, custom properties, substitution and importance.
//!
//! # Style attributes and declarations
//!
//! Style attributes share the ordinary declaration grammar used by style-rule
//! blocks. Declarations retain authored order, semantic source positions,
//! property/value coupling, custom-property text, substitution-dependent text,
//! and terminal [`CssImportance`].
//!
//! ```
//! use surgeist_css::declaration::CssImportance;
//! use surgeist_css::property::{CssKnownProperty, CssPropertyNameRef};
//! use surgeist_css::CssParser;
//!
//! let report = CssParser::default().parse_block_contents("--Theme: RGB(1, 2, var(--fallback)); mystery: 1; width: var(--size, 2px) !important");
//! assert_eq!(report.syntax().len(), 2);
//! assert_eq!(report.diagnostics().len(), 1);
//! assert_eq!(
//!     report.syntax()[0]
//!         .custom()
//!         .expect("custom declaration")
//!         .value()
//!         .value()
//!         .expect("authored custom value")
//!         .as_css(),
//!     "RGB(1, 2, var(--fallback))",
//! );
//! let width = &report.syntax()[1];
//! assert_eq!(width.importance(), CssImportance::Important);
//! assert!(matches!(width.property_name(), CssPropertyNameRef::Known(_)));
//! let value = width.known().expect("coupled width declaration");
//! assert_eq!(value.property(), CssKnownProperty::Width);
//! assert_eq!(
//!     value
//!         .substitution_dependent()
//!         .expect("symbolic authored value")
//!         .as_css(),
//!     "var(--size, 2px)",
//! );
//! ```

pub(crate) mod declaration_serialization;
pub(crate) mod specified_declaration_block;
pub(crate) mod syntax;

#[doc(inline)]
pub use specified_declaration_block::{
    CssDeclarationBlockError, CssInvalidKeyframeSourceReason, CssSpecifiedDeclarationBlock,
    CssSpecifiedDeclarationEntry, CssSpecifiedDeclarationValueRef,
};
#[doc(inline)]
pub use syntax::{
    CssAuthoredDeclarationValue, CssAuthoredFunctionArguments, CssCustomDeclaration,
    CssCustomPropertyDeclaredValue, CssCustomPropertyName, CssCustomPropertyValue, CssDeclaration,
    CssDeclarationBody, CssDeclarationList, CssGlobalKeyword, CssImportance,
    CssSubstitutionDependentValue,
};
