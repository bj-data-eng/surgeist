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

use crate::diagnostic::error::{unexpected_syntax_end, unexpected_syntax_token};
use crate::diagnostic::{CssError, CssRecoveryAction, CssRecoveryDiagnostic};
use crate::parser::syntax_types::{Attached, Site, Stream, SyntaxError};
use crate::source::{CssSourcePosition, CssSourceSnapshot, CssSourceSpan};

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

/// The line and column cssparser reports for `position`.
pub(super) fn location_of(position: crate::source::CssSourcePosition) -> SourceLocation {
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
    if error.is_refused() {
        // The meter refused a step: that is a resource failure, never the
        // end of the input.
        return refused_error(location);
    }
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

/// The error of a stream whose meter refused a step, at `location`.
pub(super) fn refused_error<'i>(location: SourceLocation) -> NativeError<'i> {
    crate::diagnostic::error::invalid_component_value(
        location,
        crate::component::CssComponentValueError::new(
            crate::component::values::CssComponentValueErrorKind::CapacityOverflow,
            crate::component::CssValueOrigin::Programmatic,
        ),
    )
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
/// Returns whether the meter refused a step and no unit was handed over for
/// it: the list was cut short without a trace, which the caller reports with
/// [`refusal_diagnostic`].
pub(super) fn drive_units<'i, G>(
    contents: &mut Stream<'i>,
    grammar: &mut G,
    mut unit: impl FnMut(Result<G::Item, Failure<'i>>),
) -> bool
where
    G: BlockGrammar<'i, Attached<'i>, Error = NativeError<'i>>,
{
    let mut stream = contents.clone();
    let mut units = stream.block_contents(&mut *grammar);
    let mut reported = false;
    for item in units.by_ref() {
        if let Err(failed) = &item {
            reported |= failed.start().document().is_refused();
        }
        unit(item);
    }
    stream = units.stream();
    let unreported = stream.is_refused() && !reported;
    *contents = stream;
    unreported
}

/// The diagnostic of a refusal that cut the list `stream` stands in short: a
/// resource failure from where the stream stopped to the end of its list.
pub(super) fn refusal_diagnostic(
    snapshot: &CssSourceSnapshot,
    stream: &Stream<'_>,
    action: CssRecoveryAction,
) -> Option<CssRecoveryDiagnostic> {
    let site = stream.site();
    let at = site.offset()?.value();
    let end = site
        .document()
        .list_end(site.list())
        .map_or(at, |end| end.value())
        .max(at);
    let error = crate::diagnostic::error::from_parse_error(
        snapshot.as_str(),
        refused_error(location_at(snapshot, at)),
    );
    let span = CssSourceSpan::new(
        CssSourcePosition::from_byte_offset_in(snapshot.as_str(), at),
        CssSourcePosition::from_byte_offset_in(snapshot.as_str(), end),
    )?;
    CssRecoveryDiagnostic::new(error, span, action)
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
    // A unit handed over after the meter refused a step, or for the refusal,
    // is the unit the refusal cut short: it is reported with the rest of the
    // list.
    let refused = failed.error().is_refused() || document.is_refused();
    if refused && let Some(list_end) = document.list_end(after.list()) {
        end = end.max(list_end.value());
    }
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
    let error = if refused {
        refused_error(at_end)
    } else if begins_declaration {
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
