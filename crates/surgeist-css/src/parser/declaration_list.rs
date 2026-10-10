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
            let document = match recovery.syntax_document(source) {
                Ok(document) => document,
                Err(error) => {
                    return crate::CssParseReport::new(
                        CssDeclarationList::new(declarations),
                        vec![syntax_bridge::arena_error(source, error)],
                    );
                }
            };
            let selected = crate::syntax_consumption::consume_declaration_list(
                &mut document.cursor(document.root),
            );
            for candidate in selected {
                use crate::syntax_consumption::{GenericDeclarationListItem, RuleTermination};
                let (range, at_rule, generic_valid) = match candidate {
                    GenericDeclarationListItem::At(rule) => {
                        let mut range = rule.range;
                        if matches!(rule.termination, RuleTermination::Semicolon(_)) {
                            range.end -= 1;
                        }
                        (range, true, false)
                    }
                    GenericDeclarationListItem::Declaration { range, parsed } => {
                        (range, false, parsed.is_ok())
                    }
                };
                let start_byte = document
                    .boundary(range.list, range.start)
                    .source
                    .expect("source candidate")
                    .offset;
                let end_byte = document
                    .boundary(range.list, range.end)
                    .source
                    .expect("source candidate end")
                    .offset;
                advance_to(&mut input, start_byte);
                let start = input.state();
                let location = input.current_source_location();
                let token = input
                    .next_including_whitespace_and_comments()
                    .expect("selected candidate")
                    .clone();
                input.reset(&start);
                let components = recovery.check_declaration_list_unit(source, &input, at_rule);
                let result = if at_rule {
                    advance_to(&mut input, end_byte);
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
                        if !generic_valid {
                            return Err(invalid_syntax(unit.current_source_location()));
                        }
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
                advance_to(&mut input, end_byte);
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
                            start_byte,
                            end_byte,
                            crate::CssRecoveryAction::DropDeclaration,
                        ) {
                            diagnostics.push(diagnostic);
                        }
                    }
                }
            }
            diagnostics.extend(recovery.take_implicit_closure_diagnostics(source));
            crate::CssParseReport::new(CssDeclarationList::new(declarations), diagnostics)
        }),
    )
}

fn advance_to(input: &mut Parser<'_, '_>, end: usize) {
    while input.position().byte_index() < end {
        let Ok(token) = input.next_including_whitespace_and_comments().cloned() else {
            break;
        };
        let _ = fragments::finish_nested_component(input, &token);
    }
}
