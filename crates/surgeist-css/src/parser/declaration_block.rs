//! Raw current block-contents declaration projection over the original source.
use super::*;
use crate::syntax_consumption::{self as syntax, CursorItem, TokenKind};
use crate::{CssComponentValueLimits, CssParseReport, CssParserContext, CssRecoveryAction};

/// Parses the declaration projection of raw CSS Syntax block contents.
///
/// No surrounding braces are required or invented. A root `}` stops consumption;
/// declarations after it are never retained. Root semicolons, trivia and EOF are
/// empty slots. Generic at-rules and qualified rules are consumed as complete
/// nested units and dropped from this declaration-only projection. Declaration
/// admission and fallback use the existing ordinary property grammar, including
/// custom-property recovery through the root semicolon. Nested rule boundaries
/// do not swallow following declarations. Ordered occurrences, duplicates,
/// importance and original name/value source identities remain authored data;
/// CSSOM winner selection is a separate operation.
///
/// Recovery/resource diagnostics are retained independently of an empty or
/// nonempty projection. The historical [`parse_declaration_list_text`] remains
/// distinct: its invalid units synchronize through a semicolon, including closers.
#[must_use]
pub fn parse_declaration_block_contents(source: &str) -> CssParseReport<CssDeclarationList> {
    parse_declaration_block_contents_with_limits(source, CssComponentValueLimits::default())
}

/// Parses with cumulative whole-input byte, component and depth limits.
///
/// Every original component is charged once, including descendants, trivia and
/// source after the root stop token. A failed allowance returns an empty projection
/// and its typed resource diagnostic; no partially admitted declarations escape.
/// Grammar probes/fallback share that admitted arena and do not reset the budget.
#[must_use]
pub fn parse_declaration_block_contents_with_limits(
    source: &str,
    limits: CssComponentValueLimits,
) -> CssParseReport<CssDeclarationList> {
    parse_with_context(source, limits, CssParserContext::default())
}

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
    ) -> Result<(), ParseError<'i, Error>>;
    fn parse_value<'i>(
        &mut self,
        name: CowRcStr<'i>,
        input: &mut Parser<'i, '_>,
        start: &ParserState,
        recovery: &RecoveryState,
    ) -> Result<Self::Declaration, ParseError<'i, Error>>
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
    ) -> Result<(), ParseError<'i, Error>> {
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
    ) -> Result<CssDeclaration, ParseError<'i, Error>>
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
    fragments::bounded_execution(source, || {
        let document = match syntax::source_document_with_limits(source, limits) {
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
            let header = input.try_parse(|unit| {
                let name = unit.expect_ident_cloned()?;
                unit.expect_colon()?;
                if !name.starts_with("--") {
                    receiver.check_name(name.as_ref(), location)?;
                }
                Ok(())
            });
            input.reset(&start);
            if let Err(error) = header {
                let _ = syntax::consume_qualified_rule_context(&mut cursor, true);
                let end_byte = offset(&cursor);
                advance_to(&mut input, end_byte);
                if let Some(diagnostic) = block_item_diagnostic_from_start(
                    source,
                    error,
                    start_byte,
                    end_byte,
                    CssRecoveryAction::DropQualifiedRule,
                ) {
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
                let _ = syntax::consume_qualified_rule_context(&mut cursor, true);
                CssRecoveryAction::DropQualifiedRule
            };
            let end_byte = offset(&cursor);
            advance_to(&mut input, end_byte);
            let resource = crate::error::is_resource_parse_error(&error);
            if let Some(diagnostic) =
                block_item_diagnostic_from_start(source, error, start_byte, end_byte, action)
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
