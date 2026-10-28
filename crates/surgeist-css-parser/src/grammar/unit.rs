//! The units the rule algorithms consume, and the failure of one.

use std::fmt;
use std::ops::Range;

use crate::{Offset, Site, Stream, SyntaxError};

/// What ended a rule or a declaration.
///
/// The driver knows how it found the end of a unit, so a diagnostic never has
/// to inspect the unit's bytes to tell a rule that ended at its semicolon from
/// one that ended at a `}` that belongs to its parent.
///
/// ```
/// use surgeist_css_parser::grammar::RuleEnd;
///
/// assert_ne!(RuleEnd::Semicolon, RuleEnd::Block);
/// ```
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum RuleEnd {
    /// A `<semicolon-token>`. An at-rule consumes it (section 5.5.2); a
    /// declaration, and a qualified rule that a declaration context stopped at,
    /// leave it for the loop of section 5.5.5, which discards it.
    Semicolon,
    /// A `{}`-block, consumed with the rule (sections 5.5.2 and 5.5.3).
    Block,
    /// The end of the token stream.
    Eof,
    /// A `<}-token>` that ends the parent block and was not consumed, in a
    /// nested context (sections 5.5.2, 5.5.3 and 5.5.6).
    CloseBrace,
}

/// The kind of unit a driver was consuming.
///
/// ```
/// use surgeist_css_parser::grammar::UnitKind;
///
/// assert_ne!(UnitKind::AtRule, UnitKind::Declaration);
/// ```
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum UnitKind {
    /// An at-rule (section 5.5.2).
    AtRule,
    /// A qualified rule (section 5.5.3).
    QualifiedRule,
    /// A declaration (section 5.5.6).
    Declaration,
}

fn range_between<P>(start: Site<'_, P>, end: Site<'_, P>) -> Option<Range<Offset>> {
    Some(start.offset()?..end.offset()?)
}

/// An at-rule consumed by section 5.5.2: its name, its prelude, its block if it
/// has one, and where it ended.
///
/// The prelude and the block are streams over the document, already
/// delimited: the driver has consumed the whole rule, so a grammar that rejects
/// it leaves the stream at the end of the rule and there is nothing to recover.
/// Whether the rule is valid in its context is the grammar's decision ("rule is
/// valid in the current context", section 5.5.2): it returns the rule's item or
/// an error from [`RuleGrammar::at_rule`](super::RuleGrammar::at_rule).
///
/// `Debug` is written by hand and asks nothing of the payload type `P`.
///
/// ```
/// use surgeist_css_parser::Document;
/// use surgeist_css_parser::grammar::{AtRule, RuleEnd, RuleGrammar};
/// use surgeist_css_parser::lexer::Options;
/// use surgeist_css_parser::limits::{DepthExcess, Limits};
/// use surgeist_css_parser::{Stream, SyntaxError};
///
/// struct Names;
/// impl<'i> RuleGrammar<'i> for Names {
///     type Item = (String, bool, RuleEnd);
///     type Error = ();
///     fn at_rule(&mut self, rule: AtRule<'i>) -> Result<Self::Item, SyntaxError<'i, ()>> {
///         Ok((rule.name().to_owned(), rule.block().is_some(), rule.rule_end()))
///     }
///     fn qualified_rule(
///         &mut self,
///         rule: surgeist_css_parser::grammar::QualifiedRule<'i>,
///     ) -> Result<Self::Item, SyntaxError<'i, ()>> {
///         Err(SyntaxError::invalid_qualified_rule_error(rule.start()))
///     }
/// }
///
/// let document: Document =
///     Document::parse("@charset 'x'; @media a {}", &Options::default(), Limits::default(), DepthExcess::Fail)?;
/// let mut grammar = Names;
/// let rules: Vec<_> = document.root().stylesheet_contents(&mut grammar).map(Result::unwrap).collect();
/// assert_eq!(rules[0], ("charset".to_owned(), false, RuleEnd::Semicolon));
/// assert_eq!(rules[1], ("media".to_owned(), true, RuleEnd::Block));
/// # Ok::<(), surgeist_css_parser::document::BuildError>(())
/// ```
pub struct AtRule<'i, P = ()> {
    pub(super) name: &'i str,
    pub(super) prelude: Stream<'i, P>,
    pub(super) block: Option<Stream<'i, P>>,
    pub(super) start: Site<'i, P>,
    pub(super) end: Site<'i, P>,
    pub(super) rule_end: RuleEnd,
}

