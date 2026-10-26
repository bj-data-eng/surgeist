//! Raw current block-contents declaration projection over the original source.
use super::*;
use crate::syntax_consumption::{self as syntax, CursorItem, TokenKind};
use crate::{CssComponentValueLimits, CssParseReport, CssParserContext, CssRecoveryAction};

pub(crate) fn parse_with_context(
    source: &str,
    limits: CssComponentValueLimits,
    parser_context: CssParserContext,
) -> CssParseReport<CssDeclarationList> {
    let (declarations, diagnostics) = parse_contents(source, limits, parser_context, |_| {
        OrdinaryReceiver { parser_context }
    })
    .into_parts();
    CssParseReport::new(
        CssDeclarationList::new(declarations.unwrap_or_default()),
        diagnostics,
    )
}

pub(crate) fn parse_snapshot(
    snapshot: &CssSourceSnapshot,
    limits: CssComponentValueLimits,
    parser_context: CssParserContext,
) -> CssParseReport<CssDeclarationList> {
    let (declarations, mut diagnostics) = parse_contents_with_snapshot(
        snapshot.as_str(),
        limits,
        parser_context,
        Some(snapshot),
        |_| OrdinaryReceiver { parser_context },
    )
    .into_parts();
    for diagnostic in &mut diagnostics {
        diagnostic.bind_source(snapshot);
    }
    CssParseReport::new(
        CssDeclarationList::new(declarations.unwrap_or_default()),
        diagnostics,
    )
}

/// Domain callbacks supply only name/value admission. Syntax unit selection,
/// source observations, synchronization and diagnostic spans have one owner.
pub(super) trait Receiver {
    type Declaration: Send;
    /// Ordinary raw admission retains its established resource recovery policy.
    /// Domain replacement must reject preparation rather than publish a prefix.
    const ABORT_ON_RESOURCE: bool = true;
    fn check_name<'i>(
        &self,
        name: &str,
        location: cssparser::SourceLocation,
    ) -> Result<(), ParseError<'i, CssError>>;
    fn parse_value<'i>(
        &mut self,
        name: CowRcStr<'i>,
        input: &mut Parser<'i, '_>,
        start: &ParserState,
        recovery: &RecoveryState,
    ) -> Result<Self::Declaration, ParseError<'i, CssError>>
    where
        Self: 'i;
    fn retained(&self, _declaration: &Self::Declaration, _recovery: &RecoveryState) {}
    fn take_diagnostics(&mut self) -> Vec<crate::CssRecoveryDiagnostic> {
        Vec::new()
    }
}

struct OrdinaryReceiver {
    parser_context: CssParserContext,
}
impl Receiver for OrdinaryReceiver {
    type Declaration = CssDeclaration;
    const ABORT_ON_RESOURCE: bool = false;
    fn check_name<'i>(
        &self,
        name: &str,
        location: cssparser::SourceLocation,
    ) -> Result<(), ParseError<'i, CssError>> {
        if !self.parser_context.selects_svg_glyph(name) && resolve_property_name(name).is_none() {
            Err(property_name_error(location, name))
        } else {
            Ok(())
        }
    }
    fn parse_value<'i>(
        &mut self,
        name: CowRcStr<'i>,
        input: &mut Parser<'i, '_>,
        start: &ParserState,
        recovery: &RecoveryState,
    ) -> Result<CssDeclaration, ParseError<'i, CssError>>
    where
        Self: 'i,
    {
        parse_declaration_core(
            DeclarationMode::Ordinary,
            name,
            input,
            start,
            recovery.source_snapshot(),
            self.parser_context,
        )
        .map(|parsed| parsed.into_declaration())
    }
    fn retained(&self, declaration: &CssDeclaration, recovery: &RecoveryState) {
        recovery.retain_navigation_diagnostic(declaration.body());
    }
}

pub(super) fn parse_contents<R: Receiver>(
    source: &str,
    limits: CssComponentValueLimits,
    parser_context: CssParserContext,
    make_receiver: impl FnOnce(RecoveryState) -> R + Send,
) -> CssParseReport<Option<Vec<R::Declaration>>> {
    parse_contents_with_snapshot(source, limits, parser_context, None, make_receiver)
}

