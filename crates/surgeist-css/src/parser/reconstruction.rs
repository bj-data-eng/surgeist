//! Bounded native replay, ancestor admission, and reconstruction over original source.

#[cfg(test)]
use super::parse_sheet;
use super::recovery::{
    self, GroupKind, RecoveryState, StructuralListContext, StructuralParent,
    StructuralPreflightOutcome, StyleContextCaptures, preflight_specialized_eof_limit,
    preflight_structural_nesting,
};
use super::{
    ScopedBodyKind, fragments, parse_scoped_context_inner, parse_sheet_inner,
    parse_style_context_inner,
};
use crate::CssSourceSnapshot;
use crate::error::from_parse_error;
use crate::syntax::*;
use crate::{CssElseRule, CssScopedElseRule, CssScopedWhenRule, CssWhenRule};

#[derive(Clone)]
pub(super) enum BoundedParseContext {
    Rules {
        top_level: bool,
    },
    OneRule,
    StyleBlock,
    GroupBlock,
    ScopedBlock {
        has_style_ancestor: bool,
        body: ScopedBodyKind,
    },
    Style,
    Scoped {
        has_style_ancestor: bool,
        body: ScopedBodyKind,
    },
}

// The replay owner preserves the public entry's admission boundary. Fragments
// retain their native grammar and cannot become recovering lists during replay.
pub(super) enum BoundedParseSyntax {
    Rules(CssSheet),
    OneRule(Option<CssRule>),
    StyleBlock(Option<CssStyleBlock>),
    GroupBlock(Option<crate::CssBlockFragment<crate::CssRuleList>>),
    ScopedBlock(Option<crate::CssBlockFragment<CssScopedRuleList>>),
}

fn bounded_report<T>(
    report: crate::CssParseReport<T>,
    syntax: impl FnOnce(T) -> BoundedParseSyntax,
) -> crate::CssParseReport<BoundedParseSyntax> {
    let (value, diagnostics) = report.into_parts();
    crate::CssParseReport::new(syntax(value), diagnostics)
}

impl BoundedParseContext {
    fn list_context(&self) -> StructuralListContext {
        match self {
            Self::Rules { top_level } => StructuralListContext::Rules {
                top_level: *top_level,
            },
            Self::OneRule | Self::StyleBlock | Self::GroupBlock => {
                StructuralListContext::Rules { top_level: false }
            }
            Self::Style
            | Self::ScopedBlock {
                has_style_ancestor: true,
                ..
            }
            | Self::ScopedBlock {
                body: ScopedBodyKind::Scope,
                ..
            }
            | Self::Scoped {
                has_style_ancestor: true,
                ..
            }
            | Self::Scoped {
                body: ScopedBodyKind::Scope,
                ..
            } => StructuralListContext::BlockContents,
            Self::Scoped {
                has_style_ancestor: false,
                body: ScopedBodyKind::OrdinaryGroup,
            }
            | Self::ScopedBlock {
                has_style_ancestor: false,
                body: ScopedBodyKind::OrdinaryGroup,
            } => StructuralListContext::Rules { top_level: false },
        }
    }

    fn is_style(&self) -> bool {
        matches!(self, Self::Style)
    }
    fn has_style_ancestor(&self) -> bool {
        matches!(
            self,
            Self::Style
                | Self::ScopedBlock {
                    has_style_ancestor: true,
                    ..
                }
                | Self::Scoped {
                    has_style_ancestor: true,
                    ..
                }
        )
    }
}

pub(super) fn parse_sheet_bounded(
    source: &str,
    source_snapshot: &CssSourceSnapshot,
    base_depth: u32,
    context: BoundedParseContext,
    style_context_captures: StyleContextCaptures,
    parser_context: crate::CssParserContext,
) -> crate::CssParseReport<CssSheet> {
    let report = parse_bounded(
        source,
        source_snapshot,
        base_depth,
        context,
        style_context_captures,
        parser_context,
    );
    let (syntax, diagnostics) = report.into_parts();
    let BoundedParseSyntax::Rules(sheet) = syntax else {
        unreachable!("rule-list replay returns its rule-list carrier")
    };
    crate::CssParseReport::new(sheet, diagnostics)
}

pub(super) fn parse_bounded(
    source: &str,
    source_snapshot: &CssSourceSnapshot,
    base_depth: u32,
    context: BoundedParseContext,
    style_context_captures: StyleContextCaptures,
    parser_context: crate::CssParserContext,
) -> crate::CssParseReport<BoundedParseSyntax> {
    let report = parse_bounded_with_captures(
        source,
        source_snapshot,
        base_depth,
        context,
        style_context_captures,
        parser_context,
    );
    let (syntax, mut diagnostics) = report.into_parts();
    // Native locations describe this admitted view, whose masks preserve bytes
    // but can change UTF-16 columns. Resolve coordinates from original bytes
    // without repeating token inference or changing responsible-token ownership.
    for diagnostic in &mut diagnostics {
        diagnostic.resolve_original_coordinates(source_snapshot.as_str());
    }
    let eof_limit = diagnostics.iter().any(|diagnostic| {
        diagnostic.action() == crate::CssRecoveryAction::StopAtNestingLimit
            && diagnostic.span().end().byte_offset().value() == source.len()
    });
    if eof_limit {
        diagnostics.retain(|diagnostic| {
            diagnostic.action() != crate::CssRecoveryAction::RetainWithImplicitClosure
        });
    }
    crate::CssParseReport::new(syntax, diagnostics)
}

