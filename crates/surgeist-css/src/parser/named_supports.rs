//! Generic Syntax block partitioning for authored supports-condition tests.

use super::*;
use crate::error::invalid_component_value;
use crate::named_supports::*;
use crate::syntax_consumption::{
    self as syntax, CursorItem, GroupEnd, ListId, NodeId, SyntaxCursor, SyntaxDocument, SyntaxNode,
    SyntaxRange, TokenKind,
};
use crate::{
    CssBlockKind, CssComponentValue, CssComponentValueError, CssComponentValueErrorKind,
    CssComponentValueLimits, CssRecoveryAction, CssRecoveryDiagnostic, CssValueOrigin,
};
use cssparser::{ParseError, Parser, ParserState};

enum PartitionIssue {
    Grammar(CssValueOrigin, CssRecoveryAction),
    Lexical(CssComponentValueError, CssRecoveryAction),
}

impl PartitionIssue {
    fn origin(&self) -> &CssValueOrigin {
        match self {
            Self::Grammar(origin, _) => origin,
            Self::Lexical(error, _) => error.origin(),
        }
    }

    fn action(&self) -> CssRecoveryAction {
        match self {
            Self::Grammar(_, action) | Self::Lexical(_, action) => *action,
        }
    }
}

pub(super) struct NamedSupportsPrelude {
    pub(super) name: CssSupportsConditionName,
}

pub(super) fn parse_prelude<'i, 't>(
    input: &mut Parser<'i, 't>,
    recovery: &RecoveryState,
) -> Result<NamedSupportsPrelude, ParseError<'i, Error>> {
    let at = input.current_source_location();
    let values = CssComponentValues::collect_from_parser(input, recovery.source_snapshot())
        .map_err(|error| invalid_component_value(at, error))?;
    let meaningful: Vec<_> = values
        .items()
        .iter()
        .filter(|value| !crate::supports::trivia(value))
        .collect();
    let [component] = meaningful.as_slice() else {
        return Err(with_at_rule_prelude_context(
            invalid_syntax(at, "one extension name"),
            "supports-condition",
            "ext.rule.supports-condition",
            "one extension name",
        ));
    };
    let name =
        CssSupportsConditionName::try_from_component((*component).clone()).map_err(|_| {
            with_at_rule_prelude_context(
                invalid_syntax(at, "one extension name"),
                "supports-condition",
                "ext.rule.supports-condition",
                "one extension name",
            )
        })?;
    Ok(NamedSupportsPrelude { name })
}

pub(super) fn parse_rule<'i, 't>(
    source: &'i str,
    prelude: NamedSupportsPrelude,
    start: &ParserState,
    input: &mut Parser<'i, 't>,
    diagnostics: &mut Vec<CssRecoveryDiagnostic>,
    recovery: &RecoveryState,
) -> Result<CssSupportsConditionRule, ParseError<'i, Error>> {
    let (body, opening, closing) = parse_body(source, input, diagnostics, recovery)?;
    let document = recovery
        .syntax_document(source)
        .map_err(|error| invalid_component_value(input.current_source_location(), error))?;
    let mut at_cursor = document
        .cursor_at_source(start.position().byte_index())
        .expect("named at-keyword");
    let CursorItem::Node(at_node) = at_cursor.consume() else {
        unreachable!("named at-keyword node");
    };
    let at_keyword = promote(&document, &[at_node])
        .map_err(|error| invalid_component_value(input.current_source_location(), error))?
        .items()[0]
        .origin()
        .clone();
    Ok(CssSupportsConditionRule::parsed(
        prelude.name,
        body,
        at_keyword,
        opening,
        closing,
    ))
}

