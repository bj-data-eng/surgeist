//! One adopted grammar for parsed and checked generalized conditions.
use super::boolean_condition::{
    BooleanConditionGrammar, ConditionFailure, parse_boolean_condition,
};
use super::descriptor_stream::collect_native_scope;
use super::query_components::ComponentCursor;
use super::recovery::RecoveryState;
use crate::CssParserContext;
use crate::component::{
    CssBlockKind, CssComponentValueLimits, CssComponentValueRef, CssComponentValues, CssValueOrigin,
};
use crate::diagnostic::{CssError, CssFeatureId};
use crate::query::supports::construction::SupportsLexical;
use crate::query::{
    CssCondition, CssConditionConstructionError, CssConditionKind, CssConditionList,
    CssGeneralEnclosed, CssWhenLeaf,
};
use crate::rule::{CssElseRule, CssRule, CssWhenRule};
use crate::source::CssSourcePosition;
use cssparser::{ParseError, Parser};

pub(super) static IMPLEMENTED_RULES: &[CssFeatureId] = &[
    CssFeatureId::new("ext.rule.when"),
    CssFeatureId::new("ext.rule.else"),
];
pub(crate) fn construct_when_condition(
    values: CssComponentValues,
    limits: CssComponentValueLimits,
    context: CssParserContext,
) -> Result<CssCondition<CssWhenLeaf>, CssConditionConstructionError> {
    condition(SupportsLexical::root(values), limits, context, false)
}
/// A parsed `@when` or `@else` condition and the offsets of its implicit closures.
pub(super) type WhenPrelude = (Option<CssCondition<CssWhenLeaf>>, Vec<usize>);
pub(super) fn parse_prelude<'i>(
    source: &'i str,
    input: &mut Parser<'i, '_>,
    recovery: &RecoveryState,
    name: &str,
    optional: bool,
) -> Result<WhenPrelude, ParseError<'i, CssError>> {
    let production = if optional {
        "ext.rule.else"
    } else {
        "ext.rule.when"
    };
    let start = input.position().byte_index();
    let implicit = recovery.check_specialized_components(source, input, production)?;
    let values = collect_native_scope(input, recovery.source_snapshot())?
        .values
        .map_err(|(error, at)| crate::diagnostic::error::invalid_component_value(at, error))?;
    if optional
        && values
            .items()
            .iter()
            .all(crate::query::supports::construction::trivia)
    {
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
    error: CssConditionConstructionError,
) -> ParseError<'i, CssError> {
    let position = crate::query::media::construction::parsed_position(error.origin())
        .unwrap_or_else(|| {
            let mut offset = fallback;
            while let Some((start, end, token)) = super::recovery::next_source_token(source, offset)
            {
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
    match error.into_kind() {
        crate::query::when::ConditionConstructionKind::Component(error) => {
            crate::diagnostic::error::invalid_component_value(location, error)
        }
        _ => {
            let token = super::recovery::next_source_token(source, position.byte_offset().value())
                .map(|(_, _, token)| token);
            crate::diagnostic::error::invalid_when_prelude(position, name, token.as_ref())
        }
    }
}
fn invalid(lexical: &SupportsLexical, index: usize) -> CssConditionConstructionError {
    let origin = lexical
        .items()
        .get(index)
        .or_else(|| {
            lexical
                .items()
                .iter()
                .rev()
                .find(|value| !crate::query::supports::construction::trivia(value))
        })
        .map_or(CssValueOrigin::Programmatic, |value| value.origin().clone());
    CssConditionConstructionError::from(
        crate::query::when::ConditionConstructionKind::InvalidConditionGrammar { origin },
    )
}
fn condition(
    lexical: SupportsLexical,
    limits: CssComponentValueLimits,
    context: CssParserContext,
    authored: bool,
) -> Result<CssCondition<CssWhenLeaf>, CssConditionConstructionError> {
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
) -> Result<CssCondition<CssWhenLeaf>, ConditionFailure<CssConditionConstructionError>> {
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
    type Node = CssCondition<CssWhenLeaf>;
    type Error = ConditionFailure<CssConditionConstructionError>;

    fn keyword(&mut self, keyword: &'static str) -> Result<Option<()>, Self::Error> {
        Ok(self.cursor.ident(keyword).then_some(()))
    }
    fn operand(&mut self) -> Result<CssCondition<CssWhenLeaf>, Self::Error> {
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
    fn negation(
        &mut self,
        (): (),
        operand: CssCondition<CssWhenLeaf>,
    ) -> CssCondition<CssWhenLeaf> {
        CssCondition::lexical(
            CssConditionKind::Not(Box::new(operand)),
            self.lexical.clone(),
        )
    }
    fn single(&mut self, operand: CssCondition<CssWhenLeaf>) -> CssCondition<CssWhenLeaf> {
        CssCondition::lexical(operand.into_kind(), self.lexical.clone())
    }
    fn junction(
        &mut self,
        conjunction: bool,
        _: Vec<()>,
        operands: Vec<CssCondition<CssWhenLeaf>>,
    ) -> CssCondition<CssWhenLeaf> {
        let list = CssConditionList::new(operands);
        CssCondition::lexical(
            if conjunction {
                CssConditionKind::And(list)
            } else {
                CssConditionKind::Or(list)
            },
            self.lexical.clone(),
        )
    }
}
fn grammar<T>(
    result: Result<T, CssConditionConstructionError>,
) -> Result<Option<T>, CssConditionConstructionError> {
    match result {
        Ok(value) => Ok(Some(value)),
        Err(error)
            if error.is_invalid_condition_grammar() || error.is_invalid_media_feature_grammar() =>
        {
            Ok(None)
        }
        Err(error) => Err(error),
    }
}
fn term(
    lexical: &SupportsLexical,
    cursor: &mut ComponentCursor<'_>,
    limits: CssComponentValueLimits,
    context: CssParserContext,
    authored: bool,
) -> Result<CssCondition<CssWhenLeaf>, ConditionFailure<CssConditionConstructionError>> {
    cursor.peek();
    let index = cursor.index;
    let component = cursor.next().ok_or(ConditionFailure::Mismatch(index))?;
    let region = lexical.select(index..index + 1);
    let kind = match component.view() {
        CssComponentValueRef::Block(block) if block.kind() == CssBlockKind::Parenthesis => {
            match nested_condition(&lexical.children(index), limits, context, authored) {
                Ok(group) => {
                    return Ok(CssCondition::lexical(
                        CssConditionKind::Parenthesized(Box::new(group)),
                        region,
                    ));
                }
                Err(ConditionFailure::Mismatch(_)) => {}
                Err(failure @ ConditionFailure::Fatal(_)) => return Err(failure),
            }
            CssConditionKind::Leaf(CssWhenLeaf::GeneralEnclosed(
                CssGeneralEnclosed::try_from_component(component.clone())
                    .expect("validated enclosure"),
            ))
        }
        CssComponentValueRef::Function(function) => {
            if function.name().eq_ignore_ascii_case("media") {
                if let Some(feature) = grammar(super::queries::construct_when_media_feature(
                    component.clone(),
                    limits,
                ))? {
                    return Ok(CssCondition::lexical(
                        CssConditionKind::Leaf(CssWhenLeaf::MediaFeature(feature)),
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
                        return Ok(CssCondition::lexical(
                            CssConditionKind::Leaf(CssWhenLeaf::SupportsDeclaration(Box::new(
                                declaration,
                            ))),
                            region,
                        ));
                    }
                    Err(error)
                        if error.is_invalid_declaration_grammar()
                            || error.is_invalid_condition_grammar() => {}
                    Err(error) => {
                        return Err(ConditionFailure::Fatal(match error.component_error() {
                            Some(component) => component.clone().into(),
                            None => CssConditionConstructionError::from(
                                crate::query::when::ConditionConstructionKind::RecoveredInput {
                                    origin: error.origin().clone(),
                                },
                            ),
                        }));
                    }
                }
            }
            CssConditionKind::Leaf(CssWhenLeaf::GeneralEnclosed(
                CssGeneralEnclosed::try_from_component(component.clone())
                    .expect("validated function"),
            ))
        }
        _ => return Err(ConditionFailure::Mismatch(index)),
    };
    Ok(CssCondition::lexical(kind, region))
}

// Keep whole-rule/vector temporaries out of the shared recursive block frame.
// Assembly occurs after child recursion returns, so the helper is never an
// additional live frame along a nested conditional chain.
#[inline(never)]
pub(super) fn assemble_when_rule(
    condition: CssCondition<CssWhenLeaf>,
    rules: Vec<CssRule>,
    position: CssSourcePosition,
) -> Vec<CssRule> {
    vec![CssRule::When(CssWhenRule::new(condition, rules, position))]
}
#[inline(never)]
pub(super) fn assemble_else_rule(
    condition: Option<CssCondition<CssWhenLeaf>>,
    rules: Vec<CssRule>,
    position: CssSourcePosition,
) -> Vec<CssRule> {
    vec![CssRule::Else(CssElseRule::new(condition, rules, position))]
}
