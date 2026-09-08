use super::{
    Error, ErrorCode, Id, Insets, PhysicalPoint, PhysicalSize, Point, Result, Size,
    geometry::{normalize_nonnegative_size, normalize_outer_position},
};
use crate::{FullscreenMode, RoleKind};

/// Requested fullscreen presentation for a [`WindowRequest`].
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub enum Fullscreen {
    /// Request normal, non-fullscreen presentation.
    #[default]
    None,
    /// Request borderless fullscreen presentation.
    Borderless,
    /// Request an exclusive fullscreen mode when the host supports it.
    Exclusive,
}

impl Fullscreen {
    /// Returns the capability dimension corresponding to this intent.
    #[must_use]
    pub const fn mode(&self) -> FullscreenMode {
        match self {
            Self::None => FullscreenMode::None,
            Self::Borderless => FullscreenMode::Borderless,
            Self::Exclusive => FullscreenMode::Exclusive,
        }
    }
}

/// Requested native window stacking level.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Level {
    /// Use the platform's normal stacking level.
    #[default]
    Normal,
    /// Request a level above normal windows.
    AlwaysOnTop,
    /// Request a level below normal windows.
    AlwaysOnBottom,
}

/// Requested availability of native title-bar controls.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Controls {
    /// Whether the native close control is requested.
    pub close: bool,
    /// Whether the native minimize control is requested.
    pub minimize: bool,
    /// Whether the native maximize control is requested.
    pub maximize: bool,
}

impl Default for Controls {
    fn default() -> Self {
        Self {
            close: true,
            minimize: true,
            maximize: true,
        }
    }
}

/// A requested or observed native appearance preference.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Theme {
    /// A light appearance preference or observation.
    Light,
    /// A dark appearance preference or observation.
    Dark,
}

/// Authored native-role intent and its parent relationship.
///
/// Planning rejects an unsupported role before it reaches a host. The current
/// `winit` mapping supports only [`Self::Root`]; non-root roles remain authored
/// intent and are rejected by that backend's capability report.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub enum Role {
    /// A top-level window without a parent.
    #[default]
    Root,
    /// Authored dialog intent attached to a parent and carrying a blocking intent.
    Dialog {
        /// Identifier of the dialog's required parent window.
        parent: Id,
        /// Requested scope in which the dialog blocks interaction.
        modality: Modality,
    },
    /// Authored tool-window intent that may optionally be parented.
    Tool {
        /// Optional identifier of the tool window's parent.
        parent: Option<Id>,
    },
    /// Authored popup intent attached to a required parent window.
    Popup {
        /// Identifier of the popup's required parent window.
        parent: Id,
    },
}

impl Role {
    /// Returns the capability dimension corresponding to this role.
    #[must_use]
    pub const fn kind(&self) -> RoleKind {
        match self {
            Self::Root => RoleKind::Root,
            Self::Dialog { .. } => RoleKind::Dialog,
            Self::Tool { .. } => RoleKind::Tool,
            Self::Popup { .. } => RoleKind::Popup,
        }
    }
}

/// Authored interaction-blocking scope for a dialog role.
///
/// This intent is subject to role capability rejection. The current `winit`
/// mapping rejects dialog roles, so it does not apply a modality there.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Modality {
    /// Block only the dialog's parent window.
    Window,
    /// Block windows belonging to the application.
    App,
    /// Do not request dialog blocking.
    #[default]
    Modeless,
}

/// Authored intent for creating a native window.
///
/// This request holds caller-selected values before host planning and native
/// observation. It does not promise that a capability-sensitive intent is
/// supported on the selected target; creation validates its invariants before
/// it reaches the host.
#[derive(Clone, Debug, PartialEq)]
pub struct WindowRequest {
    title: String,
    name: Option<String>,
    position: Option<Point>,
    inner_size: Option<Size>,
    min_inner_size: Option<Size>,
    max_inner_size: Option<Size>,
    resizable: bool,
    controls: Controls,
    decorations: bool,
    transparent: bool,
    visible: bool,
    fullscreen: Fullscreen,
    level: Level,
    theme: Option<Theme>,
    role: Role,
}

