//! Validated host observations. Consumers own document state and policy overlays.

use crate::geometry::{normalize_nonnegative_size, normalize_outer_position};
use crate::{Error, ErrorCode, Insets, Point, Rect, Result, Size, Theme};

/// A finite, strictly positive scale factor.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct ScaleFactor(f64);

impl ScaleFactor {
    /// The identity scale.
    pub const ONE: Self = Self(1.0);

    /// Rejects zero, negative, infinite and NaN factors.
    pub fn new(value: f64) -> Result<Self> {
        if !value.is_finite() || value <= 0.0 {
            return Err(invalid("scale factor must be finite and positive"));
        }
        Ok(Self(value))
    }

    /// Returns the dimensionless factor.
    #[must_use]
    pub const fn get(self) -> f64 {
        self.0
    }
}

/// Vertical output density at page zoom one, including vector output.
///
/// Raster density uses device pixels per CSS pixel; non-square pixels use the
/// vertical density. A printer supplies arbitrary-color screening density.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum OutputResolution {
    /// A finite raster density.
    Raster(ScaleFactor),
    /// Unconstrained vector output, greater than every finite density.
    Infinite,
}

impl OutputResolution {
    /// Applies page zoom, without applying visual-viewport magnification.
    pub fn at_page_zoom(self, page_zoom: ScaleFactor) -> Result<Self> {
        match self {
            Self::Raster(density) => Ok(Self::Raster(ScaleFactor::new(
                density.get() * page_zoom.get(),
            )?)),
            Self::Infinite => Ok(Self::Infinite),
        }
    }
}

/// Observed output-device availability, distinct from missing information.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum OutputDevice {
    /// The host did not report an output device.
    Unavailable,
    /// No output device exists; the device pixel ratio fallback is one.
    Absent,
    /// An observed output device and its vertical density.
    Present(OutputResolution),
}

/// Actual top-level presentation, supplied by its owning host.
///
/// This is neither an authored manifest request nor element fullscreen state.
/// Consumers propagate this observation to child contexts.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DisplayMode {
    /// Fullscreen top-level presentation.
    Fullscreen,
    /// Standalone application presentation.
    Standalone,
    /// Application presentation with minimal navigation UI.
    MinimalUi,
    /// Browser presentation.
    Browser,
    /// Picture-in-picture presentation.
    PictureInPicture,
}

/// Observed mixing of display output with the real world.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EnvironmentBlending {
    /// Output obscures the real world.
    Opaque,
    /// Light is added to the real world.
    Additive,
    /// White is transparent and dark output adds contrast by subtracting light.
    Subtractive,
}

/// The source selected by the host for exposed screen geometry.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScreenAreaSource {
    /// Output-device bounds.
    OutputDevice,
    /// The viewport bounds.
    Viewport,
    /// A host-selected hidden or quantized area.
    PrivacyAdjusted,
}

/// Screen and available areas in one CSS-pixel coordinate system.
///
/// Coordinates increase rightward and downward. Available area is explicit;
/// lack of a work-area API does not imply that the full screen is available.
#[derive(Clone, Debug, PartialEq)]
pub struct ScreenMetrics {
    area: Rect,
    available_area: Option<Rect>,
    source: ScreenAreaSource,
    color_depth: u32,
}

impl ScreenMetrics {
    /// Validates finite coordinates and nonnegative sizes. Unknown/private depth
    /// uses 24; the result is at least three times `color_component_bits`.
    pub fn new(
        area: Rect,
        available_area: Option<Rect>,
        source: ScreenAreaSource,
        color_depth: Option<u16>,
        color_component_bits: Option<u16>,
    ) -> Result<Self> {
        Ok(Self {
            area: valid_rect(area)?,
            available_area: available_area.map(valid_rect).transpose()?,
            source,
            color_depth: u32::from(color_depth.unwrap_or(24))
                .max(3 * u32::from(color_component_bits.unwrap_or(0))),
        })
    }

    /// Returns selected screen bounds.
    #[must_use]
    pub const fn area(&self) -> Rect {
        self.area
    }
    /// Returns selected available bounds, when observed.
    #[must_use]
    pub const fn available_area(&self) -> Option<Rect> {
        self.available_area
    }
    /// Returns the host's exposure choice.
    #[must_use]
    pub const fn source(&self) -> ScreenAreaSource {
        self.source
    }
    /// Returns color bits excluding alpha, also used for pixel depth.
    #[must_use]
    pub const fn color_depth(&self) -> u32 {
        self.color_depth
    }
}

/// Native client-window bounds in the selected screen coordinate system.
///
/// The origin is the content area's origin, never the decorated outer frame.
#[derive(Clone, Debug, PartialEq)]
pub struct ClientWindowMetrics {
    area: Rect,
}

