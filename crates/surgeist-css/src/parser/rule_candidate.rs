//! Complete CSS Syntax selection followed by explicit contextual admission.
use super::*;
use crate::syntax_consumption::{self as syntax, GenericRule, RuleTermination};
use crate::{
    CssComponentValueLimits, CssParseReport, CssParserContext, CssRecoveryAction,
    CssRecoveryDiagnostic, CssStyleAncestor, CssValueOrigin,
};

/// Preliminary generic rule classification, independent of supported grammar.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CssRuleSyntaxKind {
    /// A qualified rule with a real (possibly EOF-closed) body.
    Qualified,
    /// An at-rule, including unsupported names and invalid feature preludes.
    AtRule,
}

/// The actual destination grammar of a detached rule occurrence.
///
/// These contexts carry grammar permission, not live CSSOM ancestry or identity.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CssRuleAdmissionContext {
    /// Isolated stylesheet grammar; ordering against an existing sheet is downstream.
    Stylesheet,
    /// Ordinary inner grouping rule, without style or scope ancestry.
    Group,
    /// A style rule or a grouping rule with ordinary style ancestry.
    Style,
    /// A scope body with the supplied actual style ancestry.
    Scope(CssStyleAncestor),
    /// An ordinary grouping body inside a scope, with actual style ancestry.
    ScopedGroup(CssStyleAncestor),
}

/// A semantically admitted occurrence in its owning ordinary or scoped domain.
#[derive(Clone, Debug, PartialEq)]
pub enum CssAdmittedRule {
    /// Ordinary stylesheet/group/style grammar.
    Ordinary(CssRule),
    /// Scoped selector and rule grammar.
    Scoped(CssScopedRule),
}

#[derive(Clone, Debug)]
pub(super) struct RuleEnvelope {
    pub(super) start: usize,
    pub(super) at_termination: Option<RuleTermination>,
}

/// One complete generic CSS Syntax rule, retaining its original input occurrence.
///
/// Construction is exclusively through [`classify_rule_syntax`]. A candidate
/// is retained even when its name, selector or feature grammar is unsupported.
/// Its origin excludes surrounding trivia; its snapshot is the complete input.
/// Semantic admission uses the selected original unit and the existing grammar
/// callbacks, without constructing an enclosing rule or serializing the syntax.
///
/// The whole source (including descendants and trivia) is charged once to the
/// selected component limits during classification. Admission uses that immutable
/// admitted occurrence; native grammar probes and bounded replay do not start a
/// fresh component budget. Failed admission leaves this candidate reusable.
#[derive(Clone, Debug)]
pub struct CssRuleSyntax {
    origin: CssParsedOrigin,
    name: Option<Box<str>>,
    envelope: RuleEnvelope,
    limits: CssComponentValueLimits,
    metrics: syntax::SyntaxInputMetrics,
    diagnostics: Vec<CssRecoveryDiagnostic>,
}
impl CssRuleSyntax {
    /// Returns the generic classification, before semantic grammar admission.
    #[must_use]
    pub const fn kind(&self) -> CssRuleSyntaxKind {
        if self.name.is_some() {
            CssRuleSyntaxKind::AtRule
        } else {
            CssRuleSyntaxKind::Qualified
        }
    }
    /// Returns the decoded, case-preserving at-keyword name, without `@`.
    #[must_use]
    pub fn at_rule_name(&self) -> Option<&str> {
        self.name.as_deref()
    }
    /// Returns the parser-owned source occurrence and its original coordinates.
    #[must_use]
    pub const fn origin(&self) -> &CssParsedOrigin {
        &self.origin
    }
    /// Returns the limits that admitted the complete original source.
    #[must_use]
    pub const fn limits(&self) -> CssComponentValueLimits {
        self.limits
    }
    /// Returns the cumulatively charged component count, including descendants/trivia.
    #[must_use]
    pub const fn component_count(&self) -> usize {
        self.metrics.components
    }
    /// Returns preliminary recovery independently of later semantic retention.
    #[must_use]
    pub fn diagnostics(&self) -> &[CssRecoveryDiagnostic] {
        &self.diagnostics
    }
    /// Admits the candidate under default document mode and explicit placement.
    ///
    /// `None` is semantic rejection, rather than preliminary syntax rejection.
    /// A retained rule may have inner recovery diagnostics; report cleanliness is
    /// not a rule-retention test. Typed resource errors remain in diagnostics.
    #[must_use]
    pub fn admit(
        &self,
        namespaces: &CssNamespaceContext,
        context: CssRuleAdmissionContext,
    ) -> CssParseReport<Option<CssAdmittedRule>> {
        self.admit_with_context(namespaces, context, CssParserContext::default())
    }
    /// Admits under explicit document mode, namespaces and destination grammar.
    #[must_use]
    pub fn admit_with_context(
        &self,
        namespaces: &CssNamespaceContext,
        context: CssRuleAdmissionContext,
        parser_context: CssParserContext,
    ) -> CssParseReport<Option<CssAdmittedRule>> {
        let source = self.origin.source().as_str();
        let report = fragments::bounded_execution(source, || {
            let (syntax, diagnostics) = parse_bounded(
                source,
                self.origin.source(),
                0,
                BoundedParseContext::Candidate {
                    envelope: self.envelope.clone(),
                    context,
                },
                StyleContextCaptures::with_namespaces(namespaces),
                parser_context,
            )
            .into_parts();
            let BoundedParseSyntax::Candidate(rule) = syntax else {
                unreachable!("candidate replay retains its carrier")
            };
            let (rules, diagnostics) = match rule {
                Some(CssAdmittedRule::Ordinary(rule)) => {
                    let (rule, diagnostics) =
                        conditional_chains::rule(source, rule, diagnostics).into_parts();
                    (rule.map(CssAdmittedRule::Ordinary), diagnostics)
                }
                Some(CssAdmittedRule::Scoped(rule)) => {
                    let mut rules = vec![rule];
                    let mut diagnostics = diagnostics;
                    conditional_chains::completed_list(
                        source,
                        crate::syntax::CssParserRuleChildrenMut::Scoped(&mut rules),
                        &mut diagnostics,
                    );
                    (rules.pop().map(CssAdmittedRule::Scoped), diagnostics)
                }
                None => (None, diagnostics),
            };
            CssParseReport::new(rules, diagnostics)
        });
        self.with_preliminary_diagnostics(recovery::finish_report(source, report))
    }

