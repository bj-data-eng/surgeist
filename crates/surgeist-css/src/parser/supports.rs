use super::recovery::{RecoveryState, StyleContextCaptures};
use super::selectors::{SelectorRecovery, parse_rule_selector};
use crate::error::{Error, invalid_syntax, is_nesting_limit_error, with_at_rule_prelude_context};
use crate::supports::{SupportsLexical, trivia};
use crate::*;
use cssparser::{ParseError, Parser};

pub(super) static IMPLEMENTED_SHARED_VALUES: &[crate::CssFeatureId] =
    &[crate::CssFeatureId::new("ext.supports.general-enclosed")];

pub(super) static IMPLEMENTED_SELECTORS: &[crate::CssFeatureId] =
    &[crate::CssFeatureId::new("ext.supports.selector")];

fn terminal_error(error: &ParseError<'_, Error>) -> bool {
    is_nesting_limit_error(error)
        || matches!(&error.kind, cssparser::ParseErrorKind::Custom(error)
            if matches!(error.kind(), crate::ErrorKind::InvalidComponentValue(_)))
}

/// Only a grammar mismatch permits another supports interpretation. A lexical
/// or resource failure cannot become valid by selecting an opaque fallback.
pub(super) fn grammar_probe<'i, T>(
    result: Result<T, ParseError<'i, Error>>,
) -> Result<Option<T>, ParseError<'i, Error>> {
    match result {
        Ok(value) => Ok(Some(value)),
        Err(error) if terminal_error(&error) => Err(error),
        Err(_) => Ok(None),
    }
}

