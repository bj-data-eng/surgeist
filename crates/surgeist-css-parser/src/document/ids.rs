//! The identifiers of a document's nodes and lists.

use std::fmt;
use std::num::NonZeroU32;

/// The identity of one node of a [`Document`](crate::Document): a token, a
/// function, a simple block, or a denied block.
///
/// Nodes are numbered in the order they were added. For a document built from
/// text that is the order of the source, so a group's node comes before every
/// node inside it. An identifier is only meaningful for the document that
/// issued it.
///
/// An identifier holds the node's position as a `u32`, so a document has at
/// most 2^32 nodes, at positions `0` through `u32::MAX`; a build that would
/// add one more fails with a [`BuildError`](super::BuildError) for which
/// `is_capacity_overflow` holds. A node's place in its list is a `u32` too, so
/// a list holds at most 2^32 nodes.
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
pub struct NodeId(u32);

impl NodeId {
    /// The identifier of the node at `index`, or `None` when `index` is
    /// greater than `u32::MAX`.
    pub(crate) fn from_index(index: usize) -> Option<Self> {
        u32::try_from(index).ok().map(Self)
    }

    /// Returns the position of the node among all the nodes of its document,
    /// counted from zero.
    #[must_use]
    pub const fn index(self) -> usize {
        self.0 as usize
    }
}

/// The identity of one list of component values of a
/// [`Document`](crate::Document): the root list, or the contents of one
/// function or simple block.
///
/// Lists are numbered in the order they were opened. An identifier is only
/// meaningful for the document that issued it.
///
/// An identifier holds the list's position plus one in a `NonZeroU32`, so a
/// document has at most `u32::MAX` lists, at positions `0` through
/// `u32::MAX - 1`; a build that would open one more fails with a
/// [`BuildError`](super::BuildError) for which `is_capacity_overflow` holds.
/// `Debug` prints the position, so the root list is `ListId(0)`.
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
#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ListId(NonZeroU32);

impl fmt::Debug for ListId {
    /// Prints the position of the list, not the number that stores it.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_tuple("ListId")
            .field(&self.index())
            .finish()
    }
}

impl ListId {
    /// The root list, which every document has.
    pub(crate) const ROOT: Self = Self(NonZeroU32::MIN);

    /// The identifier of the list at `index`, or `None` when `index` is
    /// `u32::MAX` or greater.
    pub(crate) fn from_index(index: usize) -> Option<Self> {
        let number = u32::try_from(index.checked_add(1)?).ok()?;
        NonZeroU32::new(number).map(Self)
    }

    /// Returns the position of the list among all the lists of its document,
    /// counted from zero.
    #[must_use]
    pub const fn index(self) -> usize {
        self.0.get() as usize - 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const MAX: usize = u32::MAX as usize;

    #[test]
    fn a_node_is_numbered_up_to_the_largest_u32_position() {
        assert_eq!(NodeId::from_index(0).map(NodeId::index), Some(0));
        assert_eq!(NodeId::from_index(MAX).map(NodeId::index), Some(MAX));
        assert_eq!(NodeId::from_index(MAX + 1), None);
        assert_eq!(NodeId::from_index(usize::MAX), None);
    }

    #[test]
    fn a_list_is_numbered_up_to_the_position_before_the_largest_u32() {
        assert_eq!(ListId::from_index(0), Some(ListId::ROOT));
        assert_eq!(ListId::ROOT.index(), 0);
        assert_eq!(
            ListId::from_index(MAX - 1).map(ListId::index),
            Some(MAX - 1)
        );
        assert_eq!(ListId::from_index(MAX), None);
        assert_eq!(ListId::from_index(usize::MAX), None);
    }

    #[test]
    fn identifiers_print_their_positions() {
        assert_eq!(format!("{:?}", ListId::ROOT), "ListId(0)");
        assert_eq!(format!("{:?}", ListId::from_index(7)), "Some(ListId(7))");
        assert_eq!(format!("{:?}", NodeId::from_index(3)), "Some(NodeId(3))");
    }

    #[test]
    fn identifiers_order_by_position() {
        assert!(Some(ListId::ROOT) < ListId::from_index(1));
        assert!(ListId::from_index(1) < ListId::from_index(2));
        assert!(NodeId::from_index(1) < NodeId::from_index(2));
    }
}
