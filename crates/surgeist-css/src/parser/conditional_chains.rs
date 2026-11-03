//! Contextual admission after every bounded sibling chunk has been reconstructed.
//! Original token gaps prove parsed adjacency; reusable checked fragments use the
//! separate complete-list assembly owner and never carry these source constraints.
use crate::CssParseReport;
use crate::diagnostic::{CssRecoveryAction, CssRecoveryDiagnostic};
use crate::rule::{CssRule, CssSheet, CssStyleBlock};
use crate::source::{CssSourceSnapshot, CssSourceSpan};
use std::ops::Range;
use surgeist_css_parser::Token;
use surgeist_css_parser::lexer::{Lexer, Options};

pub(super) fn sheet(
    snapshot: &CssSourceSnapshot,
    report: CssParseReport<CssSheet>,
) -> CssParseReport<CssSheet> {
    let (mut sheet, mut diagnostics) = report.into_parts();
    completed_list(snapshot, sheet.rules_mut(), &mut diagnostics);
    CssParseReport::new(sheet, diagnostics)
}
pub(crate) fn rule(
    snapshot: &CssSourceSnapshot,
    mut rule: CssRule,
    mut diagnostics: Vec<CssRecoveryDiagnostic>,
) -> CssParseReport<Option<CssRule>> {
    if matches!(rule, CssRule::Else(_)) {
        let source = snapshot.as_str();
        let position = snapshot.position_or_walk(crate::parser::reconstruction::rule_start(&rule));
        let span = CssSourceSpan::new(
            snapshot.position_or_walk(0),
            snapshot.position_or_walk(source.len()),
        )
        .expect("whole input span");
        let diagnostic = CssRecoveryDiagnostic::new(
            crate::diagnostic::error::invalid_free_else(source, position),
            span,
            CssRecoveryAction::RejectInput,
        )
        .expect("else lies in input");
        return CssParseReport::new(None, vec![diagnostic]);
    }
    if let Some(children) = rule.parser_children_mut() {
        completed_list(snapshot, children, &mut diagnostics);
    }
    CssParseReport::new(Some(rule), diagnostics)
}
pub(super) fn style_block(
    snapshot: &CssSourceSnapshot,
    mut block: CssStyleBlock,
    mut diagnostics: Vec<CssRecoveryDiagnostic>,
) -> CssParseReport<Option<CssStyleBlock>> {
    completed_list(snapshot, block.rules_mut(), &mut diagnostics);
    CssParseReport::new(Some(block), diagnostics)
}
pub(super) fn completed_list(
    snapshot: &CssSourceSnapshot,
    list: &mut Vec<CssRule>,
    diagnostics: &mut Vec<CssRecoveryDiagnostic>,
) {
    admit(snapshot, list, diagnostics);
}

fn admit(
    snapshot: &CssSourceSnapshot,
    list: &mut Vec<CssRule>,
    diagnostics: &mut Vec<CssRecoveryDiagnostic>,
) {
    let mut work = vec![list];
    let mut rejected = Vec::new();
    let mut placements = Vec::new();
    while let Some(rules) = work.pop() {
        let mut previous_end = None;
        rules.retain_mut(|rule| {
            let conditional = matches!(
                rule,
                CssRule::Media(_)
                    | CssRule::Supports(_)
                    | CssRule::Container(_)
                    | CssRule::When(_)
                    | CssRule::Else(_)
            );
            let start = conditional.then(|| crate::parser::reconstruction::rule_start(rule));
            check(
                snapshot,
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
    snapshot: &CssSourceSnapshot,
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
    let source = snapshot.as_str();
    let end = balanced_unit_end(source, start);
    if is_else && !previous_end.is_some_and(|previous| trivia_gap(source, previous, start)) {
        *previous_end = None;
        rejected.push(start..end);
        let position = snapshot.position_or_walk(start);
        let span = CssSourceSpan::new(position, snapshot.position_or_walk(end))
            .expect("balanced original rule span");
        placements.push(
            CssRecoveryDiagnostic::new(
                crate::diagnostic::error::invalid_free_else(source, position),
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
fn trivia_gap(source: &str, offset: usize, end: usize) -> bool {
    if offset > end {
        return false;
    }
    let Some(rest) = source.get(offset..) else {
        return false;
    };
    let options = Options::default();
    let mut lexer = Lexer::new(rest, &options);
    let mut at = offset;
    while at < end {
        let Some(lexeme) = lexer.next_lexeme() else {
            return false;
        };
        let token_end = offset + lexeme.range().end.value();
        if token_end > end || !matches!(lexeme.token(), Token::Whitespace | Token::Comment) {
            return false;
        }
        at = token_end;
    }
    true
}
// Same delimiter semantics as structural recovery, without an admission budget:
// descendants may already have been discarded while the complete outer unit lives.
fn balanced_unit_end(source: &str, offset: usize) -> usize {
    let Some(rest) = source.get(offset..) else {
        return source.len();
    };
    let options = Options::default();
    let mut lexer = Lexer::new(rest, &options);
    let mut blocks = Vec::new();
    while let Some(lexeme) = lexer.next_lexeme() {
        let end = offset + lexeme.range().end.value();
        let token = lexeme.token();
        let close = match token {
            Token::CloseParen => Some(0),
            Token::CloseSquare => Some(1),
            Token::CloseCurly => Some(2),
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
            Token::Function(_) | Token::OpenParen => Some(0),
            Token::OpenSquare => Some(1),
            Token::OpenCurly => Some(2),
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
    use crate::source::CssSourcePosition;
    #[test]
    fn discarded_else_suppresses_owned_eof_closures_and_keeps_lexical_comment() {
        let source = "@else { .lost { color: rgb(1 2 3); /*";
        let report = crate::CssParser::default().parse_sheet(source);
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
        let report =
            crate::CssParser::default().parse_sheet("@layer kept { @else { .lost { color: red");
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
            crate::diagnostic::error::implicit_eof(eof),
            CssSourceSpan::new(eof, eof).unwrap(),
            CssRecoveryAction::RetainWithImplicitClosure,
        )
        .unwrap();
        assert_eq!(diagnostic, diagnostic.clone().with_opening_owner(0));
    }
}
