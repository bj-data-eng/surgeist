//! Building a document: from text, or component by component.

use std::fmt;
use std::ops::Range;

use super::component::{BlockKind, GroupEnd, GroupKind};
use super::error::{BuildError, BuildErrorKind};
use super::store::{List, Node, Shape, Stored, Text};
use super::tree::Document;
use super::{ListId, Metrics, NodeId};
use crate::lexer::{Lexer, Options};
use crate::limits::{self, DepthExcess, Limits, Meter};
use crate::{Offset, Token};

/// Makes room for `additional` more elements, charging the meter when there is
/// one.
fn reserve<T>(
    meter: Option<&dyn Meter>,
    values: &mut Vec<T>,
    additional: usize,
    at: Option<Offset>,
) -> Result<(), BuildError> {
    match meter {
        Some(meter) => limits::grow(meter, values, additional)
            .map_err(|_| BuildError::new(BuildErrorKind::Refused, at)),
        None => values
            .try_reserve(additional)
            .map_err(|_| BuildError::new(BuildErrorKind::AllocationFailure, at)),
    }
}

/// How many bytes an interval spells.
fn span(range: Option<&Range<Offset>>) -> Option<usize> {
    range.map(|range| range.end.value().saturating_sub(range.start.value()))
}

/// Builds a [`Document`] from tokens and groups a caller already holds, for
/// example the checked components of a typed value, so a grammar can parse
/// them through the same [`Stream`](crate::Stream) as parsed text (section 5.4,
/// "normalize into a token stream").
///
/// A builder charges what it adds as the document it replaces always has: for
/// every token it adds it takes one step of the [`Meter`], counts the
/// component against [`Limits::max_nodes`], compares the depth with
/// [`Limits::max_depth`] (the base depth included) and adds the token's
/// interval to the bytes it compares with [`Limits::max_bytes`]; it grows its
/// vectors through [`limits::grow`], so a host sees every byte of live
/// storage. A closing token is counted for its bytes and depth, not as a
/// component, and a group the input ended instead of closing costs nothing. A
/// failed build is dropped, and the storage it charged is released.
///
/// A builder always fails a build that nests too deeply. The alternative of
/// keeping a too-deep `{}`-block opaque, [`DepthExcess::DenyBlock`], is applied
/// by [`Document::parse`], which reads the block's extent from the text.
///
/// ```
/// use surgeist_css_parser::document::{GroupEnd, TreeBuilder};
/// use surgeist_css_parser::limits::Limits;
/// use surgeist_css_parser::{Document, Token};
///
/// let mut builder = Document::<&str>::builder(Limits::default(), 0, None)?;
/// let root = builder.root();
/// let (function, inside) = builder.open(root, Token::Function("f"), None, Some("opening"))?;
/// builder.leaf(inside, Token::Ident("a"), None, Some("argument"))?;
/// builder.close(function, GroupEnd::Closed { range: None, payload: Some("closing") })?;
/// let document = builder.finish(None);
///
/// let argument = document.list(inside)[0];
/// assert_eq!(document.payload(argument), Some(&"argument"));
/// assert_eq!(document.payload(function), Some(&"opening"));
/// # Ok::<(), surgeist_css_parser::document::BuildError>(())
/// ```
pub struct TreeBuilder<'d, P = ()> {
    document: Document<'d, P>,
    limits: Limits,
    base_depth: u32,
}

impl<P> fmt::Debug for TreeBuilder<'_, P> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("TreeBuilder")
            .field("limits", &self.limits)
            .field("base_depth", &self.base_depth)
            .field("document", &self.document)
            .finish()
    }
}

impl<'d, P> TreeBuilder<'d, P> {
    /// Returns the root list, to which a top-level component is added.
    #[must_use]
    pub const fn root(&self) -> ListId {
        ListId::ROOT
    }

    /// Adds a token that owns no list to the end of `parent`.
    ///
    /// `range` is the token's interval in the text it came from, if it has one;
    /// its length is the token's spelling. `payload` is attached to the node.
    ///
    /// # Errors
    ///
    /// Fails when the meter refuses, a limit is exceeded or storage cannot be
    /// provided.
    ///
    /// # Panics
    ///
    /// Panics if `token` opens a function or a simple block, which
    /// [`open`](Self::open) adds, if `parent` is not a list of this builder, or
    /// if `parent` is the list of a group that has been closed.
    pub fn leaf(
        &mut self,
        parent: ListId,
        token: Token<'d>,
        range: Option<Range<Offset>>,
        payload: Option<P>,
    ) -> Result<NodeId, BuildError> {
        assert!(
            GroupKind::of_opening(&token).is_none(),
            "a token that opens a group is added with `open`"
        );
        self.add_leaf(parent, Stored::borrowed(token), range, payload)
    }

