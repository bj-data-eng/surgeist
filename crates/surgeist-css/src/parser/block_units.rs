//! The units of a block that the parser crate's driver hands to a ported
//! grammar, and the failures it reports for them, in the terms the recovery
//! diagnostics of the grammars still driven by cssparser are written in.
//!
//! A ported grammar reports its own errors as the native errors those
//! diagnostics take. The driver's own failures (a unit that is not a
//! declaration, a rule the protocol rejects) are converted to the same native
//! errors here, with the location cssparser gives them.
use cssparser::{BasicParseErrorKind, CowRcStr, ParseError, ParseErrorKind, SourceLocation};
use surgeist_css_parser::Token;
use surgeist_css_parser::grammar::{BlockGrammar, Declaration, Failed, RuleEnd, UnitKind};

use crate::diagnostic::CssError;
use crate::diagnostic::error::{unexpected_syntax_end, unexpected_syntax_token};
use crate::parser::syntax_types::{Attached, Site, Stream, SyntaxError};
use crate::source::CssSourceSnapshot;
use crate::syntax_consumption::DocumentExt;

/// The error the ported grammars report: what the native callbacks report.
pub(super) type NativeError<'i> = ParseError<'i, CssError>;
/// The failure of a ported grammar, as the driver carries it.
pub(super) type GrammarError<'i> = SyntaxError<'i, NativeError<'i>>;
/// A unit the driver consumed and the grammar did not accept.
pub(super) type Failure<'i> = Failed<'i, NativeError<'i>, Attached<'i>>;
/// A declaration the driver consumed.
pub(super) type DeclarationUnit<'i> = Declaration<'i, Attached<'i>>;

/// The line and column cssparser reports for the byte `offset` of the source.
pub(super) fn location_at(snapshot: &CssSourceSnapshot, offset: usize) -> SourceLocation {
    let position = snapshot
        .position_at(offset)
        .expect("a driver boundary of the source is a character boundary");
    SourceLocation {
        line: position.line().value(),
        column: position.column().value() + 1,
    }
}

/// The byte offset of a boundary of a document of source text.
pub(super) fn offset_of(site: Site<'_>) -> usize {
    site.offset()
        .expect("a boundary of a source document has its offset")
        .value()
}

/// A grammar's own error, at the boundary `at`.
pub(super) fn grammar_error<'i>(at: Site<'i>, error: NativeError<'i>) -> GrammarError<'i> {
    SyntaxError::custom(at, error)
}

/// The native error a failure stands for: the grammar's own, or the one
/// cssparser reports for the same fault.
pub(super) fn native_error<'i>(
    snapshot: &CssSourceSnapshot,
    error: &GrammarError<'i>,
) -> NativeError<'i> {
    native_error_located(error, |site| location_at(snapshot, offset_of(site)))
}

/// [`native_error`] for a document whose boundaries `locate` places.
pub(super) fn native_error_located<'i>(
    error: &GrammarError<'i>,
    locate: impl FnOnce(Site<'i>) -> SourceLocation,
) -> NativeError<'i> {
    if let Some(native) = error.custom_value() {
        return native.clone();
    }
    let location = locate(error.at());
    let basic = |kind| ParseError {
        kind: ParseErrorKind::Basic(kind),
        location,
    };
    if let Some(name) = error.unknown_at_rule() {
        return basic(BasicParseErrorKind::AtRuleInvalid(CowRcStr::from(name)));
    }
    if error.is_invalid_at_rule_body() {
        return basic(BasicParseErrorKind::AtRuleBodyInvalid);
    }
    if error.is_invalid_qualified_rule() {
        return basic(BasicParseErrorKind::QualifiedRuleInvalid);
    }
    match error.unexpected_token() {
        Some(token) => unexpected_syntax_token(location, token),
        None => unexpected_syntax_end(location),
    }
}

/// A unit that was dropped: what the recovery diagnostic of the grammars still
/// driven by cssparser is made of.
pub(super) struct DroppedUnit<'s, 'i> {
    pub(super) kind: UnitKind,
    /// Why the unit is reported as dropped: the declaration's error for a unit
    /// that begins with an identifier, otherwise the rule's.
    pub(super) error: NativeError<'i>,
    /// The source the offsets belong to.
    source: &'s str,
    pub(super) start: usize,
    /// The end of the unit: after the `;` that ended a declaration or a rule
    /// stopped at one, which the driver leaves to the loop that discards it.
    pub(super) end: usize,
    /// Where the prelude of an at-rule ends: at the `;` or `{` that ended it,
    /// or at the end of the unit. The end of the unit for any other unit.
    prelude_end: usize,
    /// Where the contents of the unit's `{}`-block begin, for a unit that
    /// ended with one.
    pub(super) block_contents: Option<usize>,
}

/// Drives the units of `contents` through `grammar`, handing each to `unit`.
///
/// A unit may report that it took over more of the block than the driver
/// consumed for it: it returns the node of the block's list where what it
/// took ends, and the driver resumes there.
pub(super) fn drive_units<'i, G>(
    contents: &mut Stream<'i>,
    grammar: &mut G,
    mut unit: impl FnMut(Result<G::Item, Failure<'i>>) -> Option<usize>,
) where
    G: BlockGrammar<'i, Attached<'i>, Error = NativeError<'i>>,
{
    let mut stream = contents.clone();
    loop {
        let mut units = stream.block_contents(&mut *grammar);
        let mut resume = None;
        for item in units.by_ref() {
            if let Some(end) = unit(item) {
                resume = Some(end);
                break;
            }
        }
        stream = units.stream();
        let Some(end) = resume else { break };
        while stream.site().index() < end && stream.consume_token_with_comments().is_ok() {}
    }
    *contents = stream;
}