fn parse_bounded_with_captures(
    source: &str,
    source_snapshot: &CssSourceSnapshot,
    base_depth: u32,
    context: BoundedParseContext,
    style_context_captures: StyleContextCaptures,
    parser_context: crate::CssParserContext,
) -> crate::CssParseReport<BoundedParseSyntax> {
    if let Some(limit) = preflight_specialized_eof_limit(source, base_depth) {
        // A terminal fragment prelude cannot be erased and replaced by a
        // different original unit. Lists alone recover by masking this unit.
        let (syntax, mut diagnostics) = match context {
            BoundedParseContext::OneRule => (BoundedParseSyntax::OneRule(None), Vec::new()),
            BoundedParseContext::StyleBlock => (BoundedParseSyntax::StyleBlock(None), Vec::new()),
            BoundedParseContext::GroupBlock => (BoundedParseSyntax::GroupBlock(None), Vec::new()),
            BoundedParseContext::ScopedBlock { .. } => {
                (BoundedParseSyntax::ScopedBlock(None), Vec::new())
            }
            _ => {
                let masked = mask_source_span(source, limit.unit_start, source.len());
                parse_bounded_with_captures(
                    &masked,
                    source_snapshot,
                    base_depth,
                    context,
                    style_context_captures,
                    parser_context,
                )
                .into_parts()
            }
        };
        diagnostics.retain(|diagnostic| {
            diagnostic.action() != crate::CssRecoveryAction::RetainWithImplicitClosure
        });
        let error = from_parse_error(
            source,
            crate::error::nesting_limit(
                source,
                limit.opening_offset,
                recovery::STRUCTURAL_NESTING_LIMIT,
                limit.enclosing_production,
            ),
        );
        if let Some(span) = crate::CssSourceSpan::new(
            crate::CssSourcePosition::from_byte_offset_in(
                source,
                if matches!(syntax, BoundedParseSyntax::Rules(_)) {
                    limit.unit_start
                } else {
                    0
                },
            ),
            crate::CssSourcePosition::from_byte_offset_in(source, source.len()),
        ) && let Some(diagnostic) = crate::CssRecoveryDiagnostic::new(
            error,
            span,
            crate::CssRecoveryAction::StopAtNestingLimit,
        ) {
            diagnostics.push(diagnostic);
        }
        return crate::CssParseReport::new(syntax, diagnostics);
    }
    let Some(preflight) = preflight_structural_nesting(
        source,
        base_depth,
        context.is_style(),
        context.has_style_ancestor(),
        context.list_context(),
        matches!(
            context,
            BoundedParseContext::GroupBlock | BoundedParseContext::ScopedBlock { .. }
        )
        .then(|| context.list_context()),
    ) else {
        let recovery = RecoveryState::at_depth_with_snapshot(
            source,
            base_depth,
            style_context_captures,
            source_snapshot.clone(),
        )
        .with_parser_context(parser_context);
        return match context {
            BoundedParseContext::Rules { top_level } => bounded_report(
                parse_sheet_inner(source, recovery, top_level),
                BoundedParseSyntax::Rules,
            ),
            BoundedParseContext::OneRule => bounded_report(
                fragments::parse_rule_inner(source, recovery),
                BoundedParseSyntax::OneRule,
            ),
            BoundedParseContext::StyleBlock => bounded_report(
                fragments::parse_style_block_inner(source, recovery),
                BoundedParseSyntax::StyleBlock,
            ),
            BoundedParseContext::GroupBlock => bounded_report(
                fragments::parse_group_block_inner(source, recovery),
                BoundedParseSyntax::GroupBlock,
            ),
            BoundedParseContext::ScopedBlock {
                has_style_ancestor,
                body,
            } => bounded_report(
                fragments::parse_scoped_block_inner(source, recovery, has_style_ancestor, body),
                BoundedParseSyntax::ScopedBlock,
            ),
            BoundedParseContext::Style => bounded_report(
                parse_style_context_inner(source, recovery),
                BoundedParseSyntax::Rules,
            ),
            BoundedParseContext::Scoped {
                has_style_ancestor,
                body,
            } => bounded_report(
                parse_scoped_context_inner(source, recovery, has_style_ancestor, body),
                BoundedParseSyntax::Rules,
            ),
        };
    };
    if matches!(&preflight.outcome, StructuralPreflightOutcome::Split) {
        for &content_start in preflight
            .style_context_starts
            .iter()
            .chain(&preflight.style_ancestry_starts)
        {
            style_context_captures.register(content_start);
        }
    }
    let masked = mask_source_span(source, preflight.unit_start, preflight.unit_end);
    // Parse at most one bounded structural chunk at a time. Same-length masks
    // retain original byte/line coordinates, and the completed child syntax is
    // spliced back into its parser-produced enclosing groups.
    let scoped_body = match &context {
        BoundedParseContext::Scoped { body, .. }
        | BoundedParseContext::ScopedBlock { body, .. } => Some(*body),
        _ => None,
    };
    let inherited_style_ancestor = context.has_style_ancestor();
    let outer = parse_bounded_with_captures(
        &masked,
        source_snapshot,
        base_depth,
        context,
        style_context_captures.clone(),
        parser_context,
    );
    let (outer_syntax, mut diagnostics) = outer.into_parts();

    // Native admission of every authored ancestor precedes publication of its
    // isolated descendants. This also preserves whole-input fragment rejection.
    if !bounded_parent_admitted(&outer_syntax, &preflight.parents) {
        return crate::CssParseReport::new(outer_syntax, diagnostics);
    }

    match preflight.outcome {
        StructuralPreflightOutcome::Split => {
            let isolated = isolate_source_span(source, preflight.unit_start, preflight.unit_end);
            let has_style_ancestor = inherited_style_ancestor
                || preflight
                    .style_ancestry_starts
                    .last()
                    .is_some_and(|&content_start| {
                        style_context_captures.contains_parsed(content_start)
                    });
            let child_context = if preflight
                .style_context_starts
                .last()
                .is_some_and(|&content_start| style_context_captures.contains_parsed(content_start))
            {
                BoundedParseContext::Style
            } else if scoped_body.is_some()
                || preflight
                    .parents
                    .iter()
                    .any(|parent| matches!(parent.kind, GroupKind::Scope))
            {
                let body = match preflight.parents.last().map(|parent| parent.kind) {
                    Some(GroupKind::Scope) => ScopedBodyKind::Scope,
                    Some(
                        GroupKind::Media
                        | GroupKind::Supports
                        | GroupKind::Layer
                        | GroupKind::Container
                        | GroupKind::When
                        | GroupKind::Else,
                    ) => ScopedBodyKind::OrdinaryGroup,
                    _ => scoped_body.unwrap_or(ScopedBodyKind::Scope),
                };
                BoundedParseContext::Scoped {
                    has_style_ancestor,
                    body,
                }
            } else {
                BoundedParseContext::Rules { top_level: false }
            };
            let child = parse_sheet_bounded(
                &isolated,
                source_snapshot,
                preflight.parent_depth,
                child_context,
                style_context_captures,
                parser_context,
            );
            let (child_sheet, mut child_diagnostics) = child.into_parts();
            diagnostics.append(&mut child_diagnostics);
            let syntax = splice_bounded_syntax(
                outer_syntax,
                &preflight.parents,
                preflight.unit_start,
                child_sheet.rules().to_vec(),
            );
            crate::CssParseReport::new(syntax, diagnostics)
        }
        StructuralPreflightOutcome::NestingLimit {
            opening_offset,
            enclosing_production,
        } => {
            diagnostics.retain(|diagnostic| {
                diagnostic.action() != crate::CssRecoveryAction::RetainWithImplicitClosure
            });
            let error = from_parse_error(
                source,
                crate::error::nesting_limit(
                    source,
                    opening_offset,
                    recovery::STRUCTURAL_NESTING_LIMIT,
                    enclosing_production,
                ),
            );
            if let Some(span) = crate::CssSourceSpan::new(
                crate::CssSourcePosition::from_byte_offset_in(source, preflight.unit_start),
                crate::CssSourcePosition::from_byte_offset_in(source, preflight.unit_end),
            ) && let Some(diagnostic) = crate::CssRecoveryDiagnostic::new(
                error,
                span,
                crate::CssRecoveryAction::StopAtNestingLimit,
            ) {
                diagnostics.push(diagnostic);
            }
            let syntax = splice_bounded_syntax(
                outer_syntax,
                &preflight.parents,
                preflight.unit_start,
                Vec::new(),
            );
            crate::CssParseReport::new(syntax, diagnostics)
        }
    }
}

