//! Controlled UTF16 property input over the original CSS syntax owners.
use crate::source::CssUtf16InputLimits;

/// Independent preparation policy and the complete operation envelope. The
/// component limits that charge the lexical input come from the
/// [`crate::CssParser`] that drives the operation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CssUtf16PropertyValueLimits {
    /// Preparation limits for the original UTF-16 input.
    pub input: CssUtf16InputLimits,
    /// Maximum charged work: observations admitted as [`CssUtf16ValueAdmission::Work`].
    pub max_work: usize,
    /// Maximum live bytes at any one time, including the source.
    pub max_live_bytes: usize,
    /// Maximum retained bytes of the resulting source and output.
    pub max_retained_bytes: usize,
}
impl Default for CssUtf16PropertyValueLimits {
    /// Operation ceilings that start from [`CssUtf16InputLimits::default`].
    ///
    /// `input` is that default, `max_work` is `input.max_work`, and
    /// `max_live_bytes` and `max_retained_bytes` are `input.max_retained_bytes`.
    /// The operation charges its own parse steps and live observations on top of
    /// the preparation's work, and holds the source and its output together, so
    /// it can refuse an input that preparation alone would admit. A caller with
    /// larger inputs passes explicit limits.
    fn default() -> Self {
        let input = CssUtf16InputLimits::default();
        Self {
            input,
            max_work: input.max_work,
            max_live_bytes: input.max_retained_bytes,
            max_retained_bytes: input.max_retained_bytes,
        }
    }
}

/// Actual owned payload, including fixed output/source headers. Opaque allocator
/// and Arc control-block overhead is outside the repository payload tariff.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CssUtf16ValueStorage {
    pub(crate) bytes: usize,
    pub(crate) utf16_units: usize,
}
impl CssUtf16ValueStorage {
    /// Returns the owned payload bytes, including fixed output and source headers.
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
    /// One unit of charged work, admitted before its operands are read.
    Work,
    /// The source-inclusive live storage the operation prospectively or currently holds.
    Live(CssUtf16ValueStorage),
}
