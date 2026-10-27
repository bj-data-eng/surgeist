//! Context-specific parsing over the caller's unmodified source.
use crate::component::CssComponentValues;
use crate::declaration::{CssDeclaration, CssImportance};
use crate::diagnostic::CssError;
use crate::diagnostic::error::{
    basic, from_parse_error, from_rule_parse_error, with_media_query_context,
};
use crate::parser::nesting::parse_style_contents;
use crate::parser::reconstruction::{BoundedParseContext, BoundedParseSyntax, parse_bounded};
use crate::parser::recovery::{RecoveryState, StyleContextCaptures, recovery_action_for_error};
use crate::parser::rule_list::{AtRulePrelude, RuleListItem, RuleParser};
use crate::parser::selectors::SelectorRecovery;
use crate::parser::{
    DeclarationMode, PropertyValueGrammar, ScopedBodyKind, collect_declaration_value,
    color_profile, conditional_chains, counter_style, font_face, font_feature_values,
    font_palette_values, keyframes, named_supports, page, parse_declaration_core,
    parse_descriptor_boundary, parse_nested_group_rules, parse_property_value_from_parser,
    parse_scoped_rule_list, property_input, queries, recovery, selectors,
};
use crate::property::{CssPropertyGrammar, CssPropertyNameRef};
use crate::query::media::{CssMediaQuery, CssMediaQueryList, CssNeverMediaQuery};
use crate::rule::color_profile::{CssColorProfileDescriptorKind, CssColorProfileDescriptorValue};
use crate::rule::font_face::CssFontFaceDescriptorKind;
use crate::rule::font_palette_values::{
    CssFontPaletteDescriptorKind, CssFontPaletteDescriptorValue,
};
use crate::rule::{CssRule, CssStyleBlock};
use crate::selector::{
    CssNamespaceContext, CssRelativeSelectorList, CssSelector, CssStyleSelectorList,
};
use crate::source::{CssParsedOrigin, CssSourceSnapshot};
use cssparser::{
    AtRuleParser, CowRcStr, ParseError, Parser, ParserInput, ParserState, QualifiedRuleParser,
    Token,
};

/// Complete fragment parsing whose refusal retains no syntax.
pub(super) fn bounded<T: Send>(
    source: &str,
    parse: impl FnOnce() -> crate::CssParseReport<Option<T>> + Send,
) -> crate::CssParseReport<Option<T>> {
    bounded_or(source, || None, parse)
}

/// Complete fragment parsing whose refusal retains the front door's `refused` syntax.
pub(super) fn bounded_or<T: Send>(
    source: &str,
    refused: impl FnOnce() -> T,
    parse: impl FnOnce() -> crate::CssParseReport<T> + Send,
) -> crate::CssParseReport<T> {
    recovery::finish_report(source, bounded_report(source, refused, parse))
}

/// Fragment-profile execution whose caller publishes tokenizer recovery itself.
pub(crate) fn bounded_report<T: Send>(
    source: &str,
    refused: impl FnOnce() -> T,
    parse: impl FnOnce() -> crate::CssParseReport<T> + Send,
) -> crate::CssParseReport<T> {
    super::bounded_execution::report(
        super::bounded_execution::StackProfile::Fragment,
        source,
        refused,
        parse,
    )
}

pub(super) fn reject(
    source: &str,
    error: ParseError<'_, CssError>,
    action: crate::diagnostic::CssRecoveryAction,
) -> crate::diagnostic::CssRecoveryDiagnostic {
    let action = recovery_action_for_error(&error, action);
    reject_resolved(source, from_parse_error(source, error), action)
}

fn reject_resolved(
    source: &str,
    error: CssError,
    action: crate::diagnostic::CssRecoveryAction,
) -> crate::diagnostic::CssRecoveryDiagnostic {
    let span = crate::source::CssSourceSpan::new(
        crate::source::CssSourcePosition::from_byte_offset_in(source, 0),
        crate::source::CssSourcePosition::from_byte_offset_in(source, source.len()),
    )
    .expect("complete source span");
    crate::diagnostic::CssRecoveryDiagnostic::new(error, span, action)
        .expect("fragment errors originate within the complete source")
}

pub(super) fn finish_nested_component<'i>(
    input: &mut Parser<'i, '_>,
    token: &Token<'i>,
) -> Result<(), ParseError<'i, CssError>> {
    if matches!(
        token,
        Token::Function(_)
            | Token::ParenthesisBlock
            | Token::SquareBracketBlock
            | Token::CurlyBracketBlock
    ) {
        // Finish this block now so the next root token's location and source
        // slice begin after it, rather than before implicit tokenizer skipping.
        input.parse_nested_block(|nested| {
            while nested.next_including_whitespace_and_comments().is_ok() {}
            Ok(())
        })?;
    }
    Ok(())
}

fn real_brace_block<T: Send>(
    source: &str,
    production: &'static str,
    parser_context: crate::CssParserContext,
    parse_body: impl for<'i, 't> FnOnce(
        &'i str,
        &mut Parser<'i, 't>,
        &mut Vec<crate::diagnostic::CssRecoveryDiagnostic>,
        RecoveryState,
    ) -> Result<T, ParseError<'i, CssError>>
    + Send,
) -> crate::CssParseReport<Option<crate::rule::CssBlockFragment<T>>> {
    bounded(source, || {
        let state = RecoveryState::at_depth(source, 0, StyleContextCaptures::default())
            .with_parser_context(parser_context);
        real_brace_block_inner(source, production, state, parse_body)
    })
}

