//! Import construction, optional-clause selection, and interpretation protection.

use super::recovery::{RecoveryState, StyleContextCaptures};
use super::supports::{parse_supports_condition, parse_supports_declaration};
use super::url::parse_url;
use super::{CssNamespaceContext, parse_layer_name, queries, supports};
use crate::error::{Error, basic, from_parse_error, invalid_syntax};
use crate::named_supports::CssSupportsConditionName;
use crate::syntax::*;
use crate::{CssComponentValues, CssParsedOrigin};
use cssparser::{Delimiter, ParseError, Parser, ParserInput, ParserState};

pub(super) struct CssImportPrelude {
    pub(super) target: CssImportTarget,
    pub(super) layer: Option<CssImportLayer>,
    pub(super) supports: Option<CssImportSupports>,
    pub(super) media: Option<CssMediaQueryList>,
    pub(super) implicit_media_closures: Vec<usize>,
    pub(super) diagnostics: Vec<crate::CssRecoveryDiagnostic>,
    pub(super) syntax: crate::imports::ImportPreludeSyntax,
}

pub(crate) fn construct_import(
    values: crate::CssComponentValues,
    context: &CssNamespaceContext,
    limits: crate::CssComponentValueLimits,
) -> Result<CssImportRule, crate::CssImportConstructionError> {
    use crate::{CssComponentValueRef as Component, CssValueTokenRef as Value};
    values.validate_with_limits(limits)?;
    if let Some(origin) = values.first_implicit_origin() {
        return Err(crate::CssImportConstructionError::RecoveredInput {
            origin: origin.clone(),
        });
    }
    let items = values.items();
    let first = items
        .iter()
        .position(|value| !crate::supports::trivia(value))
        .ok_or_else(|| import_construction_grammar(&crate::CssValueOrigin::Programmatic))?;
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
        .find(|value| !crate::supports::trivia(value))
    {
        return Err(import_construction_grammar(value.origin()));
    }
    let serialized = values.serialize_with_limit(limits.max_css_bytes())?;
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
                let declaration =
                    CssSupportsDeclaration::try_from_components_with_limits(body, limits)?;
                let lexical = declaration.lexical().clone();
                CssSupportsCondition::new(
                    CssSupportsConditionKind::Declaration(Box::new(declaration)),
                    lexical,
                    SupportsAuthoredForm::BareDeclaration,
                )
            }
            SupportsAuthoredForm::BareName => {
                let meaningful = body
                    .items()
                    .iter()
                    .filter(|value| !crate::supports::trivia(value))
                    .collect::<Vec<_>>();
                let [component] = meaningful.as_slice() else {
                    return Err(import_construction_grammar(items[index].origin()));
                };
                let name = CssSupportsConditionName::try_from_component((*component).clone())
                    .map_err(|_| import_construction_grammar(items[index].origin()))?;
                CssSupportsCondition::new(
                    CssSupportsConditionKind::Named(name),
                    crate::supports::SupportsLexical::root(body),
                    SupportsAuthoredForm::BareName,
                )
            }
            SupportsAuthoredForm::Condition => {
                CssSupportsCondition::try_from_components_with_limits(body, context, limits)?
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
        crate::imports::ImportSyntax {
            at_keyword: items[first].clone(),
            prelude: crate::imports::ImportPreludeSyntax {
                target: target_component.clone(),
                layer: layer_component,
                supports: supports_component,
                semicolon: items[semicolon].origin().clone(),
            },
        },
    );
    rule.serialize_with_limit(limits.max_css_bytes())?;
    Ok(rule)
}
fn import_construction_grammar(
    origin: &crate::CssValueOrigin,
) -> crate::CssImportConstructionError {
    crate::CssImportConstructionError::InvalidRuleGrammar {
        origin: origin.clone(),
    }
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
    values: &crate::CssComponentValues,
    serialized: &crate::CssSerializedValue,
    context: &CssNamespaceContext,
) -> Result<ConstructedImportSelection, crate::CssImportConstructionError> {
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
    let numeric = crate::numeric::NumericInputContext::components(values, serialized);
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
            let index = |component: Option<&crate::CssComponentValue>| {
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
    result.map_err(|error: ParseError<'_, Error>| {
        let error = from_parse_error(source, error);
        let offset = match error.kind() {
            crate::ErrorKind::InvalidComponentValue(component) => {
                crate::media::parsed_position(component.origin())
                    .map_or(error.position().byte_offset().value(), |position| {
                        position.byte_offset().value()
                    })
            }
            _ => error.position().byte_offset().value(),
        };
        let origin = serialized
            .value_origin_at(offset)
            .cloned()
            .unwrap_or(crate::CssValueOrigin::Programmatic);
        let kind = match error.kind() {
            crate::ErrorKind::InvalidComponentValue(component) => Some(component.kind()),
            crate::ErrorKind::NestingLimit(_) => {
                Some(crate::CssComponentValueErrorKind::NestingLimit)
            }
            _ => None,
        };
        match kind {
            Some(kind) => crate::CssImportConstructionError::Component(
                crate::CssComponentValueError::new(kind, origin),
            ),
            None => import_construction_grammar(&origin),
        }
    })
}
fn construct_import_media(
    items: &[crate::CssComponentValue],
    limits: crate::CssComponentValueLimits,
) -> Result<Option<CssMediaQueryList>, crate::CssImportConstructionError> {
    if items.iter().all(crate::supports::trivia) {
        return Ok(None);
    }
    let mut queries = Vec::new();
    let mut commas = Vec::new();
    let mut start = 0;
    for end in 0..=items.len() {
        if end != items.len()
            && !matches!(
                items[end].view(),
                crate::CssComponentValueRef::Token(crate::CssValueTokenRef::Comma)
            )
        {
            continue;
        }
        let member = &items[start..end];
        if member.iter().all(crate::supports::trivia) {
            let origin = items
                .get(end)
                .or_else(|| start.checked_sub(1).and_then(|index| items.get(index)))
                .map_or(crate::CssValueOrigin::Programmatic, |value| {
                    value.origin().clone()
                });
            return Err(crate::CssMediaConstructionError::InvalidQueryGrammar { origin }.into());
        }
        let values = crate::CssComponentValues::try_new_with_limits(member.to_vec(), limits)?;
        queries.push(CssMediaQuery::try_from_components_with_limits(
            values, limits,
        )?);
        if let Some(comma) = items.get(end) {
            commas.push(comma.origin().clone());
        }
        start = end + 1;
    }
    Ok(Some(CssMediaQueryList::with_comma_origins(queries, commas)))
}

