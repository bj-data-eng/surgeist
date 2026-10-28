//! A saved position in a stream.

use std::fmt;

use super::Site;
use crate::document::NodeId;

/// A restore point of a stream (section 5.3, "mark").
///
/// The specification keeps marks on a stack, which a parser can leave
/// unbalanced; a value mark has the same power and cannot be. It is `Copy`, so
/// a stream can be restored to it any number of times.
/// [`Stream::restore`](crate::Stream::restore) takes it back only on the stream
/// it came from, or a copy of that stream.
///
/// `Clone`, `Copy` and `Debug` are written by hand and ask nothing of the
/// payload type `P`.
///
/// ```
/// use surgeist_css_parser::lexer::Options;
/// use surgeist_css_parser::limits::{DepthExcess, Limits};
/// use surgeist_css_parser::Document;
///
/// let document: Document =
///     Document::parse("a b", &Options::default(), Limits::default(), DepthExcess::Fail)?;
/// let mut stream = document.root();
/// let mark = stream.mark();
/// stream.consume_ident().unwrap();
/// stream.restore(mark);
/// assert_eq!(stream.consume_ident().ok(), Some("a"));
/// # Ok::<(), surgeist_css_parser::document::BuildError>(())
/// ```
pub struct Mark<'i, P = ()> {
    pub(super) site: Site<'i, P>,
    pub(super) pending: Option<NodeId>,
    pub(super) start: usize,
    pub(super) end: usize,
}

impl<P> Clone for Mark<'_, P> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<P> Copy for Mark<'_, P> {}

impl<P> fmt::Debug for Mark<'_, P> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Mark")
            .field("site", &self.site)
            .field("pending", &self.pending)
            .field("start", &self.start)
            .field("end", &self.end)
            .finish()
    }
}

impl<'i, P> Mark<'i, P> {
    /// Returns the site the mark was taken at.
    #[must_use]
    pub const fn site(&self) -> Site<'i, P> {
        self.site
    }
}