fn real_brace_block_inner<T>(
    source: &str,
    production: &'static str,
    state: RecoveryState,
    parse_body: impl for<'i, 't> FnOnce(
        &'i str,
        &mut Parser<'i, 't>,
        &mut Vec<crate::diagnostic::CssRecoveryDiagnostic>,
        RecoveryState,
    ) -> Result<T, ParseError<'i, CssError>>,
) -> crate::CssParseReport<Option<crate::rule::CssBlockFragment<T>>> {
    let working_source = crate::tokenization::prepare(source);
    let mut parser_input = ParserInput::new(&working_source);
    let mut input = Parser::new(&mut parser_input);
    let mut diagnostics = Vec::new();
    let result = (|| {
        input.skip_whitespace();
        let start = input.position().byte_index();
        input.expect_curly_bracket_block()?;
        let body = input.parse_nested_block(|input| {
            let mut depth = state.enter_rule_block(source, input, production)?;
            // The entered native owner admits each unit before collecting its
            // components. Whole-body component admission would erase recovery
            // partitions and lose valid neighbors of a failed unit.
            let body = parse_body(source, input, &mut diagnostics, state.clone())?;
            depth.retain();
            Ok(body)
        })?;
        let end = input.position().byte_index();
        input.expect_exhausted()?;
        let origin = CssParsedOrigin::from_range(state.source_snapshot(), start..end)
            .expect("consumed block boundaries belong to the original source");
        Ok(crate::rule::CssBlockFragment::from_parsed(body, origin))
    })();
    match result {
        Ok(fragment) => {
            diagnostics.extend(state.take_implicit_closure_diagnostics(source));
            crate::CssParseReport::new(Some(fragment), diagnostics)
        }
        Err(error) => crate::CssParseReport::new(
            None,
            vec![reject(
                source,
                error,
                crate::diagnostic::CssRecoveryAction::RejectInput,
            )],
        ),
    }
}

pub(crate) fn parse_group_block_with_context(
    source: &str,
    namespaces: &CssNamespaceContext,
    parser_context: crate::CssParserContext,
) -> crate::CssParseReport<Option<crate::rule::CssBlockFragment<crate::rule::CssRuleList>>> {
    bounded(source, || {
        let snapshot = CssSourceSnapshot::new(source);
        let (syntax, mut diagnostics) = parse_bounded(
            source,
            &snapshot,
            0,
            BoundedParseContext::GroupBlock,
            StyleContextCaptures::with_namespaces(namespaces),
            parser_context,
        )
        .into_parts();
        let BoundedParseSyntax::GroupBlock(block) = syntax else {
            unreachable!("ordinary group replay preserves its genuine block carrier")
        };
        let block = block.map(|block| {
            let (mut body, origin) = block.into_parts();
            conditional_chains::completed_list(source, body.rules_mut(), &mut diagnostics);
            crate::rule::CssBlockFragment::from_parsed(body, origin)
        });
        crate::CssParseReport::new(block, diagnostics)
    })
}

pub(super) fn parse_group_block_inner(
    source: &str,
    recovery: RecoveryState,
) -> crate::CssParseReport<Option<crate::rule::CssBlockFragment<crate::rule::CssRuleList>>> {
    real_brace_block_inner(
        source,
        "css.rule-list",
        recovery,
        |source, input, diagnostics, recovery| {
            let mut parsed = parse_nested_group_rules(source, input, recovery)?;
            diagnostics.append(&mut parsed.diagnostics);
            Ok(crate::rule::CssRuleList::from_parsed(parsed.syntax))
        },
    )
}

pub(crate) fn parse_scope_block_with_context(
    source: &str,
    namespaces: &CssNamespaceContext,
    ancestry: crate::rule::CssStyleAncestor,
    parser_context: crate::CssParserContext,
) -> crate::CssParseReport<Option<crate::rule::CssBlockFragment<crate::rule::CssRuleList>>> {
    scoped_block_with_context(
        source,
        namespaces,
        ancestry,
        ScopedBodyKind::Scope,
        parser_context,
    )
}

pub(crate) fn parse_scoped_group_block_with_context(
    source: &str,
    namespaces: &CssNamespaceContext,
    ancestry: crate::rule::CssStyleAncestor,
    parser_context: crate::CssParserContext,
) -> crate::CssParseReport<Option<crate::rule::CssBlockFragment<crate::rule::CssRuleList>>> {
    scoped_block_with_context(
        source,
        namespaces,
        ancestry,
        ScopedBodyKind::OrdinaryGroup,
        parser_context,
    )
}

fn scoped_block_with_context(
    source: &str,
    namespaces: &CssNamespaceContext,
    ancestry: crate::rule::CssStyleAncestor,
    body: ScopedBodyKind,
    parser_context: crate::CssParserContext,
) -> crate::CssParseReport<Option<crate::rule::CssBlockFragment<crate::rule::CssRuleList>>> {
    bounded(source, || {
        let snapshot = CssSourceSnapshot::new(source);
        let has_style_ancestor = matches!(ancestry, crate::rule::CssStyleAncestor::Present);
        let (syntax, mut diagnostics) = parse_bounded(
            source,
            &snapshot,
            0,
            BoundedParseContext::ScopedBlock {
                has_style_ancestor,
                body,
            },
            StyleContextCaptures::with_namespaces(namespaces),
            parser_context,
        )
        .into_parts();
        let BoundedParseSyntax::ScopedBlock(block) = syntax else {
            unreachable!("scoped replay preserves its genuine block carrier")
        };
        let block = block.map(|block| {
            let (mut body, origin) = block.into_parts();
            conditional_chains::completed_list(source, body.rules_mut(), &mut diagnostics);
            crate::rule::CssBlockFragment::from_parsed(body, origin)
        });
        crate::CssParseReport::new(block, diagnostics)
    })
}

pub(super) fn parse_scoped_block_inner(
    source: &str,
    recovery: RecoveryState,
    has_style_ancestor: bool,
    body: ScopedBodyKind,
) -> crate::CssParseReport<Option<crate::rule::CssBlockFragment<crate::rule::CssRuleList>>> {
    let production = match body {
        ScopedBodyKind::Scope => "css.scope-block",
        ScopedBodyKind::OrdinaryGroup => "css.scoped-group-block",
    };
    real_brace_block_inner(
        source,
        production,
        recovery,
        |source, input, diagnostics, recovery| {
            let mut parsed =
                parse_scoped_rule_list(source, input, recovery, has_style_ancestor, body)?;
            diagnostics.append(&mut parsed.diagnostics);
            Ok(crate::rule::CssRuleList::from_parsed(parsed.syntax))
        },
    )
}

#[must_use]
pub(crate) fn parse_supports_test_block(
    source: &str,
    parser_context: crate::CssParserContext,
) -> crate::CssParseReport<
    Option<crate::rule::CssBlockFragment<crate::query::supports::CssSupportsTestBody>>,
