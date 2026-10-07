//! Contextual admission after every bounded sibling chunk has been reconstructed.
//! Original token gaps prove parsed adjacency; reusable checked fragments use the
//! separate complete-list assembly owner and never carry these source constraints.
use super::recovery::next_source_token;
use crate::syntax::CssParserRuleChildrenMut;
use crate::*;
use cssparser::Token;
use std::ops::Range;

pub(super) fn sheet(source: &str, report: CssParseReport<CssSheet>) -> CssParseReport<CssSheet> {
    let (mut sheet, mut diagnostics) = report.into_parts();
    admit(
        source,
        CssParserRuleChildrenMut::Ordinary(sheet.rules_mut()),
        &mut diagnostics,
    );
    CssParseReport::new(sheet, diagnostics)
}
pub(super) fn rule(
    source: &str,
    mut rule: CssRule,
    mut diagnostics: Vec<CssRecoveryDiagnostic>,
) -> CssParseReport<Option<CssRule>> {
    if matches!(rule, CssRule::Else(_)) {
        let position = CssSourcePosition::from_byte_offset_in(source, super::rule_start(&rule));
        let span = CssSourceSpan::new(
            CssSourcePosition::from_byte_offset_in(source, 0),
            CssSourcePosition::from_byte_offset_in(source, source.len()),
        )
        .expect("whole input span");
        let diagnostic = CssRecoveryDiagnostic::new(
            crate::error::invalid_free_else(source, position),
            span,
            CssRecoveryAction::RejectInput,
        )
        .expect("else lies in input");
        return CssParseReport::new(None, vec![diagnostic]);
    }
    if let Some(children) = rule.parser_children_mut() {
        admit(source, children, &mut diagnostics);
    }
    CssParseReport::new(Some(rule), diagnostics)
}
pub(super) fn style_block(
    source: &str,
    mut block: CssStyleBlock,
    mut diagnostics: Vec<CssRecoveryDiagnostic>,
) -> CssParseReport<Option<CssStyleBlock>> {
    admit(
        source,
        CssParserRuleChildrenMut::Ordinary(block.rules_mut()),
        &mut diagnostics,
    );
    CssParseReport::new(Some(block), diagnostics)
}
fn admit(
    source: &str,
    list: CssParserRuleChildrenMut<'_>,
    diagnostics: &mut Vec<CssRecoveryDiagnostic>,
) {
    let mut work = vec![list];
    let mut rejected = Vec::new();
    let mut placements = Vec::new();
    while let Some(list) = work.pop() {
        let mut previous_end = None;
        match list {
            CssParserRuleChildrenMut::Ordinary(rules) => {
                rules.retain_mut(|rule| {
                    let conditional = matches!(
                        rule,
                        CssRule::Media(_)
                            | CssRule::Supports(_)
                            | CssRule::Container(_)
                            | CssRule::When(_)
                            | CssRule::Else(_)
                    );
                    let start = conditional.then(|| super::rule_start(rule));
                    check(
                        source,
                        matches!(rule, CssRule::Else(_)),
                        conditional,
                        start,
                        &mut previous_end,
                        &mut rejected,
                        &mut placements,
                    )
                });
                for rule in rules {
                    if let Some(children) = rule.parser_children_mut() {
                        work.push(children);
                    }
                }
            }
            CssParserRuleChildrenMut::Scoped(rules) => {
                rules.retain_mut(|rule| {
                    let conditional = matches!(
                        rule,
                        CssScopedRule::Media(_)
                            | CssScopedRule::Supports(_)
                            | CssScopedRule::Container(_)
                            | CssScopedRule::When(_)
                            | CssScopedRule::Else(_)
                    );
                    let start = conditional.then(|| super::scoped_rule_start(rule));
                    check(
                        source,
                        matches!(rule, CssScopedRule::Else(_)),
                        conditional,
                        start,
                        &mut previous_end,
                        &mut rejected,
                        &mut placements,
                    )
                });
                for rule in rules {
                    if let Some(children) = rule.parser_children_mut() {
                        work.push(children);
                    }
                }
            }
        }
    }
    diagnostics.retain(|diagnostic| {
        !rejected.iter().any(|range: &Range<usize>| {
            if let Some(owner) = diagnostic.opening_owner() {
                return range.contains(&owner);
            }
            range.start <= diagnostic.span().start().byte_offset().value()
                && diagnostic.span().end().byte_offset().value() <= range.end
        })
    });
    diagnostics.extend(placements);
}
fn check(
    source: &str,
    is_else: bool,
    conditional: bool,
    start: Option<usize>,
    previous_end: &mut Option<usize>,
    rejected: &mut Vec<Range<usize>>,
    placements: &mut Vec<CssRecoveryDiagnostic>,
) -> bool {
    if !conditional {
        *previous_end = None;
        return true;
    }
    let start = start.expect("conditional parsed start");
    let end = balanced_unit_end(source, start);
    if is_else && !previous_end.is_some_and(|previous| trivia_gap(source, previous, start)) {
        *previous_end = None;
        rejected.push(start..end);
        let position = CssSourcePosition::from_byte_offset_in(source, start);
        let span = CssSourceSpan::new(
            position,
            CssSourcePosition::from_byte_offset_in(source, end),
        )
        .expect("balanced original rule span");
        placements.push(
            CssRecoveryDiagnostic::new(
                crate::error::invalid_free_else(source, position),
                span,
                CssRecoveryAction::DropAtRule,
            )
            .expect("else start is in its unit"),
        );
        return false;
    }
    *previous_end = Some(end);
    true
}
fn trivia_gap(source: &str, mut offset: usize, end: usize) -> bool {
    if offset > end {
        return false;
    }
    while offset < end {
        let Some((_, token_end, token)) = next_source_token(source, offset) else {
            return false;
        };
        if token_end > end || !matches!(token, Token::WhiteSpace(_) | Token::Comment(_)) {
            return false;
        }
        offset = token_end;
    }
    true
}
// Same delimiter semantics as structural recovery, without an admission budget:
// descendants may already have been discarded while the complete outer unit lives.
fn balanced_unit_end(source: &str, mut offset: usize) -> usize {
    let mut blocks = Vec::new();
    while let Some((_, end, token)) = next_source_token(source, offset) {
        offset = end;
        let close = match token {
            Token::CloseParenthesis => Some(0),
            Token::CloseSquareBracket => Some(1),
            Token::CloseCurlyBracket => Some(2),
            _ => None,
        };
        if let Some(close) = close {
            if blocks.last() == Some(&close) {
                blocks.pop();
                if blocks.is_empty() && close == 2 {
                    return end;
                }
            }
            continue;
        }
        if blocks.is_empty() && matches!(token, Token::Semicolon) {
            return end;
        }
        let open = match token {
            Token::Function(_) | Token::ParenthesisBlock => Some(0),
            Token::SquareBracketBlock => Some(1),
            Token::CurlyBracketBlock => Some(2),
            _ => None,
        };
        if let Some(open) = open {
            blocks.push(open);
        }
    }
    source.len()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn discarded_else_suppresses_owned_eof_closures_and_keeps_lexical_comment() {
        let source = "@else { .lost { color: rgb(1 2 3); /*";
        let report = parse_sheet(source);
        assert!(report.syntax().rules().is_empty());
        assert_eq!(
            report
                .diagnostics()
                .iter()
                .filter(|d| d.action() == CssRecoveryAction::DropAtRule)
                .count(),
            1
        );
        assert!(
            !report
                .diagnostics()
                .iter()
                .any(|d| d.action() == CssRecoveryAction::RetainWithImplicitClosure)
        );
        assert!(
            report
                .diagnostics()
                .iter()
                .any(|d| d.action() == CssRecoveryAction::IgnoreUnterminatedComment)
        );
    }
    #[test]
    fn discarded_nested_else_does_not_suppress_ancestor_eof_closure() {
        let report = parse_sheet("@layer kept { @else { .lost { color: red");
        assert!(
            matches!(report.syntax().rules(), [CssRule::LayerBlock(rule)] if rule.rules().is_empty())
        );
        assert_eq!(
            report
                .diagnostics()
                .iter()
                .filter(|d| d.action() == CssRecoveryAction::RetainWithImplicitClosure)
                .count(),
            1
        );
    }
    #[test]
    fn private_opening_provenance_is_transparent_to_public_equality() {
        let source = "{";
        let eof = CssSourcePosition::from_byte_offset_in(source, 1);
        let diagnostic = CssRecoveryDiagnostic::new(
            crate::error::implicit_eof(source),
            CssSourceSpan::new(eof, eof).unwrap(),
            CssRecoveryAction::RetainWithImplicitClosure,
        )
        .unwrap();
        assert_eq!(diagnostic, diagnostic.clone().with_opening_owner(0));
    }
}
