//! Import construction, optional-clause selection, and interpretation protection.

use super::descriptor_stream::{
    Value, ValueError, byte_offset, collect_scope, consume_remaining, expect_exhausted, location,
    native, parse_scope,
};
use super::property_input::StreamExt as _;
use super::recovery::{RecoveryState, StyleContextCaptures};
use super::syntax_types::{Mark, Site, Stream, SyntaxError};
use super::url::parse_url;
use super::{queries, supports};
use crate::diagnostic::CssError;
use crate::diagnostic::error::{from_parse_error, invalid_syntax};
use crate::numeric::expression::NumericInputContext;
use crate::query::condition::SupportsAuthoredForm;
use crate::query::media::{CssMediaQuery, CssMediaQueryList};
use crate::query::supports::CssSupportsConditionName;
use crate::query::supports::{CssSupportsDeclaration, CssSupportsLeaf};
use crate::query::{CssCondition, CssConditionKind};
use crate::rule::{
    CssImportLayer, CssImportRule, CssImportString, CssImportSupports, CssImportTarget,
    CssImportUrl,
};
use crate::selector::CssNamespaceContext;
use crate::source::CssParsedOrigin;
use cssparser::{ParseError, Parser};
use surgeist_css_parser::stream::StopToken;

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
    let numeric = NumericInputContext::parsed(recovery.source_snapshot());
    let document = crate::syntax_consumption::source_scope_document(
        recovery.source_snapshot(),
        0..source.len(),
    )?;
    let mut input = document.root();
    let result: Value<'_, ConstructedImportSelection> = (|| {
        // The original component envelope proved the at-keyword.
        input.consume_after_whitespace()?;
        input.within_list_until(StopToken::Semicolon, |input| {
            input.discard_whitespace();
            let target_start = byte_offset(&numeric, input.site());
            let at = input.site();
            // The text locates the target; the target is read from its own
            // component, whose url modifiers keep their original sources.
            parse_import_target(input, &numeric)?;
            let selected = select_import_clauses(source, input, &numeric, &recovery)?;
            let index_at = |offset| {
                let path = serialized
                    .component_path_at(offset)
                    .expect("selected complete original component");
                debug_assert_eq!(path.len(), 1);
                path[0]
            };
            let target = checked_import_target(values, serialized, index_at(target_start), at)?;
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
            // The list must be consumed completely; media is checked strictly
            // against its original components after classification.
            consume_remaining(input);
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
    result.map_err(|error: ValueError<'_>| {
        let (kind, origin) = failure_of(
            serialized,
            from_parse_error(source, native(&numeric, &error)),
            crate::component::CssValueOrigin::Programmatic,
        );
        match kind {
            Some(kind) => crate::component::CssComponentValueError::new(kind, origin).into(),
            None => import_construction_grammar(&origin),
        }
    })
}
/// The target of the import that is the component `index` of `values`.
fn checked_import_target<'i>(
    values: &crate::component::CssComponentValues,
    serialized: &crate::component::CssSerializedValue,
    index: usize,
    at: Site<'i>,
) -> Value<'i, CssImportTarget> {
    let numeric = NumericInputContext::components(values, serialized);
    let document = crate::syntax_consumption::checked_document(
        &values.items()[index..=index],
        crate::syntax_consumption::SyntaxInputLimits::default(),
        0,
    )
    .map_err(|error| {
        SyntaxError::custom(
            at,
            crate::diagnostic::error::invalid_component_value(location(&numeric, at), error),
        )
    })?;
    let mut stream = document.root();
    parse_import_target(&mut stream, &numeric)
        .map_err(|error| SyntaxError::custom(at, native(&numeric, &error)))
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
    parse_scope(input, recovery.source_snapshot(), |stream, numeric| {
        import_prelude(source, stream, numeric, recovery)
    })
}

