//! Authored media feature domains; contextual matching and calculation resolution are external.
use crate::component::{CssComponentValue, CssComponentValues};
use crate::numeric::{CssCalculation, CssInteger, CssLength, CssNumber, CssResolution};
use crate::query::CssQueryComparison;
use crate::query::media::{
    CssColorSchemePreference, CssContrastPreference, CssDisplayMode, CssForcedColorsMode,
    CssGridMode, CssHoverCapability, CssOrientation, CssPointerCapability,
    CssReducedMotionPreference, CssReducedTransparencyPreference, CssScanMode,
};

/// An authored range, preserving source order and the selected grammar form.
#[derive(Clone, Debug, PartialEq)]
pub struct CssMediaRange<T> {
    state: MediaRangeState<T>,
}
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum MediaRangeState<T> {
    Plain {
        value: T,
    },
    Min {
        value: T,
    },
    Max {
        value: T,
    },
    FeatureFirst {
        comparison: CssQueryComparison,
        value: T,
    },
    ValueFirst {
        value: T,
        comparison: CssQueryComparison,
    },
    Ascending {
        left: T,
        left_inclusive: bool,
        right: T,
        right_inclusive: bool,
    },
    Descending {
        left: T,
        left_inclusive: bool,
        right: T,
        right_inclusive: bool,
    },
}
/// Borrowed authored range state. Bounds are neither reordered nor evaluated.
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssMediaRangeRef<'a, T> {
    /// A plain value, as in `(width: 600px)`.
    Plain {
        /// The authored value.
        value: &'a T,
    },
    /// A `min-` prefixed feature, as in `(min-width: 600px)`.
    Min {
        /// The authored lower bound.
        value: &'a T,
    },
    /// A `max-` prefixed feature, as in `(max-width: 600px)`.
    Max {
        /// The authored upper bound.
        value: &'a T,
    },
    /// A feature followed by a comparison, as in `(width >= 600px)`.
    FeatureFirst {
        /// The comparison as written, with the feature on its left.
        comparison: CssQueryComparison,
        /// The authored value on the right of the comparison.
        value: &'a T,
    },
    /// A value followed by a comparison and the feature, as in `(600px <= width)`.
    ValueFirst {
        /// The authored value on the left of the comparison.
        value: &'a T,
        /// The comparison as written, with the feature on its right.
        comparison: CssQueryComparison,
    },
    /// An interval whose bounds ascend through the feature, as in `(400px <= width < 700px)`.
    Ascending {
        /// The lower bound, written first.
        left: &'a T,
        /// Whether the comparison between the lower bound and the feature is `<=` rather than `<`.
        left_inclusive: bool,
        /// The upper bound, written last.
        right: &'a T,
        /// Whether the comparison between the feature and the upper bound is `<=` rather than `<`.
        right_inclusive: bool,
    },
    /// An interval whose bounds descend through the feature, as in `(700px > width >= 400px)`.
    Descending {
        /// The upper bound, written first.
        left: &'a T,
        /// Whether the comparison between the upper bound and the feature is `>=` rather than `>`.
        left_inclusive: bool,
        /// The lower bound, written last.
        right: &'a T,
        /// Whether the comparison between the feature and the lower bound is `>=` rather than `>`.
        right_inclusive: bool,
    },
}
impl<T> CssMediaRange<T> {
    pub(crate) fn new(state: MediaRangeState<T>) -> Self {
        Self { state }
    }
    /// Returns the range as a borrowed view that preserves the authored form, bound order and comparison operators.
    pub fn view(&self) -> CssMediaRangeRef<'_, T> {
        match &self.state {
            MediaRangeState::Plain { value } => CssMediaRangeRef::Plain { value },
            MediaRangeState::Min { value } => CssMediaRangeRef::Min { value },
            MediaRangeState::Max { value } => CssMediaRangeRef::Max { value },
            MediaRangeState::FeatureFirst { comparison, value } => CssMediaRangeRef::FeatureFirst {
                comparison: *comparison,
                value,
            },
            MediaRangeState::ValueFirst { value, comparison } => CssMediaRangeRef::ValueFirst {
                value,
                comparison: *comparison,
            },
            MediaRangeState::Ascending {
                left,
                left_inclusive,
                right,
                right_inclusive,
            } => CssMediaRangeRef::Ascending {
                left,
                left_inclusive: *left_inclusive,
                right,
                right_inclusive: *right_inclusive,
            },
            MediaRangeState::Descending {
                left,
                left_inclusive,
                right,
                right_inclusive,
            } => CssMediaRangeRef::Descending {
                left,
                left_inclusive: *left_inclusive,
                right,
                right_inclusive: *right_inclusive,
            },
        }
    }
}
/// A signed authored media length, retaining its exact numeric components.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssMediaLength {
    calculation: CssCalculation<CssLength>,
}
impl CssMediaLength {
    pub(crate) fn new(calculation: CssCalculation<CssLength>) -> Self {
        Self { calculation }
    }
    /// Returns the authored length calculation, which may be a literal `<length>`.
    pub fn calculation(&self) -> &CssCalculation<CssLength> {
        &self.calculation
    }
    /// Returns the authored component values of the length.
    pub fn components(&self) -> &CssComponentValues {
        self.calculation.components()
    }
}
/// A signed authored media integer, retaining its exact numeric components.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssMediaInteger {
    calculation: CssCalculation<CssInteger>,
}
impl CssMediaInteger {
    pub(crate) fn new(calculation: CssCalculation<CssInteger>) -> Self {
        Self { calculation }
    }
    /// Returns the authored integer calculation, which may be a literal `<integer>`.
    pub fn calculation(&self) -> &CssCalculation<CssInteger> {
        &self.calculation
    }
    /// Returns the authored component values of the integer.
    pub fn components(&self) -> &CssComponentValues {
        self.calculation.components()
    }
}
/// Signed resolution syntax or the distinct `infinite` keyword.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssMediaResolution {
    value: MediaResolutionValue,
}
#[derive(Clone, Debug, Eq, PartialEq)]
enum MediaResolutionValue {
    Numeric(CssCalculation<CssResolution>),
    Infinite(Box<CssComponentValue>),
}
/// Borrowed view of a [`CssMediaResolution`]: a `<resolution>` or the `infinite` keyword.
#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
pub enum CssMediaResolutionRef<'a> {
    /// A `<resolution>` value or calculation.
    Numeric(&'a CssCalculation<CssResolution>),
    /// The `infinite` keyword component.
    Infinite(&'a CssComponentValue),
}
impl CssMediaResolution {
    pub(crate) fn numeric(value: CssCalculation<CssResolution>) -> Self {
        Self {
            value: MediaResolutionValue::Numeric(value),
        }
    }
    pub(crate) fn infinite(value: CssComponentValue) -> Self {
        Self {
            value: MediaResolutionValue::Infinite(Box::new(value)),
        }
    }
    /// Returns the resolution as either a numeric calculation or the `infinite` keyword.
    pub fn view(&self) -> CssMediaResolutionRef<'_> {
        match &self.value {
            MediaResolutionValue::Numeric(v) => CssMediaResolutionRef::Numeric(v),
            MediaResolutionValue::Infinite(v) => CssMediaResolutionRef::Infinite(v),
        }
    }
}
/// A pair of authored number values with a nonnegative constraint on each operand.
/// Calculations retain that constraint for resolution; literal operands are already checked.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssMediaRatio {
    numerator: CssCalculation<CssNumber>,
    denominator: CssCalculation<CssNumber>,
    denominator_is_omitted: bool,
}
impl CssMediaRatio {
    pub(crate) fn new(
        numerator: CssCalculation<CssNumber>,
        denominator: CssCalculation<CssNumber>,
        denominator_is_omitted: bool,
    ) -> Self {
        Self {
            numerator,
            denominator,
            denominator_is_omitted,
        }
    }
    /// Returns the authored numerator calculation.
    pub fn numerator(&self) -> &CssCalculation<CssNumber> {
        &self.numerator
    }
    /// Returns the authored denominator calculation, which is the number `1` when the denominator was omitted.
    pub fn denominator(&self) -> &CssCalculation<CssNumber> {
        &self.denominator
    }
    /// Returns whether the `/ <number>` denominator was omitted in the authored `<ratio>`.
    pub const fn denominator_is_omitted(&self) -> bool {
        self.denominator_is_omitted
    }
}
/// An authored grid value with a deferred closed `[0,1]` integer constraint.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssMediaGrid {
    calculation: CssCalculation<CssInteger>,
    literal: Option<CssGridMode>,
}
/// Borrowed view of a [`CssMediaGrid`]: a literal grid mode or an integer calculation.
#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
pub enum CssMediaGridRef<'a> {
    /// A literal `0` or `1` value, read as a grid mode.
    Literal(CssGridMode),
    /// A calculation or other integer whose value is checked against the `[0,1]` range later.
    Calculation(&'a CssCalculation<CssInteger>),
}
impl CssMediaGrid {
    pub(crate) fn new(
        calculation: CssCalculation<CssInteger>,
        literal: Option<CssGridMode>,
    ) -> Self {
        Self {
            calculation,
            literal,
        }
    }
    /// Returns the literal grid mode when the value is a plain `0` or `1`, and otherwise the integer calculation.
    pub fn view(&self) -> CssMediaGridRef<'_> {
        match self.literal {
            Some(v) => CssMediaGridRef::Literal(v),
            None => CssMediaGridRef::Calculation(&self.calculation),
        }
    }
    /// Returns the authored component values of the grid value.
    pub fn components(&self) -> &CssComponentValues {
        self.calculation.components()
    }
}
/// The `update` media feature: the ability of the output device to modify the appearance of content once it has been rendered.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssMediaUpdate {
    /// `none`: once rendered, the layout can no longer be updated, such as printed paper.
    None,
    /// `slow`: the layout may change, but the device cannot display changes quickly enough for smooth animation.
    Slow,
    /// `fast`: the layout may change and the device can display changes quickly enough for animation.
    Fast,
}
/// The `overflow-block` media feature: the behavior of the device when content overflows the initial containing block in the block axis.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssMediaOverflowBlock {
    /// `none`: there is no affordance for block-axis overflow, so overflowing content is not displayed.
    None,
    /// `scroll`: block-axis overflow is exposed by letting users scroll to it.
    Scroll,
    /// `paged`: content is broken into discrete pages, and block-axis overflow continues on the following page.
    Paged,
}
/// The `overflow-inline` media feature: the behavior of the device when content overflows the initial containing block in the inline axis.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssMediaOverflowInline {
    /// `none`: there is no affordance for inline-axis overflow, so overflowing content is not displayed.
    None,
    /// `scroll`: inline-axis overflow is exposed by letting users scroll to it.
    Scroll,
}
/// The `color-gamut` and `video-color-gamut` media features: the approximate range of colors that the user agent and output device support.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssMediaColorGamut {
    /// `srgb`: approximately the sRGB gamut or more.
    Srgb,
    /// `p3`: approximately the Display P3 gamut or more.
    P3,
    /// `rec2020`: approximately the ITU-R BT.2020 gamut or more.
    Rec2020,
}
/// The `dynamic-range` and `video-dynamic-range` media features: the combination of maximum brightness, color depth and contrast ratio that the user agent and output device support.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssMediaDynamicRange {
    /// `standard`: matches any visual device.
    Standard,
    /// `high`: the device supports high peak brightness, a high contrast ratio and more than 8 bits per RGB component.
    High,
}
/// The `environment-blending` media feature: how the display blends the canvas with the real world.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssMediaEnvironmentBlending {
    /// `opaque`: an opaque medium, such as a traditional monitor or paper, where black is dark and white is fully light.
    Opaque,
    /// `additive`: additive mixing with the real world, where black is fully transparent and white is fully light.
    Additive,
    /// `subtractive`: subtractive mixing with the real world, where white is fully transparent and dark colors have the most contrast.
    Subtractive,
}
/// The `inverted-colors` media feature: whether the displayed colors are inverted.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssMediaInvertedColors {
    /// `none`: colors are displayed normally.
    None,
    /// `inverted`: all pixels in the displayed area have been inverted.
    Inverted,
}
/// The `nav-controls` media feature: whether the user agent provides obviously discoverable navigation controls.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssMediaNavigationControls {
    /// `none`: there are no obviously discoverable navigation controls, in particular none that move back to the previous session history entry.
    None,
    /// `back`: the user agent provides navigation controls, including an obviously discoverable control that moves back to the previous session history entry.
    Back,
}
/// The `scripting` media feature: whether scripting languages are supported for the current document.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssMediaScripting {
    /// `none`: scripts are not run for the document.
    None,
    /// `initial-only`: scripting is enabled during the initial page load but not afterwards.
    InitialOnly,
    /// `enabled`: scripting is enabled for the lifetime of the document.
    Enabled,
}
/// The `prefers-reduced-data` media feature: whether the user prefers lightweight alternate content.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssMediaReducedDataPreference {
    /// `no-preference`: the user has made no preference known; this evaluates as false in the boolean context.
    NoPreference,
    /// `reduce`: the user has expressed a preference for lightweight alternate content.
    Reduce,
}
/// Every known feature in the selected Media Queries 5 descriptor index.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum CssMediaFeatureKind {
    /// `width`: the width of the targeted display area of the output device: the viewport for continuous media, the page box for paged media.
    Width,
    /// `height`: the height of the targeted display area of the output device: the viewport for continuous media, the page box for paged media.
    Height,
    /// `device-width`: the width of the output device's rendering surface.
    DeviceWidth,
    /// `device-height`: the height of the output device's rendering surface.
    DeviceHeight,
    /// `aspect-ratio`: the ratio of the value of the `width` feature to the value of the `height` feature.
    AspectRatio,
    /// `device-aspect-ratio`: the ratio of the value of the `device-width` feature to the value of the `device-height` feature.
    DeviceAspectRatio,
    /// `resolution`: the pixel density of the output device.
    Resolution,
    /// `color`: the number of bits per color component of the output device.
    Color,
    /// `color-index`: the number of entries in the output device's color lookup table.
    ColorIndex,
    /// `monochrome`: the number of bits per pixel in the output device's monochrome frame buffer.
    Monochrome,
    /// `horizontal-viewport-segments`: the number of logical segments of the viewport in the horizontal direction.
    HorizontalViewportSegments,
    /// `vertical-viewport-segments`: the number of logical segments of the viewport in the vertical direction.
    VerticalViewportSegments,
    /// `orientation`: `portrait` when the value of the `height` feature is at least the value of the `width` feature, otherwise `landscape`.
    Orientation,
    /// `scan`: the scanning process of some output devices.
    Scan,
    /// `grid`: whether the output device is a grid device rather than a bitmap device.
    Grid,
    /// `prefers-color-scheme`: the user's light or dark color-scheme preference.
    PrefersColorScheme,
    /// `prefers-reduced-motion`: whether the user prefers less motion.
    PrefersReducedMotion,
    /// `prefers-reduced-transparency`: whether the user prefers less transparency.
    PrefersReducedTransparency,
    /// `prefers-contrast`: whether the user prefers more or less contrast.
    PrefersContrast,
    /// `forced-colors`: whether the user agent restricts the color palette.
    ForcedColors,
    /// `hover`: whether the primary pointing device can hover.
    Hover,
    /// `any-hover`: whether any available pointing device can hover.
    AnyHover,
    /// `pointer`: the presence and accuracy of the primary pointing device.
    Pointer,
    /// `any-pointer`: the presence and accuracy of the available pointing devices, of which more than one value can match.
    AnyPointer,
    /// `display-mode`: the mode in which the current browsing context is presented to the end user.
    DisplayMode,
    /// `update`: the ability of the output device to modify the appearance of content once it has been rendered.
    Update,
    /// `overflow-block`: the behavior of the device when content overflows the initial containing block in the block axis.
    OverflowBlock,
    /// `overflow-inline`: the behavior of the device when content overflows the initial containing block in the inline axis.
    OverflowInline,
    /// `color-gamut`: the approximate range of colors that the user agent and output device support.
    ColorGamut,
    /// `video-color-gamut`: the approximate range of colors that the user agent and output device support for the video plane.
    VideoColorGamut,
    /// `dynamic-range`: the combination of maximum brightness, color depth and contrast ratio that the user agent and output device support.
    DynamicRange,
    /// `video-dynamic-range`: the combination of maximum brightness, color depth and contrast ratio that the user agent and output device support for the video plane.
    VideoDynamicRange,
    /// `environment-blending`: how the display blends the canvas with the real world.
    EnvironmentBlending,
    /// `inverted-colors`: whether the displayed colors are inverted.
    InvertedColors,
    /// `nav-controls`: whether the user agent provides obviously discoverable navigation controls.
    NavControls,
    /// `scripting`: whether scripting languages are supported for the current document.
    Scripting,
    /// `prefers-reduced-data`: whether the user prefers lightweight alternate content.
    PrefersReducedData,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum MediaValueFamily {
    Length,
    Integer,
    Ratio,
    Resolution,
    Discrete,
}
// The single name/type catalog is shared by boolean, plain, and range admission.
const FEATURES: &[(CssMediaFeatureKind, &str, MediaValueFamily)] = &[
    (
        CssMediaFeatureKind::Width,
        "width",
        MediaValueFamily::Length,
    ),
    (
        CssMediaFeatureKind::Height,
        "height",
        MediaValueFamily::Length,
    ),
    (
        CssMediaFeatureKind::DeviceWidth,
        "device-width",
        MediaValueFamily::Length,
    ),
    (
        CssMediaFeatureKind::DeviceHeight,
        "device-height",
        MediaValueFamily::Length,
    ),
    (
        CssMediaFeatureKind::AspectRatio,
        "aspect-ratio",
        MediaValueFamily::Ratio,
    ),
    (
        CssMediaFeatureKind::DeviceAspectRatio,
        "device-aspect-ratio",
        MediaValueFamily::Ratio,
    ),
    (
        CssMediaFeatureKind::Resolution,
        "resolution",
        MediaValueFamily::Resolution,
    ),
    (
        CssMediaFeatureKind::Color,
        "color",
        MediaValueFamily::Integer,
    ),
    (
        CssMediaFeatureKind::ColorIndex,
        "color-index",
        MediaValueFamily::Integer,
    ),
    (
        CssMediaFeatureKind::Monochrome,
        "monochrome",
        MediaValueFamily::Integer,
    ),
    (
        CssMediaFeatureKind::HorizontalViewportSegments,
        "horizontal-viewport-segments",
        MediaValueFamily::Integer,
    ),
    (
        CssMediaFeatureKind::VerticalViewportSegments,
        "vertical-viewport-segments",
        MediaValueFamily::Integer,
    ),
    (
        CssMediaFeatureKind::Orientation,
        "orientation",
        MediaValueFamily::Discrete,
    ),
    (
        CssMediaFeatureKind::Scan,
        "scan",
        MediaValueFamily::Discrete,
    ),
    (
        CssMediaFeatureKind::Grid,
        "grid",
        MediaValueFamily::Discrete,
    ),
    (
        CssMediaFeatureKind::PrefersColorScheme,
        "prefers-color-scheme",
        MediaValueFamily::Discrete,
    ),
    (
        CssMediaFeatureKind::PrefersReducedMotion,
        "prefers-reduced-motion",
        MediaValueFamily::Discrete,
    ),
    (
        CssMediaFeatureKind::PrefersReducedTransparency,
        "prefers-reduced-transparency",
        MediaValueFamily::Discrete,
    ),
    (
        CssMediaFeatureKind::PrefersContrast,
        "prefers-contrast",
        MediaValueFamily::Discrete,
    ),
    (
        CssMediaFeatureKind::ForcedColors,
        "forced-colors",
        MediaValueFamily::Discrete,
    ),
    (
        CssMediaFeatureKind::Hover,
        "hover",
        MediaValueFamily::Discrete,
    ),
    (
        CssMediaFeatureKind::AnyHover,
        "any-hover",
        MediaValueFamily::Discrete,
    ),
    (
        CssMediaFeatureKind::Pointer,
        "pointer",
        MediaValueFamily::Discrete,
    ),
    (
        CssMediaFeatureKind::AnyPointer,
        "any-pointer",
        MediaValueFamily::Discrete,
    ),
    (
        CssMediaFeatureKind::DisplayMode,
        "display-mode",
        MediaValueFamily::Discrete,
    ),
    (
        CssMediaFeatureKind::Update,
        "update",
        MediaValueFamily::Discrete,
    ),
    (
        CssMediaFeatureKind::OverflowBlock,
        "overflow-block",
        MediaValueFamily::Discrete,
    ),
    (
        CssMediaFeatureKind::OverflowInline,
        "overflow-inline",
        MediaValueFamily::Discrete,
    ),
    (
        CssMediaFeatureKind::ColorGamut,
        "color-gamut",
        MediaValueFamily::Discrete,
    ),
    (
        CssMediaFeatureKind::VideoColorGamut,
        "video-color-gamut",
        MediaValueFamily::Discrete,
    ),
    (
        CssMediaFeatureKind::DynamicRange,
        "dynamic-range",
        MediaValueFamily::Discrete,
    ),
    (
        CssMediaFeatureKind::VideoDynamicRange,
        "video-dynamic-range",
        MediaValueFamily::Discrete,
    ),
    (
        CssMediaFeatureKind::EnvironmentBlending,
        "environment-blending",
        MediaValueFamily::Discrete,
    ),
    (
        CssMediaFeatureKind::InvertedColors,
        "inverted-colors",
        MediaValueFamily::Discrete,
    ),
    (
        CssMediaFeatureKind::NavControls,
        "nav-controls",
        MediaValueFamily::Discrete,
    ),
    (
        CssMediaFeatureKind::Scripting,
        "scripting",
        MediaValueFamily::Discrete,
    ),
    (
        CssMediaFeatureKind::PrefersReducedData,
        "prefers-reduced-data",
        MediaValueFamily::Discrete,
    ),
];
impl CssMediaFeatureKind {
    /// Returns the feature's lowercase CSS name, such as `prefers-color-scheme`.
    pub const fn as_str(self) -> &'static str {
        let mut i = 0;
        while i < FEATURES.len() {
            if self as usize == FEATURES[i].0 as usize {
                return FEATURES[i].1;
            }
            i += 1;
        }
        unreachable!()
    }
    pub(crate) fn from_name(name: &str) -> Option<Self> {
        FEATURES
            .iter()
            .find(|entry| entry.1.eq_ignore_ascii_case(name))
            .map(|entry| entry.0)
    }
    pub(crate) fn family(self) -> MediaValueFamily {
        FEATURES
            .iter()
            .find(|entry| entry.0 == self)
            .expect("complete media feature catalog")
            .2
    }
}
/// An authored media feature test, without evaluation against a device.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssMediaFeatureQuery {
    /// A feature name alone, such as `(color)`, tested in the boolean context.
    Boolean(CssMediaFeatureKind),
    /// `width`, as a plain, `min-`/`max-` prefixed, comparison or interval range.
    Width(CssMediaRange<CssMediaLength>),
    /// `height`, as a plain, `min-`/`max-` prefixed, comparison or interval range.
    Height(CssMediaRange<CssMediaLength>),
    /// `device-width`, as a plain, `min-`/`max-` prefixed, comparison or interval range.
    DeviceWidth(CssMediaRange<CssMediaLength>),
    /// `device-height`, as a plain, `min-`/`max-` prefixed, comparison or interval range.
    DeviceHeight(CssMediaRange<CssMediaLength>),
    /// `aspect-ratio`, as a plain, `min-`/`max-` prefixed, comparison or interval range of `<ratio>` values.
    AspectRatio(CssMediaRange<CssMediaRatio>),
    /// `device-aspect-ratio`, as a plain, `min-`/`max-` prefixed, comparison or interval range of `<ratio>` values.
    DeviceAspectRatio(CssMediaRange<CssMediaRatio>),
    /// `resolution`, as a plain, `min-`/`max-` prefixed, comparison or interval range of `<resolution>` values or `infinite`.
    Resolution(CssMediaRange<CssMediaResolution>),
    /// `color`, as a plain, `min-`/`max-` prefixed, comparison or interval range of integers.
    Color(CssMediaRange<CssMediaInteger>),
    /// `color-index`, as a plain, `min-`/`max-` prefixed, comparison or interval range of integers.
    ColorIndex(CssMediaRange<CssMediaInteger>),
    /// `monochrome`, as a plain, `min-`/`max-` prefixed, comparison or interval range of integers.
    Monochrome(CssMediaRange<CssMediaInteger>),
    /// `horizontal-viewport-segments`, as a plain, `min-`/`max-` prefixed, comparison or interval range of integers.
    HorizontalViewportSegments(CssMediaRange<CssMediaInteger>),
    /// `vertical-viewport-segments`, as a plain, `min-`/`max-` prefixed, comparison or interval range of integers.
    VerticalViewportSegments(CssMediaRange<CssMediaInteger>),
    /// `orientation: <keyword>`.
    Orientation(CssOrientation),
    /// `scan: <keyword>`.
    Scan(CssScanMode),
    /// `grid: <value>`, a `0` or `1` grid value or a calculation.
    Grid(CssMediaGrid),
    /// `prefers-color-scheme: <keyword>`.
    PrefersColorScheme(CssColorSchemePreference),
    /// `prefers-reduced-motion: <keyword>`.
    PrefersReducedMotion(CssReducedMotionPreference),
    /// `prefers-reduced-transparency: <keyword>`.
    PrefersReducedTransparency(CssReducedTransparencyPreference),
    /// `prefers-contrast: <keyword>`.
    PrefersContrast(CssContrastPreference),
    /// `forced-colors: <keyword>`.
    ForcedColors(CssForcedColorsMode),
    /// `hover: <keyword>`.
    Hover(CssHoverCapability),
    /// `any-hover: <keyword>`.
    AnyHover(CssHoverCapability),
    /// `pointer: <keyword>`.
    Pointer(CssPointerCapability),
    /// `any-pointer: <keyword>`.
    AnyPointer(CssPointerCapability),
    /// `display-mode: <keyword>`.
    DisplayMode(CssDisplayMode),
    /// `update: <keyword>`.
    Update(CssMediaUpdate),
    /// `overflow-block: <keyword>`.
    OverflowBlock(CssMediaOverflowBlock),
    /// `overflow-inline: <keyword>`.
    OverflowInline(CssMediaOverflowInline),
    /// `color-gamut: <keyword>`.
    ColorGamut(CssMediaColorGamut),
    /// `video-color-gamut: <keyword>`.
    VideoColorGamut(CssMediaColorGamut),
    /// `dynamic-range: <keyword>`.
    DynamicRange(CssMediaDynamicRange),
    /// `video-dynamic-range: <keyword>`.
    VideoDynamicRange(CssMediaDynamicRange),
    /// `environment-blending: <keyword>`.
    EnvironmentBlending(CssMediaEnvironmentBlending),
    /// `inverted-colors: <keyword>`.
    InvertedColors(CssMediaInvertedColors),
    /// `nav-controls: <keyword>`.
    NavControls(CssMediaNavigationControls),
    /// `scripting: <keyword>`.
    Scripting(CssMediaScripting),
    /// `prefers-reduced-data: <keyword>`.
    PrefersReducedData(CssMediaReducedDataPreference),
}
impl CssMediaFeatureQuery {
    /// Returns the feature's lowercase CSS name, such as `prefers-color-scheme`.
    pub const fn as_str(&self) -> &'static str {
        self.kind().as_str()
    }
    /// Returns the kind of the feature, which is the catalog entry for the name this query carries.
    pub const fn kind(&self) -> CssMediaFeatureKind {
        match self {
            Self::Boolean(id) => *id,
            Self::Width(..) => CssMediaFeatureKind::Width,
            Self::Height(..) => CssMediaFeatureKind::Height,
            Self::DeviceWidth(..) => CssMediaFeatureKind::DeviceWidth,
            Self::DeviceHeight(..) => CssMediaFeatureKind::DeviceHeight,
            Self::AspectRatio(..) => CssMediaFeatureKind::AspectRatio,
            Self::DeviceAspectRatio(..) => CssMediaFeatureKind::DeviceAspectRatio,
            Self::Resolution(..) => CssMediaFeatureKind::Resolution,
            Self::Color(..) => CssMediaFeatureKind::Color,
            Self::ColorIndex(..) => CssMediaFeatureKind::ColorIndex,
            Self::Monochrome(..) => CssMediaFeatureKind::Monochrome,
            Self::HorizontalViewportSegments(..) => CssMediaFeatureKind::HorizontalViewportSegments,
            Self::VerticalViewportSegments(..) => CssMediaFeatureKind::VerticalViewportSegments,
            Self::Orientation(..) => CssMediaFeatureKind::Orientation,
            Self::Scan(..) => CssMediaFeatureKind::Scan,
            Self::Grid(..) => CssMediaFeatureKind::Grid,
            Self::PrefersColorScheme(..) => CssMediaFeatureKind::PrefersColorScheme,
            Self::PrefersReducedMotion(..) => CssMediaFeatureKind::PrefersReducedMotion,
            Self::PrefersReducedTransparency(..) => CssMediaFeatureKind::PrefersReducedTransparency,
            Self::PrefersContrast(..) => CssMediaFeatureKind::PrefersContrast,
            Self::ForcedColors(..) => CssMediaFeatureKind::ForcedColors,
            Self::Hover(..) => CssMediaFeatureKind::Hover,
            Self::AnyHover(..) => CssMediaFeatureKind::AnyHover,
            Self::Pointer(..) => CssMediaFeatureKind::Pointer,
            Self::AnyPointer(..) => CssMediaFeatureKind::AnyPointer,
            Self::DisplayMode(..) => CssMediaFeatureKind::DisplayMode,
            Self::Update(..) => CssMediaFeatureKind::Update,
            Self::OverflowBlock(..) => CssMediaFeatureKind::OverflowBlock,
            Self::OverflowInline(..) => CssMediaFeatureKind::OverflowInline,
            Self::ColorGamut(..) => CssMediaFeatureKind::ColorGamut,
            Self::VideoColorGamut(..) => CssMediaFeatureKind::VideoColorGamut,
            Self::DynamicRange(..) => CssMediaFeatureKind::DynamicRange,
            Self::VideoDynamicRange(..) => CssMediaFeatureKind::VideoDynamicRange,
            Self::EnvironmentBlending(..) => CssMediaFeatureKind::EnvironmentBlending,
            Self::InvertedColors(..) => CssMediaFeatureKind::InvertedColors,
            Self::NavControls(..) => CssMediaFeatureKind::NavControls,
            Self::Scripting(..) => CssMediaFeatureKind::Scripting,
            Self::PrefersReducedData(..) => CssMediaFeatureKind::PrefersReducedData,
        }
    }
}