> {
    real_brace_block(
        source,
        "ext.rule.supports-condition",
        parser_context,
        |source, input, diagnostics, recovery| {
            named_supports::parse_body(source, input, diagnostics, &recovery)
                .map(|(body, _, _)| body)
        },
    )
}

#[must_use]
pub(crate) fn parse_font_face_block(
    source: &str,
    parser_context: crate::CssParserContext,
) -> crate::CssParseReport<
    Option<crate::rule::CssBlockFragment<crate::rule::font_face::CssFontFaceDescriptors>>,
> {
    real_brace_block(
        source,
        "baseline.rule.font-face",
        parser_context,
        font_face::parse_body,
    )
}

#[must_use]
pub(crate) fn parse_counter_style_block(
    source: &str,
    parser_context: crate::CssParserContext,
) -> crate::CssParseReport<
    Option<crate::rule::CssBlockFragment<crate::rule::counter_style::CssCounterStyleDescriptors>>,
> {
    real_brace_block(
        source,
        "later.rule.counter-style",
        parser_context,
        counter_style::parse_body,
    )
}

#[must_use]
pub(crate) fn parse_font_palette_values_block(
    source: &str,
    parser_context: crate::CssParserContext,
) -> crate::CssParseReport<
    Option<
        crate::rule::CssBlockFragment<
            Vec<crate::rule::font_palette_values::CssFontPaletteDescriptor>,
        >,
    >,
> {
    real_brace_block(
        source,
        "later.rule.font-palette-values",
        parser_context,
        |source, input, diagnostics, state| {
            let descriptors = font_palette_values::parse_body(source, input, diagnostics, state);
            crate::rule::font_palette_values::rule::validate_descriptors(&descriptors).map_err(
                |_| crate::diagnostic::error::invalid_syntax(input.current_source_location()),
            )?;
            Ok(descriptors)
        },
    )
}

#[must_use]
pub(crate) fn parse_color_profile_block(
    source: &str,
    parser_context: crate::CssParserContext,
) -> crate::CssParseReport<
    Option<
        crate::rule::CssBlockFragment<Vec<crate::rule::color_profile::CssColorProfileDescriptor>>,
    >,
> {
    real_brace_block(
        source,
        "interop.rule.color-profile",
        parser_context,
        |source, input, diagnostics, state| {
            Ok(color_profile::parse_body(source, input, diagnostics, state))
        },
    )
}

#[must_use]
pub(crate) fn parse_font_feature_values_block(
    source: &str,
    parser_context: crate::CssParserContext,
) -> crate::CssParseReport<
    Option<
        crate::rule::CssBlockFragment<
            Vec<crate::rule::font_feature_values::CssFontFeatureValuesItem>,
        >,
    >,
> {
    real_brace_block(
        source,
        "later.rule.font-feature-values",
        parser_context,
        |source, input, diagnostics, state| {
            let recovered = font_feature_values::parse_outer_body(source, input, state);
            diagnostics.extend(recovered.diagnostics);
            Ok(recovered.syntax)
        },
    )
}

#[must_use]
pub(crate) fn parse_font_feature_value_block(
    source: &str,
    kind: crate::rule::font_feature_values::CssFontFeatureValueKind,
    parser_context: crate::CssParserContext,
) -> crate::CssParseReport<
    Option<
        crate::rule::CssBlockFragment<crate::rule::font_feature_values::CssFontFeatureValueBlock>,
    >,
> {
    real_brace_block(
        source,
        "later.rule.font-feature-values",
        parser_context,
        move |source, input, diagnostics, state| {
            let recovered = font_feature_values::parse_definition_body(source, input, kind, state);
            let block = crate::rule::font_feature_values::CssFontFeatureValueBlock::try_new(
                kind,
                recovered.syntax,
            )
            .map_err(|_| {
                crate::diagnostic::error::invalid_syntax(input.current_source_location())
            })?;
            diagnostics.extend(recovered.diagnostics);
            Ok(block)
        },
    )
}

pub(crate) fn parse_keyframes_block_with_context(
    source: &str,
    parser_context: crate::CssParserContext,
) -> crate::CssParseReport<
    Option<crate::rule::CssBlockFragment<Vec<crate::rule::keyframes::CssKeyframeBlock>>>,
> {
    real_brace_block(
        source,
        "baseline.rule.keyframes",
        parser_context,
        |source, input, diagnostics, state| {
            keyframes::parse_body(source, input, diagnostics, state)
        },
    )
}

pub(crate) fn parse_keyframe_declaration_block_with_context(
    source: &str,
    parser_context: crate::CssParserContext,
) -> crate::CssParseReport<
    Option<crate::rule::CssBlockFragment<crate::rule::keyframes::CssKeyframeDeclarationList>>,
> {
    real_brace_block(
        source,
        "baseline.keyframes.block",
        parser_context,
        keyframes::parse_declarations,
    )
}

pub(crate) fn parse_page_block_with_context(
    source: &str,
    parser_context: crate::CssParserContext,
) -> crate::CssParseReport<Option<crate::rule::CssBlockFragment<crate::rule::page::CssPageBody>>> {
    real_brace_block(
        source,
        "later.rule.page",
        parser_context,
        |source, input, diagnostics, state| Ok(page::parse_body(source, input, diagnostics, state)),
    )
}