pub(super) fn parse_body<'i, 't>(
    source: &'i str,
    input: &mut Parser<'i, 't>,
    diagnostics: &mut Vec<CssRecoveryDiagnostic>,
    recovery: &RecoveryState,
) -> Result<(CssSupportsTestBody, CssValueOrigin, CssValueOrigin), ParseError<'i, Error>> {
    let body_start = input.position().byte_index();
    // Arena preparation may omit resource-invalid components. Actual current
    // source admission must therefore precede checked promotion of this body.
    recovery.check_entered_curly_contents(source, input, "ext.rule.supports-condition")?;
    let document = recovery
        .syntax_document(source)
        .map_err(|error| invalid_component_value(input.current_source_location(), error))?;
    let list = document
        .list_at_source_start(body_start)
        .expect("named body list");
    while input.next_including_whitespace_and_comments().is_ok() {}
    let end = input.position().byte_index();
    let opening = parsed_delimiter(
        recovery.source_snapshot(),
        body_start.saturating_sub(1),
        b'{',
    );
    let closing = if source.as_bytes().get(end) == Some(&b'}') {
        parsed_delimiter(recovery.source_snapshot(), end, b'}')
    } else if let (CssValueOrigin::Parsed(opening), Some(at)) = (
        &opening,
        CssParsedOrigin::from_range(recovery.source_snapshot(), end..end),
    ) {
        recovery.retain_component_closures(vec![body_start.saturating_sub(1)]);
        CssValueOrigin::ImplicitClosure {
            opening: opening.clone(),
            at,
        }
    } else {
        CssValueOrigin::Programmatic
    };
    let mut issues = Vec::new();
    let body = partition(&document, list, Some(closing.clone()), true, &mut issues)
        .map_err(|error| invalid_component_value(input.current_source_location(), error))?;
    for issue in issues {
        if let Some(diagnostic) = issue_diagnostic(source, issue) {
            diagnostics.push(diagnostic);
        }
    }
    let mut nodes = document.lists[list].clone();
    let mut implicit = Vec::new();
    while let Some(node) = nodes.pop() {
        let value = &document.nodes[node];
        if let Some(children) = value.children() {
            nodes.extend(&document.lists[children]);
            if matches!(value.end(), Some(GroupEnd::Implicit { .. }))
                && let CssValueOrigin::Parsed(origin) = value.token().origin.as_ref()
            {
                implicit.push(origin.span().end().byte_offset().value().saturating_sub(1));
            }
        } else if !matches!(value, SyntaxNode::Error { .. }) {
            implicit.extend(
                promote(&document, &[node])
                    .map_err(|error| {
                        invalid_component_value(input.current_source_location(), error)
                    })?
                    .implicit_opening_offsets(),
            );
        }
    }
    recovery.retain_component_closures(implicit);
    Ok((body, opening, closing))
}

fn promote(
    document: &SyntaxDocument<'_>,
    nodes: &[NodeId],
) -> Result<CssComponentValues, CssComponentValueError> {
    crate::component_values::promote_nodes(document, nodes, CssComponentValueLimits::default())
}

fn parsed_delimiter(source: &CssSourceSnapshot, offset: usize, expected: u8) -> CssValueOrigin {
    if source.as_str().as_bytes().get(offset) == Some(&expected) {
        CssParsedOrigin::from_range(source, offset..offset + 1)
            .map(CssValueOrigin::Parsed)
            .unwrap_or(CssValueOrigin::Programmatic)
    } else {
        CssValueOrigin::Programmatic
    }
}

fn issue_diagnostic(source: &str, issue: PartitionIssue) -> Option<CssRecoveryDiagnostic> {
    let origin = issue.origin();
    let position = crate::media::parsed_position(origin)?;
    let location = cssparser::SourceLocation {
        line: position.line().value(),
        column: position.column().value() + 1,
    };
    let error = match &issue {
        PartitionIssue::Grammar(_, _) => {
            from_parse_error(source, invalid_syntax(location, "invalid test candidate"))
        }
        PartitionIssue::Lexical(detail, _) => {
            from_parse_error(source, invalid_component_value(location, detail.clone()))
        }
    };
    let span = match origin {
        CssValueOrigin::Parsed(parsed) => parsed.span(),
        CssValueOrigin::ImplicitClosure { opening, .. } => opening.span(),
        _ => return None,
    };
    CssRecoveryDiagnostic::new(error, span, issue.action())
}

