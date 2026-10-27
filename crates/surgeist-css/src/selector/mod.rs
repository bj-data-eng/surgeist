//! Selector syntax, namespace context and style, relative and scope selector lists.

pub(crate) mod namespace;
pub(crate) mod selector_anchors;
pub(crate) mod selector_serialization;
pub(crate) mod selector_syntax;
pub(crate) mod style_selectors;
pub(crate) mod syntax;

#[doc(inline)]
pub use namespace::CssNamespaceContext;
#[doc(inline)]
pub use selector_serialization::CssScopeSelectorCssomContext;
#[doc(inline)]
pub use selector_syntax::{
    CssAttributeCaseSensitivity, CssAttributeMatcher, CssAttributeName, CssAttributeSelector,
    CssComplexSelector, CssComplexSelectorPart, CssCompoundSelector, CssCompoundSelectorArgument,
    CssEmptyLanguageRangeList, CssInvalidNestingSelectorItem, CssLanguageRange,
    CssLanguageRangeKind, CssLanguageRangeList, CssNamespaceConstraint, CssNthAnPlusB,
    CssNthChildPattern, CssNthPattern, CssPartName, CssPartNameList, CssPseudoClass,
    CssPseudoElement, CssPseudoElementSegment, CssPseudoElementSequence, CssPseudoSelectorList,
    CssPseudoSelectorListItem, CssQualifiedAttributeName, CssQualifiedNamePrefix,
    CssQualifiedSelectorName, CssRelativeSelector, CssRelativeSelectorList, CssSelector,
    CssSelectorCombinator, CssSelectorConstructionError, CssSelectorList, CssSimpleSelector,
    CssUnknownWebkitPseudoElement,
};
#[doc(inline)]
pub use style_selectors::{CssParsedStyleSelectors, CssStyleSelectorContext};
#[doc(inline)]
pub use syntax::{CssScopeSelectorList, CssStyleSelector, CssStyleSelectorList};
