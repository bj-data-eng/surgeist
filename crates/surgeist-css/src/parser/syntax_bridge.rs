//! Selected generic units bridged to existing contextual grammar callbacks.
use super::*;
use crate::syntax_consumption::{
    self as syntax, GenericFaultKind, GenericRule, GenericRuleResult, RuleTermination,
    SyntaxDocument,
};

pub(super) fn rules(
    source: &str,
    input: &Parser<'_, '_>,
    recovery: &RecoveryState,
    top_level: bool,
) -> Result<
    (std::rc::Rc<SyntaxDocument<'static>>, Vec<GenericRuleResult>),
    crate::CssComponentValueError,
> {
    let document = recovery.syntax_document(source)?;
    let list = document
        .list_at_source_start(input.position().byte_index())
        .expect("source rule-list starts at its original root or block boundary");
    let selected = syntax::consume_rules(&mut document.cursor(list), top_level);
    Ok((document, selected))
}

pub(super) fn one<'i, 't, P, R>(
    source: &'i str,
    input: &mut Parser<'i, 't>,
    parser: &mut P,
    recovery: &RecoveryState,
) -> Result<R, ParseError<'i, Error>>
where
    P: AtRuleParser<'i, AtRule = R, Error = Error>
        + QualifiedRuleParser<'i, QualifiedRule = R, Error = Error>,
{
    let document = recovery.syntax_document(source).map_err(|error| {
        crate::error::invalid_component_value(input.current_source_location(), error)
    })?;
    let selected = syntax::consume_one_rule(&mut document.cursor(document.root));
    let selected = match selected {
        Err(error) if error.kind == GenericFaultKind::EmptyInput => {
            return Err(input.new_error(cssparser::BasicParseErrorKind::EndOfInput));
        }
        Err(error) if error.kind == GenericFaultKind::TrailingInput => {
            // Preserve contextual first-unit failure precedence. This reuses
            // the already charged arena; it never consumes the second unit.
            let mut cursor = document.cursor(document.root);
            cursor.skip_trivia();
            let syntax::CursorItem::Node(node) = cursor.peek() else {
                unreachable!("nonempty first unit")
            };
            if matches!(
                document.nodes[node].token().kind(),
                syntax::TokenKind::AtKeyword(_)
            ) {
                Ok(GenericRule::At(syntax::consume_at_rule(&mut cursor)))
            } else {
                syntax::consume_qualified_rule(&mut cursor).map(GenericRule::Qualified)
            }
        }
        selected => selected,
    };
    let value = parse_selected(source, input, parser, recovery, &document, &selected)
        .map_err(|error| *error.0)?;
    input.expect_exhausted()?;
    Ok(value)
}

// This is the actual unit driver. Generic consumption chooses dispatch and
// boundaries; native callbacks only admit the selected original source unit.
#[inline(never)]
pub(super) fn parse_selected<'i, 't, P, R>(
    source: &'i str,
    input: &mut Parser<'i, 't>,
    parser: &mut P,
    recovery: &RecoveryState,
    document: &SyntaxDocument<'_>,
    selected: &GenericRuleResult,
) -> Result<R, (Box<ParseError<'i, Error>>, &'i str)>
where
    P: AtRuleParser<'i, AtRule = R, Error = Error>
        + QualifiedRuleParser<'i, QualifiedRule = R, Error = Error>,
{
    let range = match selected {
        Ok(GenericRule::At(rule)) => &rule.range,
        Ok(GenericRule::Qualified(rule)) => &rule.range,
        Err(error) => &error.range,
    };
    let start_offset = document
        .boundary(range.list, range.start)
        .source
        .expect("source candidate has its original boundary")
        .offset;
    while input.position().byte_index() < start_offset {
        if input.next_including_whitespace_and_comments().is_err() {
            break;
        }
    }
    let start = input.state();
    let result = match selected {
        Ok(GenericRule::At(rule)) => parse_at_selected(
            source,
            input,
            parser,
            recovery,
            &start,
            rule.termination.clone(),
        ),
        Ok(GenericRule::Qualified(_)) => {
            parse_qualified_selected(source, input, parser, recovery, &start, false)
        }
        Err(error) => {
            debug_assert_eq!(error.kind, GenericFaultKind::QualifiedRuleEndOfInput);
            parse_qualified_selected(source, input, parser, recovery, &start, true)
        }
    };
    result.map_err(|error| {
        (
            Box::new(error),
            &source[start_offset..input.position().byte_index()],
        )
    })
}

// Keep contextual prelude/error temporaries in their own dispatch branch. These
// drivers are recursive through the existing callbacks, so unrelated grammar
// branches must not reserve one another's debug-stack slots.
#[inline(never)]
fn parse_at_selected<'i, 't, P, R>(
    source: &'i str,
    input: &mut Parser<'i, 't>,
    parser: &mut P,
    recovery: &RecoveryState,
    start: &ParserState,
    termination: RuleTermination,
) -> Result<R, ParseError<'i, Error>>
where
    P: AtRuleParser<'i, AtRule = R, Error = Error>,
{
    let name = match input.next_including_whitespace_and_comments()?.clone() {
        Token::AtKeyword(name) => name,
        token => return Err(input.new_unexpected_token_error(token)),
    };
    let prelude = input.parse_until_before(
        Delimiter::Semicolon | Delimiter::CurlyBracketBlock,
        |input| AtRuleParser::parse_prelude(parser, name, input),
    );
    match termination {
        RuleTermination::Block => {
            input.expect_curly_bracket_block()?;
            let prelude = match prelude {
                Ok(prelude) => prelude,
                Err(error) => {
                    let resource =
                        consume_failed_rule_block(source, input, true, recovery, "css.at-rule").1;
                    return Err(resource.unwrap_or(error));
                }
            };
            input.parse_nested_block(|input| {
                AtRuleParser::parse_block(parser, prelude, start, input)
            })
        }
        RuleTermination::Semicolon(_) => {
            input.expect_semicolon()?;
            parser
                .rule_without_block(prelude?, start)
                .map_err(|()| input.new_unexpected_token_error(Token::Semicolon))
        }
        RuleTermination::EndOfInput(_) => parser
            .rule_without_block(prelude?, start)
            .map_err(|()| input.new_error(cssparser::BasicParseErrorKind::EndOfInput)),
    }
}

#[inline(never)]
fn parse_qualified_selected<'i, 't, P, R>(
    source: &'i str,
    input: &mut Parser<'i, 't>,
    parser: &mut P,
    recovery: &RecoveryState,
    start: &ParserState,
    end_of_input: bool,
) -> Result<R, ParseError<'i, Error>>
where
    P: QualifiedRuleParser<'i, QualifiedRule = R, Error = Error>,
{
    let prelude = input.parse_until_before(Delimiter::CurlyBracketBlock, |input| {
        QualifiedRuleParser::parse_prelude(parser, input)
    });
    // The generic qualified-rule EOF owns this error even if a feature prelude
    // also failed. A real block still permits the existing resource recovery.
    input.expect_curly_bracket_block()?;
    let prelude = match prelude {
        Ok(prelude) => prelude,
        Err(error) => {
            if end_of_input {
                return Err(error);
            }
            let resource =
                consume_failed_rule_block(source, input, true, recovery, "css.qualified-rule").1;
            return Err(resource.unwrap_or(error));
        }
    };
    input
        .parse_nested_block(|input| QualifiedRuleParser::parse_block(parser, prelude, start, input))
}

pub(super) fn arena_error(
    source: &str,
    error: crate::CssComponentValueError,
) -> crate::CssRecoveryDiagnostic {
    let position = match error.origin() {
        crate::CssValueOrigin::Parsed(origin) => origin.span().start(),
        _ => crate::CssSourcePosition::from_byte_offset_in(source, 0),
    };
    let parsed = crate::error::invalid_component_value(
        cssparser::SourceLocation {
            line: position.line().value(),
            column: position.column().value() + 1,
        },
        error,
    );
    let action = recovery_action_for_error(&parsed, crate::CssRecoveryAction::DropQualifiedRule);
    crate::CssRecoveryDiagnostic::new(
        from_parse_error(source, parsed),
        crate::CssSourceSpan::new(
            crate::CssSourcePosition::from_byte_offset_in(source, 0),
            crate::CssSourcePosition::from_byte_offset_in(source, source.len()),
        )
        .expect("complete source span"),
        action,
    )
    .expect("arena failure belongs to this source")
}

// Native RuleBodyParser conflates semicolon and EOF only after admitting the
// prelude. The callback marker preserves that phase; the shared source arena
// supplies its actual termination without relexing or admitting another grammar.
pub(super) fn rejected_at_rule_error<'i>(
    source: &str,
    recovery: &RecoveryState,
    rejected_start: Option<usize>,
    mut error: ParseError<'i, Error>,
) -> ParseError<'i, Error> {
    if !matches!(
        &error.kind,
        cssparser::ParseErrorKind::Basic(cssparser::BasicParseErrorKind::UnexpectedToken(
            Token::Semicolon
        ))
    ) {
        return error;
    }
    let Some(start) = rejected_start else {
        return error;
    };
    let Ok(document) = recovery.syntax_document(source) else {
        return error;
    };
    let Some(mut cursor) = document.cursor_at_source(start) else {
        return error;
    };
    let syntax::CursorItem::Node(node) = cursor.peek() else {
        return error;
    };
    if !matches!(
        document.nodes[node].token().kind(),
        syntax::TokenKind::AtKeyword(_)
    ) {
        return error;
    }
    if matches!(
        syntax::consume_at_rule(&mut cursor).termination,
        RuleTermination::EndOfInput(_)
    ) {
        error.kind = cssparser::ParseErrorKind::Basic(cssparser::BasicParseErrorKind::EndOfInput);
    }
    error
}