// The bounded parser's internal carrier holds already-validated chunks. Scoped selector
// context remains in the selector model; splicing restores scoped rule-list wrappers.
// This conversion never reparses a scoped prelude as an ordinary stylesheet selector.
pub(super) fn scoped_rule_into_chunk_rule(rule: CssScopedRule) -> CssRule {
    match rule {
        CssScopedRule::NestedDeclarations(rule) => CssRule::NestedDeclarations(rule),
        CssScopedRule::Style(rule) => CssRule::Style(CssStyleRule::new(
            CssStyleSelectorList::new(
                rule.selectors()
                    .selectors()
                    .iter()
                    .map(|selector| match selector {
                        CssScopedStyleSelector::Selector(selector) => {
                            CssStyleSelector::Selector(selector.clone())
                        }
                        CssScopedStyleSelector::Relative(relative) => {
                            CssStyleSelector::Relative(relative.clone())
                        }
                    })
                    .collect(),
            ),
            rule.declarations().clone(),
            rule.rules().to_vec(),
            rule.position(),
        )),
        CssScopedRule::Media(rule) => CssRule::Media(CssMediaRule::new(
            rule.query().clone(),
            rule.rules()
                .rules()
                .iter()
                .cloned()
                .map(scoped_rule_into_chunk_rule)
                .collect(),
            rule.position().expect("parser-produced group position"),
        )),
        CssScopedRule::Supports(rule) => CssRule::Supports(CssSupportsRule::new(
            rule.condition().clone(),
            rule.rules()
                .rules()
                .iter()
                .cloned()
                .map(scoped_rule_into_chunk_rule)
                .collect(),
            rule.position().expect("parser-produced group position"),
        )),
        CssScopedRule::SupportsCondition(rule) => CssRule::SupportsCondition(rule),
        CssScopedRule::When(rule) => CssRule::When(CssWhenRule::new(
            rule.condition().clone(),
            rule.rules()
                .rules()
                .iter()
                .cloned()
                .map(scoped_rule_into_chunk_rule)
                .collect(),
            rule.position()
                .expect("parser-produced conditional position"),
        )),
        CssScopedRule::Else(rule) => CssRule::Else(CssElseRule::new(
            rule.condition().cloned(),
            rule.rules()
                .rules()
                .iter()
                .cloned()
                .map(scoped_rule_into_chunk_rule)
                .collect(),
            rule.position()
                .expect("parser-produced conditional position"),
        )),
        CssScopedRule::Container(rule) => CssRule::Container(CssContainerRule::new(
            rule.prelude().clone(),
            rule.rules()
                .rules()
                .iter()
                .cloned()
                .map(scoped_rule_into_chunk_rule)
                .collect(),
            rule.position().expect("parser-produced group position"),
        )),
        CssScopedRule::LayerBlock(rule) => CssRule::LayerBlock(CssLayerBlockRule::new(
            rule.name().cloned(),
            rule.rules()
                .rules()
                .iter()
                .cloned()
                .map(scoped_rule_into_chunk_rule)
                .collect(),
            rule.position().expect("parser-produced group position"),
        )),
        CssScopedRule::LayerStatement(rule) => CssRule::LayerStatement(CssLayerStatementRule::new(
            rule.names().clone(),
            rule.position(),
        )),
        CssScopedRule::CounterStyle(rule) => CssRule::CounterStyle(rule),
        CssScopedRule::FontFace(rule) => CssRule::FontFace(rule),
        CssScopedRule::Keyframes(rule) => CssRule::Keyframes(rule),
        CssScopedRule::Page(rule) => CssRule::Page(rule),
        CssScopedRule::Scope(rule) => CssRule::Scope(rule),
        CssScopedRule::CustomMedia(rule) => CssRule::CustomMedia(rule),
        CssScopedRule::FontFeatureValues(rule) => CssRule::FontFeatureValues(rule),
        CssScopedRule::FontPaletteValues(rule) => CssRule::FontPaletteValues(rule),
        CssScopedRule::ColorProfile(rule) => CssRule::ColorProfile(rule),
    }
}

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
    String::from_utf8(masked).expect("ASCII masking preserves UTF-8")
}

