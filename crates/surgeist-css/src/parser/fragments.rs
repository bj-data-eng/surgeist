//! Context-specific parsing over the caller's unmodified source.
use super::*;

fn bounded<T: Send>(source: &str, parse: impl FnOnce() -> T + Send) -> T {
    // Selector recursion has larger frames than structural rule parsing. Route
    // deeply nested fragments conservatively; this is not an admission limit.
    if recovery::maximum_nested_depth(source) < 64 {
        return parse();
    }
    std::thread::scope(|scope| {
        let thread = std::thread::Builder::new()
            .name("surgeist-css-fragment-parser".into())
            .stack_size(16 * 1024 * 1024)
            .spawn_scoped(scope, parse)
            .expect("bounded CSS parser thread must be available");
        match thread.join() {
            Ok(value) => value,
            Err(panic) => std::panic::resume_unwind(panic),
        }
    })
}

fn reject(
    source: &str,
    error: ParseError<'_, Error>,
    action: crate::CssRecoveryAction,
) -> crate::CssRecoveryDiagnostic {
    let action = recovery_action_for_error(&error, action);
    let span = crate::CssSourceSpan::new(
        crate::CssSourcePosition::from_byte_offset_in(source, 0),
        crate::CssSourcePosition::from_byte_offset_in(source, source.len()),
    )
    .expect("complete source span");
    crate::CssRecoveryDiagnostic::new(from_parse_error(source, error), span, action)
        .expect("fragment errors originate within the complete source")
}

/// Parses exactly one complete, grammar-valid ordinary declaration from raw source.
///
/// Surrounding whitespace and comments are accepted. The source must contain a
/// recognized property or a valid custom property, its colon, and its value;
/// top-level semicolons and stray closing delimiters reject the entire input.
/// Use [`super::parse_style_attribute`] for a declaration list with separators.
/// Name and value origins share the original source snapshot, and importance is
/// recognized by the ordinary declaration grammar. No annotation span is added.
/// Rejection returns `None` with `RejectInput`, except resource exhaustion retains
/// `StopAtNestingLimit`. Implicit EOF closures are reported only after the complete
/// declaration survives. This validates property grammar beyond CSS Syntax's
/// generic consume-declaration algorithm and performs no contextual resolution.
pub fn parse_declaration(source: &str) -> crate::CssParseReport<Option<CssDeclaration>> {
    bounded(source, || {
        let state = RecoveryState::at_depth(source, 0, StyleContextCaptures::default());
        let mut parser_input = ParserInput::new(source);
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
                    return Err(location.new_unexpected_token_error::<Error>(token));
                }
                if matches!(
                    token,
                    Token::Function(_)
                        | Token::ParenthesisBlock
                        | Token::SquareBracketBlock
                        | Token::CurlyBracketBlock
                ) {
                    // Finish this block now so the next root token's location
                    // is measured after it, rather than before tokenizer skipping.
                    input.parse_nested_block(|nested| {
                        while nested.next_including_whitespace_and_comments().is_ok() {}
                        Ok::<_, ParseError<'_, Error>>(())
                    })?;
                }
            }
            input.reset(&value_start);
            let declaration = parse_declaration_core(
                DeclarationMode::Ordinary,
                name,
                &mut input,
                &declaration_start,
                state.source_snapshot(),
            )?;
            input.expect_exhausted()?;
            state.retain_component_closures(openings);
            Ok(declaration.into_declaration())
        })();
        match result {
            Ok(declaration) => crate::CssParseReport::new(
                Some(declaration),
                state.take_implicit_closure_diagnostics(source),
            ),
            Err(error) => crate::CssParseReport::new(
                None,
                vec![reject(source, error, crate::CssRecoveryAction::RejectInput)],
            ),
        }
    })
}

