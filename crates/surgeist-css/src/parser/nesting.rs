use super::Recovered;
use super::block_units::{location_at, native_error, offset_of};
use super::recovery::RecoveryState;
use super::rule_list::{RuleListKind, RuleParser, StyleSink, recover_rule_list};
use super::syntax_types::Stream;
use crate::diagnostic::CssFeatureId;
use crate::rule::syntax::StyleContents;
use crate::rule::{CssRule, CssStyleRule};
use crate::selector::CssStyleSelectorList;

pub(super) static IMPLEMENTED_SELECTORS: &[CssFeatureId] =
    &[CssFeatureId::new("baseline.selector.nesting")];

/// The style rule whose block holds `contents`.
pub(crate) fn parse_style_rule_block<'i>(
    source: &str,
    selectors: CssStyleSelectorList,
    position: crate::source::CssSourcePosition,
    contents: Stream<'i>,
    recovery: RecoveryState,
) -> Recovered<Vec<CssRule>> {
    recovery.record_style_context(offset_of(contents.start_site()));
    let recovered = parse_style_contents(source, contents, recovery);
    Recovered {
        syntax: vec![CssRule::Style(CssStyleRule::new(
            selectors,
            recovered.syntax.declarations,
            recovered.syntax.rules,
            position,
        ))],
        diagnostics: recovered.diagnostics,
    }
}

/// The declarations and nested rules of a style rule's `contents`.
pub(crate) fn parse_style_contents<'i>(
    source: &str,
    contents: Stream<'i>,
    recovery: RecoveryState,
) -> Recovered<StyleContents> {
    recovery.record_style_context(offset_of(contents.start_site()));
    let mut grammar = RuleParser::style(source, recovery);
    let mut sink = StyleSink::new();
    recover_rule_list(
        source,
        contents,
        RuleListKind::Declarations,
        &mut grammar,
        &mut sink,
    );
    Recovered {
        syntax: sink.finish(),
        diagnostics: grammar.diagnostics,
    }
}

// A declaration run ends at a rule, and a rule's own boundary decides whether a
// failure of it ends the run too. A semicolon/EOF failure can enter the
// prelude but never reaches a block. Only actual block consumption completes
// that qualified-rule boundary.
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

/// Admits the one rule `source` holds through the grammar `build` makes: its
/// diagnostics, and the rule or the diagnostic that rejects it.
pub(super) fn admit_one_rule(
    source: &str,
    recovery: &RecoveryState,
    build: impl for<'i> FnOnce(&'i str, RecoveryState) -> RuleParser<'i>,
) -> (
    Result<CssRule, crate::diagnostic::CssRecoveryDiagnostic>,
    Vec<crate::diagnostic::CssRecoveryDiagnostic>,
) {
    let snapshot = recovery.source_snapshot().clone();
    let document = match recovery.syntax_document(source) {
        Ok(document) => document,
        Err(error) => {
            let error =
                crate::diagnostic::error::invalid_component_value(location_at(&snapshot, 0), error);
            return (
                Err(super::fragments::reject(
                    source,
                    error,
                    crate::diagnostic::CssRecoveryAction::RejectInput,
                )),
                Vec::new(),
            );
        }
    };
    let mut parser = build(source, recovery.clone());
    let result = match document.root().parse_rule(&mut parser) {
        Ok(item) => match <[CssRule; 1]>::try_from(item.into_rules()) {
            Ok([rule]) => Ok(rule),
            // A rule that is not one ordinary rule is no rule of this source.
            Err(_) => Err(super::fragments::reject(
                source,
                crate::diagnostic::error::unexpected_syntax_end(location_at(&snapshot, 0)),
                crate::diagnostic::CssRecoveryAction::RejectInput,
            )),
        },
        Err(failed) => {
            let error = if failed.error().is_denied_block() {
                super::rule_list::denied_rule_error(source, &mut parser, &document.root(), &failed)
            } else if let Some(resource) = parser.take_block_resource_error() {
                resource
            } else {
                native_error(&snapshot, failed.error())
            };
            Err(super::fragments::reject(
                source,
                error,
                crate::diagnostic::CssRecoveryAction::RejectInput,
            ))
        }
    };
    (result, parser.diagnostics)
}
