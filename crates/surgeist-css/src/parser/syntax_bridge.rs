//! Selected generic units bridged to existing contextual grammar callbacks.
use crate::diagnostic::CssError;
use crate::diagnostic::error::from_parse_error;
use crate::parser::consume_failed_rule_block;
use crate::parser::recovery::{RecoveryState, recovery_action_for_error};
use crate::parser::syntax_types::Document;
use crate::syntax_consumption::{
    self as syntax, DocumentExt, GenericFaultKind, GenericRule, GenericRuleResult, RuleListSyntax,
    RuleTermination,
};
use cssparser::{
    AtRuleParser, Delimiter, ParseError, Parser, ParserState, QualifiedRuleParser, Token,
};

pub(super) fn rules(
    source: &str,
    input: &Parser<'_, '_>,
    recovery: &RecoveryState,
    list: RuleListSyntax,
) -> Result<
    (std::rc::Rc<Document<'static>>, Vec<GenericRuleResult>),
    crate::component::CssComponentValueError,
> {
    let document = recovery.syntax_document(source)?;
    let start = document
        .list_at_source_start(input.position().byte_index())
        .expect("source rule-list starts at its original root or block boundary");
    let selected = syntax::consume_rules(&mut document.cursor(start), list);
    Ok((document, selected))
}

/// Discards the `;` and trivia that consume a block's contents leaves after
/// its last selected unit, so the native nested block is exhausted.
pub(super) fn discard_block_contents_tail(input: &mut Parser<'_, '_>) {
    loop {
        let state = input.state();
        match input.next_including_whitespace_and_comments() {
            Ok(Token::WhiteSpace(_) | Token::Comment(_) | Token::Semicolon) => {}
            _ => {
                input.reset(&state);
                break;
            }
        }
    }
}

pub(super) fn one<'i, 't, P, R>(
    source: &'i str,
    input: &mut Parser<'i, 't>,
    parser: &mut P,
    recovery: &RecoveryState,
) -> Result<R, ParseError<'i, CssError>>
where
    P: AtRuleParser<'i, AtRule = R, Error = CssError>
        + QualifiedRuleParser<'i, QualifiedRule = R, Error = CssError>,
{
    let document = recovery.syntax_document(source).map_err(|error| {
        crate::diagnostic::error::invalid_component_value(input.current_source_location(), error)
    })?;
    let selected = syntax::consume_one_rule(&mut document.cursor(document.root_list()));
    let selected = match selected {
        Err(error) if error.kind == GenericFaultKind::EmptyInput => {
            return Err(input.new_error(cssparser::BasicParseErrorKind::EndOfInput));
        }
        Err(error) if error.kind == GenericFaultKind::TrailingInput => {
            // Preserve contextual first-unit failure precedence. This reuses
            // the already charged arena; it never consumes the second unit.
            let mut cursor = document.cursor(document.root_list());
            cursor.skip_trivia();
            let syntax::CursorItem::Node(node) = cursor.peek() else {
                unreachable!("nonempty first unit")
            };
            if matches!(document.kind(node), syntax::TokenKind::AtKeyword(_)) {
                Ok(GenericRule::At(syntax::consume_at_rule(&mut cursor)))
            } else {
                syntax::consume_qualified_rule(&mut cursor).map(GenericRule::Qualified)
            }
        }
        selected => selected,
    };
    let (_, value) = parse_selected(source, input, parser, recovery, &document, &selected);
    let value = value.map_err(|error| *error.0)?;
    input.expect_exhausted()?;
    Ok(value)
}

// One selected unit's admission: its item, or its error with the original
// source slice from the unit's start to the failure.
type SelectedAdmission<'i, R> = Result<R, (Box<ParseError<'i, CssError>>, &'i str)>;

// This is the actual unit driver. Generic consumption chooses dispatch and
// boundaries; native callbacks only admit the selected original source unit.
// Returns the unit's original start offset with its admission.
#[inline(never)]
pub(super) fn parse_selected<'i, 't, P, R>(
    source: &'i str,
    input: &mut Parser<'i, 't>,
    parser: &mut P,
    recovery: &RecoveryState,
    document: &Document<'_>,
    selected: &GenericRuleResult,
) -> (usize, SelectedAdmission<'i, R>)
where
    P: AtRuleParser<'i, AtRule = R, Error = CssError>
        + QualifiedRuleParser<'i, QualifiedRule = R, Error = CssError>,
{
    let range = match selected {
        Ok(GenericRule::At(rule)) => &rule.range,
        Ok(GenericRule::Qualified(rule)) => &rule.range,
        Err(error) => &error.range,
    };
    let start_offset = document
        .boundary(range.list, range.start)
        .offset
        .expect("source candidate has its original boundary");
    while input.position().byte_index() < start_offset {
        if input.next_including_whitespace_and_comments().is_err() {
            break;
        }
    }
    let start = input.state();
    let result =
        admit_selected(source, input, parser, recovery, &start, selected).map_err(|error| {
            (
                Box::new(error),
                &source[start_offset..input.position().byte_index()],
            )
        });
    (start_offset, result)
}

pub(super) fn admit_envelope<'i, 't, P, R>(
    source: &'i str,
    input: &mut Parser<'i, 't>,
    parser: &mut P,
    recovery: &RecoveryState,
    envelope: &super::rule_candidate::RuleEnvelope,
) -> Result<R, ParseError<'i, CssError>>
where
    P: AtRuleParser<'i, AtRule = R, Error = CssError>
        + QualifiedRuleParser<'i, QualifiedRule = R, Error = CssError>,
{
    input.skip_whitespace();
    let start = input.state();
    debug_assert_eq!(start.position().byte_index(), envelope.start);
    let value = if let Some(termination) = &envelope.at_termination {
        parse_at_selected(source, input, parser, recovery, &start, termination.clone())?
    } else {
        parse_qualified_selected(source, input, parser, recovery, &start, false)?
    };
    input.expect_exhausted()?;
    Ok(value)
}

fn admit_selected<'i, 't, P, R>(
    source: &'i str,
    input: &mut Parser<'i, 't>,
    parser: &mut P,
    recovery: &RecoveryState,
    start: &ParserState,
    selected: &GenericRuleResult,
) -> Result<R, ParseError<'i, CssError>>
where
    P: AtRuleParser<'i, AtRule = R, Error = CssError>
        + QualifiedRuleParser<'i, QualifiedRule = R, Error = CssError>,
{
    match selected {
        Ok(GenericRule::At(rule)) => parse_at_selected(
            source,
            input,
            parser,
            recovery,
            start,
            rule.termination.clone(),
        ),
        Ok(GenericRule::Qualified(_)) => {
            parse_qualified_selected(source, input, parser, recovery, start, false)
        }
        Err(error) => match error.kind {
            GenericFaultKind::QualifiedRuleEndOfInput => {
                parse_qualified_selected(source, input, parser, recovery, start, true)
            }
            GenericFaultKind::QualifiedRuleStopToken | GenericFaultKind::CustomPropertyRule => {
                reject_bounded_qualified(input, document_offset(error), error.kind)
            }
            kind => unreachable!("rule-list consumers select no {kind:?} unit"),
        },
    }
}

fn document_offset(error: &syntax::GenericSyntaxFault) -> usize {
    error
        .at
        .offset
        .expect("source rule-list fault has its original boundary")
}

// Syntax 3 (2026 CRD) §5.5.3: a nested qualified rule that meets its `;`
// stop token is a parse error and returns nothing, and one whose prelude
// begins like a custom property consumes the remnants of a bad declaration
// (nested) or its block (top level). None returns a rule, so no contextual
// grammar is admitted; a nested range ends before the `;` that the enclosing
// block's contents discards.
#[inline(never)]
fn reject_bounded_qualified<'i, R>(
    input: &mut Parser<'i, '_>,
    end: usize,
    kind: GenericFaultKind,
) -> Result<R, ParseError<'i, CssError>> {
    let location = input.current_source_location();
    while input.position().byte_index() < end {
        let Ok(token) = input.next_including_whitespace_and_comments() else {
            break;
        };
        if matches!(
            token,
            Token::Function(_)
                | Token::ParenthesisBlock
                | Token::SquareBracketBlock
                | Token::CurlyBracketBlock
        ) {
            // A selected group is one component; finish it before comparing
            // the following boundary, so the unit cannot absorb its `;`.
            let _: Result<(), ParseError<'_, ()>> = input.parse_nested_block(|nested| {
                while nested.next_including_whitespace_and_comments().is_ok() {}
                Ok(())
            });
        }
    }
    Err(if kind == GenericFaultKind::QualifiedRuleStopToken {
        input.new_unexpected_token_error(Token::Semicolon)
    } else {
        location.new_error(cssparser::BasicParseErrorKind::QualifiedRuleInvalid)
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
) -> Result<R, ParseError<'i, CssError>>
where
    P: AtRuleParser<'i, AtRule = R, Error = CssError>,
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
                    let resource = consume_failed_rule_block(
                        source,
                        input,
                        start.position().byte_index(),
                        recovery,
                        "css.at-rule",
                    )
                    .1;
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
) -> Result<R, ParseError<'i, CssError>>
where
    P: QualifiedRuleParser<'i, QualifiedRule = R, Error = CssError>,
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
            let resource = consume_failed_rule_block(
                source,
                input,
                start.position().byte_index(),
                recovery,
                "css.qualified-rule",
            )
            .1;
            return Err(resource.unwrap_or(error));
        }
    };
    input
        .parse_nested_block(|input| QualifiedRuleParser::parse_block(parser, prelude, start, input))
}