/// Builder for authored [`WindowRequest`] intent.
///
/// It starts with the request defaults and an optional live lookup/uniqueness
/// label, then each method replaces one authored field without observing a
/// native window. A name never derives a runtime [`Id`]; [`Metrics`] is the sole
/// runtime `Id` source.
#[derive(Clone, Debug, PartialEq)]
pub struct WindowRequestBuilder {
    pub(crate) request: WindowRequest,
}

impl Default for WindowRequest {
    fn default() -> Self {
        Self {
            title: String::from("Surgeist"),
            name: None,
            position: None,
            inner_size: None,
            min_inner_size: None,
            max_inner_size: None,
            resizable: true,
            controls: Controls::default(),
            decorations: true,
            transparent: false,
            visible: true,
            fullscreen: Fullscreen::None,
            level: Level::Normal,
            theme: None,
            role: Role::Root,
        }
    }
}

impl WindowRequest {
    /// Starts a request whose optional name is a live lookup/uniqueness label.
    ///
    /// This label does not derive a runtime [`Id`]; [`Metrics`] supplies it.
    #[must_use]
    pub fn builder(name: impl Into<String>) -> WindowRequestBuilder {
        WindowRequestBuilder {
            request: Self {
                name: Some(name.into()),
                ..Self::default()
            },
        }
    }

    /// Returns the authored title.
    #[must_use]
    pub fn title(&self) -> &str {
        self.title.as_str()
    }

    /// Returns the authored optional live lookup/uniqueness label, when supplied.
    ///
    /// The label never derives the runtime [`Id`], which comes from [`Metrics`].
    #[must_use]
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// Returns the requested logical outer position, if authored.
    #[must_use]
    pub const fn position(&self) -> Option<Point> {
        self.position
    }

    /// Returns the requested logical inner size, if authored.
    #[must_use]
    pub const fn inner_size(&self) -> Option<Size> {
        self.inner_size
    }

    /// Returns the authored logical minimum inner size, if any.
    #[must_use]
    pub const fn min_inner_size(&self) -> Option<Size> {
        self.min_inner_size
    }

    /// Returns the authored logical maximum inner size, if any.
    #[must_use]
    pub const fn max_inner_size(&self) -> Option<Size> {
        self.max_inner_size
    }

    /// Returns whether user resizing is requested.
    #[must_use]
    pub const fn resizable(&self) -> bool {
        self.resizable
    }

    /// Returns the requested native title-bar controls.
    #[must_use]
    pub const fn controls(&self) -> Controls {
        self.controls
    }

    /// Returns whether native decorations are requested.
    #[must_use]
    pub const fn decorations(&self) -> bool {
        self.decorations
    }

    /// Returns whether transparent presentation is requested.
    #[must_use]
    pub const fn transparent(&self) -> bool {
        self.transparent
    }

    /// Returns whether the window is requested to become visible at creation.
    #[must_use]
    pub const fn visible(&self) -> bool {
        self.visible
    }

    /// Returns the authored fullscreen intent.
    #[must_use]
    pub fn fullscreen(&self) -> Fullscreen {
        self.fullscreen.clone()
    }

    /// Returns the authored native stacking level.
    #[must_use]
    pub const fn level(&self) -> Level {
        self.level
    }

    /// Returns an explicit appearance preference, or `None` to defer to the host.
    #[must_use]
    pub const fn theme(&self) -> Option<Theme> {
        self.theme
    }

    /// Returns the authored role and parent relationship.
    #[must_use]
    pub const fn role(&self) -> &Role {
        &self.role
    }

    #[cfg(feature = "accessibility")]
    pub(crate) const fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    pub(crate) const fn set_min_inner_size(&mut self, size: Option<Size>) {
        self.min_inner_size = size;
    }

    pub(crate) const fn set_max_inner_size(&mut self, size: Option<Size>) {
        self.max_inner_size = size;
    }

