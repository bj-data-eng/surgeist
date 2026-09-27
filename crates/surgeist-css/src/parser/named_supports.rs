//! Generic Syntax block partitioning for authored supports-condition tests.

use super::*;
use crate::error::invalid_component_value;
use crate::named_supports::*;
use crate::{
    CssBlockKind, CssComponentValue, CssComponentValueError, CssComponentValueLimits,
    CssComponentValueRef, CssRecoveryAction, CssRecoveryDiagnostic, CssSimpleBlock, CssValueOrigin,
    CssValueTokenRef,
};
use cssparser::{ParseError, Parser, ParserState};
use std::ops::Range;

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
    let body_start = input.position().byte_index();
    let (values, lexical) = CssComponentValue::collect_recovering_named_test(
        input,
        recovery.source_snapshot(),
        recovery.structural_depth(),
    )
    .map_err(|error| invalid_component_value(input.current_source_location(), error))?;
    let end = input.position().byte_index();
    let mut issues = Vec::new();
    let mut body = partition(values.items(), true, &mut issues, &lexical, body_start..end);
    for issue in issues {
        if let Some(diagnostic) = issue_diagnostic(source, issue) {
            diagnostics.push(diagnostic);
        }
    }
    let at_keyword = {
        let mut parser_input = cssparser::ParserInput::new(source);
        let mut parser = Parser::new(&mut parser_input);
        parser.reset(start);
        CssComponentValue::collect_from_parser(&mut parser, recovery.source_snapshot())
            .map_err(|error| invalid_component_value(parser.current_source_location(), error))?
    };
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
    if body.recovery_origin.is_none() {
        body.recovery_origin = values.first_implicit_origin().cloned().or_else(|| {
            matches!(closing, CssValueOrigin::ImplicitClosure { .. }).then(|| closing.clone())
        });
    }
    recovery.retain_component_closures(values.implicit_opening_offsets());
    Ok(CssSupportsConditionRule::parsed(
        prelude.name,
        body,
        at_keyword.origin().clone(),
        opening,
        closing,
    ))
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
    let mut issues = Vec::new();
    let body = partition(values.items(), false, &mut issues, &[], 0..0);
    if let Some(issue) = issues.into_iter().next() {
        return Err(CssNamedSupportsConstructionError::InvalidBodyGrammar {
            origin: issue.origin().clone(),
        });
    }
    Ok(body)
}

fn is_trivia(value: &CssComponentValue) -> bool {
    crate::supports::trivia(value)
}

fn token(value: &CssComponentValue) -> Option<CssValueTokenRef<'_>> {
    match value.view() {
        CssComponentValueRef::Token(token) => Some(token),
        _ => None,
    }
}

