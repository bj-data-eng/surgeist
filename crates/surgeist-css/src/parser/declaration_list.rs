//! Raw ordinary-property declaration lists, with CSS Syntax 3 §5.4.5 synchronization.
use super::*;

/// Parses raw declaration-list text with recovery and ordinary property grammar.
///
/// Empty input, whitespace, comments and semicolon-only slots are clean. Retained
/// declarations preserve source order, duplicates, custom-name case, importance,
/// parser context and original name/value origins. No CSSOM winner selection or
/// shorthand coalescing occurs here.
///
/// An ident-started candidate consumes complete components through the next root
/// semicolon or EOF, then uses the same declaration grammar as style attributes.
/// At-rules consume their complete generic block or semicolon unit and are
/// rejected once by this ordinary-property consumer. Every other initial token,
/// including a stray closing delimiter, starts one invalid unit consumed through
/// the next root semicolon or EOF. Nested semicolons do not synchronize recovery.
/// Unlike [`super::parse_style_attribute`], a stray closer therefore discards the
/// declarations following it in that same unit. Qualified rules and descriptors
/// are not admitted. Diagnostic spans exclude the recovery semicolon.
///
/// ```
/// use surgeist_css::{parse_declaration_list_text, CssKnownProperty};
/// let report = parse_declaration_list_text("} color: red; width: 2px;");
/// assert_eq!(report.syntax().len(), 1);
/// assert_eq!(report.syntax()[0].known().unwrap().property(), CssKnownProperty::Width);
/// assert_eq!(report.diagnostics().len(), 1);
/// ```
#[must_use]
pub fn parse_declaration_list_text(source: &str) -> crate::CssParseReport<CssDeclarationList> {
    parse_declaration_list_text_with_context(source, crate::CssParserContext::default())
}

pub(crate) fn parse_declaration_list_text_with_context(
    source: &str,
    parser_context: crate::CssParserContext,
) -> crate::CssParseReport<CssDeclarationList> {
    recovery::finish_report(
        source,
        fragments::bounded_execution(source, || {
            let recovery = RecoveryState::at_depth(source, 0, StyleContextCaptures::default())
                .with_parser_context(parser_context);
            let working_source = crate::tokenization::prepare(source);
            let mut parser_input = ParserInput::new(&working_source);
            let mut input = Parser::new(&mut parser_input);
            let mut declarations = Vec::new();
            let mut diagnostics = Vec::new();
            loop {
                let progress = RecoveryProgress::record(&input);
                let start = input.state();
                let location = input.current_source_location();
                let Ok(token) = input.next_including_whitespace_and_comments().cloned() else {
                    break;
                };
                if matches!(
                    token,
                    Token::WhiteSpace(_) | Token::Comment(_) | Token::Semicolon
                ) {
                    continue;
                }
                input.reset(&start);
                let at_rule = matches!(token, Token::AtKeyword(_));
                let components = recovery.check_declaration_list_unit(source, &input, at_rule);
                let result = if at_rule {
                    // Consume Syntax's generic at-rule independently of known at-rule
                    // grammar. Its body cannot contribute ordinary declarations.
                    input
                        .next_including_whitespace_and_comments()
                        .expect("peeked at-keyword");
                    consume_at_rule(&mut input);
                    components
                        .and_then(|_| Err(location.new_unexpected_token_error::<Error>(token)))
                } else {
                    input.parse_until_before(Delimiter::Semicolon, |unit| {
                        let openings = components?;
                        if !matches!(token, Token::Ident(_)) {
                            return Err(location.new_unexpected_token_error::<Error>(token));
                        }
                        let name = unit.expect_ident_cloned()?;
                        unit.expect_colon()?;
                        let declaration = parse_declaration_core(
                            DeclarationMode::Ordinary,
                            name,
                            unit,
                            &start,
                            recovery.source_snapshot(),
                            parser_context,
                        )?;
                        unit.expect_exhausted()?;
                        Ok((declaration, openings))
                    })
                };
                let retained = result.is_ok();
                let end = input.position().byte_index();
                match result {
                    Ok((declaration, openings)) => {
                        recovery.retain_component_closures(openings);
                        recovery.retain_navigation_diagnostic(&declaration.body);
                        declarations.push(declaration.into_declaration());
                    }
                    Err(error) => {
                        if let Some(diagnostic) = block_item_diagnostic_from_start(
                            source,
                            error,
                            start.position().byte_index(),
                            end,
                            crate::CssRecoveryAction::DropDeclaration,
                        ) {
                            diagnostics.push(diagnostic);
                        }
                    }
                }
                if progress.finish(&mut input, retained) == RecoveryLoopOutcome::Terminated {
                    break;
                }
            }
            diagnostics.extend(recovery.take_implicit_closure_diagnostics(source));
            crate::CssParseReport::new(CssDeclarationList::new(declarations), diagnostics)
        }),
    )
}

fn consume_at_rule(input: &mut Parser<'_, '_>) {
    let _: Result<(), ParseError<'_, Error>> = input.parse_until_before(
        Delimiter::Semicolon | Delimiter::CurlyBracketBlock,
        |prelude| {
            while prelude.next_including_whitespace_and_comments().is_ok() {}
            Ok(())
        },
    );
    let end = input.state();
    if let Ok(token) = input.next_including_whitespace_and_comments().cloned() {
        if matches!(token, Token::CurlyBracketBlock) {
            let _ = fragments::finish_nested_component(input, &token);
        } else {
            // Leave a terminating semicolon to the empty-slot branch; the span
            // describes the discarded at-rule up to its recovery delimiter.
            input.reset(&end);
        }
    }
}