pub(super) fn with_supports_prelude_context<'i>(
    error: ParseError<'i, Error>,
) -> ParseError<'i, Error> {
    if terminal_error(&error) {
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
    _diagnostics: &mut Vec<CssRecoveryDiagnostic>,
    recovery: &RecoveryState,
) -> Result<CssSupportsCondition, ParseError<'i, Error>> {
    let implicit =
        recovery.check_specialized_components(source, input, "baseline.rule.supports")?;
    let start = input.current_source_location();
    let values = CssComponentValues::collect_from_parser(input, recovery.source_snapshot())
        .map_err(|e| crate::error::invalid_component_value(start, e))?;
    let result = condition(
        SupportsLexical::root(values),
        recovery,
        true,
        CssComponentValueLimits::default(),
    )
    .map_err(|e| parser_error(e, start))?;
    recovery.retain_component_closures(implicit);
    Ok(result)
}
pub(super) fn parse_supports_declaration<'i, 't>(
    input: &mut Parser<'i, 't>,
    source: &CssSourceSnapshot,
) -> Result<CssSupportsDeclaration, ParseError<'i, Error>> {
    let start = input.current_source_location();
    let values = CssComponentValues::collect_from_parser(input, source)
        .map_err(|e| crate::error::invalid_component_value(start, e))?;
    declaration(
        SupportsLexical::root(values),
        true,
        CssComponentValueLimits::default(),
    )
    .map_err(|e| parser_error(e, start))
}
fn parser_error<'i>(
    error: CssSupportsConstructionError,
    fallback: cssparser::SourceLocation,
) -> ParseError<'i, Error> {
    let location = crate::media::parsed_position(error.origin()).map_or(fallback, |p| {
        cssparser::SourceLocation {
            line: p.line().value(),
            column: p.column().value() + 1,
        }
    });
    match error {
        CssSupportsConstructionError::Component(e) => {
            crate::error::invalid_component_value(location, e)
        }
        _ => invalid_syntax(
            location,
            "expected a valid supports condition or declaration",
        ),
    }
}
fn validate(
    values: &CssComponentValues,
    limits: CssComponentValueLimits,
) -> Result<(), CssSupportsConstructionError> {
    values.validate_with_limits(limits)?;
    if let Some(origin) = values.first_implicit_origin() {
        return Err(CssSupportsConstructionError::RecoveredInput {
            origin: origin.clone(),
        });
    }
    Ok(())
}
pub(crate) fn construct_supports_condition(
    values: CssComponentValues,
    namespaces: &CssNamespaceContext,
    limits: CssComponentValueLimits,
) -> Result<CssSupportsCondition, CssSupportsConstructionError> {
    validate(&values, limits)?;
    let recovery = RecoveryState::at_depth("", 0, StyleContextCaptures::default());
    if let Some(name) = &namespaces.0.default {
        recovery.activate_namespace(None, name.clone());
    }
    for (prefix, name) in &namespaces.0.named {
        recovery.activate_namespace(Some(prefix.clone()), name.clone());
    }
    let value = condition(SupportsLexical::root(values), &recovery, false, limits)?;
    value.serialize_with_limit(limits.max_css_bytes())?;
    Ok(value)
}
pub(crate) fn construct_supports_declaration(
    values: CssComponentValues,
    limits: CssComponentValueLimits,
) -> Result<CssSupportsDeclaration, CssSupportsConstructionError> {
    validate(&values, limits)?;
    let value = declaration(SupportsLexical::root(values), false, limits)?;
    value.serialize_with_limit(limits.max_css_bytes())?;
    Ok(value)
}
fn invalid_condition(origin: &CssValueOrigin) -> CssSupportsConstructionError {
    CssSupportsConstructionError::InvalidConditionGrammar {
        origin: origin.clone(),
    }
}
fn grammar<T>(
    result: Result<T, CssSupportsConstructionError>,
) -> Result<Option<T>, CssSupportsConstructionError> {
    match result {
        Ok(value) => Ok(Some(value)),
        Err(
            CssSupportsConstructionError::InvalidConditionGrammar { .. }
            | CssSupportsConstructionError::InvalidDeclarationGrammar { .. },
        ) => Ok(None),
        Err(error) => Err(error),
    }
}
fn ident(value: &CssComponentValue) -> Option<&str> {
    match value.view() {
        CssComponentValueRef::Token(CssValueTokenRef::Ident(v)) => Some(v),
        _ => None,
    }
}
fn condition(
    lexical: SupportsLexical,
    recovery: &RecoveryState,
    authored: bool,
    limits: CssComponentValueLimits,
) -> Result<CssSupportsCondition, CssSupportsConstructionError> {
    let significant: Vec<_> = lexical
        .items()
        .iter()
        .enumerate()
        .filter(|(_, v)| !trivia(v))
        .map(|(i, _)| i)
        .collect();
    let Some(&first) = significant.first() else {
        return Err(invalid_condition(lexical.first_origin()));
    };
    if ident(&lexical.items()[first]).is_some_and(|v| v.eq_ignore_ascii_case("not")) {
        if significant.len() != 2 {
            return Err(invalid_condition(lexical.first_origin()));
        }
        let operand = operand(
            lexical.select(significant[1]..significant[1] + 1),
            recovery,
            authored,
            limits,
        )?;
        return Ok(CssSupportsCondition::new(
            CssSupportsConditionKind::Not(Box::new(operand)),
            lexical,
            false,
        ));
    }
    if significant.len() == 1 {
        let parsed = operand(lexical.select(first..first + 1), recovery, authored, limits)?;
        return Ok(CssSupportsCondition::new(
            parsed.into_kind(),
            lexical,
            false,
        ));
    }
    if significant.len() % 2 == 0 {
        return Err(invalid_condition(
            lexical.items()[*significant.last().expect("nonempty")].origin(),
        ));
    }
    let operator = ident(&lexical.items()[significant[1]])
        .ok_or_else(|| invalid_condition(lexical.items()[significant[1]].origin()))?;
    let is_and = operator.eq_ignore_ascii_case("and");
    if !is_and && !operator.eq_ignore_ascii_case("or") {
        return Err(invalid_condition(lexical.items()[significant[1]].origin()));
    }
    let mut conditions = Vec::new();
    for (slot, &index) in significant.iter().enumerate() {
        if slot % 2 == 0 {
            conditions.push(operand(
                lexical.select(index..index + 1),
                recovery,
                authored,
                limits,
            )?);
        } else if !ident(&lexical.items()[index]).is_some_and(|v| v.eq_ignore_ascii_case(operator))
        {
            return Err(invalid_condition(lexical.items()[index].origin()));
        }
    }
    let list = CssSupportsConditionList::new(conditions);
    Ok(CssSupportsCondition::new(
        if is_and {
            CssSupportsConditionKind::And(list)
        } else {
            CssSupportsConditionKind::Or(list)
        },
        lexical,
        false,
    ))
}
fn operand(
    lexical: SupportsLexical,
    recovery: &RecoveryState,
    authored: bool,
    limits: CssComponentValueLimits,
) -> Result<CssSupportsCondition, CssSupportsConstructionError> {
    let value = &lexical.items()[0];
    let kind = match value.view() {
        CssComponentValueRef::Block(block) if block.kind() == CssBlockKind::Parenthesis => {
            let children = lexical.children(0);
            if let Some(declaration) = grammar(declaration(children.clone(), authored, limits))? {
                CssSupportsConditionKind::Declaration(Box::new(declaration))
            } else if let Some(group) = grammar(condition(children, recovery, authored, limits))? {
                group.into_kind()
            } else {
                enclosed(value)?
            }
        }
        CssComponentValueRef::Function(function) => {
            if function.name().eq_ignore_ascii_case("selector") {
                let mut out =
                    crate::component_values::CssCanonicalBuilder::new(limits.max_css_bytes());
                out.push_components(function.values().items())?;
                let serialized = out.finish()?;
                let source = serialized.as_css();
                let mut parser_input = cssparser::ParserInput::new(source);
                let mut input = Parser::new(&mut parser_input);
                let mut diagnostics = Vec::new();
                let mut selector_recovery =
                    SelectorRecovery::new(source, &mut diagnostics, recovery.clone());
                let parsed = parse_rule_selector(&mut input, &mut selector_recovery);
                match parsed {
                    Ok(selector) if input.is_exhausted() && diagnostics.is_empty() => {
                        CssSupportsConditionKind::Selector(selector)
                    }
                    Err(error) if terminal_error(&error) => {
                        let kind = match &error.kind {
                            cssparser::ParseErrorKind::Custom(error) => match error.kind() {
                                ErrorKind::InvalidComponentValue(component) => component.kind(),
                                _ => CssComponentValueErrorKind::NestingLimit,
                            },
                            _ => CssComponentValueErrorKind::NestingLimit,
                        };
                        let position = crate::error::from_parse_error(source, error).position();
                        let origin = serialized
                            .value_origin_at(position.byte_offset().value())
                            .unwrap_or(value.origin())
                            .clone();
                        return Err(CssSupportsConstructionError::Component(
                            CssComponentValueError::new(kind, origin),
                        ));
                    }
                    _ => enclosed(value)?,
                }
            } else {
                enclosed(value)?
            }
        }
        _ => return Err(invalid_condition(value.origin())),
    };
    Ok(CssSupportsCondition::new(kind, lexical, false))
}
fn enclosed(
    value: &CssComponentValue,
) -> Result<CssSupportsConditionKind, CssSupportsConstructionError> {
    Ok(CssSupportsConditionKind::GeneralEnclosed(
        CssGeneralEnclosed::try_from_component(value.clone()).expect("function or parenthesis"),
    ))
}
fn declaration(
    lexical: SupportsLexical,
    authored: bool,
    limits: CssComponentValueLimits,
) -> Result<CssSupportsDeclaration, CssSupportsConstructionError> {
    let invalid = || CssSupportsConstructionError::InvalidDeclarationGrammar {
        origin: lexical.first_origin().clone(),
    };
    let items = lexical.items();
    let significant: Vec<_> = items
        .iter()
        .enumerate()
        .filter(|(_, v)| !trivia(v))
        .map(|(i, _)| i)
        .collect();
    let Some(&property_index) = significant.first() else {
        return Err(invalid());
    };
    let name = ident(&items[property_index]).ok_or_else(invalid)?;
    let Some(&colon) = significant.get(1) else {
        return Err(invalid());
    };
    if !matches!(
        items[colon].view(),
        CssComponentValueRef::Token(CssValueTokenRef::Colon)
    ) {
        return Err(invalid());
    }
    let mut importance = CssImportance::Normal;
    let mut value_end = items.len();
    for (slot, &index) in significant.iter().enumerate().skip(2) {
        match items[index].view() {
            CssComponentValueRef::Token(CssValueTokenRef::Semicolon) => return Err(invalid()),
            CssComponentValueRef::Token(CssValueTokenRef::Delim('!')) => {
                if slot + 2 != significant.len()
                    || !ident(&items[significant[slot + 1]])
                        .is_some_and(|name| name.eq_ignore_ascii_case("important"))
                {
                    return Err(invalid());
                }
                importance = CssImportance::Important;
                value_end = index;
                break;
            }
            _ => {}
        }
    }
    let known = if authored {
        parsed_known(name, &items[colon + 1..value_end])?
    } else if let Some(grammar) = CssPropertyGrammar::from_name(name) {
        let values =
            CssComponentValues::try_new_with_limits(items[colon + 1..value_end].to_vec(), limits)?;
        match crate::property_value::checked_grammar_value_body(grammar, &values) {
            Ok(CssDeclarationBody::Known(known)) => Some(known),
            Err(error) => {
                let component_kind = match error.kind() {
                    CssPropertyValueErrorKind::Component(kind) => Some(*kind),
                    CssPropertyValueErrorKind::Grammar(ErrorKind::InvalidComponentValue(
                        component,
                    )) => Some(component.kind()),
                    CssPropertyValueErrorKind::Grammar(ErrorKind::NestingLimit(_)) => {
                        Some(CssComponentValueErrorKind::NestingLimit)
                    }
                    _ => None,
                };
                if let Some(kind) = component_kind {
                    let origin = match error.origin() {
                        CssSerializedOrigin::Token(origin)
                        | CssSerializedOrigin::End(Some(origin)) => origin.clone(),
                        CssSerializedOrigin::Separator { after, .. } => after.clone(),
                        CssSerializedOrigin::End(None) => CssValueOrigin::Programmatic,
                    };
                    return Err(CssSupportsConstructionError::Component(
                        CssComponentValueError::new(kind, origin),
                    ));
                }
                None
            }
            _ => None,
        }
    } else {
        None
    };
    let mut spelling = crate::component_values::CssCanonicalBuilder::new(limits.max_css_bytes());
    spelling.push_component(&items[property_index])?;
    let property = spelling.finish()?.as_css().to_owned();
    let authored = if authored {
        items
            .first()
            .and_then(CssComponentValue::parsed_origin)
            .zip(items.last().and_then(CssComponentValue::parsed_origin))
            .map(|(first, last)| {
                first.source().as_str()[first.span().start().byte_offset().value()
                    ..last.span().end().byte_offset().value()]
                    .to_owned()
            })
    } else {
        None
    };
    Ok(CssSupportsDeclaration::new(
        authored,
        property,
        importance,
        known,
        lexical,
        property_index,
        colon + 1..value_end,
    ))
}

