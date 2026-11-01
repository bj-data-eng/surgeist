//! Generic Syntax block partitioning for authored supports-condition tests.

use crate::CssParseReport;
use crate::component::CssComponentValues;
use crate::component::values::CssComponentValueErrorKind;
use crate::component::{
    CssComponentValue, CssComponentValueError, CssComponentValueLimits, CssValueOrigin,
};
use crate::declaration::CssCustomPropertyName;
use crate::diagnostic::CssError;
use crate::diagnostic::error::invalid_component_value;
use crate::diagnostic::error::{from_parse_error, invalid_syntax, with_at_rule_prelude_context};
use crate::diagnostic::{CssRecoveryAction, CssRecoveryDiagnostic};
use crate::parser::recovery::RecoveryState;
use crate::parser::syntax_types::Document;
use crate::parser::{fragments, recovery, syntax_bridge};
use crate::query::supports::CssSupportsConstructionError;
use crate::query::supports::named_supports::retained_component_recovery;
use crate::query::supports::{
    CssSupportsAtRuleTest, CssSupportsConditionName, CssSupportsQualifiedRuleTest,
    CssSupportsTestBody, CssSupportsTestChild, CssSupportsTestChildRef, CssSupportsTestDeclaration,
    CssSupportsTestDeclarations, CssSupportsTestItem,
};
use crate::rule::CssSupportsConditionRule;
use crate::source::{CssParsedOrigin, CssSourceSnapshot};
use crate::syntax_consumption::{
    self as syntax, CursorItem, DocumentExt, SyntaxCursor, SyntaxRange, TokenKind,
};
use cssparser::{ParseError, Parser, ParserState};
use surgeist_css_parser::document::{BlockKind, Component, Fault, GroupEnd, ListId, NodeId};

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

pub(crate) struct NamedSupportsPrelude {
    pub(super) name: CssSupportsConditionName,
}

