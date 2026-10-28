//! The document: every token of a text, in a tree of lists.

use std::cell::Cell;
use std::fmt;
use std::mem::size_of;
use std::ops::Range;

use super::component::{Component, GroupEnd, GroupKind};
use super::store::{List, Node};
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
    /// How many bytes at the start of the pool are the text of a document that
    /// owns it, [`Document::parse_owned`].
    pub(super) owned_len: Option<usize>,
    /// What the host attached to the document as a whole.
    pub(super) payload: Option<P>,
    /// The offset at which the source begins in a larger text.
    pub(super) origin: usize,
    pub(super) nodes: Vec<Node<'s, P>>,
    pub(super) lists: Vec<List<P>>,
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
            .field(
                "has_source",
                &(self.source.is_some() || self.owned_len.is_some()),
            )
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
        bytes = bytes.saturating_add(self.lists.capacity().saturating_mul(size_of::<List<P>>()));
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
    pub fn source(&self) -> Option<&str> {
        self.source
            .or_else(|| self.owned_len.and_then(|length| self.pool.get(..length)))
    }

    /// Returns the payload the host attached to the document as a whole with
    /// [`attach`](Self::attach), if it did.
    ///
    /// A node's payload is data about that node; this is data about the
    /// document, which a host that reads nodes through a
    /// [`Stream`](crate::Stream) or a [`Site`](crate::Site) otherwise has no
    /// way to reach, for example where the text came from.
    #[must_use]
    pub const fn attached(&self) -> Option<&P> {
        self.payload.as_ref()
    }

    /// Attaches `payload` to the document as a whole, and returns the one it
    /// replaces.
    ///
    /// ```
    /// use surgeist_css_parser::lexer::Options;
    /// use surgeist_css_parser::limits::{DepthExcess, Limits};
    /// use surgeist_css_parser::Document;
    ///
    /// let mut document: Document<&str> =
    ///     Document::parse("a", &Options::default(), Limits::default(), DepthExcess::Fail)?;
    /// assert_eq!(document.attached(), None);
    /// assert_eq!(document.attach("the host"), None);
    /// assert_eq!(document.attached(), Some(&"the host"));
    /// # Ok::<(), surgeist_css_parser::document::BuildError>(())
    /// ```
    pub fn attach(&mut self, payload: P) -> Option<P> {
        self.payload.replace(payload)
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
        let range = node.range()?;
        let end = match node.children {
            Some(children) => self.lists[children.index()].group()?.opening_end?,
            None => range.end,
        };
        self.slice(range.start, end)
    }

    /// Returns the interval of `node` in the text: for a function or block
    /// that has ended, through its closing token or the end of the text.
    #[must_use]
    pub fn range(&self, node: NodeId) -> Option<Range<Offset>> {
        self.node(node).range()
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
        let token = self.token_of(stored);
        let Some(children) = stored.children else {
            return if stored.token.is_denied() {
                Component::Denied {
                    node,
                    range: stored.range(),
                    payload,
                }
            } else {
                Component::Token {
                    node,
                    token,
                    payload,
                }
            };
        };
        match (self.lists[children.index()].kind(), token) {
            (Some(GroupKind::Block(kind)), _) => Component::Block {
                node,
                kind,
                children,
                payload,
            },
            (Some(GroupKind::Function), Token::Function(name)) => Component::Function {
                node,
                name,
                children,
                payload,
            },
            (_, token) => Component::Token {
                node,
                token,
                payload,
            },
        }
    }

    /// Returns what kind of group `node` is, or `None` for a node that owns no
    /// list.
    #[must_use]
    pub fn group_kind(&self, node: NodeId) -> Option<GroupKind> {
        self.lists[self.node(node).children?.index()].kind()
    }

    /// Returns how the group `node` ended, or `None` for a node that is not a
    /// group.
    #[must_use]
    pub fn group_end(&self, node: NodeId) -> Option<&GroupEnd<P>> {
        self.group_end_of(self.node(node))
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

    /// Returns every node, in the order they were added; for a document built
    /// from text, the order of the source.
    ///
    /// ```
    /// use surgeist_css_parser::lexer::Options;
    /// use surgeist_css_parser::limits::{DepthExcess, Limits};
    /// use surgeist_css_parser::{Document, Token};
    ///
    /// let document: Document =
    ///     Document::parse("f(a) b", &Options::default(), Limits::default(), DepthExcess::Fail)?;
    /// let tokens: Vec<_> = document.nodes().map(|node| document.token(node)).collect();
    /// assert_eq!(
    ///     tokens,
    ///     [Token::Function("f"), Token::Ident("a"), Token::Whitespace, Token::Ident("b")]
    /// );
    /// # Ok::<(), surgeist_css_parser::document::BuildError>(())
    /// ```
    pub fn nodes(&self) -> impl ExactSizeIterator<Item = NodeId> + use<'s, P> {
        (0..self.nodes.len())
            .map(|index| NodeId::from_index(index).expect("a document holds at most 2^32 nodes"))
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
        self.node(node).children
    }

    /// Returns the group that owns `list`, or `None` for the root list.
    #[must_use]
    pub fn parent_group(&self, list: ListId) -> Option<NodeId> {
        self.lists[list.index()].parent()
    }

    /// Returns the list that holds `node` and the node's place in it.
    #[must_use]
    pub fn location(&self, node: NodeId) -> (ListId, usize) {
        let node = self.node(node);
        (node.list, node.index as usize)
    }

    /// Returns where `list` ends in the text: at its closing token, or at the
    /// end of the text if the group never closed. `None` when the document does
    /// not know.
    #[must_use]
    pub fn list_end(&self, list: ListId) -> Option<Offset> {
        self.lists[list.index()].end()
    }

    /// Returns the boundary before the node at `index` of `list`, which is the
    /// end of the list when `index` is the list's length. `None` when `index`
    /// is past the end.
    ///
    /// ```
    /// use surgeist_css_parser::lexer::Options;
    /// use surgeist_css_parser::limits::{DepthExcess, Limits};
    /// use surgeist_css_parser::Document;
    ///
    /// let document: Document =
    ///     Document::parse("a b", &Options::default(), Limits::default(), DepthExcess::Fail)?;
    /// let root = document.root_list();
    /// assert_eq!(document.site(root, 2).and_then(|site| site.token()), Some(surgeist_css_parser::Token::Ident("b")));
    /// assert!(document.site(root, 3).is_some_and(|site| site.node().is_none()));
    /// assert!(document.site(root, 4).is_none());
    /// # Ok::<(), surgeist_css_parser::document::BuildError>(())
    /// ```
    #[must_use]
    pub fn site(&self, list: ListId, index: usize) -> Option<Site<'_, P>> {
        (index <= self.lists[list.index()].nodes.len()).then(|| Site::new(self, list, index, false))
    }

    /// Returns the closing boundary of the group `group`, or `None` when the
    /// node is not a function or a simple block.
    #[must_use]
    pub fn closing_site(&self, group: NodeId) -> Option<Site<'_, P>> {
        let node = self.node(group);
        node.children?;
        Some(Site::new(self, node.list, node.index as usize, true))
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
        let before = self
            .nodes
            .partition_point(|node| node.start().is_some_and(|start| start <= offset));
        let found = self.nodes.get(before.checked_sub(1)?)?;
        if found.start() == Some(offset) {
            return Some(Site::new(self, found.list, found.index as usize, false));
        }
        let mut node = found;
        loop {
            if let Some(GroupEnd::Closed {
                range: Some(closer),
                ..
            }) = self.group_end_of(node)
                && closer.start == offset
            {
                return Some(Site::new(self, node.list, node.index as usize, true));
            }
            let group = self.lists[node.list.index()].parent()?;
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

    pub(super) fn list_data(&self, list: ListId) -> &List<P> {
        &self.lists[list.index()]
    }

    /// How the group `node` ended; none for a node that owns no list.
    fn group_end_of(&self, node: &Node<'s, P>) -> Option<&GroupEnd<P>> {
        let children = node.children?;
        self.lists[children.index()].group().map(|group| &group.end)
    }

    pub(super) fn token_of<'a>(&'a self, node: &Node<'s, P>) -> Token<'a> {
        node.token.token(&self.pool)
    }

    /// The text of the source between two offsets, which are in the coordinates
    /// the build reported (its origin included).
    pub(crate) fn slice(&self, from: Offset, to: Offset) -> Option<&str> {
        let from = from.value().checked_sub(self.origin)?;
        let to = to.value().checked_sub(self.origin)?;
        self.source().and_then(|text| text.get(from..to))
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use super::*;
    use crate::limits::{Failure, Limits, Refused};

    /// Keeps the bytes admitted and not yet returned.
    #[derive(Default)]
    struct Live(Cell<usize>);

    impl Meter for Live {
        fn step(&self) -> Result<(), Refused> {
            Ok(())
        }
        fn add(&self, bytes: usize) -> Result<(), Refused> {
            self.0.set(self.0.get() + bytes);
            Ok(())
        }
        fn released(&self, bytes: usize) {
            self.0.set(self.0.get() - bytes);
        }
        fn note(&self, _: Failure) -> Refused {
            Refused
        }
    }

    /// The charge is the size of what is stored: once a metered build has
    /// finished, the storage a meter holds admitted is the capacity of the node
    /// vector, of the list vector and of every list's node vector, each times
    /// the size of its element; dropping the document returns all of it. That
    /// is all the storage of a metered build: the decode pool and the lexical
    /// errors are filled only by `Document::parse`, which has no meter, and a
    /// builder borrows its texts.
    #[test]
    fn a_metered_build_holds_charged_exactly_the_storage_it_keeps() {
        let meter = Live::default();
        let mut builder = Document::<u64>::builder(Limits::default(), 0, Some(&meter)).unwrap();
        let root = builder.root();
        for index in 0..40_u64 {
            let (group, inside) = builder
                .open(root, Token::Function("f"), None, Some(index))
                .unwrap();
            for _ in 0..(index % 7) {
                builder.leaf(inside, Token::Comma, None, None).unwrap();
            }
            builder.close(group, GroupEnd::Eof { at: None }).unwrap();
            builder.leaf(root, Token::Ident("a"), None, None).unwrap();
        }
        let document = builder.finish(None);
        let stored = document.nodes.capacity() * size_of::<Node<'_, u64>>()
            + document.lists.capacity() * size_of::<List<u64>>()
            + document
                .lists
                .iter()
                .map(|list| list.nodes.capacity() * size_of::<NodeId>())
                .sum::<usize>();
        assert_eq!(meter.0.get(), stored);
        drop(document);
        assert_eq!(meter.0.get(), 0);
    }
}
