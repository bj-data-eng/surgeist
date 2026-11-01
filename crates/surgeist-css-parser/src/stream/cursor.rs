//! The token stream.

use std::fmt;
use std::ops::Range;
use std::ptr;

use super::{Mark, Site, StopToken, StopTokens, StreamError, SyntaxError};
use crate::document::{BlockKind, Component, GroupKind, ListId, NodeId};
use crate::limits::{self, Meter};
use crate::{Document, Offset, Token};

/// A token stream (section 5.3): a position in one list of a [`Document`], from
/// which a grammar consumes tokens, enters functions and blocks, and splits
/// lists.
///
/// A stream covers a span of a list. [`next_token`](Self::next_token),
/// [`consume_token`](Self::consume_token) and the other consuming methods move
/// its index forward; [`mark`](Self::mark) and [`restore`](Self::restore) move
/// it back. A function or simple block is one token to a stream: consuming its
/// opening token leaves the group unentered, and [`within_block`](Self::within_block)
/// enters it, so a grammar that does not enter a group skips it in one step.
/// [`within_list_until`](Self::within_list_until) hands a grammar the part of
/// the stream before the next stop token as a stream of its own.
///
/// The methods follow the terms of section 5.3: "next token" (the token a
/// consume would return, or the `<eof-token>`, which is `None` here), "empty",
/// "consume a token", "discard whitespace", "mark" and "restore a mark".
/// Comments, which Syntax 3 does not tokenize, are transparent to all of them
/// except [`consume_token_with_comments`](Self::consume_token_with_comments);
/// [`discard_whitespace`](Self::discard_whitespace) discards them like
/// whitespace.
///
/// # Metering
///
/// When the document was built with a [`Meter`], every observation of the
/// stream takes one step of it per node it examines (a comment skipped counts,
/// and so does finding the end of the stream, and so does each node
/// [`within_list_until`](Self::within_list_until) looks at for a stop token):
/// [`next_token`](Self::next_token), the consuming methods,
/// [`only_whitespace_remains`](Self::only_whitespace_remains),
/// [`require_empty`](Self::require_empty),
/// [`discard_whitespace`](Self::discard_whitespace) and
/// [`within_list_until`](Self::within_list_until).
///
/// The first step the meter refuses is latched on the document
/// ([`Document::is_refused`]): from then on every metered step of every stream
/// over it fails without asking the meter again, so a host need not remember
/// that it refused. A method that can return an error returns one for which
/// `is_refused()` holds. The methods that cannot report a failure fall back to
/// the end of the stream: `next_token` returns `None`, `is_empty` and
/// [`only_whitespace_remains`](Self::only_whitespace_remains) return `true` and
/// [`discard_whitespace`](Self::discard_whitespace) stops;
/// [`is_refused`](Self::is_refused) tells that fallback from a stream that
/// really ended.
///
/// `Clone` and `Debug` are written by hand and ask nothing of the payload type
/// `P`.
///
/// ```
/// use surgeist_css_parser::lexer::Options;
/// use surgeist_css_parser::limits::{DepthExcess, Limits};
/// use surgeist_css_parser::stream::StopToken;
/// use surgeist_css_parser::{Document, Stream, SyntaxError};
///
/// // A comma-separated list of identifier pairs.
/// fn pair<'i>(stream: &mut Stream<'i>) -> Result<(&'i str, &'i str), SyntaxError<'i, ()>> {
///     Ok((stream.consume_ident()?, stream.consume_ident()?))
/// }
///
/// let document: Document = Document::parse(
///     "a b, c d",
///     &Options::default(),
///     Limits::default(),
///     0,
///     DepthExcess::Fail,
/// )?;
/// let mut stream = document.root();
/// let pairs = stream.comma_segments(pair).unwrap();
/// assert_eq!(pairs, [("a", "b"), ("c", "d")]);
/// # Ok::<(), surgeist_css_parser::document::BuildError>(())
/// ```
pub struct Stream<'i, P = ()> {
    document: &'i Document<'i, P>,
    list: ListId,
    start: usize,
    next: usize,
    end: usize,
    /// The group the last consume opened and nobody entered.
    pending: Option<NodeId>,
}