    pub(crate) fn normalize(mut self) -> Result<Self> {
        if matches!(self.name.as_deref(), Some("")) {
            return Err(Error::new(
                ErrorCode::InvalidRequest,
                "window name must not be empty",
            ));
        }

        self.position = self.position.map(normalize_outer_position).transpose()?;
        self.inner_size = self
            .inner_size
            .map(|size| normalize_nonnegative_size(size, "requested inner size"))
            .transpose()?;
        self.min_inner_size = self
            .min_inner_size
            .map(|size| normalize_nonnegative_size(size, "minimum inner size"))
            .transpose()?;
        self.max_inner_size = self
            .max_inner_size
            .map(|size| normalize_nonnegative_size(size, "maximum inner size"))
            .transpose()?;

        if let (Some(minimum), Some(maximum)) = (self.min_inner_size, self.max_inner_size)
            && (minimum.width > maximum.width || minimum.height > maximum.height)
        {
            return Err(Error::new(
                ErrorCode::InvalidRequest,
                "minimum inner size must not exceed maximum inner size",
            ));
        }

        if let Some(size) = self.inner_size {
            self.inner_size = Some(clamp_inner_size(
                size,
                self.min_inner_size,
                self.max_inner_size,
            ));
        }

        Ok(self)
    }
}

fn clamp_inner_size(size: Size, minimum: Option<Size>, maximum: Option<Size>) -> Size {
    let width = minimum.map_or(size.width, |minimum| size.width.max(minimum.width));
    let height = minimum.map_or(size.height, |minimum| size.height.max(minimum.height));

    Size {
        width: maximum.map_or(width, |maximum| width.min(maximum.width)),
        height: maximum.map_or(height, |maximum| height.min(maximum.height)),
    }
}

impl WindowRequestBuilder {
    /// Returns the assembled authored request without applying host validation.
    #[must_use]
    pub fn build(self) -> WindowRequest {
        self.request
    }

    /// Replaces the authored window title.
    #[must_use]
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.request.title = title.into();
        self
    }

    /// Replaces the authored optional live lookup/uniqueness label.
    ///
    /// The label never derives a runtime [`Id`], which comes from [`Metrics`].
    #[must_use]
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.request.name = Some(name.into());
        self
    }

    /// Sets the requested logical outer position.
    #[must_use]
    pub fn position(mut self, point: impl Into<Point>) -> Self {
        self.request.position = Some(point.into());
        self
    }

    /// Sets the requested logical inner size.
    #[must_use]
    pub fn inner_size(mut self, size: impl Into<Size>) -> Self {
        self.request.inner_size = Some(size.into());
        self
    }

    /// Sets the requested logical minimum inner size.
    #[must_use]
    pub fn min_inner_size(mut self, size: impl Into<Size>) -> Self {
        self.request.min_inner_size = Some(size.into());
        self
    }

    /// Sets the requested logical maximum inner size.
    #[must_use]
    pub fn max_inner_size(mut self, size: impl Into<Size>) -> Self {
        self.request.max_inner_size = Some(size.into());
        self
    }

    /// Sets whether the user may resize the window.
    #[must_use]
    pub const fn resizable(mut self, resizable: bool) -> Self {
        self.request.resizable = resizable;
        self
    }

    /// Disables user resizing.
    #[must_use]
    pub const fn fixed(self) -> Self {
        self.resizable(false)
    }

    /// Replaces the requested native title-bar controls.
    #[must_use]
    pub fn controls(mut self, controls: impl Into<Controls>) -> Self {
        self.request.controls = controls.into();
        self
    }

    /// Sets whether native window decorations are requested.
    #[must_use]
    pub const fn decorations(mut self, enabled: bool) -> Self {
        self.request.decorations = enabled;
        self
    }

    /// Sets whether transparent presentation is requested.
    #[must_use]
    pub const fn transparent(mut self, transparent: bool) -> Self {
        self.request.transparent = transparent;
        self
    }

    /// Sets whether the new window is requested to become visible.
    #[must_use]
    pub const fn visible(mut self, visible: bool) -> Self {
        self.request.visible = visible;
        self
    }

    /// Requests an initially hidden window.
    #[must_use]
    pub const fn hidden(self) -> Self {
        self.visible(false)
    }

    /// Replaces the authored fullscreen intent.
    #[must_use]
    pub fn fullscreen(mut self, fullscreen: impl Into<Fullscreen>) -> Self {
        self.request.fullscreen = fullscreen.into();
        self
    }

    /// Requests borderless fullscreen presentation.
    #[must_use]
    pub fn borderless(mut self) -> Self {
        self.request.fullscreen = Fullscreen::Borderless;
        self
    }

    /// Sets the requested native stacking level.
    #[must_use]
    pub const fn level(mut self, level: Level) -> Self {
        self.request.level = level;
        self
    }

    /// Sets an explicit appearance preference, or clears it with `None`.
    #[must_use]
    pub fn theme(mut self, theme: impl Into<Option<Theme>>) -> Self {
        self.request.theme = theme.into();
        self
    }

    /// Replaces the authored native role and parent relationship.
    #[must_use]
    pub const fn role(mut self, role: Role) -> Self {
        self.request.role = role;
        self
    }

    /// Selects the top-level root role.
    #[must_use]
    pub const fn root(self) -> Self {
        self.role(Role::Root)
    }

    /// Selects a dialog role that blocks its parent window.
    #[must_use]
    pub const fn dialog(self, parent: Id) -> Self {
        self.role(Role::Dialog {
            parent,
            modality: Modality::Window,
        })
    }

    /// Selects a dialog role with the supplied parent and blocking intent.
    #[must_use]
    pub const fn modal(self, parent: Id, modality: Modality) -> Self {
        self.role(Role::Dialog { parent, modality })
    }

    /// Selects a tool role with an optional parent window.
    #[must_use]
    pub const fn tool(self, parent: Option<Id>) -> Self {
        self.role(Role::Tool { parent })
    }

    /// Selects a popup role attached to `parent`.
    #[must_use]
    pub const fn popup(self, parent: Id) -> Self {
        self.role(Role::Popup { parent })
    }
}