pub(super) fn isolate_source_span(source: &str, start: usize, end: usize) -> String {
    let mut isolated = source.as_bytes().to_vec();
    for (offset, byte) in isolated.iter_mut().enumerate() {
        if (offset < start || offset >= end) && !matches!(*byte, b'\n' | b'\r' | b'\x0c') {
            *byte = b' ';
        }
    }
    String::from_utf8(isolated).expect("ASCII masking preserves UTF-8")
}

fn bounded_parent_admitted(syntax: &BoundedParseSyntax, parents: &[StructuralParent]) -> bool {
    match syntax {
        BoundedParseSyntax::Rules(sheet) => admitted_rule_path(sheet.rules(), parents),
        BoundedParseSyntax::OneRule(Some(rule)) => {
            admitted_rule_path(std::slice::from_ref(rule), parents)
        }
        BoundedParseSyntax::StyleBlock(Some(block)) => {
            let Some((root, parents)) = parents.split_first() else {
                return true;
            };
            root.start == block.origin().span().start().byte_offset().value()
                && matches!(root.kind, GroupKind::Style)
                && admitted_rule_path(block.rules(), parents)
        }
        BoundedParseSyntax::GroupBlock(Some(block)) => {
            let Some((root, parents)) = parents.split_first() else {
                return true;
            };
            root.start == block.origin().span().start().byte_offset().value()
                && matches!(root.kind, GroupKind::FragmentBody)
                && admitted_rule_path(block.body().rules(), parents)
        }
        BoundedParseSyntax::ScopedBlock(Some(block)) => {
            let Some((root, parents)) = parents.split_first() else {
                return true;
            };
            root.start == block.origin().span().start().byte_offset().value()
                && matches!(root.kind, GroupKind::FragmentBody)
                && admitted_scoped_path(block.body().rules(), parents)
        }
        BoundedParseSyntax::OneRule(None)
        | BoundedParseSyntax::StyleBlock(None)
        | BoundedParseSyntax::GroupBlock(None)
        | BoundedParseSyntax::ScopedBlock(None) => false,
    }
}

fn admitted_rule_path(rules: &[CssRule], parents: &[StructuralParent]) -> bool {
    let Some((parent, remaining)) = parents.split_first() else {
        return true;
    };
    let Some(rule) = rules.iter().find(|rule| rule_start(rule) == parent.start) else {
        return false;
    };
    match rule {
        CssRule::Scope(scope) => admitted_scoped_path(scope.rules().rules(), remaining),
        CssRule::Style(style) => admitted_rule_path(style.rules(), remaining),
        _ => group_rules(rule).is_some_and(|rules| admitted_rule_path(rules, remaining)),
    }
}

fn admitted_scoped_path(rules: &[CssScopedRule], parents: &[StructuralParent]) -> bool {
    let Some((parent, remaining)) = parents.split_first() else {
        return true;
    };
    let Some(rule) = rules
        .iter()
        .find(|rule| scoped_rule_start(rule) == parent.start)
    else {
        return false;
    };
    match rule {
        CssScopedRule::Style(style) => admitted_rule_path(style.rules(), remaining),
        _ => scoped_group_rules(rule).is_some_and(|rules| admitted_scoped_path(rules, remaining)),
    }
}

