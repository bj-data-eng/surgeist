use cssparser::{ParseError, Parser, ParserInput, ParserState, Token};

use super::recovery::RecoveryState;
use super::selectors::{SelectorRecovery, parse_rule_selector};
use super::{DeclarationMode, parse_declaration_core};
use crate::error::{
    Error, basic, invalid_syntax, is_nesting_limit_error, selector_basic,
    with_at_rule_prelude_context,
};
use crate::syntax::*;

pub(super) static IMPLEMENTED_SHARED_VALUES: &[crate::CssFeatureId] =
    &[crate::CssFeatureId::new("ext.supports.general-enclosed")];

pub(super) static IMPLEMENTED_SELECTORS: &[crate::CssFeatureId] =
    &[crate::CssFeatureId::new("ext.supports.selector")];

enum ParenthesizedCondition {
    Parsed(CssSupportsConditionKind),
    GeneralEnclosed,
}

pub(super) fn with_supports_prelude_context<'i>(
    error: ParseError<'i, Error>,
) -> ParseError<'i, Error> {
    if is_nesting_limit_error(&error) {
        error
    } else {
        with_at_rule_prelude_context(
            error,
            "supports",
            "baseline.rule.supports",
            "a valid supports condition",
        )
    }
}

pub(super) fn parse_supports_condition<'i, 't>(
    source: &'i str,
    input: &mut Parser<'i, 't>,
    diagnostics: &mut Vec<crate::CssRecoveryDiagnostic>,
    recovery: &RecoveryState,
) -> Result<CssSupportsCondition, ParseError<'i, Error>> {
    let implicit =
        recovery.check_specialized_components(source, input, "baseline.rule.supports")?;
    let condition = parse_condition(source, input, diagnostics, recovery)?;
    input.expect_exhausted().map_err(basic)?;
    recovery.retain_component_closures(implicit);
    Ok(condition)
}

fn parse_condition<'i, 't>(
    source: &'i str,
    input: &mut Parser<'i, 't>,
    diagnostics: &mut Vec<crate::CssRecoveryDiagnostic>,
    recovery: &RecoveryState,
) -> Result<CssSupportsCondition, ParseError<'i, Error>> {
    let position = first_non_trivia_position(input);
    if input
        .try_parse(|input| input.expect_ident_matching("not"))
        .is_ok()
    {
        let operand = parse_condition_operand(source, input, diagnostics, recovery)?;
        return Ok(CssSupportsCondition::new(
            CssSupportsConditionKind::Not(Box::new(operand)),
            position,
        ));
    }

    let first = parse_condition_operand(source, input, diagnostics, recovery)?;
    if input.is_exhausted() {
        return Ok(first);
    }

    let operator = input.expect_ident_cloned().map_err(basic)?;
    let is_and = if operator.eq_ignore_ascii_case("and") {
        true
    } else if operator.eq_ignore_ascii_case("or") {
        false
    } else {
        return Err(invalid_syntax(
            input.current_source_location(),
            "expected `and` or `or` between supports conditions",
        ));
    };

    let mut conditions = vec![first];
    loop {
        conditions.push(parse_condition_operand(
            source,
            input,
            diagnostics,
            recovery,
        )?);
        if input.is_exhausted() {
            break;
        }
        let next = input.expect_ident_cloned().map_err(basic)?;
        if (is_and && !next.eq_ignore_ascii_case("and"))
            || (!is_and && !next.eq_ignore_ascii_case("or"))
        {
            return Err(invalid_syntax(
                input.current_source_location(),
                "supports conditions cannot mix `and` and `or` without grouping",
            ));
        }
    }

    let list = CssSupportsConditionList::new(conditions);
    Ok(CssSupportsCondition::new(
        if is_and {
            CssSupportsConditionKind::And(list)
        } else {
            CssSupportsConditionKind::Or(list)
        },
        position,
    ))
}

fn parse_condition_operand<'i, 't>(
    source: &'i str,
    input: &mut Parser<'i, 't>,
    diagnostics: &mut Vec<crate::CssRecoveryDiagnostic>,
    recovery: &RecoveryState,
) -> Result<CssSupportsCondition, ParseError<'i, Error>> {
    let (start, token) = next_non_trivia(input)?;
    let position =
        crate::CssSourcePosition::from_cssparser(start.position(), start.source_location());
    match token {
        Token::ParenthesisBlock => {
            let parsed = input.parse_nested_block(|nested| {
                if let Ok(declaration) = nested.try_parse(parse_supports_declaration) {
                    nested.expect_exhausted().map_err(basic)?;
                    return Ok(ParenthesizedCondition::Parsed(
                        CssSupportsConditionKind::Declaration(Box::new(declaration)),
                    ));
                }
                if let Ok(grouped) = nested.try_parse(|nested| {
                    let grouped = parse_condition(source, nested, diagnostics, recovery)?;
                    nested.expect_exhausted().map_err(basic)?;
                    Ok::<_, ParseError<'i, Error>>(grouped)
                }) {
                    return Ok(ParenthesizedCondition::Parsed(grouped.into_kind()));
                }
                consume_all(nested);
                Ok(ParenthesizedCondition::GeneralEnclosed)
            })?;
            let authored = input.slice_from(start.position()).to_owned();
            let kind = match parsed {
                ParenthesizedCondition::GeneralEnclosed => {
                    CssSupportsConditionKind::GeneralEnclosed(CssGeneralEnclosed::new(
                        authored, position,
                    ))
                }
                ParenthesizedCondition::Parsed(kind) => kind,
            };
            Ok(CssSupportsCondition::new(kind, position))
        }
        Token::Function(name) => {
            let is_selector = name.eq_ignore_ascii_case("selector");
            let parsed = input.parse_nested_block(|nested| {
                if is_selector {
                    let mut local_diagnostics = Vec::new();
                    if let Ok(selector) = nested.try_parse(|nested| {
                        let mut selector_recovery =
                            SelectorRecovery::new(source, &mut local_diagnostics, recovery.clone());
                        let selector = parse_rule_selector(nested, &mut selector_recovery)?;
                        nested.expect_exhausted().map_err(selector_basic)?;
                        Ok::<_, ParseError<'i, Error>>(selector)
                    }) && local_diagnostics.is_empty()
                    {
                        return Ok(Some(selector));
                    }
                }
                consume_all(nested);
                Ok(None)
            })?;
            if let Some(selector) = parsed {
                Ok(CssSupportsCondition::new(
                    CssSupportsConditionKind::Selector(selector),
                    position,
                ))
            } else {
                Ok(CssSupportsCondition::new(
                    CssSupportsConditionKind::GeneralEnclosed(CssGeneralEnclosed::new(
                        input.slice_from(start.position()),
                        position,
                    )),
                    position,
                ))
            }
        }
        _ => Err(invalid_syntax(
            start.source_location(),
            "expected a parenthesized or functional supports condition",
        )),
    }
}

