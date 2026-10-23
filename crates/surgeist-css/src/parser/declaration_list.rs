//! Raw ordinary-property declaration lists, with CSS Syntax 3 (2021-12-24) §5.4.5 synchronization,
//! and the bad-declaration remnants shared with the style-attribute front.
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
/// As in [`super::parse_style_attribute`], a stray closer therefore discards the
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
    let refused = || CssDeclarationList::new(Vec::new());
    recovery::finish_report(
        source,
        fragments::bounded_report(source, refused, || {
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
                if !at_rule && !matches!(token, Token::Ident(_)) {
                    diagnostics.extend(drop_bad_declaration(
                        source, &mut input, &recovery, start_byte, end_byte,
                    ));
                    continue;
                }
                let components = recovery.check_declaration_list_unit(source, &input, at_rule);
                let result = if at_rule {
                    advance_to(&mut input, end_byte);
                    components
                        .and_then(|_| Err(location.new_unexpected_token_error::<CssError>(token)))
                } else {
                    input.parse_until_before(Delimiter::Semicolon, |unit| {
                        let openings = components?;
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

/// Skips the empty slots before a declaration-list unit: whitespace,
/// comments and `;`. Returns the unit's start.
pub(super) fn skip_empty_declaration_slots(input: &mut Parser<'_, '_>) -> usize {
    loop {
        let state = input.state();
        match input.next_including_whitespace_and_comments() {
            Ok(Token::WhiteSpace(_) | Token::Comment(_) | Token::Semicolon) => {}
            Ok(_) | Err(_) => {
                input.reset(&state);
                return input.position().byte_index();
            }
        }
    }
}

/// Whether the next unit starts with neither an identifier nor an at-keyword,
/// so it cannot begin a declaration or a rule.
pub(super) fn starts_bad_declaration(input: &mut Parser<'_, '_>) -> bool {
    let state = input.state();
    let bad = input
        .next_including_whitespace_and_comments()
        .is_ok_and(|token| !matches!(token, Token::Ident(_) | Token::AtKeyword(_)));
    input.reset(&state);
    bad
}

/// The end of the remnants of a bad declaration that starts at `start` in
/// the unwrapped root list.
pub(super) fn bad_declaration_end(
    document: &crate::syntax_consumption::SyntaxDocument<'_>,
    start: usize,
) -> usize {
    let mut cursor = document
        .root_cursor_at_source(start)
        .expect("a declaration-list unit starts at a root token");
    crate::syntax_consumption::consume_declaration_candidate(&mut cursor);
    cursor
        .boundary()
        .source
        .expect("original source boundary")
        .offset
}

/// Syntax 3 (2026 CRD) §5.5.6: a declaration whose first token is not an
/// `<ident-token>` consumes the remnants of a bad declaration (not nested:
/// a `}` is discarded, and whole blocks and functions are consumed) through
/// the next root `;` or EOF. The dropped unit excludes that `;`.
/// <https://www.w3.org/TR/2026/CRD-css-syntax-3-20261001/#consume-the-remnants-of-a-bad-declaration>
pub(super) fn drop_bad_declaration<'i>(
    source: &'i str,
    input: &mut Parser<'i, '_>,
    recovery: &RecoveryState,
    start: usize,
    end: usize,
) -> Option<crate::CssRecoveryDiagnostic> {
    advance_to(input, start);
    let state = input.state();
    let location = input.current_source_location();
    let token = input
        .next_including_whitespace_and_comments()
        .expect("a bad declaration starts with its first token")
        .clone();
    input.reset(&state);
    // A structural resource failure inside the remnants owns the diagnostic.
    let error = match recovery.check_declaration_list_unit(source, input, false) {
        Ok(_) => location.new_unexpected_token_error::<CssError>(token),
        Err(resource) => resource,
    };
    advance_to(input, end);
    block_item_diagnostic_from_start(
        source,
        error,
        start,
        end,
        crate::CssRecoveryAction::DropDeclaration,
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