fn curly(value: &CssComponentValue) -> Option<&CssSimpleBlock> {
    match value.view() {
        CssComponentValueRef::Block(block) if block.kind() == CssBlockKind::CurlyBracket => {
            Some(block)
        }
        _ => None,
    }
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

fn origin_range(origin: &CssValueOrigin) -> Option<Range<usize>> {
    match origin {
        CssValueOrigin::Parsed(parsed) => Some(
            parsed.span().start().byte_offset().value()..parsed.span().end().byte_offset().value(),
        ),
        CssValueOrigin::ImplicitClosure { at, .. } => {
            let offset = at.span().start().byte_offset().value();
            Some(offset..offset)
        }
        _ => None,
    }
}

fn component_end(component: &CssComponentValue) -> Option<usize> {
    match component.view() {
        CssComponentValueRef::Block(block) => {
            origin_range(block.closing_origin()).map(|span| span.end)
        }
        CssComponentValueRef::Function(function) => {
            origin_range(function.closing_origin()).map(|span| span.end)
        }
        _ => origin_range(component.origin()).map(|span| span.end),
    }
}

fn record_faults(
    lexical: &[CssComponentValueError],
    range: Range<usize>,
    action: CssRecoveryAction,
    issues: &mut Vec<PartitionIssue>,
) -> bool {
    let mut found = false;
    for error in lexical {
        let Some(span) = origin_range(error.origin()) else {
            continue;
        };
        if range.contains(&span.start) {
            found = true;
            if !issues.iter().any(|issue| matches!(issue, PartitionIssue::Lexical(previous, _) if previous.origin() == error.origin())) {
                issues.push(PartitionIssue::Lexical(error.clone(), action));
            }
        }
    }
    found
}

fn mark_implicit_recovery(
    body: &mut CssSupportsTestBody,
    values: &CssComponentValues,
    closing: &CssValueOrigin,
) {
    if body.recovery_origin.is_none() {
        body.recovery_origin = values.first_implicit_origin().cloned().or_else(|| {
            matches!(closing, CssValueOrigin::ImplicitClosure { .. }).then(|| closing.clone())
        });
    }
}

enum PendingChild {
    AtRule(Box<PendingAtRule>),
    Qualified(Box<PendingQualified>),
}

struct PendingAtRule {
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
    components: &'a [CssComponentValue],
    values: Option<&'a CssComponentValues>,
    closing: Option<&'a CssValueOrigin>,
    source_range: Range<usize>,
    index: usize,
    cursor: usize,
    initial_issues: usize,
    items: Vec<CssSupportsTestItem>,
    run: Vec<CssSupportsTestDeclaration>,
    pending_child: Option<PendingChild>,
}

impl<'a> PartitionFrame<'a> {
    fn new(
        components: &'a [CssComponentValue],
        values: Option<&'a CssComponentValues>,
        closing: Option<&'a CssValueOrigin>,
        source_range: Range<usize>,
        initial_issues: usize,
    ) -> Self {
        Self {
            components,
            values,
            closing,
            cursor: source_range.start,
            source_range,
            index: 0,
            initial_issues,
            items: Vec::new(),
            run: Vec::new(),
            pending_child: None,
        }
    }
}

fn partition(
    components: &[CssComponentValue],
    recover: bool,
    issues: &mut Vec<PartitionIssue>,
    lexical: &[CssComponentValueError],
    source_range: Range<usize>,
) -> CssSupportsTestBody {
    let mut frames = vec![PartitionFrame::new(
        components,
        None,
        None,
        source_range,
        issues.len(),
    )];
    loop {
        let frame = frames
            .last_mut()
            .expect("root frame remains until completion");
        while frame.index < frame.components.len() && is_trivia(&frame.components[frame.index]) {
            frame.index += 1;
        }
        if frame.index == frame.components.len() {
            record_faults(
                lexical,
                frame.source_range.clone(),
                CssRecoveryAction::DropDeclaration,
                issues,
            );
            flush_run(&mut frame.run, &mut frame.items);
            let mut completed = CssSupportsTestBody {
                items: std::mem::take(&mut frame.items),
                recovery_origin: issues
                    .get(frame.initial_issues)
                    .map(|issue| issue.origin().clone()),
            };
            if let (Some(values), Some(closing)) = (frame.values, frame.closing) {
                mark_implicit_recovery(&mut completed, values, closing);
            }
            frames.pop();
            let Some(parent) = frames.last_mut() else {
                return completed;
            };
            let pending = parent
                .pending_child
                .take()
                .expect("child has parent continuation");
            match pending {
                PendingChild::AtRule(pending) => {
                    if !pending.malformed {
                        flush_run(&mut parent.run, &mut parent.items);
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
                        flush_run(&mut parent.run, &mut parent.items);
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
        }

        if matches!(
            token(&frame.components[frame.index]),
            Some(CssValueTokenRef::Semicolon)
        ) {
            let end = component_end(&frame.components[frame.index]).unwrap_or(frame.cursor);
            record_faults(
                lexical,
                frame.cursor..end,
                CssRecoveryAction::DropDeclaration,
                issues,
            );
            frame.cursor = end;
            frame.index += 1;
            continue;
        }

        let start = frame.index;
        if let Some(CssValueTokenRef::AtKeyword(name)) = token(&frame.components[start]) {
            let name = name.to_owned();
            let at_keyword = frame.components[start].clone();
            frame.index += 1;
            while frame.index < frame.components.len()
                && !matches!(
                    token(&frame.components[frame.index]),
                    Some(CssValueTokenRef::Semicolon)
                )
                && curly(&frame.components[frame.index]).is_none()
            {
                frame.index += 1;
            }
            let prelude =
                CssComponentValues::try_new(frame.components[start + 1..frame.index].to_vec())
                    .expect("validated component subsequence");
            if let Some(block) = frame.components.get(frame.index).and_then(curly) {
                let opening = frame.components[frame.index].origin().clone();
                let closing = block.closing_origin().clone();
                let opening_range = origin_range(&opening);
                let closing_range = origin_range(&closing);
                let child_range = opening_range
                    .as_ref()
                    .zip(closing_range.as_ref())
                    .map(|(opening, closing)| opening.end..closing.start)
                    .unwrap_or(0..0);
                let prelude_end = opening_range.map_or(frame.cursor, |span| span.start);
                let malformed = record_faults(
                    lexical,
                    frame.cursor..prelude_end,
                    CssRecoveryAction::DropAtRule,
                    issues,
                );
                frame.cursor =
                    component_end(&frame.components[frame.index]).unwrap_or(frame.cursor);
                frame.index += 1;
                frame.pending_child = Some(PendingChild::AtRule(Box::new(PendingAtRule {
                    name,
                    at_keyword,
                    prelude,
                    opening,
                    closing,
                    malformed,
                })));
                frames.push(PartitionFrame::new(
                    block.values().items(),
                    Some(block.values()),
                    Some(block.closing_origin()),
                    child_range,
                    issues.len(),
                ));
                continue;
            }
            let closing = frame
                .components
                .get(frame.index)
                .map_or(CssValueOrigin::Programmatic, |value| value.origin().clone());
            let candidate_end = frame
                .components
                .get(frame.index)
                .and_then(component_end)
                .unwrap_or(frame.source_range.end);
            if frame.index < frame.components.len() {
                frame.index += 1;
            }
            let malformed = record_faults(
                lexical,
                frame.cursor..candidate_end,
                CssRecoveryAction::DropAtRule,
                issues,
            );
            frame.cursor = candidate_end;
            if malformed {
                if !recover {
                    frame.index = frame.components.len();
                }
                continue;
            }
            flush_run(&mut frame.run, &mut frame.items);
            frame
                .items
                .push(CssSupportsTestItem::AtRule(CssSupportsAtRuleTest::new(
                    name, at_keyword, prelude, None, None, closing,
                )));
            continue;
        }

        // Consume a complete declaration attempt through semicolon/end. Only
        // a failed attempt resets to the first curly for qualified-rule fallback.
        let declaration_end = (start..frame.components.len())
            .find(|&at| {
                matches!(
                    token(&frame.components[at]),
                    Some(CssValueTokenRef::Semicolon)
                )
            })
            .unwrap_or(frame.components.len());
        let mut first_two = frame.components[start..declaration_end]
            .iter()
            .filter(|value| !is_trivia(value));
        let custom_name = matches!(first_two.next().and_then(token), Some(CssValueTokenRef::Ident(name)) if name.starts_with("--"))
            && matches!(
                first_two.next().and_then(token),
                Some(CssValueTokenRef::Colon)
            );
        let terminator = frame
            .components
            .get(declaration_end)
            .map_or(CssValueOrigin::Programmatic, |value| value.origin().clone());
        let candidate_end = frame
            .components
            .get(declaration_end)
            .and_then(component_end)
            .unwrap_or(frame.source_range.end);
        if let Some(declaration) =
            declaration_candidate(&frame.components[start..declaration_end], terminator)
        {
            let malformed = record_faults(
                lexical,
                frame.cursor..candidate_end,
                CssRecoveryAction::DropDeclaration,
                issues,
            );
            if !malformed {
                frame.run.push(declaration);
            }
            frame.index = (declaration_end + 1).min(frame.components.len());
            frame.cursor = candidate_end;
            if malformed && !recover {
                frame.index = frame.components.len();
            }
            continue;
        }
        let first_curly =
            (start..declaration_end).find(|&at| curly(&frame.components[at]).is_some());
        if let Some(block_index) = first_curly
            && let Some(block) = curly(&frame.components[block_index])
            && !custom_name
        {
            let prelude =
                CssComponentValues::try_new(frame.components[start..block_index].to_vec())
                    .expect("validated component subsequence");
            let opening = frame.components[block_index].origin().clone();
            let closing = block.closing_origin().clone();
            let opening_range = origin_range(&opening);
            let closing_range = origin_range(&closing);
            let prelude_end = opening_range
                .as_ref()
                .map_or(frame.cursor, |span| span.start);
            let malformed = record_faults(
                lexical,
                frame.cursor..prelude_end,
                CssRecoveryAction::DropQualifiedRule,
                issues,
            );
            let child_range = opening_range
                .as_ref()
                .zip(closing_range.as_ref())
                .map(|(opening, closing)| opening.end..closing.start)
                .unwrap_or(0..0);
            frame.cursor = component_end(&frame.components[block_index]).unwrap_or(frame.cursor);
            frame.index = block_index + 1;
            frame.pending_child = Some(PendingChild::Qualified(Box::new(PendingQualified {
                prelude,
                opening,
                closing,
                malformed,
            })));
            frames.push(PartitionFrame::new(
                block.values().items(),
                Some(block.values()),
                Some(block.closing_origin()),
                child_range,
                issues.len(),
            ));
            continue;
        }
        let malformed = record_faults(
            lexical,
            frame.cursor..candidate_end,
            CssRecoveryAction::DropDeclaration,
            issues,
        );
        if !malformed {
            issues.push(PartitionIssue::Grammar(
                frame.components[start].origin().clone(),
                CssRecoveryAction::DropDeclaration,
            ));
        }
        frame.index = (declaration_end + 1).min(frame.components.len());
        frame.cursor = candidate_end;
        if !recover {
            frame.index = frame.components.len();
        }
    }
}

fn declaration_candidate(
    components: &[CssComponentValue],
    terminator: CssValueOrigin,
) -> Option<CssSupportsTestDeclaration> {
    let significant: Vec<_> = components
        .iter()
        .enumerate()
        .filter(|(_, component)| !is_trivia(component))
        .map(|(index, _)| index)
        .collect();
    let &property_index = significant.first()?;
    let name = match token(&components[property_index])? {
        CssValueTokenRef::Ident(name) => name.to_owned(),
        _ => return None,
    };
    let &colon = significant.get(1)?;
    if !matches!(token(&components[colon]), Some(CssValueTokenRef::Colon)) {
        return None;
    }
    let mut value_end = components.len();
    let mut importance_range = None;
    if significant.len() >= 4 {
        let bang = significant[significant.len() - 2];
        let important = significant[significant.len() - 1];
        if matches!(token(&components[bang]), Some(CssValueTokenRef::Delim('!')))
            && matches!(token(&components[important]), Some(CssValueTokenRef::Ident(name)) if name.eq_ignore_ascii_case("important"))
        {
            value_end = bang;
            importance_range = Some(bang..components.len());
        }
    }
    while value_end > colon + 1 && is_trivia(&components[value_end - 1]) {
        value_end -= 1;
    }
    if crate::CssCustomPropertyName::from_ident_token(&name).is_none() {
        let meaningful_values = components[colon + 1..value_end]
            .iter()
            .filter(|value| !is_trivia(value))
            .collect::<Vec<_>>();
        if meaningful_values.iter().any(|value| curly(value).is_some())
            && !(meaningful_values.len() == 1 && curly(meaningful_values[0]).is_some())
        {
            return None;
        }
    }
    Some(CssSupportsTestDeclaration {
        components: components.to_vec(),
        property_index,
        value_range: colon + 1..value_end,
        importance_range,
        name,
        terminator,
    })
}
