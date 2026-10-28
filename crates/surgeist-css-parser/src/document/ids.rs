//! The identifiers of a document's nodes and lists.

/// The identity of one node of a [`Document`](crate::Document): a token, a
/// function, a simple block, or a denied block.
///
/// Nodes are numbered in the order they were added. For a document built from
/// text that is the order of the source, so a group's node comes before every
/// node inside it. An identifier is only meaningful for the document that
/// issued it.
///
/// ```
/// use surgeist_css_parser::Document;
/// use surgeist_css_parser::lexer::Options;
/// use surgeist_css_parser::limits::{DepthExcess, Limits};
///
/// let document: Document =
///     Document::parse("a b", &Options::default(), Limits::default(), DepthExcess::Fail)?;
/// let nodes = document.list(document.root_list());
/// assert!(nodes[0] < nodes[1]);
/// assert_eq!(nodes[0].index(), 0);
/// # Ok::<(), surgeist_css_parser::document::BuildError>(())
/// ```
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct NodeId(usize);

impl NodeId {
    pub(crate) const fn new(index: usize) -> Self {
        Self(index)
    }

    /// Returns the position of the node among all the nodes of its document,
    /// counted from zero.
    #[must_use]
    pub const fn index(self) -> usize {
        self.0
    }
}

/// The identity of one list of component values of a
/// [`Document`](crate::Document): the root list, or the contents of one
/// function or simple block.
///
/// Lists are numbered in the order they were opened. An identifier is only
/// meaningful for the document that issued it.
///
/// ```
/// use surgeist_css_parser::Document;
/// use surgeist_css_parser::lexer::Options;
/// use surgeist_css_parser::limits::{DepthExcess, Limits};
///
/// let document: Document =
///     Document::parse("f(a)", &Options::default(), Limits::default(), DepthExcess::Fail)?;
/// assert_eq!(document.root_list().index(), 0);
/// assert_eq!(document.list_count(), 2);
/// # Ok::<(), surgeist_css_parser::document::BuildError>(())
/// ```
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ListId(usize);

impl ListId {
    /// The root list, which every document has.
    pub(crate) const ROOT: Self = Self(0);

    pub(crate) const fn new(index: usize) -> Self {
        Self(index)
    }

    /// Returns the position of the list among all the lists of its document,
    /// counted from zero.
    #[must_use]
    pub const fn index(self) -> usize {
        self.0
    }
}