pub(super) fn arena_error(
    source: &str,
    error: crate::component::CssComponentValueError,
) -> crate::diagnostic::CssRecoveryDiagnostic {
    component_error_diagnostic(
        source,
        error,
        crate::source::CssSourceSpan::new(
            crate::source::CssSourcePosition::from_byte_offset_in(source, 0),
            crate::source::CssSourcePosition::from_byte_offset_in(source, source.len()),
        )
        .expect("complete source span"),
        crate::diagnostic::CssRecoveryAction::DropQualifiedRule,
    )
}

pub(super) fn component_error_diagnostic(
    source: &str,
    error: crate::component::CssComponentValueError,
    span: crate::source::CssSourceSpan,
    ordinary_action: crate::diagnostic::CssRecoveryAction,
) -> crate::diagnostic::CssRecoveryDiagnostic {
    let position = match error.origin() {
        crate::component::CssValueOrigin::Parsed(origin) => origin.span().start(),
        _ => crate::source::CssSourcePosition::from_byte_offset_in(source, 0),
    };
    let parsed = crate::diagnostic::error::invalid_component_value(
        cssparser::SourceLocation {
            line: position.line().value(),
            column: position.column().value() + 1,
        },
        error,
    );
    let action = recovery_action_for_error(&parsed, ordinary_action);
    crate::diagnostic::CssRecoveryDiagnostic::new(from_parse_error(source, parsed), span, action)
        .expect("arena failure belongs to this source")
}

