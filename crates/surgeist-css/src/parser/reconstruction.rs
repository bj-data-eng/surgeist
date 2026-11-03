//! The entry of every rule-list and rule-fragment parse over the caller's
//! unmodified source, and the one source whose end leaves a prelude open past
//! the nesting limit.

use super::recovery::{self, RecoveryState, SharedNamespaces};
use super::structure::{self, StructuralListContext};
use super::{ScopedBodyKind, fragments, parse_sheet_inner};
use crate::diagnostic::error::from_parse_error;
use crate::rule::{CssRule, CssSheet, CssStyleBlock};
use crate::source::CssSourceSnapshot;

#[derive(Clone)]
pub(crate) enum BoundedParseContext {
    Sheet,
    OneRule,
    Candidate {
        context: crate::rule::CssRuleAdmissionContext,
    },
    StyleBlock,
    GroupBlock,
    ScopedBlock {
        has_style_ancestor: bool,
        body: ScopedBodyKind,
    },
}

// The entry preserves the public entry's admission boundary. Fragments retain
// their native grammar and cannot become recovering lists.
pub(crate) enum BoundedParseSyntax {
    Rules(CssSheet),
    OneRule(Option<CssRule>),
    Candidate(Option<CssRule>),
    StyleBlock(Option<CssStyleBlock>),
    GroupBlock(Option<crate::rule::CssBlockFragment<crate::rule::CssRuleList>>),
    ScopedBlock(Option<crate::rule::CssBlockFragment<crate::rule::CssRuleList>>),
}

fn bounded_report<T>(
    report: crate::CssParseReport<T>,
    syntax: impl FnOnce(T) -> BoundedParseSyntax,
) -> crate::CssParseReport<BoundedParseSyntax> {
    let (value, diagnostics) = report.into_parts();
    crate::CssParseReport::new(syntax(value), diagnostics)
}

impl BoundedParseContext {
    /// How the list the context reads delimits its units.
    fn list_context(&self) -> StructuralListContext {
        match self {
            Self::Sheet => StructuralListContext::Stylesheet,
            Self::Candidate { .. }
            | Self::OneRule
            | Self::StyleBlock
            | Self::GroupBlock
            | Self::ScopedBlock {
                has_style_ancestor: false,
                body: ScopedBodyKind::OrdinaryGroup,
            } => StructuralListContext::OneRule,
            Self::ScopedBlock {
                has_style_ancestor: true,
                ..
            }
            | Self::ScopedBlock {
                body: ScopedBodyKind::Scope,
                ..
            } => StructuralListContext::BlockContents,
        }
    }
}

pub(crate) fn parse_sheet_bounded(
    source: &str,
    source_snapshot: &CssSourceSnapshot,
    context: BoundedParseContext,
    namespaces: SharedNamespaces,
    parser_context: crate::CssParserContext,
) -> crate::CssParseReport<CssSheet> {
    let report = parse_bounded(source, source_snapshot, context, namespaces, parser_context);
    let (syntax, diagnostics) = report.into_parts();
    let BoundedParseSyntax::Rules(sheet) = syntax else {
        return crate::CssParseReport::new(CssSheet::new(), diagnostics);
    };
    crate::CssParseReport::new(sheet, diagnostics)
}

pub(crate) fn parse_bounded(
    source: &str,
    source_snapshot: &CssSourceSnapshot,
    context: BoundedParseContext,
    namespaces: SharedNamespaces,
    parser_context: crate::CssParserContext,
) -> crate::CssParseReport<BoundedParseSyntax> {
    let state = RecoveryState::at_depth_with_snapshot(0, namespaces, source_snapshot.clone())
        .with_parser_context(parser_context);
    let report = parse_over(source, context, state);
    let (syntax, mut diagnostics) = report.into_parts();
    // Native locations describe this admitted view, whose masks preserve bytes
    // but can change UTF-16 columns. Resolve coordinates from original bytes
    // without repeating token inference or changing responsible-token ownership.
    for diagnostic in &mut diagnostics {
        diagnostic.resolve_original_coordinates(source_snapshot);
    }
    let eof_limit = diagnostics.iter().any(|diagnostic| {
        diagnostic.action() == crate::diagnostic::CssRecoveryAction::StopAtNestingLimit
            && diagnostic.span().end().byte_offset().value() == source.len()
    });
    if eof_limit {
        diagnostics.retain(|diagnostic| {
            diagnostic.action() != crate::diagnostic::CssRecoveryAction::RetainWithImplicitClosure
        });
    }
    crate::CssParseReport::new(syntax, diagnostics)
}

