//! The component values of a document, the kinds of their groups, and how a
//! group ends.

use std::ops::Range;

use super::{ListId, NodeId};
use crate::{Offset, Token};

/// The kind of a simple block (section 5.2): the token that opens it names
/// it, and its closing token is the matching one.
///
/// The set is closed: it mirrors the three simple blocks of the pinned
/// edition.
///
/// ```
/// use surgeist_css_parser::Document;
/// use surgeist_css_parser::document::{BlockKind, GroupKind};
/// use surgeist_css_parser::lexer::Options;
/// use surgeist_css_parser::limits::{DepthExcess, Limits};
///
/// let document: Document =
///     Document::parse("[a]", &Options::default(), Limits::default(), DepthExcess::Fail)?;
/// let node = document.list(document.root_list())[0];
/// assert_eq!(document.group_kind(node), Some(GroupKind::Block(BlockKind::Square)));
/// # Ok::<(), surgeist_css_parser::document::BuildError>(())
/// ```
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum BlockKind {
    /// A `()`-block, opened by a `<(-token>`.
    Paren,
    /// A `[]`-block, opened by a `<[-token>`.
    Square,
    /// A `{}`-block, opened by a `<{-token>`.
    Curly,
}

/// The kind of a component value that owns a list of component values: a
/// function or a simple block (section 5.2).
///
/// The set is closed: it mirrors the two kinds of the pinned edition.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum GroupKind {
    /// A function, opened by a `<function-token>` and closed by a
    /// `<)-token>`.
    Function,
    /// A simple block.
    Block(BlockKind),
}

impl GroupKind {
    /// Returns the kind of group a token opens, or `None` for a token that
    /// opens none.
    #[must_use]
    pub(crate) const fn of_opening(token: &Token<'_>) -> Option<Self> {
        match token {
            Token::Function(_) => Some(Self::Function),
            Token::OpenParen => Some(Self::Block(BlockKind::Paren)),
            Token::OpenSquare => Some(Self::Block(BlockKind::Square)),
            Token::OpenCurly => Some(Self::Block(BlockKind::Curly)),
            _ => None,
        }
    }

    /// Returns the kind of simple block whose closing token `token` is, or
    /// `None` for any other token. A function is closed by a `<)-token>`, which
    /// is also the closing token of a `()`-block.
    #[must_use]
    pub(crate) const fn of_closing(token: &Token<'_>) -> Option<BlockKind> {
        match token {
            Token::CloseParen => Some(BlockKind::Paren),
            Token::CloseSquare => Some(BlockKind::Square),
            Token::CloseCurly => Some(BlockKind::Curly),
            _ => None,
        }
    }

    /// Returns the kind of block this group is closed by.
    #[must_use]
    pub(crate) const fn closing_block(self) -> BlockKind {
        match self {
            Self::Function | Self::Block(BlockKind::Paren) => BlockKind::Paren,
            Self::Block(kind) => kind,
        }
    }

    /// Returns the closing token of this group.
    #[must_use]
    pub(crate) const fn closing_token(self) -> Token<'static> {
        match self.closing_block() {
            BlockKind::Paren => Token::CloseParen,
            BlockKind::Square => Token::CloseSquare,
            BlockKind::Curly => Token::CloseCurly,
        }
    }
}

/// A token the parser keeps although the specification says it is always a
/// parse error (section 5.2).
///
/// Section 5.2 preserves these in the token stream so that other
/// specifications can define finer error handling than dropping a whole
/// declaration or block. [`Component::fault`] reports which one a component
/// value is.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum Fault {
    /// A `<bad-string-token>`.
    BadString,
    /// A `<bad-url-token>`.
    BadUrl,
    /// A `<}-token>`, `<)-token>` or `<]-token>` that closes no open group.
    UnexpectedCloser,
}