fn parse_contents_with_snapshot<R: Receiver>(
    source: &str,
    limits: CssComponentValueLimits,
    parser_context: CssParserContext,
    snapshot: Option<&CssSourceSnapshot>,
    make_receiver: impl FnOnce(RecoveryState) -> R + Send,
) -> CssParseReport<Option<Vec<R::Declaration>>> {
    let refused = || None;
    fragments::bounded_report(source, refused, || {
        let document = match snapshot.map_or_else(
            || syntax::source_document_with_limits(source, limits),
            |snapshot| syntax::source_document_snapshot_with_limits(snapshot, limits),
        ) {
            Ok(document) => document,
            Err(error) => {
                return CssParseReport::new(None, vec![syntax_bridge::arena_error(source, error)]);
            }
        };
        let snapshot = document
            .source
            .as_ref()
            .expect("original source document")
            .original
            .clone();
        let recovery = RecoveryState::at_depth_with_snapshot(
            source,
            0,
            StyleContextCaptures::default(),
            snapshot,
        )
        .with_parser_context(parser_context);
        let mut receiver = make_receiver(recovery.clone());
        let working_source = crate::tokenization::prepare(source);
        let stop = document.lists[document.root]
            .iter()
            .find_map(|node| {
                (document.nodes[*node].token().kind()
                    == TokenKind::Closing(crate::CssBlockKind::CurlyBracket))
                .then(|| document.range(*node).expect("source closer").start)
            })
            .unwrap_or(source.len());
        let mut parser_input = ParserInput::new(&working_source[..stop]);
        let mut input = Parser::new(&mut parser_input);
        let mut cursor = document.cursor(document.root);
        let mut declarations = Vec::new();
        let mut diagnostics = Vec::new();
        let mut resource_failure = false;
        let consumed_end = loop {
            cursor.skip_trivia();
            if syntax::block_contents_end(&cursor) {
                break offset(&cursor);
            }
            let CursorItem::Node(node) = cursor.peek() else {
                unreachable!("nonterminal")
            };
            if document.nodes[node].token().kind() == TokenKind::Semicolon {
                cursor.consume();
                continue;
            }
            let start_byte = offset(&cursor);
            advance_to(&mut input, start_byte);
            let start = input.state();
            let location = input.current_source_location();
            let token = input
                .next_including_whitespace_and_comments()
                .expect("selected source unit")
                .clone();
            input.reset(&start);
            if matches!(document.nodes[node].token().kind(), TokenKind::AtKeyword(_)) {
                let _ = syntax::consume_at_rule_context(&mut cursor, true);
                let end_byte = offset(&cursor);
                advance_to(&mut input, end_byte);
                if let Some(diagnostic) = block_item_diagnostic_from_start(
                    source,
                    location.new_unexpected_token_error(token),
                    start_byte,
                    end_byte,
                    CssRecoveryAction::DropAtRule,
                ) {
                    diagnostics.push(diagnostic);
                }
                continue;
            }
            // The selected Syntax algorithm permits declaration early exits
            // for an unrecognized name or missing colon. Probe only the owned
            // header grammar here, so a run of rule bodies without semicolons
            // is not repeatedly scanned as one declaration value.
            let header = input
                .try_parse(|unit| {
                    let name = unit.expect_ident_cloned()?;
                    unit.expect_colon()?;
                    if !name.starts_with("--") {
                        receiver.check_name(name.as_ref(), location)?;
                    }
                    Ok(())
                })
                .map_err(semicolon_as_end);
            input.reset(&start);
            if let Err(error) = header {
                let action = reconsume_as_rule(&mut cursor);
                let end_byte = offset(&cursor);
                advance_to(&mut input, end_byte);
                let span_end = dropped_end(&cursor, end_byte, action);
                if let Some(diagnostic) =
                    block_item_diagnostic_from_start(source, error, start_byte, span_end, action)
                {
                    diagnostics.push(diagnostic);
                }
                continue;
            }
            let mut declaration_cursor = cursor.clone();
            let range = syntax::consume_block_declaration_candidate(&mut declaration_cursor);
            let generic = syntax::consume_block_declaration(&mut document.cursor_range(&range));
            let declaration = input.parse_until_before(Delimiter::Semicolon, |unit| {
                let name = unit.expect_ident_cloned()?;
                unit.expect_colon()?;
                if generic.is_err() {
                    return Err(invalid_syntax(unit.current_source_location()));
                }
                let declaration = receiver.parse_value(name, unit, &start, &recovery)?;
                unit.expect_exhausted()?;
                Ok(declaration)
            });
            input.reset(&start);
            let error = match declaration {
                Ok(declaration) => {
                    cursor = declaration_cursor;
                    let end_byte = offset(&cursor);
                    advance_to(&mut input, end_byte);
                    recovery.retain_component_closures(
                        recovery.component_openings_in(start_byte..end_byte),
                    );
                    receiver.retained(&declaration, &recovery);
                    declarations.push(declaration);
                    continue;
                }
                Err(error) => error,
            };
            let custom = generic.is_ok()
                && matches!(document.nodes[node].token().kind(), TokenKind::Ident(name) if name.starts_with("--"));
            let action = if custom || crate::error::is_resource_parse_error(&error) {
                cursor = declaration_cursor;
                CssRecoveryAction::DropDeclaration
            } else {
                reconsume_as_rule(&mut cursor)
            };
            let end_byte = offset(&cursor);
            advance_to(&mut input, end_byte);
            let span_end = dropped_end(&cursor, end_byte, action);
            let resource = crate::error::is_resource_parse_error(&error);
            if let Some(diagnostic) =
                block_item_diagnostic_from_start(source, error, start_byte, span_end, action)
            {
                diagnostics.push(diagnostic);
            }
            if resource && R::ABORT_ON_RESOURCE {
                resource_failure = true;
                break end_byte;
            }
        };
        diagnostics.extend(receiver.take_diagnostics());
        if R::ABORT_ON_RESOURCE {
            resource_failure |= recovery::has_resource_failure(&diagnostics);
        }
        diagnostics.extend(recovery.take_implicit_closure_diagnostics(source));
        // Tokenizer recovery belongs to consumed contents, not text after `}`.
        // The source snapshot still retains the complete original input.
        recovery::finish_report(
            &source[..consumed_end],
            CssParseReport::new((!resource_failure).then_some(declarations), diagnostics),
        )
    })
}

