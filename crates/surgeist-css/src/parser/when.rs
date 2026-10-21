//! One adopted grammar for parsed and checked generalized conditions.
use super::boolean_condition::{
    BooleanConditionGrammar, ConditionFailure, parse_boolean_condition,
};
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
    match nested_condition(&lexical, limits, context, authored) {
        Ok(condition) => Ok(condition),
        Err(ConditionFailure::Mismatch(index)) => Err(invalid(&lexical, index)),
        Err(ConditionFailure::Fatal(error)) => Err(error),
    }
}
fn nested_condition(
    lexical: &SupportsLexical,
    limits: CssComponentValueLimits,
    context: CssParserContext,
    authored: bool,
) -> Result<CssWhenCondition, ConditionFailure<CssWhenConstructionError>> {
    parse_boolean_condition(
        &mut WhenGrammar {
            lexical,
            cursor: ComponentCursor::new(lexical.items()),
            limits,
            context,
            authored,
        },
        true,
    )
}
// Conditional 5 §3 `<boolean-condition>` over one lexical list, whose
// operands are `<boolean-condition-in-parens>` terms.
struct WhenGrammar<'a> {
    lexical: &'a SupportsLexical,
    cursor: ComponentCursor<'a>,
    limits: CssComponentValueLimits,
    context: CssParserContext,
    authored: bool,
}
impl BooleanConditionGrammar for WhenGrammar<'_> {
    type Operator = ();
    type Node = CssWhenCondition;
    type Error = ConditionFailure<CssWhenConstructionError>;

    fn keyword(&mut self, keyword: &'static str) -> Result<Option<()>, Self::Error> {
        Ok(self.cursor.ident(keyword).then_some(()))
    }
    fn operand(&mut self) -> Result<CssWhenCondition, Self::Error> {
        term(
            self.lexical,
            &mut self.cursor,
            self.limits,
            self.context,
            self.authored,
        )
    }
    fn finish(&mut self) -> Result<(), Self::Error> {
        if self.cursor.done() {
            Ok(())
        } else {
            Err(ConditionFailure::Mismatch(self.cursor.index))
        }
    }
    fn negation(&mut self, (): (), operand: CssWhenCondition) -> CssWhenCondition {
        CssWhenCondition::new(
            CssWhenConditionKind::Not(Box::new(operand)),
            self.lexical.clone(),
        )
    }
    fn single(&mut self, operand: CssWhenCondition) -> CssWhenCondition {
        CssWhenCondition::new(operand.into_kind(), self.lexical.clone())
    }
    fn junction(
        &mut self,
        conjunction: bool,
        _: Vec<()>,
        operands: Vec<CssWhenCondition>,
    ) -> CssWhenCondition {
        let list = CssWhenConditionList::new(operands);
        CssWhenCondition::new(
            if conjunction {
                CssWhenConditionKind::And(list)
            } else {
                CssWhenConditionKind::Or(list)
            },
            self.lexical.clone(),
        )
    }
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
) -> Result<CssWhenCondition, ConditionFailure<CssWhenConstructionError>> {
    cursor.peek();
    let index = cursor.index;
    let component = cursor.next().ok_or(ConditionFailure::Mismatch(index))?;
    let region = lexical.select(index..index + 1);
    let kind = match component.view() {
        CssComponentValueRef::Block(block) if block.kind() == CssBlockKind::Parenthesis => {
            match nested_condition(&lexical.children(index), limits, context, authored) {
                Ok(group) => {
                    return Ok(CssWhenCondition::new(
                        CssWhenConditionKind::Parenthesized(Box::new(group)),
                        region,
                    ));
                }
                Err(ConditionFailure::Mismatch(_)) => {}
                Err(failure @ ConditionFailure::Fatal(_)) => return Err(failure),
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
                    Err(CssSupportsConstructionError::Component(error)) => {
                        return Err(ConditionFailure::Fatal(error.into()));
                    }
                    Err(error) => {
                        return Err(ConditionFailure::Fatal(
                            CssWhenConstructionError::RecoveredInput {
                                origin: error.origin().clone(),
                            },
                        ));
                    }
                }
            }
            CssWhenConditionKind::GeneralEnclosed(
                CssGeneralEnclosed::try_from_component(component.clone())
                    .expect("validated function"),
            )
        }
        _ => return Err(ConditionFailure::Mismatch(index)),
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
