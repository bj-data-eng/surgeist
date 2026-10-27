//! Controlled UTF16 property input over the original CSS syntax owners.
use crate::source::CssUtf16InputLimits;

/// Independent preparation policy and the complete operation envelope. The
/// component limits that charge the lexical input come from the
/// [`crate::CssParser`] that drives the operation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CssUtf16PropertyValueLimits {
    pub input: CssUtf16InputLimits,
    pub max_work: usize,
    pub max_live_bytes: usize,
    pub max_retained_bytes: usize,
}

/// Actual owned payload, including fixed output/source headers. Opaque allocator
/// and Arc control-block overhead is outside the repository payload tariff.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CssUtf16ValueStorage {
    pub(crate) bytes: usize,
    pub(crate) utf16_units: usize,
}
impl CssUtf16ValueStorage {
    #[must_use]
    pub const fn bytes(self) -> usize {
        self.bytes
    }
    /// Original UTF16 Vec capacity; normalized UTF8 and decoded names add bytes.
    #[must_use]
    pub const fn utf16_units(self) -> usize {
        self.utf16_units
    }
}

/// One actual observation or prospective/current source-inclusive live extent.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CssUtf16ValueAdmission {
    Work,
    Live(CssUtf16ValueStorage),
}
