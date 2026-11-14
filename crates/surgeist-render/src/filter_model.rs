//! Shared resolved native-filter semantics. Defaults belong to each operation.

/// Numeric working space selected by the resolved filter consumer.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FilterColorSpace {
    /// Operate on numeric sRGB components.
    Srgb,
    /// Decode sRGB transfer before filtering and encode after filtering.
    LinearSrgb,
}

/// Neighborhood addressing against the semantic source domain.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FilterEdgeMode {
    /// Extend the nearest boundary sample.
    Duplicate,
    /// Repeat the semantic source extent periodically.
    Wrap,
    /// Reflect with a twice-extent period, repeating boundary samples.
    Mirror,
    /// Sample transparent black beyond the semantic source extent.
    None,
}
