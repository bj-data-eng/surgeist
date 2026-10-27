use cssparser::{ParseError, Parser};

use super::Recovered;
use super::recovery::RecoveryState;
use super::rule_list::{RuleListUnits, RuleParser, StyleSink, recover_rule_list};
use crate::diagnostic::{CssError, CssFeatureId};
use crate::rule::syntax::StyleContents;
use crate::rule::{CssRule, CssStyleRule};
use crate::selector::CssStyleSelectorList;

pub(super) static IMPLEMENTED_SELECTORS: &[CssFeatureId] =
    &[CssFeatureId::new("baseline.selector.nesting")];

pub(crate) fn parse_style_rule_block<'i, 't>(
    source: &'i str,
    selectors: CssStyleSelectorList,
    position: crate::source::CssSourcePosition,
    input: &mut Parser<'i, 't>,
    recovery: RecoveryState,
) -> std::result::Result<Recovered<Vec<CssRule>>, ParseError<'i, CssError>> {
    recovery.record_style_context(input.position().byte_index());
    let recovered = parse_style_contents(source, input, recovery)?;
    Ok(Recovered {
        syntax: vec![CssRule::Style(CssStyleRule::new(
            selectors,
            recovered.syntax.declarations,
            recovered.syntax.rules,
            position,
        ))],
        diagnostics: recovered.diagnostics,
    })
}

pub(crate) fn parse_style_contents<'i, 't>(
    source: &'i str,
    input: &mut Parser<'i, 't>,
    recovery: RecoveryState,
) -> std::result::Result<Recovered<StyleContents>, ParseError<'i, CssError>> {
    recovery.record_style_context(input.position().byte_index());
    let mut grammar = RuleParser::style(source, recovery);
    let mut sink = StyleSink::new();
    recover_rule_list(
        source,
        input,
        &mut grammar,
        &mut sink,
        &RuleListUnits::Declarations,
    );
    Ok(Recovered {
        syntax: sink.finish(),
        diagnostics: grammar.diagnostics,
    })
}

// cssparser's custom-property-looking fallback bypasses qualified callbacks.
// A semicolon/EOF failure can enter the prelude callback but never reaches a
// block. Only actual block consumption completes that qualified-rule boundary.
#[derive(Clone, Copy)]
pub(super) enum StyleRuleBoundary {
    None,
    AtRule,
    QualifiedPrelude,
    QualifiedBlock,
}

impl StyleRuleBoundary {
    pub(super) fn partitions(self, failed_at_block: bool) -> bool {
        matches!(self, Self::AtRule | Self::QualifiedBlock)
            || (matches!(self, Self::QualifiedPrelude) && failed_at_block)
    }
}

pub(super) fn admit_detached<'i, 't>(
    source: &'i str,
    input: &mut Parser<'i, 't>,
    recovery: RecoveryState,
    envelope: &super::rule_candidate::RuleEnvelope,
) -> (
    Result<CssRule, ParseError<'i, CssError>>,
    Vec<crate::diagnostic::CssRecoveryDiagnostic>,
) {
    recovery.record_style_context(0);
    let mut parser = RuleParser::style(source, recovery.clone());
    let result =
        super::syntax_bridge::admit_envelope(source, input, &mut parser, &recovery, envelope).map(
            |item| {
                let [rule]: [CssRule; 1] = item
                    .into_rules()
                    .try_into()
                    .expect("one detached nested rule");
                rule
            },
        );
    (result, parser.diagnostics)
}