impl<P> Clone for Stream<'_, P> {
    fn clone(&self) -> Self {
        Self {
            document: self.document,
            list: self.list,
            start: self.start,
            next: self.next,
            end: self.end,
            pending: self.pending,
        }
    }
}

impl<P> fmt::Debug for Stream<'_, P> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Stream")
            .field("list", &self.list)
            .field("start", &self.start)
            .field("next", &self.next)
            .field("end", &self.end)
            .field("pending", &self.pending)
            .finish()
    }
}

impl<'i, P> Stream<'i, P> {
    /// Creates a stream over the nodes `range` of `list`.
    ///
    /// # Panics
    ///
    /// Panics if the range is not within the list.
    pub(crate) fn bounded(
        document: &'i Document<'i, P>,
        list: ListId,
        range: Range<usize>,
    ) -> Self {
        assert!(
            range.start <= range.end && range.end <= document.list(list).len(),
            "a stream covers part of one list"
        );
        Self {
            document,
            list,
            start: range.start,
            next: range.start,
            end: range.end,
            pending: None,
        }
    }

    // ----- the stream's own state

    /// Returns the meter the stream's document was built with.
    ///
    /// A grammar that builds storage of its own while parsing, a list of
    /// values for example, charges it through the same meter, with
    /// [`limits::grow`] and the other helpers of that module.
    #[must_use]
    pub fn meter(&self) -> Option<&'i dyn Meter> {
        let document: &'i Document<'i, P> = self.document;
        document.meter()
    }

    /// Returns the boundary after the token consumed last, which is also the
    /// boundary before the next token when no comment stands between them.
    /// Comments are transparent to the consuming methods but not to this
    /// boundary: a comment after the token consumed last is after this
    /// boundary. [`next_site`](Self::next_site) is the boundary before the next
    /// token itself.
    #[must_use]
    pub const fn site(&self) -> Site<'i, P> {
        Site::new(self.document, self.list, self.next, false)
    }

    /// Returns the boundary before the next token: [`site`](Self::site) with the
    /// comments in front of that token passed over, which is where the token
    /// [`consume_token`](Self::consume_token) would return starts. At the end of
    /// the stream, or after a refusal of the meter, it is the end boundary and
    /// the current boundary respectively.
    ///
    /// Like [`next_token`](Self::next_token) it examines nodes, and takes one
    /// meter step for each it examines: the comments it passes over and the
    /// token it stops at, or the end of the stream.
    ///
    /// ```
    /// use surgeist_css_parser::lexer::Options;
    /// use surgeist_css_parser::limits::{DepthExcess, Limits};
    /// use surgeist_css_parser::{Document, Offset};
    ///
    /// let document: Document =
    ///     Document::parse("a/**/b", &Options::default(), Limits::default(), 0, DepthExcess::Fail)?;
    /// let mut stream = document.root();
    /// stream.consume_ident().unwrap();
    /// assert_eq!(stream.site().offset(), Some(Offset::new(1)));
    /// assert_eq!(stream.next_site().offset(), Some(Offset::new(5)));
    /// # Ok::<(), surgeist_css_parser::document::BuildError>(())
    /// ```
    #[must_use]
    pub fn next_site(&self) -> Site<'i, P> {
        match self.find(false) {
            Ok(Some(index)) => Site::new(self.document, self.list, index, false),
            Ok(None) => self.end_site(),
            Err(_) => self.site(),
        }
    }

    /// Returns the boundary at the start of the span the stream covers.
    #[must_use]
    pub const fn start_site(&self) -> Site<'i, P> {
        Site::new(self.document, self.list, self.start, false)
    }

    /// Returns the boundary at the end of the span the stream covers.
    #[must_use]
    pub const fn end_site(&self) -> Site<'i, P> {
        Site::new(self.document, self.list, self.end, false)
    }

    /// Returns where the stream is in the text, as [`Site::offset`] does for
    /// [`site`](Self::site).
    #[must_use]
    pub fn offset(&self) -> Option<Offset> {
        self.site().offset()
    }

    /// Returns a restore point.
    #[must_use]
    pub const fn mark(&self) -> Mark<'i, P> {
        Mark {
            site: self.site(),
            pending: self.pending,
            start: self.start,
            end: self.end,
        }
    }

    /// Restores the stream to `mark`, which this stream or a copy of it took.
    ///
    /// # Panics
    ///
    /// Panics if `mark` was taken on a stream over another document, another
    /// list or another span: using it would be a bug.
    pub fn restore(&mut self, mark: Mark<'i, P>) {
        assert!(
            ptr::eq(self.document, mark.site.document())
                && self.list == mark.site.list()
                && self.start == mark.start
                && self.end == mark.end
                && (self.start..=self.end).contains(&mark.site.index()),
            "a mark is restored on the stream it was taken on"
        );
        self.next = mark.site.index();
        self.pending = mark.pending;
    }

    // ----- observing

    fn node_at(&self, index: usize) -> NodeId {
        self.document.list(self.list)[index]
    }

    fn token_at(&self, index: usize) -> Token<'i> {
        let document: &'i Document<'i, P> = self.document;
        document.token(self.node_at(index))
    }

    fn step(&self) -> Result<(), StreamError<'i, P>> {
        let document: &'i Document<'i, P> = self.document;
        if document.admit_step() {
            Ok(())
        } else {
            Err(StreamError::refused(self.site()))
        }
    }

    /// Returns whether the meter has refused an action asked of it through a
    /// stream over this stream's document. A grammar uses it to tell a stream
    /// that ended from one that was refused after a method that cannot report
    /// a failure ([`next_token`](Self::next_token), [`is_empty`](Self::is_empty),
    /// [`only_whitespace_remains`](Self::only_whitespace_remains)) fell back to
    /// the end of the stream. See [`Document::is_refused`].
    #[must_use]
    pub fn is_refused(&self) -> bool {
        self.document.is_refused()
    }

    /// Finds the index of the next token, skipping comments unless `comments`.
    fn find(&self, comments: bool) -> Result<Option<usize>, StreamError<'i, P>> {
        let mut index = self.next;
        loop {
            self.step()?;
            if index >= self.end {
                return Ok(None);
            }
            if !comments && self.token_at(index) == Token::Comment {
                index += 1;
                continue;
            }
            return Ok(Some(index));
        }
    }

    /// Consumes the token at `index`.
    fn take(&mut self, index: usize) -> Token<'i> {
        let node = self.node_at(index);
        self.next = index + 1;
        self.pending = self.document.children(node).map(|_| node);
        self.token_at(index)
    }

    /// Returns the next token without consuming it: the token
    /// [`consume_token`](Self::consume_token) would return. `None` is the
    /// `<eof-token>`. Whitespace is a token; comments are skipped.
    ///
    /// ```
    /// use surgeist_css_parser::lexer::Options;
    /// use surgeist_css_parser::limits::{DepthExcess, Limits};
    /// use surgeist_css_parser::{Document, Token};
    ///
    /// let document: Document =
    ///     Document::parse("/* c */ a", &Options::default(), Limits::default(), 0, DepthExcess::Fail)?;
    /// let stream = document.root();
    /// assert_eq!(stream.next_token(), Some(Token::Whitespace));
    /// # Ok::<(), surgeist_css_parser::document::BuildError>(())
    /// ```
    #[must_use]
    pub fn next_token(&self) -> Option<Token<'i>> {
        match self.find(false) {
            Ok(Some(index)) => Some(self.token_at(index)),
            Ok(None) | Err(_) => None,
        }
    }

    /// Returns whether the stream is empty: its next token is the
    /// `<eof-token>`. A stream holding only whitespace is not empty; see
    /// [`only_whitespace_remains`](Self::only_whitespace_remains).
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.next_token().is_none()
    }

    /// Returns whether every token left in the stream is whitespace or a
    /// comment. An empty stream satisfies it. After a refusal it returns `true`;
    /// [`is_refused`](Self::is_refused) tells the two apart.
    #[must_use]
    pub fn only_whitespace_remains(&self) -> bool {
        self.whitespace_left().unwrap_or(true)
    }

    /// Whether only whitespace and comments are left, or the refusal that
    /// stopped the scan.
    pub(crate) fn whitespace_left(&self) -> Result<bool, StreamError<'i, P>> {
        let mut index = self.next;
        loop {
            self.step()?;
            if index >= self.end {
                return Ok(true);
            }
            if !self.token_at(index).is_trivia() {
                return Ok(false);
            }
            index += 1;
        }
    }

    /// Discards whitespace (section 5.3): every whitespace token and comment
    /// before the next other token. The group opened by the last consume, if it
    /// was not entered, is no longer pending.
    pub fn discard_whitespace(&mut self) {
        while self.next < self.end {
            if self.step().is_err() || !self.token_at(self.next).is_trivia() {
                break;
            }
            self.next += 1;
        }
        self.pending = None;
    }

    // ----- consuming

    /// Consumes a token (section 5.3). Comments are transparent: they are
    /// skipped, and whitespace is a token. If the token opens a function or a
    /// simple block, the group is left unentered; see
    /// [`within_block`](Self::within_block).
    ///
    /// # Errors
    ///
    /// Fails with `is_end_of_stream()` if the stream is empty, and with
    /// `is_refused()` if the meter refuses.
    pub fn consume_token(&mut self) -> Result<Token<'i>, StreamError<'i, P>> {
        match self.find(false)? {
            Some(index) => Ok(self.take(index)),
            None => Err(StreamError::end_of_stream(self.site())),
        }
    }

    /// Consumes a token, comments included: the one method through which a
    /// [`Token::Comment`] is visible. Section 9 allows an implementation to
    /// preserve comments provided they have no effect on parsing.
    ///
    /// # Errors
    ///
    /// As [`consume_token`](Self::consume_token).
    pub fn consume_token_with_comments(&mut self) -> Result<Token<'i>, StreamError<'i, P>> {
        match self.find(true)? {
            Some(index) => Ok(self.take(index)),
            None => Err(StreamError::end_of_stream(self.site())),
        }
    }

    /// Discards whitespace and then consumes a token.
    ///
    /// # Errors
    ///
    /// As [`consume_token`](Self::consume_token).
    pub fn consume_after_whitespace(&mut self) -> Result<Token<'i>, StreamError<'i, P>> {
        self.discard_whitespace();
        self.consume_token()
    }

    /// Requires the stream to be empty once whitespace is discarded.
    ///
    /// On success the stream is at its end. On failure it is where it was.
    ///
    /// # Errors
    ///
    /// Fails with `is_unexpected_token()` naming the first token left, and with
    /// `is_refused()` if the meter refuses.
    pub fn require_empty(&mut self) -> Result<(), StreamError<'i, P>> {
        let mark = self.mark();
        self.discard_whitespace();
        let at = self.site();
        match self.find(false) {
            Ok(None) => Ok(()),
            Ok(Some(index)) => {
                let token = self.token_at(index);
                self.restore(mark);
                Err(StreamError::unexpected(at, token))
            }
            Err(error) => {
                self.restore(mark);
                Err(error)
            }
        }
    }

    /// Discards whitespace, consumes a token and returns it if `accept` takes
    /// it, and otherwise fails naming it as unexpected. The token stays
    /// consumed either way; a caller that wants to try another reading
    /// uses [`attempt`](Self::attempt).
    fn consume_matching<T>(
        &mut self,
        accept: impl FnOnce(Token<'i>) -> Option<T>,
    ) -> Result<T, StreamError<'i, P>> {
        self.discard_whitespace();
        let at = self.site();
        let token = self.consume_token()?;
        accept(token).ok_or_else(|| StreamError::unexpected(at, token))
    }

    /// Discards whitespace and consumes an `<ident-token>`, returning its
    /// decoded value.
    ///
    /// # Errors
    ///
    /// Fails with `is_unexpected_token()` if the next token is another kind, and
    /// as [`consume_token`](Self::consume_token) otherwise.
    pub fn consume_ident(&mut self) -> Result<&'i str, StreamError<'i, P>> {
        self.consume_matching(|token| match token {
            Token::Ident(value) => Some(value),
            _ => None,
        })
    }

    /// Discards whitespace and consumes an `<ident-token>` that matches
    /// `keyword` ASCII case-insensitively.
    ///
    /// # Errors
    ///
    /// As [`consume_ident`](Self::consume_ident).
    pub fn consume_keyword(&mut self, keyword: &str) -> Result<(), StreamError<'i, P>> {
        self.consume_matching(|token| match token {
            Token::Ident(value) if value.eq_ignore_ascii_case(keyword) => Some(()),
            _ => None,
        })
    }

    /// Discards whitespace and consumes a `<function-token>`, returning its
    /// decoded name. The function's contents follow as a block to enter with
    /// [`within_block`](Self::within_block).
    ///
    /// # Errors
    ///
    /// As [`consume_ident`](Self::consume_ident).
    pub fn consume_function_name(&mut self) -> Result<&'i str, StreamError<'i, P>> {
        self.consume_matching(|token| match token {
            Token::Function(name) => Some(name),
            _ => None,
        })
    }

    /// Discards whitespace and consumes a `<function-token>` whose name matches
    /// `name` ASCII case-insensitively.
    ///
    /// # Errors
    ///
    /// As [`consume_ident`](Self::consume_ident).
    pub fn consume_function_named(&mut self, name: &str) -> Result<(), StreamError<'i, P>> {
        self.consume_matching(|token| match token {
            Token::Function(value) if value.eq_ignore_ascii_case(name) => Some(()),
            _ => None,
        })
    }

    /// Discards whitespace and consumes a `<string-token>`, returning its
    /// decoded value.
    ///
    /// # Errors
    ///
    /// As [`consume_ident`](Self::consume_ident).
    pub fn consume_string(&mut self) -> Result<&'i str, StreamError<'i, P>> {
        self.consume_matching(|token| match token {
            Token::String(value) => Some(value),
            _ => None,
        })
    }

    /// Discards whitespace and consumes the `<delim-token>` for `delim`.
    ///
    /// # Errors
    ///
    /// As [`consume_ident`](Self::consume_ident).
    pub fn consume_delim(&mut self, delim: char) -> Result<(), StreamError<'i, P>> {
        self.consume_matching(|token| match token {
            Token::Delim(value) if value == delim => Some(()),
            _ => None,
        })
    }

    /// Discards whitespace and consumes a `<comma-token>`.
    ///
    /// # Errors
    ///
    /// As [`consume_ident`](Self::consume_ident).
    pub fn consume_comma(&mut self) -> Result<(), StreamError<'i, P>> {
        self.consume_matching(|token| (token == Token::Comma).then_some(()))
    }

    /// Discards whitespace and consumes a `<colon-token>`.
    ///
    /// # Errors
    ///
    /// As [`consume_ident`](Self::consume_ident).
    pub fn consume_colon(&mut self) -> Result<(), StreamError<'i, P>> {
        self.consume_matching(|token| (token == Token::Colon).then_some(()))
    }

    /// Discards whitespace and consumes a `<semicolon-token>`.
    ///
    /// # Errors
    ///
    /// As [`consume_ident`](Self::consume_ident).
    pub fn consume_semicolon(&mut self) -> Result<(), StreamError<'i, P>> {
        self.consume_matching(|token| (token == Token::Semicolon).then_some(()))
    }

    // ----- groups and lists

    /// Returns the kind of the function or simple block that the last consume
    /// opened and nobody has entered, if there is one.
    #[must_use]
    pub fn unentered_block(&self) -> Option<GroupKind> {
        self.pending.and_then(|node| self.document.group_kind(node))
    }

    /// Runs `f` over the contents of the function or simple block the last
    /// consume opened (section 5.5.9 and 5.5.10) and requires the contents to be
    /// empty when `f` is done, whitespace aside. Entering consumes the group:
    /// afterwards the stream is past it, whether or not `f` succeeded.
    ///
    /// # Errors
    ///
    /// Fails with `is_block_expected()` when no group is waiting to be entered
    /// (no panic: a grammar fed untrusted text can reach this), with `f`'s own
    /// error, and with the error of [`require_empty`](Self::require_empty) when
    /// `f` leaves tokens behind.
    ///
    /// ```
    /// use surgeist_css_parser::lexer::Options;
    /// use surgeist_css_parser::limits::{DepthExcess, Limits};
    /// use surgeist_css_parser::{Document, SyntaxError};
    ///
    /// let document: Document =
    ///     Document::parse("f(a) b", &Options::default(), Limits::default(), 0, DepthExcess::Fail)?;
    /// let mut stream = document.root();
    /// stream.consume_function_named("F").unwrap();
    /// let inside = stream
    ///     .within_block(|inner| Ok::<_, SyntaxError<'_, ()>>(inner.consume_ident()?))
    ///     .unwrap();
    /// assert_eq!(inside, "a");
    /// assert_eq!(stream.consume_ident().ok(), Some("b"));
    /// # Ok::<(), surgeist_css_parser::document::BuildError>(())
    /// ```
    pub fn within_block<T, E>(
        &mut self,
        f: impl FnOnce(&mut Stream<'i, P>) -> Result<T, SyntaxError<'i, E, P>>,
    ) -> Result<T, SyntaxError<'i, E, P>> {
        let Some(node) = self.pending.take() else {
            return Err(StreamError::block_expected(self.site()).into());
        };
        let Some(children) = self.document.children(node) else {
            return Err(StreamError::block_expected(self.site()).into());
        };
        let mut nested = self.document.stream(children);
        let value = f(&mut nested)?;
        nested.require_empty()?;
        Ok(value)
    }

    fn is_stop(&self, index: usize, stops: StopTokens) -> bool {
        let stop = match self.document.component(self.node_at(index)) {
            Component::Token { token, .. } => match token {
                Token::Comma => StopToken::Comma,
                Token::Semicolon => StopToken::Semicolon,
                Token::Delim('!') => StopToken::ExclamationMark,
                _ => return false,
            },
            Component::Block {
                kind: BlockKind::Curly,
                ..
            }
            | Component::Denied { .. } => StopToken::OpenCurly,
            _ => return false,
        };
        stops.contains(stop)
    }

    /// Runs `f` over the part of the stream before the next stop token, a token
    /// of `stops` in this stream's own list (section 5.5.7), and requires that
    /// part to be empty when `f` is done, whitespace aside. Afterwards the stream
    /// is at the stop token, which is not consumed, or at its end.
    ///
    /// Looking for the stop token takes one meter step per node examined, the
    /// stop token included.
    ///
    /// # Errors
    ///
    /// With `f`'s own error, the error of [`require_empty`](Self::require_empty),
    /// and `is_refused()` if the meter refuses a step of the scan, in which case
    /// the stream has not moved.
    pub fn within_list_until<T, E>(
        &mut self,
        stops: impl Into<StopTokens>,
        f: impl FnOnce(&mut Stream<'i, P>) -> Result<T, SyntaxError<'i, E, P>>,
    ) -> Result<T, SyntaxError<'i, E, P>> {
        let stops = stops.into();
        let mut end = self.end;
        for index in self.next..self.end {
            self.step()?;
            if self.is_stop(index, stops) {
                end = index;
                break;
            }
        }
        let mut nested = Stream::bounded(self.document, self.list, self.next..end);
        let result = f(&mut nested).and_then(|value| {
            nested.require_empty()?;
            Ok(value)
        });
        self.next = end;
        self.pending = None;
        result
    }

    /// Runs `f` over each of the segments between top-level commas and collects
    /// the results: the helper for the `#` multiplier of Values 4, where the
    /// grammar decides what an empty segment means.
    ///
    /// Every segment is handed to `f`, empty ones included. A stream with no
    /// comma is one segment, and an empty stream is one empty segment, so
    /// `a,` is two segments. This is **not** the algorithm of section 5.4.2
    /// ("parse a comma-separated list according to a CSS grammar"), which returns
    /// an empty list for a stream of whitespace and ignores a final comma; that
    /// entry point is [`parse_comma_separated_list`](Self::parse_comma_separated_list).
    ///
    /// # Errors
    ///
    /// With `f`'s own error for the first segment it rejects, and with
    /// `is_refused()` if the meter refuses, whether a step or the growth of the
    /// result; a refusal is never turned into a shorter list.
    ///
    /// ```
    /// use surgeist_css_parser::lexer::Options;
    /// use surgeist_css_parser::limits::{DepthExcess, Limits};
    /// use surgeist_css_parser::{Document, SyntaxError};
    ///
    /// let document: Document =
    ///     Document::parse("a, b ,c", &Options::default(), Limits::default(), 0, DepthExcess::Fail)?;
    /// let names = document
    ///     .root()
    ///     .comma_segments(|item| Ok::<_, SyntaxError<'_, ()>>(item.consume_ident()?))
    ///     .unwrap();
    /// assert_eq!(names, ["a", "b", "c"]);
    /// # Ok::<(), surgeist_css_parser::document::BuildError>(())
    /// ```
    pub fn comma_segments<T, E>(
        &mut self,
        mut f: impl FnMut(&mut Stream<'i, P>) -> Result<T, SyntaxError<'i, E, P>>,
    ) -> Result<Vec<T>, SyntaxError<'i, E, P>> {
        let mut values = Vec::new();
        loop {
            let value = self.within_list_until(StopToken::Comma, &mut f)?;
            self.push_charged(&mut values, value)?;
            if self.whitespace_left()? {
                return Ok(values);
            }
            self.consume_comma()?;
        }
    }

    /// Runs `f`, and restores the stream to where it was if `f` fails.
    ///
    /// # Errors
    ///
    /// With `f`'s own error.
    pub fn attempt<T, E>(&mut self, f: impl FnOnce(&mut Self) -> Result<T, E>) -> Result<T, E> {
        let mark = self.mark();
        let result = f(self);
        if result.is_err() {
            self.restore(mark);
        }
        result
    }

    /// Runs `f` and then requires the stream to be empty (section 5.4.1, parse
    /// something according to a CSS grammar): the grammar must have matched
    /// everything.
    ///
    /// # Errors
    ///
    /// With `f`'s own error and the error of [`require_empty`](Self::require_empty).
    pub fn parse_complete<T, E>(
        &mut self,
        f: impl FnOnce(&mut Self) -> Result<T, SyntaxError<'i, E, P>>,
    ) -> Result<T, SyntaxError<'i, E, P>> {
        let value = f(self)?;
        self.require_empty()?;
        Ok(value)
    }

    // ----- source text

    /// Returns the text between two sites of this stream's document, as it was
    /// written. `None` when the document has no source text, when either site
    /// has no offset (components built without intervals), when the sites are
    /// of another document, or when `from` is after `to`.
    ///
    /// Retained output and the original text of a declaration (section 5.5.6,
    /// step 8) are taken this way; a grammar never parses this string.
    #[must_use]
    pub fn text_between(&self, from: Site<'i, P>, to: Site<'i, P>) -> Option<&'i str> {
        if !ptr::eq(self.document, from.document()) || !ptr::eq(self.document, to.document()) {
            return None;
        }
        let (from, to) = (from.offset()?, to.offset()?);
        if from > to {
            return None;
        }
        self.document.slice(from, to)
    }

    /// Returns the text from `from` to the stream's current site.
    #[must_use]
    pub fn text_since(&self, from: Site<'i, P>) -> Option<&'i str> {
        self.text_between(from, self.site())
    }

    // ----- crate-internal primitives of the rule drivers (`grammar`)

    /// Returns the document the stream reads.
    pub(crate) const fn document(&self) -> &'i Document<'i, P> {
        self.document
    }

    /// Returns the index of the next node.
    pub(crate) const fn next_index(&self) -> usize {
        self.next
    }

    /// Returns the index at which the stream ends.
    pub(crate) const fn end_index(&self) -> usize {
        self.end
    }

    /// Returns the boundary before the node at `index`.
    pub(crate) const fn site_at(&self, index: usize) -> Site<'i, P> {
        Site::new(self.document, self.list, index, false)
    }

    /// Moves the next node to `index`, which is within the stream, and drops
    /// the group left unentered.
    pub(crate) fn seek(&mut self, index: usize) {
        debug_assert!(self.start <= index && index <= self.end);
        self.next = index;
        self.pending = None;
    }

    /// Returns a stream over the nodes `from` to `to` of this stream's list.
    pub(crate) fn span(&self, from: usize, to: usize) -> Stream<'i, P> {
        debug_assert!(self.start <= from && from <= to && to <= self.end);
        Stream::bounded(self.document, self.list, from..to)
    }

    /// Examines the node at `index`: one meter step, whatever it finds,
    /// including the end of the stream (`None`).
    ///
    /// # Errors
    ///
    /// Fails with `is_refused()` when the meter refuses the step.
    pub(crate) fn peek_at(
        &self,
        index: usize,
    ) -> Result<Option<Component<'i, P>>, StreamError<'i, P>> {
        self.step()?;
        Ok(self.component_unmetered(index))
    }

    /// The node at `index` as a component, or `None` at or past the end of the
    /// stream, without a meter step: for a node that was already examined.
    pub(crate) fn component_unmetered(&self, index: usize) -> Option<Component<'i, P>> {
        if index >= self.end {
            return None;
        }
        let document: &'i Document<'i, P> = self.document;
        Some(document.component(self.node_at(index)))
    }

    /// Appends `value` to `values`, charging the growth of `values` to the
    /// meter if there is one. A refusal is latched and reported, and `value` is
    /// not appended: a result list never silently stops growing.
    ///
    /// # Errors
    ///
    /// Fails with `is_refused()` when the meter refuses.
    pub(crate) fn push_charged<T>(
        &self,
        values: &mut Vec<T>,
        value: T,
    ) -> Result<(), StreamError<'i, P>> {
        if let Some(meter) = self.meter() {
            let document: &'i Document<'i, P> = self.document;
            if document.is_refused() || limits::grow(meter, values, 1).is_err() {
                document.latch_refusal();
                return Err(StreamError::refused(self.site()));
            }
        }
        values.push(value);
        Ok(())
    }

    // ----- errors

    /// Returns the error for a stream that is empty where the grammar needs a
    /// token, at the current site.
    #[must_use]
    pub fn end_of_stream<E>(&self) -> SyntaxError<'i, E, P> {
        self.site().end_of_stream()
    }

    /// Returns the error for a token the grammar did not expect, at the current
    /// site.
    #[must_use]
    pub fn unexpected<E>(&self, token: Token<'i>) -> SyntaxError<'i, E, P> {
        self.site().unexpected(token)
    }

    /// Returns the grammar's own error `error`, converted into the grammar's
    /// error type `F`, at the current site; see [`Site::grammar_error`].
    #[must_use]
    pub fn grammar_error<E, F: From<E>>(&self, error: E) -> SyntaxError<'i, F, P> {
        self.site().grammar_error(error)
    }
}
