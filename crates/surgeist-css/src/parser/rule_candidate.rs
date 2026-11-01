//! Complete CSS Syntax selection followed by explicit contextual admission.
use crate::CssParseReport;
use crate::component::{CssComponentValueLimits, CssValueOrigin};
use crate::diagnostic::error::invalid_syntax;
use crate::diagnostic::{CssRecoveryAction, CssRecoveryDiagnostic};
use crate::parser::nesting::admit_one_rule;
use crate::parser::recovery::{RecoveryState, StyleContextCaptures};
use crate::parser::rule_list::RuleParser;
use crate::parser::{ScopedBodyKind, fragments, recovery, syntax_bridge};
use crate::rule::CssRule;
use crate::rule::CssStyleAncestor;
use crate::rule::{CssRuleAdmissionContext, CssRuleSyntax};
use crate::source::CssParsedOrigin;
use crate::syntax_consumption::DocumentExt;
use crate::syntax_consumption::{self as syntax, GenericRule};
use surgeist_css_parser::document::Fault;

#[must_use]
pub(crate) fn classify_rule_syntax_with_limits(
    source: &str,
    limits: CssComponentValueLimits,
) -> CssParseReport<Option<CssRuleSyntax>> {
    let document = match syntax::source_document_with_limits(source, limits) {
        Ok(document) => document,
        Err(error) => {
            return CssParseReport::new(None, vec![syntax_bridge::arena_error(source, error)]);
        }
    };
    let snapshot = document
        .snapshot()
        .expect("original source document")
        .clone();
    let selected = syntax::consume_one_rule(&mut document.cursor(document.root_list()));
    let selected = match selected {
        Ok(GenericRule::Qualified(rule))
            if syntax::custom_property_rule_prelude(&document, &rule.prelude) =>
        {
            Err(document
                .boundary(rule.range.list, rule.range.start)
                .offset
                .expect("source qualified start"))
        }
        Ok(rule) => Ok(rule),
        Err(fault) => Err(fault.at.offset.expect("source syntax fault")),
    };
    let rule = match selected {
        Ok(rule) => rule,
        Err(offset) => {
            let position = snapshot
                .position_at(offset)
                .expect("actual syntax boundary");
            let error = invalid_syntax(cssparser::SourceLocation {
                line: position.line().value(),
                column: position.column().value() + 1,
            });
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
    let start = document
        .boundary(range.list, range.start)
        .offset
        .expect("source boundary");
    let end = if range.end > range.start {
        document
            .range(document.list(range.list)[range.end - 1])
            .expect("source node")
            .end
            .value()
    } else {
        start
    };
    let state = RecoveryState::at_depth_with_snapshot(
        source,
        0,
        StyleContextCaptures::default(),
        snapshot.clone(),
    );
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
                // A parse error the tree newly preserves is still an invalid token.
                _ => crate::component::values::CssComponentValueErrorKind::InvalidToken,
            };
            let token_origin = document.origin(node);
            let CssValueOrigin::Parsed(origin) = &token_origin else {
                unreachable!("source token origin")
            };
            let diagnostic = syntax_bridge::component_error_diagnostic(
                source,
                crate::component::CssComponentValueError::new(kind, token_origin.clone()),
                origin.span(),
                CssRecoveryAction::RetainSyntaxCandidate,
            );
            diagnostics.push(diagnostic);
        }
    }
    if at_eof {
        let span = CssParsedOrigin::from_range(&snapshot, start..end)
            .expect("at-rule unit")
            .span();
        diagnostics.push(
            CssRecoveryDiagnostic::new(
                crate::diagnostic::error::unterminated_at_rule(source, end),
                span,
                CssRecoveryAction::RetainSyntaxCandidate,
            )
            .expect("EOF is in the selected at-rule"),
        );
    }
    let (_, diagnostics) =
        recovery::finish_report(source, CssParseReport::new((), diagnostics)).into_parts();
    let candidate = CssRuleSyntax {
        origin: CssParsedOrigin::from_range(&snapshot, start..end).expect("generic source range"),
        name,
        limits,
        metrics: document.metrics(),
        diagnostics: diagnostics.clone(),
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
        CssRuleAdmissionContext::Style => {
            state.record_style_context(0);
            admit_one_rule(source, &state, |source, state| {
                RuleParser::style(source, state)
            })
        }
        CssRuleAdmissionContext::Scope(ancestor)
        | CssRuleAdmissionContext::ScopedGroup(ancestor) => {
            let body = if matches!(context, CssRuleAdmissionContext::Scope(_)) {
                ScopedBodyKind::Scope
            } else {
                ScopedBodyKind::OrdinaryGroup
            };
            let style_ancestor = ancestor == CssStyleAncestor::Present;
            if style_ancestor {
                state.record_style_context(0);
            }
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
