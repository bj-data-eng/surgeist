//! The document: every token of a text, in a tree of lists.

use std::cell::Cell;
use std::fmt;
use std::mem::size_of;
use std::ops::Range;

use super::component::{Component, GroupEnd, GroupKind};
use super::store::{List, Node, Shape};
use super::{ListId, Metrics, NodeId};
use crate::lexer::LexicalError;
use crate::limits::Meter;
use crate::{Offset, Site, Stream, Token};

/// A token tree: every token of a text, or of a list of components, as the
/// parsing results of CSS Syntax Module Level 3 section 5.2 describe them.
///
/// A function or simple block is one node in its parent's list that owns a
/// list of its own, so skipping a group, which section 5.5.8 does whenever a
/// grammar does not look inside it, costs nothing. The document is built once,
/// iteratively, by [`parse`](Self::parse) over text or by a
/// [`TreeBuilder`](super::TreeBuilder) over components a caller already holds;
/// afterwards it is read-only, and [`Stream`] walks it.
///
/// `P` is the payload each node may carry: data a consumer attaches to the
/// nodes it builds with a [`TreeBuilder`](super::TreeBuilder), for example the
/// component a checked token came from. A document built from text carries
/// none, and the default `()` means no consumer needs any. Nothing is asked of
/// `P`.
///
/// A [`NodeId`] or [`ListId`] is only meaningful for the document that issued
/// it. A method that takes one panics when given another document's.
///
/// ```
/// use surgeist_css_parser::lexer::Options;
/// use surgeist_css_parser::limits::{DepthExcess, Limits};
/// use surgeist_css_parser::{Document, SyntaxError};
///
/// let document: Document =
///     Document::parse("a f(b)", &Options::default(), Limits::default(), DepthExcess::Fail)?;
/// let mut stream = document.root();
/// assert_eq!(stream.consume_ident().ok(), Some("a"));
/// assert_eq!(stream.consume_function_name().ok(), Some("f"));
/// let inner = stream
///     .within_block(|inner| Ok::<_, SyntaxError<'_, ()>>(inner.consume_ident()?))
///     .unwrap();
/// assert_eq!(inner, "b");
/// assert!(stream.next_token().is_none());
/// # Ok::<(), surgeist_css_parser::document::BuildError>(())
/// ```
pub struct Document<'s, P = ()> {
    pub(super) source: Option<&'s str>,
    /// The offset at which the source begins in a larger text.
    pub(super) origin: usize,
    pub(super) nodes: Vec<Node<'s, P>>,
    pub(super) lists: Vec<List>,
    /// The decoded text of names and strings that differ from their spelling.
    pub(super) pool: String,
    pub(super) errors: Vec<LexicalError>,
    pub(super) metrics: Metrics,
    pub(super) meter: Option<&'s dyn Meter>,
    /// Whether every node has an interval and the intervals start in node
    /// order, which is what [`node_at_offset`](Self::node_at_offset) needs.
    pub(super) ordered: bool,
    /// Set by the first refusal of the meter through a stream; see
    /// [`is_refused`](Self::is_refused).
    pub(super) refused: Cell<bool>,
}

impl<P> fmt::Debug for Document<'_, P> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Document")
            .field("nodes", &self.nodes.len())
            .field("lists", &self.lists.len())
            .field("has_source", &self.source.is_some())
            .field("metered", &self.meter.is_some())
            .field("metrics", &self.metrics)
            .finish()
    }
}

impl<P> Drop for Document<'_, P> {
    /// Returns the storage a metered build was charged for.
    fn drop(&mut self) {
        let Some(meter) = self.meter else { return };
        let mut bytes = self
            .nodes
            .capacity()
            .saturating_mul(size_of::<Node<'_, P>>());
        bytes = bytes.saturating_add(self.lists.capacity().saturating_mul(size_of::<List>()));
        for list in &self.lists {
            bytes = bytes.saturating_add(list.nodes.capacity().saturating_mul(size_of::<NodeId>()));
        }
        meter.released(bytes);
    }
}

impl<'s, P> Document<'s, P> {
    // ----- reading