fn selector_fragment<'i, T>(
    source: &'i str,
    context: &CssNamespaceContext,
    single: bool,
    parse: impl FnOnce(
        &mut Parser<'i, '_>,
        &mut SelectorRecovery<'_>,
    ) -> Result<T, ParseError<'i, Error>>,
) -> crate::CssParseReport<Option<T>> {
    let state = RecoveryState::at_depth(source, 0, StyleContextCaptures::default());
    if let Some(name) = &context.0.default {
        state.activate_namespace(None, name.clone());
    }
    for (prefix, name) in &context.0.named {
        state.activate_namespace(Some(prefix.clone()), name.clone());
    }
    let mut input = ParserInput::new(source);
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
            .map_err(crate::error::selector_basic)?;
        // Preflight sees every lexical opening. Forgiving recovery can discard
        // a whole member, so its functions must not claim semantic retention.
        openings.retain(|opening| {
            !diagnostics.iter().any(|diagnostic| {
                diagnostic.action() == crate::CssRecoveryAction::DropSelectorListItem
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
            diagnostics.push(reject(source, error, crate::CssRecoveryAction::RejectInput));
            None
        }
    };
    crate::CssParseReport::new(syntax, diagnostics)
}

/// Parses exactly one ordinary selector using the supplied namespace bindings.
///
/// Rejected outer syntax is `None` with diagnostics. Forgiving inner lists retain
/// valid members and report discarded members. Positions refer directly to `source`;
/// implicit EOF closures are reported only when the outer syntax is retained.
/// Relative selectors and nesting selectors are not admitted by this front door.
pub fn parse_selector(
    source: &str,
    context: &CssNamespaceContext,
) -> crate::CssParseReport<Option<CssSelector>> {
    bounded(source, || {
        selector_fragment(source, context, true, selectors::parse_rule_selector)
    })
}

/// Parses a complete ordinary selector list atomically, retaining forgiving inner recovery.
pub fn parse_selector_list(
    source: &str,
    context: &CssNamespaceContext,
) -> crate::CssParseReport<Option<CssStyleSelectorList>> {
    bounded(source, || {
        selector_fragment(source, context, false, |input, recovery| {
            selectors::parse_rule_selector_list(input, recovery).map(|selectors| {
                CssStyleSelectorList::new(
                    selectors
                        .into_iter()
                        .map(CssStyleSelector::Selector)
                        .collect(),
                )
            })
        })
    })
}

/// Parses exactly one media query, replacing rejected input with a never-matching query.
pub fn parse_media_query(source: &str) -> crate::CssParseReport<CssMediaQuery> {
    bounded(source, || {
        let state = RecoveryState::at_depth(source, 0, StyleContextCaptures::default());
        let mut input = ParserInput::new(source);
        let mut input = Parser::new(&mut input);
        let result = state
            .check_comma_member_components(source, &input, "baseline.media.query-list")
            .and_then(|openings| {
                let query = queries::parse_media_query(source, &mut input)?;
                input.expect_exhausted()?;
                state.retain_component_closures(openings);
                Ok(query)
            });
        match result {
            Ok(query) => {
                crate::CssParseReport::new(query, state.take_implicit_closure_diagnostics(source))
            }
            Err(error) => crate::CssParseReport::new(
                CssMediaQuery::Never(CssNeverMediaQuery::new(
                    recovery::first_non_trivia_position(source, 0, source.len()),
                )),
                vec![reject(
                    source,
                    if is_nesting_limit_error(&error) {
                        error
                    } else {
                        with_media_query_context(error, None)
                    },
                    crate::CssRecoveryAction::ReplaceMediaQueryWithNever,
                )],
            ),
        }
    })
}

/// Parses a complete media query list with independent recovery for each comma member.
///
/// An empty list is valid. Malformed members become `Never`, while grammatically
/// valid unknown features remain symbolic defined-false conditions. This operation
/// performs no matching or contextual evaluation.
pub fn parse_media_query_list(source: &str) -> crate::CssParseReport<CssMediaQueryList> {
    bounded(source, || {
        let state = RecoveryState::at_depth(source, 0, StyleContextCaptures::default());
        let mut input = ParserInput::new(source);
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
                    crate::CssRecoveryAction::ReplaceMediaQueryWithNever,
                ));
                CssMediaQueryList::new(vec![CssMediaQuery::Never(CssNeverMediaQuery::new(
                    recovery::first_non_trivia_position(source, 0, source.len()),
                ))])
            }
        };
        diagnostics.extend(state.take_implicit_closure_diagnostics(source));
        crate::CssParseReport::new(query, diagnostics)
    })
}

/// Parses one complete raw `@font-face` descriptor value in the selected grammar.
///
/// The source contains only the value, without a descriptor name, annotation or
/// declaration delimiter. The returned typed value has no fabricated name position.
/// Diagnostics use the original source's UTF-8 bytes, zero-based UTF-16 columns and
/// actual EOF. Invalid outer values return `None` with `RejectInput`; resource limits
/// retain `StopAtNestingLimit`. A retained `src` list can report discarded members.
/// Implicit closures are reported only for retained components. This does not require
/// surrounding `font-family` or `src` descriptors, match fonts, or load resources.
pub fn parse_font_face_descriptor_value(
    source: &str,
    descriptor: CssFontFaceDescriptorKind,
) -> crate::CssParseReport<Option<CssFontFaceDescriptorValue>> {
    bounded(source, || {
        let state = RecoveryState::at_depth(source, 0, StyleContextCaptures::default());
        let mut parser_input = ParserInput::new(source);
        let mut input = Parser::new(&mut parser_input);
        let mut diagnostics = Vec::new();
        let result = (|| {
            let mut openings = state.check_component_values(source, &input, "css.descriptor")?;
            // A stylesheet parser bounds descriptor values before semicolons. A raw
            // value has no such enclosing parser; reject its own delimiters before
            // src member recovery could discard them as part of an invalid member.
            let start = input.state();
            loop {
                let token_start = input.position();
                let location = input.current_source_location();
                let Ok(token) = input.next_including_whitespace_and_comments().cloned() else {
                    break;
                };
                if matches!(
                    token,
                    Token::Semicolon
                        | Token::CurlyBracketBlock
                        | Token::CloseCurlyBracket
                        | Token::CloseParenthesis
                        | Token::CloseSquareBracket
                ) {
                    return Err(crate::error::invalid_descriptor_token_at(
                        location,
                        "font-face",
                        descriptor.css_name(),
                        &token,
                        input.slice_from(token_start),
                    ));
                }
            }
            input.reset(&start);
            let value = font_face::parse_font_face_value(
                source,
                &mut input,
                descriptor,
                &mut diagnostics,
                &mut openings,
            )?;
            input.expect_exhausted().map_err(|error| {
                crate::error::with_descriptor_context(
                    error.into(),
                    "font-face",
                    descriptor.css_name(),
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
                diagnostics.push(reject(source, error, crate::CssRecoveryAction::RejectInput));
                None
            }
        };
        crate::CssParseReport::new(syntax, diagnostics)
    })
}
