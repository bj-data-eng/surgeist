//! Style sheets, rules and their admission, construction and block fragments.

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
