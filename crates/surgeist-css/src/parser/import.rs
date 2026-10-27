//! Import construction, optional-clause selection, and interpretation protection.

use super::property_input::NativeValueRange;
use super::recovery::{RecoveryState, StyleContextCaptures};
use super::supports::{parse_supports_condition, parse_supports_declaration};
use super::url::parse_url;
use super::{parse_layer_name, queries, supports};
use crate::component::CssComponentValues;
use crate::diagnostic::CssError;
use crate::diagnostic::error::{basic, from_parse_error, invalid_syntax};
use crate::query::condition::SupportsAuthoredForm;
use crate::query::media::{CssMediaQuery, CssMediaQueryList};
use crate::query::supports::CssSupportsConditionName;
use crate::query::supports::{CssSupportsDeclaration, CssSupportsLeaf};
use crate::query::{CssCondition, CssConditionKind};
use crate::rule::{
    CssImportLayer, CssImportRule, CssImportString, CssImportSupports, CssImportTarget,
    CssImportUrl, CssLayerName,
};
use crate::selector::CssNamespaceContext;
use crate::source::CssParsedOrigin;
use cssparser::{Delimiter, ParseError, Parser, ParserInput, ParserState};

pub(crate) struct CssImportPrelude {
    pub(super) target: CssImportTarget,
    pub(super) layer: Option<CssImportLayer>,
    pub(super) supports: Option<CssImportSupports>,
    pub(super) media: Option<CssMediaQueryList>,
    pub(super) implicit_media_closures: Vec<usize>,
    pub(super) diagnostics: Vec<crate::diagnostic::CssRecoveryDiagnostic>,
    pub(super) syntax: crate::rule::imports::ImportPreludeSyntax,
}