impl<'i, P> AtRule<'i, P> {
    /// Returns the decoded name, without the `@`.
    #[must_use]
    pub const fn name(&self) -> &'i str {
        self.name
    }

    /// Returns the prelude: the component values between the name and the
    /// semicolon, the block or the end, whitespace included.
    #[must_use]
    pub fn prelude(&self) -> Stream<'i, P> {
        self.prelude.clone()
    }

    /// Returns the contents of the rule's `{}`-block, if it has one: the
    /// component values between the braces, for the grammar to read with
    /// [`block_contents`](Stream::block_contents) or as it sees fit.
    #[must_use]
    pub fn block(&self) -> Option<Stream<'i, P>> {
        self.block.clone()
    }

    /// Returns the boundary before the at-keyword.
    #[must_use]
    pub const fn start(&self) -> Site<'i, P> {
        self.start
    }

    /// Returns the boundary after the rule: after its semicolon or block, or at
    /// the `}` or the end that ended it.
    #[must_use]
    pub const fn end(&self) -> Site<'i, P> {
        self.end
    }

    /// Returns the interval of the rule in the text, when the document knows
    /// it.
    #[must_use]
    pub fn range(&self) -> Option<Range<Offset>> {
        range_between(self.start, self.end)
    }

    /// Returns what ended the rule.
    #[must_use]
    pub const fn rule_end(&self) -> RuleEnd {
        self.rule_end
    }
}

impl<P> fmt::Debug for AtRule<'_, P> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("AtRule")
            .field("name", &self.name)
            .field("prelude", &self.prelude)
            .field("block", &self.block)
            .field("start", &self.start)
            .field("end", &self.end)
            .field("rule_end", &self.rule_end)
            .finish()
    }
}

/// A qualified rule consumed by section 5.5.3: a prelude and a `{}`-block.
///
/// A qualified rule always has its block; one that reached the end of the
/// stream, a stop token or a `}` first is not a rule and is reported as a
/// failure. As for an [`AtRule`], the driver has consumed the whole rule before
/// the grammar sees it.
///
/// `Debug` is written by hand and asks nothing of the payload type `P`.
pub struct QualifiedRule<'i, P = ()> {
    pub(super) prelude: Stream<'i, P>,
    pub(super) block: Stream<'i, P>,
    pub(super) start: Site<'i, P>,
    pub(super) end: Site<'i, P>,
}

impl<'i, P> QualifiedRule<'i, P> {
    /// Returns the prelude: the component values before the block, whitespace
    /// included.
    #[must_use]
    pub fn prelude(&self) -> Stream<'i, P> {
        self.prelude.clone()
    }

    /// Returns the contents of the rule's `{}`-block.
    #[must_use]
    pub fn block(&self) -> Stream<'i, P> {
        self.block.clone()
    }

    /// Returns the boundary before the rule's first component value.
    #[must_use]
    pub const fn start(&self) -> Site<'i, P> {
        self.start
    }

    /// Returns the boundary after the rule's block.
    #[must_use]
    pub const fn end(&self) -> Site<'i, P> {
        self.end
    }

    /// Returns the interval of the rule in the text, when the document knows
    /// it.
    #[must_use]
    pub fn range(&self) -> Option<Range<Offset>> {
        range_between(self.start, self.end)
    }
}

impl<P> fmt::Debug for QualifiedRule<'_, P> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("QualifiedRule")
            .field("prelude", &self.prelude)
            .field("block", &self.block)
            .field("start", &self.start)
            .field("end", &self.end)
            .finish()
    }
}

/// A declaration consumed by section 5.5.6: a name, a value, and whether it was
/// marked `!important`.
///
/// The value is the component values after the colon with the leading and
/// trailing whitespace removed and, when the declaration is important, without
/// the `!` and `important` (steps 4, 6 and 7). The driver has already refused
/// the declarations the section refuses itself: one that does not start with an
/// ident and a colon, and a non-custom property whose value holds a `{}`-block
/// beside any other value (step 8). Whether the declaration is valid in its
/// context is the grammar's decision.
///
/// `Debug` is written by hand and asks nothing of the payload type `P`.
///
/// ```
/// use surgeist_css_parser::Document;
/// use surgeist_css_parser::grammar::{BlockGrammar, Declaration, QualifiedRule, RuleGrammar, AtRule};
/// use surgeist_css_parser::lexer::Options;
/// use surgeist_css_parser::limits::{DepthExcess, Limits};
/// use surgeist_css_parser::SyntaxError;
///
/// struct Pairs;
/// impl<'i> RuleGrammar<'i> for Pairs {
///     type Item = String;
///     type Error = ();
///     fn at_rule(&mut self, rule: AtRule<'i>) -> Result<String, SyntaxError<'i, ()>> {
///         Err(SyntaxError::unknown_at_rule_error(rule.start(), rule.name()))
///     }
///     fn qualified_rule(&mut self, rule: QualifiedRule<'i>) -> Result<String, SyntaxError<'i, ()>> {
///         Err(SyntaxError::invalid_qualified_rule_error(rule.start()))
///     }
/// }
/// impl<'i> BlockGrammar<'i> for Pairs {
///     fn declaration(&mut self, declaration: Declaration<'i>) -> Result<String, SyntaxError<'i, ()>> {
///         Ok(format!("{}{}", declaration.name(), if declaration.is_important() { "!" } else { "" }))
///     }
/// }
///
/// let document: Document =
///     Document::parse("color: red !important; --x: {a}", &Options::default(), Limits::default(), DepthExcess::Fail)?;
/// let mut grammar = Pairs;
/// let items: Vec<_> = document.root().block_contents(&mut grammar).map(Result::unwrap).collect();
/// assert_eq!(items, ["color!", "--x"]);
/// # Ok::<(), surgeist_css_parser::document::BuildError>(())
/// ```
pub struct Declaration<'i, P = ()> {
    pub(super) name: &'i str,
    pub(super) value: Stream<'i, P>,
    pub(super) important: bool,
    pub(super) original_text: Option<&'i str>,
    pub(super) start: Site<'i, P>,
    pub(super) end: Site<'i, P>,
    pub(super) rule_end: RuleEnd,
}

