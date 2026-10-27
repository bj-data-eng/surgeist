//! Serialization limits and errors, rule-graph input and lexical serializers.

pub(crate) mod common_serialization;
pub(crate) mod cssom_rule_serialization;
pub(crate) mod edited_rule;
pub(crate) mod exact_decimal;
pub(crate) mod numeric_formatting;
pub(crate) mod pending_serialization;
pub(crate) mod serialization_escaping;
#[cfg(test)]
pub(crate) mod specified_provider_composition_tests;
pub(crate) mod specified_rule_serialization;
pub(crate) mod specified_serialization;

#[doc(inline)]
pub use common_serialization::{
    serialize_css_comma_separated_list, serialize_css_identifier, serialize_css_string,
    serialize_css_whitespace_separated_list,
};
#[doc(inline)]
pub use cssom_rule_serialization::{
    CssRuleCssomFormat, CssRuleCssomKind, CssRuleSerializationError,
};
#[doc(inline)]
pub use edited_rule::{CssGroupPreludeRef, CssRuleGraphInput, CssRuleSerializationLimits};
#[doc(inline)]
pub use specified_serialization::{CssSerializationError, CssSerializationLimits};
