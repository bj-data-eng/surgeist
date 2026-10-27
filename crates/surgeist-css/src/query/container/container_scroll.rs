//! Authored scroll-state queries. Snapshots, direction resolution and truth are external.
use crate::component::values::serialize::CssCanonicalBuilder;
use crate::component::{
    CssComponentValueError, CssComponentValues, CssSerializedValue, CssValueOrigin,
};
use crate::query::container::{
    CssContainerGeneralEnclosed, CssContainerPendingValue, CssContainerValueDomain,
};
use crate::source::CssSourcePosition;

/// A leaf of a scroll-state query: a scroll-state feature, or any other enclosure.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssScrollStateLeaf {
    /// A recognized scroll-state feature test, such as `stuck: top` or `scrollable`.
    Feature(CssContainerScrollFeature),
    /// A `<general-enclosed>` production that is not a recognized scroll-state feature.
    GeneralEnclosed(CssContainerGeneralEnclosed),
}

/// The four discrete features of a scroll-state query.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssContainerScrollFeatureKind {
    /// `stuck`: whether a sticky positioned container is shifted to stay inside an edge.
    Stuck,
    /// `snapped`: whether the container is, or would be, snapped to its scroll snap container.
    Snapped,
    /// `scrollable`: whether a scroll container has clipped scrollable overflow content in a direction that is reachable through user-initiated scrolling.
    Scrollable,
    /// `scrolled`: the direction of the container's most recent relative scroll.
    Scrolled,
}
impl CssContainerScrollFeatureKind {
    /// Returns the feature's lowercase CSS name, such as `scrollable`.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Stuck => "stuck",
            Self::Snapped => "snapped",
            Self::Scrollable => "scrollable",
            Self::Scrolled => "scrolled",
        }
    }
    pub(crate) fn parse(name: &str) -> Option<Self> {
        [Self::Stuck, Self::Snapped, Self::Scrollable, Self::Scrolled]
            .into_iter()
            .find(|kind| name.eq_ignore_ascii_case(kind.as_str()))
    }
}

/// Coupled feature/value domains; scrollable and scrolled share direction syntax.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssContainerScrollFeature {
    /// A feature name written without a value, such as `scrollable`.
    Boolean(CssContainerScrollFeatureKind),
    /// `stuck: <value>`, carrying a stuck-edge keyword or a pending operand.
    Stuck(CssContainerStuckValue),
    /// `snapped: <value>`, carrying a snapping-axis keyword or a pending operand.
    Snapped(CssContainerSnappedValue),
    /// `scrollable: <value>`, carrying a direction keyword or a pending operand.
    Scrollable(CssContainerScrollDirectionValue),
    /// `scrolled: <value>`, carrying a direction keyword or a pending operand.
    Scrolled(CssContainerScrollDirectionValue),
}

