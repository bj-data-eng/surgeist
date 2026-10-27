/// A CSS `<string>` value stored as decoded text.
///
/// The text never contains U+0000.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssContentString {
    value: String,
}

impl CssContentString {
    /// Checks decoded string text.
    ///
    /// Returns `None` when the text contains U+0000.
    #[must_use]
    pub fn try_new(value: impl Into<String>) -> Option<Self> {
        let value = value.into();
        if value.contains('\0') {
            None
        } else {
            Some(Self { value })
        }
    }

    /// Returns the decoded string text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.value
    }
}

/// # Specified serialization
///
/// Serializes a checked marker position without placing a marker.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssListStylePosition {
    /// `inside`: no special effect; the `::marker` is an inline element at the start of the list item's contents.
    Inside,
    /// `outside`: for a block-container list item, the marker box is placed outside the principal block box, on the inline-start side; for an inline-box list item, equivalent to `inside`.
    Outside,
}

/// One authored opening/closing pair in a `quotes` value.
///
/// # Specified serialization
///
/// Serializes one checked opening/closing pair with canonical string escapes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssQuotePair {
    open: CssContentString,
    close: CssContentString,
}

impl CssQuotePair {
    /// Creates an opening and closing quotation-mark pair.
    #[must_use]
    pub const fn new(open: CssContentString, close: CssContentString) -> Self {
        Self { open, close }
    }

    /// Returns the opening quotation mark.
    #[must_use]
    pub const fn open(&self) -> &CssContentString {
        &self.open
    }

    /// Returns the closing quotation mark.
    #[must_use]
    pub const fn close(&self) -> &CssContentString {
        &self.close
    }
}

/// A nonempty authored list of quotation-mark pairs.
///
/// # Specified serialization
///
/// Serializes the nonempty list in authored pair order.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssQuotePairList {
    pairs: Vec<CssQuotePair>,
}

impl CssQuotePairList {
    /// Checks a quotation-mark pair list.
    ///
    /// Returns `None` when `pairs` is empty.
    #[must_use]
    pub fn try_new(pairs: Vec<CssQuotePair>) -> Option<Self> {
        (!pairs.is_empty()).then_some(Self { pairs })
    }

    /// Returns the pairs in authored order; the slice is never empty.
    #[must_use]
    pub fn pairs(&self) -> &[CssQuotePair] {
        &self.pairs
    }
}

/// The authored CSS Generated Content 3 `quotes` value. Language-dependent
/// quotation marks remain unresolved until the element's context is known.
///
/// # Specified serialization
///
/// Serializes the symbolic keyword or checked pair list without resolving language.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssQuotes {
    /// `auto`: a typographically appropriate quotation mark system is chosen automatically from the content language of the parent (of the element itself if it has no parent).
    Auto,
    /// `none`: `open-quote` and `close-quote` produce no quotation marks, as if they were `no-open-quote` and `no-close-quote`.
    None,
    /// `match-parent`: the same quotation mark system as the parent; `auto` resolves using the content language the parent used.
    MatchParent,
    /// An explicit list of opening and closing quotation-mark pairs, the first for the outermost quote depth, the next for the next depth, and so on.
    Pairs(CssQuotePairList),
}
