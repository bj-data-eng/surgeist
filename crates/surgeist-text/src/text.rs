use std::sync::Arc;

/// Immutable exact UTF-16 source text, including isolated surrogate code units.
#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub struct Text(Arc<[u16]>);

impl Text {
    /// Owns the supplied code units without repairing or normalizing them.
    #[must_use]
    pub fn from_code_units(units: Vec<u16>) -> Self {
        Self(units.into())
    }

    /// Borrows the original code units.
    #[must_use]
    pub fn code_units(&self) -> &[u16] {
        &self.0
    }

    /// Returns the length in UTF-16 code units.
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub(crate) fn is_scalar_boundary(&self, index: usize) -> bool {
        index <= self.len()
            && !(index > 0
                && index < self.len()
                && (0xd800..=0xdbff).contains(&self.0[index - 1])
                && (0xdc00..=0xdfff).contains(&self.0[index]))
    }
}

impl From<&str> for Text {
    fn from(text: &str) -> Self {
        Self::from_code_units(text.encode_utf16().collect())
    }
}

impl From<String> for Text {
    fn from(text: String) -> Self {
        Self::from(text.as_str())
    }
}
