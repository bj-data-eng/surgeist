//! How a document stores its nodes and lists.
//!
//! A node keeps its token as a [`Stored`], whose texts are either borrowed
//! from the text the document was built over or from the document's decode
//! pool. A [`Stored`] resolves to a [`Token`] on demand, borrowing the
//! document, so a token stays `Copy` and nothing is decoded twice.
//!
//! A node holds what every node has and nothing else. What only a function or
//! a simple block has (its kind, how it ended, where its opening token ends)
//! lives in the [`List`] the group owns, which exists once per group, so a leaf
//! pays nothing for it.

use std::ops::Range;

use super::component::{GroupEnd, GroupKind};
use super::{ListId, NodeId};
use crate::Offset;
use crate::Token;
use crate::token::{HashKind, Number, NumberKind};
use crate::unicode_range::CodePointRange;

/// A text of a stored token.
#[derive(Clone, Copy, Debug)]
pub(super) enum Text<'s> {
    /// Text that outlives the document's own storage: a slice of the source or
    /// of a string a builder was given.
    Borrowed(&'s str),
    /// Text in the document's decode pool, as byte positions in it.
    Pooled { start: u32, end: u32 },
}

impl<'s> Text<'s> {
    /// The text that `length` bytes make at the end of a pool that holds
    /// `pooled` bytes, or `None` when the pool would then hold more than
    /// `u32::MAX` bytes, the greatest position a pooled text can address.
    pub(super) fn pooled(pooled: usize, length: usize) -> Option<Self> {
        let start = u32::try_from(pooled).ok()?;
        let end = u32::try_from(pooled.checked_add(length)?).ok()?;
        Some(Self::Pooled { start, end })
    }

    fn resolve<'a>(self, pool: &'a str) -> &'a str
    where
        's: 'a,
    {
        match self {
            Self::Borrowed(text) => text,
            Self::Pooled { start, end } => pool.get(start as usize..end as usize).unwrap_or(""),
        }
    }
}

/// A token that carries no text, as a node stores it.
///
/// It mirrors the variants of [`Token`] that hold nothing but, at most, a code
/// point or a range of code points, in the least room.
#[derive(Clone, Copy, Debug)]
pub(super) enum Bare {
    BadString,
    BadUrl,
    Delim(char),
    UnicodeRange(CodePointRange),
    Whitespace,
    Comment,
    Colon,
    Semicolon,
    Comma,
    OpenSquare,
    OpenParen,
    OpenCurly,
    CloseSquare,
    CloseParen,
    CloseCurly,
    Cdo,
    Cdc,
}

impl Bare {
    const fn token(self) -> Token<'static> {
        match self {
            Self::BadString => Token::BadString,
            Self::BadUrl => Token::BadUrl,
            Self::Delim(point) => Token::Delim(point),
            Self::UnicodeRange(range) => Token::UnicodeRange(range),
            Self::Whitespace => Token::Whitespace,
            Self::Comment => Token::Comment,
            Self::Colon => Token::Colon,
            Self::Semicolon => Token::Semicolon,
            Self::Comma => Token::Comma,
            Self::OpenSquare => Token::OpenSquare,
            Self::OpenParen => Token::OpenParen,
            Self::OpenCurly => Token::OpenCurly,
            Self::CloseSquare => Token::CloseSquare,
            Self::CloseParen => Token::CloseParen,
            Self::CloseCurly => Token::CloseCurly,
            Self::Cdo => Token::Cdo,
            Self::Cdc => Token::Cdc,
        }
    }
}