pub(crate) fn construct_import(
    values: crate::component::CssComponentValues,
    context: &CssNamespaceContext,
    limits: crate::component::CssComponentValueLimits,
) -> Result<CssImportRule, crate::rule::CssRuleConstructionError> {
    use crate::component::CssComponentValueRef as Component;
    use crate::component::CssValueTokenRef as Value;
    values.validate_with_limits(limits)?;
    if let Some(origin) = values.first_implicit_origin() {
        return Err(
            crate::rule::imports::ImportConstructionKind::RecoveredInput {
                origin: origin.clone(),
            }
            .into(),
        );
    }
    let items = values.items();
    let first = items
        .iter()
        .position(|value| !crate::query::supports::construction::trivia(value))
        .ok_or_else(|| {
            import_construction_grammar(&crate::component::CssValueOrigin::Programmatic)
        })?;
    if !matches!(items[first].view(), Component::Token(Value::AtKeyword(name)) if name.eq_ignore_ascii_case("import"))
    {
        return Err(import_construction_grammar(items[first].origin()));
    }
    let semicolon = items
        .iter()
        .position(|value| matches!(value.view(), Component::Token(Value::Semicolon)))
        .ok_or_else(|| {
            import_construction_grammar(items.last().expect("nonempty import input").origin())
        })?;
    if let Some(value) = items[semicolon + 1..]
        .iter()
        .find(|value| !crate::query::supports::construction::trivia(value))
    {
        return Err(import_construction_grammar(value.origin()));
    }
    let serialized = values.serialize_tokens(limits)?;
    let selected = classify_constructed_import(&values, &serialized, context)?;
    let target_component = &items[selected.target_index];
    let layer_component = selected.layer_index.map(|index| items[index].clone());
    let supports_component = selected.supports_index.map(|index| items[index].clone());
    let supports = if let Some(index) = selected.supports_index {
        let Component::Function(function) = items[index].view() else {
            unreachable!("selected supports function")
        };
        let body = function.values().clone();
        let condition = match selected.supports_form {
            SupportsAuthoredForm::BareDeclaration => {
                let declaration = CssSupportsDeclaration::try_from_components(body, limits)?;
                let lexical = declaration.lexical().clone();
                CssCondition::lexical_form(
                    CssConditionKind::Leaf(CssSupportsLeaf::Declaration(Box::new(declaration))),
                    lexical,
                    SupportsAuthoredForm::BareDeclaration,
                )
            }
            SupportsAuthoredForm::BareName => {
                let meaningful = body
                    .items()
                    .iter()
                    .filter(|value| !crate::query::supports::construction::trivia(value))
                    .collect::<Vec<_>>();
                let [component] = meaningful.as_slice() else {
                    return Err(import_construction_grammar(items[index].origin()));
                };
                let name = CssSupportsConditionName::try_from_component((*component).clone())
                    .map_err(|_| import_construction_grammar(items[index].origin()))?;
                CssCondition::lexical_form(
                    CssConditionKind::Leaf(CssSupportsLeaf::Named(name)),
                    crate::query::supports::construction::SupportsLexical::root(body),
                    SupportsAuthoredForm::BareName,
                )
            }
            SupportsAuthoredForm::Condition => {
                CssCondition::<CssSupportsLeaf>::try_from_components(body, context, limits)?
            }
        };
        Some(CssImportSupports::new(condition))
    } else {
        None
    };
    let last_clause = selected
        .supports_index
        .or(selected.layer_index)
        .unwrap_or(selected.target_index);
    let media = construct_import_media(&items[last_clause + 1..semicolon], limits)?;
    let rule = CssImportRule::new(
        selected.target,
        selected.layer,
        supports,
        media,
        crate::rule::imports::ImportSyntax {
            at_keyword: items[first].clone(),
            prelude: crate::rule::imports::ImportPreludeSyntax {
                target: target_component.clone(),
                layer: layer_component,
                supports: supports_component,
                semicolon: items[semicolon].origin().clone(),
            },
        },
    );
    rule.serialize_tokens(limits)?;
    Ok(rule)
}
fn import_construction_grammar(
    origin: &crate::component::CssValueOrigin,
) -> crate::rule::CssRuleConstructionError {
    crate::rule::imports::ImportConstructionKind::InvalidRuleGrammar {
        origin: origin.clone(),
    }
    .into()
}
struct ConstructedImportSelection {
    target: CssImportTarget,
    layer: Option<CssImportLayer>,
    target_index: usize,
    layer_index: Option<usize>,
    supports_index: Option<usize>,
    supports_form: SupportsAuthoredForm,
}
fn classify_constructed_import(
    values: &crate::component::CssComponentValues,
    serialized: &crate::component::CssSerializedValue,
    context: &CssNamespaceContext,
) -> Result<ConstructedImportSelection, crate::rule::CssRuleConstructionError> {
    let source = serialized.as_css();
    let recovery = RecoveryState::at_depth(source, 0, StyleContextCaptures::default());
    if let Some(name) = &context.0.default {
        recovery.activate_namespace(None, name.clone());
    }
    for (prefix, name) in &context.0.named {
        recovery.activate_namespace(Some(prefix.clone()), name.clone());
    }
    let working_source = crate::tokenization::prepare(source);
    let mut buffer = ParserInput::new(&working_source);
    let mut input = Parser::new(&mut buffer);
    let numeric = crate::numeric::expression::NumericInputContext::components(values, serialized);
    let result = (|| {
        input.next()?; // The original component envelope proved the at-keyword.
        input.parse_until_before(Delimiter::Semicolon, |input| {
            input.skip_whitespace();
            let target_start = input.position().byte_index();
            let target = parse_import_target(input, &numeric)?;
            let selected = select_import_clauses(source, input, &recovery)?;
            let index_at = |offset| {
                let path = serialized
                    .component_path_at(offset)
                    .expect("selected complete original component");
                debug_assert_eq!(path.len(), 1);
                path[0]
            };
            let index = |component: Option<&crate::component::CssComponentValue>| {
                component.map(|component| {
                    index_at(
                        component
                            .parsed_origin()
                            .expect("transport component")
                            .span()
                            .start()
                            .byte_offset()
                            .value(),
                    )
                })
            };
            let layer_index = index(selected.layer_component.as_ref());
            let supports_index = index(selected.supports_component.as_ref());
            let supports_form = selected
                .supports
                .as_ref()
                .map_or(SupportsAuthoredForm::Condition, |supports| {
                    supports.condition().authored_form()
                });
            // parse_until_before requires complete consumption; media is checked
            // strictly against its original components after classification.
            while input.next_including_whitespace_and_comments().is_ok() {}
            Ok(ConstructedImportSelection {
                target,
                layer: selected.layer,
                target_index: index_at(target_start),
                layer_index,
                supports_index,
                supports_form,
            })
        })
    })();
    result.map_err(|error: ParseError<'_, CssError>| {
        let error = from_parse_error(source, error);
        let offset = match error.kind() {
            crate::diagnostic::error::ErrorKind::InvalidComponentValue(component) => {
                crate::query::media::construction::parsed_position(component.origin())
                    .map_or(error.position().byte_offset().value(), |position| {
                        position.byte_offset().value()
                    })
            }
            _ => error.position().byte_offset().value(),
        };
        let origin = serialized
            .value_origin_at(offset)
            .cloned()
            .unwrap_or(crate::component::CssValueOrigin::Programmatic);
        let kind = match error.kind() {
            crate::diagnostic::error::ErrorKind::InvalidComponentValue(component) => {
                Some(component.kind)
            }
            crate::diagnostic::error::ErrorKind::NestingLimit(_) => {
                Some(crate::component::values::CssComponentValueErrorKind::NestingLimit)
            }
            crate::diagnostic::error::ErrorKind::ParserUnavailable(_) => {
                Some(crate::component::values::CssComponentValueErrorKind::ParserUnavailable)
            }
            _ => None,
        };
        match kind {
            Some(kind) => crate::component::CssComponentValueError::new(kind, origin).into(),
            None => import_construction_grammar(&origin),
        }
    })
}
fn construct_import_media(
    items: &[crate::component::CssComponentValue],
    limits: crate::component::CssComponentValueLimits,
) -> Result<Option<CssMediaQueryList>, crate::rule::CssRuleConstructionError> {
    if items
        .iter()
        .all(crate::query::supports::construction::trivia)
    {
        return Ok(None);
    }
    let mut queries = Vec::new();
    let mut commas = Vec::new();
    let mut start = 0;
    for end in 0..=items.len() {
        if end != items.len()
            && !matches!(
                items[end].view(),
                crate::component::CssComponentValueRef::Token(
                    crate::component::CssValueTokenRef::Comma
                )
            )
        {
            continue;
        }
        let member = &items[start..end];
        // Syntax 3 (2026 CRD) §5.4.10: a final comma followed by no token,
        // only comments, appends no member.
        if end == items.len()
            && start > 0
            && member.iter().all(|value| {
                matches!(
                    value.view(),
                    crate::component::CssComponentValueRef::Comment(_)
                )
            })
        {
            break;
        }
        if let Some(error) =
            crate::query::media::construction::empty_media_member_error(items, start, end)
        {
            return Err(error.into());
        }
        let values = crate::component::CssComponentValues::try_new(member.to_vec(), limits)?;
        queries.push(CssMediaQuery::try_from_components(values, limits)?);
        if let Some(comma) = items.get(end) {
            commas.push(comma.origin().clone());
        }
        start = end + 1;
    }
    Ok(Some(CssMediaQueryList::with_comma_origins(queries, commas)))
}