pub(crate) fn parse_declaration_with_context(
    source: &str,
    parser_context: crate::CssParserContext,
) -> crate::CssParseReport<Option<CssDeclaration>> {
    bounded(source, || {
        let state = RecoveryState::at_depth(source, 0, StyleContextCaptures::default())
            .with_parser_context(parser_context);
        let working_source = crate::tokenization::prepare(source);
        let mut parser_input = ParserInput::new(&working_source);
        let mut input = Parser::new(&mut parser_input);
        let result = (|| {
            input.skip_whitespace();
            let declaration_start = input.state();
            let name = input.expect_ident_cloned()?;
            input.expect_colon()?;
            let openings = state.check_component_values(source, &input, "css.declaration")?;
            // RuleBodyParser normally supplies list boundaries. This singular
            // entry instead rejects all root delimiters, while letting the
            // tokenizer skip nested blocks (including custom-property braces).
            let value_start = input.state();
            loop {
                let location = input.current_source_location();
                let Ok(token) = input.next_including_whitespace_and_comments().cloned() else {
                    break;
                };
                if matches!(
                    token,
                    Token::Semicolon
                        | Token::CloseCurlyBracket
                        | Token::CloseParenthesis
                        | Token::CloseSquareBracket
                ) {
                    return Err(location.new_unexpected_token_error::<CssError>(token));
                }
                finish_nested_component(&mut input, &token)?;
            }
            input.reset(&value_start);
            let declaration = parse_declaration_core(
                DeclarationMode::Ordinary,
                name,
                &mut input,
                &declaration_start,
                state.source_snapshot(),
                parser_context,
            )?;
            input.expect_exhausted()?;
            state.retain_component_closures(openings);
            state.retain_navigation_diagnostic(&declaration.body);
            Ok(declaration.into_declaration())
        })();
        match result {
            Ok(declaration) => crate::CssParseReport::new(
                Some(declaration),
                state.take_implicit_closure_diagnostics(source),
            ),
            Err(error) => crate::CssParseReport::new(
                None,
                vec![reject(
                    source,
                    error,
                    crate::diagnostic::CssRecoveryAction::RejectInput,
                )],
            ),
        }
    })
}

fn selector_fragment<T>(
    source: &str,
    context: &CssNamespaceContext,
    single: bool,
    parse: impl for<'i> FnOnce(
        &mut Parser<'i, '_>,
        &mut SelectorRecovery<'_>,
    ) -> Result<T, ParseError<'i, CssError>>,
) -> crate::CssParseReport<Option<T>> {
    let state = RecoveryState::at_depth(source, 0, StyleContextCaptures::default());
    if let Some(name) = &context.0.default {
        state.activate_namespace(None, name.clone());
    }
    for (prefix, name) in &context.0.named {
        state.activate_namespace(Some(prefix.clone()), name.clone());
    }
    selector_fragment_with_state(source, state, single, parse)
}

pub(super) fn selector_fragment_with_state<T>(
    source: &str,
    state: RecoveryState,
    single: bool,
    parse: impl for<'i> FnOnce(
        &mut Parser<'i, '_>,
        &mut SelectorRecovery<'_>,
    ) -> Result<T, ParseError<'i, CssError>>,
) -> crate::CssParseReport<Option<T>> {
    let working_source = crate::tokenization::prepare(source);
    let mut input = ParserInput::new(&working_source);
    let mut input = Parser::new(&mut input);
    let mut diagnostics = Vec::new();
    let openings = if single {
        state.check_comma_member_components(source, &input, "baseline.selector.complex")
    } else {
        state.check_specialized_components(source, &input, "baseline.selector.complex")
    };
    let result = openings.and_then(|mut openings| {
        let value = parse(
            &mut input,
            &mut SelectorRecovery::new(source, &mut diagnostics, state.clone()),
        )?;
        input
            .expect_exhausted()
            .map_err(crate::diagnostic::error::selector_basic)?;
        // Preflight sees every lexical opening. Forgiving recovery can discard
        // a whole member, so its functions must not claim semantic retention.
        openings.retain(|opening| {
            !diagnostics.iter().any(|diagnostic| {
                diagnostic.action() == crate::diagnostic::CssRecoveryAction::DropSelectorListItem
                    && diagnostic.span().start().byte_offset().value() <= *opening
                    && *opening < diagnostic.span().end().byte_offset().value()
            })
        });
        state.retain_component_closures(openings);
        Ok(value)
    });
    let syntax = match result {
        Ok(value) => {
            diagnostics.extend(state.take_implicit_closure_diagnostics(source));
            Some(value)
        }
        Err(error) => {
            diagnostics.push(reject(
                source,
                error,
                crate::diagnostic::CssRecoveryAction::RejectInput,
            ));
            None
        }
    };
    crate::CssParseReport::new(syntax, diagnostics)
}

pub(crate) fn parse_selector(
    source: &str,
    context: &CssNamespaceContext,
) -> crate::CssParseReport<Option<CssSelector>> {
    bounded(source, || {
        selector_fragment(source, context, true, selectors::parse_rule_selector)
    })
}

pub(crate) fn parse_relative_selector_list(
    source: &str,
    context: &CssNamespaceContext,
) -> crate::CssParseReport<Option<CssRelativeSelectorList>> {
    bounded(source, || {
        selector_fragment(
            source,
            context,
            false,
            selectors::parse_general_relative_selector_list,
        )
    })
}

/// The never-matching replacement covering a refused media source's significant extent.
fn never_media_query(snapshot: &crate::source::CssSourceSnapshot) -> CssMediaQuery {
    let source = snapshot.as_str();
    CssMediaQuery::Never(CssNeverMediaQuery::new(
        crate::source::CssParsedOrigin::from_range(
            snapshot,
            recovery::first_non_trivia_position(source, 0, source.len())
                .byte_offset()
                .value()..source.len(),
        )
        .expect("parsed media recovery origin"),
    ))
}

pub(crate) fn parse_media_query(source: &str) -> crate::CssParseReport<CssMediaQuery> {
    let refused = || never_media_query(&crate::source::CssSourceSnapshot::new(source));
    bounded_or(source, refused, || {
        let state = RecoveryState::at_depth(source, 0, StyleContextCaptures::default());
        let working_source = crate::tokenization::prepare(source);
        let mut input = ParserInput::new(&working_source);
        let mut input = Parser::new(&mut input);
        let result = queries::check_media_member_components(source, &mut input, &state).and_then(
            |openings| {
                let query = queries::parse_media_query(
                    source,
                    &mut input,
                    &crate::numeric::expression::NumericInputContext::parsed(
                        state.source_snapshot(),
                    ),
                )?;
                input.expect_exhausted()?;
                state.retain_component_closures(openings);
                Ok(query)
            },
        );
        match result {
            Ok(query) => {
                crate::CssParseReport::new(query, state.take_implicit_closure_diagnostics(source))
            }
            Err(error) => crate::CssParseReport::new(
                CssMediaQuery::Never(CssNeverMediaQuery::new(
                    crate::source::CssParsedOrigin::from_range(
                        state.source_snapshot(),
                        recovery::first_non_trivia_position(source, 0, source.len())
                            .byte_offset()
                            .value()..source.len(),
                    )
                    .expect("parsed media recovery origin"),
                )),
                vec![reject(
                    source,
                    if queries::media_terminal_error(&error) {
                        error
                    } else {
                        with_media_query_context(error, None)
                    },
                    crate::diagnostic::CssRecoveryAction::ReplaceMediaQueryWithNever,
                )],
            ),
        }
    })
}

