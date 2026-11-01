//! Raw current block-contents declaration projection over the original source.
//!
//! The units are the parser crate's block-contents units of the source's token
//! tree (Syntax 3 §5.5.5): every at-rule is dropped, and a declaration the
//! receiver does not admit is dropped for what it consumed once read as a rule.
use crate::component::CssComponentValueLimits;
use crate::declaration::{CssDeclaration, CssDeclarationList};
use crate::diagnostic::CssRecoveryAction;
use crate::diagnostic::error::{
    invalid_syntax, property_name_error, unexpected_syntax_end, unexpected_syntax_token,
};
use crate::parser::block_units::{
    DeclarationUnit, Failure, GrammarError, NativeError, grammar_error, location_at, native_error,
    offset_of, refused_declaration,
};
use crate::parser::declaration_unit::{DeclarationText, parse_declaration_unit};
use crate::parser::recovery::{RecoveryState, StyleContextCaptures};
use crate::parser::syntax_types::{Attached, SyntaxError};
use crate::parser::{
    DeclarationMode, block_item_diagnostic_from_start, fragments, recovery, syntax_bridge,
};
use crate::property::properties::resolve_property_name;
use crate::source::CssSourceSnapshot;
use crate::syntax_consumption::DocumentExt;
use crate::{CssParseReport, CssParserContext};
use cssparser::SourceLocation;
use surgeist_css_parser::Token;
use surgeist_css_parser::grammar::{
    AtRule, BlockGrammar, QualifiedRule, RuleEnd, RuleGrammar, UnitKind,
};

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
    fn check_name<'i>(&self, name: &str, location: SourceLocation) -> Result<(), NativeError<'i>>;
    /// Admits the declaration named `name` that lies at `text`.
    fn admit<'i>(
        &mut self,
        name: &str,
        text: &DeclarationText,
        recovery: &RecoveryState,
    ) -> Result<Self::Declaration, NativeError<'i>>;
    fn retained(&self, _declaration: &Self::Declaration, _recovery: &RecoveryState) {}
    fn take_diagnostics(&mut self) -> Vec<crate::diagnostic::CssRecoveryDiagnostic> {
        Vec::new()
    }
}

struct OrdinaryReceiver {
    parser_context: CssParserContext,
}
impl Receiver for OrdinaryReceiver {
    type Declaration = CssDeclaration;
    const ABORT_ON_RESOURCE: bool = false;
    fn check_name<'i>(&self, name: &str, location: SourceLocation) -> Result<(), NativeError<'i>> {
        if !self.parser_context.selects_svg_glyph(name) && resolve_property_name(name).is_none() {
            Err(property_name_error(location, name))
        } else {
            Ok(())
        }
    }
    fn admit<'i>(
        &mut self,
        name: &str,
        text: &DeclarationText,
        recovery: &RecoveryState,
    ) -> Result<CssDeclaration, NativeError<'i>> {
        parse_declaration_unit(
            DeclarationMode::Ordinary,
            name,
            text,
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

/// The grammar of a block's contents for a receiver: every at-rule and every
/// rule is dropped, and a declaration is the receiver's to admit.
struct Contents<'r, R> {
    receiver: &'r mut R,
    recovery: &'r RecoveryState,
    /// Where the declaration last admitted began.
    start: Option<usize>,
}

impl<'i, R: Receiver> RuleGrammar<'i, Attached<'i>> for Contents<'_, R> {
    type Item = R::Declaration;
    type Error = NativeError<'i>;

    fn at_rule(&mut self, rule: AtRule<'i, Attached<'i>>) -> Result<Self::Item, GrammarError<'i>> {
        let at = rule.start();
        let location = location_at(self.recovery.source_snapshot(), offset_of(at));
        Err(grammar_error(
            at,
            unexpected_syntax_token(location, Token::AtKeyword(rule.name())),
        ))
    }

    fn qualified_rule(
        &mut self,
        rule: QualifiedRule<'i, Attached<'i>>,
    ) -> Result<Self::Item, GrammarError<'i>> {
        Err(SyntaxError::invalid_qualified_rule_error(rule.start()))
    }
}

impl<'i, R: Receiver> BlockGrammar<'i, Attached<'i>> for Contents<'_, R> {
    fn declaration(
        &mut self,
        declaration: DeclarationUnit<'i>,
    ) -> Result<Self::Item, GrammarError<'i>> {
        let at = declaration.start();
        self.start = Some(offset_of(at));
        let name = declaration.name();
        let text = DeclarationText::of(&declaration);
        let snapshot = self.recovery.source_snapshot();
        if !name.starts_with("--") {
            self.receiver
                .check_name(name, location_at(snapshot, offset_of(at)))
                .map_err(|error| grammar_error(at, error))?;
        }
        self.receiver
            .admit(name, &text, self.recovery)
            .map_err(|error| grammar_error(at, error))
    }
}