/// Parses `source`, whose tokens `state` reads, in `context`.
fn parse_over(
    source: &str,
    context: BoundedParseContext,
    state: RecoveryState,
) -> crate::CssParseReport<BoundedParseSyntax> {
    if let Some(limit) = state
        .structure()
        .ok()
        .and_then(|document| structure::eof_limit(&document, 0, context.list_context()))
    {
        // A terminal fragment prelude cannot be erased and replaced by a
        // different original unit. Lists alone recover by masking this unit.
        let (syntax, mut diagnostics) = match context {
            BoundedParseContext::OneRule => (BoundedParseSyntax::OneRule(None), Vec::new()),
            BoundedParseContext::Candidate { .. } => {
                (BoundedParseSyntax::Candidate(None), Vec::new())
            }
            BoundedParseContext::StyleBlock => (BoundedParseSyntax::StyleBlock(None), Vec::new()),
            BoundedParseContext::GroupBlock => (BoundedParseSyntax::GroupBlock(None), Vec::new()),
            BoundedParseContext::ScopedBlock { .. } => {
                (BoundedParseSyntax::ScopedBlock(None), Vec::new())
            }
            BoundedParseContext::Sheet => {
                let masked = mask_source_span(source, limit.unit_start, source.len());
                let state = state.with_text(&masked);
                parse_over(&masked, context, state).into_parts()
            }
        };
        diagnostics.retain(|diagnostic| {
            diagnostic.action() != crate::diagnostic::CssRecoveryAction::RetainWithImplicitClosure
        });
        let error = from_parse_error(
            source,
            crate::diagnostic::error::nesting_limit(
                state.source_position(limit.opening_offset),
                recovery::STRUCTURAL_NESTING_LIMIT,
                limit.enclosing_production,
            ),
        );
        if let Some(span) = crate::source::CssSourceSpan::new(
            crate::source::CssSourcePosition::from_byte_offset_in(
                source,
                if matches!(syntax, BoundedParseSyntax::Rules(_)) {
                    limit.unit_start
                } else {
                    0
                },
            ),
            crate::source::CssSourcePosition::from_byte_offset_in(source, source.len()),
        ) && let Some(diagnostic) = crate::diagnostic::CssRecoveryDiagnostic::new(
            error,
            span,
            crate::diagnostic::CssRecoveryAction::StopAtNestingLimit,
        ) {
            diagnostics.push(diagnostic);
        }
        return crate::CssParseReport::new(syntax, diagnostics);
    }
    match context {
        BoundedParseContext::Sheet => {
            bounded_report(parse_sheet_inner(source, state), BoundedParseSyntax::Rules)
        }
        BoundedParseContext::OneRule => bounded_report(
            fragments::parse_rule_inner(source, state),
            BoundedParseSyntax::OneRule,
        ),
        BoundedParseContext::Candidate { context } => bounded_report(
            super::rule_candidate::admit_inner(source, state, context),
            BoundedParseSyntax::Candidate,
        ),
        BoundedParseContext::StyleBlock => bounded_report(
            fragments::parse_style_block_inner(source, state),
            BoundedParseSyntax::StyleBlock,
        ),
        BoundedParseContext::GroupBlock => bounded_report(
            fragments::parse_group_block_inner(source, state),
            BoundedParseSyntax::GroupBlock,
        ),
        BoundedParseContext::ScopedBlock {
            has_style_ancestor,
            body,
        } => bounded_report(
            fragments::parse_scoped_block_inner(source, state, has_style_ancestor, body),
            BoundedParseSyntax::ScopedBlock,
        ),
    }
}

/// `source` with the bytes of `start..end` blanked, keeping the line breaks, so
/// that every offset stays where it was.
fn mask_source_span(source: &str, start: usize, end: usize) -> String {
    let mut masked = source.as_bytes().to_vec();
    for byte in masked
        .get_mut(start.min(source.len())..end.min(source.len()))
        .into_iter()
        .flatten()
    {
        if !matches!(*byte, b'\n' | b'\r' | b'\x0c') {
            *byte = b' ';
        }
    }
    String::from_utf8(masked).unwrap_or_else(|_| source.to_owned())
}

pub(crate) fn rule_start(rule: &CssRule) -> usize {
    match rule {
        CssRule::CustomMedia(rule) => {
            return rule
                .position()
                .expect("parsed custom-media rule")
                .byte_offset()
                .value();
        }
        CssRule::FontFeatureValues(rule) => {
            return rule
                .position()
                .expect("parser-owned font rule has a source position")
                .byte_offset()
                .value();
        }
        CssRule::FontPaletteValues(rule) => {
            return rule
                .position()
                .expect("parser-owned palette rule has a source position")
                .byte_offset()
                .value();
        }
        CssRule::ColorProfile(rule) => {
            return rule
                .position()
                .expect("parser-owned profile rule has a source position")
                .byte_offset()
                .value();
        }
        CssRule::Import(rule) => rule.position().expect("parsed import rule"),
        CssRule::Namespace(rule) => rule.position().expect("parsed namespace rule"),
        CssRule::CounterStyle(rule) => rule.position(),
        CssRule::Page(rule) => rule.position(),
        CssRule::LayerStatement(rule) => rule.position(),
        CssRule::LayerBlock(rule) => rule.position().expect("parser-produced rule position"),
        CssRule::FontFace(rule) => rule.position().expect("parser-produced rule position"),
        CssRule::Keyframes(rule) => rule.position(),
        CssRule::Style(rule) => rule.position(),
        CssRule::NestedDeclarations(rule) => rule.position(),
        CssRule::Media(rule) => rule.position().expect("parser-produced rule position"),
        CssRule::Supports(rule) => rule.position().expect("parser-produced rule position"),
        CssRule::SupportsCondition(rule) => {
            return rule
                .position()
                .expect("parsed named supports rule")
                .byte_offset()
                .value();
        }
        CssRule::Container(rule) => rule.position().expect("parser-produced rule position"),
        CssRule::When(rule) => rule
            .position()
            .expect("parser-produced conditional position"),
        CssRule::Else(rule) => rule
            .position()
            .expect("parser-produced conditional position"),
        CssRule::Scope(rule) => rule.position().expect("parser-produced rule position"),
    }
    .byte_offset()
    .value()
}