fn splice_bounded_syntax(
    syntax: BoundedParseSyntax,
    parents: &[StructuralParent],
    child_start: usize,
    child_rules: Vec<CssRule>,
) -> BoundedParseSyntax {
    match syntax {
        BoundedParseSyntax::Rules(sheet) => BoundedParseSyntax::Rules(splice_preflight_rules(
            &sheet,
            parents,
            child_start,
            child_rules,
        )),
        BoundedParseSyntax::OneRule(Some(rule)) => {
            let [rule]: [CssRule; 1] = splice_rule_list(
                std::slice::from_ref(&rule),
                parents,
                child_start,
                child_rules,
            )
            .try_into()
            .expect("replay preserves its admitted single outer rule");
            BoundedParseSyntax::OneRule(Some(rule))
        }
        BoundedParseSyntax::StyleBlock(Some(block)) => {
            let (_, parents) = parents
                .split_first()
                .expect("a style-block replay retains its genuine outer brace");
            let (declarations, rules) = splice_style_body(
                block.declarations(),
                block.rules(),
                parents,
                child_start,
                child_rules,
            );
            BoundedParseSyntax::StyleBlock(Some(CssStyleBlock::new(
                StyleContents {
                    declarations,
                    rules,
                },
                block.origin().clone(),
            )))
        }
        BoundedParseSyntax::GroupBlock(Some(block)) => {
            let (_, parents) = parents
                .split_first()
                .expect("group replay retains its genuine root brace");
            let rules = splice_rule_list(block.body().rules(), parents, child_start, child_rules);
            BoundedParseSyntax::GroupBlock(Some(crate::CssBlockFragment::from_parsed(
                crate::CssRuleList::from_parsed(rules),
                block.origin().clone(),
            )))
        }
        BoundedParseSyntax::ScopedBlock(Some(block)) => {
            let (_, parents) = parents
                .split_first()
                .expect("scoped replay retains its genuine root brace");
            let rules =
                splice_scoped_rule_list(block.body().rules(), parents, child_start, child_rules);
            BoundedParseSyntax::ScopedBlock(Some(crate::CssBlockFragment::from_parsed(
                CssScopedRuleList::from_rules(rules),
                block.origin().clone(),
            )))
        }
        BoundedParseSyntax::OneRule(None)
        | BoundedParseSyntax::StyleBlock(None)
        | BoundedParseSyntax::GroupBlock(None)
        | BoundedParseSyntax::ScopedBlock(None) => {
            unreachable!("rejected fragment roots cannot receive replayed descendants")
        }
    }
}

fn splice_preflight_rules(
    sheet: &CssSheet,
    parents: &[StructuralParent],
    child_start: usize,
    child_rules: Vec<CssRule>,
) -> CssSheet {
    let rules = splice_rule_list(sheet.rules(), parents, child_start, child_rules);
    let mut rebuilt = CssSheet::new();
    for rule in rules {
        rebuilt.push_rule(rule);
    }
    rebuilt
}

fn splice_rule_list(
    rules: &[CssRule],
    parents: &[StructuralParent],
    child_start: usize,
    child_rules: Vec<CssRule>,
) -> Vec<CssRule> {
    if parents.is_empty() {
        let mut combined = rules
            .iter()
            .cloned()
            .flat_map(|rule| split_style_declaration_run(rule, child_start))
            .collect::<Vec<_>>();
        let insertion = combined
            .iter()
            .position(|rule| rule_start(rule) > child_start)
            .unwrap_or(combined.len());
        combined.splice(insertion..insertion, child_rules);
        return combined;
    }
    let parent = &parents[0];
    rules
        .iter()
        .cloned()
        .map(|rule| {
            if rule_start(&rule) != parent.start {
                return rule;
            }
            if matches!(parent.kind, GroupKind::Scope)
                && let CssRule::Scope(scope) = rule
            {
                let rebuilt = splice_scoped_rule_list(
                    scope.rules().rules(),
                    &parents[1..],
                    child_start,
                    child_rules.clone(),
                );
                return CssRule::Scope(CssScopeRule::new(
                    scope.root().cloned(),
                    scope.limit().cloned(),
                    CssScopedRuleList::from_rules(rebuilt),
                    scope.position().expect("parser-produced scope position"),
                ));
            }
            if let CssRule::Style(style) = rule {
                let (declarations, rules) = splice_style_body(
                    style.declarations(),
                    style.rules(),
                    &parents[1..],
                    child_start,
                    child_rules.clone(),
                );
                return CssRule::Style(CssStyleRule::new(
                    style.selectors().clone(),
                    declarations,
                    rules,
                    style.position(),
                ));
            }
            let nested = group_rules(&rule).unwrap_or_default();
            let rebuilt = splice_rule_list(nested, &parents[1..], child_start, child_rules.clone());
            rebuild_group_rule(rule, rebuilt)
        })
        .collect()
}

fn splice_style_body(
    leading: &CssDeclarationList,
    rules: &[CssRule],
    parents: &[StructuralParent],
    child_start: usize,
    child_rules: Vec<CssRule>,
) -> (CssDeclarationList, Vec<CssRule>) {
    let mut declarations = leading.as_slice().to_vec();
    let mut nested = rules.to_vec();
    if parents.is_empty() {
        let split = declarations.partition_point(|declaration| {
            declaration
                .position()
                .expect("parsed declarations have source positions")
                .byte_offset()
                .value()
                < child_start
        });
        // A rejected complete chunk still transfers a nonempty preceding run.
        // With no preceding list and no retained child, the leading slot stays
        // available. Structural split candidates exclude custom declarations.
        let trailing = if split > 0 || !child_rules.is_empty() {
            declarations.split_off(split)
        } else {
            Vec::new()
        };
        if !trailing.is_empty() {
            nested.insert(
                0,
                CssRule::NestedDeclarations(CssNestedDeclarationsRule::new(
                    CssDeclarationList::new(trailing),
                )),
            );
        }
    }
    let rebuilt = splice_rule_list(&nested, parents, child_start, child_rules);
    (CssDeclarationList::new(declarations), rebuilt)
}

