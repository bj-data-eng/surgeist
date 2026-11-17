use crate::{capability::SymbolicColorPolicy, paint::Color};

pub mod background;
pub mod clip;
pub mod decoration;
pub mod filter;
pub mod image;
pub mod mask;

pub(crate) use self::clip::validate_clip_input;
#[cfg(test)]
pub(crate) use self::filter::ColorFilterPipeline;
#[cfg(test)]
pub(crate) use self::filter::filter_drop_shadow_payload_accepts_shadow_for_test;

#[derive(Clone, Copy, Debug, PartialEq)]
/// A concrete color accepted by the current style-facing rendering model.
///
/// Symbolic colors are not stored here; callers resolve them before constructing
/// this value, as reported by [`Self::symbolic_policy`].
pub struct StyleColor {
    color: Color,
}

impl StyleColor {
    /// Wraps an already resolved rendering color.
    #[must_use]
    pub const fn new(color: Color) -> Self {
        Self { color }
    }

    /// Returns the concrete rendering color.
    #[must_use]
    pub const fn color(self) -> Color {
        self.color
    }

    /// Returns the current requirement for resolving symbolic colors.
    #[must_use]
    pub const fn symbolic_policy() -> SymbolicColorPolicy {
        SymbolicColorPolicy::RootResolvedOnly
    }
}
