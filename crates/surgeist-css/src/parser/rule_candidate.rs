//! Complete CSS Syntax selection followed by explicit contextual admission.
use crate::CssParseReport;
use crate::component::{CssComponentValueLimits, CssValueOrigin};
use crate::diagnostic::error::invalid_syntax;
use crate::diagnostic::{CssRecoveryAction, CssRecoveryDiagnostic};
use crate::parser::nesting::admit_one_rule;
use crate::parser::recovery::{RecoveryState, SharedNamespaces};
use crate::parser::rule_list::RuleParser;
use crate::parser::{ScopedBodyKind, fragments, recovery, tree_build_diagnostics};
use crate::rule::CssRule;
use crate::rule::CssStyleAncestor;
use crate::rule::{CssRuleAdmissionContext, CssRuleSyntax};
use crate::source::{CssParsedOrigin, CssSourceSnapshot};
use crate::syntax_consumption::DocumentExt;
use crate::syntax_consumption::{self as syntax, GenericRule};
use surgeist_css_parser::document::Fault;

#[must_use]
pub(crate) fn classify_rule_syntax_with_limits(
    source: &str,
    limits: CssComponentValueLimits,
) -> CssParseReport<Option<CssRuleSyntax>> {
    classify_rule_input(source, None, limits)
}

pub(crate) fn classify_rule_snapshot_with_limits(
    source: &CssSourceSnapshot,
    limits: CssComponentValueLimits,
) -> CssParseReport<Option<CssRuleSyntax>> {
    classify_rule_input(source.as_str(), Some(source), limits)
}

fn classify_rule_input(
    source: &str,
    original: Option<&CssSourceSnapshot>,
    limits: CssComponentValueLimits,
) -> CssParseReport<Option<CssRuleSyntax>> {
    let document = match original {
        Some(snapshot) => syntax::source_document_snapshot_with_limits(snapshot, limits),
        None => syntax::source_document_with_limits(source, limits),
    };
    let (mut candidate, mut diagnostics) = match document {
        Ok(document) => classify_document(source, &document, limits),
        Err(error) => CssParseReport::new(
            None,
            vec![tree_build_diagnostics::tree_build_failure(source, error)],
        ),
    }
    .into_parts();
    if let Some(snapshot) = original {
        for diagnostic in &mut diagnostics {
            diagnostic.bind_source(snapshot);
        }
    }
    if let Some(candidate) = &mut candidate {
        candidate.diagnostics = diagnostics.clone();
    }
    CssParseReport::new(candidate, diagnostics)
}