    /// Admits this complete raw occurrence as a margin child in a Page destination.
    /// No containing Page rule is fabricated. Outside-Page hierarchy and live
    /// insertion order remain CSSOM decisions after preliminary classification.
    #[must_use]
    pub fn admit_page_margin_rule(&self) -> CssParseReport<Option<crate::CssMarginRule>> {
        self.admit_page_margin_rule_with_context(CssParserContext::default())
    }
    /// Uses the actual document mode and the already charged original occurrence.
    /// The existing Page name/prelude/body callbacks preserve applicable
    /// declarations, priority, local recovery and original coordinates. Resource
    /// failure rejects the complete preparation with its typed cause; a reusable
    /// candidate never exposes a partial child. Unsupported names, invalid
    /// framing/preludes and Page placement failures keep distinct typed errors.
    #[must_use]
    pub fn admit_page_margin_rule_with_context(
        &self,
        parser_context: CssParserContext,
    ) -> CssParseReport<Option<crate::CssMarginRule>> {
        let source = self.origin.source().as_str();
        let report = fragments::bounded_execution(source, || {
            let state = RecoveryState::at_depth_with_snapshot(
                source,
                0,
                StyleContextCaptures::default(),
                self.origin.source().clone(),
            )
            .with_parser_context(parser_context);
            super::page::admit_margin_rule(source, state, &self.envelope)
        });
        let (margin, diagnostics) = recovery::finish_report(source, report).into_parts();
        let margin = margin.map(|margin| margin.with_detached_origin(self.origin.clone()));
        self.with_preliminary_diagnostics(CssParseReport::new(margin, diagnostics))
    }

    fn with_preliminary_diagnostics<T>(&self, report: CssParseReport<T>) -> CssParseReport<T> {
        let (rule, mut diagnostics) = report.into_parts();
        // Preserve preliminary recovery as a multiset: several EOF closures may
        // share a span/action. Only equal occurrences already emitted are reused.
        let mut additional = Vec::new();
        for (index, diagnostic) in self.diagnostics.iter().enumerate() {
            let position = diagnostic.error().position().byte_offset();
            let start = self
                .diagnostics
                .partition_point(|item| item.error().position().byte_offset() < position);
            let needed = self.diagnostics[start..=index]
                .iter()
                .filter(|item| *item == diagnostic)
                .count();
            let first = diagnostics
                .partition_point(|item| item.error().position().byte_offset() < position);
            let end = diagnostics
                .partition_point(|item| item.error().position().byte_offset() <= position);
            if diagnostics[first..end]
                .iter()
                .filter(|item| *item == diagnostic)
                .count()
                < needed
            {
                additional.push(diagnostic.clone());
            }
        }
        diagnostics.extend(additional);
        CssParseReport::new(rule, diagnostics)
    }
}

/// Classifies exactly one complete CSS Syntax rule over unmodified original input.
///
/// Empty/whitespace-only input, a qualified candidate lacking a body, a custom
/// property-shaped qualified candidate, and trailing nontrivia return `None`.
/// `@unknown {}` and `@import;` retain candidates independently of semantic
/// admission. No BOM, CDO/CDC or fabricated wrapper is stripped or introduced.
/// Recovery and resource diagnostics remain distinct from candidate retention.
#[must_use]
pub fn classify_rule_syntax(source: &str) -> CssParseReport<Option<CssRuleSyntax>> {
    classify_rule_syntax_with_limits(source, CssComponentValueLimits::default())
}