    /// Adds a function or simple block to the end of `parent`, opened by
    /// `opening`, and returns its node and the list that holds its contents.
    ///
    /// Add the contents to that list and end the group with
    /// [`close`](Self::close).
    ///
    /// # Errors
    ///
    /// Fails when the meter refuses, a limit is exceeded (the nesting depth
    /// included) or storage cannot be provided.
    ///
    /// # Panics
    ///
    /// Panics if `opening` is not a function token or the opening token of a
    /// simple block, if `parent` is not a list of this builder, or if `parent` is
    /// the list of a group that has been closed.
    pub fn open(
        &mut self,
        parent: ListId,
        opening: Token<'d>,
        range: Option<Range<Offset>>,
        payload: Option<P>,
    ) -> Result<(NodeId, ListId), BuildError> {
        let kind = GroupKind::of_opening(&opening)
            .expect("`open` takes the token that opens a function or a simple block");
        self.add_group(parent, Stored::borrowed(opening), kind, range, payload)
    }

    /// Ends the group `group`, which has not ended yet.
    ///
    /// A [`GroupEnd::Closed`] is a closing token the caller holds: it is
    /// charged like any token, for its bytes and depth. A [`GroupEnd::Eof`]
    /// records that the input ended instead; nothing is charged for it.
    ///
    /// # Errors
    ///
    /// Fails when the meter refuses or a limit is exceeded.
    ///
    /// # Panics
    ///
    /// Panics if `group` is not a function or simple block of this builder, or
    /// if it has been closed already.
    pub fn close(&mut self, group: NodeId, end: GroupEnd<P>) -> Result<(), BuildError> {
        let node = self.document.node(group);
        let Shape::Group {
            children, closed, ..
        } = &node.shape
        else {
            panic!("`close` takes a function or a simple block");
        };
        assert!(!closed, "a group is closed once");
        let children = *children;
        let depth = self.document.list_data(children).depth;
        let (list_end, node_end) = match &end {
            GroupEnd::Closed { range, .. } => {
                self.charge(
                    range.as_ref().map(|range| range.start),
                    span(range.as_ref()),
                    false,
                    depth,
                )?;
                (
                    range.as_ref().map(|range| range.start),
                    range.as_ref().map(|range| range.end),
                )
            }
            GroupEnd::Eof { at } => (*at, *at),
        };
        self.document.lists[children.index()].end = list_end;
        let node = &mut self.document.nodes[group.index()];
        if let (Some(range), Some(node_end)) = (&mut node.range, node_end) {
            range.end = node_end;
        }
        if let Shape::Group {
            end: slot, closed, ..
        } = &mut node.shape
        {
            *slot = end;
            *closed = true;
        }
        Ok(())
    }

    /// Counts `bytes` more bytes of spelling against [`Limits::max_bytes`].
    ///
    /// A token added with an interval already counts that interval. A caller
    /// that knows how long a token's text was but not where it was, as for a
    /// checked component, reports it here.
    ///
    /// # Errors
    ///
    /// Fails when the total is not representable or exceeds the limit.
    pub fn charge_spelling(&mut self, bytes: usize) -> Result<(), BuildError> {
        self.add_spelling(bytes, None)
    }