pub(crate) fn parse_media_query_list_snapshot(
    snapshot: &crate::source::CssSourceSnapshot,
    context: crate::CssParserContext,
) -> crate::CssParseReport<CssMediaQueryList> {
    let source = snapshot.as_str();
    let refused = || CssMediaQueryList::new(vec![never_media_query(snapshot)]);
    let (mut list, mut diagnostics) = bounded_or(source, refused, || {
        let state = RecoveryState::at_depth_with_snapshot(
            source,
            0,
            StyleContextCaptures::default(),
            snapshot.clone(),
        )
        .with_parser_context(context);
        let working_source = crate::tokenization::prepare(source);
        let mut input = ParserInput::new(&working_source);
        let mut input = Parser::new(&mut input);
        let mut diagnostics = Vec::new();
        let query = match queries::parse_media_query_list_with_closures(
            source,
            &mut input,
            &mut diagnostics,
            &state,
        ) {
            Ok(parsed) => {
                state.retain_component_closures(parsed.implicit_closures);
                parsed.queries
            }
            Err(error) => {
                diagnostics.push(reject(
                    source,
                    error,
                    crate::diagnostic::CssRecoveryAction::ReplaceMediaQueryWithNever,
                ));
                CssMediaQueryList::new(vec![CssMediaQuery::Never(CssNeverMediaQuery::new(
                    crate::source::CssParsedOrigin::from_range(
                        state.source_snapshot(),
                        recovery::first_non_trivia_position(source, 0, source.len())
                            .byte_offset()
                            .value()..source.len(),
                    )
                    .expect("parsed media recovery origin"),
                ))])
            }
        };
        diagnostics.extend(state.take_implicit_closure_diagnostics(source));
        crate::CssParseReport::new(query, diagnostics)
    })
    .into_parts();
    list.bind_parse_source(crate::source::CssParseSource::new(
        snapshot.clone(),
        context,
    ));
    for diagnostic in &mut diagnostics {
        diagnostic.bind_source(snapshot);
    }
    crate::CssParseReport::new(list, diagnostics)
}

pub(crate) fn parse_cssom_media_query(
    source: &str,
    context: crate::CssParserContext,
) -> crate::CssParseReport<Option<CssMediaQuery>> {
    let (list, mut diagnostics) =
        parse_media_query_list_snapshot(&crate::source::CssSourceSnapshot::new(source), context)
            .into_parts();
    // Condition depth counts `not` and grouping levels, so shallow source can
    // still need the bounded thread here. Without it the query is refused.
    let deep = list
        .queries()
        .iter()
        .find(|query| crate::query::media::construction::query_is_deep(query))
        .map(|query| query.origin().clone());
    let mut list = Some(list);
    let single = super::bounded_execution::run_deep(deep.is_some(), || {
        list.take()
            .expect("one single-query projection")
            .into_single_query()
    });
    match single {
        Ok(query) => crate::CssParseReport::new(query, diagnostics),
        Err(unavailable) => {
            let (at, snapshot) = match &deep {
                Some(crate::component::CssValueOrigin::Parsed(origin)) => (
                    origin.span().start().byte_offset().value(),
                    Some(origin.source().clone()),
                ),
                _ => (0, None),
            };
            let mut refusal = unavailable.diagnostic_at(source, at);
            if let Some(snapshot) = &snapshot {
                refusal.bind_source(snapshot);
            }
            diagnostics.push(refusal);
            crate::CssParseReport::new(None, diagnostics)
        }
    }
}

pub(crate) fn parse_font_face_descriptor_value(
    source: &str,
    descriptor: CssFontFaceDescriptorKind,
) -> crate::CssParseReport<Option<crate::rule::font_face::CssAuthoredFontFaceDescriptorValue>> {
    bounded(source, || {
        let state = RecoveryState::at_depth(source, 0, StyleContextCaptures::default());
        let working_source = crate::tokenization::prepare(source);
        let mut parser_input = ParserInput::new(&working_source);
        let mut input = Parser::new(&mut parser_input);
        let mut diagnostics = Vec::new();
        let result = (|| {
            let mut openings = state.check_component_values(source, &input, "css.descriptor")?;
            // A stylesheet parser bounds descriptor values before semicolons. A raw
            // value has no such enclosing parser; reject its own delimiters before
            // src member recovery could discard them as part of an invalid member.
            let value = font_face::parse_authored_font_face_value(
                source,
                &mut input,
                descriptor,
                state.source_snapshot(),
                &crate::numeric::expression::NumericInputContext::parsed(state.source_snapshot()),
                &mut diagnostics,
                &mut openings,
            )?;
            input.expect_exhausted().map_err(|error| {
                crate::diagnostic::error::with_descriptor_context(
                    error.into(),
                    "font-face",
                    descriptor.as_str(),
                )
            })?;
            state.retain_component_closures(openings);
            Ok(value)
        })();
        let syntax = match result {
            Ok(value) => {
                diagnostics.extend(state.take_implicit_closure_diagnostics(source));
                Some(value)
            }
            Err(error) => {
                // Failed enclosing values cannot claim partial member retention.
                diagnostics.clear();
                diagnostics.push(reject(
                    source,
                    error,
                    crate::diagnostic::CssRecoveryAction::RejectInput,
                ));
                None
            }
        };
        crate::CssParseReport::new(syntax, diagnostics)
    })
}

#[must_use]
pub(crate) fn parse_page_selector_list(
    source: &str,
) -> crate::CssParseReport<Option<crate::rule::page::CssPageSelectorList>> {
    descriptor_value_fragment(source, None, |input, state| {
        page::parse_page_selector(input, state.source_snapshot())
    })
}
#[must_use]
pub(crate) fn parse_page_descriptor_value(
    source: &str,
    kind: crate::rule::page::CssPageDescriptorKind,
) -> crate::CssParseReport<Option<crate::rule::page::CssPageDescriptorValue>> {
    descriptor_value_fragment(source, Some(("page", kind.as_str())), |input, state| {
        let (value, components, origin) =
            collect_declaration_value(input, state.source_snapshot(), |input| {
                page::parse_descriptor_value(input, kind, state.source_snapshot())
            })?;
        Ok(crate::rule::page::CssPageDescriptorValue::from_parsed(
            kind, value, components, origin,
        ))
    })
}
pub(crate) fn parse_margin_block_with_context(
    source: &str,
    context: crate::CssParserContext,
) -> crate::CssParseReport<
    Option<crate::rule::CssBlockFragment<crate::rule::page::CssMarginDeclarationBlock>>,