fn splice_scoped_rule_list(
    rules: &[CssScopedRule],
    parents: &[StructuralParent],
    child_start: usize,
    child_rules: Vec<CssRule>,
) -> Vec<CssScopedRule> {
    if parents.is_empty() {
        let mut combined: Vec<_> = rules
            .iter()
            .cloned()
            .flat_map(|rule| {
                if let CssScopedRule::NestedDeclarations(run) = rule {
                    split_style_declaration_run(CssRule::NestedDeclarations(run), child_start)
                        .into_iter()
                        .filter_map(into_scoped_rule)
                        .collect()
                } else {
                    vec![rule]
                }
            })
            .collect();
        let insertion = combined
            .iter()
            .position(|rule| scoped_rule_start(rule) > child_start)
            .unwrap_or(combined.len());
        combined.splice(
            insertion..insertion,
            child_rules.into_iter().filter_map(into_scoped_rule),
        );
        return combined;
    }
    let parent = &parents[0];
    rules
        .iter()
        .cloned()
        .map(|rule| {
            if scoped_rule_start(&rule) != parent.start {
                return rule;
            }
            if let CssScopedRule::Style(style) = rule {
                let (declarations, nested) = splice_style_body(
                    style.declarations(),
                    style.rules(),
                    &parents[1..],
                    child_start,
                    child_rules.clone(),
                );
                return CssScopedRule::Style(CssScopedStyleRule::new(
                    style.selectors().clone(),
                    declarations,
                    nested,
                    style.position(),
                ));
            }
            let nested = scoped_group_rules(&rule).unwrap_or_default();
            let rebuilt =
                splice_scoped_rule_list(nested, &parents[1..], child_start, child_rules.clone());
            rebuild_scoped_group_rule(rule, rebuilt)
        })
        .collect()
}

fn split_style_declaration_run(rule: CssRule, child_start: usize) -> Vec<CssRule> {
    let CssRule::NestedDeclarations(run) = rule else {
        return vec![rule];
    };
    let declarations = run.declarations().as_slice();
    let split = declarations.partition_point(|declaration| {
        declaration
            .position()
            .expect("parsed declarations have source positions")
            .byte_offset()
            .value()
            < child_start
    });
    if split == 0 || split == declarations.len() {
        return vec![CssRule::NestedDeclarations(run)];
    }
    vec![
        CssRule::NestedDeclarations(CssNestedDeclarationsRule::new(CssDeclarationList::new(
            declarations[..split].to_vec(),
        ))),
        CssRule::NestedDeclarations(CssNestedDeclarationsRule::new(CssDeclarationList::new(
            declarations[split..].to_vec(),
        ))),
    ]
}

fn scoped_group_rules(rule: &CssScopedRule) -> Option<&[CssScopedRule]> {
    match rule {
        CssScopedRule::Media(rule) => Some(rule.rules().rules()),
        CssScopedRule::Supports(rule) => Some(rule.rules().rules()),
        CssScopedRule::Container(rule) => Some(rule.rules().rules()),
        CssScopedRule::LayerBlock(rule) => Some(rule.rules().rules()),
        CssScopedRule::Scope(rule) => Some(rule.rules().rules()),
        CssScopedRule::When(rule) => Some(rule.rules().rules()),
        CssScopedRule::Else(rule) => Some(rule.rules().rules()),
        _ => None,
    }
}

fn rebuild_scoped_group_rule(rule: CssScopedRule, rules: Vec<CssScopedRule>) -> CssScopedRule {
    let rules = CssScopedRuleList::from_rules(rules);
    match rule {
        CssScopedRule::Media(rule) => CssScopedRule::Media(CssScopedMediaRule::new(
            rule.query().clone(),
            rules,
            rule.position().expect("parser-produced group position"),
        )),
        CssScopedRule::Supports(rule) => CssScopedRule::Supports(CssScopedSupportsRule::new(
            rule.condition().clone(),
            rules,
            rule.position().expect("parser-produced group position"),
        )),
        CssScopedRule::When(rule) => CssScopedRule::When(CssScopedWhenRule::new(
            rule.condition().clone(),
            rules,
            rule.position()
                .expect("parser-produced conditional position"),
        )),
        CssScopedRule::Else(rule) => CssScopedRule::Else(CssScopedElseRule::new(
            rule.condition().cloned(),
            rules,
            rule.position()
                .expect("parser-produced conditional position"),
        )),
        CssScopedRule::Container(rule) => CssScopedRule::Container(CssScopedContainerRule::new(
            rule.prelude().clone(),
            rules,
            rule.position().expect("parser-produced group position"),
        )),
        CssScopedRule::LayerBlock(rule) => CssScopedRule::LayerBlock(CssScopedLayerBlockRule::new(
            rule.name().cloned(),
            rules,
            rule.position().expect("parser-produced group position"),
        )),
        CssScopedRule::Scope(rule) => CssScopedRule::Scope(CssScopeRule::new(
            rule.root().cloned(),
            rule.limit().cloned(),
            rules,
            rule.position().expect("parser-produced group position"),
        )),
        _ => rule,
    }
}