pub(super) fn parse_prelude<'i, 't>(
    input: &mut Parser<'i, 't>,
    recovery: &RecoveryState,
) -> Result<NamedSupportsPrelude, ParseError<'i, CssError>> {
    let collected =
        super::descriptor_stream::collect_native_scope(input, recovery.source_snapshot())?;
    let at = collected.start;
    let values = collected
        .values
        .map_err(|(error, _)| invalid_component_value(at, error))?;
    let meaningful: Vec<_> = values
        .items()
        .iter()
        .filter(|value| !crate::query::supports::construction::trivia(value))
        .collect();
    let [component] = meaningful.as_slice() else {
        return Err(with_at_rule_prelude_context(
            invalid_syntax(at),
            "supports-condition",
            "ext.rule.supports-condition",
            "one extension name",
        ));
    };
    let name =
        CssSupportsConditionName::try_from_component((*component).clone()).map_err(|_| {
            with_at_rule_prelude_context(
                invalid_syntax(at),
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
) -> Result<CssSupportsConditionRule, ParseError<'i, CssError>> {
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
) -> Result<(CssSupportsTestBody, CssValueOrigin, CssValueOrigin), ParseError<'i, CssError>> {
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
    let mut nodes = document.list(list).to_vec();
    let mut implicit = Vec::new();
    while let Some(node) = nodes.pop() {
        if let Some(children) = document.children(node) {
            nodes.extend(document.list(children));
            if matches!(document.group_end(node), Some(GroupEnd::Eof { .. }))
                && let CssValueOrigin::Parsed(origin) = &document.origin(node)
            {
                implicit.push(origin.span().end().byte_offset().value().saturating_sub(1));
            }
        } else if document.component(node).fault().is_none() {
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
    document: &Document<'_>,
    nodes: &[NodeId],
) -> Result<CssComponentValues, CssComponentValueError> {
    crate::component::values::parse::promote_nodes(
        document,
        nodes,
        CssComponentValueLimits::default(),
    )
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
    let position = crate::query::media::construction::parsed_position(origin)?;
    let location = cssparser::SourceLocation {
        line: position.line().value(),
        column: position.column().value() + 1,
    };
    let error = match &issue {
        PartitionIssue::Grammar(_, _) => from_parse_error(source, invalid_syntax(location)),
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
) -> Result<CssSupportsTestBody, CssSupportsConstructionError> {
    values.validate_with_limits(limits)?;
    let document = syntax::checked_document(
        values.items(),
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
    let body = partition(&document, document.root_list(), None, false, &mut issues)?;
    if let Some(issue) = issues.into_iter().next() {
        return Err(CssSupportsConstructionError::from(
            crate::query::supports::construction::SupportsConstructionKind::InvalidBodyGrammar {
                origin: issue.origin().clone(),
            },
        ));
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
    document: &Document<'_>,
    nodes: &[NodeId],
    action: CssRecoveryAction,
    issues: &mut Vec<PartitionIssue>,
) -> bool {
    let before = issues.len();
    let mut pending: Vec<_> = nodes.iter().rev().copied().collect();
    while let Some(id) = pending.pop() {
        if let Some(fault) = document.component(id).fault() {
            let kind = match fault {
                Fault::BadString => CssComponentValueErrorKind::BadString,
                Fault::BadUrl => CssComponentValueErrorKind::BadUrl,
                Fault::UnexpectedCloser => CssComponentValueErrorKind::UnmatchedClosingDelimiter,
                // A parse error the tree newly preserves is still an invalid token.
                _ => CssComponentValueErrorKind::InvalidToken,
            };
            issues.push(PartitionIssue::Lexical(
                CssComponentValueError::new(kind, document.origin(id)),
                action,
            ));
        } else if let Some(children) = document.children(id) {
            pending.extend(document.list(children).iter().rev().copied());
        }
    }
    issues.len() != before
}

fn curly(document: &Document<'_>, node: NodeId) -> bool {
    matches!(
        document.component(node),
        Component::Block {
            kind: BlockKind::Curly,
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
    local_recovery: Vec<CssValueOrigin>,
    items: Vec<CssSupportsTestItem>,
    run: Vec<CssSupportsTestDeclaration>,
    pending: Option<PendingChild>,
}
impl<'a> PartitionFrame<'a> {
    fn new(document: &'a Document<'a>, list: ListId, closing: Option<CssValueOrigin>) -> Self {
        Self {
            cursor: document.cursor(list),
            closing,
            local_recovery: Vec::new(),
            items: Vec::new(),
            run: Vec::new(),
            pending: None,
        }
    }
}

fn partition(
    document: &Document<'_>,
    list: ListId,
    closing: Option<CssValueOrigin>,
    recover: bool,
    issues: &mut Vec<PartitionIssue>,
) -> Result<CssSupportsTestBody, CssComponentValueError> {
    let mut frames = vec![PartitionFrame::new(document, list, closing)];
    loop {
        let frame = frames.last_mut().expect("partition frame");
        while let CursorItem::Node(node) = frame.cursor.peek() {
            if !document.is_trivia(node) {
                break;
            }
            frame.local_recovery.extend(retained_component_recovery(
                promote(document, &[node])?.items(),
            ));
            frame.cursor.consume();
        }
        let CursorItem::Node(first) = frame.cursor.peek() else {
            flush_run(&mut frame.run, &mut frame.items);
            if let Some(origin @ CssValueOrigin::ImplicitClosure { .. }) = &frame.closing {
                frame.local_recovery.push(origin.clone());
            }
            let completed = CssSupportsTestBody::from_parts(
                std::mem::take(&mut frame.items),
                std::mem::take(&mut frame.local_recovery),
            );
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
                    } else {
                        parent
                            .local_recovery
                            .extend(completed.recovery_origins().cloned());
                    }
                }
                PendingChild::Qualified(pending) => {
                    if !pending.malformed {
                        parent.items.push(CssSupportsTestItem::QualifiedRule(
                            CssSupportsQualifiedRuleTest::new(
                                pending.prelude,
                                Box::new(completed),
                                pending.opening,
                                pending.closing,
                            ),
                        ));
                    } else {
                        parent
                            .local_recovery
                            .extend(completed.recovery_origins().cloned());
                    }
                }
            }
            continue;
        };
        if document.kind(first) == TokenKind::Semicolon {
            frame.cursor.consume();
            continue;
        }
        if let TokenKind::AtKeyword(name) = document.kind(first) {
            let name = name.to_owned();
            flush_run(&mut frame.run, &mut frame.items);
            let candidate = syntax::consume_at_rule(&mut frame.cursor);
            let issue_start = issues.len();
            let malformed = faults(
                document,
                &candidate.prelude,
                CssRecoveryAction::DropAtRule,
                issues,
            );
            frame.local_recovery.extend(
                issues[issue_start..]
                    .iter()
                    .map(|issue| issue.origin().clone()),
            );
            let prelude = if malformed {
                CssComponentValues::try_new(Vec::new(), Default::default())?
            } else {
                promote(document, &candidate.prelude)?
            };
            let at_keyword = promote(document, &[candidate.name])?.items()[0].clone();
            if let Some(block) = candidate.block {
                let opening = document.origin(block);
                let closing = document.closing_origin(block);
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
                    document.children(block).expect("body"),
                    Some(closing),
                ));
            } else if !malformed {
                let terminator = match candidate.termination {
                    syntax::RuleTermination::Semicolon(node) => document.origin(node),
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
                    local_recovery: Vec::new(),
                    recovery_origin: None,
                });
            }
            continue;
        }
        let mut attempt = frame.cursor.clone();
        let range = syntax::consume_declaration_candidate(&mut attempt);
        let nodes = &document.list(range.list)[range.start..range.end];
        let parsed = syntax::consume_declaration(&mut document.cursor_range(&range));
        let custom_name = matches!(document.kind(first), TokenKind::Ident(name) if name.starts_with("--"))
            && parsed.is_ok();
        let valid_custom_name = matches!(document.kind(first), TokenKind::Ident(name) if CssCustomPropertyName::from_ident_token(name).is_some());
        let permitted_value = parsed.as_ref().is_ok_and(|declaration| {
            let meaningful: Vec<_> = declaration
                .value
                .iter()
                .filter(|node| !document.is_trivia(**node))
                .collect();
            valid_custom_name
                || !meaningful.iter().any(|node| curly(document, **node))
                || (meaningful.len() == 1 && curly(document, *meaningful[0]))
        });
        if let Ok(declaration) = parsed
            && permitted_value
        {
            let issue_start = issues.len();
            let malformed = faults(document, nodes, CssRecoveryAction::DropDeclaration, issues);
            frame.local_recovery.extend(
                issues[issue_start..]
                    .iter()
                    .map(|issue| issue.origin().clone()),
            );
            let terminator = match attempt.peek() {
                CursorItem::Node(node) => document.origin(node),
                _ => CssValueOrigin::Programmatic,
            };
            frame.cursor = attempt;
            if matches!(frame.cursor.peek(), CursorItem::Node(_)) {
                frame.cursor.consume();
            }
            if !malformed {
                let components = promote(document, nodes)?;
                let TokenKind::Ident(name) = document.kind(declaration.name) else {
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
                    recovery: retained_component_recovery(components.items()),
                    terminator,
                });
            } else if !recover {
                return Ok(CssSupportsTestBody {
                    items: std::mem::take(&mut frame.items),
                    local_recovery: Vec::new(),
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
            let issue_start = issues.len();
            let malformed = faults(
                document,
                &candidate.prelude,
                CssRecoveryAction::DropQualifiedRule,
                issues,
            );
            frame.local_recovery.extend(
                issues[issue_start..]
                    .iter()
                    .map(|issue| issue.origin().clone()),
            );
            let prelude = if malformed {
                CssComponentValues::try_new(Vec::new(), Default::default())?
            } else {
                promote(document, &candidate.prelude)?
            };
            let opening = document.origin(candidate.block);
            let closing = document.closing_origin(candidate.block);
            frame.cursor = document.cursor_range(&SyntaxRange {
                list: range.list,
                start: candidate.range.end,
                end: document.list(range.list).len(),
            });
            frame.pending = Some(PendingChild::Qualified(Box::new(PendingQualified {
                prelude,
                opening,
                closing: closing.clone(),
                malformed,
            })));
            frames.push(PartitionFrame::new(
                document,
                document.children(candidate.block).expect("body"),
                Some(closing),
            ));
            continue;
        }
        let issue_start = issues.len();
        if !faults(document, nodes, CssRecoveryAction::DropDeclaration, issues) {
            issues.push(PartitionIssue::Grammar(
                document.origin(first),
                CssRecoveryAction::DropDeclaration,
            ));
        }
        frame.local_recovery.extend(
            issues[issue_start..]
                .iter()
                .map(|issue| issue.origin().clone()),
        );
        frame.cursor = attempt;
        if matches!(frame.cursor.peek(), CursorItem::Node(_)) {
            frame.cursor.consume();
        }
        if !recover {
            return Ok(CssSupportsTestBody {
                items: std::mem::take(&mut frame.items),
                local_recovery: Vec::new(),
                recovery_origin: None,
            });
        }
    }
}

#[must_use]
pub(crate) fn parse_supports_test_contents_with_limits(
    source: &str,
    limits: CssComponentValueLimits,
) -> CssParseReport<Option<CssSupportsTestBody>> {
    let refused = || None;
    fragments::bounded_report(source, refused, || {
        let document = match syntax::source_document_with_limits(source, limits) {
            Ok(document) => document,
            Err(error) => {
                return CssParseReport::new(None, vec![syntax_bridge::arena_error(source, error)]);
            }
        };
        let mut issues = Vec::new();
        let body = match partition(&document, document.root_list(), None, true, &mut issues) {
            Ok(body) => body,
            Err(error) => {
                return CssParseReport::new(None, vec![syntax_bridge::arena_error(source, error)]);
            }
        };
        let mut diagnostics: Vec<_> = issues
            .into_iter()
            .filter_map(|issue| issue_diagnostic(source, issue))
            .collect();
        for origin in body.recovery_origins() {
            if let CssValueOrigin::ImplicitClosure { opening, at } = origin
                && let Some(diagnostic) = CssRecoveryDiagnostic::new(
                    crate::diagnostic::error::implicit_eof(source),
                    crate::source::CssSourceSpan::new(opening.span().start(), at.span().end())
                        .expect("actual implicit source extent"),
                    CssRecoveryAction::RetainWithImplicitClosure,
                )
            {
                diagnostics.push(diagnostic);
            }
        }
        recovery::finish_report(source, CssParseReport::new(Some(body), diagnostics))
    })
}

#[must_use]
pub(crate) fn parse_supports_test_child_with_limits(
    source: &str,
    limits: CssComponentValueLimits,
) -> CssParseReport<Option<CssSupportsTestChild>> {
    let report = parse_supports_test_contents_with_limits(source, limits);
    let (body, mut diagnostics) = report.into_parts();
    let Some(body) = body else {
        return CssParseReport::new(None, diagnostics);
    };
    if !diagnostics.is_empty() || body.recovery_origin().is_some() {
        return CssParseReport::new(None, diagnostics);
    }
    let mut children = body.children();
    let first = children.next();
    let extra = children.next();
    if first.is_none() || extra.is_some() {
        let snapshot = CssSourceSnapshot::new(source);
        let origin = extra
            .map(CssSupportsTestChildRef::origin)
            .cloned()
            .unwrap_or_else(|| {
                CssValueOrigin::Parsed(
                    CssParsedOrigin::from_range(&snapshot, 0..0).expect("original empty position"),
                )
            });
        let kind = if first.is_none() {
            CssComponentValueErrorKind::EmptyInput
        } else {
            CssComponentValueErrorKind::TrailingInput
        };
        diagnostics.push(syntax_bridge::arena_error(
            source,
            CssComponentValueError::new(kind, origin),
        ));
        return CssParseReport::new(None, diagnostics);
    }
    let child = match first.expect("one child") {
        CssSupportsTestChildRef::Declaration(v) => CssSupportsTestChild::Declaration(v.clone()),
        CssSupportsTestChildRef::QualifiedRule(v) => CssSupportsTestChild::QualifiedRule(v.clone()),
        CssSupportsTestChildRef::AtRule(v) => CssSupportsTestChild::AtRule(v.clone()),
    };
    CssParseReport::new(Some(child), diagnostics)
}

#[cfg(test)]
pub(crate) mod candidate_membership_independent_tests;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ordered_fault_membership_does_not_require_monotonic_or_present_source_offsets() {
        use syntax::Supplied::{Checked, Fault};
        let before = crate::component::values::parse_component_values(
            "                                                  before:yes;",
        )
        .unwrap();
        let bad = crate::component::values::parse_component_values("bad:;").unwrap();
        let after = crate::component::values::parse_component_values("after:yes;").unwrap();
        let mut tokens: Vec<_> = before
            .items()
            .iter()
            .filter(|value| !crate::query::supports::construction::trivia(value))
            .map(Checked)
            .collect();
        tokens.extend(bad.items()[..2].iter().map(Checked));
        tokens.push(Fault(surgeist_css_parser::Token::BadUrl));
        tokens.push(Checked(&bad.items()[2]));
        tokens.extend(after.items().iter().map(Checked));
        let document = syntax::supplied_document(&tokens, None);
        let mut issues = Vec::new();
        let body = partition(&document, document.root_list(), None, true, &mut issues).unwrap();
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
