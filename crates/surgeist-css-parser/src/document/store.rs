//! How a document stores its nodes and lists.
//!
//! A node keeps its token as a [`Stored`], whose texts are either borrowed
//! from the text the document was built over or from the document's decode
//! pool. A [`Stored`] resolves to a [`Token`] on demand, borrowing the
//! document, so a token stays `Copy` and nothing is decoded twice.

use std::ops::Range;

use super::component::{GroupEnd, GroupKind};
use super::{ListId, NodeId};
use crate::Offset;
use crate::Token;
use crate::token::{HashKind, Number, NumberKind};

/// A text of a stored token.
#[derive(Clone, Copy, Debug)]
pub(super) enum Text<'s> {
    /// Text that outlives the document's own storage: a slice of the source or
    /// of a string a builder was given.
    Borrowed(&'s str),
    /// Text in the document's decode pool.
    Pooled { start: usize, end: usize },
}

impl<'s> Text<'s> {
    fn resolve<'a>(self, pool: &'a str) -> &'a str
    where
        's: 'a,
    {
        match self {
            Self::Borrowed(text) => text,
            Self::Pooled { start, end } => pool.get(start..end).unwrap_or(""),
        }
    }
}

/// A token as a node stores it.
///
/// The tokens that carry no text are held as themselves.
#[derive(Clone, Copy, Debug)]
pub(super) enum Stored<'s> {
    Plain(Token<'static>),
    Ident(Text<'s>),
    Function(Text<'s>),
    AtKeyword(Text<'s>),
    Hash(Text<'s>, HashKind),
    String(Text<'s>),
    Url(Text<'s>),
    Number(Text<'s>, NumberKind),
    Percentage(Text<'s>, NumberKind),
    Dimension {
        number: Text<'s>,
        kind: NumberKind,
        unit: Text<'s>,
    },
}

impl<'s> Stored<'s> {
    /// Stores `token`, asking `text` to place each text it carries.
    pub(super) fn of<'a>(token: Token<'a>, text: &mut dyn FnMut(&'a str) -> Text<'s>) -> Self {
        match token {
            Token::Ident(value) => Self::Ident(text(value)),
            Token::Function(value) => Self::Function(text(value)),
            Token::AtKeyword(value) => Self::AtKeyword(text(value)),
            Token::Hash { value, kind } => Self::Hash(text(value), kind),
            Token::String(value) => Self::String(text(value)),
            Token::Url(value) => Self::Url(text(value)),
            Token::Number(number) => Self::Number(text(number.representation()), number.kind()),
            Token::Percentage(number) => {
                Self::Percentage(text(number.representation()), number.kind())
            }
            Token::Dimension { number, unit } => Self::Dimension {
                number: text(number.representation()),
                kind: number.kind(),
                unit: text(unit),
            },
            Token::BadString => Self::Plain(Token::BadString),
            Token::BadUrl => Self::Plain(Token::BadUrl),
            Token::Delim(point) => Self::Plain(Token::Delim(point)),
            Token::UnicodeRange(range) => Self::Plain(Token::UnicodeRange(range)),
            Token::Whitespace => Self::Plain(Token::Whitespace),
            Token::Comment => Self::Plain(Token::Comment),
            Token::Colon => Self::Plain(Token::Colon),
            Token::Semicolon => Self::Plain(Token::Semicolon),
            Token::Comma => Self::Plain(Token::Comma),
            Token::OpenSquare => Self::Plain(Token::OpenSquare),
            Token::OpenParen => Self::Plain(Token::OpenParen),
            Token::OpenCurly => Self::Plain(Token::OpenCurly),
            Token::CloseSquare => Self::Plain(Token::CloseSquare),
            Token::CloseParen => Self::Plain(Token::CloseParen),
            Token::CloseCurly => Self::Plain(Token::CloseCurly),
            Token::Cdo => Self::Plain(Token::Cdo),
            Token::Cdc => Self::Plain(Token::Cdc),
        }
    }

    /// Stores a token whose texts outlive the document.
    pub(super) fn borrowed(token: Token<'s>) -> Self {
        Self::of(token, &mut |value| Text::Borrowed(value))
    }

    /// Resolves the token, reading pooled texts from `pool`.
    pub(super) fn token<'a>(&self, pool: &'a str) -> Token<'a>
    where
        's: 'a,
    {
        match *self {
            Self::Plain(token) => token,
            Self::Ident(text) => Token::Ident(text.resolve(pool)),
            Self::Function(text) => Token::Function(text.resolve(pool)),
            Self::AtKeyword(text) => Token::AtKeyword(text.resolve(pool)),
            Self::Hash(text, kind) => Token::Hash {
                value: text.resolve(pool),
                kind,
            },
            Self::String(text) => Token::String(text.resolve(pool)),
            Self::Url(text) => Token::Url(text.resolve(pool)),
            Self::Number(text, kind) => Token::Number(Number::new(text.resolve(pool), kind)),
            Self::Percentage(text, kind) => {
                Token::Percentage(Number::new(text.resolve(pool), kind))
            }
            Self::Dimension { number, kind, unit } => Token::Dimension {
                number: Number::new(number.resolve(pool), kind),
                unit: unit.resolve(pool),
            },
        }
    }
}

/// What a node is besides its token.
#[derive(Debug)]
pub(super) enum Shape<P> {
    /// A token that owns nothing.
    Leaf,
    /// A function or a simple block.
    Group {
        kind: GroupKind,
        children: ListId,
        end: GroupEnd<P>,
        /// Where the opening token ends, when its interval is known.
        opening_end: Option<Offset>,
        /// Whether the group has ended; nothing is added to it afterwards.
        closed: bool,
    },
    /// A `{}`-block denied for nesting too deep.
    Denied,
}

/// One node of a document.
#[derive(Debug)]
pub(super) struct Node<'s, P> {
    pub(super) token: Stored<'s>,
    pub(super) shape: Shape<P>,
    /// The interval of the token, or of the whole group once it is closed.
    pub(super) range: Option<Range<Offset>>,
    pub(super) payload: Option<P>,
    /// The list that holds the node, and the node's place in it.
    pub(super) list: ListId,
    pub(super) index: usize,
}

impl<P> Node<'_, P> {
    pub(super) const fn children(&self) -> Option<ListId> {
        match self.shape {
            Shape::Group { children, .. } => Some(children),
            Shape::Leaf | Shape::Denied => None,
        }
    }

    pub(super) const fn group_end(&self) -> Option<&GroupEnd<P>> {
        match &self.shape {
            Shape::Group { end, .. } => Some(end),
            Shape::Leaf | Shape::Denied => None,
        }
    }
}

/// One list of a document.
#[derive(Debug)]
pub(super) struct List {
    pub(super) nodes: Vec<NodeId>,
    /// The group that owns the list; none for the root.
    pub(super) parent: Option<NodeId>,
    /// Where the list ends in the text: its closing token, or the end of the
    /// text.
    pub(super) end: Option<Offset>,
    /// How many groups are open around the list, not counting the base depth
    /// a build started with.
    pub(super) depth: u32,
}

#[cfg(test)]
mod tests {
    use std::mem::{align_of, size_of};

    use super::*;

    fn pinned<T>(name: &str, size: usize, align: usize) {
        assert_eq!(size_of::<T>(), size, "size of {name}");
        assert_eq!(align_of::<T>(), align, "alignment of {name}");
    }

    /// The storage of one node is what a metered build charges per element of
    /// its node vector, and what a document costs per token, so it is pinned:
    /// a change is a decision about receipts and memory, not an accident. The
    /// pins describe 64-bit targets. These types are private, so the pins live
    /// here; the measured 168 bytes per node (against the 24 to 32 the design
    /// estimated) are recorded, not yet optimized; `tests/layout_pins.rs` pins the public handles.
    #[test]
    #[cfg(target_pointer_width = "64")]
    fn the_storage_of_a_node_keeps_its_measured_layout() {
        pinned::<Text<'static>>("Text", 24, 8);
        pinned::<Stored<'static>>("Stored", 56, 8);
        pinned::<Shape<()>>("Shape<()>", 64, 8);
        pinned::<Node<'static, ()>>("Node<()>", 168, 8);
        pinned::<Node<'static, u64>>("Node<u64>", 184, 8);
        pinned::<List>("List", 64, 8);
    }
}