/// The error a declaration that failed is reported with: the declaration
/// attempt's own, with a `;` where the declaration needed more reported as the
/// end of the declaration.
fn declaration_failure<'i>(snapshot: &CssSourceSnapshot, failed: &Failure<'i>) -> NativeError<'i> {
    let error = failed.declaration_error().unwrap_or_else(|| failed.error());
    // A `;` or the `}` that ends the contents is the end of the declaration.
    if matches!(
        error.unexpected_token(),
        Some(Token::Semicolon | Token::CloseCurly)
    ) {
        let mut at = offset_of(error.at());
        if matches!(error.unexpected_token(), Some(Token::Semicolon)) {
            // The colon is missing where the name ends.
            let start = failed.start();
            let document = start.document();
            let nodes = document.list(start.list());
            if let Some(name) = nodes
                .get(start.index())
                .filter(|node| matches!(document.token(**node), Token::Ident(_)))
                && let Some(range) = document.range(*name)
            {
                at = range.end.value();
            }
        }
        unexpected_syntax_end(location_at(snapshot, at))
    } else {
        native_error(snapshot, error)
    }
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
            || crate::syntax_consumption::source_document_with_limits(source, limits),
            |snapshot| {
                crate::syntax_consumption::source_document_snapshot_with_limits(snapshot, limits)
            },
        ) {
            Ok(document) => document,
            Err(error) => {
                return CssParseReport::new(None, vec![syntax_bridge::arena_error(source, error)]);
            }
        };
        let snapshot = document
            .snapshot()
            .expect("original source document")
            .clone();
        let recovery = RecoveryState::at_depth_with_snapshot(
            source,
            0,
            StyleContextCaptures::default(),
            snapshot.clone(),
        )
        .with_parser_context(parser_context);
        let mut receiver = make_receiver(recovery.clone());
        let mut stream = document.root();
        let mut declarations = Vec::new();
        let mut diagnostics = Vec::new();
        let mut resource_failure = false;
        let consumed_end = loop {
            let before = stream.clone();
            let mut grammar = Contents {
                receiver: &mut receiver,
                recovery: &recovery,
                start: None,
            };
            let mut units = stream.block_contents(&mut grammar);
            let unit = units.next();
            stream = units.stream();
            let after = offset_of(stream.site());
            let failed = match unit {
                None => break after,
                Some(Ok(declaration)) => {
                    // An admitted declaration began where the grammar saw it.
                    let start = grammar.start.unwrap_or(after);
                    recovery
                        .retain_component_closures(recovery.component_openings_in(start..after));
                    grammar.receiver.retained(&declaration, &recovery);
                    declarations.push(declaration);
                    continue;
                }
                Some(Err(failed)) => failed,
            };
            let start = failed.start();
            let document = start.document();
            let list = start.list();
            let nodes = document.list(list);
            let start_offset = offset_of(start);
            // What the unit consumed once read as it was dropped, and how it
            // is reported.
            let (error, action, mut end_index) = if failed.kind() == UnitKind::AtRule {
                (
                    match start.token() {
                        Some(token) => {
                            unexpected_syntax_token(location_at(&snapshot, start_offset), token)
                        }
                        None => unexpected_syntax_end(location_at(&snapshot, start_offset)),
                    },
                    CssRecoveryAction::DropAtRule,
                    failed.end().index(),
                )
            } else {
                let mut error = declaration_failure(&snapshot, &failed);
                if let Some((begin, declaration_end)) = refused_declaration(&failed) {
                    // The generic declaration consumption refuses a value that
                    // holds a `{}`-block beside others, after the header
                    // admitted the name.
                    let name = match begin.token() {
                        Some(Token::Ident(name)) => name,
                        _ => "",
                    };
                    let text = DeclarationText::read(begin, declaration_end);
                    let named = if name.starts_with("--") {
                        Ok(())
                    } else {
                        grammar
                            .receiver
                            .check_name(name, location_at(&snapshot, start_offset))
                    };
                    error = match named {
                        Ok(()) => invalid_syntax(location_at(&snapshot, text.value_start)),
                        Err(named) => named,
                    };
                }
                if failed.kind() == UnitKind::QualifiedRule
                    && crate::diagnostic::error::is_resource_parse_error(&error)
                {
                    // A declaration that failed for its resources is dropped
                    // as the declaration it is, to its `;`.
                    let stop = (start.index()..nodes.len())
                        .find(|&index| {
                            matches!(
                                document.token(nodes[index]),
                                Token::Semicolon | Token::CloseCurly
                            )
                        })
                        .unwrap_or(nodes.len());
                    (error, CssRecoveryAction::DropDeclaration, stop)
                } else if failed.kind() == UnitKind::QualifiedRule {
                    (
                        error,
                        CssRecoveryAction::DropQualifiedRule,
                        failed.end().index(),
                    )
                } else {
                    (
                        error,
                        CssRecoveryAction::DropDeclaration,
                        failed.end().index(),
                    )
                }
            };
            let resource = crate::diagnostic::error::is_resource_parse_error(&error);
            // A rule that returned nothing consumed the `;` that ended it.
            if failed.kind() == UnitKind::Declaration
                && failed.rule_end() == Some(RuleEnd::Semicolon)
                && end_index > 0
                && nodes
                    .get(end_index - 1)
                    .is_some_and(|node| matches!(document.token(*node), Token::Semicolon))
            {
                end_index -= 1;
            }
            let end_byte = document
                .site(list, end_index)
                .map_or_else(|| offset_of(failed.end()), offset_of);
            // A dropped declaration is reported to end after the `;` that
            // terminates it, which stays unconsumed for the next unit.
            let terminated = nodes
                .get(end_index)
                .is_some_and(|node| matches!(document.token(*node), Token::Semicolon));
            let span_end = if action == CssRecoveryAction::DropDeclaration && terminated {
                end_byte + 1
            } else {
                end_byte
            };
            if end_index != failed.end().index() {
                // The drop consumed other than the rule attempt did.
                stream = before.clone();
                while stream.site().index() < end_index
                    && stream.consume_token_with_comments().is_ok()
                {}
            }
            if let Some(diagnostic) =
                block_item_diagnostic_from_start(source, error, start_offset, span_end, action)
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