pub(crate) fn parse_import_prelude<'i, 't>(
    source: &'i str,
    input: &mut Parser<'i, 't>,
    _diagnostics: &mut Vec<crate::diagnostic::CssRecoveryDiagnostic>,
    recovery: &RecoveryState,
) -> Result<CssImportPrelude, ParseError<'i, CssError>> {
    input.skip_whitespace();
    let target_start = input.state();
    let numeric =
        crate::numeric::expression::NumericInputContext::parsed(recovery.source_snapshot());
    let target = parse_import_target(input, &numeric)?;
    let target_end = input.state();
    input.reset(&target_start);
    let target_component =
        crate::component::CssComponentValue::collect_from_parser(input, recovery.source_snapshot())
            .map_err(|error| {
                crate::diagnostic::error::invalid_component_value(
                    input.current_source_location(),
                    error,
                )
            })?;
    input.reset(&target_end);
    queries::check_media_import_components(source, input, recovery)?;
    // Include the accepted target in closure ownership: URL tokenization can
    // succeed at EOF without consuming a closing parenthesis.
    input.reset(&target_start);
    let prelude_closures =
        recovery.check_specialized_components(source, input, "official.media.query-list-core")?;
    input.reset(&target_end);
    let selected = select_import_clauses(source, input, recovery)?;
    let probe = recovery.detached_probe();
    let mut diagnostics = selected.diagnostics;
    let mut implicit = selected.implicit_closures;
    let clauses_end = input.position().byte_index();
    implicit.extend(
        prelude_closures
            .into_iter()
            .filter(|opening| *opening < clauses_end),
    );
    let media = if input.is_exhausted() {
        None
    } else {
        let parsed =
            queries::parse_media_query_list_with_closures(source, input, &mut diagnostics, &probe)?;
        implicit.extend(parsed.implicit_closures);
        Some(parsed.queries)
    };
    while input.next_including_whitespace_and_comments().is_ok() {}
    let end = input.position().byte_index();
    let semicolon = if source.as_bytes().get(end) == Some(&b';') {
        crate::component::CssValueOrigin::Parsed(
            CssParsedOrigin::from_range(recovery.source_snapshot(), end..end + 1)
                .expect("parsed import terminator"),
        )
    } else {
        crate::component::CssValueOrigin::Programmatic
    };
    Ok(CssImportPrelude {
        target,
        layer: selected.layer,
        supports: selected.supports,
        media,
        implicit_media_closures: implicit,
        diagnostics,
        syntax: crate::rule::imports::ImportPreludeSyntax {
            target: target_component,
            layer: selected.layer_component,
            supports: selected.supports_component,
            semicolon,
        },
    })
}
struct ImportSelection {
    layer: Option<CssImportLayer>,
    supports: Option<CssImportSupports>,
    layer_component: Option<crate::component::CssComponentValue>,
    supports_component: Option<crate::component::CssComponentValue>,
    diagnostics: Vec<crate::diagnostic::CssRecoveryDiagnostic>,
    implicit_closures: Vec<usize>,
}
fn import_clause_component<'i>(
    input: &mut Parser<'i, '_>,
    start: &ParserState,
    recovery: &RecoveryState,
) -> Result<crate::component::CssComponentValue, ParseError<'i, CssError>> {
    let end = input.state();
    input.reset(start);
    let result =
        crate::component::CssComponentValue::collect_from_parser(input, recovery.source_snapshot())
            .map_err(|error| {
                crate::diagnostic::error::invalid_component_value(
                    input.current_source_location(),
                    error,
                )
            });
    input.reset(&end);
    result
}
fn select_import_clauses<'i>(
    source: &'i str,
    input: &mut Parser<'i, '_>,
    recovery: &RecoveryState,
) -> Result<ImportSelection, ParseError<'i, CssError>> {
    let start = input.state();
    let mut fallback = None;
    for (layer_present, supports_present) in
        [(true, true), (true, false), (false, true), (false, false)]
    {
        input.reset(&start);
        let probe = recovery.detached_probe();
        let mut diagnostics = Vec::new();
        let candidate = (|| {
            let (layer, layer_component) = if layer_present {
                input.skip_whitespace();
                let at = input.state();
                let layer = parse_import_layer(input)?
                    .ok_or_else(|| invalid_syntax(input.current_source_location()))?;
                (
                    Some(layer),
                    Some(import_clause_component(input, &at, &probe)?),
                )
            } else {
                (None, None)
            };
            let (supports, supports_component) = if supports_present {
                input.skip_whitespace();
                let at = input.state();
                let supports = parse_import_supports(source, input, &mut diagnostics, &probe)?
                    .ok_or_else(|| invalid_syntax(input.current_source_location()))?;
                (
                    Some(supports),
                    Some(import_clause_component(input, &at, &probe)?),
                )
            } else {
                (None, None)
            };
            Ok(ImportSelection {
                layer,
                supports,
                layer_component,
                supports_component,
                diagnostics,
                implicit_closures: probe.pending_component_closures(),
            })
        })();
        let candidate = match candidate {
            Ok(value) => value,
            Err(error) if queries::media_terminal_error(&error) => return Err(error),
            Err(_) => continue,
        };
        let media_start = input.state();
        let clean = if candidate.diagnostics.is_empty() {
            probe_import_media(source, input, &probe)
        } else {
            Ok(false)
        }?;
        input.reset(&media_start);
        if clean {
            return Ok(candidate);
        }
        if fallback.is_none() {
            fallback = Some((candidate, media_start));
        }
    }
    let (selected, media_start) =
        fallback.expect("absent optional clauses always form a candidate");
    input.reset(&media_start);
    Ok(selected)
}
fn probe_import_media<'i>(
    source: &'i str,
    input: &mut Parser<'i, '_>,
    recovery: &RecoveryState,
) -> Result<bool, ParseError<'i, CssError>> {
    if input.is_exhausted() {
        return Ok(true);
    }
    loop {
        let result = input.parse_until_before(Delimiter::Comma, |member| {
            queries::parse_media_query(
                source,
                member,
                &crate::numeric::expression::NumericInputContext::parsed(
                    recovery.source_snapshot(),
                ),
            )?;
            member.expect_exhausted()?;
            Ok(())
        });
        match result {
            Ok(()) => {}
            Err(error) if queries::media_terminal_error(&error) => return Err(error),
            Err(_) => return Ok(false),
        }
        if input.is_exhausted() {
            return Ok(true);
        }
        input.expect_comma().map_err(basic)?;
        if input.is_exhausted() {
            return Ok(false);
        }
    }
}
pub(crate) fn import_boundaries_match(
    serialized: &crate::component::CssSerializedValue,
    expected: [Option<usize>; 2],
    original_origin: &crate::component::CssValueOrigin,
) -> Result<bool, crate::rule::imports::ImportSerializationFailure> {
    let source = serialized.as_css();
    let recovery = RecoveryState::at_depth(source, 0, StyleContextCaptures::default());
    let working_source = crate::tokenization::prepare(source);
    let mut buffer = ParserInput::new(&working_source);
    let mut input = Parser::new(&mut buffer);
    let result = (|| {
        let selected = select_import_clauses(source, &mut input, &recovery)?;
        let actual = [
            selected.layer_component.as_ref(),
            selected.supports_component.as_ref(),
        ]
        .map(|component| {
            component
                .and_then(crate::component::CssComponentValue::parsed_origin)
                .map(|origin| origin.span().end().byte_offset().value())
        });
        Ok(actual == expected && probe_import_media(source, &mut input, &recovery)?)
    })();
    result.map_err(|error: ParseError<'_, CssError>| {
        let error = from_parse_error(source, error);
        let offset = match error.kind() {
            crate::diagnostic::error::ErrorKind::InvalidComponentValue(component) => {
                crate::query::media::construction::parsed_position(component.origin())
                    .map_or(error.position().byte_offset().value(), |position| {
                        position.byte_offset().value()
                    })
            }
            _ => error.position().byte_offset().value(),
        };
        let origin = serialized
            .value_origin_at(offset)
            .unwrap_or(original_origin)
            .clone();
        let kind = match error.kind() {
            crate::diagnostic::error::ErrorKind::InvalidComponentValue(component) => {
                Some(component.kind)
            }
            crate::diagnostic::error::ErrorKind::NestingLimit(_) => {
                Some(crate::component::values::CssComponentValueErrorKind::NestingLimit)
            }
            crate::diagnostic::error::ErrorKind::ParserUnavailable(_) => {
                Some(crate::component::values::CssComponentValueErrorKind::ParserUnavailable)
            }
            _ => None,
        };
        match kind {
            Some(kind) => crate::rule::imports::ImportSerializationFailure::Component(
                crate::component::CssComponentValueError::new(kind, origin),
            ),
            None => {
                crate::rule::imports::ImportSerializationFailure::InterpretationChanged { origin }
            }
        }
    })
}
fn parse_import_supports<'i, 't>(
    source: &'i str,
    input: &mut Parser<'i, 't>,
    diagnostics: &mut Vec<crate::diagnostic::CssRecoveryDiagnostic>,
    recovery: &RecoveryState,
) -> std::result::Result<Option<CssImportSupports>, ParseError<'i, CssError>> {
    if input
        .try_parse(|input| input.expect_function_matching("supports"))
        .is_err()
    {
        return Ok(None);
    }

    let condition = input.parse_nested_block(|nested| {
        if let Some(declaration) = supports::grammar_probe(nested.try_parse(|nested| {
            let declaration = parse_supports_declaration(
                nested,
                recovery.source_snapshot(),
                recovery.parser_context(),
            )?;
            nested.expect_exhausted().map_err(basic)?;
            Ok::<_, ParseError<'i, CssError>>(declaration)
        }))? {
            let lexical = declaration.lexical().clone();
            return Ok(CssCondition::lexical_form(
                CssConditionKind::Leaf(CssSupportsLeaf::Declaration(Box::new(declaration))),
                lexical,
                SupportsAuthoredForm::BareDeclaration,
            ));
        }

        if let Some(named) = supports::grammar_probe(nested.try_parse(|nested| {
            let at = nested.current_source_location();
            let values =
                CssComponentValues::collect_from_parser(nested, recovery.source_snapshot())
                    .map_err(|error| {
                        crate::diagnostic::error::invalid_component_value(at, error)
                    })?;
            let meaningful = values
                .items()
                .iter()
                .filter(|value| !crate::query::supports::construction::trivia(value))
                .collect::<Vec<_>>();
            let [component] = meaningful.as_slice() else {
                return Err(invalid_syntax(at));
            };
            let name = CssSupportsConditionName::try_from_component((*component).clone())
                .map_err(|_| invalid_syntax(at))?;
            Ok::<_, ParseError<'i, CssError>>(CssCondition::lexical_form(
                CssConditionKind::Leaf(CssSupportsLeaf::Named(name)),
                crate::query::supports::construction::SupportsLexical::root(values),
                SupportsAuthoredForm::BareName,
            ))
        }))? {
            return Ok(named);
        }

        parse_supports_condition(source, nested, diagnostics, recovery)
    })?;

    Ok(Some(CssImportSupports::new(condition)))
}

