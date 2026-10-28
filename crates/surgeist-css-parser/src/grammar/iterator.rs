//! The drivers of sections 5.5.1 and 5.5.5, as iterators of units.

use std::fmt;

use super::algorithm::{
    Class, Qualified, classify, consume_at_rule, consume_declaration, consume_qualified_rule,
};
use super::{BlockGrammar, Failed, RuleEnd, RuleGrammar, Unit, UnitKind};
use crate::document::Component;
use crate::stream::StreamError;
use crate::{Stream, SyntaxError};

/// What a driver finds next: a unit, the end of its stream, or a refusal.
type Advanced<'i, T, E, P> = Result<Option<Unit<'i, T, E, P>>, StreamError<'i, P>>;

/// Turns a qualified rule attempt that reached its block into the unit.
pub(super) fn qualified_unit<'i, T, E, P>(
    result: Result<T, SyntaxError<'i, E, P>>,
    start: crate::Site<'i, P>,
    end: crate::Site<'i, P>,
) -> Unit<'i, T, E, P> {
    result.map_err(|error| {
        Failed::new(
            UnitKind::QualifiedRule,
            error,
            start,
            end,
            Some(RuleEnd::Block),
        )
        .with_rule_verdict(None)
    })
}

/// The record of a refused step.
pub(super) fn refusal<'i, E, P>(error: StreamError<'i, P>, kind: UnitKind) -> Failed<'i, E, P> {
    let at = error.at();
    Failed::new(kind, error.into(), at, at, None)
}

/// The units of a stylesheet's contents (section 5.5.1): an iterator over the
/// rules of a stream, each handed to the grammar once its prelude and block
/// have been consumed.
///
/// Whitespace, `<CDO-token>` and `<CDC-token>` between rules are discarded.
/// An at-keyword starts an at-rule (section 5.5.2) and anything else a
/// qualified rule (section 5.5.3); both are consumed whole before the grammar
/// sees them. Each item is the grammar's item for a rule it accepted, or the
/// [`Failed`] record of a rule it did not: the rules the specification drops
/// ("return nothing", an invalid rule error) are reported rather than
/// vanishing, so a diagnostic can say what was dropped and where. A qualified
/// rule that looks like a custom property (an ident that starts with two
/// hyphen-minuses and a colon before its block) is consumed with its block and
/// reported as nothing, because the specification does not call it an error.
///
/// If the meter refuses a step, the next item is a `Failed` for which
/// `is_refused()` holds, and the iterator ends. A refusal is never reported as
/// the end of the stream or as a rule that was dropped.
///
/// Created by [`Stream::stylesheet_contents`].
///
/// ```
/// use surgeist_css_parser::grammar::{AtRule, QualifiedRule, RuleGrammar};
/// use surgeist_css_parser::lexer::Options;
/// use surgeist_css_parser::limits::{DepthExcess, Limits};
/// use surgeist_css_parser::{Document, SyntaxError};
///
/// struct Selectors;
/// impl<'i> RuleGrammar<'i> for Selectors {
///     type Item = String;
///     type Error = ();
///     fn at_rule(&mut self, rule: AtRule<'i>) -> Result<String, SyntaxError<'i, ()>> {
///         Err(SyntaxError::unknown_at_rule_error(rule.start(), rule.name()))
///     }
///     fn qualified_rule(&mut self, rule: QualifiedRule<'i>) -> Result<String, SyntaxError<'i, ()>> {
///         let mut prelude = rule.prelude();
///         let name = prelude.consume_ident()?;
///         Ok(name.to_owned())
///     }
/// }
///
/// let document: Document = Document::parse(
///     "<!-- a {} @x; b {} -->",
///     &Options::default(),
///     Limits::default(),
///     DepthExcess::Fail,
/// )?;
/// let mut grammar = Selectors;
/// let units: Vec<_> = document.root().stylesheet_contents(&mut grammar).collect();
/// assert_eq!(units.len(), 3);
/// assert_eq!(units[0].as_ref().unwrap(), "a");
/// assert!(units[1].as_ref().unwrap_err().error().is_unknown_at_rule());
/// assert_eq!(units[2].as_ref().unwrap(), "b");
/// # Ok::<(), surgeist_css_parser::document::BuildError>(())
/// ```
pub struct StylesheetContents<'i, 'g, G, P = ()> {
    pub(super) stream: Stream<'i, P>,
    pub(super) grammar: &'g mut G,
    pub(super) done: bool,
    pub(super) attempt: UnitKind,
}

impl<'i, G, P> StylesheetContents<'i, '_, G, P> {
    /// Returns the stream as far as the driver has consumed it.
    #[must_use]
    pub fn stream(&self) -> Stream<'i, P> {
        self.stream.clone()
    }
}

impl<G, P> fmt::Debug for StylesheetContents<'_, '_, G, P> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("StylesheetContents")
            .field("stream", &self.stream)
            .field("done", &self.done)
            .finish()
    }
}

