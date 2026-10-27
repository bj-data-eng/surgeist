//! Checked authored generalized conditions and conditional group fragments.
use crate::query::CssCondition;
use crate::query::CssWhenLeaf;
use crate::rule::{CssRule, CssRuleConstructionError};
use crate::selector::CssNamespaceContext;
use crate::source::CssSourcePosition;
macro_rules! conditional_rule {
    ($name:ident, $condition:ty, $borrowed:ty, $get:ident) => {
        /// A checked authored conditional group fragment. Complete enclosing assembly
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
            #[must_use]
            pub fn condition(&self) -> $borrowed {
                self.condition.$get()
            }
            #[must_use]
            pub fn rules(&self) -> &[CssRule] {
                &self.rules
            }
            pub(crate) fn rules_mut(&mut self) -> &mut Vec<CssRule> {
                &mut self.rules
            }
            #[must_use]
            pub const fn position(&self) -> Option<CssSourcePosition> {
                self.position
            }
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
    borrow_condition
);
conditional_rule!(
    CssElseRule,
    Option<CssCondition<CssWhenLeaf>>,
    Option<&CssCondition<CssWhenLeaf>>,
    as_ref
);