/// A declaration unit ends at its `;`, so a `;` found where the header needed more
/// input is the end of that unit, as it is for every declaration list.
fn semicolon_as_end(error: ParseError<'_, CssError>) -> ParseError<'_, CssError> {
    match error.kind {
        cssparser::ParseErrorKind::Basic(cssparser::BasicParseErrorKind::UnexpectedToken(
            cssparser::Token::Semicolon,
        )) => ParseError {
            kind: cssparser::ParseErrorKind::Basic(cssparser::BasicParseErrorKind::EndOfInput),
            location: error.location,
        },
        _ => error,
    }
}

/// Syntax 3 (2026 CRD) §5.5.5: a unit that is not a declaration is reconsumed as
/// a qualified rule with `;` as its stop token. The drop is reported for what it
/// consumed: a rule that reached its `{}` block is a dropped qualified rule, and
/// a unit that ended at the stop token, a root `}` or EOF without a block is a
/// dropped declaration, as in every other declaration list.
fn reconsume_as_rule(cursor: &mut syntax::SyntaxCursor<'_>) -> CssRecoveryAction {
    if syntax::consume_qualified_rule_context(cursor, true).is_ok() {
        CssRecoveryAction::DropQualifiedRule
    } else {
        CssRecoveryAction::DropDeclaration
    }
}

/// A dropped declaration's span ends after the `;` that terminates it, as in every
/// other declaration list. The `;` itself stays unconsumed: the next iteration
/// discards it as an empty slot.
fn dropped_end(
    cursor: &syntax::SyntaxCursor<'_>,
    end_byte: usize,
    action: CssRecoveryAction,
) -> usize {
    let terminated = matches!(
        cursor.peek(),
        CursorItem::Node(node)
            if cursor.document.nodes[node].token().kind() == TokenKind::Semicolon
    );
    if action == CssRecoveryAction::DropDeclaration && terminated {
        end_byte + 1
    } else {
        end_byte
    }
}

fn offset(cursor: &syntax::SyntaxCursor<'_>) -> usize {
    cursor
        .boundary()
        .source
        .expect("original source boundary")
        .offset
}
fn advance_to(input: &mut Parser<'_, '_>, end: usize) {
    while input.position().byte_index() < end {
        let Ok(token) = input.next_including_whitespace_and_comments().cloned() else {
            break;
        };
        let _ = fragments::finish_nested_component(input, &token);
    }
}