fn classify_document(
    source: &str,
    document: &crate::parser::syntax_types::Document<'_>,
    limits: CssComponentValueLimits,
) -> CssParseReport<Option<CssRuleSyntax>> {
    let snapshot = document
        .snapshot()
        .expect("original source document")
        .clone();
    let selected = syntax::consume_one_rule(&mut document.cursor(document.root_list()));
    let selected = match selected {
        Ok(GenericRule::Qualified(rule))
            if syntax::custom_property_rule_prelude(document, &rule.prelude) =>
        {
            Err(document
                .boundary(rule.range.list, rule.range.start)
                .source_offset())
        }
        Ok(rule) => Ok(rule),
        Err(fault) => Err(fault.at.source_offset()),
    };
    let rule = match selected {
        Ok(rule) => rule,
        Err(offset) => {
            let error = invalid_syntax(snapshot.position_or_walk(offset));
            return recovery::finish_report(
                source,
                CssParseReport::new(
                    None,
                    vec![fragments::reject(
                        source,
                        error,
                        CssRecoveryAction::RejectInput,
                    )],
                ),
            );
        }
    };
    let (range, name, at_eof) = match rule {
        GenericRule::At(rule) => {
            let syntax::TokenKind::AtKeyword(name) = document.kind(rule.name) else {
                unreachable!("at dispatch")
            };
            (
                rule.range,
                Some(Box::<str>::from(name)),
                rule.fault.is_some(),
            )
        }
        GenericRule::Qualified(rule) => (rule.range, None, false),
    };
    let start = document.boundary(range.list, range.start).source_offset();
    let end = range
        .end
        .checked_sub(1)
        .filter(|&last| last >= range.start)
        .and_then(|last| document.list(range.list).get(last))
        .and_then(|&node| document.range(node))
        .map_or(start, |interval| interval.end.value());
    let state =
        RecoveryState::at_depth_with_snapshot(0, SharedNamespaces::default(), snapshot.clone());
    state.retain_component_closures(state.component_openings_in(0..source.len()));
    let mut diagnostics = state.take_implicit_closure_diagnostics(source);
    for node in document.nodes() {
        if let Some(fault) = document.component(node).fault() {
            let kind = match fault {
                Fault::BadString => crate::component::values::CssComponentValueErrorKind::BadString,
                Fault::BadUrl => crate::component::values::CssComponentValueErrorKind::BadUrl,
                Fault::UnexpectedCloser => {
                    crate::component::values::CssComponentValueErrorKind::UnmatchedClosingDelimiter
                }
                // A parse error the tree preserves is an invalid token.
                _ => crate::component::values::CssComponentValueErrorKind::InvalidToken,
            };
            let token_origin = document.origin(node);
            let CssValueOrigin::Parsed(origin) = &token_origin else {
                unreachable!("source token origin")
            };
            let diagnostic = tree_build_diagnostics::component_error_diagnostic(
                source,
                crate::component::CssComponentValueError::new(kind, token_origin.clone()),
                origin.span(),
                CssRecoveryAction::RetainSyntaxCandidate,
            );
            diagnostics.push(diagnostic);
        }
    }
    if at_eof {
        let span = CssParsedOrigin::covering(&snapshot, start..end).span();
        diagnostics.push(
            CssRecoveryDiagnostic::new(
                crate::diagnostic::error::unterminated_at_rule(span.end()),
                span,
                CssRecoveryAction::RetainSyntaxCandidate,
            )
            .expect("EOF is in the selected at-rule"),
        );
    }
    let (_, diagnostics) =
        recovery::finish_report(source, CssParseReport::new((), diagnostics)).into_parts();
    let candidate = CssRuleSyntax {
        origin: CssParsedOrigin::covering(&snapshot, start..end),
        name,
        limits,
        metrics: document.metrics(),
        diagnostics: Vec::new(),
    };
    CssParseReport::new(Some(candidate), diagnostics)
}

pub(super) fn admit_inner(
    source: &str,
    state: RecoveryState,
    context: CssRuleAdmissionContext,
) -> CssParseReport<Option<CssRule>> {
    let (result, mut diagnostics) = match context {
        CssRuleAdmissionContext::Stylesheet => admit_one_rule(source, &state, |source, state| {
            RuleParser::top_level(source, state)
        }),
        CssRuleAdmissionContext::Group => admit_one_rule(source, &state, |source, state| {
            RuleParser::group(source, state)
        }),
        CssRuleAdmissionContext::Style => admit_one_rule(source, &state, |source, state| {
            RuleParser::style(source, state)
        }),
        CssRuleAdmissionContext::Scope(ancestor)
        | CssRuleAdmissionContext::ScopedGroup(ancestor) => {
            let body = if matches!(context, CssRuleAdmissionContext::Scope(_)) {
                ScopedBodyKind::Scope
            } else {
                ScopedBodyKind::OrdinaryGroup
            };
            let style_ancestor = ancestor == CssStyleAncestor::Present;
            admit_one_rule(source, &state, move |source, state| {
                RuleParser::scoped(source, state, style_ancestor, body)
            })
        }
    };
    match result {
        Ok(rule) => {
            diagnostics.extend(state.take_implicit_closure_diagnostics(source));
            CssParseReport::new(Some(rule), diagnostics)
        }
        Err(diagnostic) => CssParseReport::new(None, vec![diagnostic]),
    }
}