// Keep parser recovery policy and authored alias selection while retaining the
// original snapshot. Masking only preceding bytes preserves component offsets;
// ending the cursor at the original value boundary preserves EOF recovery.
fn parsed_known(
    name: &str,
    values: &[CssComponentValue],
) -> Result<Option<CssKnownDeclaration>, CssSupportsConstructionError> {
    let Some(resolved) = crate::properties::resolve_property_name(name) else {
        return Ok(None);
    };
    let Some(first) = values.first().and_then(CssComponentValue::parsed_origin) else {
        return Ok(None);
    };
    let last = values
        .last()
        .and_then(CssComponentValue::parsed_origin)
        .expect("parsed lexical root");
    let start = first.span().start().byte_offset().value();
    let end = last.span().end().byte_offset().value();
    let source = super::isolate_source_span(&first.source().as_str()[..end], start, end);
    let mut parser_input = cssparser::ParserInput::new(&source);
    let mut input = Parser::new(&mut parser_input);
    let numeric = crate::numeric::NumericInputContext::parsed(first.source());
    match super::parse_known_declaration_body(resolved, &mut input, &numeric) {
        Ok(CssDeclarationBody::Known(known)) if input.is_exhausted() => Ok(Some(known)),
        Err(error) if terminal_error(&error) => {
            if let cssparser::ParseErrorKind::Custom(error) = error.kind
                && let ErrorKind::InvalidComponentValue(component) = error.kind()
            {
                return Err(CssSupportsConstructionError::Component(
                    component.as_ref().clone(),
                ));
            }
            Err(CssSupportsConstructionError::Component(
                CssComponentValueError::new(
                    CssComponentValueErrorKind::NestingLimit,
                    values[0].origin().clone(),
                ),
            ))
        }
        _ => Ok(None),
    }
}