/// How a function or simple block ended.
///
/// A source document ends a group either at its closing token or at the end of
/// the text (section 5.5.9 and 5.5.10: end of the input closes a group, which
/// is a parse error). A document built from checked or programmatic
/// components says the same, with positions only when it has them.
///
/// ```
/// use surgeist_css_parser::Document;
/// use surgeist_css_parser::document::GroupEnd;
/// use surgeist_css_parser::lexer::Options;
/// use surgeist_css_parser::limits::{DepthExcess, Limits};
///
/// let document: Document =
///     Document::parse("(a) (b", &Options::default(), Limits::default(), DepthExcess::Fail)?;
/// let nodes = document.list(document.root_list());
/// assert!(matches!(document.group_end(nodes[0]), Some(GroupEnd::Closed { .. })));
/// assert!(matches!(document.group_end(nodes[2]), Some(GroupEnd::Eof { .. })));
/// # Ok::<(), surgeist_css_parser::document::BuildError>(())
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum GroupEnd<P> {
    /// The group ended at its closing token.
    Closed {
        /// The interval of the closing token, when it has one.
        range: Option<Range<Offset>>,
        /// The payload attached to the closing token.
        payload: Option<P>,
    },
    /// The text ended before the closing token, so the group was closed by the
    /// end of the input.
    Eof {
        /// Where the text ended, when it is known.
        at: Option<Offset>,
    },
}

/// A component value of a document (section 5.2): a preserved token, a
/// function, or a simple block.
///
/// A component borrows the document. A function or simple block does not copy
/// its contents: [`children`](Self::children) names the list that holds them,
/// so a consumer that does not enter a group never pays for it.
///
/// The set is not closed. A `{}`-block that nested deeper than the build's
/// [`Limits::max_depth`](crate::limits::Limits::max_depth) allows is kept as a
/// [`Denied`](Self::Denied) value under
/// [`DepthExcess::DenyBlock`](crate::limits::DepthExcess::DenyBlock), which is
/// not a component value of the specification.
///
/// ```
/// use surgeist_css_parser::Document;
/// use surgeist_css_parser::document::Component;
/// use surgeist_css_parser::lexer::Options;
/// use surgeist_css_parser::limits::{DepthExcess, Limits};
///
/// let document: Document =
///     Document::parse("f(a)", &Options::default(), Limits::default(), DepthExcess::Fail)?;
/// let node = document.list(document.root_list())[0];
/// let Component::Function { name, children, .. } = document.component(node) else {
///     panic!("a function");
/// };
/// assert_eq!(name, "f");
/// assert_eq!(document.list(children).len(), 1);
/// # Ok::<(), surgeist_css_parser::document::BuildError>(())
/// ```
#[non_exhaustive]
pub enum Component<'d, P = ()> {
    /// A preserved token: any token but the ones that open a function or a
    /// simple block. That includes the tokens section 5.2 keeps although they
    /// are always parse errors; see [`fault`](Self::fault).
    Token {
        /// The node.
        node: NodeId,
        /// The token.
        token: Token<'d>,
        /// The payload attached to the node.
        payload: Option<&'d P>,
    },
    /// A function.
    Function {
        /// The node.
        node: NodeId,
        /// The decoded name, without the parenthesis.
        name: &'d str,
        /// The list that holds the function's component values.
        children: ListId,
        /// The payload attached to the node.
        payload: Option<&'d P>,
    },
    /// A simple block.
    Block {
        /// The node.
        node: NodeId,
        /// The kind of block.
        kind: BlockKind,
        /// The list that holds the block's component values.
        children: ListId,
        /// The payload attached to the node.
        payload: Option<&'d P>,
    },
    /// A `{}`-block denied because it nested too deep. It has no children.
    Denied {
        /// The node.
        node: NodeId,
        /// The interval of the whole block in the text, when known.
        range: Option<Range<Offset>>,
        /// The payload attached to the node.
        payload: Option<&'d P>,
    },
}