fn parse_import_target<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::expression::NumericInputContext<'_>,
) -> std::result::Result<CssImportTarget, ParseError<'i, CssError>> {
    let location = input.current_source_location();

    if let Ok(value) = input.try_parse(Parser::expect_string_cloned) {
        return Ok(CssImportTarget::String(CssImportString::new(
            value.as_ref(),
        )));
    }

    if let Ok(value) = input.try_parse(|input| {
        super::property_input::adapt_native(input, numeric, NativeValueRange::Component, parse_url)
    }) {
        return Ok(CssImportTarget::Url(CssImportUrl::new(value)));
    }

    Err(invalid_syntax(location))
}

fn parse_import_layer<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<Option<CssImportLayer>, ParseError<'i, CssError>> {
    if input
        .try_parse(|input| input.expect_ident_matching("layer"))
        .is_ok()
    {
        return Ok(Some(CssImportLayer::Anonymous));
    }

    if input
        .try_parse(|input| input.expect_function_matching("layer"))
        .is_ok()
    {
        let layer_name = input.parse_nested_block(parse_import_layer_name)?;
        return Ok(Some(CssImportLayer::Named(layer_name)));
    }

    Ok(None)
}

fn parse_import_layer_name<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssLayerName, ParseError<'i, CssError>> {
    let name = parse_layer_name(input)?;
    if !input.is_exhausted() {
        return Err(invalid_syntax(input.current_source_location()));
    }
    Ok(name)
}
