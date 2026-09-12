//! Context-specific parsing over the caller's unmodified source.
use super::*;

fn bounded<T: Send>(
    source: &str,
    parse: impl FnOnce() -> crate::CssParseReport<T> + Send,
) -> crate::CssParseReport<T> {
    // Selector recursion has larger frames than structural rule parsing. Route
    // deeply nested fragments conservatively; this is not an admission limit.
    if recovery::maximum_nested_depth(source) < 64 {
        return recovery::finish_report(source, parse());
    }
    let report = std::thread::scope(|scope| {
        let thread = std::thread::Builder::new()
            .name("surgeist-css-fragment-parser".into())
            .stack_size(16 * 1024 * 1024)
            .spawn_scoped(scope, parse)
            .expect("bounded CSS parser thread must be available");
        match thread.join() {
            Ok(value) => value,
            Err(panic) => std::panic::resume_unwind(panic),
        }
    });
    recovery::finish_report(source, report)
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

pub(super) fn finish_nested_component<'i>(
    input: &mut Parser<'i, '_>,
    token: &Token<'i>,
) -> Result<(), ParseError<'i, Error>> {
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
                finish_nested_component(&mut input, &token)?;
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
/// Explicit `&` anchors remain symbolic; without a parent selector list they match
/// the context's scope elements and contribute zero specificity. Leading relative
/// combinators are not admitted by this front door.
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
        let result = queries::check_media_member_components(source, &mut input, &state).and_then(
            |openings| {
                let query = queries::parse_media_query(
                    source,
                    &mut input,
                    &crate::numeric::NumericInputContext::parsed(state.source_snapshot()),
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
                    crate::CssParsedOrigin::from_range(
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
                    crate::CssRecoveryAction::ReplaceMediaQueryWithNever,
                )],
            ),
        }
    })
}

/// Parses a complete media query list with independent recovery for each comma member.
///
/// An empty list is valid. Malformed members become `Never`, while grammatically
/// valid unknown features retain unknown truth. This operation
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
                    crate::CssParsedOrigin::from_range(
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
                finish_nested_component(&mut input, &token)?;
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

/// Parses a complete raw property value using a supplied semantic property name.
///
/// Importance is supplied separately: root annotations, semicolons and stray
/// closing delimiters reject the entire source, including after substitutions.
/// Empty custom values and nested custom-property punctuation are accepted.
/// The declaration has no parsed name or name position; its parsed value and
/// components share the original source snapshot. No source wrapper or value
/// serialization is introduced. Values remain authored and symbolic.
/// Invalid grammar returns `None` with `RejectInput`; resource exhaustion uses
/// `StopAtNestingLimit`. Implicit EOF closures are reported only on retention.
pub fn parse_property_value_text(
    source: &str,
    property: CssPropertyNameRef<'_>,
    importance: CssImportance,
) -> crate::CssParseReport<Option<CssDeclaration>> {
    let grammar = match property {
        CssPropertyNameRef::Known(property) => PropertyValueGrammar::Known(property.grammar()),
        CssPropertyNameRef::Custom(name) => PropertyValueGrammar::Custom(name),
    };
    property_value_text(source, grammar, importance)
}

/// Parses a complete raw value with an explicit canonical or legacy grammar.
///
/// This shares source provenance, separate importance, resource bounds and
/// complete-input rejection with [`parse_property_value_text`]. The declaration
/// retains the selected grammar for ordinary, CSS-wide and symbolic values.
pub fn parse_property_value_text_for_grammar(
    source: &str,
    grammar: CssPropertyGrammar,
    importance: CssImportance,
) -> crate::CssParseReport<Option<CssDeclaration>> {
    property_value_text(source, PropertyValueGrammar::Known(grammar), importance)
}

fn property_value_text(
    source: &str,
    grammar: PropertyValueGrammar<'_>,
    importance: CssImportance,
) -> crate::CssParseReport<Option<CssDeclaration>> {
    bounded(source, || {
        let state = RecoveryState::at_depth(source, 0, StyleContextCaptures::default());
        let mut parser_input = ParserInput::new(source);
        let mut input = Parser::new(&mut parser_input);
        let result = (|| {
            let openings = state.check_component_values(source, &input, "css.declaration")?;
            let (body, components, origin) =
                collect_declaration_value(&mut input, state.source_snapshot(), |input| {
                    parse_property_value_from_parser(
                        grammar,
                        source,
                        input,
                        &crate::numeric::NumericInputContext::parsed(state.source_snapshot()),
                    )
                })?;
            input.expect_exhausted()?;
            state.retain_component_closures(openings);
            Ok(CssDeclaration::new_parsed_value(
                body, importance, components, origin,
            ))
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

// Distinguish a rejected first rule from parse_one_rule's trailing-input error.
// The latter must not be reinterpreted as an error in the completed rule body.
struct SingleRuleParser<'s> {
    grammar: StrictRuleParser<'s>,
    completed: bool,
}

impl<'i> AtRuleParser<'i> for SingleRuleParser<'i> {
    type Prelude = StrictAtRulePrelude;
    type AtRule = Vec<CssRule>;
    type Error = Error;

    fn parse_prelude<'t>(
        &mut self,
        name: CowRcStr<'i>,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self::Prelude, ParseError<'i, Error>> {
        AtRuleParser::parse_prelude(&mut self.grammar, name, input)
    }

    fn rule_without_block(
        &mut self,
        prelude: Self::Prelude,
        start: &ParserState,
    ) -> Result<Self::AtRule, ()> {
        let result = self.grammar.rule_without_block(prelude, start);
        self.completed = result.is_ok();
        result
    }

    fn parse_block<'t>(
        &mut self,
        prelude: Self::Prelude,
        start: &ParserState,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self::AtRule, ParseError<'i, Error>> {
        let result = AtRuleParser::parse_block(&mut self.grammar, prelude, start, input);
        self.completed = result.is_ok();
        result
    }
}

impl<'i> QualifiedRuleParser<'i> for SingleRuleParser<'i> {
    type Prelude = Vec<CssSelector>;
    type QualifiedRule = Vec<CssRule>;
    type Error = Error;

    fn parse_prelude<'t>(
        &mut self,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self::Prelude, ParseError<'i, Error>> {
        QualifiedRuleParser::parse_prelude(&mut self.grammar, input)
    }

    fn parse_block<'t>(
        &mut self,
        prelude: Self::Prelude,
        start: &ParserState,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self::QualifiedRule, ParseError<'i, Error>> {
        let result = QualifiedRuleParser::parse_block(&mut self.grammar, prelude, start, input);
        self.completed = result.is_ok();
        result
    }
}