impl<'i, G, P> StylesheetContents<'i, '_, G, P>
where
    G: RuleGrammar<'i, P>,
{
    fn advance(&mut self) -> Advanced<'i, G::Item, G::Error, P> {
        let mut index = self.stream.next_index();
        loop {
            self.attempt = UnitKind::QualifiedRule;
            let Some(component) = self.stream.peek_at(index)? else {
                self.stream.seek(self.stream.end_index());
                self.done = true;
                return Ok(None);
            };
            match classify(&component) {
                // Whitespace, CDO and CDC are discarded (a comment is not a
                // token of the specification, so it goes with them).
                Class::Whitespace | Class::Comment | Class::Cdo | Class::Cdc => index += 1,
                Class::AtKeyword(name) => {
                    self.attempt = UnitKind::AtRule;
                    self.stream.seek(index);
                    let outcome = consume_at_rule(&mut self.stream, name, false, self.grammar)?;
                    return Ok(Some(outcome.unit));
                }
                _ => {
                    self.stream.seek(index);
                    let first = Some(component);
                    match consume_qualified_rule(
                        &mut self.stream,
                        first,
                        false,
                        false,
                        self.grammar,
                    )? {
                        Qualified::Rule { result, start, end } => {
                            return Ok(Some(qualified_unit(result, start, end)));
                        }
                        Qualified::Nothing {
                            reason,
                            start,
                            end,
                            rule_end,
                        } => {
                            return Ok(Some(Err(Failed::new(
                                UnitKind::QualifiedRule,
                                reason.into(),
                                start,
                                end,
                                Some(rule_end),
                            ))));
                        }
                        Qualified::Quiet { .. } => index = self.stream.next_index(),
                    }
                }
            }
        }
    }
}

impl<'i, G, P> Iterator for StylesheetContents<'i, '_, G, P>
where
    G: RuleGrammar<'i, P>,
{
    type Item = Unit<'i, G::Item, G::Error, P>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }
        match self.advance() {
            Ok(unit) => unit,
            Err(error) => {
                self.done = true;
                Some(Err(refusal(error, self.attempt)))
            }
        }
    }
}

/// The units of a block's contents (section 5.5.5): an iterator over the
/// declarations and rules of a stream, each handed to the grammar once it has
/// been consumed.
///
/// Whitespace and semicolons between units are discarded. An at-keyword starts
/// an at-rule; anything else is first tried as a declaration and, when that is
/// not accepted, read again from the same place as a qualified rule that a
/// semicolon ends (section 5.5.5, "mark" and "restore a mark"). A `}` ends the
/// contents and is left in the stream, as is the end of the stream; so a
/// `{}`-block's list, which has no `}` of its own, ends at its end.
///
/// Each item is the grammar's item for a unit it accepted, or the [`Failed`]
/// record of one it did not. Section 5.5.5 groups the declarations it
/// collects: a run of declarations is one list, and the list ends before an
/// at-rule, before a qualified rule and after an invalid rule error, but not
/// when a unit returned nothing. A consumer rebuilds that grouping from the
/// items: an at-rule (accepted or not), an accepted qualified rule and a
/// [`Failed`] for which [`is_invalid_rule_error`](Failed::is_invalid_rule_error)
/// holds end the run; a declaration, and any other failure, do not. A qualified
/// rule whose `{}`-block was denied for nesting too deep is not an invalid rule
/// error: it is a `Failed` of kind
/// [`QualifiedRule`](UnitKind::QualifiedRule) whose error holds
/// [`is_denied_block`](SyntaxError::is_denied_block) and for which
/// `is_invalid_rule_error` is false, because the grammar never judged it. A
/// consumer that groups treats it as any other failure (it ends no run), or as
/// it treats an invalid rule error if it wants the denied rule to separate the
/// declarations around it.
///
/// A declaration the grammar rejects, whose text then reads as a qualified rule
/// that is a custom-property look-alike, is reported as a failed declaration
/// with the grammar's error: the specification drops the remnants without an
/// error, but the grammar's verdict is not lost.
///
/// If the meter refuses a step, the next item is a `Failed` for which
/// `is_refused()` holds, and the iterator ends.
///
/// Created by [`Stream::block_contents`].
///
/// ```
/// use surgeist_css_parser::grammar::{AtRule, BlockGrammar, Declaration, QualifiedRule, RuleGrammar};
/// use surgeist_css_parser::lexer::Options;
/// use surgeist_css_parser::limits::{DepthExcess, Limits};
/// use surgeist_css_parser::{Document, SyntaxError};
///
/// struct Names;
/// impl<'i> RuleGrammar<'i> for Names {
///     type Item = String;
///     type Error = ();
///     fn at_rule(&mut self, rule: AtRule<'i>) -> Result<String, SyntaxError<'i, ()>> {
///         Ok(format!("@{}", rule.name()))
///     }
///     fn qualified_rule(&mut self, _: QualifiedRule<'i>) -> Result<String, SyntaxError<'i, ()>> {
///         Ok("rule".to_owned())
///     }
/// }
/// impl<'i> BlockGrammar<'i> for Names {
///     fn declaration(&mut self, d: Declaration<'i>) -> Result<String, SyntaxError<'i, ()>> {
///         Ok(d.name().to_owned())
///     }
/// }
///
/// let document: Document = Document::parse(
///     "color: red; &:hover { x: y } @media print {} width: 1px",
///     &Options::default(),
///     Limits::default(),
///     DepthExcess::Fail,
/// )?;
/// let mut grammar = Names;
/// let items: Vec<_> = document.root().block_contents(&mut grammar).map(Result::unwrap).collect();
/// assert_eq!(items, ["color", "rule", "@media", "width"]);
/// # Ok::<(), surgeist_css_parser::document::BuildError>(())
/// ```
pub struct BlockContents<'i, 'g, G, P = ()> {
    pub(super) stream: Stream<'i, P>,
    pub(super) grammar: &'g mut G,
    pub(super) done: bool,
    pub(super) attempt: UnitKind,
}