/// Observed state of an existing native window.
///
/// A snapshot is not authored creation intent. Optional observations are `None`
/// when the host has not supplied that fact; convenience predicates use the
/// documented fallback only where their implementation provides one.
#[derive(Clone, Debug, PartialEq)]
pub struct WindowSnapshot {
    title: String,
    name: Option<String>,
    metrics: Metrics,
    focused: bool,
    visible: Option<bool>,
    minimized: Option<bool>,
    maximized: bool,
    occluded: Option<bool>,
    fullscreen: bool,
    theme: Option<Theme>,
    role: Role,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct WindowSnapshotSeed {
    pub(crate) title: String,
    pub(crate) name: Option<String>,
    pub(crate) metrics: Metrics,
    pub(crate) focused: bool,
    pub(crate) visible: Option<bool>,
    pub(crate) minimized: Option<bool>,
    pub(crate) maximized: bool,
    pub(crate) occluded: Option<bool>,
    pub(crate) fullscreen: bool,
    pub(crate) theme: Option<Theme>,
    pub(crate) role: Role,
}

impl WindowSnapshot {
    /// Creates a snapshot with known title and invariant-preserving metrics.
    ///
    /// The remaining fields use their source defaults until native observations
    /// or internal transitions update them.
    #[must_use]
    pub fn new(title: impl Into<String>, metrics: Metrics) -> Self {
        Self {
            title: title.into(),
            name: None,
            metrics,
            focused: false,
            visible: None,
            minimized: None,
            maximized: false,
            occluded: None,
            fullscreen: false,
            theme: None,
            role: Role::Root,
        }
    }

    #[must_use]
    pub(crate) fn from_seed(seed: WindowSnapshotSeed) -> Self {
        Self {
            title: seed.title,
            name: seed.name,
            metrics: seed.metrics,
            focused: seed.focused,
            visible: seed.visible,
            minimized: seed.minimized,
            maximized: seed.maximized,
            occluded: seed.occluded,
            fullscreen: seed.fullscreen,
            theme: seed.theme,
            role: seed.role,
        }
    }

    /// Adds an optional live lookup/uniqueness label to this snapshot.
    ///
    /// The label never derives this snapshot's runtime [`Id`]; its [`Metrics`]
    /// value is the sole `Id` source.
    #[must_use]
    pub fn named(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Records a known visibility observation.
    #[must_use]
    pub const fn with_visible(mut self, visible: bool) -> Self {
        self.visible = Some(visible);
        self
    }

    /// Records whether the window is focused.
    #[must_use]
    pub const fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }

    /// Returns the identity carried by this snapshot's metrics.
    #[must_use]
    pub const fn id(&self) -> Id {
        self.metrics.id()
    }

