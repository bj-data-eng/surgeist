//! Complete CSS Syntax selection followed by explicit contextual admission.
use crate::CssParseReport;
use crate::component::{CssComponentValueLimits, CssValueOrigin};
use crate::diagnostic::error::invalid_syntax;
use crate::diagnostic::{CssRecoveryAction, CssRecoveryDiagnostic};
use crate::parser::recovery::{RecoveryState, StyleContextCaptures};
use crate::parser::rule_list::RuleParser;
use crate::parser::{ScopedBodyKind, fragments, nesting, recovery, syntax_bridge};
use crate::rule::CssRule;
use crate::rule::CssStyleAncestor;
use crate::rule::{CssRuleAdmissionContext, CssRuleSyntax};
use crate::source::CssParsedOrigin;
use crate::syntax_consumption::DocumentExt;
use crate::syntax_consumption::{self as syntax, GenericRule, RuleTermination};
use cssparser::{Parser, ParserInput};
use surgeist_css_parser::document::Fault;

#[derive(Clone, Debug)]
pub(crate) struct RuleEnvelope {
    pub(super) start: usize,
    pub(super) at_termination: Option<RuleTermination>,
}

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
    let (range, name, termination, at_eof) = match rule {
        GenericRule::At(rule) => {
            let syntax::TokenKind::AtKeyword(name) = document.kind(rule.name) else {
                unreachable!("at dispatch")
            };
            (
                rule.range,
                Some(Box::<str>::from(name)),
                Some(rule.termination),
                rule.fault.is_some(),
            )
        }
        GenericRule::Qualified(rule) => (rule.range, None, None, false),
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
        envelope: RuleEnvelope {
            start,
            at_termination: termination,
        },
        limits,
        metrics: document.metrics(),
        diagnostics: diagnostics.clone(),
    };
    CssParseReport::new(Some(candidate), diagnostics)
}

pub(super) fn admit_inner(
    source: &str,
    state: RecoveryState,
    envelope: &RuleEnvelope,
    context: CssRuleAdmissionContext,
) -> CssParseReport<Option<CssRule>> {
    let working_source = crate::tokenization::prepare(source);
    let mut parser_input = ParserInput::new(&working_source);
    let mut input = Parser::new(&mut parser_input);
    let (result, mut diagnostics) = match context {
        CssRuleAdmissionContext::Stylesheet | CssRuleAdmissionContext::Group => {
            let mut parser = if context == CssRuleAdmissionContext::Stylesheet {
                RuleParser::top_level(source, state.clone())
            } else {
                RuleParser::group(source, state.clone())
            };
            let result =
                syntax_bridge::admit_envelope(source, &mut input, &mut parser, &state, envelope)
                    .map(|item| {
                        let [rule]: [CssRule; 1] =
                            item.into_rules().try_into().expect("one ordinary rule");
                        rule
                    });
            (result, parser.diagnostics)
        }
        CssRuleAdmissionContext::Style => {
            nesting::admit_detached(source, &mut input, state.clone(), envelope)
        }
        CssRuleAdmissionContext::Scope(ancestor)
        | CssRuleAdmissionContext::ScopedGroup(ancestor) => {
            let mut parser = RuleParser::scoped(
                source,
                state.clone(),
                ancestor == CssStyleAncestor::Present,
                if matches!(context, CssRuleAdmissionContext::Scope(_)) {
                    ScopedBodyKind::Scope
                } else {
                    ScopedBodyKind::OrdinaryGroup
                },
            );
            if ancestor == CssStyleAncestor::Present {
                state.record_style_context(0);
            }
            let result =
                syntax_bridge::admit_envelope(source, &mut input, &mut parser, &state, envelope)
                    .map(|item| {
                        let [rule]: [CssRule; 1] =
                            item.into_rules().try_into().expect("one scoped rule");
                        rule
                    });
            (result, parser.diagnostics)
        }
    };
    match result {
        Ok(rule) => {
            diagnostics.extend(state.take_implicit_closure_diagnostics(source));
            CssParseReport::new(Some(rule), diagnostics)
        }
        Err(error) => CssParseReport::new(
            None,
            vec![fragments::reject(
                source,
                error,
                CssRecoveryAction::RejectInput,
            )],
        ),
    }
}
