//! One complete detached Page margin rule over its original supplied input.
use crate::{CssComponentValueLimits, CssMarginRule, CssParseReport, CssParserContext};

/// Parses exactly one complete margin at-rule in a Page destination.
///
/// The decoded at-keyword and empty prelude use the actual Page grammar;
/// declaration recovery retains applicable properties/custom declarations and
/// priority. Real framing and origins come from the complete raw input, including
/// escaped names and surrounding trivia. No containing Page or brace is invented.
/// Extra nontrivia rejects the complete input. EOF closures use existing recovery.
///
/// `None` means failed preparation: typed diagnostics distinguish preliminary
/// Syntax, placement, unsupported-name and resource failures. A retained child
/// may have local recovery. This API establishes a Page destination; live CSSOM
/// validates outside-Page hierarchy and insertion order after classification.
#[must_use]
pub fn parse_page_margin_rule(source: &str) -> CssParseReport<Option<CssMarginRule>> {
    parse_page_margin_rule_with_limits(source, CssComponentValueLimits::default())
}
/// Charges the full original input once, including trivia/descendants/trailing
/// source, then admits that exact candidate under existing bounded grammar work.
/// Resource failure publishes no child and preserves its typed cause for retry.
#[must_use]
pub fn parse_page_margin_rule_with_limits(
    source: &str,
    limits: CssComponentValueLimits,
) -> CssParseReport<Option<CssMarginRule>> {
    parse_with_context(source, limits, CssParserContext::default())
}
pub(crate) fn parse_with_context(
    source: &str,
    limits: CssComponentValueLimits,
    context: CssParserContext,
) -> CssParseReport<Option<CssMarginRule>> {
    let (candidate, diagnostics) =
        super::classify_rule_syntax_with_limits(source, limits).into_parts();
    match candidate {
        Some(candidate) => candidate.admit_page_margin_rule_with_context(context),
        None => CssParseReport::new(None, diagnostics),
    }
}