/// Classifies with cumulative limits over the complete original input.
///
/// Bytes include trailing input. Components include all descendants and trivia,
/// not one fresh allowance per rule or parsing stage. Resource failure publishes
/// no candidate; retrying with a larger allowance creates a new source occurrence.
#[must_use]
pub fn classify_rule_syntax_with_limits(
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
        .source
        .as_ref()
        .expect("original source document")
        .original
        .clone();
    let selected = syntax::consume_one_rule(&mut document.cursor(document.root));
    let selected = match selected {
        Ok(GenericRule::Qualified(rule))
            if syntax::custom_property_rule_prelude(&document, &rule.prelude) =>
        {
            Err(document
                .boundary(rule.range.list, rule.range.start)
                .source
                .expect("source qualified start")
                .offset)
        }
        Ok(rule) => Ok(rule),
        Err(fault) => Err(fault.at.source.expect("source syntax fault").offset),
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
            let syntax::TokenKind::AtKeyword(name) = document.nodes[rule.name].token().kind()
            else {
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
        .source
        .expect("source boundary")
        .offset;
    let end = if range.end > range.start {
        document
            .range(document.lists[range.list][range.end - 1])
            .expect("source node")
            .end
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
    for node in &document.nodes {
        if let syntax::SyntaxNode::Error { token, cause } = node {
            let kind = match cause {
                syntax::SyntaxTokenFault::BadString => crate::CssComponentValueErrorKind::BadString,
                syntax::SyntaxTokenFault::BadUrl => crate::CssComponentValueErrorKind::BadUrl,
                syntax::SyntaxTokenFault::UnexpectedCloser => {
                    crate::CssComponentValueErrorKind::UnmatchedClosingDelimiter
                }
            };
            let CssValueOrigin::Parsed(origin) = token.origin.as_ref() else {
                unreachable!("source token origin")
            };
            let diagnostic = syntax_bridge::component_error_diagnostic(
                source,
                crate::CssComponentValueError::new(kind, token.origin.clone().into_owned()),
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
                crate::error::unterminated_at_rule(source, end),
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
        metrics: document.metrics,
        diagnostics: diagnostics.clone(),
    };
    CssParseReport::new(Some(candidate), diagnostics)
}

pub(super) fn admit_inner(
    source: &str,
    state: RecoveryState,
    envelope: &RuleEnvelope,
    context: CssRuleAdmissionContext,
) -> CssParseReport<Option<CssAdmittedRule>> {
    let working_source = crate::tokenization::prepare(source);
    let mut parser_input = ParserInput::new(&working_source);
    let mut input = Parser::new(&mut parser_input);
    let (result, mut diagnostics) = match context {
        CssRuleAdmissionContext::Stylesheet | CssRuleAdmissionContext::Group => {
            let mut parser = if context == CssRuleAdmissionContext::Stylesheet {
                StrictRuleParser::top_level(source, state.clone())
            } else {
                StrictRuleParser::nested(source, state.clone())
            };
            let result =
                syntax_bridge::admit_envelope(source, &mut input, &mut parser, &state, envelope)
                    .map(|rules| {
                        let [rule]: [CssRule; 1] = rules.try_into().expect("one ordinary rule");
                        CssAdmittedRule::Ordinary(rule)
                    });
            (result, parser.diagnostics)
        }
        CssRuleAdmissionContext::Style => {
            nesting::admit_detached(source, &mut input, state.clone(), envelope)
        }
        CssRuleAdmissionContext::Scope(ancestor)
        | CssRuleAdmissionContext::ScopedGroup(ancestor) => {
            let mut parser = ScopedRuleParser {
                source,
                diagnostics: Vec::new(),
                recovery: state.clone(),
                has_style_ancestor: ancestor == CssStyleAncestor::Present,
                body: if matches!(context, CssRuleAdmissionContext::Scope(_)) {
                    ScopedBodyKind::Scope
                } else {
                    ScopedBodyKind::OrdinaryGroup
                },
                boundary: nesting::StyleRuleBoundary::None,
                qualified_resource_error: None,
                rejected_at_rule_start: None,
            };
            if ancestor == CssStyleAncestor::Present {
                state.record_style_context(0);
            }
            let result =
                syntax_bridge::admit_envelope(source, &mut input, &mut parser, &state, envelope)
                    .map(|item| {
                        let ScopedBlockItem::Rules(rules) = item else {
                            unreachable!("detached driver never admits a declaration")
                        };
                        let [rule]: [CssScopedRule; 1] = rules.try_into().expect("one scoped rule");
                        CssAdmittedRule::Scoped(rule)
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
