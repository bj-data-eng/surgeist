//! A boundary in a document.

use std::fmt;
use std::mem::size_of;
use std::ptr;

use super::{StreamError, SyntaxError};
use crate::document::{GroupEnd, ListId, NodeId};
use crate::{Document, Offset, Token};

/// A real boundary in a document: a list, a place in it, and whether it is the
/// closing boundary of the group at that place.
///
/// A site sits before a token (or at the end of its list) in the token stream,
/// or, when [`is_closing`](Self::is_closing), at the closing token of the group
/// whose node is at the place. It exists for documents built from text and
/// from components alike; only the former always have an
/// [`offset`](Self::offset).
///
/// `Clone`, `Copy`, `Debug`, `PartialEq` and `Eq` are written by hand and ask
/// nothing of the payload type `P`: two sites are equal when they are in the
/// same document (the same value, not an equal one) at the same list, place and
/// closing flag, and `Debug` prints the place, never the document.
///
/// ```
/// use surgeist_css_parser::lexer::Options;
/// use surgeist_css_parser::limits::{DepthExcess, Limits};
/// use surgeist_css_parser::Document;
///
/// let document: Document =
///     Document::parse("a b", &Options::default(), Limits::default(), DepthExcess::Fail)?;
/// let mut stream = document.root();
/// let before = stream.site();
/// stream.consume_ident().unwrap();
/// assert_ne!(stream.site(), before);
/// assert_eq!(before.token().map(|token| token.kind()), Some(surgeist_css_parser::token::TokenKind::Ident));
/// assert_eq!(before.spelling(), Some("a"));
/// # Ok::<(), surgeist_css_parser::document::BuildError>(())
/// ```
pub struct Site<'i, P = ()> {
    document: &'i Document<'i, P>,
    list: ListId,
    index: usize,
    closing: bool,
}

impl<P> Clone for Site<'_, P> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<P> Copy for Site<'_, P> {}

impl<P> fmt::Debug for Site<'_, P> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Site")
            .field("list", &self.list)
            .field("index", &self.index)
            .field("closing", &self.closing)
            .finish()
    }
}

impl<P> PartialEq for Site<'_, P> {
    fn eq(&self, other: &Self) -> bool {
        ptr::eq(self.document, other.document)
            && self.list == other.list
            && self.index == other.index
            && self.closing == other.closing
    }
}

impl<P> Eq for Site<'_, P> {}

impl<'i, P> Site<'i, P> {
    pub(crate) const fn new(
        document: &'i Document<'i, P>,
        list: ListId,
        index: usize,
        closing: bool,
    ) -> Self {
        Self {
            document,
            list,
            index,
            closing,
        }
    }

    /// Returns the document the site is in.
    #[must_use]
    pub const fn document(self) -> &'i Document<'i, P> {
        self.document
    }

    /// Returns the list the site is in.
    #[must_use]
    pub const fn list(self) -> ListId {
        self.list
    }

    /// Returns the place of the site in its list: the number of nodes before
    /// it.
    #[must_use]
    pub const fn index(self) -> usize {
        self.index
    }

    /// Returns whether the site is the closing boundary of the group at its
    /// place, rather than the boundary before that node.
    #[must_use]
    pub const fn is_closing(self) -> bool {
        self.closing
    }

    /// Returns the node at the site: the node that follows the boundary, or the
    /// group whose closing boundary this is. `None` at the end of a list.
    #[must_use]
    pub fn node(self) -> Option<NodeId> {
        self.document.list(self.list).get(self.index).copied()
    }

    /// Returns the payload attached to the token at the site: to the closing
    /// token of a closing site, to the node's own token otherwise.
    #[must_use]
    pub fn payload(self) -> Option<&'i P> {
        let document = self.document;
        let node = self.node()?;
        if self.closing {
            match document.group_end(node)? {
                GroupEnd::Closed { payload, .. } => payload.as_ref(),
                GroupEnd::Eof { .. } => None,
            }
        } else {
            document.payload(node)
        }
    }

    /// Returns where the site is in the text: the start of the token it sits
    /// before, of the closing token of a closing site, or the end of its list.
    /// `None` when the document does not know, as for components built without
    /// intervals.
    #[must_use]
    pub fn offset(self) -> Option<Offset> {
        let document = self.document;
        let Some(node) = self.node() else {
            return document.list_end(self.list);
        };
        if self.closing {
            return match document.group_end(node)? {
                GroupEnd::Closed { range, .. } => range.as_ref().map(|range| range.start),
                GroupEnd::Eof { at } => *at,
            };
        }
        document.range(node).map(|range| range.start)
    }

    /// Returns the token the site sits before, or for a closing site the group's
    /// closing token. `None` at the end of a list, and for a closing site of a
    /// group the input ended instead of closing.
    #[must_use]
    pub fn token(self) -> Option<Token<'i>> {
        let document = self.document;
        let node = self.node()?;
        if self.closing {
            return match document.group_end(node)? {
                GroupEnd::Closed { .. } => Some(document.group_kind(node)?.closing_token()),
                GroupEnd::Eof { .. } => None,
            };
        }
        Some(document.token(node))
    }

    /// Returns the text the token at the site was written as, which can differ
    /// from its decoded value (an escaped name, a quoted string). For a group it
    /// is the opening token only. `None` when the document has no text or the
    /// token has no interval.
    #[must_use]
    pub fn spelling(self) -> Option<&'i str> {
        let document = self.document;
        let node = self.node()?;
        if self.closing {
            return match document.group_end(node)? {
                GroupEnd::Closed {
                    range: Some(range), ..
                } => document.slice(range.start, range.end),
                GroupEnd::Closed { range: None, .. } | GroupEnd::Eof { .. } => None,
            };
        }
        document.spelling(node)
    }

    // ----- errors at the site

    /// Returns the error for a token the grammar did not expect at this site.
    ///
    /// [`Stream::unexpected`](crate::Stream::unexpected) is this at the
    /// stream's current site; a grammar that reports a failure at a boundary it
    /// passed earlier asks the site.
    #[must_use]
    pub fn unexpected<E>(self, token: Token<'i>) -> SyntaxError<'i, E, P> {
        StreamError::unexpected(self, token).into()
    }

    /// Returns the error for a stream that is empty at this site where the
    /// grammar needs a token.
    #[must_use]
    pub fn end_of_stream<E>(self) -> SyntaxError<'i, E, P> {
        StreamError::end_of_stream(self).into()
    }

    /// Returns the grammar's own error `error`, converted into the grammar's
    /// error type `F`, at this site.
    ///
    /// When the document is metered, the boxed error is charged to the meter
    /// as the size of `F`, and a refusal (a latched one included) becomes an
    /// error for which `is_refused()` holds instead. The charge is not
    /// returned: an error is a result the host reports and then discards.
    #[must_use]
    pub fn grammar_error<E, F: From<E>>(self, error: E) -> SyntaxError<'i, F, P> {
        if !self.document.admit_add(size_of::<F>()) {
            return StreamError::refused(self).into();
        }
        SyntaxError::custom(self, F::from(error))
    }
}