    /// Finishes the document. `source` is the text the components came from,
    /// if the caller has it; positions of nodes added with intervals refer to
    /// it.
    #[must_use]
    pub fn finish(mut self, source: Option<&'d str>) -> Document<'d, P> {
        self.document.source = source;
        self.document.lists[0].end = source.map(|text| Offset::new(text.len()));
        self.document
    }

    // ----- shared with the document constructors

    pub(super) fn new(
        limits: Limits,
        base_depth: u32,
        meter: Option<&'d dyn Meter>,
    ) -> Result<Self, BuildError> {
        let mut lists = Vec::new();
        reserve(meter, &mut lists, 1, None)?;
        lists.push(List {
            nodes: Vec::new(),
            parent: None,
            end: None,
            depth: 0,
        });
        Ok(Self {
            document: Document {
                source: None,
                origin: 0,
                nodes: Vec::new(),
                lists,
                pool: String::new(),
                errors: Vec::new(),
                metrics: Metrics::default(),
                meter,
                ordered: true,
                refused: std::cell::Cell::new(false),
            },
            limits,
            base_depth,
        })
    }

    /// Latches the document's refusal if `result` is the meter's refusal, so a
    /// caller that ignores the error and finishes the build gets a document that
    /// reports it ([`Document::is_refused`]) and refuses its streams.
    fn latched<T>(&self, result: Result<T, BuildError>) -> Result<T, BuildError> {
        if let Err(error) = &result
            && error.is_refused()
        {
            self.document.latch_refusal();
        }
        result
    }

    /// Panics if `parent` belongs to a group that has ended.
    fn assert_open(&self, parent: ListId) {
        if let Some(group) = self.document.list_data(parent).parent {
            assert!(
                !matches!(
                    self.document.node(group).shape,
                    Shape::Group { closed: true, .. }
                ),
                "nothing is added to a group after it is closed"
            );
        }
    }

    fn charge(
        &mut self,
        at: Option<Offset>,
        spelling: Option<usize>,
        component: bool,
        depth: u32,
    ) -> Result<(), BuildError> {
        let fail = |kind| BuildError::new(kind, at);
        if !self.document.admit_step() {
            return Err(fail(BuildErrorKind::Refused));
        }
        if component {
            let metrics = &mut self.document.metrics;
            metrics.components = metrics
                .components
                .checked_add(1)
                .ok_or_else(|| fail(BuildErrorKind::CapacityOverflow))?;
            if metrics.components > self.limits.max_nodes() {
                return Err(fail(BuildErrorKind::NodeLimit));
            }
        }
        let depth = self
            .base_depth
            .checked_add(depth)
            .ok_or_else(|| fail(BuildErrorKind::CapacityOverflow))?;
        if depth > self.limits.max_depth() {
            return Err(fail(BuildErrorKind::DepthLimit));
        }
        let metrics = &mut self.document.metrics;
        metrics.maximum_depth = metrics.maximum_depth.max(depth);
        if let Some(bytes) = spelling {
            self.add_spelling(bytes, at)?;
        }
        Ok(())
    }

    fn add_spelling(&mut self, bytes: usize, at: Option<Offset>) -> Result<(), BuildError> {
        let metrics = &mut self.document.metrics;
        metrics.spelling_bytes = metrics
            .spelling_bytes
            .checked_add(bytes)
            .ok_or_else(|| BuildError::new(BuildErrorKind::CapacityOverflow, at))?;
        if metrics.spelling_bytes > self.limits.max_bytes() {
            return Err(BuildError::new(BuildErrorKind::ByteLimit, at));
        }
        Ok(())
    }

    fn push(
        &mut self,
        parent: ListId,
        token: Stored<'d>,
        shape: Shape<P>,
        range: Option<Range<Offset>>,
        payload: Option<P>,
    ) -> Result<NodeId, BuildError> {
        let at = range.as_ref().map(|range| range.start);
        let meter = self.document.meter;
        let grown = reserve(meter, &mut self.document.nodes, 1, at);
        self.latched(grown)?;
        let grown = reserve(meter, &mut self.document.lists[parent.index()].nodes, 1, at);
        self.latched(grown)?;
        let ordered = match (&range, self.document.nodes.last()) {
            (None, _) => false,
            (Some(range), Some(last)) => last
                .range
                .as_ref()
                .is_some_and(|last| last.start <= range.start),
            (Some(_), None) => true,
        };
        self.document.ordered &= ordered;
        let id = NodeId::new(self.document.nodes.len());
        let list = &mut self.document.lists[parent.index()];
        let index = list.nodes.len();
        list.nodes.push(id);
        self.document.nodes.push(Node {
            token,
            shape,
            range,
            payload,
            list: parent,
            index,
        });
        Ok(id)
    }

    pub(super) fn add_leaf(
        &mut self,
        parent: ListId,
        token: Stored<'d>,
        range: Option<Range<Offset>>,
        payload: Option<P>,
    ) -> Result<NodeId, BuildError> {
        self.assert_open(parent);
        let depth = self.document.list_data(parent).depth;
        self.charge(
            range.as_ref().map(|range| range.start),
            span(range.as_ref()),
            true,
            depth,
        )?;
        self.push(parent, token, Shape::Leaf, range, payload)
    }

    pub(super) fn add_group(
        &mut self,
        parent: ListId,
        token: Stored<'d>,
        kind: GroupKind,
        range: Option<Range<Offset>>,
        payload: Option<P>,
    ) -> Result<(NodeId, ListId), BuildError> {
        self.assert_open(parent);
        let at = range.as_ref().map(|range| range.start);
        let depth = self
            .document
            .list_data(parent)
            .depth
            .checked_add(1)
            .ok_or_else(|| BuildError::new(BuildErrorKind::CapacityOverflow, at))?;
        self.charge(at, span(range.as_ref()), true, depth)?;
        let meter = self.document.meter;
        let grown = reserve(meter, &mut self.document.lists, 1, at);
        self.latched(grown)?;
        let children = ListId::new(self.document.lists.len());
        self.document.lists.push(List {
            nodes: Vec::new(),
            parent: None,
            end: None,
            depth,
        });
        let opening_end = range.as_ref().map(|range| range.end);
        let node = self.push(
            parent,
            token,
            Shape::Group {
                kind,
                children,
                end: GroupEnd::Eof { at: None },
                opening_end,
                closed: false,
            },
            range,
            payload,
        )?;
        self.document.lists[children.index()].parent = Some(node);
        Ok((node, children))
    }

    /// Adds a `{}`-block that nested too deep, kept as one opaque node.
    pub(super) fn add_denied(
        &mut self,
        parent: ListId,
        range: Range<Offset>,
    ) -> Result<NodeId, BuildError> {
        let depth = self.document.list_data(parent).depth;
        self.charge(Some(range.start), span(Some(&range)), true, depth)?;
        self.push(
            parent,
            Stored::Plain(Token::OpenCurly),
            Shape::Denied,
            Some(range),
            None,
        )
    }
}

/// An open group while parsing text.
struct Frame {
    node: NodeId,
    /// The list of the group's contents.
    list: ListId,
    /// The block whose closing token ends the group.
    closed_by: BlockKind,
}

/// Where a pending denied block began, and the groups it still has open.
struct Denial {
    start: Offset,
    open: Vec<BlockKind>,
}

impl Denial {
    /// Takes the token of the denied block; returns whether it ended the
    /// block.
    fn take(&mut self, token: &Token<'_>) -> bool {
        if let Some(kind) = GroupKind::of_opening(token) {
            self.open.push(kind.closing_block());
            return false;
        }
        match GroupKind::of_closing(token) {
            Some(block) if self.open.last() == Some(&block) => {
                self.open.pop();
                self.open.is_empty()
            }
            _ => false,
        }
    }
}

/// Places `value`: a slice of `source` stays a borrow of it, and any other
/// text goes to the pool.
fn place<'s>(source: &'s str, pool: &mut String, value: &str) -> Text<'s> {
    if value.is_empty() {
        return Text::Borrowed("");
    }
    let base = source.as_ptr().addr();
    let at = value.as_ptr().addr();
    if let Some(from) = at.checked_sub(base)
        && let Some(to) = from.checked_add(value.len())
        && let Some(slice) = source.get(from..to)
    {
        return Text::Borrowed(slice);
    }
    let start = pool.len();
    pool.push_str(value);
    Text::Pooled {
        start,
        end: pool.len(),
    }
}