pub(crate) fn construct_test_body(
    values: CssComponentValues,
    limits: CssComponentValueLimits,
) -> Result<CssSupportsTestBody, CssNamedSupportsConstructionError> {
    values.validate_with_limits(limits)?;
    let document = syntax::normalize(
        syntax::SyntaxInput::CheckedComponents(values.items()),
        syntax::SyntaxInputLimits {
            max_depth: limits.max_nesting_depth(),
            max_components: limits.max_components(),
            // Public source/output byte admission remains owned by values and the
            // checked writer. Do not reinterpret it as an input spelling metric.
            max_known_spelling_bytes: usize::MAX,
        },
        0,
    )?;
    let mut issues = Vec::new();
    let body = partition(&document, document.root, None, false, &mut issues)?;
    if let Some(issue) = issues.into_iter().next() {
        return Err(CssNamedSupportsConstructionError::InvalidBodyGrammar {
            origin: issue.origin().clone(),
        });
    }
    Ok(body)
}

fn flush_run(run: &mut Vec<CssSupportsTestDeclaration>, items: &mut Vec<CssSupportsTestItem>) {
    if !run.is_empty() {
        items.push(CssSupportsTestItem::Declarations(
            CssSupportsTestDeclarations {
                declarations: std::mem::take(run),
            },
        ));
    }
}

fn faults(
    document: &SyntaxDocument<'_>,
    nodes: &[NodeId],
    action: CssRecoveryAction,
    issues: &mut Vec<PartitionIssue>,
) -> bool {
    let before = issues.len();
    let mut pending: Vec<_> = nodes.iter().rev().copied().collect();
    while let Some(id) = pending.pop() {
        match &document.nodes[id] {
            SyntaxNode::Error { token, cause } => {
                let kind = match cause {
                    syntax::SyntaxTokenFault::BadString => CssComponentValueErrorKind::BadString,
                    syntax::SyntaxTokenFault::BadUrl => CssComponentValueErrorKind::BadUrl,
                    syntax::SyntaxTokenFault::UnexpectedCloser => {
                        CssComponentValueErrorKind::UnmatchedClosingDelimiter
                    }
                };
                issues.push(PartitionIssue::Lexical(
                    CssComponentValueError::new(kind, token.origin.clone().into_owned()),
                    action,
                ));
            }
            node => {
                if let Some(children) = node.children() {
                    pending.extend(document.lists[children].iter().rev().copied());
                }
            }
        }
    }
    issues.len() != before
}

fn closing_origin(document: &SyntaxDocument<'_>, node: NodeId) -> CssValueOrigin {
    match document.nodes[node].end().expect("group end") {
        GroupEnd::Explicit(token) => token.origin.clone().into_owned(),
        GroupEnd::Implicit { opening, at } => match (opening.as_ref(), &at.source) {
            (CssValueOrigin::Parsed(opening), Some(at)) => CssValueOrigin::ImplicitClosure {
                opening: opening.clone(),
                at: CssParsedOrigin::from_range(&at.snapshot, at.offset..at.offset)
                    .expect("actual child EOF"),
            },
            _ => CssValueOrigin::Programmatic,
        },
    }
}

fn curly(document: &SyntaxDocument<'_>, node: NodeId) -> bool {
    matches!(
        document.nodes[node],
        SyntaxNode::Block {
            kind: CssBlockKind::CurlyBracket,
            ..
        }
    )
}