impl ClientWindowMetrics {
    /// Validates the finite origin and nonnegative dimensions.
    pub fn new(area: Rect) -> Result<Self> {
        Ok(Self {
            area: valid_rect(area)?,
        })
    }
    /// Returns client bounds, with rightward/downward positive axes.
    #[must_use]
    pub const fn area(&self) -> Rect {
        self.area
    }
}

/// Large/default, small and dynamic viewport sizes in CSS pixels.
#[derive(Clone, Debug, PartialEq)]
pub struct ViewportSizes {
    large: Size,
    small: Size,
    dynamic: Size,
}

impl ViewportSizes {
    /// Records retracted, expanded and current interface states.
    pub fn new(large: Size, small: Size, dynamic: Size) -> Result<Self> {
        let large = normalize_nonnegative_size(large, "large viewport")?;
        let small = normalize_nonnegative_size(small, "small viewport")?;
        let dynamic = normalize_nonnegative_size(dynamic, "dynamic viewport")?;
        if small.width > dynamic.width
            || dynamic.width > large.width
            || small.height > dynamic.height
            || dynamic.height > large.height
        {
            return Err(invalid(
                "dynamic viewport must lie between small and large viewports",
            ));
        }
        Ok(Self {
            large,
            small,
            dynamic,
        })
    }
    /// Uses one content area for a host without retractable interfaces.
    pub fn fixed(size: Size) -> Result<Self> {
        Self::new(size, size, size)
    }
    /// Changes the current interface extent without changing stable extrema.
    /// During interface animation the host may supply either endpoint.
    pub fn with_dynamic(&self, dynamic: Size) -> Result<Self> {
        Self::new(self.large, self.small, dynamic)
    }
    /// Returns the large/default viewport size.
    #[must_use]
    pub const fn large(&self) -> Size {
        self.large
    }
    /// Returns the small viewport size.
    #[must_use]
    pub const fn small(&self) -> Size {
        self.small
    }
    /// Returns the dynamic viewport size.
    #[must_use]
    pub const fn dynamic(&self) -> Size {
        self.dynamic
    }
}

/// Observed visual viewport in the layout viewport's CSS coordinate space.
///
/// Magnification does not alter layout coordinates or cause layout reflow.
/// Document activity, retained-object access and identity belong to consumers.
#[derive(Clone, Debug, PartialEq)]
pub struct VisualViewportMetrics {
    offset: Point,
    page_position: Point,
    size: Size,
    scale: ScaleFactor,
}

impl VisualViewportMetrics {
    /// Records visual-to-layout offset, position relative to the initial
    /// containing block, size excluding fixed classic scrollbars, and scale.
    pub fn new(
        offset: Point,
        page_position: Point,
        size: Size,
        scale: ScaleFactor,
    ) -> Result<Self> {
        Ok(Self {
            offset: normalize_outer_position(offset)?,
            page_position: normalize_outer_position(page_position)?,
            size: normalize_nonnegative_size(size, "visual viewport")?,
            scale,
        })
    }
    /// Returns the offset relative to the layout viewport.
    #[must_use]
    pub const fn offset(&self) -> Point {
        self.offset
    }
    /// Returns position relative to the initial containing block.
    #[must_use]
    pub const fn page_position(&self) -> Point {
        self.page_position
    }
    /// Returns the visual size, excluding fixed classic scrollbars.
    #[must_use]
    pub const fn size(&self) -> Size {
        self.size
    }
    /// Returns visual magnification only.
    #[must_use]
    pub const fn scale(&self) -> ScaleFactor {
        self.scale
    }
}

/// Viewport observations supplied by the owning host, in CSS pixels.
#[derive(Clone, Debug, PartialEq)]
pub struct ViewportMetrics {
    sizes: ViewportSizes,
    layout: Size,
    initial_containing_block: Option<Size>,
    paged_area: Option<Size>,
    scroll: Option<Point>,
    root_scrollbars: Option<Insets>,
    visual: Option<VisualViewportMetrics>,
}