macro_rules! keywords {
    ($(#[$doc:meta])* $name:ident { $($(#[$vdoc:meta])* $variant:ident => $spelling:literal),+ $(,)? }) => {
        $(#[$doc])*
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        #[non_exhaustive]
        pub enum $name { $($(#[$vdoc])* $variant),+ }
        impl $name {
            /// Returns the keyword's lowercase CSS spelling.
            pub const fn as_str(self) -> &'static str {
                match self { $(Self::$variant => $spelling),+ }
            }
            pub(crate) fn parse(name: &str) -> Option<Self> {
                $(if name.eq_ignore_ascii_case($spelling) { return Some(Self::$variant); })+
                None
            }
        }
    };
}
keywords!(
    /// Authored stuck edges. Logical edges remain unresolved.
    CssContainerStuckKeyword {
        /// `none`: the sticky container is not shifted in any direction.
        None => "none",
        /// `top`: the container is shifted to stay inside the top edge.
        Top => "top",
        /// `right`: the container is shifted to stay inside the right edge.
        Right => "right",
        /// `bottom`: the container is shifted to stay inside the bottom edge.
        Bottom => "bottom",
        /// `left`: the container is shifted to stay inside the left edge.
        Left => "left",
        /// `block-start`: the container is shifted to stay inside the block-start edge.
        BlockStart => "block-start",
        /// `inline-start`: the container is shifted to stay inside the inline-start edge.
        InlineStart => "inline-start",
        /// `block-end`: the container is shifted to stay inside the block-end edge.
        BlockEnd => "block-end",
        /// `inline-end`: the container is shifted to stay inside the inline-end edge.
        InlineEnd => "inline-end",
    }
);
keywords!(
    /// Authored snapping axes, including simultaneous snapping on both axes.
    CssContainerSnappedKeyword {
        /// `none`: the container is not a snap target.
        None => "none",
        /// `x`: the container is a snap target in the horizontal direction.
        X => "x",
        /// `y`: the container is a snap target in the vertical direction.
        Y => "y",
        /// `block`: the container is a snap target in the block direction.
        Block => "block",
        /// `inline`: the container is a snap target in the inline direction.
        Inline => "inline",
        /// `both`: the container is a snap target in both directions.
        Both => "both",
    }
);
keywords!(
    /// Authored directions for scrollable and scrolled, without history or axis resolution.
    CssContainerScrollDirectionKeyword {
        /// `none`: no scrollable overflow in any direction (`scrollable`), or no relative scroll yet (`scrolled`).
        None => "none",
        /// `top`: scrollable overflow past the top edge (`scrollable`), or the latest relative scroll was toward it (`scrolled`).
        Top => "top",
        /// `right`: scrollable overflow past the right edge (`scrollable`), or the latest relative scroll was toward it (`scrolled`).
        Right => "right",
        /// `bottom`: scrollable overflow past the bottom edge (`scrollable`), or the latest relative scroll was toward it (`scrolled`).
        Bottom => "bottom",
        /// `left`: scrollable overflow past the left edge (`scrollable`), or the latest relative scroll was toward it (`scrolled`).
        Left => "left",
        /// `block-start`: scrollable overflow past the block-start edge (`scrollable`), or the latest relative scroll was toward it (`scrolled`).
        BlockStart => "block-start",
        /// `inline-start`: scrollable overflow past the inline-start edge (`scrollable`), or the latest relative scroll was toward it (`scrolled`).
        InlineStart => "inline-start",
        /// `block-end`: scrollable overflow past the block-end edge (`scrollable`), or the latest relative scroll was toward it (`scrolled`).
        BlockEnd => "block-end",
        /// `inline-end`: scrollable overflow past the inline-end edge (`scrollable`), or the latest relative scroll was toward it (`scrolled`).
        InlineEnd => "inline-end",
        /// `x`: horizontally scrollable overflow (`scrollable`), or a horizontal latest relative scroll (`scrolled`).
        X => "x",
        /// `y`: vertically scrollable overflow (`scrollable`), or a vertical latest relative scroll (`scrolled`).
        Y => "y",
        /// `block`: scrollable overflow in the block direction (`scrollable`), or a block-direction latest relative scroll (`scrolled`).
        Block => "block",
        /// `inline`: scrollable overflow in the inline direction (`scrollable`), or an inline-direction latest relative scroll (`scrolled`).
        Inline => "inline",
    }
);

#[derive(Clone, Debug, PartialEq)]
enum ScrollValue<K> {
    Keyword {
        keyword: K,
        components: CssComponentValues,
    },
    Pending(CssContainerPendingValue),
}
macro_rules! operand {
    ($(#[$doc:meta])* $name:ident, $view:ident, $keyword:ty, $domain:ident) => {
        $(#[$doc])*
        #[derive(Clone, Debug, PartialEq)]
        pub struct $name { value: ScrollValue<$keyword> }
        #[derive(Clone, Copy, Debug)]
        #[non_exhaustive]
        #[doc = concat!("Borrowed view of a [`", stringify!($name), "`]: either its keyword or its pending operand.")]
        pub enum $view<'a> {
            /// The admitted keyword.
            Keyword($keyword),
            /// A whole operand containing `var()` that awaits substitution into the feature's domain.
            Pending(&'a CssContainerPendingValue),
        }
        impl $name {
            pub(crate) fn typed(keyword: $keyword, components: CssComponentValues) -> Self {
                Self { value: ScrollValue::Keyword { keyword, components } }
            }
            pub(crate) fn pending(components: CssComponentValues) -> Self {
                Self { value: ScrollValue::Pending(CssContainerPendingValue::new(CssContainerValueDomain::$domain, components)) }
            }
            /// Returns the keyword, or the pending operand when the value awaits substitution.
            pub fn view(&self) -> $view<'_> {
                match &self.value {
                    ScrollValue::Keyword { keyword, .. } => $view::Keyword(*keyword),
                    ScrollValue::Pending(value) => $view::Pending(value),
                }
            }
            /// Returns the authored component values of the whole operand.
            pub fn components(&self) -> &CssComponentValues {
                match &self.value {
                    ScrollValue::Keyword { components, .. } => components,
                    ScrollValue::Pending(value) => value.components(),
                }
            }
            /// Returns the origin of the first non-trivia component of the operand.
            pub fn origin(&self) -> &CssValueOrigin {
                self.components().items().iter().find(|item| !crate::query::supports::construction::trivia(item))
                    .expect("admitted keyword or pending operand").origin()
            }
            /// Returns the source position of the operand, or `None` when its origin was not parsed from source.
            pub fn position(&self) -> Option<CssSourcePosition> {
                crate::query::media::construction::parsed_position(self.origin())
            }
            /// `limits.max_css_bytes()` caps the output, including inserted syntax; the other
            /// component limits do not apply.
            pub fn serialize_tokens(
                &self,
                limits: crate::component::CssComponentValueLimits,
            ) -> Result<CssSerializedValue, CssComponentValueError> {
                let max_css_bytes = limits.max_css_bytes();
                let mut out = CssCanonicalBuilder::new(max_css_bytes);
                out.push_components(self.components().items())?;
                out.finish()
            }
        }
    };
}
operand!(
    /// A stuck keyword or a whole value awaiting substitution into that domain.
    CssContainerStuckValue, CssContainerStuckValueRef, CssContainerStuckKeyword, Stuck
);
operand!(
    /// A snapped keyword or a whole value awaiting substitution into that domain.
    CssContainerSnappedValue, CssContainerSnappedValueRef, CssContainerSnappedKeyword, Snapped
);
operand!(
    /// A direction keyword or a whole value awaiting substitution into that domain.
    CssContainerScrollDirectionValue, CssContainerScrollDirectionValueRef, CssContainerScrollDirectionKeyword, ScrollDirection
);
