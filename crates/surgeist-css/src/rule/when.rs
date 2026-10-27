//! Checked authored generalized conditions and conditional group fragments.
use crate::query::CssCondition;
use crate::query::CssWhenLeaf;
use crate::rule::{CssRule, CssRuleConstructionError};
use crate::selector::CssNamespaceContext;
use crate::source::CssSourcePosition;
macro_rules! conditional_rule {
    ($name:ident, $condition:ty, $borrowed:ty, $get:ident, $condition_doc:literal, $rule_doc:literal) => {
        #[doc = $rule_doc]
        ///
        /// This is a checked authored conditional group fragment. Complete enclosing assembly
        /// rechecks actual sibling adjacency and nearest-style placement atomically.
        #[derive(Clone, Debug, PartialEq)]
        pub struct $name {
            condition: $condition,
            rules: Vec<CssRule>,
            position: Option<CssSourcePosition>,
        }
        impl $name {
            pub(crate) fn new(
                condition: $condition,
                rules: Vec<CssRule>,
                position: CssSourcePosition,
            ) -> Self {
                Self {
                    condition,
                    rules,
                    position: Some(position),
                }
            }
            #[doc = $condition_doc]
            #[must_use]
            pub fn condition(&self) -> $borrowed {
                self.condition.$get()
            }
            /// Returns the child rules of the group in authored order.
            #[must_use]
            pub fn rules(&self) -> &[CssRule] {
                &self.rules
            }
            pub(crate) fn rules_mut(&mut self) -> &mut Vec<CssRule> {
                &mut self.rules
            }
            /// Returns the source position of the rule, or `None` when it was constructed rather than parsed.
            #[must_use]
            pub const fn position(&self) -> Option<CssSourcePosition> {
                self.position
            }
            /// Constructs the rule from a condition and its child rules, checking the children as
            /// the body of a conditional group outside any style rule.
            ///
            /// # Errors
            ///
            /// Returns [`CssRuleConstructionError`] when a child rule is not valid in a conditional
            /// group body under `context`.
            pub fn try_new(
                condition: $condition,
                rules: Vec<CssRule>,
                context: &CssNamespaceContext,
            ) -> Result<Self, CssRuleConstructionError> {
                Self::assemble(condition, rules, context, false)
            }
            /// Checks descendants under an explicit style ancestor. Enclosing assembly
            /// rechecks the actual ancestry; no selector or style claim is fabricated.
            pub fn try_new_in_style(
                condition: $condition,
                rules: Vec<CssRule>,
                context: &CssNamespaceContext,
            ) -> Result<Self, CssRuleConstructionError> {
                Self::assemble(condition, rules, context, true)
            }
            fn assemble(
                condition: $condition,
                rules: Vec<CssRule>,
                context: &CssNamespaceContext,
                style: bool,
            ) -> Result<Self, CssRuleConstructionError> {
                crate::rule::rule_construction::conditional_group(&rules, context, style)?;
                Ok(Self {
                    condition,
                    rules,
                    position: None,
                })
            }
        }
    };
}
conditional_rule!(
    CssWhenRule,
    CssCondition<CssWhenLeaf>,
    &CssCondition<CssWhenLeaf>,
    borrow_condition,
    "Returns the authored `@when` condition.",
    "A `@when` rule: a conditional group whose child rules apply when its generalized boolean condition holds."
);
conditional_rule!(
    CssElseRule,
    Option<CssCondition<CssWhenLeaf>>,
    Option<&CssCondition<CssWhenLeaf>>,
    as_ref,
    "Returns the authored `@else` condition, or `None` for a bare `@else` with no condition.",
    "An `@else` rule: a conditional group that continues a conditional rule chain and applies only when no earlier rule in the chain matched; an omitted condition is always true."
);
