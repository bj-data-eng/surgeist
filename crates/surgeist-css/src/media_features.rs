//! Authored media feature domains; contextual matching and calculation resolution are external.
use crate::*;

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
    Plain {
        value: &'a T,
    },
    Min {
        value: &'a T,
    },
    Max {
        value: &'a T,
    },
    FeatureFirst {
        comparison: CssQueryComparison,
        value: &'a T,
    },
    ValueFirst {
        value: &'a T,
        comparison: CssQueryComparison,
    },
    Ascending {
        left: &'a T,
        left_inclusive: bool,
        right: &'a T,
        right_inclusive: bool,
    },
    Descending {
        left: &'a T,
        left_inclusive: bool,
        right: &'a T,
        right_inclusive: bool,
    },
}
impl<T> CssMediaRange<T> {
    pub(crate) fn new(state: MediaRangeState<T>) -> Self {
        Self { state }
    }
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
    calculation: CssLengthCalculation,
}
impl CssMediaLength {
    pub(crate) fn new(calculation: CssLengthCalculation) -> Self {
        Self { calculation }
    }
    pub fn calculation(&self) -> &CssLengthCalculation {
        &self.calculation
    }
    pub fn components(&self) -> &CssComponentValues {
        self.calculation.components()
    }
}
/// A signed authored media integer, retaining its exact numeric components.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssMediaInteger {
    calculation: CssIntegerCalculation,
}
impl CssMediaInteger {
    pub(crate) fn new(calculation: CssIntegerCalculation) -> Self {
        Self { calculation }
    }
    pub fn calculation(&self) -> &CssIntegerCalculation {
        &self.calculation
    }
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
    Numeric(CssResolutionCalculation),
    Infinite(Box<CssComponentValue>),
}
#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
pub enum CssMediaResolutionRef<'a> {
    Numeric(&'a CssResolutionCalculation),
    Infinite(&'a CssComponentValue),
}
impl CssMediaResolution {
    pub(crate) fn numeric(value: CssResolutionCalculation) -> Self {
        Self {
            value: MediaResolutionValue::Numeric(value),
        }
    }
    pub(crate) fn infinite(value: CssComponentValue) -> Self {
        Self {
            value: MediaResolutionValue::Infinite(Box::new(value)),
        }
    }
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
    numerator: CssNumberCalculation,
    denominator: CssNumberCalculation,
    denominator_is_omitted: bool,
}
impl CssMediaRatio {
    pub(crate) fn new(
        numerator: CssNumberCalculation,
        denominator: CssNumberCalculation,
        denominator_is_omitted: bool,
    ) -> Self {
        Self {
            numerator,
            denominator,
            denominator_is_omitted,
        }
    }
    pub fn numerator(&self) -> &CssNumberCalculation {
        &self.numerator
    }
    pub fn denominator(&self) -> &CssNumberCalculation {
        &self.denominator
    }
    pub const fn denominator_is_omitted(&self) -> bool {
        self.denominator_is_omitted
    }
}
/// An authored grid value with a deferred closed `[0,1]` integer constraint.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssMediaGrid {
    calculation: CssIntegerCalculation,
    literal: Option<CssGridMode>,
}
#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
pub enum CssMediaGridRef<'a> {
    Literal(CssGridMode),
    Calculation(&'a CssIntegerCalculation),
}
impl CssMediaGrid {
    pub(crate) fn new(calculation: CssIntegerCalculation, literal: Option<CssGridMode>) -> Self {
        Self {
            calculation,
            literal,
        }
    }
    pub fn view(&self) -> CssMediaGridRef<'_> {
        match self.literal {
            Some(v) => CssMediaGridRef::Literal(v),
            None => CssMediaGridRef::Calculation(&self.calculation),
        }
    }
    pub fn components(&self) -> &CssComponentValues {
        self.calculation.components()
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssMediaUpdate {
    None,
    Slow,
    Fast,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssMediaOverflowBlock {
    None,
    Scroll,
    Paged,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssMediaOverflowInline {
    None,
    Scroll,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssMediaColorGamut {
    Srgb,
    P3,
    Rec2020,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssMediaDynamicRange {
    Standard,
    High,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssMediaEnvironmentBlending {
    Opaque,
    Additive,
    Subtractive,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssMediaInvertedColors {
    None,
    Inverted,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssMediaNavigationControls {
    None,
    Back,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssMediaScripting {
    None,
    InitialOnly,
    Enabled,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssMediaReducedDataPreference {
    NoPreference,
    Reduce,
}
/// Every known feature in the selected Media Queries 5 descriptor index.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum CssMediaFeatureKind {
    Width,
    Height,
    DeviceWidth,
    DeviceHeight,
    AspectRatio,
    DeviceAspectRatio,
    Resolution,
    Color,
    ColorIndex,
    Monochrome,
    HorizontalViewportSegments,
    VerticalViewportSegments,
    Orientation,
    Scan,
    Grid,
    PrefersColorScheme,
    PrefersReducedMotion,
    PrefersReducedTransparency,
    PrefersContrast,
    ForcedColors,
    Hover,
    AnyHover,
    Pointer,
    AnyPointer,
    DisplayMode,
    Update,
    OverflowBlock,
    OverflowInline,
    ColorGamut,
    VideoColorGamut,
    DynamicRange,
    VideoDynamicRange,
    EnvironmentBlending,
    InvertedColors,
    NavControls,
    Scripting,
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
    pub const fn name(self) -> &'static str {
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
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssMediaFeatureQuery {
    Boolean(CssMediaFeatureKind),
    Width(CssMediaRange<CssMediaLength>),
    Height(CssMediaRange<CssMediaLength>),
    DeviceWidth(CssMediaRange<CssMediaLength>),
    DeviceHeight(CssMediaRange<CssMediaLength>),
    AspectRatio(CssMediaRange<CssMediaRatio>),
    DeviceAspectRatio(CssMediaRange<CssMediaRatio>),
    Resolution(CssMediaRange<CssMediaResolution>),
    Color(CssMediaRange<CssMediaInteger>),
    ColorIndex(CssMediaRange<CssMediaInteger>),
    Monochrome(CssMediaRange<CssMediaInteger>),
    HorizontalViewportSegments(CssMediaRange<CssMediaInteger>),
    VerticalViewportSegments(CssMediaRange<CssMediaInteger>),
    Orientation(CssOrientation),
    Scan(CssScanMode),
    Grid(CssMediaGrid),
    PrefersColorScheme(CssColorSchemePreference),
    PrefersReducedMotion(CssReducedMotionPreference),
    PrefersReducedTransparency(CssReducedTransparencyPreference),
    PrefersContrast(CssContrastPreference),
    ForcedColors(CssForcedColorsMode),
    Hover(CssHoverCapability),
    AnyHover(CssHoverCapability),
    Pointer(CssPointerCapability),
    AnyPointer(CssPointerCapability),
    DisplayMode(CssDisplayMode),
    Update(CssMediaUpdate),
    OverflowBlock(CssMediaOverflowBlock),
    OverflowInline(CssMediaOverflowInline),
    ColorGamut(CssMediaColorGamut),
    VideoColorGamut(CssMediaColorGamut),
    DynamicRange(CssMediaDynamicRange),
    VideoDynamicRange(CssMediaDynamicRange),
    EnvironmentBlending(CssMediaEnvironmentBlending),
    InvertedColors(CssMediaInvertedColors),
    NavControls(CssMediaNavigationControls),
    Scripting(CssMediaScripting),
    PrefersReducedData(CssMediaReducedDataPreference),
}
impl CssMediaFeatureQuery {
    pub const fn name(&self) -> &'static str {
        self.kind().name()
    }
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