impl ViewportMetrics {
    /// Records layout size including rendered scrollbars and viewport variants.
    /// Other observations remain unavailable until supplied explicitly.
    pub fn new(layout: Size, sizes: ViewportSizes) -> Result<Self> {
        Ok(Self {
            sizes,
            layout: normalize_nonnegative_size(layout, "layout viewport")?,
            initial_containing_block: None,
            paged_area: None,
            scroll: None,
            root_scrollbars: None,
            visual: None,
        })
    }
    /// Supplies independently observed layout-owned geometry. Scrollbar insets
    /// report occupied/reserved space; consumers decide whether root CSS makes
    /// that reservation unconditional for viewport-unit resolution.
    pub fn with_layout_observations(
        mut self,
        initial_containing_block: Option<Size>,
        paged_area: Option<Size>,
        scroll: Option<Point>,
        root_scrollbars: Option<Insets>,
    ) -> Result<Self> {
        self.initial_containing_block = initial_containing_block
            .map(|s| normalize_nonnegative_size(s, "initial containing block"))
            .transpose()?;
        self.paged_area = paged_area
            .map(|s| normalize_nonnegative_size(s, "paged area"))
            .transpose()?;
        self.scroll = scroll.map(normalize_outer_position).transpose()?;
        if let Some(insets) = root_scrollbars
            && [insets.top, insets.right, insets.bottom, insets.left]
                .into_iter()
                .any(|v| !v.is_finite() || v < 0.0)
        {
            return Err(invalid(
                "root scrollbar insets must be finite and nonnegative",
            ));
        }
        self.root_scrollbars = root_scrollbars;
        Ok(self)
    }
    /// Adds a visual viewport without altering layout geometry.
    #[must_use]
    pub fn with_visual(mut self, visual: VisualViewportMetrics) -> Self {
        self.visual = Some(visual);
        self
    }
    /// Returns viewport variants.
    #[must_use]
    pub const fn sizes(&self) -> &ViewportSizes {
        &self.sizes
    }
    /// Returns layout dimensions including rendered scrollbars.
    #[must_use]
    pub const fn layout(&self) -> Size {
        self.layout
    }
    /// Returns the independently observed initial containing block.
    #[must_use]
    pub const fn initial_containing_block(&self) -> Option<Size> {
        self.initial_containing_block
    }
    /// Returns the paged area when applicable and observed.
    #[must_use]
    pub const fn paged_area(&self) -> Option<Size> {
        self.paged_area
    }
    /// Returns scroll relative to the initial containing block's origin.
    #[must_use]
    pub const fn scroll(&self) -> Option<Point> {
        self.scroll
    }
    /// Returns root scrollbar occupied/reserved space when observed.
    #[must_use]
    pub const fn root_scrollbars(&self) -> Option<Insets> {
        self.root_scrollbars
    }
    /// Returns independently observed visual-viewport geometry.
    #[must_use]
    pub const fn visual(&self) -> Option<&VisualViewportMetrics> {
        self.visual.as_ref()
    }
}

/// Accuracy of an observed pointing device; zoom does not change this fact.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PointerAccuracy {
    /// No pointing capability.
    None,
    /// Limited pointing accuracy.
    Coarse,
    /// Fine pointing accuracy.
    Fine,
}

/// One UA-selected or available pointer's capability.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PointerCapability {
    /// Observed accuracy, potentially downgraded by host accessibility policy.
    pub accuracy: PointerAccuracy,
    /// Whether hover is convenient. Inconvenient/emulated hover is false.
    pub hover: bool,
}

/// Primary and union pointer observations. Keyboard/d-pad presence is unrelated.
#[derive(Clone, Debug, PartialEq)]
pub struct InputCapabilities {
    primary: PointerCapability,
    coarse: bool,
    fine: bool,
    hover: bool,
}

impl InputCapabilities {
    /// Supplies the UA-selected primary pointer and every available pointer.
    /// The primary can be downgraded independently by accessibility policy.
    pub fn new(
        primary: PointerCapability,
        pointers: impl IntoIterator<Item = PointerCapability>,
    ) -> Result<Self> {
        let mut result = Self {
            primary,
            coarse: false,
            fine: false,
            hover: false,
        };
        if primary.accuracy == PointerAccuracy::None && primary.hover {
            return Err(invalid(
                "a pointer without pointing capability cannot hover",
            ));
        }
        for pointer in pointers {
            match pointer.accuracy {
                PointerAccuracy::None if pointer.hover => {
                    return Err(invalid(
                        "a pointer without pointing capability cannot hover",
                    ));
                }
                PointerAccuracy::None => {}
                PointerAccuracy::Coarse => result.coarse = true,
                PointerAccuracy::Fine => result.fine = true,
            }
            result.hover |= pointer.hover;
        }
        if (primary.accuracy != PointerAccuracy::None && result.no_pointer())
            || (primary.accuracy == PointerAccuracy::Fine && !result.fine)
            || (primary.hover && !result.hover)
        {
            return Err(invalid(
                "selected primary capability is not available from the pointer set",
            ));
        }
        Ok(result)
    }
    /// Returns UA-selected primary pointer capability.
    #[must_use]
    pub const fn primary(&self) -> PointerCapability {
        self.primary
    }
    /// Returns whether any available pointer is coarse.
    #[must_use]
    pub const fn any_coarse(&self) -> bool {
        self.coarse
    }
    /// Returns whether any available pointer is fine.
    #[must_use]
    pub const fn any_fine(&self) -> bool {
        self.fine
    }
    /// Returns true only when no available pointer has pointing capability.
    #[must_use]
    pub const fn no_pointer(&self) -> bool {
        !self.coarse && !self.fine
    }
    /// Returns whether any available pointer supports convenient hover.
    #[must_use]
    pub const fn any_hover(&self) -> bool {
        self.hover
    }
}