/// Parses exactly one complete ordinary rule using supplied namespace bindings.
///
/// The original source must contain one supported style rule or at-rule, with
/// optional surrounding whitespace/comments. Multiple rules, trailing nontrivia
/// and rejected outer grammar return `None` with `RejectInput`. Encoding directives
/// are stylesheet metadata and cannot produce a rule; no BOM or CDO/CDC is stripped.
/// Imports and namespaces use isolated top-level grammar, without insertion-order
/// validation against an existing sheet. The supplied namespace context is immutable.
///
/// A retained rule preserves its inner declaration, selector, query and child-rule
/// recovery diagnostics. Implicit EOF closures are reported only when the outer
/// rule survives; resource diagnostics preserve `StopAtNestingLimit`. Positions
/// refer to the original UTF-8 source and zero-based UTF-16 coordinates. Nested
/// contents remain inside their owning rule. This performs no matching, cascade,
/// CSSOM insertion or contextual resolution.
pub fn parse_rule(
    source: &str,
    context: &CssNamespaceContext,
) -> crate::CssParseReport<Option<CssRule>> {
    bounded(source, || {
        let state = RecoveryState::at_depth(source, 0, StyleContextCaptures::default());
        if let Some(name) = &context.0.default {
            state.activate_namespace(None, name.clone());
        }
        for (prefix, name) in &context.0.named {
            state.activate_namespace(Some(prefix.clone()), name.clone());
        }
        let mut parser_input = ParserInput::new(source);
        let mut input = Parser::new(&mut parser_input);
        let mut parser = SingleRuleParser {
            grammar: StrictRuleParser::isolated_rule(source, state.clone()),
            completed: false,
        };
        input.skip_whitespace();
        let rule_start = input.position().byte_index();
        match cssparser::parse_one_rule(&mut input, &mut parser) {
            Ok(rules) => {
                // The ordinary grammar emits one outer node. Encoding, which
                // emits no node, was excluded by the isolated constructor.
                let [rule]: [CssRule; 1] = rules
                    .try_into()
                    .expect("a successful isolated ordinary rule emits one outer node");
                let mut diagnostics = parser.grammar.diagnostics;
                diagnostics.extend(state.take_implicit_closure_diagnostics(source));
                crate::CssParseReport::new(Some(rule), diagnostics)
            }
            Err(error) => {
                let action =
                    recovery_action_for_error(&error, crate::CssRecoveryAction::RejectInput);
                let error = if parser.completed {
                    error
                } else {
                    let location = error.location;
                    let failed_unit = &source[rule_start..input.position().byte_index()];
                    location.new_custom_error(from_rule_parse_error(source, failed_unit, error))
                };
                crate::CssParseReport::new(None, vec![reject(source, error, action)])
            }
        }
    })
}

/// Parses exactly one real-brace style block with supplied namespace bindings.
///
/// The source contains the braces and optional surrounding whitespace/comments,
/// without a selector. Empty and recovered-empty blocks are retained. A second
/// block, missing opening brace or trailing nontrivia rejects the complete input
/// with `RejectInput`; resource failures preserve `StopAtNestingLimit`.
///
/// Inner declarations, nested rules and subsequent declaration runs use ordinary
/// style-body grammar and preserve their recovery diagnostics. Relative child
/// selectors and explicit anchors remain symbolic. No parent selector is invented.
/// The block origin includes its actual braces or ends at implicit EOF, excludes
/// surrounding trivia, and shares the original source snapshot with declarations.
/// Implicit closure diagnostics are published only after the outer block survives.
pub fn parse_style_block(
    source: &str,
    context: &CssNamespaceContext,
) -> crate::CssParseReport<Option<CssStyleBlock>> {
    bounded(source, || {
        let state = RecoveryState::at_depth(source, 0, StyleContextCaptures::default());
        if let Some(name) = &context.0.default {
            state.activate_namespace(None, name.clone());
        }
        for (prefix, name) in &context.0.named {
            state.activate_namespace(Some(prefix.clone()), name.clone());
        }
        let mut parser_input = ParserInput::new(source);
        let mut input = Parser::new(&mut parser_input);
        let result = (|| {
            input.skip_whitespace();
            let start = input.position().byte_index();
            input.expect_curly_bracket_block()?;
            let recovered = input.parse_nested_block(|input| {
                let mut depth =
                    state.enter_rule_block(source, input, "official.value.style-block")?;
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
                vec![reject(source, error, crate::CssRecoveryAction::RejectInput)],
            ),
        }
    })
}