pub(super) fn retain_statement_eof(
    source: &str,
    recovery: &RecoveryState,
    start: &ParserState,
    diagnostics: &mut Vec<crate::CssRecoveryDiagnostic>,
) {
    let document = recovery
        .syntax_document(source)
        .expect("source driver admitted its shared arena");
    let mut cursor = document
        .cursor_at_source(start.position().byte_index())
        .expect("statement callback owns a selected at-keyword");
    let rule = syntax::consume_at_rule(&mut cursor);
    if let (Some(problem), RuleTermination::EndOfInput(at)) = (rule.fault, rule.termination) {
        let error_end = problem
            .at
            .source
            .expect("source statement fault has its responsible EOF point")
            .offset;
        let end = at
            .source
            .expect("source statement EOF has a bounded source point")
            .offset;
        let span = crate::CssSourceSpan::new(
            crate::CssSourcePosition::from_byte_offset_in(source, start.position().byte_index()),
            crate::CssSourcePosition::from_byte_offset_in(source, end),
        )
        .expect("source statement range");
        diagnostics.push(
            crate::CssRecoveryDiagnostic::new(
                crate::error::unterminated_at_rule(source, error_end),
                span,
                crate::CssRecoveryAction::RetainNonconformingRule,
            )
            .expect("statement EOF lies within its source unit"),
        );
    }
}