enum PendingChild {
    At(Box<PendingAt>),
    Qualified(Box<PendingQualified>),
}
struct PendingAt {
    name: String,
    at_keyword: CssComponentValue,
    prelude: CssComponentValues,
    opening: CssValueOrigin,
    closing: CssValueOrigin,
    malformed: bool,
}
struct PendingQualified {
    prelude: CssComponentValues,
    opening: CssValueOrigin,
    closing: CssValueOrigin,
    malformed: bool,
}
struct PartitionFrame<'a> {
    cursor: SyntaxCursor<'a>,
    closing: Option<CssValueOrigin>,
    initial_issues: usize,
    items: Vec<CssSupportsTestItem>,
    run: Vec<CssSupportsTestDeclaration>,
    pending: Option<PendingChild>,
    implicit: Option<CssValueOrigin>,
}
impl<'a> PartitionFrame<'a> {
    fn new(
        document: &'a SyntaxDocument<'a>,
        list: ListId,
        closing: Option<CssValueOrigin>,
        initial_issues: usize,
    ) -> Self {
        Self {
            cursor: document.cursor(list),
            closing,
            initial_issues,
            items: Vec::new(),
            run: Vec::new(),
            pending: None,
            implicit: None,
        }
    }
    fn retain_implicit(&mut self, values: &CssComponentValues) {
        if self.implicit.is_none() {
            self.implicit = values.first_implicit_origin().cloned();
        }
    }
}