pub(super) fn parse_supports_declaration<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> Result<CssSupportsDeclaration, ParseError<'i, Error>> {
    let start = input.state();
    let (name_start, token) = next_non_trivia(input)?;
    if !matches!(token, Token::Ident(_)) {
        return Err(invalid_syntax(
            name_start.source_location(),
            "expected a declaration property name",
        ));
    }
    let name_end = input.position();
    let property = input.slice(name_start.position()..name_end).to_owned();
    input.expect_colon().map_err(basic)?;
    consume_all(input);
    let authored = input.slice_from(start.position()).to_owned();
    let (known, parsed_importance) = parse_known_declaration(&authored);
    let importance = parsed_importance.unwrap_or_else(|| authored_importance(&authored));
    Ok(CssSupportsDeclaration::new(
        authored,
        property,
        importance,
        known,
        crate::CssSourcePosition::from_cssparser(
            name_start.position(),
            name_start.source_location(),
        ),
    ))
}

fn parse_known_declaration(authored: &str) -> (Option<CssKnownDeclaration>, Option<CssImportance>) {
    let mut input = ParserInput::new(authored);
    let mut parser = Parser::new(&mut input);
    let start = parser.state();
    let Ok(name) = parser.expect_ident_cloned() else {
        return (None, None);
    };
    if parser.expect_colon().is_err() {
        return (None, None);
    }
    let Ok(parsed) = parse_declaration_core(DeclarationMode::Ordinary, name, &mut parser, &start)
    else {
        return (None, None);
    };
    if !parser.is_exhausted() {
        return (None, None);
    }
    let known = match parsed.body {
        CssDeclarationBody::Known(known) => Some(known),
        CssDeclarationBody::Custom(_) => None,
    };
    (known, Some(parsed.importance))
}

fn authored_importance(authored: &str) -> CssImportance {
    let mut declaration_input = ParserInput::new(authored);
    let mut declaration = Parser::new(&mut declaration_input);
    if declaration.expect_ident_cloned().is_err() || declaration.expect_colon().is_err() {
        return CssImportance::Normal;
    }
    let value = declaration.slice_from(declaration.position()).to_owned();
    let mut input = ParserInput::new(&value);
    let mut parser = Parser::new(&mut input);
    let mut previous_was_bang = false;
    let mut terminal_important = false;
    while let Ok(token) = parser.next_including_whitespace_and_comments() {
        match token {
            Token::WhiteSpace(_) | Token::Comment(_) => {}
            Token::Ident(name) if previous_was_bang && name.eq_ignore_ascii_case("important") => {
                previous_was_bang = false;
                terminal_important = true;
            }
            Token::Delim('!') => {
                previous_was_bang = true;
                terminal_important = false;
            }
            _ => {
                previous_was_bang = false;
                terminal_important = false;
            }
        }
    }
    if terminal_important {
        CssImportance::Important
    } else {
        CssImportance::Normal
    }
}

fn first_non_trivia_position(input: &mut Parser<'_, '_>) -> crate::CssSourcePosition {
    let initial = input.state();
    let position = loop {
        let token_start = input.state();
        match input.next_including_whitespace_and_comments() {
            Ok(Token::WhiteSpace(_) | Token::Comment(_)) => {}
            Ok(_) => {
                break crate::CssSourcePosition::from_cssparser(
                    token_start.position(),
                    token_start.source_location(),
                );
            }
            Err(_) => {
                break crate::CssSourcePosition::from_cssparser(
                    input.position(),
                    input.current_source_location(),
                );
            }
        }
    };
    input.reset(&initial);
    position
}

fn next_non_trivia<'i>(
    input: &mut Parser<'i, '_>,
) -> Result<(ParserState, Token<'i>), ParseError<'i, Error>> {
    loop {
        let start = input.state();
        match input.next_including_whitespace_and_comments() {
            Ok(Token::WhiteSpace(_) | Token::Comment(_)) => {}
            Ok(token) => return Ok((start, token.clone())),
            Err(error) => return Err(basic(error)),
        }
    }
}

fn consume_all(input: &mut Parser<'_, '_>) {
    while input.next_including_whitespace_and_comments().is_ok() {}
}
