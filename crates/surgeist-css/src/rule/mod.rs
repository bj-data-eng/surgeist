//! Style sheets, rules and their admission, construction and block fragments.
//!
//! A [`CssSheet`] is the ordered list of [`CssRule`] values a parse retains: style rules with
//! their nested rules and declarations, conditional and layer groups, `@import`, `@namespace`,
//! `@scope`, `@when` and `@else`, and the at-rules that have submodules of their own. Rules keep
//! authored order and parser positions; nothing is cascaded or evaluated.
//!
//! # Stylesheets and recovery
//!
//! ```
//! use surgeist_css::diagnostic::{CssErrorCode, CssRecoveryAction};
//! use surgeist_css::rule::CssRule;
//! use surgeist_css::CssParser;
//!
//! let report = CssParser::default().parse_sheet(".before { color: red; } @unknown value; .after { color: blue; }");
//! assert_eq!(report.syntax().rules().len(), 2);
//! assert!(matches!(report.syntax().rules()[0], CssRule::Style(_)));
//! let diagnostic = &report.diagnostics()[0];
//! assert_eq!(diagnostic.error().code(), CssErrorCode::UnknownAtRule);
//! assert_eq!(diagnostic.action(), CssRecoveryAction::DropAtRule);
//! ```
//!
//! # Authored nesting
//!
//! A style rule owns its complete selector list, leading declarations, and ordered child
//! rules. Later declaration runs use [`CssRule::NestedDeclarations`], preserving their place
//! after a nested rule without selecting cascade winners or expanding parent selectors.
//!
//! ```
//! use surgeist_css::rule::CssRule;
//! use surgeist_css::CssParser;
//!
//! let report = CssParser::default().parse_sheet(".card, #featured { color: red; &:hover { opacity: 1; } color: blue; }");
//! assert!(report.is_clean());
//! let [CssRule::Style(parent)] = report.syntax().rules() else { unreachable!() };
//! assert_eq!(parent.selectors().selectors().len(), 2);
//! assert_eq!(parent.declarations().len(), 1);
//! let [CssRule::Style(child), CssRule::NestedDeclarations(after)] = parent.rules() else {
//!     unreachable!()
//! };
//! assert_eq!(child.declarations().len(), 1);
//! assert_eq!(after.declarations().len(), 1);
//! ```
//!
//! # Prelude order, layers and imports
//!
//! Initial layer statements may precede imports and namespaces, as specified by
//! [Cascade 5](https://www.w3.org/TR/2022/CR-css-cascade-5-20220113/#layer-empty).
//! Imports precede namespaces. A layer statement after either import or namespace
//! declarations, or a body rule, closes both prelude sequences. Invalid or ignored
//! rules do not change the phase or active bindings. Malformed, block-form, nested,
//! or late namespaces recover as one [`CssRecoveryAction::DropAtRule`](crate::diagnostic::CssRecoveryAction::DropAtRule).
//!
//! An import prelude is parsed in target, optional `layer`, optional `supports()`, optional media
//! order. A successful initial layer statement may precede imports; a later body rule closes the
//! import phase. Invalid order or clauses drop only the import and leave later siblings eligible.
//! Import targets and conditions remain symbolic: URL resolution, resource loading, condition
//! evaluation, cascade, selector matching, and root/sibling lowering are downstream work.
//!
//! ```
//! use surgeist_css::query::CssConditionKind;
//! use surgeist_css::rule::{CssImportLayer, CssRule};
//! use surgeist_css::query::supports::CssSupportsLeaf;
//! use surgeist_css::CssParser;
//!
//! let report = CssParser::default().parse_sheet(concat!(
//!     "@layer reset; ",
//!     "@import url(theme.css) layer(theme) supports(display: grid) print;",
//! ));
//! assert!(report.is_clean());
//! let [CssRule::LayerStatement(_), CssRule::Import(import)] = report.syntax().rules() else {
//!     panic!("expected initial layer and import");
//! };
//! assert!(matches!(import.layer(), Some(CssImportLayer::Named(_))));
//! assert!(matches!(
//!     import.supports().expect("supports clause").condition().kind(),
//!     CssConditionKind::Leaf(CssSupportsLeaf::Declaration(_))
//! ));
//! assert!(import.media().is_some());
//! ```

pub(crate) mod block_fragments;
pub(crate) mod candidate;
pub mod color_profile;
pub mod counter_style;
pub(crate) mod custom_media;
pub(crate) mod descriptor_values;
pub mod font_face;
pub mod font_feature_values;
pub mod font_palette_values;
pub(crate) mod imports;
pub mod keyframes;
pub(crate) mod named_supports;
pub(crate) mod names;
pub mod page;
pub(crate) mod query_rule_serialization;
pub(crate) mod rule_construction;
pub(crate) mod specified_rule_graph;
pub(crate) mod syntax;
pub(crate) mod when;

#[doc(inline)]
pub use block_fragments::{
    CssBlockBody, CssBlockContext, CssBlockFragment, CssRuleList, CssStyleAncestor,
};
#[doc(inline)]
pub use candidate::{CssRuleAdmissionContext, CssRuleSyntax, CssRuleSyntaxKind};
#[doc(inline)]
pub use custom_media::CssCustomMediaRule;
#[doc(inline)]
pub use imports::CssImportRuleView;
#[doc(inline)]
pub use named_supports::CssSupportsConditionRule;
#[doc(inline)]
pub use names::{CssAtRuleName, CssDescriptorName};
#[doc(inline)]
pub use rule_construction::CssRuleConstructionError;
#[doc(inline)]
pub use syntax::{
    CssContainerRule, CssDescriptorOccurrence, CssImportLayer, CssImportRule, CssImportString,
    CssImportSupports, CssImportTarget, CssImportUrl, CssLayerBlockRule, CssLayerName,
    CssLayerNameList, CssLayerStatementRule, CssMediaRule, CssNamespaceName, CssNamespacePrefix,
    CssNamespaceRule, CssNestedDeclarationsRule, CssRule, CssScopeNestingContext, CssScopeRule,
    CssSheet, CssStyleBlock, CssStyleRule, CssSupportsRule,
};
#[doc(inline)]
pub use when::{CssElseRule, CssWhenRule};
