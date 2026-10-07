//! One adopted grammar for parsed and checked generalized conditions.
use super::query_components::ComponentCursor;
use super::recovery::RecoveryState;
use crate::supports::SupportsLexical;
use crate::*;
use cssparser::{ParseError, Parser};

pub(super) static IMPLEMENTED_RULES: &[CssFeatureId] = &[
    CssFeatureId::new("ext.rule.when"),
    CssFeatureId::new("ext.rule.else"),
];
pub(crate) fn construct_when_condition(
    values: CssComponentValues,
    limits: CssComponentValueLimits,
    context: CssParserContext,
) -> Result<CssWhenCondition, CssWhenConstructionError> {
    condition(SupportsLexical::root(values), limits, context, false)
}
pub(super) fn parse_prelude<'i>(
    source: &'i str,
    input: &mut Parser<'i, '_>,
    recovery: &RecoveryState,
    name: &str,
    optional: bool,
) -> Result<(Option<CssWhenCondition>, Vec<usize>), ParseError<'i, Error>> {
    let production = if optional {
        "ext.rule.else"
    } else {
        "ext.rule.when"
    };
    let start = input.position().byte_index();
    let implicit = recovery.check_specialized_components(source, input, production)?;
    let values = CssComponentValues::collect_from_parser(input, recovery.source_snapshot())
        .map_err(|error| {
            crate::error::invalid_component_value(input.current_source_location(), error)
        })?;
    if optional && values.items().iter().all(crate::supports::trivia) {
        return Ok((None, implicit));
    }
    let parsed = condition(
        SupportsLexical::root(values),
        CssComponentValueLimits::default(),
        recovery.parser_context(),
        true,
    )
    .map_err(|error| prelude_error(source, name, start, error))?;
    Ok((Some(parsed), implicit))
}
fn prelude_error<'i>(
    source: &'i str,
    name: &str,
    fallback: usize,
    error: CssWhenConstructionError,
) -> ParseError<'i, Error> {
    let position = crate::media::parsed_position(error.origin()).unwrap_or_else(|| {
        let mut offset = fallback;
        while let Some((start, end, token)) = super::recovery::next_source_token(source, offset) {
            if !matches!(
                token,
                cssparser::Token::WhiteSpace(_) | cssparser::Token::Comment(_)
            ) {
                return CssSourcePosition::from_byte_offset_in(source, start);
            }
            offset = end;
        }
        CssSourcePosition::from_byte_offset_in(source, source.len())
    });
    let location = cssparser::SourceLocation {
        line: position.line().value(),
        column: position.column().value() + 1,
    };
    match error {
        CssWhenConstructionError::Component(error) => {
            crate::error::invalid_component_value(location, error)
        }
        _ => {
            let token = super::recovery::next_source_token(source, position.byte_offset().value())
                .map(|(_, _, token)| token);
            crate::error::invalid_when_prelude(position, name, token.as_ref())
        }
    }
}
fn invalid(lexical: &SupportsLexical, index: usize) -> CssWhenConstructionError {
    let origin = lexical
        .items()
        .get(index)
        .or_else(|| {
            lexical
                .items()
                .iter()
                .rev()
                .find(|value| !crate::supports::trivia(value))
        })
        .map_or(CssValueOrigin::Programmatic, |value| value.origin().clone());
    CssWhenConstructionError::InvalidConditionGrammar { origin }
}
fn condition(
    lexical: SupportsLexical,
    limits: CssComponentValueLimits,
    context: CssParserContext,
    authored: bool,
) -> Result<CssWhenCondition, CssWhenConstructionError> {
    let mut cursor = ComponentCursor::new(lexical.items());
    if cursor.ident("not") {
        let term = term(&lexical, &mut cursor, limits, context, authored)?;
        if !cursor.done() {
            return Err(invalid(&lexical, cursor.index));
        }
        return Ok(CssWhenCondition::new(
            CssWhenConditionKind::Not(Box::new(term)),
            lexical,
        ));
    }
    let first = term(&lexical, &mut cursor, limits, context, authored)?;
    let and = if cursor.ident("and") {
        true
    } else if cursor.ident("or") {
        false
    } else {
        if !cursor.done() {
            return Err(invalid(&lexical, cursor.index));
        }
        return Ok(CssWhenCondition::new(first.into_kind(), lexical));
    };
    let mut terms = vec![first];
    loop {
        terms.push(term(&lexical, &mut cursor, limits, context, authored)?);
        if !cursor.ident(if and { "and" } else { "or" }) {
            break;
        }
    }
    if !cursor.done() {
        return Err(invalid(&lexical, cursor.index));
    }
    let list = CssWhenConditionList::new(terms);
    Ok(CssWhenCondition::new(
        if and {
            CssWhenConditionKind::And(list)
        } else {
            CssWhenConditionKind::Or(list)
        },
        lexical,
    ))
}
fn grammar<T>(
    result: Result<T, CssWhenConstructionError>,
) -> Result<Option<T>, CssWhenConstructionError> {
    match result {
        Ok(value) => Ok(Some(value)),
        Err(
            CssWhenConstructionError::InvalidConditionGrammar { .. }
            | CssWhenConstructionError::InvalidMediaFeatureGrammar { .. },
        ) => Ok(None),
        Err(error) => Err(error),
    }
}
fn term(
    lexical: &SupportsLexical,
    cursor: &mut ComponentCursor<'_>,
    limits: CssComponentValueLimits,
    context: CssParserContext,
    authored: bool,
) -> Result<CssWhenCondition, CssWhenConstructionError> {
    cursor.peek();
    let index = cursor.index;
    let component = cursor.next().ok_or_else(|| invalid(lexical, index))?;
    let region = lexical.select(index..index + 1);
    let kind = match component.view() {
        CssComponentValueRef::Block(block) if block.kind() == CssBlockKind::Parenthesis => {
            if let Some(group) = grammar(condition(
                lexical.children(index),
                limits,
                context,
                authored,
            ))? {
                return Ok(CssWhenCondition::new(
                    CssWhenConditionKind::Parenthesized(Box::new(group)),
                    region,
                ));
            }
            CssWhenConditionKind::GeneralEnclosed(
                CssGeneralEnclosed::try_from_component(component.clone())
                    .expect("validated enclosure"),
            )
        }
        CssComponentValueRef::Function(function) => {
            if function.name().eq_ignore_ascii_case("media") {
                if let Some(feature) = grammar(super::queries::construct_when_media_feature(
                    component.clone(),
                    limits,
                ))? {
                    return Ok(CssWhenCondition::new(
                        CssWhenConditionKind::MediaFeature(feature),
                        region,
                    ));
                }
            } else if function.name().eq_ignore_ascii_case("supports") {
                match super::supports::declaration(
                    lexical.children(index),
                    authored,
                    limits,
                    context,
                ) {
                    Ok(declaration) => {
                        return Ok(CssWhenCondition::new(
                            CssWhenConditionKind::SupportsDeclaration(Box::new(declaration)),
                            region,
                        ));
                    }
                    Err(
                        CssSupportsConstructionError::InvalidDeclarationGrammar { .. }
                        | CssSupportsConstructionError::InvalidConditionGrammar { .. },
                    ) => {}
                    Err(CssSupportsConstructionError::Component(error)) => return Err(error.into()),
                    Err(error) => {
                        return Err(CssWhenConstructionError::RecoveredInput {
                            origin: error.origin().clone(),
                        });
                    }
                }
            }
            CssWhenConditionKind::GeneralEnclosed(
                CssGeneralEnclosed::try_from_component(component.clone())
                    .expect("validated function"),
            )
        }
        _ => return Err(invalid(lexical, index)),
    };
    Ok(CssWhenCondition::new(kind, region))
}

// Keep whole-rule/vector temporaries out of the shared recursive block frame.
// Assembly occurs after child recursion returns, so the helper is never an
// additional live frame along a nested conditional chain.
#[inline(never)]
pub(super) fn assemble_when_rule(
    condition: CssWhenCondition,
    rules: Vec<CssRule>,
    position: CssSourcePosition,
) -> Vec<CssRule> {
    vec![CssRule::When(CssWhenRule::new(condition, rules, position))]
}
#[inline(never)]
pub(super) fn assemble_else_rule(
    condition: Option<CssWhenCondition>,
    rules: Vec<CssRule>,
    position: CssSourcePosition,
) -> Vec<CssRule> {
    vec![CssRule::Else(CssElseRule::new(condition, rules, position))]
}
