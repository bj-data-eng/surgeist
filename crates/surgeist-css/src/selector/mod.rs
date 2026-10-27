//! Selector syntax, namespace context and style, relative and scope selector lists.
//!
//! # Namespaces and complete Selectors 3 syntax
//!
//! [`CssRule::Namespace`](crate::rule::CssRule::Namespace) retains an optional decoded, case-sensitive
//! [`CssNamespacePrefix`](crate::rule::CssNamespacePrefix), a literal [`CssNamespaceName`](crate::rule::CssNamespaceName), and its parser-produced position.
//! Empty and non-URI names remain valid authored values; this crate does not normalize, resolve,
//! or load them. Selector type, universal, and attribute names expose
//! [`CssNamespaceConstraint`] and [`CssQualifiedSelectorName`]. `Named` requires an earlier active
//! prefix, `ExplicitNone` represents `|`, `Any` represents `*|`, and `Default` represents an
//! unqualified type or universal selector while a default declaration is active. Unqualified
//! attributes are always `ExplicitNone`.
//!
//! ```
//! use surgeist_css::selector::{CssNamespaceConstraint, CssPseudoElement, CssPseudoElementSegment, CssSelector};
//! use surgeist_css::rule::CssRule;
//! use surgeist_css::CssParser;
//!
//! let report = CssParser::default().parse_sheet(concat!(
//!     "@namespace svg \"urn:svg\";",
//!     "svg|a#first#second[|lang]::first-line { color: red; }",
//! ));
//! assert!(report.is_clean());
//! let [CssRule::Namespace(namespace), CssRule::Style(style)] = report.syntax().rules() else {
//!     panic!("expected namespace and style rules");
//! };
//! assert_eq!(namespace.prefix().expect("named prefix").as_str(), "svg");
//! assert_eq!(namespace.name().as_str(), "urn:svg");
//!
//! let CssSelector::Compound(selector) = style.selectors().selectors()[0].selector() else {
//!     panic!("expected compound selector");
//! };
//! let qualified = selector.type_selector().expect("qualified type selector");
//! assert!(matches!(
//!     qualified.namespace(),
//!     CssNamespaceConstraint::Named(prefix) if prefix.as_str() == "svg"
//! ));
//! assert_eq!(qualified.local_name(), Some("a"));
//! assert_eq!(selector.ids(), ["first", "second"]);
//! let [attribute] = selector.attributes() else {
//!     panic!("expected one attribute selector");
//! };
//! assert_eq!(attribute.namespace(), &CssNamespaceConstraint::ExplicitNone);
//! assert!(matches!(
//!     selector
//!         .pseudo_elements()
//!         .expect("pseudo-element sequence")
//!         .segments(),
//!     [CssPseudoElementSegment::PseudoElement(CssPseudoElement::FirstLine)]
//! ));
//! ```
//!
//! # Selector syntax
//!
//! Complete Selectors 3 syntax includes all attribute matchers and four combinators, ordered
//! repeated IDs and classes, the structural/UI/dynamic pseudo-class families, `:lang()`, and
//! first-line/first-letter pseudo-elements. Legacy single-colon `before`, `after`, `first-line`,
//! and `first-letter` map to the same typed pseudo-elements. Undeclared namespace prefixes follow
//! consumer recovery contract: `:is()` and `:where()` retain an invalid member containing a
//! delimiter `&` as explicit match-nothing syntax and otherwise drop only that member,
//! while unforgiving style, scope, nesting, `:not()`, `:has()`, and nth `of` consumers drop their
//! established containing unit. Matching, specificity, cascade, namespace URI resolution,
//! CSSOM serialization, and cross-crate lowering remain downstream.
//! `:autofill` and required `:-webkit-autofill` share [`CssPseudoClass::Autofill`]
//! and emit the standard spelling. Nonfunctional unknown `::-webkit-*` names
//! retain checked [`CssUnknownWebkitPseudoElement`] identities with ASCII-lowercase
//! semantic names and escaped specified output. They remain symbolic match-nothing
//! pseudos subject to ordinary attachment, suffix and receiving-context restrictions;
//! unknown functional forms remain invalid. User-action suffixes on non-tree-abiding
//! pseudos retain the selected draft's at-risk status. No currently admitted pseudo
//! has internal combinator permission.
//! Pseudo 4 adds [`CssPseudoElement::Prefix`], [`CssPseudoElement::Suffix`],
//! [`CssPseudoElement::SearchText`], [`CssPseudoElement::TargetText`],
//! [`CssPseudoElement::SpellingError`], [`CssPseudoElement::GrammarError`],
//! [`CssPseudoElement::Highlight`], [`CssPseudoElement::Placeholder`],
//! [`CssPseudoElement::FileSelectorButton`] and [`CssPseudoElement::DetailsContent`].
//! Highlight retains one checked, case-preserving [`CssCustomIdent`](crate::values::common::CssCustomIdent). FirstLetter
//! permits Prefix and Suffix children; recognized initial Prefix/Suffix names
//! retain their implied universal origin. Nonfunctional [`CssPseudoClass::Current`]
//! is admitted only in SearchText and element-backed suffix contexts, including
//! inherited logical arguments. Ordinary `:current`, `:current()`, `:past` and
//! `:future` remain unsupported. Part, FileSelectorButton and DetailsContent admit
//! otherwise supported pseudo syntax, with each following segment resetting the
//! receiving permissions. This syntax does not evaluate matching prohibitions,
//! live highlights or optional UA implementation of SearchText.
//! View Transitions 1 retains [`CssViewTransitionName`](crate::values::view_transition::CssViewTransitionName) as `none` or a checked,
//! case-sensitive [`CssViewTransitionIdent`](crate::values::view_transition::CssViewTransitionIdent), excluding `none` and `auto` from
//! the custom-name branch. [`CssPseudoElement::ViewTransition`] and its Group,
//! ImagePair, Old and New functional variants retain wildcard or custom-ident
//! [`CssViewTransitionNameSelector`](crate::values::view_transition::CssViewTransitionNameSelector) arguments. Named descendants add OnlyChild
//! to generic suffix permission, inherited by Not/Is/Where. Exact compounds follow
//! root → group → image-pair → old/new, and Slotted admits all five tree-abiding
//! identities. Participant discovery, matching and live transitions belong downstream.

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