impl<'i, P> Declaration<'i, P> {
    /// Returns the decoded name. Its case is as written: section 5.5.6 does not
    /// fold it, and the grammar decides which names are case-insensitive.
    #[must_use]
    pub const fn name(&self) -> &'i str {
        self.name
    }

    /// Returns the value; see the type's documentation for what it excludes.
    #[must_use]
    pub fn value(&self) -> Stream<'i, P> {
        self.value.clone()
    }

    /// Returns whether the declaration ended in `!important`, the `important`
    /// matched ASCII case-insensitively (step 6).
    #[must_use]
    pub const fn is_important(&self) -> bool {
        self.important
    }

    /// Returns the segment of the original text that the tokens of the value
    /// were read from, for the two declarations whose value section 5.5.6 step 8
    /// asks for it: a custom property (a name starting with two hyphen-minuses),
    /// whose original text is the value, and `unicode-range` (ASCII
    /// case-insensitive), whose value is read again from the text with
    /// [`unicode_range::consume_value`](crate::unicode_range::consume_value).
    /// `None` for any other declaration, and for a document without source text.
    ///
    /// For `unicode-range` the specification replaces the declaration's value
    /// with the tokens read again. A value here is a stream over the document
    /// that was built, which cannot be replaced, so the grammar makes the
    /// replacement itself from this text; the tokens it reads are those the
    /// specification would have put in the value.
    #[must_use]
    pub const fn original_text(&self) -> Option<&'i str> {
        self.original_text
    }

    /// Returns the boundary before the name.
    #[must_use]
    pub const fn start(&self) -> Site<'i, P> {
        self.start
    }

    /// Returns the boundary after the declaration's value, at the semicolon,
    /// `}` or end that ended it. It is not consumed with the declaration.
    #[must_use]
    pub const fn end(&self) -> Site<'i, P> {
        self.end
    }

    /// Returns the interval of the declaration in the text, when the document
    /// knows it.
    #[must_use]
    pub fn range(&self) -> Option<Range<Offset>> {
        range_between(self.start, self.end)
    }

    /// Returns what ended the declaration: a semicolon, a `}` or the end of the
    /// stream.
    #[must_use]
    pub const fn rule_end(&self) -> RuleEnd {
        self.rule_end
    }
}

impl<P> fmt::Debug for Declaration<'_, P> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Declaration")
            .field("name", &self.name)
            .field("value", &self.value)
            .field("important", &self.important)
            .field("original_text", &self.original_text)
            .field("start", &self.start)
            .field("end", &self.end)
            .field("rule_end", &self.rule_end)
            .finish()
    }
}

/// A unit of a stylesheet or a block that was consumed and not accepted.
///
/// Section 5.5 drops what it does not accept ("return nothing"); a driver does
/// the same, and a diagnostic wants to know what and where. `Failed` is that
/// record: the kind of unit, why it was refused ([`error`](Self::error)), the
/// boundaries it covered, and how it ended.
///
/// A unit in a block that failed as a declaration is read again as a qualified
/// rule (section 5.5.5). When both attempts fail, the record holds the verdict
/// that decides the unit and, when it is the rule's, the declaration's error as
/// well ([`declaration_error`](Self::declaration_error)):
///
/// - the unit is a [`QualifiedRule`](UnitKind::QualifiedRule) when the second
///   attempt reached a `{}`-block, so the grammar judged it, and its
///   [`error`](Self::error) is the grammar's;
/// - the unit is a [`Declaration`](UnitKind::Declaration) when the second
///   attempt found no block (the stream, a stop token or a `}` came first), and
///   its error is the declaration's.
///
/// [`is_invalid_rule_error`](Self::is_invalid_rule_error) is the specification's
/// "invalid rule error": the grammar rejected a qualified rule that had its
/// block. Section 5.5.5 ends a run of declarations at one, and only at one.
///
/// A step that the meter refused is a `Failed` for which `is_refused()` holds.
/// The driver that reported it reports nothing after it.
///
/// A `Failed` is one allocation that the caller owns and drops; the drivers do
/// not charge it to the meter. `Debug` needs `E: Debug` and asks nothing of `P`;
/// `Display` shows the error.
pub struct Failed<'i, E, P = ()> {
    details: Box<Details<'i, E, P>>,
}

