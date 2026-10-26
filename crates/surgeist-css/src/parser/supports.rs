use super::boolean_condition::{
    BooleanConditionGrammar, ConditionFailure, parse_boolean_condition,
};
use super::recovery::{RecoveryState, StyleContextCaptures};
use super::selectors::{SelectorRecovery, parse_rule_selector};
use crate::error::{
    CssError, invalid_syntax, is_nesting_limit_error, with_at_rule_prelude_context,
};
use crate::supports::{SupportsLexical, trivia};
use crate::*;
use cssparser::{ParseError, Parser};

pub(super) static IMPLEMENTED_SHARED_VALUES: &[crate::CssFeatureId] = &[
    crate::CssFeatureId::new("ext.supports.general-enclosed"),
    crate::CssFeatureId::new("ext.supports.named"),
    crate::CssFeatureId::new("ext.supports.font-tech"),
    crate::CssFeatureId::new("ext.supports.font-format"),
    crate::CssFeatureId::new("ext.supports.at-rule"),
];

pub(super) static IMPLEMENTED_SELECTORS: &[crate::CssFeatureId] =
    &[crate::CssFeatureId::new("ext.supports.selector")];

fn terminal_error(error: &ParseError<'_, CssError>) -> bool {
    is_nesting_limit_error(error)
        || matches!(&error.kind, cssparser::ParseErrorKind::Custom(error)
            if matches!(error.kind(), crate::ErrorKind::InvalidComponentValue(_)))
}

/// Only a grammar mismatch permits another supports interpretation. A lexical
/// or resource failure cannot become valid by selecting an opaque fallback.
pub(super) fn grammar_probe<'i, T>(
    result: Result<T, ParseError<'i, CssError>>,
) -> Result<Option<T>, ParseError<'i, CssError>> {
    match result {
        Ok(value) => Ok(Some(value)),
        Err(error) if terminal_error(&error) => Err(error),
        Err(_) => Ok(None),
    }
}

pub(super) fn with_supports_prelude_context<'i>(
    error: ParseError<'i, CssError>,
) -> ParseError<'i, CssError> {
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
) -> Result<CssCondition<CssSupportsLeaf>, ParseError<'i, CssError>> {
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
    parser_context: crate::CssParserContext,
) -> Result<CssSupportsDeclaration, ParseError<'i, CssError>> {
    let start = input.current_source_location();
    let values = CssComponentValues::collect_from_parser(input, source)
        .map_err(|e| crate::error::invalid_component_value(start, e))?;
    declaration(
        SupportsLexical::root(values),
        true,
        CssComponentValueLimits::default(),
        parser_context,
    )
    .map_err(|e| parser_error(e, start))
}
fn parser_error<'i>(
    error: CssSupportsConstructionError,
    fallback: cssparser::SourceLocation,
) -> ParseError<'i, CssError> {
    let location = crate::media::parsed_position(error.origin()).map_or(fallback, |p| {
        cssparser::SourceLocation {
            line: p.line().value(),
            column: p.column().value() + 1,
        }
    });
    match error.into_kind() {
        crate::supports::SupportsConstructionKind::Component(e) => {
            crate::error::invalid_component_value(location, e)
        }
        _ => invalid_syntax(location),
    }
}
fn validate(
    values: &CssComponentValues,
    limits: CssComponentValueLimits,
) -> Result<(), CssSupportsConstructionError> {
    values.validate_with_limits(limits)?;
    if let Some(origin) = values.first_implicit_origin() {
        return Err(CssSupportsConstructionError::from(
            crate::supports::SupportsConstructionKind::RecoveredInput {
                origin: origin.clone(),
            },
        ));
    }
    Ok(())
}
pub(crate) fn construct_supports_condition(
    values: CssComponentValues,
    namespaces: &CssNamespaceContext,
    limits: CssComponentValueLimits,
) -> Result<CssCondition<CssSupportsLeaf>, CssSupportsConstructionError> {
    construct_supports_condition_with_context(
        values,
        namespaces,
        limits,
        crate::CssParserContext::default(),
    )
}

