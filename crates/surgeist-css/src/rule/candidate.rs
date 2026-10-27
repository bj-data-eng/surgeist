//! Complete CSS Syntax selection followed by explicit contextual admission.
use crate::component::CssComponentValueLimits;
use crate::diagnostic::CssRecoveryDiagnostic;
use crate::parser::reconstruction::{BoundedParseContext, BoundedParseSyntax, parse_bounded};
use crate::parser::recovery::{RecoveryState, StyleContextCaptures};
use crate::parser::rule_candidate::{RuleEnvelope, classify_rule_syntax_with_limits};
use crate::parser::{conditional_chains, fragments, recovery};
use crate::rule::CssRule;
use crate::rule::CssStyleAncestor;
use crate::selector::CssNamespaceContext;
use crate::source::CssParsedOrigin;
use crate::syntax_consumption::{self as syntax};
use crate::{CssParseReport, CssParserContext};

/// Preliminary generic rule classification, independent of supported grammar.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
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
#[non_exhaustive]
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

/// One complete generic CSS Syntax rule, retaining its original input occurrence.
///
/// Construction is exclusively through [`crate::rule::CssRuleSyntax::classify`]. A candidate
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
    pub(crate) origin: CssParsedOrigin,
    pub(crate) name: Option<Box<str>>,
    pub(crate) envelope: RuleEnvelope,
    pub(crate) limits: CssComponentValueLimits,
    pub(crate) metrics: syntax::SyntaxInputMetrics,
    pub(crate) diagnostics: Vec<CssRecoveryDiagnostic>,
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
    ) -> CssParseReport<Option<CssRule>> {
        self.admit_with_context(namespaces, context, CssParserContext::default())
    }
    /// Admits under explicit document mode, namespaces and destination grammar.
    #[must_use]
    pub fn admit_with_context(
        &self,
        namespaces: &CssNamespaceContext,
        context: CssRuleAdmissionContext,
        parser_context: CssParserContext,
    ) -> CssParseReport<Option<CssRule>> {
        let source = self.origin.source().as_str();
        let refused = || None;
        let report = fragments::bounded_report(source, refused, || {
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
            let (rule, diagnostics) = match rule {
                Some(rule) => conditional_chains::rule(source, rule, diagnostics).into_parts(),
                None => (None, diagnostics),
            };
            CssParseReport::new(rule, diagnostics)
        });
        self.with_preliminary_diagnostics(recovery::finish_report(source, report))
    }

    /// Admits this complete raw occurrence as a margin child in a Page destination.
    /// No containing Page rule is fabricated. Outside-Page hierarchy and live
    /// insertion order remain CSSOM decisions after preliminary classification.
    #[must_use]
    pub fn admit_page_margin_rule(
        &self,
    ) -> CssParseReport<Option<crate::rule::page::CssMarginRule>> {
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
    ) -> CssParseReport<Option<crate::rule::page::CssMarginRule>> {
        let source = self.origin.source().as_str();
        let refused = || None;
        let report = fragments::bounded_report(source, refused, || {
            let state = RecoveryState::at_depth_with_snapshot(
                source,
                0,
                StyleContextCaptures::default(),
                self.origin.source().clone(),
            )
            .with_parser_context(parser_context);
            crate::parser::page::admit_margin_rule(source, state, &self.envelope)
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
impl CssRuleSyntax {
    /// Classifies with cumulative limits over the complete original input.
    ///
    /// Bytes include trailing input. Components include all descendants and trivia,
    /// not one fresh allowance per rule or parsing stage. Resource failure publishes
    /// no candidate; retrying with a larger allowance creates a new source occurrence.
    ///
    /// The parser supplies the cumulative component limits.
    #[must_use]
    pub fn classify(
        parser: &crate::CssParser,
        source: impl AsRef<str>,
    ) -> CssParseReport<Option<CssRuleSyntax>> {
        classify_rule_syntax_with_limits(source.as_ref(), parser.component_limits())
    }
}