    /// Returns a stream over the root list, from its first node.
    #[must_use]
    pub fn root(&self) -> Stream<'_, P> {
        self.stream(ListId::ROOT)
    }

    /// Returns a stream over all of `list`, from its first node.
    #[must_use]
    pub fn stream(&self, list: ListId) -> Stream<'_, P> {
        let len = self.lists[list.index()].nodes.len();
        Stream::bounded(self, list, 0..len)
    }

    /// Returns the root list: the component values of the whole text.
    #[must_use]
    pub const fn root_list(&self) -> ListId {
        ListId::ROOT
    }

    /// Returns the text the document was built over, or `None` for a document
    /// built from components.
    #[must_use]
    pub const fn source(&self) -> Option<&'s str> {
        self.source
    }

    /// Returns the meter the document was built with, if any. A stream over the
    /// document admits its observations through it.
    #[must_use]
    pub fn meter(&self) -> Option<&dyn Meter> {
        self.meter
    }

    /// Returns whether the meter has refused an action asked of it through a
    /// stream over this document.
    ///
    /// The first refusal latches: from then on every metered observation of
    /// every stream over the document fails without asking the meter again,
    /// so a host need not keep refusing and a grammar sees one consistent
    /// answer. Methods that cannot return an error ([`Stream::next_token`],
    /// [`Stream::is_empty`], [`Stream::only_whitespace_remains`],
    /// [`Stream::discard_whitespace`]) fall back to behaving as at the end of
    /// the stream; this method, and [`Stream::is_refused`], tell the two apart.
    #[must_use]
    pub fn is_refused(&self) -> bool {
        self.refused.get()
    }

    /// Returns the source text of `node`'s own token: for a function or block,
    /// its opening token only. `None` when the document has no source text or
    /// the node has no interval.
    ///
    /// ```
    /// use surgeist_css_parser::lexer::Options;
    /// use surgeist_css_parser::limits::{DepthExcess, Limits};
    /// use surgeist_css_parser::Document;
    ///
    /// let document: Document =
    ///     Document::parse("\\66 (a)", &Options::default(), Limits::default(), DepthExcess::Fail)?;
    /// let node = document.list(document.root_list())[0];
    /// // The function's name is `f`, but its spelling is the escape.
    /// assert_eq!(document.spelling(node), Some("\\66 ("));
    /// # Ok::<(), surgeist_css_parser::document::BuildError>(())
    /// ```
    #[must_use]
    pub fn spelling(&self, node: NodeId) -> Option<&str> {
        let node = self.node(node);
        let range = node.range.as_ref()?;
        let end = match &node.shape {
            Shape::Group { opening_end, .. } => (*opening_end)?,
            Shape::Leaf | Shape::Denied => range.end,
        };
        self.slice(range.start, end)
    }

    /// Returns the interval of `node` in the text: for a function or block
    /// that has ended, through its closing token or the end of the text.
    #[must_use]
    pub fn range(&self, node: NodeId) -> Option<Range<Offset>> {
        self.node(node).range.clone()
    }

    /// Returns the payload attached to `node`.
    #[must_use]
    pub fn payload(&self, node: NodeId) -> Option<&P> {
        self.node(node).payload.as_ref()
    }

    /// Returns the token of `node`: for a function or block, its opening token.
    #[must_use]
    pub fn token(&self, node: NodeId) -> Token<'_> {
        self.token_of(self.node(node))
    }

    /// Returns `node` as a component value.
    #[must_use]
    pub fn component(&self, node: NodeId) -> Component<'_, P> {
        let stored = self.node(node);
        let payload = stored.payload.as_ref();
        match &stored.shape {
            Shape::Leaf => Component::Token {
                node,
                token: self.token_of(stored),
                payload,
            },
            Shape::Denied => Component::Denied {
                node,
                range: stored.range.clone(),
                payload,
            },
            Shape::Group { kind, children, .. } => match (kind, self.token_of(stored)) {
                (GroupKind::Block(kind), _) => Component::Block {
                    node,
                    kind: *kind,
                    children: *children,
                    payload,
                },
                (GroupKind::Function, Token::Function(name)) => Component::Function {
                    node,
                    name,
                    children: *children,
                    payload,
                },
                (GroupKind::Function, token) => Component::Token {
                    node,
                    token,
                    payload,
                },
            },
        }
    }

    /// Returns what kind of group `node` is, or `None` for a node that owns no
    /// list.
    #[must_use]
    pub fn group_kind(&self, node: NodeId) -> Option<GroupKind> {
        match &self.node(node).shape {
            Shape::Group { kind, .. } => Some(*kind),
            Shape::Leaf | Shape::Denied => None,
        }
    }

    /// Returns how the group `node` ended, or `None` for a node that is not a
    /// group.
    #[must_use]
    pub fn group_end(&self, node: NodeId) -> Option<&GroupEnd<P>> {
        self.node(node).group_end()
    }

    /// Returns the greatest number of groups open at once, the build's base
    /// depth included.
    #[must_use]
    pub const fn maximum_depth(&self) -> u32 {
        self.metrics.maximum_depth
    }

    /// Returns what the build counted.
    #[must_use]
    pub const fn metrics(&self) -> Metrics {
        self.metrics
    }

    /// Returns the places where the text broke a rule of section 4.3 that the
    /// specification calls a parse error, in the order the lexer met them. A
    /// document built from components has none.
    #[must_use]
    pub fn lexical_errors(&self) -> &[LexicalError] {
        &self.errors
    }

    // ----- navigation

    /// Returns the number of nodes.
    #[must_use]
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Returns the number of lists, the root list included.
    #[must_use]
    pub fn list_count(&self) -> usize {
        self.lists.len()
    }

    /// Returns the nodes of `list`, in order.
    #[must_use]
    pub fn list(&self, list: ListId) -> &[NodeId] {
        &self.lists[list.index()].nodes
    }

    /// Returns the list that the group `node` owns, or `None` for a node that
    /// is not a group.
    #[must_use]
    pub fn children(&self, node: NodeId) -> Option<ListId> {
        self.node(node).children()
    }

    /// Returns the group that owns `list`, or `None` for the root list.
    #[must_use]
    pub fn parent_group(&self, list: ListId) -> Option<NodeId> {
        self.lists[list.index()].parent
    }

    /// Returns the list that holds `node` and the node's place in it.
    #[must_use]
    pub fn location(&self, node: NodeId) -> (ListId, usize) {
        let node = self.node(node);
        (node.list, node.index)
    }

    /// Returns where `list` ends in the text: at its closing token, or at the
    /// end of the text if the group never closed. `None` when the document does
    /// not know.
    #[must_use]
    pub fn list_end(&self, list: ListId) -> Option<Offset> {
        self.lists[list.index()].end
    }

    /// Returns the boundary that sits at `offset`: the node whose token starts
    /// there, or the closing boundary of the group whose closing token does.
    /// `None` if no token or closing token starts at `offset`, and for a
    /// document whose nodes lack intervals or are not in text order.
    ///
    /// The search is a binary search over the nodes followed by a walk up the
    /// groups around the found node, so its cost is logarithmic in the node
    /// count plus the depth.
    ///
    /// ```
    /// use surgeist_css_parser::lexer::Options;
    /// use surgeist_css_parser::limits::{DepthExcess, Limits};
    /// use surgeist_css_parser::{Document, Offset};
    ///
    /// let document: Document =
    ///     Document::parse("a (b)", &Options::default(), Limits::default(), DepthExcess::Fail)?;
    /// let at_b = document.node_at_offset(Offset::new(3)).unwrap();
    /// assert!(!at_b.is_closing());
    /// let at_close = document.node_at_offset(Offset::new(4)).unwrap();
    /// assert!(at_close.is_closing());
    /// assert!(document.node_at_offset(Offset::new(1)).is_some()); // the whitespace
    /// assert!(document.node_at_offset(Offset::new(9)).is_none());
    /// # Ok::<(), surgeist_css_parser::document::BuildError>(())
    /// ```
    #[must_use]
    pub fn node_at_offset(&self, offset: Offset) -> Option<Site<'_, P>> {
        if !self.ordered {
            return None;
        }
        let before = self.nodes.partition_point(|node| {
            node.range
                .as_ref()
                .is_some_and(|range| range.start <= offset)
        });
        let found = self.nodes.get(before.checked_sub(1)?)?;
        if found
            .range
            .as_ref()
            .is_some_and(|range| range.start == offset)
        {
            return Some(Site::new(self, found.list, found.index, false));
        }
        let mut node = found;
        loop {
            if let Some(GroupEnd::Closed {
                range: Some(closer),
                ..
            }) = node.group_end()
                && closer.start == offset
            {
                return Some(Site::new(self, node.list, node.index, true));
            }
            let group = self.lists[node.list.index()].parent?;
            node = self.node(group);
        }
    }

    // ----- crate internals

    /// Admits one action through the meter, if there is one; false if refused,
    /// now or before.
    pub(crate) fn admit_step(&self) -> bool {
        if self.refused.get() {
            return false;
        }
        match self.meter {
            Some(meter) if meter.step().is_err() => {
                self.refused.set(true);
                false
            }
            _ => true,
        }
    }

    /// Admits `bytes` of live storage through the meter, if there is one; false
    /// if refused, now or before.
    pub(crate) fn admit_add(&self, bytes: usize) -> bool {
        if self.refused.get() {
            return false;
        }
        match self.meter {
            Some(meter) if meter.add(bytes).is_err() => {
                self.refused.set(true);
                false
            }
            _ => true,
        }
    }

    /// Records a refusal the meter made to a caller that used it directly.
    pub(crate) fn latch_refusal(&self) {
        self.refused.set(true);
    }

    pub(super) fn node(&self, node: NodeId) -> &Node<'s, P> {
        &self.nodes[node.index()]
    }

    pub(super) fn list_data(&self, list: ListId) -> &List {
        &self.lists[list.index()]
    }

    pub(super) fn token_of<'a>(&'a self, node: &Node<'s, P>) -> Token<'a> {
        node.token.token(&self.pool)
    }

    /// The text of the source between two offsets, which are in the coordinates
    /// the build reported (its origin included).
    pub(crate) fn slice(&self, from: Offset, to: Offset) -> Option<&'s str> {
        let from = from.value().checked_sub(self.origin)?;
        let to = to.value().checked_sub(self.origin)?;
        self.source?.get(from..to)
    }
}
