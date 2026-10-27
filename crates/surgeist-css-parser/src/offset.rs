//! The position type shared by every part of the crate.

/// A byte offset into the text a tokenizer was given.
///
/// For UTF-8 input the offset addresses the original bytes. For UTF-16 input
/// that was preprocessed first it addresses the normalized text. An `Offset`
/// is neither a line nor a column; callers that report those derive them from
/// the offset and the text.
///
/// # Examples
///
/// ```
/// use surgeist_css_parser::Offset;
///
/// let start = Offset::new(4);
/// assert_eq!(start.value(), 4);
/// assert!(start < Offset::new(5));
/// ```
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Offset(usize);

impl Offset {
    /// Creates the offset `value` bytes into the text.
    #[must_use]
    pub const fn new(value: usize) -> Self {
        Self(value)
    }

    /// Returns the number of bytes between the start of the text and this
    /// offset.
    #[must_use]
    pub const fn value(self) -> usize {
        self.0
    }
}