> {
    real_brace_block(
        source,
        "later.rule.page",
        context,
        |source, input, diagnostics, state| {
            Ok(page::parse_margin_body(source, input, diagnostics, state))
        },
    )
}

// These name-free fronts share whole-source rejection, descriptor annotation
// policy and lexical/resource ownership; each closure uses its family provider.
fn descriptor_value_fragment<T: Send>(
    source: &str,
    descriptor: Option<(&str, &str)>,
    parse: impl for<'i, 't> FnOnce(
        &mut Parser<'i, 't>,
        &RecoveryState,
    ) -> Result<T, ParseError<'i, CssError>>
    + Send,
) -> crate::CssParseReport<Option<T>> {
    bounded(source, || {
        let state = RecoveryState::at_depth(source, 0, StyleContextCaptures::default());
        let working_source = crate::tokenization::prepare(source);
        let mut parser_input = ParserInput::new(&working_source);
        let mut input = Parser::new(&mut parser_input);
        let result = (|| {
            let openings = state.check_component_values(source, &input, "css.descriptor")?;
            let value = if let Some((owner, descriptor)) = descriptor {
                parse_descriptor_boundary(&mut input, owner, descriptor, |input| {
                    let value = parse(input, &state)?;
                    input.expect_exhausted().map_err(basic)?;
                    Ok(value)
                })?
            } else {
                parse(&mut input, &state)?
            };
            input.expect_exhausted().map_err(basic)?;
            state.retain_component_closures(openings);
            Ok(value)
        })()
        .map_err(|error| match descriptor {
            Some((owner, descriptor)) => {
                crate::diagnostic::error::with_descriptor_value_context(error, owner, descriptor)
            }
            None => error,
        });
        let (syntax, diagnostics) = match result {
            Ok(value) => (Some(value), state.take_implicit_closure_diagnostics(source)),
            Err(error) => (
                None,
                vec![reject(
                    source,
                    error,
                    crate::diagnostic::CssRecoveryAction::RejectInput,
                )],
            ),
        };
        crate::CssParseReport::new(syntax, diagnostics)
    })
}

#[must_use]
pub(crate) fn parse_counter_style_descriptor_value(
    source: &str,
    descriptor: crate::rule::counter_style::CssCounterStyleDescriptorKind,
) -> crate::CssParseReport<Option<crate::rule::counter_style::CssCounterStyleDescriptorValue>> {
    descriptor_value_fragment(
        source,
        Some(("counter-style", descriptor.as_str())),
        |input, state| {
            let (value, components, origin) =
                collect_declaration_value(input, state.source_snapshot(), |input| {
                    // Keep the collector's complete source range, while an
                    // empty grammar reports the EOF after leading trivia.
                    input.skip_whitespace();
                    counter_style::parse_authored_value(input, descriptor, state.source_snapshot())
                })?;
            Ok(
                crate::rule::counter_style::CssCounterStyleDescriptorValue::from_parsed(
                    descriptor, value, components, origin,
                ),
            )
        },
    )
}

#[must_use]
pub(crate) fn parse_font_feature_display_value(
    source: &str,
) -> crate::CssParseReport<Option<crate::rule::font_feature_values::CssFontFeatureDisplayValue>> {
    descriptor_value_fragment(
        source,
        Some(("font-feature-values", "font-display")),
        |input, state| {
            let (value, components, origin) =
                collect_declaration_value(input, state.source_snapshot(), |input| {
                    let value =
                        font_feature_values::parse_display_value(input, state.source_snapshot())?;
                    input.expect_exhausted().map_err(basic)?;
                    Ok(value)
                })?;
            Ok(
                crate::rule::font_feature_values::CssFontFeatureDisplayValue::from_parsed(
                    value, components, origin,
                ),
            )
        },
    )
}

#[must_use]
pub(crate) fn parse_font_feature_value(
    source: &str,
    kind: crate::rule::font_feature_values::CssFontFeatureValueKind,
) -> crate::CssParseReport<Option<crate::rule::font_feature_values::CssFontFeatureValue>> {
    descriptor_value_fragment(source, None, |input, state| {
        let (value, components, origin) =
            collect_declaration_value(input, state.source_snapshot(), |input| {
                font_feature_values::parse_feature_value(input, kind, state.source_snapshot())
            })?;
        Ok(
            crate::rule::font_feature_values::CssFontFeatureValue::from_parsed(
                kind, value, components, origin,
            ),
        )
    })
}

pub(crate) fn parse_font_palette_descriptor_value(
    source: &str,
    descriptor: CssFontPaletteDescriptorKind,
) -> crate::CssParseReport<Option<CssFontPaletteDescriptorValue>> {
    bounded(source, || {
        let state = RecoveryState::at_depth(source, 0, StyleContextCaptures::default());
        let working_source = crate::tokenization::prepare(source);
        let mut parser_input = ParserInput::new(&working_source);
        let mut input = Parser::new(&mut parser_input);
        let result = (|| {
            let openings = state.check_component_values(source, &input, "css.descriptor")?;
            let value = font_palette_values::parse_descriptor_value_from_parser(
                &mut input, descriptor, &state,
            )?;
            input.expect_exhausted().map_err(basic)?;
            state.retain_component_closures(openings);
            Ok(value)
        })()
        .map_err(|error| {
            crate::diagnostic::error::with_descriptor_value_context(
                error,
                "font-palette-values",
                descriptor.as_str(),
            )
        });
        let (syntax, diagnostics) = match result {
            Ok(value) => (Some(value), state.take_implicit_closure_diagnostics(source)),
            Err(error) => (
                None,
                vec![reject(
                    source,
                    error,
                    crate::diagnostic::CssRecoveryAction::RejectInput,
                )],
            ),
        };
        crate::CssParseReport::new(syntax, diagnostics)
    })
}