// Native RuleBodyParser conflates semicolon and EOF only after admitting the
// prelude. The callback marker preserves that phase; the shared source arena
// supplies its actual termination without relexing or admitting another grammar.
pub(super) fn rejected_at_rule_error<'i>(
    source: &str,
    recovery: &RecoveryState,
    rejected_start: Option<usize>,
    mut error: ParseError<'i, CssError>,
) -> ParseError<'i, CssError> {
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
    if !matches!(document.kind(node), syntax::TokenKind::AtKeyword(_)) {
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
    diagnostics: &mut Vec<crate::diagnostic::CssRecoveryDiagnostic>,
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
            .offset
            .expect("source statement fault has its responsible EOF point");
        let end = at
            .offset
            .expect("source statement EOF has a bounded source point");
        let span = crate::source::CssSourceSpan::new(
            crate::source::CssSourcePosition::from_byte_offset_in(
                source,
                start.position().byte_index(),
            ),
            crate::source::CssSourcePosition::from_byte_offset_in(source, end),
        )
        .expect("source statement range");
        diagnostics.push(
            crate::diagnostic::CssRecoveryDiagnostic::new(
                crate::diagnostic::error::unterminated_at_rule(source, error_end),
                span,
                crate::diagnostic::CssRecoveryAction::RetainNonconformingRule,
            )
            .expect("statement EOF lies within its source unit"),
        );
    }
}
