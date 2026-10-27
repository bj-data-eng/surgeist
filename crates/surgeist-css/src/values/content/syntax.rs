#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssContentString {
    value: String,
}

impl CssContentString {
    #[must_use]
    pub fn try_new(value: impl Into<String>) -> Option<Self> {
        let value = value.into();
        if value.contains('\0') {
            None
        } else {
            Some(Self { value })
        }
    }

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
    Inside,
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
    #[must_use]
    pub const fn new(open: CssContentString, close: CssContentString) -> Self {
        Self { open, close }
    }

    #[must_use]
    pub const fn open(&self) -> &CssContentString {
        &self.open
    }

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
    #[must_use]
    pub fn try_new(pairs: Vec<CssQuotePair>) -> Option<Self> {
        (!pairs.is_empty()).then_some(Self { pairs })
    }

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
    Auto,
    None,
    MatchParent,
    Pairs(CssQuotePairList),
}