pub(crate) fn parse_color_profile_descriptor_value(
    source: &str,
    descriptor: CssColorProfileDescriptorKind,
) -> crate::CssParseReport<Option<CssColorProfileDescriptorValue>> {
    bounded(source, || {
        let state = RecoveryState::at_depth(source, 0, StyleContextCaptures::default());
        let working_source = crate::tokenization::prepare(source);
        let mut parser_input = ParserInput::new(&working_source);
        let mut input = Parser::new(&mut parser_input);
        let result = (|| {
            let openings = state.check_component_values(source, &input, "css.descriptor")?;
            let value =
                color_profile::parse_descriptor_value_from_parser(&mut input, descriptor, &state)?;
            input.expect_exhausted().map_err(basic)?;
            state.retain_component_closures(openings);
            Ok(value)
        })()
        .map_err(|error| {
            crate::diagnostic::error::with_descriptor_value_context(
                error,
                "color-profile",
                descriptor.as_str(),
            )
        });
        let (syntax, diagnostics) = match result {
            Ok(value) => (Some(value), state.take_implicit_closure_diagnostics(source)),
            Err(error) => (
                None,
                vec![reject(
                    source,
                    error,
                    crate::diagnostic::CssRecoveryAction::RejectInput,
                )],
            ),
        };
        crate::CssParseReport::new(syntax, diagnostics)
    })
}

pub(crate) fn parse_property_value_text_with_context(
    source: &str,
    property: CssPropertyNameRef<'_>,
    importance: CssImportance,
    parser_context: crate::CssParserContext,
) -> crate::CssParseReport<Option<CssDeclaration>> {
    let grammar = match property {
        CssPropertyNameRef::Known(property) => PropertyValueGrammar::Known(property.grammar()),
        CssPropertyNameRef::Custom(name) => PropertyValueGrammar::Custom(name),
        CssPropertyNameRef::SvgGlyphOrientationVertical => PropertyValueGrammar::SvgGlyph(
            crate::property::svg_glyph::SvgGlyphAdmission::css(parser_context),
        ),
    };
    property_value_text(source, grammar, importance, parser_context)
}

pub(crate) fn parse_property_value_text_for_grammar_with_context(
    source: &str,
    grammar: CssPropertyGrammar,
    importance: CssImportance,
    parser_context: crate::CssParserContext,
) -> crate::CssParseReport<Option<CssDeclaration>> {
    property_value_text(
        source,
        PropertyValueGrammar::Known(grammar),
        importance,
        parser_context,
    )
}

pub(crate) fn parse_svg_glyph_attribute_value(
    source: &str,
    context: crate::CssParserContext,
) -> crate::CssParseReport<Option<CssDeclaration>> {
    property_value_text(
        source,
        PropertyValueGrammar::SvgGlyph(crate::property::svg_glyph::SvgGlyphAdmission::attribute(
            context,
        )),
        CssImportance::Normal,
        context,
    )
}

fn property_value_text(
    source: &str,
    grammar: PropertyValueGrammar<'_>,
    importance: CssImportance,
    parser_context: crate::CssParserContext,
) -> crate::CssParseReport<Option<CssDeclaration>> {
    bounded(source, || {
        let state = RecoveryState::at_depth(source, 0, StyleContextCaptures::default())
            .with_parser_context(parser_context);
        let working_source = crate::tokenization::prepare(source);
        let mut parser_input = ParserInput::new(&working_source);
        let mut input = Parser::new(&mut parser_input);
        let result = (|| {
            let openings = state.check_component_values(source, &input, "css.declaration")?;
            let document = crate::syntax_consumption::normalize(
                crate::syntax_consumption::SyntaxInput::Source(
                    crate::syntax_consumption::SourceWindow {
                        text: std::borrow::Cow::Borrowed(source),
                        range: 0..source.len(),
                        original: state.source_snapshot().clone(),
                    },
                ),
                crate::syntax_consumption::SyntaxInputLimits::default(),
                0,
            )
            .map_err(|error| {
                crate::diagnostic::error::invalid_component_value(
                    input.current_source_location(),
                    error,
                )
            })?;
            let mut property_input = property_input::Parser::new(&document);
            let body = parse_property_value_from_parser(
                grammar,
                &mut property_input,
                &crate::numeric::expression::NumericInputContext::parsed(state.source_snapshot()),
                parser_context,
            )
            .map_err(|error| {
                let error = error.into_source_error();
                let position = error.position();
                cssparser::SourceLocation {
                    line: position.line().value(),
                    column: position.column().value() + 1,
                }
                .new_custom_error(error)
            })?;
            // Retain owned source syntax only after grammar admission succeeds.
            let components =
                CssComponentValues::collect_from_parser(&mut input, state.source_snapshot())
                    .map_err(|error| {
                        crate::diagnostic::error::invalid_component_value(
                            input.current_source_location(),
                            error,
                        )
                    })?;
            let origin = CssParsedOrigin::from_range(state.source_snapshot(), 0..source.len())
                .expect("complete raw property value belongs to its source");
            input.expect_exhausted()?;
            state.retain_component_closures(openings);
            state.retain_navigation_diagnostic(&body);
            Ok(CssDeclaration::new_parsed_value(
                parser_context,
                body,
                importance,
                components,
                origin,
            ))
        })();
        match result {
            Ok(declaration) => crate::CssParseReport::new(
                Some(declaration),
                state.take_implicit_closure_diagnostics(source),
            ),
            Err(error) => crate::CssParseReport::new(
                None,
                vec![reject(
                    source,
                    error,
                    crate::diagnostic::CssRecoveryAction::RejectInput,
                )],
            ),
        }
    })
}

// Distinguish a rejected first rule from parse_one_rule's trailing-input error.
// The latter must not be reinterpreted as an error in the completed rule body.
struct SingleRuleParser<'s> {
    grammar: RuleParser<'s>,
    completed: bool,
}