impl<'s, P> Document<'s, P> {
    /// Tokenizes `source` and builds the document, without a meter.
    ///
    /// The lexer runs once, free-running, and the tree is built iteratively,
    /// so the depth of the text threatens no stack. Every text builds: an
    /// unclosed group is closed by the end of the text, a closing token that
    /// matches no open group is kept as a token in the list it appeared in,
    /// and bad strings and bad urls are kept as tokens. Section 5's
    /// preserved parse errors are therefore all visible to a consumer.
    ///
    /// A closing token matches the innermost open group only: a `]` does not
    /// close a `(`-block that is open, and does not close an outer
    /// `[`-block behind it either.
    ///
    /// Text past the limits fails the build. `depth_excess` says what happens
    /// to a `{}`-block that nests deeper than [`Limits::max_depth`] allows:
    /// [`DepthExcess::Fail`] fails the build, and [`DepthExcess::DenyBlock`]
    /// keeps it as one opaque [`Component::Denied`](super::Component::Denied)
    /// that spans the block to its closing token or the end of the text, and
    /// carries on. Any other kind of group nested too deeply fails the build in
    /// both cases.
    ///
    /// `P` is the payload type of the result; a text carries no payloads, so
    /// every node has `None`.
    ///
    /// # Errors
    ///
    /// Returns a [`BuildError`] when a limit is exceeded or storage cannot be
    /// provided.
    ///
    /// ```
    /// use surgeist_css_parser::lexer::Options;
    /// use surgeist_css_parser::limits::{DepthExcess, Limits};
    /// use surgeist_css_parser::Document;
    ///
    /// let document: Document =
    ///     Document::parse("a{b:c}", &Options::default(), Limits::default(), DepthExcess::Fail)?;
    /// assert_eq!(document.node_count(), 5); // a, the block, b, the colon, c
    /// assert_eq!(document.metrics().maximum_depth(), 1);
    /// # Ok::<(), surgeist_css_parser::document::BuildError>(())
    /// ```
    pub fn parse(
        source: &'s str,
        options: &Options,
        limits: Limits,
        depth_excess: DepthExcess,
    ) -> Result<Self, BuildError> {
        let mut builder = TreeBuilder::new(limits, 0, None)?;
        let origin = options.origin_offset().value();
        let end = origin
            .checked_add(source.len())
            .map(Offset::new)
            .ok_or_else(|| BuildError::new(BuildErrorKind::CapacityOverflow, None))?;
        builder.document.origin = origin;
        let mut lexer = Lexer::new(source, options);
        let mut frames: Vec<Frame> = Vec::new();
        let mut denial: Option<Denial> = None;
        while let Some(lexeme) = lexer.next_lexeme() {
            let range = lexeme.range();
            let token = lexeme.token();
            if let Some(pending) = &mut denial {
                if pending.take(&token) {
                    let block = pending.start..range.end;
                    denial = None;
                    let parent = frames.last().map_or(ListId::ROOT, |frame| frame.list);
                    builder.add_denied(parent, block)?;
                }
                continue;
            }
            let parent = frames.last().map_or(ListId::ROOT, |frame| frame.list);
            if let Some(kind) = GroupKind::of_opening(&token) {
                let depth = u32::try_from(frames.len() + 1).unwrap_or(u32::MAX);
                if depth_excess == DepthExcess::DenyBlock
                    && token == Token::OpenCurly
                    && depth > limits.max_depth()
                {
                    denial = Some(Denial {
                        start: range.start,
                        open: vec![BlockKind::Curly],
                    });
                    continue;
                }
                let stored = Stored::of(token, &mut |value| {
                    place(source, &mut builder.document.pool, value)
                });
                let (node, list) = builder.add_group(parent, stored, kind, Some(range), None)?;
                frames.push(Frame {
                    node,
                    list,
                    closed_by: kind.closing_block(),
                });
            } else if let Some(block) = GroupKind::of_closing(&token)
                && frames.last().is_some_and(|frame| frame.closed_by == block)
                && let Some(frame) = frames.pop()
            {
                builder.close(
                    frame.node,
                    GroupEnd::Closed {
                        range: Some(range),
                        payload: None,
                    },
                )?;
            } else {
                let stored = Stored::of(token, &mut |value| {
                    place(source, &mut builder.document.pool, value)
                });
                builder.add_leaf(parent, stored, Some(range), None)?;
            }
        }
        if let Some(pending) = denial {
            let parent = frames.last().map_or(ListId::ROOT, |frame| frame.list);
            builder.add_denied(parent, pending.start..end)?;
        }
        while let Some(frame) = frames.pop() {
            builder.close(frame.node, GroupEnd::Eof { at: Some(end) })?;
        }
        builder.document.errors = lexer.errors().to_vec();
        builder.document.source = Some(source);
        builder.document.lists[0].end = Some(end);
        Ok(builder.document)
    }

    /// Starts building a document from components a caller holds.
    ///
    /// `limits` bound the build. `base_depth` is the nesting depth the
    /// components already sit at; it counts against [`Limits::max_depth`] and
    /// appears in [`Metrics::maximum_depth`]. A `meter`, if given, admits every
    /// action and byte of storage the build takes, and it stays with the
    /// finished document for the streams over it.
    ///
    /// # Errors
    ///
    /// Fails when the meter refuses the first allocation.
    pub fn builder(
        limits: Limits,
        base_depth: u32,
        meter: Option<&'s dyn Meter>,
    ) -> Result<TreeBuilder<'s, P>, BuildError> {
        TreeBuilder::new(limits, base_depth, meter)
    }
}