    /// Returns the invariant-preserving observed geometry and display scale.
    #[must_use]
    pub const fn metrics(&self) -> &Metrics {
        &self.metrics
    }

    /// Returns the observed logical outer position, if available.
    #[must_use]
    pub const fn position(&self) -> Option<Point> {
        self.metrics.outer_position()
    }

    /// Returns whether focus is currently observed.
    #[must_use]
    pub const fn is_focused(&self) -> bool {
        self.focused
    }

    /// Returns the observed title.
    #[must_use]
    pub fn title(&self) -> &str {
        self.title.as_str()
    }

    /// Returns the optional live lookup/uniqueness label, if known.
    ///
    /// It does not derive the runtime [`Id`], which comes from [`Metrics`].
    #[must_use]
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// Returns the native visibility observation, if the host supplied one.
    #[must_use]
    pub const fn visible(&self) -> Option<bool> {
        self.visible
    }

    /// Returns the observed appearance, if the host supplied one.
    #[must_use]
    pub const fn theme(&self) -> Option<Theme> {
        self.theme
    }

    /// Returns whether the window is visible, treating an absent observation as visible.
    #[must_use]
    pub fn is_visible(&self) -> bool {
        self.visible.unwrap_or(true)
    }

    /// Returns whether the window is occluded, treating an absent observation as not occluded.
    #[must_use]
    pub fn is_occluded(&self) -> bool {
        self.occluded.unwrap_or(false)
    }

    /// Returns the committed fullscreen state recorded in this snapshot.
    ///
    /// This is not necessarily an independently observed native fullscreen fact.
    #[must_use]
    pub const fn is_fullscreen(&self) -> bool {
        self.fullscreen
    }

    pub(crate) fn set_title(&mut self, title: String) {
        self.title = title;
    }

    pub(crate) const fn set_visible(&mut self, visible: Option<bool>) {
        self.visible = visible;
    }

    pub(crate) fn set_metrics(&mut self, metrics: Metrics) -> Result<()> {
        let snapshot_id = self.id();
        let metrics_id = metrics.id();
        if metrics_id != snapshot_id {
            return Err(Error::new(
                ErrorCode::InvalidRequest,
                format!(
                    "metrics identity {} does not match snapshot identity {}",
                    metrics_id.as_u64(),
                    snapshot_id.as_u64()
                ),
            )
            .with_id(metrics_id));
        }

        self.metrics = metrics;
        Ok(())
    }

    pub(crate) fn set_position(&mut self, position: Option<Point>) -> Result<()> {
        self.metrics = self.metrics.clone().with_outer_position(position)?;
        Ok(())
    }

    pub(crate) const fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }

    pub(crate) const fn set_theme(&mut self, theme: Option<Theme>) {
        self.theme = theme;
    }

    pub(crate) const fn set_occluded(&mut self, occluded: Option<bool>) {
        self.occluded = occluded;
    }

    pub(crate) const fn set_fullscreen(&mut self, fullscreen: bool) {
        self.fullscreen = fullscreen;
    }
}

/// Invariant-preserving observed geometry and display scale for one window.
///
/// Logical size and outer geometry are `f64` logical units. Physical size is
/// integer native pixels. This record is the sole runtime [`Id`] source; names
/// are only optional live lookup/uniqueness labels. Its identity is preserved
/// when it is installed into a [`WindowSnapshot`].
#[derive(Clone, Debug, PartialEq)]
pub struct Metrics {
    id: Id,
    logical_size: Size,
    physical_size: PhysicalSize,
    outer_position: Option<Point>,
    outer_size: Option<Size>,
    scale_factor: f64,
    safe_area: Insets,
}

impl Metrics {
    /// Returns the sole runtime identity source for the window these metrics describe.
    #[must_use]
    pub const fn id(&self) -> Id {
        self.id
    }

    /// Returns the observed logical inner size.
    #[must_use]
    pub const fn logical_size(&self) -> Size {
        self.logical_size
    }

    /// Returns the observed integer native-pixel inner size.
    #[must_use]
    pub const fn physical_size(&self) -> PhysicalSize {
        self.physical_size
    }

    /// Returns the observed logical outer position, if the host supplied it.
    #[must_use]
    pub const fn outer_position(&self) -> Option<Point> {
        self.outer_position
    }