/// A token as a node stores it.
///
/// The tokens that carry no text are held in the room of a [`Bare`].
#[derive(Clone, Copy, Debug)]
pub(super) enum Stored<'s> {
    Bare(Bare),
    /// A `{}`-block denied for nesting too deep. Its token is the `<{-token>`
    /// that opened it.
    Denied,
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
    /// Stores `token`, asking `place` to place each text it carries.
    ///
    /// # Errors
    ///
    /// Returns the first error `place` returns.
    pub(super) fn of<'a, E>(
        token: Token<'a>,
        mut place: impl FnMut(&'a str) -> Result<Text<'s>, E>,
    ) -> Result<Self, E> {
        Ok(match token {
            Token::Ident(value) => Self::Ident(place(value)?),
            Token::Function(value) => Self::Function(place(value)?),
            Token::AtKeyword(value) => Self::AtKeyword(place(value)?),
            Token::Hash { value, kind } => Self::Hash(place(value)?, kind),
            Token::String(value) => Self::String(place(value)?),
            Token::Url(value) => Self::Url(place(value)?),
            Token::Number(number) => Self::Number(place(number.representation())?, number.kind()),
            Token::Percentage(number) => {
                Self::Percentage(place(number.representation())?, number.kind())
            }
            Token::Dimension { number, unit } => Self::Dimension {
                number: place(number.representation())?,
                kind: number.kind(),
                unit: place(unit)?,
            },
            Token::BadString => Self::Bare(Bare::BadString),
            Token::BadUrl => Self::Bare(Bare::BadUrl),
            Token::Delim(point) => Self::Bare(Bare::Delim(point)),
            Token::UnicodeRange(range) => Self::Bare(Bare::UnicodeRange(range)),
            Token::Whitespace => Self::Bare(Bare::Whitespace),
            Token::Comment => Self::Bare(Bare::Comment),
            Token::Colon => Self::Bare(Bare::Colon),
            Token::Semicolon => Self::Bare(Bare::Semicolon),
            Token::Comma => Self::Bare(Bare::Comma),
            Token::OpenSquare => Self::Bare(Bare::OpenSquare),
            Token::OpenParen => Self::Bare(Bare::OpenParen),
            Token::OpenCurly => Self::Bare(Bare::OpenCurly),
            Token::CloseSquare => Self::Bare(Bare::CloseSquare),
            Token::CloseParen => Self::Bare(Bare::CloseParen),
            Token::CloseCurly => Self::Bare(Bare::CloseCurly),
            Token::Cdo => Self::Bare(Bare::Cdo),
            Token::Cdc => Self::Bare(Bare::Cdc),
        })
    }

    /// Stores a token whose texts outlive the document.
    pub(super) fn borrowed(token: Token<'s>) -> Self {
        match Self::of(token, |value| {
            Ok::<_, std::convert::Infallible>(Text::Borrowed(value))
        }) {
            Ok(stored) => stored,
        }
    }

    /// Whether this is the token of a denied block.
    pub(super) const fn is_denied(&self) -> bool {
        matches!(self, Self::Denied)
    }

    /// Resolves the token, reading pooled texts from `pool`.
    pub(super) fn token<'a>(&self, pool: &'a str) -> Token<'a>
    where
        's: 'a,
    {
        match *self {
            Self::Bare(bare) => bare.token(),
            Self::Denied => Token::OpenCurly,
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

/// One node of a document.
///
/// A node that owns a list (a function or a simple block) names it in
/// `children`; everything about the group itself is in that [`List`].
#[derive(Debug)]
pub(super) struct Node<'s, P> {
    pub(super) token: Stored<'s>,
    /// Where the token starts, when `ranged`.
    start: Offset,
    /// Where the token ends, or the whole group once it is closed, when
    /// `ranged`.
    end: Offset,
    ranged: bool,
    /// The list that holds the node, and the node's place in it.
    pub(super) list: ListId,
    pub(super) index: u32,
    /// The list the node owns, for a function or a simple block.
    pub(super) children: Option<ListId>,
    pub(super) payload: Option<P>,
}

impl<'s, P> Node<'s, P> {
    pub(super) fn new(
        token: Stored<'s>,
        range: Option<Range<Offset>>,
        place: (ListId, u32),
        children: Option<ListId>,
        payload: Option<P>,
    ) -> Self {
        let ranged = range.is_some();
        let (start, end) = range.map_or((Offset::new(0), Offset::new(0)), |range| {
            (range.start, range.end)
        });
        Self {
            token,
            start,
            end,
            ranged,
            list: place.0,
            index: place.1,
            children,
            payload,
        }
    }

    /// The interval of the token, or of the whole group once it is closed.
    pub(super) fn range(&self) -> Option<Range<Offset>> {
        self.ranged.then_some(self.start..self.end)
    }

    /// Where the token starts.
    pub(super) const fn start(&self) -> Option<Offset> {
        if self.ranged { Some(self.start) } else { None }
    }

    /// Moves the end of the interval, which a node without one does not have.
    pub(super) const fn set_end(&mut self, end: Offset) {
        if self.ranged {
            self.end = end;
        }
    }
}

/// What a list is besides its nodes.
#[derive(Debug)]
pub(super) enum Owner<P> {
    /// The root list.
    Root {
        /// Where the text ends, when it is known.
        end: Option<Offset>,
    },
    /// The contents of a function or a simple block.
    Group(Group<P>),
}

/// The function or simple block that owns a list.
#[derive(Debug)]
pub(super) struct Group<P> {
    pub(super) node: NodeId,
    pub(super) kind: GroupKind,
    /// How the group ended; until it does, the end of the input at an unknown
    /// place.
    pub(super) end: GroupEnd<P>,
    /// Where the opening token ends, when its interval is known.
    pub(super) opening_end: Option<Offset>,
    /// Whether the group has ended; nothing is added to it afterwards.
    pub(super) closed: bool,
}

/// One list of a document.
#[derive(Debug)]
pub(super) struct List<P> {
    pub(super) nodes: Vec<NodeId>,
    /// How many groups are open around the list, not counting the base depth
    /// a build started with.
    pub(super) depth: u32,
    pub(super) owner: Owner<P>,
}

impl<P> List<P> {
    pub(super) const fn root() -> Self {
        Self {
            nodes: Vec::new(),
            depth: 0,
            owner: Owner::Root { end: None },
        }
    }

    pub(super) const fn of_group(group: Group<P>, depth: u32) -> Self {
        Self {
            nodes: Vec::new(),
            depth,
            owner: Owner::Group(group),
        }
    }

    /// The group that owns the list; none for the root.
    pub(super) const fn group(&self) -> Option<&Group<P>> {
        match &self.owner {
            Owner::Group(group) => Some(group),
            Owner::Root { .. } => None,
        }
    }

    pub(super) const fn group_mut(&mut self) -> Option<&mut Group<P>> {
        match &mut self.owner {
            Owner::Group(group) => Some(group),
            Owner::Root { .. } => None,
        }
    }

    pub(super) fn kind(&self) -> Option<GroupKind> {
        self.group().map(|group| group.kind)
    }

    pub(super) fn parent(&self) -> Option<NodeId> {
        self.group().map(|group| group.node)
    }

    /// Where the list ends in the text: its closing token, or the end of the
    /// text.
    pub(super) fn end(&self) -> Option<Offset> {
        match &self.owner {
            Owner::Root { end } => *end,
            Owner::Group(group) => match &group.end {
                GroupEnd::Closed { range, .. } => range.as_ref().map(|range| range.start),
                GroupEnd::Eof { at } => *at,
            },
        }
    }

    /// Records where the text of the root list ends.
    pub(super) const fn set_root_end(&mut self, at: Option<Offset>) {
        if let Owner::Root { end } = &mut self.owner {
            *end = at;
        }
    }
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
    /// here; `tests/layout_pins.rs` pins the public handles.
    #[test]
    #[cfg(target_pointer_width = "64")]
    fn the_storage_of_a_node_keeps_its_measured_layout() {
        pinned::<Text<'static>>("Text", 16, 8);
        pinned::<Bare>("Bare", 12, 4);
        pinned::<Stored<'static>>("Stored", 40, 8);
        pinned::<Node<'static, ()>>("Node<()>", 72, 8);
        pinned::<Node<'static, u64>>("Node<u64>", 88, 8);
        pinned::<Option<ListId>>("Option<ListId>", 4, 4);
        pinned::<List<()>>("List<()>", 88, 8);
    }

    #[test]
    fn a_pooled_text_is_refused_when_the_pool_outgrows_its_positions() {
        assert!(matches!(
            Text::pooled(0, 3),
            Some(Text::Pooled { start: 0, end: 3 })
        ));
        assert!(Text::pooled(u32::MAX as usize, 1).is_none());
        assert!(Text::pooled(usize::MAX, 1).is_none());
        assert!(Text::pooled(1, usize::MAX).is_none());
        assert!(matches!(
            Text::pooled(u32::MAX as usize - 1, 1),
            Some(Text::Pooled { .. })
        ));
    }
}
