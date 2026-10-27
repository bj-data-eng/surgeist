//! Name-free complete keyframe consumer boundaries over original source.

use crate::parser::keyframe_fragments::{
    parse_keyframe_rule_with_context, parse_keyframe_selector_list,
};
use crate::rule::keyframes::{CssKeyframeBlock, CssKeyframeSelectorList};
use crate::source::CssParsedOrigin;

/// A complete raw selector list with its unmodified source observation.
///
/// This parser-produced carrier preserves surrounding whitespace and comments
/// in its origin. Its selector list retains authored order and duplicates.
#[derive(Clone, Debug, PartialEq)]
pub struct CssParsedKeyframeSelectors {
    pub(crate) selectors: CssKeyframeSelectorList,
    pub(crate) origin: CssParsedOrigin,
}

impl CssParsedKeyframeSelectors {
    /// Borrows selectors before endpoint normalization or numeric formatting.
    #[must_use]
    pub const fn selectors(&self) -> &CssKeyframeSelectorList {
        &self.selectors
    }

    /// Borrows the complete original input, including trivia.
    #[must_use]
    pub const fn origin(&self) -> &CssParsedOrigin {
        &self.origin
    }
}

/// Exactly one admitted detached keyframe rule and its original source range.
///
/// No enclosing `@keyframes` name or brace is created. The range includes the
/// actual selector prelude and declaration braces, excluding surrounding trivia.
/// Declaration recovery belongs to the parse report.
#[derive(Clone, Debug, PartialEq)]
pub struct CssParsedKeyframeRule {
    pub(crate) block: CssKeyframeBlock,
    pub(crate) origin: CssParsedOrigin,
}

impl CssParsedKeyframeRule {
    /// Borrows the parser-only authored block and genuine declaration positions.
    #[must_use]
    pub const fn block(&self) -> &CssKeyframeBlock {
        &self.block
    }

    /// Borrows the source range of the complete detached rule.
    #[must_use]
    pub const fn origin(&self) -> &CssParsedOrigin {
        &self.origin
    }
}
impl CssParsedKeyframeSelectors {
    /// Parses a complete raw keyframe selector list without a synthetic rule.
    ///
    /// The owning authored grammar admits `from`, `to`, bounded literal percentages
    /// and supported symbolic percentage math. Empty, invalid, or trailing input
    /// returns `None` with `RejectInput`; resource failures preserve their typed
    /// diagnostic action. Order, duplicates, trivia and original coordinates survive.
    /// Lexical EOF recovery remains visible and fails clean-report validation.
    ///
    /// The parser is not read: neither its document mode nor its component limits apply to this grammar, and only the fixed nesting ceiling bounds the input.
    #[expect(
        unused_variables,
        reason = "this grammar does not read the parser configuration"
    )]
    #[must_use]
    pub fn parse(
        parser: &crate::CssParser,
        source: impl AsRef<str>,
    ) -> crate::CssParseReport<Option<CssParsedKeyframeSelectors>> {
        parse_keyframe_selector_list(source.as_ref())
    }
}
impl CssParsedKeyframeRule {
    /// Parses exactly one complete detached keyframe rule on unmodified input.
    ///
    /// A selector prelude and real declaration braces are required. Multiple rules,
    /// outer at-rules, trailing nontrivia and structural children reject the whole
    /// input. Invalid declarations use normal keyframe recovery and retain admitted
    /// siblings with ordered original diagnostics. Empty blocks are admitted.
    /// Genuine braces count toward the fixed nesting ceiling; input and component
    /// resources share the existing parser owner. Implicit EOF closure is retained
    /// with diagnostics. This supplies no live rule identity or mutation policy.
    ///
    /// The parser supplies the document mode of the declaration admission.
    #[must_use]
    pub fn parse(
        parser: &crate::CssParser,
        source: impl AsRef<str>,
    ) -> crate::CssParseReport<Option<CssParsedKeyframeRule>> {
        parse_keyframe_rule_with_context(source.as_ref(), parser.context())
    }
}