/// A declaration that Syntax 3 §5.5.6 step 8 refused (a `{}`-block beside
/// other values in the value of a property that is not custom), which the
/// driver never shows to the grammar. A block's recovery reads the value of
/// such a declaration to its `;` all the same, and reports what the grammar
/// says of it (until the declaration paths follow step 8). Returns the
/// identifier the declaration starts at and the node of its list that ends its
/// value.
pub(super) fn refused_declaration<'i>(failed: &Failure<'i>) -> Option<(Site<'i>, usize)> {
    if failed.kind() == UnitKind::AtRule {
        return None;
    }
    let declaration_error = failed.declaration_error().unwrap_or_else(|| failed.error());
    if !declaration_error.is_stream_error() {
        return None;
    }
    let start = failed.start();
    let document = start.document();
    let nodes = document.list(start.list());
    if !matches!(document.token(*nodes.get(start.index())?), Token::Ident(_)) {
        return None;
    }
    let mut colon = start.index() + 1;
    while nodes
        .get(colon)
        .is_some_and(|node| document.is_trivia(*node))
    {
        colon += 1;
    }
    if !matches!(
        nodes.get(colon).map(|node| document.token(*node)),
        Some(Token::Colon)
    ) {
        return None;
    }
    let end = (colon + 1..nodes.len())
        .find(|&index| matches!(document.token(nodes[index]), Token::Semicolon))
        .unwrap_or(nodes.len());
    Some((start, end))
}

/// Reads a failed unit of the source `source`.
///
/// A block parser that tries an identifier as a declaration first reports the
/// declaration's error, even when the unit turns out to be a rule; any other
/// unit is a rule from its first token, and is reported for what the rule
/// attempt found.
pub(super) fn dropped_unit<'s, 'i>(
    snapshot: &CssSourceSnapshot,
    source: &'s str,
    failed: &Failure<'i>,
) -> DroppedUnit<'s, 'i> {
    let kind = failed.kind();
    let rule_end = failed.rule_end();
    let first = failed.start();
    let after = failed.end();
    let start = offset_of(first);
    let mut end = offset_of(after);
    let mut block_contents = None;
    let document = after.document();
    let nodes = document.list(after.list());
    let at_end = location_at(snapshot, end);
    match rule_end {
        Some(RuleEnd::Block) => {
            // The block is the last node the driver consumed.
            let block = nodes[after.index() - 1];
            block_contents = Some(
                document
                    .range(block)
                    .expect("a source block has its interval")
                    .start
                    .value()
                    + 1,
            );
        }
        Some(RuleEnd::Semicolon) if kind != UnitKind::AtRule => {
            // A declaration, or a rule a semicolon stopped, leaves its
            // semicolon, which the dropped unit includes. The remnants of a
            // custom property look-alike end after theirs, which the driver
            // has consumed.
            let is_semicolon = |index: usize| {
                nodes
                    .get(index)
                    .is_some_and(|node| matches!(document.token(*node), Token::Semicolon))
            };
            if after.index() > 0 && is_semicolon(after.index() - 1) {
                // Consumed already.
            } else if is_semicolon(after.index()) {
                end += 1;
            }
        }
        _ => {}
    }
    let begins_declaration = kind != UnitKind::AtRule
        && matches!(
            nodes.get(first.index()).map(|node| document.token(*node)),
            Some(Token::Ident(_))
        );
    let error = if begins_declaration {
        // Failed::error is the declaration's error when no block was found,
        // and the rule's when one was; the declaration's is the other.
        let declaration_error = failed.declaration_error().unwrap_or_else(|| failed.error());
        if matches!(declaration_error.unexpected_token(), Some(Token::Semicolon)) {
            // The declaration's tokens end at its `;`, which is the end of
            // its input.
            unexpected_syntax_end(location_at(snapshot, offset_of(declaration_error.at())))
        } else {
            native_error(snapshot, declaration_error)
        }
    } else if kind == UnitKind::Declaration {
        // The rule attempt found no block: it met the `;` that stops it or
        // the end of the contents.
        match rule_end {
            Some(RuleEnd::Semicolon) => unexpected_syntax_token(at_end, Token::Semicolon),
            _ => unexpected_syntax_end(at_end),
        }
    } else {
        native_error(snapshot, failed.error())
    };
    let prelude_end = match (kind, rule_end) {
        (UnitKind::AtRule, Some(RuleEnd::Semicolon)) => end - 1,
        (UnitKind::AtRule, Some(RuleEnd::Block)) => block_contents.map_or(end, |at| at - 1),
        _ => end,
    };
    DroppedUnit {
        kind,
        source,
        start,
        end,
        prelude_end,
        block_contents,
        error,
    }
}

impl<'s> DroppedUnit<'s, '_> {
    /// The text of the unit, to the end of the unit.
    pub(super) fn text(&self) -> &'s str {
        &self.source[self.start..self.end]
    }

    /// The text of the unit up to the end of its prelude, which is where an
    /// at-rule whose prelude was rejected ends: before the `;` or `{` that
    /// ended it.
    pub(super) fn text_to_prelude_end(&self) -> &'s str {
        &self.source[self.start..self.prelude_end]
    }

    /// The text of the unit up to the opening token of its block, for a unit
    /// that ends there with its block skipped. The end of the unit for a unit
    /// without one.
    pub(super) fn text_to_block(&self) -> (&'s str, usize) {
        let end = self.block_contents.unwrap_or(self.end);
        (&self.source[self.start..end], end)
    }
}