    /// Returns the observed logical outer size, if the host supplied it.
    #[must_use]
    pub const fn outer_size(&self) -> Option<Size> {
        self.outer_size
    }

    /// Returns the positive display scale used for logical/physical conversion.
    #[must_use]
    pub const fn scale_factor(&self) -> f64 {
        self.scale_factor
    }

    /// Returns the observed logical safe-area insets.
    #[must_use]
    pub const fn safe_area(&self) -> Insets {
        self.safe_area
    }

    /// Builds validated metrics from an integer native-pixel inner size.
    ///
    /// The logical size is physical pixels divided by `scale_factor`; the scale
    /// must be finite and greater than zero.
    pub fn from_physical_size(
        id: Id,
        physical_size: PhysicalSize,
        scale_factor: f64,
    ) -> Result<Self> {
        let scale_factor = validate_scale_factor(scale_factor)?;
        let logical_size = normalize_nonnegative_size(
            Size {
                width: f64::from(physical_size.width) / scale_factor,
                height: f64::from(physical_size.height) / scale_factor,
            },
            "observed logical size",
        )?;
        Ok(Self {
            id,
            logical_size,
            physical_size,
            outer_position: None,
            outer_size: None,
            scale_factor,
            safe_area: Insets::default(),
        })
    }

    /// Adds validated observed logical outer geometry to these metrics.
    pub fn with_outer_geometry(
        mut self,
        outer_position: Option<Point>,
        outer_size: Option<Size>,
    ) -> Result<Self> {
        self.outer_position = outer_position.map(normalize_outer_position).transpose()?;
        self.outer_size = outer_size
            .map(|size| normalize_nonnegative_size(size, "observed outer size"))
            .transpose()?;
        Ok(self)
    }

    pub(crate) fn with_outer_position(self, outer_position: Option<Point>) -> Result<Self> {
        let outer_size = self.outer_size;
        self.with_outer_geometry(outer_position, outer_size)
    }

    /// Adds validated observed logical safe-area insets to these metrics.
    pub fn with_safe_area(mut self, safe_area: Insets) -> Result<Self> {
        self.safe_area = normalize_safe_area(safe_area)?;
        Ok(self)
    }

    /// Converts a logical point to integer native pixels.
    ///
    /// Each axis is multiplied by this metrics scale, rounded, then converted
    /// with Rust's `f64 as i32` behavior: `NaN` becomes zero, and finite or
    /// infinite values outside the `i32` range saturate at the corresponding
    /// bound. Rounding and saturation make this conversion non-reversible.
    #[must_use]
    pub fn logical_to_physical_point(&self, point: Point) -> PhysicalPoint {
        PhysicalPoint {
            x: (point.x * self.scale_factor).round() as i32,
            y: (point.y * self.scale_factor).round() as i32,
        }
    }

    /// Converts integer native pixels to logical coordinates using this scale.
    #[must_use]
    pub fn physical_to_logical_point(&self, point: PhysicalPoint) -> Point {
        Point {
            x: f64::from(point.x) / self.scale_factor,
            y: f64::from(point.y) / self.scale_factor,
        }
    }
}

fn validate_scale_factor(scale_factor: f64) -> Result<f64> {
    if !scale_factor.is_finite() || scale_factor <= 0.0 {
        return Err(Error::new(
            ErrorCode::InvalidRequest,
            "observed scale factor must be finite and greater than zero",
        ));
    }

    Ok(scale_factor)
}

fn normalize_safe_area(safe_area: Insets) -> Result<Insets> {
    let valid = [
        safe_area.top,
        safe_area.right,
        safe_area.bottom,
        safe_area.left,
    ]
    .into_iter()
    .all(|inset| inset.is_finite() && inset >= 0.0);

    if !valid {
        return Err(Error::new(
            ErrorCode::InvalidRequest,
            "observed safe area must use finite nonnegative insets",
        ));
    }

    Ok(Insets {
        top: canonical_zero(safe_area.top),
        right: canonical_zero(safe_area.right),
        bottom: canonical_zero(safe_area.bottom),
        left: canonical_zero(safe_area.left),
    })
}

const fn canonical_zero(value: f64) -> f64 {
    if value == 0.0 { 0.0 } else { value }
}