fn partition(
    document: &SyntaxDocument<'_>,
    list: ListId,
    closing: Option<CssValueOrigin>,
    recover: bool,
    issues: &mut Vec<PartitionIssue>,
) -> Result<CssSupportsTestBody, CssComponentValueError> {
    let mut frames = vec![PartitionFrame::new(document, list, closing, issues.len())];
    loop {
        let frame = frames.last_mut().expect("partition frame");
        frame.cursor.skip_trivia();
        let CursorItem::Node(first) = frame.cursor.peek() else {
            flush_run(&mut frame.run, &mut frame.items);
            let completed = CssSupportsTestBody {
                items: std::mem::take(&mut frame.items),
                recovery_origin: issues
                    .get(frame.initial_issues)
                    .map(|issue| issue.origin().clone())
                    .or_else(|| frame.implicit.clone())
                    .or_else(|| {
                        frame
                            .closing
                            .as_ref()
                            .filter(|origin| {
                                matches!(origin, CssValueOrigin::ImplicitClosure { .. })
                            })
                            .cloned()
                    }),
            };
            frames.pop();
            let Some(parent) = frames.last_mut() else {
                return Ok(completed);
            };
            match parent.pending.take().expect("child continuation") {
                PendingChild::At(pending) => {
                    if !pending.malformed {
                        parent
                            .items
                            .push(CssSupportsTestItem::AtRule(CssSupportsAtRuleTest::new(
                                pending.name,
                                pending.at_keyword,
                                pending.prelude,
                                Some(Box::new(completed)),
                                Some(pending.opening),
                                pending.closing,
                            )));
                    }
                }
                PendingChild::Qualified(pending) => {
                    if !pending.malformed {
                        parent.items.push(CssSupportsTestItem::QualifiedRule(
                            CssSupportsQualifiedRuleTest {
                                prelude: pending.prelude,
                                body: Box::new(completed),
                                opening: pending.opening,
                                closing: pending.closing,
                            },
                        ));
                    }
                }
            }
            continue;
        };
        if document.nodes[first].token().kind() == TokenKind::Semicolon {
            frame.cursor.consume();
            continue;
        }
        if let TokenKind::AtKeyword(name) = document.nodes[first].token().kind() {
            let name = name.to_owned();
            flush_run(&mut frame.run, &mut frame.items);
            let candidate = syntax::consume_at_rule(&mut frame.cursor);
            let malformed = faults(
                document,
                &candidate.prelude,
                CssRecoveryAction::DropAtRule,
                issues,
            );
            let prelude = if malformed {
                CssComponentValues::try_new(Vec::new())?
            } else {
                promote(document, &candidate.prelude)?
            };
            frame.retain_implicit(&prelude);
            let at_keyword = promote(document, &[candidate.name])?.items()[0].clone();
            if let Some(block) = candidate.block {
                let opening = document.nodes[block].token().origin.clone().into_owned();
                let closing = closing_origin(document, block);
                frame.pending = Some(PendingChild::At(Box::new(PendingAt {
                    name,
                    at_keyword,
                    prelude,
                    opening,
                    closing: closing.clone(),
                    malformed,
                })));
                frames.push(PartitionFrame::new(
                    document,
                    document.nodes[block].children().expect("body"),
                    Some(closing),
                    issues.len(),
                ));
            } else if !malformed {
                let terminator = match candidate.termination {
                    syntax::RuleTermination::Semicolon(node) => {
                        document.nodes[node].token().origin.clone().into_owned()
                    }
                    _ => CssValueOrigin::Programmatic,
                };
                frame
                    .items
                    .push(CssSupportsTestItem::AtRule(CssSupportsAtRuleTest::new(
                        name, at_keyword, prelude, None, None, terminator,
                    )));
            } else if !recover {
                return Ok(CssSupportsTestBody {
                    items: std::mem::take(&mut frame.items),
                    recovery_origin: None,
                });
            }
            continue;
        }
        let mut attempt = frame.cursor.clone();
        let range = syntax::consume_declaration_candidate(&mut attempt);
        let nodes = &document.lists[range.list][range.start..range.end];
        let parsed = syntax::consume_declaration(&mut document.cursor_range(&range));
        let custom_name = matches!(document.nodes[first].token().kind(), TokenKind::Ident(name) if name.starts_with("--"))
            && parsed.is_ok();
        let valid_custom_name = matches!(document.nodes[first].token().kind(), TokenKind::Ident(name) if CssCustomPropertyName::from_ident_token(name).is_some());
        let permitted_value = parsed.as_ref().is_ok_and(|declaration| {
            let meaningful: Vec<_> = declaration
                .value
                .iter()
                .filter(|node| !document.nodes[**node].is_trivia())
                .collect();
            valid_custom_name
                || !meaningful.iter().any(|node| curly(document, **node))
                || (meaningful.len() == 1 && curly(document, *meaningful[0]))
        });
        if let Ok(declaration) = parsed
            && permitted_value
        {
            let malformed = faults(document, nodes, CssRecoveryAction::DropDeclaration, issues);
            let terminator = match attempt.peek() {
                CursorItem::Node(node) => document.nodes[node].token().origin.clone().into_owned(),
                _ => CssValueOrigin::Programmatic,
            };
            frame.cursor = attempt;
            if matches!(frame.cursor.peek(), CursorItem::Node(_)) {
                frame.cursor.consume();
            }
            if !malformed {
                let components = promote(document, nodes)?;
                frame.retain_implicit(&components);
                let TokenKind::Ident(name) = document.nodes[declaration.name].token().kind() else {
                    unreachable!("declaration name");
                };
                frame.run.push(CssSupportsTestDeclaration {
                    components: components.items().to_vec(),
                    property_index: declaration.range.start - range.start,
                    value_range: declaration.colon + 1 - range.start
                        ..declaration.value_range.end - range.start,
                    importance_range: if declaration.important {
                        declaration
                            .importance_range
                            .map(|range_| range_.start - range.start..range_.end - range.start)
                    } else {
                        None
                    },
                    name: name.to_owned(),
                    terminator,
                });
            } else if !recover {
                return Ok(CssSupportsTestBody {
                    items: std::mem::take(&mut frame.items),
                    recovery_origin: None,
                });
            }
            continue;
        }
        let candidate = if !custom_name {
            syntax::consume_qualified_rule(&mut document.cursor_range(&range)).ok()
        } else {
            None
        };
        if let Some(candidate) = candidate {
            flush_run(&mut frame.run, &mut frame.items);
            let malformed = faults(
                document,
                &candidate.prelude,
                CssRecoveryAction::DropQualifiedRule,
                issues,
            );
            let prelude = if malformed {
                CssComponentValues::try_new(Vec::new())?
            } else {
                promote(document, &candidate.prelude)?
            };
            frame.retain_implicit(&prelude);
            let opening = document.nodes[candidate.block]
                .token()
                .origin
                .clone()
                .into_owned();
            let closing = closing_origin(document, candidate.block);
            frame.cursor = document.cursor_range(&SyntaxRange {
                list: range.list,
                start: candidate.range.end,
                end: document.lists[range.list].len(),
            });
            frame.pending = Some(PendingChild::Qualified(Box::new(PendingQualified {
                prelude,
                opening,
                closing: closing.clone(),
                malformed,
            })));
            frames.push(PartitionFrame::new(
                document,
                document.nodes[candidate.block].children().expect("body"),
                Some(closing),
                issues.len(),
            ));
            continue;
        }
        if !faults(document, nodes, CssRecoveryAction::DropDeclaration, issues) {
            issues.push(PartitionIssue::Grammar(
                document.nodes[first].token().origin.clone().into_owned(),
                CssRecoveryAction::DropDeclaration,
            ));
        }
        frame.cursor = attempt;
        if matches!(frame.cursor.peek(), CursorItem::Node(_)) {
            frame.cursor.consume();
        }
        if !recover {
            return Ok(CssSupportsTestBody {
                items: std::mem::take(&mut frame.items),
                recovery_origin: None,
            });
        }
    }
}