pub(super) fn parse_import_prelude<'i, 't>(
    source: &'i str,
    input: &mut Parser<'i, 't>,
    _diagnostics: &mut Vec<crate::CssRecoveryDiagnostic>,
    recovery: &RecoveryState,
) -> Result<CssImportPrelude, ParseError<'i, Error>> {
    input.skip_whitespace();
    let target_start = input.state();
    let numeric = crate::numeric::NumericInputContext::parsed(recovery.source_snapshot());
    let target = parse_import_target(input, &numeric)?;
    let target_end = input.state();
    input.reset(&target_start);
    let target_component =
        crate::CssComponentValue::collect_from_parser(input, recovery.source_snapshot()).map_err(
            |error| crate::error::invalid_component_value(input.current_source_location(), error),
        )?;
    input.reset(&target_end);
    queries::check_media_import_components(source, input, recovery)?;
    // Include the accepted target in closure ownership: URL tokenization can
    // succeed at EOF without consuming a closing parenthesis.
    input.reset(&target_start);
    let prelude_closures =
        recovery.check_specialized_components(source, input, "baseline.media.query-list")?;
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
        crate::CssValueOrigin::Parsed(
            CssParsedOrigin::from_range(recovery.source_snapshot(), end..end + 1)
                .expect("parsed import terminator"),
        )
    } else {
        crate::CssValueOrigin::Programmatic
    };
    Ok(CssImportPrelude {
        target,
        layer: selected.layer,
        supports: selected.supports,
        media,
        implicit_media_closures: implicit,
        diagnostics,
        syntax: crate::imports::ImportPreludeSyntax {
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
    layer_component: Option<crate::CssComponentValue>,
    supports_component: Option<crate::CssComponentValue>,
    diagnostics: Vec<crate::CssRecoveryDiagnostic>,
    implicit_closures: Vec<usize>,
}
fn import_clause_component<'i>(
    input: &mut Parser<'i, '_>,
    start: &ParserState,
    recovery: &RecoveryState,
) -> Result<crate::CssComponentValue, ParseError<'i, Error>> {
    let end = input.state();
    input.reset(start);
    let result = crate::CssComponentValue::collect_from_parser(input, recovery.source_snapshot())
        .map_err(|error| {
            crate::error::invalid_component_value(input.current_source_location(), error)
        });
    input.reset(&end);
    result
}
fn select_import_clauses<'i>(
    source: &'i str,
    input: &mut Parser<'i, '_>,
    recovery: &RecoveryState,
) -> Result<ImportSelection, ParseError<'i, Error>> {
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
                let layer = parse_import_layer(input)?.ok_or_else(|| {
                    invalid_syntax(
                        input.current_source_location(),
                        "expected import layer clause",
                    )
                })?;
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
                    .ok_or_else(|| {
                        invalid_syntax(
                            input.current_source_location(),
                            "expected import supports clause",
                        )
                    })?;
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
) -> Result<bool, ParseError<'i, Error>> {
    if input.is_exhausted() {
        return Ok(true);
    }
    loop {
        let result = input.parse_until_before(Delimiter::Comma, |member| {
            queries::parse_media_query(
                source,
                member,
                &crate::numeric::NumericInputContext::parsed(recovery.source_snapshot()),
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
    serialized: &crate::CssSerializedValue,
    expected: [Option<usize>; 2],
    original_origin: &crate::CssValueOrigin,
) -> Result<bool, crate::CssImportSerializationError> {
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
                .and_then(crate::CssComponentValue::parsed_origin)
                .map(|origin| origin.span().end().byte_offset().value())
        });
        Ok(actual == expected && probe_import_media(source, &mut input, &recovery)?)
    })();
    result.map_err(|error: ParseError<'_, Error>| {
        let error = from_parse_error(source, error);
        let offset = match error.kind() {
            crate::ErrorKind::InvalidComponentValue(component) => {
                crate::media::parsed_position(component.origin())
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
            crate::ErrorKind::InvalidComponentValue(component) => Some(component.kind()),
            crate::ErrorKind::NestingLimit(_) => {
                Some(crate::CssComponentValueErrorKind::NestingLimit)
            }
            _ => None,
        };
        match kind {
            Some(kind) => crate::CssImportSerializationError::Component(
                crate::CssComponentValueError::new(kind, origin),
            ),
            None => crate::CssImportSerializationError::InterpretationChanged { origin },
        }
    })
}
fn parse_import_supports<'i, 't>(
    source: &'i str,
    input: &mut Parser<'i, 't>,
    diagnostics: &mut Vec<crate::CssRecoveryDiagnostic>,
    recovery: &RecoveryState,
) -> std::result::Result<Option<CssImportSupports>, ParseError<'i, Error>> {
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
            Ok::<_, ParseError<'i, Error>>(declaration)
        }))? {
            let lexical = declaration.lexical().clone();
            return Ok(CssSupportsCondition::new(
                CssSupportsConditionKind::Declaration(Box::new(declaration)),
                lexical,
                SupportsAuthoredForm::BareDeclaration,
            ));
        }

        if let Some(named) = supports::grammar_probe(nested.try_parse(|nested| {
            let at = nested.current_source_location();
            let values =
                CssComponentValues::collect_from_parser(nested, recovery.source_snapshot())
                    .map_err(|error| crate::error::invalid_component_value(at, error))?;
            let meaningful = values
                .items()
                .iter()
                .filter(|value| !crate::supports::trivia(value))
                .collect::<Vec<_>>();
            let [component] = meaningful.as_slice() else {
                return Err(invalid_syntax(at, "one named supports reference"));
            };
            let name = CssSupportsConditionName::try_from_component((*component).clone())
                .map_err(|_| invalid_syntax(at, "one named supports reference"))?;
            Ok::<_, ParseError<'i, Error>>(CssSupportsCondition::new(
                CssSupportsConditionKind::Named(name),
                crate::supports::SupportsLexical::root(values),
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
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssImportTarget, ParseError<'i, Error>> {
    let location = input.current_source_location();

    if let Ok(value) = input.try_parse(Parser::expect_string_cloned) {
        return Ok(CssImportTarget::String(CssImportString::new(
            value.as_ref(),
        )));
    }

    if let Ok(value) = input.try_parse(|input| parse_url(input, numeric)) {
        return Ok(CssImportTarget::Url(CssImportUrl::new(value)));
    }

    Err(invalid_syntax(
        location,
        "expected string or URL import target",
    ))
}

fn parse_import_layer<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<Option<CssImportLayer>, ParseError<'i, Error>> {
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
) -> std::result::Result<CssLayerName, ParseError<'i, Error>> {
    let name = parse_layer_name(input)?;
    if !input.is_exhausted() {
        return Err(invalid_syntax(
            input.current_source_location(),
            "unexpected token in import layer name",
        ));
    }
    Ok(name)
}