impl<P> std::fmt::Debug for Component<'_, P> {
    /// Prints the component without its payload, which needs no `Debug`; whether
    /// there is one is shown.
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Token {
                node,
                token,
                payload,
            } => formatter
                .debug_struct("Token")
                .field("node", node)
                .field("token", token)
                .field("has_payload", &payload.is_some())
                .finish(),
            Self::Function {
                node,
                name,
                children,
                payload,
            } => formatter
                .debug_struct("Function")
                .field("node", node)
                .field("name", name)
                .field("children", children)
                .field("has_payload", &payload.is_some())
                .finish(),
            Self::Block {
                node,
                kind,
                children,
                payload,
            } => formatter
                .debug_struct("Block")
                .field("node", node)
                .field("kind", kind)
                .field("children", children)
                .field("has_payload", &payload.is_some())
                .finish(),
            Self::Denied {
                node,
                range,
                payload,
            } => formatter
                .debug_struct("Denied")
                .field("node", node)
                .field("range", range)
                .field("has_payload", &payload.is_some())
                .finish(),
        }
    }
}

impl<P> Clone for Component<'_, P> {
    fn clone(&self) -> Self {
        match self {
            Self::Token {
                node,
                token,
                payload,
            } => Self::Token {
                node: *node,
                token: *token,
                payload: *payload,
            },
            Self::Function {
                node,
                name,
                children,
                payload,
            } => Self::Function {
                node: *node,
                name,
                children: *children,
                payload: *payload,
            },
            Self::Block {
                node,
                kind,
                children,
                payload,
            } => Self::Block {
                node: *node,
                kind: *kind,
                children: *children,
                payload: *payload,
            },
            Self::Denied {
                node,
                range,
                payload,
            } => Self::Denied {
                node: *node,
                range: range.clone(),
                payload: *payload,
            },
        }
    }
}

impl<'d, P> Component<'d, P> {
    /// Returns the node of the component.
    #[must_use]
    pub const fn node(&self) -> NodeId {
        match self {
            Self::Token { node, .. }
            | Self::Function { node, .. }
            | Self::Block { node, .. }
            | Self::Denied { node, .. } => *node,
        }
    }

    /// Returns the payload attached to the component's node.
    #[must_use]
    pub const fn payload(&self) -> Option<&'d P> {
        match self {
            Self::Token { payload, .. }
            | Self::Function { payload, .. }
            | Self::Block { payload, .. }
            | Self::Denied { payload, .. } => *payload,
        }
    }

    /// Returns the list that holds the component's own component values, for a
    /// function or a simple block.
    #[must_use]
    pub const fn children(&self) -> Option<ListId> {
        match self {
            Self::Function { children, .. } | Self::Block { children, .. } => Some(*children),
            Self::Token { .. } | Self::Denied { .. } => None,
        }
    }

    /// Returns which kept parse error the component is, if it is one: a bad
    /// string, a bad url, or a closing token that closes nothing.
    ///
    /// The answer comes from the token's kind; nothing is stored for it.
    ///
    /// ```
    /// use surgeist_css_parser::Document;
    /// use surgeist_css_parser::document::Fault;
    /// use surgeist_css_parser::lexer::Options;
    /// use surgeist_css_parser::limits::{DepthExcess, Limits};
    ///
    /// let document: Document =
    ///     Document::parse("} a", &Options::default(), Limits::default(), DepthExcess::Fail)?;
    /// let first = document.list(document.root_list())[0];
    /// assert_eq!(document.component(first).fault(), Some(Fault::UnexpectedCloser));
    /// let second = document.list(document.root_list())[1];
    /// assert_eq!(document.component(second).fault(), None);
    /// # Ok::<(), surgeist_css_parser::document::BuildError>(())
    /// ```
    #[must_use]
    pub const fn fault(&self) -> Option<Fault> {
        match self {
            Self::Token { token, .. } => match token {
                Token::BadString => Some(Fault::BadString),
                Token::BadUrl => Some(Fault::BadUrl),
                Token::CloseParen | Token::CloseSquare | Token::CloseCurly => {
                    Some(Fault::UnexpectedCloser)
                }
                _ => None,
            },
            _ => None,
        }
    }
}