#[cfg(test)]
mod candidate_membership_independent_tests;

#[cfg(test)]
mod tests {
    use super::*;
    use std::borrow::Cow;

    #[test]
    fn ordered_fault_membership_does_not_require_monotonic_or_present_source_offsets() {
        fn checked(component: &CssComponentValue) -> syntax::SyntaxToken<'_> {
            syntax::SyntaxToken {
                payload: syntax::TokenPayload::Checked {
                    component,
                    lexeme: syntax::CheckedLexeme::Leaf,
                },
                spelling: Some(Cow::Borrowed(component.structural_lexeme(false).unwrap().0)),
                origin: Cow::Borrowed(component.origin()),
            }
        }
        let before = crate::parse_component_values(
            "                                                  before:yes;",
        )
        .unwrap();
        let bad = crate::parse_component_values("bad:;").unwrap();
        let after = crate::parse_component_values("after:yes;").unwrap();
        let mut tokens: Vec<_> = before
            .items()
            .iter()
            .filter(|value| !crate::supports::trivia(value))
            .map(checked)
            .collect();
        tokens.extend(bad.items()[..2].iter().map(checked));
        tokens.push(syntax::SyntaxToken {
            payload: syntax::TokenPayload::Native(Token::BadUrl("a b".into())),
            spelling: Some(Cow::Borrowed("url(a b)")),
            origin: Cow::Owned(CssValueOrigin::Programmatic),
        });
        tokens.push(checked(&bad.items()[2]));
        tokens.extend(after.items().iter().map(checked));
        let document = syntax::normalize(
            syntax::SyntaxInput::Tokens(&tokens),
            syntax::SyntaxInputLimits::default(),
            0,
        )
        .unwrap();
        let mut issues = Vec::new();
        let body = partition(&document, document.root, None, true, &mut issues).unwrap();
        let [CssSupportsTestItem::Declarations(run)] = body.items() else {
            panic!("one retained run");
        };
        assert_eq!(
            run.declarations()
                .iter()
                .map(|declaration| declaration.property())
                .collect::<Vec<_>>(),
            ["before", "after"]
        );
        assert_eq!(
            run.declarations()[0]
                .position()
                .unwrap()
                .byte_offset()
                .value(),
            50
        );
        assert_eq!(
            run.declarations()[1]
                .position()
                .unwrap()
                .byte_offset()
                .value(),
            0
        );
        let [PartitionIssue::Lexical(error, action)] = issues.as_slice() else {
            panic!("one candidate-owned lexical fault");
        };
        assert_eq!(error.kind(), CssComponentValueErrorKind::BadUrl);
        assert_eq!(error.origin(), &CssValueOrigin::Programmatic);
        assert_eq!(*action, CssRecoveryAction::DropDeclaration);
    }
}