fn into_scoped_rule(rule: CssRule) -> Option<CssScopedRule> {
    match rule {
        CssRule::NestedDeclarations(rule) => Some(CssScopedRule::NestedDeclarations(rule)),
        CssRule::Style(rule) => {
            let selectors = CssScopedStyleSelectorList::try_new(
                rule.selectors()
                    .selectors()
                    .iter()
                    .map(|selector| match selector {
                        CssStyleSelector::Selector(selector) => {
                            CssScopedStyleSelector::Selector(selector.clone())
                        }
                        CssStyleSelector::Relative(relative) => {
                            CssScopedStyleSelector::Relative(relative.clone())
                        }
                    })
                    .collect(),
            )?;
            Some(CssScopedRule::Style(CssScopedStyleRule::new(
                selectors,
                rule.declarations().clone(),
                rule.rules().to_vec(),
                rule.position(),
            )))
        }
        CssRule::LayerStatement(rule) => Some(CssScopedRule::LayerStatement(
            CssScopedLayerStatementRule::new(rule.names().clone(), rule.position()),
        )),
        CssRule::LayerBlock(rule) => Some(CssScopedRule::LayerBlock(CssScopedLayerBlockRule::new(
            rule.name().cloned(),
            CssScopedRuleList::from_rules(
                rule.rules()
                    .iter()
                    .cloned()
                    .filter_map(into_scoped_rule)
                    .collect(),
            ),
            rule.position().expect("parser-produced group position"),
        ))),
        CssRule::Media(rule) => Some(CssScopedRule::Media(CssScopedMediaRule::new(
            rule.query().clone(),
            CssScopedRuleList::from_rules(
                rule.rules()
                    .iter()
                    .cloned()
                    .filter_map(into_scoped_rule)
                    .collect(),
            ),
            rule.position().expect("parser-produced group position"),
        ))),
        CssRule::Supports(rule) => Some(CssScopedRule::Supports(CssScopedSupportsRule::new(
            rule.condition().clone(),
            CssScopedRuleList::from_rules(
                rule.rules()
                    .iter()
                    .cloned()
                    .filter_map(into_scoped_rule)
                    .collect(),
            ),
            rule.position().expect("parser-produced group position"),
        ))),
        CssRule::SupportsCondition(rule) => Some(CssScopedRule::SupportsCondition(rule)),
        CssRule::When(rule) => Some(CssScopedRule::When(CssScopedWhenRule::new(
            rule.condition().clone(),
            CssScopedRuleList::from_rules(
                rule.rules()
                    .iter()
                    .cloned()
                    .filter_map(into_scoped_rule)
                    .collect(),
            ),
            rule.position()
                .expect("parser-produced conditional position"),
        ))),
        CssRule::Else(rule) => Some(CssScopedRule::Else(CssScopedElseRule::new(
            rule.condition().cloned(),
            CssScopedRuleList::from_rules(
                rule.rules()
                    .iter()
                    .cloned()
                    .filter_map(into_scoped_rule)
                    .collect(),
            ),
            rule.position()
                .expect("parser-produced conditional position"),
        ))),
        CssRule::Container(rule) => Some(CssScopedRule::Container(CssScopedContainerRule::new(
            rule.prelude().clone(),
            CssScopedRuleList::from_rules(
                rule.rules()
                    .iter()
                    .cloned()
                    .filter_map(into_scoped_rule)
                    .collect(),
            ),
            rule.position().expect("parser-produced group position"),
        ))),
        CssRule::CounterStyle(rule) => Some(CssScopedRule::CounterStyle(rule)),
        CssRule::FontFace(rule) => Some(CssScopedRule::FontFace(rule)),
        CssRule::Keyframes(rule) => Some(CssScopedRule::Keyframes(rule)),
        CssRule::Page(rule) => Some(CssScopedRule::Page(rule)),
        CssRule::Scope(rule) => Some(CssScopedRule::Scope(rule)),
        CssRule::CustomMedia(rule) => Some(CssScopedRule::CustomMedia(rule)),
        CssRule::FontFeatureValues(rule) => Some(CssScopedRule::FontFeatureValues(rule)),
        CssRule::FontPaletteValues(rule) => Some(CssScopedRule::FontPaletteValues(rule)),
        CssRule::ColorProfile(rule) => Some(CssScopedRule::ColorProfile(rule)),
        CssRule::Import(_) | CssRule::Namespace(_) => None,
    }
}

