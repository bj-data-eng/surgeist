//! One complete detached Page margin rule over its original supplied input.
use crate::{CssComponentValueLimits, CssMarginRule, CssParseReport, CssParserContext};

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
