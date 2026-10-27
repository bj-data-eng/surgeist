//! Authored container-feature values. Contextual substitution and matching are external.
use crate::component::values::serialize::CssCanonicalBuilder;
use crate::component::{CssComponentValueError, CssComponentValues, CssSerializedValue};
use crate::numeric::{CssCalculation, CssLength};
use crate::query::media::{CssMediaRatio, CssOrientation};

/// The six size features defined by the selected Conditional Rules grammar.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum CssContainerSizeFeatureKind {
    /// `width`: the width of the query container's content box.
    Width,
    /// `height`: the height of the query container's content box.
    Height,
    /// `inline-size`: the size of the query container's content box in the container's inline axis.
    InlineSize,
    /// `block-size`: the size of the query container's content box in the container's block axis.
    BlockSize,
    /// `aspect-ratio`: the ratio of the value of the container `width` feature to the value of the container `height` feature.
    AspectRatio,
    /// `orientation`: `portrait` when the container `height` feature is at least the container `width` feature, otherwise `landscape`.
    Orientation,
}
impl CssContainerSizeFeatureKind {
    /// Returns the feature's lowercase CSS name, such as `inline-size`.
    pub const fn as_str(self) -> &'static str {
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
    /// A `<length>`, as used by the `width`, `height`, `inline-size` and `block-size` features.
    Length,
    /// A `<ratio>`, as used by the `aspect-ratio` feature.
    Ratio,
    /// An orientation keyword, as used by the `orientation` feature.
    Orientation,
    /// A `stuck` scroll-state keyword.
    Stuck,
    /// A `snapped` scroll-state keyword.
    Snapped,
    /// A scroll-direction keyword, as used by the `scrollable` and `scrolled` scroll-state features.
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
    /// Returns the grammar the substituted value must match.
    pub const fn domain(&self) -> CssContainerValueDomain {
        self.domain
    }
    /// Returns the authored component values of the whole operand, including its `var()` references.
    pub fn components(&self) -> &CssComponentValues {
        &self.components
    }
    /// `limits.max_css_bytes()` caps the output, including inserted syntax; the other component
    /// limits do not apply.
    pub fn serialize_tokens(
        &self,
        limits: crate::component::CssComponentValueLimits,
    ) -> Result<CssSerializedValue, CssComponentValueError> {
        let max_css_bytes = limits.max_css_bytes();
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
        #[doc = concat!("Borrowed view of a [`", stringify!($name), "`]: either its typed value or its pending operand.")]
        pub enum $view<'a> {
            /// The typed value that was admitted without `var()`.
            $variant(&'a $value),
            /// A whole operand containing `var()` that awaits substitution into the feature's domain.
            Pending(&'a CssContainerPendingValue),
        }
        impl $name {
            pub(crate) fn typed(value: $value, components: CssComponentValues) -> Self {
                Self { value: $state::$variant { value, components } }
            }
            pub(crate) fn pending(value: CssContainerPendingValue) -> Self {
                Self { value: $state::Pending(value) }
            }
            /// Returns the typed value, or the pending operand when the value awaits substitution.
            pub fn view(&self) -> $view<'_> {
                match &self.value {
                    $state::$variant { value, .. } => $view::$variant(value),
                    $state::Pending(value) => $view::Pending(value),
                }
            }
            /// Returns the authored component values of the whole operand.
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
    CssContainerLength, CssContainerLengthRef, ContainerLengthState, Numeric, CssCalculation<CssLength>
);
operand!(
    /// An exact number pair, including degenerate ratios, or a pending whole ratio.
    CssContainerRatio, CssContainerRatioRef, ContainerRatioState, Numeric, CssMediaRatio
);
operand!(
    /// A discrete orientation keyword or a pending orientation operand.
    CssContainerOrientation, CssContainerOrientationRef, ContainerOrientationState, Keyword, CssOrientation
);