pub(crate) fn construct_supports_condition_with_context(
    values: CssComponentValues,
    namespaces: &CssNamespaceContext,
    limits: CssComponentValueLimits,
    parser_context: crate::CssParserContext,
) -> Result<CssCondition<CssSupportsLeaf>, CssSupportsConstructionError> {
    validate(&values, limits)?;
    let recovery = RecoveryState::at_depth("", 0, StyleContextCaptures::default())
        .with_parser_context(parser_context);
    if let Some(name) = &namespaces.0.default {
        recovery.activate_namespace(None, name.clone());
    }
    for (prefix, name) in &namespaces.0.named {
        recovery.activate_namespace(Some(prefix.clone()), name.clone());
    }
    let value = condition(SupportsLexical::root(values), &recovery, false, limits)?;
    value.serialize_tokens(limits)?;
    Ok(value)
}
pub(crate) fn construct_supports_declaration(
    values: CssComponentValues,
    limits: CssComponentValueLimits,
) -> Result<CssSupportsDeclaration, CssSupportsConstructionError> {
    construct_supports_declaration_with_context(values, limits, crate::CssParserContext::default())
}
pub(crate) fn construct_supports_declaration_with_context(
    values: CssComponentValues,
    limits: CssComponentValueLimits,
    context: crate::CssParserContext,
) -> Result<CssSupportsDeclaration, CssSupportsConstructionError> {
    validate(&values, limits)?;
    let value = declaration(SupportsLexical::root(values), false, limits, context)?;
    value.serialize_tokens(limits)?;
    Ok(value)
}
fn invalid_condition(origin: &CssValueOrigin) -> CssSupportsConstructionError {
    CssSupportsConstructionError::from(
        crate::supports::SupportsConstructionKind::InvalidConditionGrammar {
            origin: origin.clone(),
        },
    )
}
fn grammar<T>(
    result: Result<T, CssSupportsConstructionError>,
) -> Result<Option<T>, CssSupportsConstructionError> {
    match result {
        Ok(value) => Ok(Some(value)),
        Err(error)
            if error.is_invalid_condition_grammar() || error.is_invalid_declaration_grammar() =>
        {
            Ok(None)
        }
        Err(error) => Err(error),
    }
}
fn ident(value: &CssComponentValue) -> Option<&str> {
    match value.view() {
        CssComponentValueRef::Token(CssValueTokenRef::Ident(v)) => Some(v),
        _ => None,
    }
}
// Conditional 3 `<supports-condition>`. The operator shape (a lone `not`
// operand, an odd alternation and its first operator) is checked before any
// operand is admitted; the shared algebra then admits operands in order.
fn condition(
    lexical: SupportsLexical,
    recovery: &RecoveryState,
    authored: bool,
    limits: CssComponentValueLimits,
) -> Result<CssCondition<CssSupportsLeaf>, CssSupportsConstructionError> {
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
    let items = lexical.items();
    if ident(&items[first]).is_some_and(|v| v.eq_ignore_ascii_case("not")) {
        if significant.len() != 2 {
            return Err(invalid_condition(lexical.first_origin()));
        }
    } else if significant.len() > 1 {
        if significant.len() % 2 == 0 {
            return Err(invalid_condition(
                items[*significant.last().expect("nonempty")].origin(),
            ));
        }
        let operator = ident(&items[significant[1]])
            .ok_or_else(|| invalid_condition(items[significant[1]].origin()))?;
        if !operator.eq_ignore_ascii_case("and") && !operator.eq_ignore_ascii_case("or") {
            return Err(invalid_condition(items[significant[1]].origin()));
        }
    }
    let mut grammar = SupportsGrammar {
        lexical: &lexical,
        significant: &significant,
        slot: 0,
        recovery,
        authored,
        limits,
    };
    match parse_boolean_condition(&mut grammar, true) {
        Ok(condition) => Ok(condition),
        Err(ConditionFailure::Mismatch(index)) => Err(invalid_condition(items[index].origin())),
        Err(ConditionFailure::Fatal(error)) => Err(error),
    }
}
struct SupportsGrammar<'a> {
    lexical: &'a SupportsLexical,
    significant: &'a [usize],
    slot: usize,
    recovery: &'a RecoveryState,
    authored: bool,
    limits: CssComponentValueLimits,
}
impl BooleanConditionGrammar for SupportsGrammar<'_> {
    type Operator = ();
    type Node = CssCondition<CssSupportsLeaf>;
    type Error = ConditionFailure<CssSupportsConstructionError>;

    fn keyword(&mut self, keyword: &'static str) -> Result<Option<()>, Self::Error> {
        let matched = self.significant.get(self.slot).is_some_and(|&index| {
            ident(&self.lexical.items()[index]).is_some_and(|v| v.eq_ignore_ascii_case(keyword))
        });
        if matched {
            self.slot += 1;
        }
        Ok(matched.then_some(()))
    }
    fn operand(&mut self) -> Result<CssCondition<CssSupportsLeaf>, Self::Error> {
        let index = self.significant[self.slot];
        self.slot += 1;
        Ok(operand(
            self.lexical.select(index..index + 1),
            self.recovery,
            self.authored,
            self.limits,
        )?)
    }
    fn finish(&mut self) -> Result<(), Self::Error> {
        match self.significant.get(self.slot) {
            None => Ok(()),
            Some(&index) => Err(ConditionFailure::Mismatch(index)),
        }
    }
    fn negation(
        &mut self,
        (): (),
        operand: CssCondition<CssSupportsLeaf>,
    ) -> CssCondition<CssSupportsLeaf> {
        CssCondition::lexical(
            CssConditionKind::Not(Box::new(operand)),
            self.lexical.clone(),
        )
    }
    fn single(&mut self, operand: CssCondition<CssSupportsLeaf>) -> CssCondition<CssSupportsLeaf> {
        CssCondition::lexical(operand.into_kind(), self.lexical.clone())
    }
    fn junction(
        &mut self,
        conjunction: bool,
        _: Vec<()>,
        operands: Vec<CssCondition<CssSupportsLeaf>>,
    ) -> CssCondition<CssSupportsLeaf> {
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
fn operand(
    lexical: SupportsLexical,
    recovery: &RecoveryState,
    authored: bool,
    limits: CssComponentValueLimits,
) -> Result<CssCondition<CssSupportsLeaf>, CssSupportsConstructionError> {
    let value = &lexical.items()[0];
    let kind = match value.view() {
        CssComponentValueRef::Block(block) if block.kind() == CssBlockKind::Parenthesis => {
            let children = lexical.children(0);
            let named = children
                .items()
                .iter()
                .filter(|value| !trivia(value))
                .collect::<Vec<_>>();
            if let [component] = named.as_slice()
                && let Ok(name) = CssSupportsConditionName::try_from_component((*component).clone())
            {
                CssConditionKind::Leaf(CssSupportsLeaf::Named(name))
            } else if let Some(declaration) = grammar(declaration(
                children.clone(),
                authored,
                limits,
                recovery.parser_context(),
            ))? {
                CssConditionKind::Leaf(CssSupportsLeaf::Declaration(Box::new(declaration)))
            } else if let Some(group) = grammar(condition(children, recovery, authored, limits))? {
                CssConditionKind::Parenthesized(Box::new(group))
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
                let working_source = crate::tokenization::prepare(source);
                let mut parser_input = cssparser::ParserInput::new(&working_source);
                let mut input = Parser::new(&mut parser_input);
                let mut diagnostics = Vec::new();
                let mut selector_recovery = SelectorRecovery::new(
                    source,
                    &mut diagnostics,
                    recovery.generated_selector_probe(source),
                );
                let parsed = parse_rule_selector(&mut input, &mut selector_recovery);
                match parsed {
                    Ok(selector) if input.is_exhausted() && diagnostics.is_empty() => {
                        CssConditionKind::Leaf(CssSupportsLeaf::Selector(selector))
                    }
                    Err(error) if terminal_error(&error) => {
                        let kind = match &error.kind {
                            cssparser::ParseErrorKind::Custom(error) => match error.kind() {
                                ErrorKind::InvalidComponentValue(component) => component.kind,
                                _ => CssComponentValueErrorKind::NestingLimit,
                            },
                            _ => CssComponentValueErrorKind::NestingLimit,
                        };
                        let position = crate::error::from_parse_error(source, error).position();
                        let origin = serialized
                            .value_origin_at(position.byte_offset().value())
                            .unwrap_or(value.origin())
                            .clone();
                        return Err(CssSupportsConstructionError::from(
                            crate::supports::SupportsConstructionKind::Component(
                                CssComponentValueError::new(kind, origin),
                            ),
                        ));
                    }
                    _ => enclosed(value)?,
                }
            } else if let Some(predicate) = function_predicate(function) {
                CssConditionKind::Leaf(predicate)
            } else {
                enclosed(value)?
            }
        }
        _ => return Err(invalid_condition(value.origin())),
    };
    Ok(CssCondition::lexical(kind, lexical))
}
// Keep leaf assembly outside the recursive operand frame. The lexical condition
// already owns validation, delimiters and output; these are transparent views.
#[inline(never)]
fn function_predicate(function: &CssFunctionValue) -> Option<CssSupportsLeaf> {
    let mut arguments = function
        .values()
        .items()
        .iter()
        .filter(|value| !trivia(value));
    let argument = arguments.next()?;
    if arguments.next().is_some() {
        return None;
    }
    if function.name().eq_ignore_ascii_case("font-tech") {
        return CssFontTechHint::from_ascii_name(ident(argument)?.as_bytes())
            .map(CssSupportsLeaf::FontTech);
    }
    if function.name().eq_ignore_ascii_case("font-format") {
        let format = match argument.view() {
            CssComponentValueRef::Token(CssValueTokenRef::Ident(name)) => {
                CssFontFormat::Keyword(CssFontFormatHint::from_ascii_name(name.as_bytes())?)
            }
            CssComponentValueRef::Token(CssValueTokenRef::String(value)) => {
                CssFontFormat::String(CssFontFormatString::new(value))
            }
            _ => return None,
        };
        return Some(CssSupportsLeaf::FontFormat(format));
    }
    if function.name().eq_ignore_ascii_case("at-rule") {
        return CssSupportsAtRule::from_component(argument.clone()).map(CssSupportsLeaf::AtRule);
    }
    None
}
fn enclosed(
    value: &CssComponentValue,
) -> Result<CssConditionKind<CssSupportsLeaf>, CssSupportsConstructionError> {
    Ok(CssConditionKind::Leaf(CssSupportsLeaf::GeneralEnclosed(
        CssGeneralEnclosed::try_from_component(value.clone()).expect("function or parenthesis"),
    )))
}
pub(super) fn declaration(
    lexical: SupportsLexical,
    authored: bool,
    limits: CssComponentValueLimits,
    parser_context: crate::CssParserContext,
) -> Result<CssSupportsDeclaration, CssSupportsConstructionError> {
    let invalid = || {
        CssSupportsConstructionError::from(
            crate::supports::SupportsConstructionKind::InvalidDeclarationGrammar {
                origin: lexical.first_origin().clone(),
            },
        )
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
    let admitted = if authored {
        parsed_admitted(name, &items[colon + 1..value_end], parser_context)?
    } else if parser_context.selects_svg_glyph(name)
        || CssPropertyGrammar::from_name(name).is_some()
    {
        let values = &items[colon + 1..value_end];
        let depth = values
            .iter()
            .filter_map(CssComponentValue::child_values)
            .map(|children| children.nesting_depth() + 1)
            .max()
            .unwrap_or(0);
        let grammar = if parser_context.selects_svg_glyph(name) {
            super::PropertyValueGrammar::SvgGlyph(crate::svg_glyph::SvgGlyphAdmission::css(
                parser_context,
            ))
        } else {
            super::PropertyValueGrammar::Known(
                CssPropertyGrammar::from_name(name).expect("recognized grammar"),
            )
        };
        let checked =
            super::parse_property_value_slice_selected(grammar, values, depth, parser_context);
        match checked {
            Ok(
                body @ (CssDeclarationBody::Known(_)
                | CssDeclarationBody::SvgGlyphOrientationVertical(_)),
            ) => Some(body),
            Err(error) => {
                let component_kind = match error.component_error() {
                    Some(component) => Some(component.kind),
                    None if error.code() == Some(crate::CssErrorCode::NestingLimit) => {
                        Some(CssComponentValueErrorKind::NestingLimit)
                    }
                    None => None,
                };
                if let Some(kind) = component_kind {
                    let origin = match error.origin() {
                        CssSerializedOrigin::Token(origin)
                        | CssSerializedOrigin::End(Some(origin)) => origin,
                        CssSerializedOrigin::Separator { after, .. } => after,
                        CssSerializedOrigin::End(None) => CssValueOrigin::Programmatic,
                    };
                    return Err(CssSupportsConstructionError::from(
                        crate::supports::SupportsConstructionKind::Component(
                            CssComponentValueError::new(kind, origin),
                        ),
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
        admitted,
        lexical,
        property_index,
        colon + 1..value_end,
    ))
}

// Keep parser recovery policy and authored alias selection while retaining the
// original snapshot. Masking only preceding bytes preserves component offsets;
// ending the cursor at the original value boundary preserves EOF recovery.
fn parsed_admitted(
    name: &str,
    values: &[CssComponentValue],
    parser_context: crate::CssParserContext,
) -> Result<Option<CssDeclarationBody>, CssSupportsConstructionError> {
    let resolved = crate::properties::resolve_property_name(name);
    if resolved.is_none() && !parser_context.selects_svg_glyph(name) {
        return Ok(None);
    }
    let Some(first) = values.first().and_then(CssComponentValue::parsed_origin) else {
        return Ok(None);
    };
    let last = values
        .last()
        .and_then(CssComponentValue::parsed_origin)
        .expect("parsed lexical root");
    let start = first.span().start().byte_offset().value();
    let end = last.span().end().byte_offset().value();
    let document = crate::syntax_consumption::normalize(
        crate::syntax_consumption::SyntaxInput::Source(crate::syntax_consumption::SourceWindow {
            text: std::borrow::Cow::Borrowed(first.source().as_str()),
            range: start..end,
            original: first.source().clone(),
        }),
        crate::syntax_consumption::SyntaxInputLimits::default(),
        0,
    )?;
    let mut input = super::property_input::Parser::new(&document);
    let numeric = crate::numeric::NumericInputContext::parsed(first.source());
    let parsed = if parser_context.selects_svg_glyph(name) {
        super::parse_svg_glyph_declaration_body_from_property_input(
            &mut input,
            &numeric,
            crate::svg_glyph::SvgGlyphAdmission::css(parser_context),
        )
    } else {
        super::parse_known_declaration_body_from_property_input(
            resolved.expect("recognized known grammar"),
            &mut input,
            &numeric,
            parser_context,
        )
    };
    match parsed {
        Ok(
            body @ (CssDeclarationBody::Known(_)
            | CssDeclarationBody::SvgGlyphOrientationVertical(_)),
        ) if input.is_exhausted() => Ok(Some(body)),
        Err(error) if crate::error::property::is_resource_parse_error(&error) => {
            let (kind, origin) = error.into_parts();
            if let ErrorKind::InvalidComponentValue(component) = kind {
                return Err(CssSupportsConstructionError::from(
                    crate::supports::SupportsConstructionKind::Component(*component),
                ));
            }
            let origin = match origin {
                CssSerializedOrigin::Token(origin) | CssSerializedOrigin::End(Some(origin)) => {
                    origin
                }
                CssSerializedOrigin::Separator { after, .. } => after,
                CssSerializedOrigin::End(None) => values[0].origin().clone(),
            };
            Err(CssSupportsConstructionError::from(
                crate::supports::SupportsConstructionKind::Component(CssComponentValueError::new(
                    CssComponentValueErrorKind::NestingLimit,
                    origin,
                )),
            ))
        }
        _ => Ok(None),
    }
}

#[cfg(test)]
mod generated_selector_tests {
    use super::*;

    #[test]
    fn live_depth_limit_maps_generated_function_error_to_original_unicode_source() {
        let source = "/*😀*/\r\nselector(:is(:where(.A)))";
        let values = crate::parse_component_values(source).unwrap();
        let function = values
            .items()
            .iter()
            .find_map(|item| match item.view() {
                CssComponentValueRef::Function(function) => Some(function),
                _ => None,
            })
            .unwrap();
        let CssComponentValueRef::Function(is) = function.values().items()[1].view() else {
            panic!("Is")
        };
        let expected = is.values().items()[1].origin().clone();
        let state = RecoveryState::at_depth(source, 254, StyleContextCaptures::default());
        let value = condition(
            SupportsLexical::root(values.clone()),
            &state,
            false,
            CssComponentValueLimits::default(),
        )
        .unwrap();
        assert!(matches!(
            value.kind(),
            CssConditionKind::Leaf(CssSupportsLeaf::Selector(_))
        ));
        assert_eq!(state.structural_depth(), 254);
        let state = RecoveryState::at_depth(source, 255, StyleContextCaptures::default());
        let error = condition(
            SupportsLexical::root(values),
            &state,
            false,
            CssComponentValueLimits::default(),
        )
        .unwrap_err();
        let Some(error) = error.component_error() else {
            panic!("terminal resource failure")
        };
        assert_eq!(error.kind, CssComponentValueErrorKind::NestingLimit);
        assert_eq!(error.origin(), &expected);
        let crate::CssValueOrigin::Parsed(origin) = error.origin() else {
            panic!("original origin")
        };
        assert_eq!(origin.source().as_str(), source);
        assert_eq!(origin.span().start().byte_offset().value(), 24);
        assert_eq!(origin.span().start().line().value(), 1);
        assert_eq!(origin.span().start().column().value(), 14);
        assert_eq!(state.structural_depth(), 255);
    }
}