/// Discoverable UA navigation controls, independent of current history.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NavigationControls {
    /// No discoverable UA navigation control; gestures and page buttons do not count.
    None,
    /// A discoverable back control, even when temporarily disabled.
    Back,
}

/// Observed preference for reducing a host characteristic.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReductionPreference {
    /// No reduction preference.
    NoPreference,
    /// Reduction requested.
    Reduce,
}

/// Observed contrast preference. Custom is neither less nor more contrast.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContrastPreference {
    /// No active preference.
    NoPreference,
    /// Less contrast preferred.
    Less,
    /// More contrast preferred.
    More,
    /// A custom palette is preferred; observation only, not a request value.
    Custom,
}

/// Baseline host preferences, before any origin-specific override.
///
/// `None` means unavailable, not `NoPreference`. Consumers retain this baseline
/// while applying overlays, so clearing an override reveals current host facts.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct UserPreferences {
    /// Baseline reduced-motion observation.
    pub reduced_motion: Option<ReductionPreference>,
    /// Baseline reduced-transparency observation.
    pub reduced_transparency: Option<ReductionPreference>,
    /// Baseline reduced-data observation.
    pub reduced_data: Option<ReductionPreference>,
    /// Baseline contrast observation.
    pub contrast: Option<ContrastPreference>,
    /// Baseline color scheme; a host with no active preference supplies light.
    pub color_scheme: Option<Theme>,
}

/// Coherent platform observations for one window.
///
/// Missing facts remain explicit. Document activity/object identity, origin and
/// permission decisions, preference overrides, CSS matching and privacy policy
/// are consumer-owned. Native scale, page zoom and visual scale are distinct.
#[derive(Clone, Debug, PartialEq)]
pub struct HostEnvironment {
    /// Observed native logical-to-device scale; unavailable on an unknown host.
    pub native_scale: Option<ScaleFactor>,
    /// Page zoom, which changes the initial viewport. Unknown is explicit.
    pub page_zoom: Option<ScaleFactor>,
    /// Output availability and vertical density at page zoom one.
    pub output: OutputDevice,
    /// Selected screen geometry, when observed.
    pub screen: Option<ScreenMetrics>,
    /// Viewport geometry, when observed. `None` does not prove no viewport exists.
    pub viewport: Option<ViewportMetrics>,
    /// Client-window area in the screen coordinate system, when observed.
    pub client_area: Option<ClientWindowMetrics>,
    /// Actual top-level presentation mode, when observed.
    pub display_mode: Option<DisplayMode>,
    /// Output blending, when observed.
    pub blending: Option<EnvironmentBlending>,
    /// Selected-primary and all-pointer capabilities, when observed.
    pub input: Option<InputCapabilities>,
    /// Discoverable UA navigation, when observed.
    pub navigation: Option<NavigationControls>,
    /// Actual baseline preferences, independent of origin overlays.
    pub preferences: UserPreferences,
}

impl Default for HostEnvironment {
    fn default() -> Self {
        Self {
            native_scale: None,
            page_zoom: None,
            output: OutputDevice::Unavailable,
            screen: None,
            viewport: None,
            client_area: None,
            display_mode: None,
            blending: None,
            input: None,
            navigation: None,
            preferences: UserPreferences::default(),
        }
    }
}

impl HostEnvironment {
    /// Returns resolution including page zoom and excluding visual scale.
    /// An absent output uses the CSSOM View device-pixel-ratio fallback of one;
    /// unavailable output/zoom remains unavailable.
    pub fn resolution(&self) -> Result<Option<OutputResolution>> {
        match self.output {
            OutputDevice::Unavailable => Ok(None),
            OutputDevice::Absent => Ok(Some(OutputResolution::Raster(ScaleFactor::ONE))),
            OutputDevice::Present(resolution) => self
                .page_zoom
                .map(|zoom| resolution.at_page_zoom(zoom))
                .transpose(),
        }
    }
}

fn valid_rect(rect: Rect) -> Result<Rect> {
    Ok(Rect {
        origin: normalize_outer_position(rect.origin)?,
        size: normalize_nonnegative_size(rect.size, "screen area")?,
    })
}

fn invalid(message: &str) -> Error {
    Error::new(ErrorCode::InvalidRequest, message)
}
