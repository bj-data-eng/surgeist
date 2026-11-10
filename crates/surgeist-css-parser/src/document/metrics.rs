//! What a build counted.

/// The sizes a build measured: how many component values it admitted, how deep
/// they nest, and how many source bytes they spell.
///
/// The counts are the ones the build's [`Limits`](crate::limits::Limits)
/// are compared with: opening a function or a simple block counts one
/// component and its closing token counts none, and a group that the end of the
/// text closed contributes no bytes for the closing token it never had.
///
/// ```
/// use surgeist_css_parser::Document;
/// use surgeist_css_parser::lexer::Options;
/// use surgeist_css_parser::limits::{DepthExcess, Limits};
///
/// let document: Document =
///     Document::parse("f(a)", &Options::default(), Limits::default(), 0, DepthExcess::Fail)?;
/// let metrics = document.metrics();
/// assert_eq!(metrics.components(), 2); // the function and `a`
/// assert_eq!(metrics.maximum_depth(), 1);
/// assert_eq!(metrics.spelling_bytes(), 4); // `f(`, `a` and `)`
/// # Ok::<(), surgeist_css_parser::document::BuildError>(())
/// ```
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct Metrics {
    pub(super) components: usize,
    pub(super) maximum_depth: u32,
    pub(super) spelling_bytes: usize,
}

impl Metrics {
    /// Returns the number of component values: tokens, functions, simple
    /// blocks and denied blocks, but not closing tokens that end a group.
    #[must_use]
    pub const fn components(self) -> usize {
        self.components
    }

    /// Returns the greatest number of functions and simple blocks open at once,
    /// plus the base depth the build started with.
    #[must_use]
    pub const fn maximum_depth(self) -> u32 {
        self.maximum_depth
    }

    /// Returns the number of source bytes the admitted tokens span. A token
    /// added without an interval contributes the bytes its builder was told
    /// about with [`TreeBuilder::charge_spelling`](super::TreeBuilder::charge_spelling),
    /// and none otherwise.
    #[must_use]
    pub const fn spelling_bytes(self) -> usize {
        self.spelling_bytes
    }
}