impl<'i> AtRuleParser<'i> for SingleRuleParser<'i> {
    type Prelude = AtRulePrelude;
    type AtRule = Vec<CssRule>;
    type Error = CssError;

    fn parse_prelude<'t>(
        &mut self,
        name: CowRcStr<'i>,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self::Prelude, ParseError<'i, CssError>> {
        AtRuleParser::parse_prelude(&mut self.grammar, name, input)
    }

    fn rule_without_block(
        &mut self,
        prelude: Self::Prelude,
        start: &ParserState,
    ) -> Result<Self::AtRule, ()> {
        let result = self.grammar.rule_without_block(prelude, start);
        self.completed = result.is_ok();
        result.map(RuleListItem::into_rules)
    }

    fn parse_block<'t>(
        &mut self,
        prelude: Self::Prelude,
        start: &ParserState,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self::AtRule, ParseError<'i, CssError>> {
        let result = AtRuleParser::parse_block(&mut self.grammar, prelude, start, input);
        self.completed = result.is_ok();
        result.map(RuleListItem::into_rules)
    }
}

impl<'i> QualifiedRuleParser<'i> for SingleRuleParser<'i> {
    type Prelude = CssStyleSelectorList;
    type QualifiedRule = Vec<CssRule>;
    type Error = CssError;

    fn parse_prelude<'t>(
        &mut self,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self::Prelude, ParseError<'i, CssError>> {
        QualifiedRuleParser::parse_prelude(&mut self.grammar, input)
    }

    fn parse_block<'t>(
        &mut self,
        prelude: Self::Prelude,
        start: &ParserState,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self::QualifiedRule, ParseError<'i, CssError>> {
        let result = QualifiedRuleParser::parse_block(&mut self.grammar, prelude, start, input);
        self.completed = result.is_ok();
        result.map(RuleListItem::into_rules)
    }
}

pub(crate) fn parse_rule_with_context(
    source: &str,
    context: &CssNamespaceContext,
    parser_context: crate::CssParserContext,
) -> crate::CssParseReport<Option<CssRule>> {
    bounded(source, || {
        let snapshot = CssSourceSnapshot::new(source);
        let report = parse_bounded(
            source,
            &snapshot,
            0,
            BoundedParseContext::OneRule,
            StyleContextCaptures::with_namespaces(context),
            parser_context,
        );
        let (syntax, diagnostics) = report.into_parts();
        let BoundedParseSyntax::OneRule(rule) = syntax else {
            unreachable!("exact-one replay preserves its fragment carrier")
        };
        match rule {
            Some(rule) => conditional_chains::rule(source, rule, diagnostics),
            None => crate::CssParseReport::new(None, diagnostics),
        }
    })
}

pub(super) fn parse_rule_inner(
    source: &str,
    state: RecoveryState,
) -> crate::CssParseReport<Option<CssRule>> {
    let working_source = crate::tokenization::prepare(source);
    let mut parser_input = ParserInput::new(&working_source);
    let mut input = Parser::new(&mut parser_input);
    let mut parser = SingleRuleParser {
        grammar: RuleParser::top_level(source, state.clone()),
        completed: false,
    };
    input.skip_whitespace();
    let rule_start = input.position().byte_index();
    match super::syntax_bridge::one(source, &mut input, &mut parser, &state) {
        Ok(rules) => {
            // The ordinary grammar emits one outer node for a successful rule.
            let [rule]: [CssRule; 1] = rules
                .try_into()
                .expect("a successful isolated ordinary rule emits one outer node");
            let mut diagnostics = parser.grammar.diagnostics;
            diagnostics.extend(state.take_implicit_closure_diagnostics(source));
            crate::CssParseReport::new(Some(rule), diagnostics)
        }
        Err(error) => {
            let action = recovery_action_for_error(
                &error,
                crate::diagnostic::CssRecoveryAction::RejectInput,
            );
            let error = if parser.completed {
                from_parse_error(source, error)
            } else {
                let failed_unit = &source[rule_start..input.position().byte_index()];
                from_rule_parse_error(source, failed_unit, error)
            };
            crate::CssParseReport::new(None, vec![reject_resolved(source, error, action)])
        }
    }
}

pub(crate) fn parse_style_block_with_context(
    source: &str,
    context: &CssNamespaceContext,
    parser_context: crate::CssParserContext,
) -> crate::CssParseReport<Option<CssStyleBlock>> {
    bounded(source, || {
        let snapshot = CssSourceSnapshot::new(source);
        let report = parse_bounded(
            source,
            &snapshot,
            0,
            BoundedParseContext::StyleBlock,
            StyleContextCaptures::with_namespaces(context),
            parser_context,
        );
        let (syntax, diagnostics) = report.into_parts();
        let BoundedParseSyntax::StyleBlock(block) = syntax else {
            unreachable!("style-block replay preserves its genuine-brace carrier")
        };
        match block {
            Some(block) => conditional_chains::style_block(source, block, diagnostics),
            None => crate::CssParseReport::new(None, diagnostics),
        }
    })
}

pub(super) fn parse_style_block_inner(
    source: &str,
    state: RecoveryState,
) -> crate::CssParseReport<Option<CssStyleBlock>> {
    let working_source = crate::tokenization::prepare(source);
    let mut parser_input = ParserInput::new(&working_source);
    let mut input = Parser::new(&mut parser_input);
    let result = (|| {
        input.skip_whitespace();
        let start = input.position().byte_index();
        input.expect_curly_bracket_block()?;
        let recovered = input.parse_nested_block(|input| {
            let mut depth = state.enter_rule_block(source, input, "official.value.style-block")?;
            let recovered = parse_style_contents(source, input, state.clone())?;
            depth.retain();
            Ok(recovered)
        })?;
        let end = input.position().byte_index();
        input.expect_exhausted()?;
        let origin = CssParsedOrigin::from_range(state.source_snapshot(), start..end)
            .expect("consumed block boundaries belong to the original source");
        Ok((
            CssStyleBlock::new(recovered.syntax, origin),
            recovered.diagnostics,
        ))
    })();
    match result {
        Ok((block, mut diagnostics)) => {
            diagnostics.extend(state.take_implicit_closure_diagnostics(source));
            crate::CssParseReport::new(Some(block), diagnostics)
        }
        Err(error) => crate::CssParseReport::new(
            None,
            vec![reject(
                source,
                error,
                crate::diagnostic::CssRecoveryAction::RejectInput,
            )],
        ),
    }
}
