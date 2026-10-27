//! `@page` and margin rules and page projection.
//!
//! # Page rules
//!
//! [`CssRule::Page`](crate::rule::CssRule::Page) retains an empty prelude or a complete named and compound
//! [`CssPageSelectorList`], including comma lists and logical `:recto`/`:verso`.
//! Selectors expose the Page-specific three-component specificity. Page declarations
//! preserve a distinct typed descriptor domain (`size`, `page-orientation`, `marks`,
//! `bleed`) alongside applicable shared properties and custom values. Sixteen canonical
//! margin-box rule names retain ordered children. Shared property grammars and symbolic
//! values are reused, including font-relative lengths; no layout or variable lookup runs.
//! `CssDeclaration::try_from_components`, `CssPageDescriptorValue::parse`, and `CssBlockFragment::parse`
//! check detached inputs with genuine origins. Checked selected blocks and borrowed
//! [`CssPageRuleView`] support immutable edits without re-expanding authored shorthands.
//! Page and margin CSSOM output follows the adopted Blink whole-rule wrapper under one
//! cumulative atomic budget. Page matching, pagination, cascade, counter registration,
//! inheritance resolution, and generated-marker execution remain downstream.
//!
//! ```
//! use surgeist_css::rule::counter_style::CssCounterStyleSystem;
//! use surgeist_css::rule::page::CssPagePseudo;
//! use surgeist_css::rule::CssRule;
//! use surgeist_css::CssParser;
//!
//! let report = CssParser::default().parse_sheet(concat!(
//!     "@counter-style digits { system: numeric; symbols: \"0\" \"1\"; suffix: \".\"; } ",
//!     "@page :left { margin-left: -12mm; margin-right: 10%; }",
//! ));
//! assert!(report.is_clean());
//! let [CssRule::CounterStyle(counter), CssRule::Page(page)] = report.syntax().rules() else {
//!     panic!("expected counter-style and page rules");
//! };
//! assert_eq!(counter.name().as_str(), "digits");
//! assert!(matches!(
//!     counter.descriptors().system().map(|value| value.value().view()),
//!     Some(surgeist_css::rule::counter_style::CssCounterStyleDescriptorValueRef::System(&CssCounterStyleSystem::Numeric))
//! ));
//! assert_eq!(counter.descriptors().occurrences().count(), 3);
//! assert_eq!(page.selectors().selectors()[0].pseudos(), &[CssPagePseudo::Left]);
//! assert_eq!(page.declarations().properties().len(), 2);
//! ```

pub(crate) mod page_projection;
pub(crate) mod page_serialization;
pub(crate) mod syntax;

#[doc(inline)]
pub use page_projection::{
    CssMarginRuleView, CssPageProjectionError, CssPageRuleView, CssSpecifiedPageDeclarationBlock,
    CssSpecifiedPageDeclarationEntry,
};
#[doc(inline)]
pub use syntax::{
    CssMarginBox, CssMarginDeclarationBlock, CssMarginRule, CssPageBlockError, CssPageBody,
    CssPageDeclaration, CssPageDeclarationBlock, CssPageDescriptor, CssPageDescriptorKind,
    CssPageDescriptorValue, CssPageDescriptorValueRef, CssPageMarks, CssPageNamedSize,
    CssPageOrientation, CssPageOutputOrientation, CssPagePseudo, CssPageRule, CssPageSelector,
    CssPageSelectorList, CssPageSize, CssPageSizeValue, CssPageSpecificity,
};