impl<'i, G, P> BlockContents<'i, '_, G, P> {
    /// Returns the stream as far as the driver has consumed it: after the last
    /// unit, and at the `}` that ended the contents if one did.
    #[must_use]
    pub fn stream(&self) -> Stream<'i, P> {
        self.stream.clone()
    }
}

impl<G, P> fmt::Debug for BlockContents<'_, '_, G, P> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("BlockContents")
            .field("stream", &self.stream)
            .field("done", &self.done)
            .finish()
    }
}

impl<'i, G, P> BlockContents<'i, '_, G, P>
where
    G: BlockGrammar<'i, P>,
{
    fn advance(&mut self) -> Advanced<'i, G::Item, G::Error, P> {
        let mut index = self.stream.next_index();
        loop {
            self.attempt = UnitKind::Declaration;
            let Some(component) = self.stream.peek_at(index)? else {
                self.stream.seek(self.stream.end_index());
                self.done = true;
                return Ok(None);
            };
            match classify(&component) {
                Class::Whitespace | Class::Comment | Class::Semicolon => index += 1,
                Class::CloseCurly => {
                    self.stream.seek(index);
                    self.done = true;
                    return Ok(None);
                }
                Class::AtKeyword(name) => {
                    self.attempt = UnitKind::AtRule;
                    self.stream.seek(index);
                    let outcome = consume_at_rule(&mut self.stream, name, true, self.grammar)?;
                    return Ok(Some(outcome.unit));
                }
                _ => {
                    self.stream.seek(index);
                    return Ok(Some(self.declaration_or_rule(component)?));
                }
            }
        }
    }

    /// The "anything else" branch of section 5.5.5: mark, consume a
    /// declaration, and when it is not accepted restore the mark and consume a
    /// qualified rule with a semicolon as the stop token.
    fn declaration_or_rule(
        &mut self,
        first: Component<'i, P>,
    ) -> Result<Unit<'i, G::Item, G::Error, P>, StreamError<'i, P>> {
        let mark = self.stream.mark();
        let declaration_error =
            match consume_declaration(&mut self.stream, Some(first.clone()), true) {
                Ok(declaration) => match self.grammar.declaration(declaration) {
                    Ok(item) => return Ok(Ok(item)),
                    Err(error) => error,
                },
                Err(error) if error.is_refused() => return Err(error),
                Err(error) => SyntaxError::from(error),
            };
        self.stream.restore(mark);
        self.attempt = UnitKind::QualifiedRule;
        let unit = match consume_qualified_rule(
            &mut self.stream,
            Some(first),
            true,
            true,
            self.grammar,
        )? {
            Qualified::Rule {
                result: Ok(item), ..
            } => Ok(item),
            Qualified::Rule {
                result: Err(error),
                start,
                end,
            } => Err(Failed::new(
                UnitKind::QualifiedRule,
                error,
                start,
                end,
                Some(RuleEnd::Block),
            )
            .with_rule_verdict(Some(declaration_error))),
            Qualified::Nothing {
                start,
                end,
                rule_end,
                ..
            }
            | Qualified::Quiet {
                start,
                end,
                rule_end,
            } => Err(Failed::new(
                UnitKind::Declaration,
                declaration_error,
                start,
                end,
                Some(rule_end),
            )),
        };
        Ok(unit)
    }
}

impl<'i, G, P> Iterator for BlockContents<'i, '_, G, P>
where
    G: BlockGrammar<'i, P>,
{
    type Item = Unit<'i, G::Item, G::Error, P>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }
        match self.advance() {
            Ok(unit) => unit,
            Err(error) => {
                self.done = true;
                Some(Err(refusal(error, self.attempt)))
            }
        }
    }
}
