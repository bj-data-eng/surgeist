//! Authored container-feature values. Contextual substitution and matching are external.
use crate::component_values::CssCanonicalBuilder;
use crate::*;

/// The six size features defined by the selected Conditional Rules grammar.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum CssContainerSizeFeatureKind {
    Width,
    Height,
    InlineSize,
    BlockSize,
    AspectRatio,
    Orientation,
}
impl CssContainerSizeFeatureKind {
    pub const fn name(self) -> &'static str {
        match self {
            Self::Width => "width",
            Self::Height => "height",
            Self::InlineSize => "inline-size",
            Self::BlockSize => "block-size",
            Self::AspectRatio => "aspect-ratio",
            Self::Orientation => "orientation",
        }
    }
}

/// Required grammar after external substitution, not a resolved numeric type.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssContainerValueDomain {
    Length,
    Ratio,
    Orientation,
    Stuck,
    Snapped,
    ScrollDirection,
}

/// A complete operand with checked `var()` syntax, awaiting external substitution.
/// A fallback need not match the final domain before substitution chooses it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssContainerPendingValue {
    domain: CssContainerValueDomain,
    components: CssComponentValues,
}
impl CssContainerPendingValue {
    pub(crate) fn new(domain: CssContainerValueDomain, components: CssComponentValues) -> Self {
        Self { domain, components }
    }
    pub const fn domain(&self) -> CssContainerValueDomain {
        self.domain
    }
    pub fn components(&self) -> &CssComponentValues {
        &self.components
    }
    pub fn serialize(&self) -> Result<CssSerializedValue, CssComponentValueError> {
        self.serialize_with_limit(usize::MAX)
    }
    pub fn serialize_with_limit(
        &self,
        max_css_bytes: usize,
    ) -> Result<CssSerializedValue, CssComponentValueError> {
        let mut out = CssCanonicalBuilder::new(max_css_bytes);
        out.push_components(self.components.items())?;
        out.finish()
    }
}

macro_rules! operand {
    ($(#[$doc:meta])* $name:ident, $view:ident, $state:ident, $variant:ident, $value:ty) => {
        $(#[$doc])*
        #[derive(Clone, Debug, PartialEq)]
        pub struct $name {
            value: $state,
        }
        #[derive(Clone, Debug, PartialEq)]
        enum $state {
            $variant { value: $value, components: CssComponentValues },
            Pending(CssContainerPendingValue),
        }
        #[derive(Clone, Copy, Debug)]
        #[non_exhaustive]
        pub enum $view<'a> {
            $variant(&'a $value),
            Pending(&'a CssContainerPendingValue),
        }
        impl $name {
            pub(crate) fn typed(value: $value, components: CssComponentValues) -> Self {
                Self { value: $state::$variant { value, components } }
            }
            pub(crate) fn pending(value: CssContainerPendingValue) -> Self {
                Self { value: $state::Pending(value) }
            }
            pub fn view(&self) -> $view<'_> {
                match &self.value {
                    $state::$variant { value, .. } => $view::$variant(value),
                    $state::Pending(value) => $view::Pending(value),
                }
            }
            pub fn components(&self) -> &CssComponentValues {
                match &self.value {
                    $state::$variant { components, .. } => components,
                    $state::Pending(value) => value.components(),
                }
            }
        }
    };
}

operand!(
    /// A signed exact length or a complete substitution-dependent operand.
    /// Typed calculations may contain container-context tree-counting functions;
    /// reconstruct them through the container boundary, not a pure numeric constructor.
    CssContainerLength, CssContainerLengthRef, ContainerLengthState, Numeric, CssLengthCalculation
);
operand!(
    /// An exact number pair, including degenerate ratios, or a pending whole ratio.
    CssContainerRatio, CssContainerRatioRef, ContainerRatioState, Numeric, CssMediaRatio
);
operand!(
    /// A discrete orientation keyword or a pending orientation operand.
    CssContainerOrientation, CssContainerOrientationRef, ContainerOrientationState, Keyword, CssOrientation
);