fn import_prelude<'i>(
    source: &str,
    input: &mut Stream<'i>,
    numeric: &NumericInputContext<'_>,
    recovery: &RecoveryState,
) -> Value<'i, CssImportPrelude> {
    input.discard_whitespace();
    let target_start = input.mark();
    let target = parse_import_target(input, numeric)?;
    let target_end = input.mark();
    input.restore(target_start);
    let target_component = import_clause_component(input, target_start, numeric)?;
    input.restore(target_end);
    queries::check_media_import_components(source, input, numeric, recovery)?;
    // Include the accepted target in closure ownership: URL tokenization can
    // succeed at EOF without consuming a closing parenthesis.
    input.restore(target_start);
    let prelude_closures = recovery
        .check_specialized_components_from(
            source,
            byte_offset(numeric, input.site()),
            "official.media.query-list-core",
        )
        .map_err(|error| SyntaxError::custom(input.site(), error))?;
    input.restore(target_end);
    let selected = select_import_clauses(source, input, numeric, recovery)?;
    let probe = recovery.detached_probe();
    let mut diagnostics = selected.diagnostics;
    let mut implicit = selected.implicit_closures;
    let clauses_end = byte_offset(numeric, input.site());
    implicit.extend(
        prelude_closures
            .into_iter()
            .filter(|opening| *opening < clauses_end),
    );
    let media = if input.only_whitespace_remains() {
        None
    } else {
        let parsed = queries::media_query_list(source, input, numeric, &mut diagnostics, &probe)?;
        implicit.extend(parsed.implicit_closures);
        Some(parsed.queries)
    };
    consume_remaining(input);
    let end = byte_offset(numeric, input.site());
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
/// A syntax failure at `at`.
fn invalid<'i>(numeric: &NumericInputContext<'_>, at: Site<'i>) -> ValueError<'i> {
    SyntaxError::custom(at, invalid_syntax(location(numeric, at)))
}
/// The component that begins at the mark `start`.
fn import_clause_component<'i>(
    input: &mut Stream<'i>,
    start: Mark<'i>,
    numeric: &NumericInputContext<'_>,
) -> Value<'i, crate::component::CssComponentValue> {
    let end = input.mark();
    input.restore(start);
    let result = input
        .collect_component()
        .map(std::borrow::Cow::into_owned)
        .map_err(|error| {
            SyntaxError::custom(
                input.site(),
                crate::diagnostic::error::invalid_component_value(
                    location(numeric, input.site()),
                    error,
                ),
            )
        });
    input.restore(end);
    result
}
fn select_import_clauses<'i>(
    source: &str,
    input: &mut Stream<'i>,
    numeric: &NumericInputContext<'_>,
    recovery: &RecoveryState,
) -> Value<'i, ImportSelection> {
    let start = input.mark();
    let mut fallback = None;
    for (layer_present, supports_present) in
        [(true, true), (true, false), (false, true), (false, false)]
    {
        input.restore(start);
        let probe = recovery.detached_probe();
        let diagnostics = Vec::new();
        let candidate: Value<'i, ImportSelection> = (|| {
            let (layer, layer_component) = if layer_present {
                input.discard_whitespace();
                let at = input.mark();
                let layer = parse_import_layer(input, numeric)?
                    .ok_or_else(|| invalid(numeric, input.site()))?;
                (
                    Some(layer),
                    Some(import_clause_component(input, at, numeric)?),
                )
            } else {
                (None, None)
            };
            let (supports, supports_component) = if supports_present {
                input.discard_whitespace();
                let at = input.mark();
                let supports = parse_import_supports(source, input, numeric, &probe)?
                    .ok_or_else(|| invalid(numeric, input.site()))?;
                (
                    Some(supports),
                    Some(import_clause_component(input, at, numeric)?),
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
            Err(error) if queries::media_terminal(&error) => return Err(error),
            Err(_) => continue,
        };
        let media_start = input.mark();
        let clean = if candidate.diagnostics.is_empty() {
            probe_import_media(source, input, numeric)
        } else {
            Ok(false)
        }?;
        input.restore(media_start);
        if clean {
            return Ok(candidate);
        }
        if fallback.is_none() {
            fallback = Some((candidate, media_start));
        }
    }
    let (selected, media_start) =
        fallback.expect("absent optional clauses always form a candidate");
    input.restore(media_start);
    Ok(selected)
}
fn probe_import_media<'i>(
    source: &str,
    input: &mut Stream<'i>,
    numeric: &NumericInputContext<'_>,
) -> Value<'i, bool> {
    if input.only_whitespace_remains() {
        return Ok(true);
    }
    loop {
        let result = input.within_list_until(StopToken::Comma, |member| {
            queries::parse_media_query(source, member, numeric)?;
            expect_exhausted(member)?;
            Ok(())
        });
        match result {
            Ok(()) => {}
            Err(error) if queries::media_terminal(&error) => return Err(error),
            Err(_) => return Ok(false),
        }
        if input.only_whitespace_remains() {
            return Ok(true);
        }
        input.consume_comma()?;
        if input.only_whitespace_remains() {
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
    let numeric = NumericInputContext::parsed(recovery.source_snapshot());
    let document = crate::syntax_consumption::source_scope_document(
        recovery.source_snapshot(),
        0..source.len(),
    )
    .map_err(crate::rule::imports::ImportSerializationFailure::Component)?;
    let mut input = document.root();
    let result = (|| {
        let selected = select_import_clauses(source, &mut input, &numeric, &recovery)?;
        let actual = [
            selected.layer_component.as_ref(),
            selected.supports_component.as_ref(),
        ]
        .map(|component| {
            component
                .and_then(crate::component::CssComponentValue::parsed_origin)
                .map(|origin| origin.span().end().byte_offset().value())
        });
        Ok(actual == expected && probe_import_media(source, &mut input, &numeric)?)
    })();
    result.map_err(|error: ValueError<'_>| {
        let (kind, origin) = failure_of(
            serialized,
            from_parse_error(source, native(&numeric, &error)),
            original_origin.clone(),
        );
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
/// What a failure of the import grammar over the serialization says: the kind of
/// component failure it is, when it is one, and the origin of the component it
/// is at, or `fallback` when no component is.
fn failure_of(
    serialized: &crate::component::CssSerializedValue,
    error: CssError,
    fallback: crate::component::CssValueOrigin,
) -> (
    Option<crate::component::values::CssComponentValueErrorKind>,
    crate::component::CssValueOrigin,
) {
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
        .unwrap_or(fallback);
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
    (kind, origin)
}
fn parse_import_supports<'i>(
    source: &str,
    input: &mut Stream<'i>,
    numeric: &NumericInputContext<'_>,
    recovery: &RecoveryState,
) -> Value<'i, Option<CssImportSupports>> {
    if input
        .attempt(|input| input.consume_function_named("supports"))
        .is_err()
    {
        return Ok(None);
    }

    let condition = input.within_block(|nested| {
        if let Some(declaration) = supports::grammar_probe(nested.attempt(|nested| {
            let declaration =
                supports::supports_declaration(nested, numeric, recovery.parser_context())?;
            expect_exhausted(nested)?;
            Ok(declaration)
        }))? {
            let lexical = declaration.lexical().clone();
            return Ok(CssCondition::lexical_form(
                CssConditionKind::Leaf(CssSupportsLeaf::Declaration(Box::new(declaration))),
                lexical,
                SupportsAuthoredForm::BareDeclaration,
            ));
        }

        if let Some(named) = supports::grammar_probe(nested.attempt(|nested| {
            let at = nested.site();
            let start = location(numeric, at);
            let collected = collect_scope(nested, numeric);
            consume_remaining(nested);
            let values = collected.map_err(|(error, _)| {
                SyntaxError::custom(
                    at,
                    crate::diagnostic::error::invalid_component_value(start, error),
                )
            })?;
            let meaningful = values
                .items()
                .iter()
                .filter(|value| !crate::query::supports::construction::trivia(value))
                .collect::<Vec<_>>();
            let [component] = meaningful.as_slice() else {
                return Err(invalid(numeric, at));
            };
            let name = CssSupportsConditionName::try_from_component((*component).clone())
                .map_err(|_| invalid(numeric, at))?;
            Ok::<_, ValueError<'i>>(CssCondition::lexical_form(
                CssConditionKind::Leaf(CssSupportsLeaf::Named(name)),
                crate::query::supports::construction::SupportsLexical::root(values),
                SupportsAuthoredForm::BareName,
            ))
        }))? {
            return Ok(named);
        }

        supports::supports_condition(source, nested, numeric, recovery)
    })?;

    Ok(Some(CssImportSupports::new(condition)))
}

fn parse_import_target<'i>(
    input: &mut Stream<'i>,
    numeric: &NumericInputContext<'_>,
) -> Value<'i, CssImportTarget> {
    let at = input.site();

    if let Ok(value) = input.attempt(Stream::consume_string) {
        return Ok(CssImportTarget::String(CssImportString::new(value)));
    }

    if let Ok(value) = input.attempt(|input| parse_url(input, numeric)) {
        return Ok(CssImportTarget::Url(CssImportUrl::new(value)));
    }

    Err(invalid(numeric, at))
}

fn parse_import_layer<'i>(
    input: &mut Stream<'i>,
    numeric: &NumericInputContext<'_>,
) -> Value<'i, Option<CssImportLayer>> {
    if input
        .attempt(|input| input.consume_keyword("layer"))
        .is_ok()
    {
        return Ok(Some(CssImportLayer::Anonymous));
    }

    if input
        .attempt(|input| input.consume_function_named("layer"))
        .is_ok()
    {
        let layer_name = input.within_block(|input| super::read_layer_name(input, numeric))?;
        return Ok(Some(CssImportLayer::Named(layer_name)));
    }

    Ok(None)
}