/// What a [`Failed`] holds, boxed so that a `Result` of a unit stays small.
pub(super) struct Details<'i, E, P> {
    pub(super) kind: UnitKind,
    pub(super) error: SyntaxError<'i, E, P>,
    pub(super) declaration_error: Option<SyntaxError<'i, E, P>>,
    pub(super) start: Site<'i, P>,
    pub(super) end: Site<'i, P>,
    pub(super) rule_end: Option<RuleEnd>,
    pub(super) invalid_rule: bool,
}

impl<'i, E, P> Failed<'i, E, P> {
    pub(super) fn new(
        kind: UnitKind,
        error: SyntaxError<'i, E, P>,
        start: Site<'i, P>,
        end: Site<'i, P>,
        rule_end: Option<RuleEnd>,
    ) -> Self {
        Self {
            details: Box::new(Details {
                kind,
                error,
                declaration_error: None,
                start,
                end,
                rule_end,
                invalid_rule: false,
            }),
        }
    }

    /// Marks the unit as the specification's invalid rule error, and records the
    /// declaration attempt's error if there was one.
    pub(super) fn with_rule_verdict(
        mut self,
        declaration_error: Option<SyntaxError<'i, E, P>>,
    ) -> Self {
        self.details.invalid_rule = !self.details.error.is_denied_block();
        self.details.declaration_error = declaration_error;
        self
    }

    /// Returns the kind of unit that failed. For a step the meter refused
    /// before the driver had read enough to know, it is the kind the driver
    /// tries first: a qualified rule in a stylesheet, a declaration in a
    /// block.
    #[must_use]
    pub fn kind(&self) -> UnitKind {
        self.details.kind
    }

    /// Returns why the unit was not accepted.
    #[must_use]
    pub fn error(&self) -> &SyntaxError<'i, E, P> {
        &self.details.error
    }

    /// Takes the error out.
    #[must_use]
    pub fn into_error(self) -> SyntaxError<'i, E, P> {
        self.details.error
    }

    /// Returns the error of the declaration attempt, when the unit was a
    /// qualified rule in a block that was first tried as a declaration.
    #[must_use]
    pub fn declaration_error(&self) -> Option<&SyntaxError<'i, E, P>> {
        self.details.declaration_error.as_ref()
    }

    /// Returns the boundary before the unit.
    #[must_use]
    pub fn start(&self) -> Site<'i, P> {
        self.details.start
    }

    /// Returns the boundary after what the driver consumed of the unit.
    #[must_use]
    pub fn end(&self) -> Site<'i, P> {
        self.details.end
    }

    /// Returns the interval of the unit in the text, when the document knows
    /// it.
    #[must_use]
    pub fn range(&self) -> Option<Range<Offset>> {
        range_between(self.details.start, self.details.end)
    }

    /// Returns what ended the unit, or `None` when the meter refused a step
    /// before the unit ended, or there was no unit to end.
    #[must_use]
    pub fn rule_end(&self) -> Option<RuleEnd> {
        self.details.rule_end
    }

    /// Returns whether the unit is the specification's "invalid rule error": a
    /// qualified rule that reached its `{}`-block and was rejected as invalid in
    /// its context.
    #[must_use]
    pub fn is_invalid_rule_error(&self) -> bool {
        self.details.invalid_rule
    }

    /// Returns whether the meter refused a step of the unit.
    #[must_use]
    pub fn is_refused(&self) -> bool {
        self.details.error.is_refused()
    }
}

impl<E: fmt::Debug, P> fmt::Debug for Failed<'_, E, P> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Failed")
            .field("kind", &self.details.kind)
            .field("error", &self.details.error)
            .field("declaration_error", &self.details.declaration_error)
            .field("start", &self.details.start)
            .field("end", &self.details.end)
            .field("rule_end", &self.details.rule_end)
            .field("invalid_rule", &self.details.invalid_rule)
            .finish()
    }
}

impl<E: fmt::Display, P> fmt::Display for Failed<'_, E, P> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.details.error, formatter)
    }
}

impl<E: fmt::Debug + fmt::Display, P> std::error::Error for Failed<'_, E, P> {}

/// A unit a driver consumed: the grammar's item, or the record of why it was
/// not accepted.
pub type Unit<'i, T, E, P = ()> = Result<T, Failed<'i, E, P>>;