pub(super) fn scoped_rule_start(rule: &CssScopedRule) -> usize {
    match rule {
        CssScopedRule::CustomMedia(rule) => {
            return rule
                .position()
                .expect("parsed custom-media rule")
                .byte_offset()
                .value();
        }
        CssScopedRule::FontFeatureValues(rule) => {
            return rule
                .position()
                .expect("parser-owned font rule has a source position")
                .byte_offset()
                .value();
        }
        CssScopedRule::FontPaletteValues(rule) => {
            return rule
                .position()
                .expect("parser-owned palette rule has a source position")
                .byte_offset()
                .value();
        }
        CssScopedRule::ColorProfile(rule) => {
            return rule
                .position()
                .expect("parser-owned profile rule has a source position")
                .byte_offset()
                .value();
        }
        CssScopedRule::NestedDeclarations(rule) => rule.position(),
        CssScopedRule::CounterStyle(rule) => rule.position(),
        CssScopedRule::FontFace(rule) => rule.position().expect("parser-produced rule position"),
        CssScopedRule::Keyframes(rule) => rule.position(),
        CssScopedRule::Page(rule) => rule.position(),
        CssScopedRule::Style(rule) => rule.position(),
        CssScopedRule::Media(rule) => rule.position().expect("parser-produced rule position"),
        CssScopedRule::Supports(rule) => rule.position().expect("parser-produced rule position"),
        CssScopedRule::SupportsCondition(rule) => {
            return rule
                .position()
                .expect("parsed named supports rule")
                .byte_offset()
                .value();
        }
        CssScopedRule::Container(rule) => rule.position().expect("parser-produced rule position"),
        CssScopedRule::When(rule) => rule
            .position()
            .expect("parser-produced conditional position"),
        CssScopedRule::Else(rule) => rule
            .position()
            .expect("parser-produced conditional position"),
        CssScopedRule::LayerStatement(rule) => rule.position(),
        CssScopedRule::LayerBlock(rule) => rule.position().expect("parser-produced rule position"),
        CssScopedRule::Scope(rule) => rule.position().expect("parser-produced rule position"),
    }
    .byte_offset()
    .value()
}

fn group_rules(rule: &CssRule) -> Option<&[CssRule]> {
    match rule {
        CssRule::LayerBlock(rule) => Some(rule.rules()),
        CssRule::Media(rule) => Some(rule.rules()),
        CssRule::Supports(rule) => Some(rule.rules()),
        CssRule::Container(rule) => Some(rule.rules()),
        CssRule::When(rule) => Some(rule.rules()),
        CssRule::Else(rule) => Some(rule.rules()),
        _ => None,
    }
}

fn rebuild_group_rule(rule: CssRule, rules: Vec<CssRule>) -> CssRule {
    match rule {
        CssRule::LayerBlock(rule) => CssRule::LayerBlock(CssLayerBlockRule::new(
            rule.name().cloned(),
            rules,
            rule.position().expect("parser-produced group position"),
        )),
        CssRule::Media(rule) => CssRule::Media(CssMediaRule::new(
            rule.query().clone(),
            rules,
            rule.position().expect("parser-produced group position"),
        )),
        CssRule::Supports(rule) => CssRule::Supports(CssSupportsRule::new(
            rule.condition().clone(),
            rules,
            rule.position().expect("parser-produced group position"),
        )),
        CssRule::When(rule) => CssRule::When(CssWhenRule::new(
            rule.condition().clone(),
            rules,
            rule.position()
                .expect("parser-produced conditional position"),
        )),
        CssRule::Else(rule) => CssRule::Else(CssElseRule::new(
            rule.condition().cloned(),
            rules,
            rule.position()
                .expect("parser-produced conditional position"),
        )),
        CssRule::Container(rule) => CssRule::Container(CssContainerRule::new(
            rule.prelude().clone(),
            rules,
            rule.position().expect("parser-produced group position"),
        )),
        _ => rule,
    }
}

pub(super) fn rule_start(rule: &CssRule) -> usize {
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

#[cfg(test)]
mod splice_tests {
    use super::*;

    #[test]
    fn scoped_splice_orders_empty_styles_by_parser_produced_rule_positions() {
        let source = "@scope{.before-empty{}@layer{}.after-empty{}}";
        let report = parse_sheet(source);
        assert!(report.is_clean(), "{:?}", report.diagnostics());
        let [CssRule::Scope(scope)] = report.syntax().rules() else {
            panic!("expected one scope rule");
        };
        let [
            CssScopedRule::Style(before),
            CssScopedRule::LayerBlock(recovered),
            CssScopedRule::Style(after),
        ] = scope.rules().rules()
        else {
            panic!("expected scoped splice fixture in authored order");
        };
        assert!(before.declarations().is_empty());
        assert!(after.declarations().is_empty());
        assert!(
            before.position().byte_offset()
                < recovered
                    .position()
                    .expect("parsed layer position")
                    .byte_offset()
                && recovered
                    .position()
                    .expect("parsed layer position")
                    .byte_offset()
                    < after.position().byte_offset()
        );

        let spliced = splice_scoped_rule_list(
            &[
                CssScopedRule::Style(before.clone()),
                CssScopedRule::Style(after.clone()),
            ],
            &[],
            recovered
                .position()
                .expect("parsed layer position")
                .byte_offset()
                .value(),
            vec![CssRule::LayerBlock(CssLayerBlockRule::new(
                recovered.name().cloned(),
                Vec::new(),
                recovered.position().expect("parsed layer position"),
            ))],
        );
        assert_eq!(spliced, scope.rules().rules());

        let stable_tie = splice_scoped_rule_list(
            &[
                CssScopedRule::Style(before.clone()),
                CssScopedRule::Style(after.clone()),
            ],
            &[],
            before.position().byte_offset().value(),
            vec![CssRule::LayerBlock(CssLayerBlockRule::new(
                recovered.name().cloned(),
                Vec::new(),
                recovered.position().expect("parsed layer position"),
            ))],
        );
        assert_eq!(
            stable_tie,
            [
                CssScopedRule::Style(before.clone()),
                CssScopedRule::LayerBlock(recovered.clone()),
                CssScopedRule::Style(after.clone()),
            ]
        );
    }
}
