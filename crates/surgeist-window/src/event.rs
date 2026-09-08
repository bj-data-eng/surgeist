#[cfg(feature = "accessibility")]
use super::AccessibilityEvent;
use super::{Id, Metrics, PhysicalPoint, Point, Rect, Theme, WindowSnapshot};
use std::{path::PathBuf, time::Instant};

/// Native window event payload emitted by this crate.
///
/// Close requests and completed destruction use the specialized
/// [`Handler::close`](crate::Handler::close) and
/// [`Handler::closed`](crate::Handler::closed) callbacks rather than generic
/// events.
///
/// ```compile_fail
/// use surgeist_window::{EventKind, Id};
///
/// let _ = EventKind::Destroyed(Id::from_u64(1));
/// ```
///
/// ```compile_fail
/// use surgeist_window::{EventKind, Id};
///
/// let _ = EventKind::CloseRequested(Id::from_u64(1));
/// ```
#[derive(Clone, Debug, PartialEq)]
pub enum EventKind {
    /// Produced after a successful open transition, before the ready callback.
    Created(WindowSnapshot),
    /// Produced by the shared lifecycle pump when a window is suspended.
    Suspended(Id),
    /// Produced by the shared lifecycle pump while a window is resumed.
    Resumed(Id),
    /// Produced by a committed native focus transition.
    Focused {
        /// Identity of the window whose focus changed.
        id: Id,
        /// New observed focus state.
        focused: bool,
    },
    /// Produced by a committed native metrics transition reporting a new size.
    Resized(Metrics),
    /// Produced by a committed native metrics transition reporting a scale change.
    ScaleFactorChanged(Metrics),
    /// Produced by a committed native position transition.
    Moved {
        /// Identity of the window that moved.
        id: Id,
        /// New logical outer position after scale conversion by the host adapter.
        position: Point,
    },
    /// Produced by a committed native occlusion transition.
    Occluded {
        /// Identity of the window whose occlusion changed.
        id: Id,
        /// Whether native content is currently occluded.
        occluded: bool,
    },
    /// Produced by a committed native theme transition.
    ThemeChanged {
        /// Identity of the window whose appearance changed.
        id: Id,
        /// Observed appearance, or `None` when the host supplies no preference.
        theme: Option<Theme>,
    },
    /// Produced by lossless native file-drag transitions.
    FileDrag(FileDragEvent),
    /// Produced by typed native input lowering.
    Input(InputEvent),
    #[cfg(feature = "accessibility")]
    /// Produced by the typed accessibility adapter when the accessibility feature is enabled.
    Accessibility(AccessibilityEvent),
}

impl EventKind {
    /// Returns the identity of the window that produced this event.
    #[must_use]
    pub fn id(&self) -> Id {
        match self {
            Self::Created(state) => state.id(),
            Self::Suspended(id) | Self::Resumed(id) => *id,
            Self::Focused { id, .. }
            | Self::Moved { id, .. }
            | Self::Occluded { id, .. }
            | Self::ThemeChanged { id, .. } => *id,
            Self::Resized(metrics) | Self::ScaleFactorChanged(metrics) => metrics.id(),
            Self::FileDrag(event) => event.id(),
            Self::Input(event) => event.id(),
            #[cfg(feature = "accessibility")]
            Self::Accessibility(event) => event.id(),
        }
    }
}

/// Native file-drag payloads that retain paths without converting them to text.
///
/// Native lowering delivers these in native event order. Each [`PathBuf`] is
/// preserved losslessly, including paths that are not valid UTF-8.
#[derive(Clone, Debug, PartialEq)]
pub enum FileDragEvent {
    /// The first hovered native file entered this window's drag target.
    Entered {
        /// Identity of the target window.
        id: Id,
        /// Native paths supplied by the platform.
        paths: Vec<PathBuf>,
    },
    /// A native file hover updated the current ordered set of paths.
    Hovered {
        /// Identity of the target window.
        id: Id,
        /// Native paths supplied by the platform, in hover order.
        paths: Vec<PathBuf>,
        /// Last observed logical mouse position, if the adapter has one.
        position: Option<Point>,
    },
    /// Native file drag completion dropped paths onto the window.
    Dropped {
        /// Identity of the target window.
        id: Id,
        /// Native paths supplied by the platform.
        paths: Vec<PathBuf>,
        /// Last observed logical mouse position, if the adapter has one.
        position: Option<Point>,
    },
    /// The native file-drag operation was cancelled.
    Cancelled {
        /// Identity of the target window.
        id: Id,
    },
}

impl FileDragEvent {
    /// Returns the identity of the target window.
    #[must_use]
    pub fn id(&self) -> Id {
        match self {
            Self::Entered { id, .. }
            | Self::Hovered { id, .. }
            | Self::Dropped { id, .. }
            | Self::Cancelled { id } => *id,
        }
    }
}

/// Native input payload for one target window.
///
/// Input is delivered in source order through [`EventKind::Input`]. Current
/// `winit` lowering produces [`PointerEvent`] values only for mouse and touch;
/// pen and unknown pointer kinds have no current `winit` producer. It lowers
/// [`winit::event::WindowEvent::MouseWheel`] to [`Self::Wheel`],
/// [`winit::event::WindowEvent::KeyboardInput`] to [`Self::Key`], and
/// [`winit::event::WindowEvent::ModifiersChanged`] to [`Self::Modifiers`]. It
/// lowers [`winit::event::WindowEvent::Ime`] to [`ImeEvent::Enabled`],
/// [`ImeEvent::Disabled`], [`ImeEvent::Preedit`], or [`ImeEvent::Commit`];
/// [`ImeEvent::DeleteSurrounding`] and [`Self::StandardKeyBinding`] have no
/// current `winit` producer. Native platform support determines which of these
/// events are emitted.
#[derive(Clone, Debug, PartialEq)]
pub enum InputEvent {
    /// Pointer contact, movement, button, or boundary input.
    Pointer(PointerEvent),
    /// Pointer-wheel input.
    Wheel(WheelEvent),
    /// Keyboard input with logical and physical identities.
    Key(KeyEvent),
    /// A native modifier-state change.
    Modifiers {
        /// Identity of the target window.
        id: Id,
        /// Complete modifier state reported by the native event.
        modifiers: ModifierState,
    },
    /// IME composition or text-commit input.
    Ime(ImeEvent),
    /// A platform-standard key binding represented as text.
    ///
    /// The current `winit` lowering does not produce this variant.
    StandardKeyBinding(StandardKeyBindingEvent),
}

impl InputEvent {
    /// Returns the identity of the target window.
    #[must_use]
    pub fn id(&self) -> Id {
        match self {
            Self::Pointer(event) => event.id,
            Self::Wheel(event) => event.id,
            Self::Key(event) => event.id,
            Self::Modifiers { id, .. } => *id,
            Self::Ime(event) => event.id(),
            Self::StandardKeyBinding(event) => event.id,
        }
    }
}

/// Stage of a pointer interaction reported by the native host.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PointerPhase {
    /// A pointing device entered the window.
    Entered,
    /// A pointing device changed position.
    Moved,
    /// A pointer button or contact began.
    Pressed,
    /// A pointer button or contact ended.
    Released,
    /// A pointing device left the window.
    Left,
    /// A native pointer contact was cancelled.
    Cancelled,
}

/// Pointing-device category carried by [`PointerEvent`].
///
/// Current `winit` lowering emits only [`Self::Mouse`] and [`Self::Touch`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PointerKind {
    /// Mouse input.
    Mouse,
    /// Touch contact input.
    Touch,
    /// Pen or stylus input retained for a host that reports it.
    ///
    /// The current `winit` lowering does not produce this kind.
    Pen,
    /// A pointing device with no represented category.
    ///
    /// The current `winit` lowering does not produce this kind.
    Unknown,
}

/// One pointer payload, with optional facts omitted when the source has none.
///
/// Current `winit` lowering produces only mouse and touch payloads. Pen and
/// unknown kinds remain meaningful retained representations, but have no current
/// production `winit` producer.
#[derive(Clone, Debug, PartialEq)]
pub struct PointerEvent {
    /// Identity of the target window.
    pub id: Id,
    /// Native interaction stage.
    pub phase: PointerPhase,
    /// Native pointing-device category.
    pub kind: PointerKind,
    /// Native contact identifier for multi-pointer input, if reported.
    pub pointer_id: Option<u64>,
    /// Logical window position, if the event carries one.
    pub position: Option<Point>,
    /// Integer native-pixel position, if the event carries one.
    pub physical_position: Option<PhysicalPoint>,
    /// Logical movement delta, if native lowering can derive one.
    pub delta: Option<Point>,
    /// Changed button, if this event represents a button transition.
    pub button: Option<PointerButton>,
    /// Modifier state captured with this pointer event.
    pub modifiers: ModifierState,
    /// Optional device measurements supplied with this event.
    pub device: PointerDeviceData,
    /// Monotonic receipt timestamp, if the event source recorded one.
    pub timestamp: Option<Instant>,
}

/// Native pointer button identity.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PointerButton {
    /// Primary pointer button.
    Primary,
    /// Secondary pointer button.
    Secondary,
    /// Middle pointer button.
    Middle,
    /// Browser-style back pointer button.
    Back,
    /// Browser-style forward pointer button.
    Forward,
    /// A native button with an unrecognized numeric identity.
    Other(u16),
}

/// Optional pressure and orientation measurements from a pointing device.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PointerDeviceData {
    /// Normalized force measurement in the range `0.0..=1.0`, if supplied.
    /// Raw force and calibration information are not retained.
    pub force: Option<f64>,
    /// Normalized pressure measurement, if supplied.
    pub pressure: Option<f64>,
    /// Tangential pressure measurement, if supplied.
    pub tangential_pressure: Option<f64>,
    /// Tilt about the horizontal axis, if supplied.
    pub tilt_x: Option<f64>,
    /// Tilt about the vertical axis, if supplied.
    pub tilt_y: Option<f64>,
    /// Stylus rotation measurement, if supplied.
    pub twist: Option<f64>,
    /// Stylus altitude measurement, if supplied.
    pub altitude: Option<f64>,
    /// Stylus azimuth measurement, if supplied.
    pub azimuth: Option<f64>,
}

/// Native wheel input for one target window.
#[derive(Clone, Debug, PartialEq)]
pub struct WheelEvent {
    /// Identity of the target window.
    pub id: Id,
    /// Native scroll delta and its unit.
    pub delta: WheelDelta,
    /// Native touch-scroll phase associated with this wheel event.
    pub phase: TouchPhase,
    /// Logical pointer position, if the adapter has one.
    pub position: Option<Point>,
    /// Modifier state captured with this wheel event.
    pub modifiers: ModifierState,
    /// Monotonic receipt timestamp, if the event source recorded one.
    pub timestamp: Option<Instant>,
}

/// A wheel delta whose unit is retained from the native source.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum WheelDelta {
    /// Line-based horizontal and vertical scroll units.
    Lines {
        /// Horizontal line delta.
        x: f64,
        /// Vertical line delta.
        y: f64,
    },
    /// Native-pixel horizontal and vertical scroll units.
    Pixels {
        /// Horizontal native-pixel delta.
        x: f64,
        /// Vertical native-pixel delta.
        y: f64,
    },
}

/// Stage of a touch-associated wheel sequence.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TouchPhase {
    /// The scroll sequence started.
    Started,
    /// The scroll sequence continued.
    Moved,
    /// The scroll sequence ended.
    Ended,
    /// The scroll sequence was cancelled.
    Cancelled,
}

/// Native keyboard input with both semantic and physical identities.
#[derive(Clone, Debug, PartialEq)]
pub struct KeyEvent {
    /// Identity of the target window.
    pub id: Id,
    /// Layout-aware semantic key identity reported by the platform.
    pub logical_key: keyboard_types::Key,
    /// Layout-independent physical key location reported by the platform.
    pub physical_key: keyboard_types::Code,
    /// Physical location of the key on the input device.
    pub location: keyboard_types::Location,
    /// Whether the key was pressed or released.
    pub state: KeyState,
    /// Whether this is a repeated key press.
    pub repeat: bool,
    /// Whether the platform marks the event as synthetic.
    pub synthetic: bool,
    /// Modifier state captured with this key event.
    pub modifiers: ModifierState,
    /// Monotonic receipt timestamp, if the event source recorded one.
    pub timestamp: Option<Instant>,
}

/// Native key transition state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum KeyState {
    /// A key became pressed.
    Pressed,
    /// A key became released.
    Released,
}

/// Modifier state reported with a native input event.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ModifierState {
    /// Whether Shift is active.
    pub shift: bool,
    /// Whether Control is active.
    pub control: bool,
    /// Whether Alt is active.
    pub alt: bool,
    /// Whether the platform Super or Command key is active.
    pub super_key: bool,
}

/// IME composition, commit, and surrounding-text deletion input.
///
/// Current `winit` lowering produces [`Self::Enabled`], [`Self::Disabled`],
/// [`Self::Preedit`], and [`Self::Commit`]. [`Self::DeleteSurrounding`] retains
/// the request's before/after payload meaning for hosts that provide it, but has
/// no current production `winit` producer.
#[derive(Clone, Debug, PartialEq)]
pub enum ImeEvent {
    /// The native IME became enabled for this window.
    Enabled {
        /// Identity of the target window.
        id: Id,
    },
    /// The native IME became disabled for this window.
    Disabled {
        /// Identity of the target window.
        id: Id,
    },
    /// The IME updated uncommitted composition text.
    Preedit {
        /// Identity of the target window.
        id: Id,
        /// Uncommitted composition text.
        text: String,
        /// Byte-range cursor selection in `text`, if the native IME reported one.
        cursor: Option<(usize, usize)>,
    },
    /// The IME committed text to the application.
    Commit {
        /// Identity of the target window.
        id: Id,
        /// Committed text.
        text: String,
    },
    /// An IME requested deletion around the current selection.
    ///
    /// The counts retain the source request before and after the selection. The
    /// current `winit` lowering does not produce this variant.
    DeleteSurrounding {
        /// Identity of the target window.
        id: Id,
        /// Number of units requested before the selection.
        before: usize,
        /// Number of units requested after the selection.
        after: usize,
    },
}

impl ImeEvent {
    /// Returns the identity of the target window.
    #[must_use]
    pub fn id(&self) -> Id {
        match self {
            Self::Enabled { id }
            | Self::Disabled { id }
            | Self::Preedit { id, .. }
            | Self::Commit { id, .. }
            | Self::DeleteSurrounding { id, .. } => *id,
        }
    }
}

/// A platform-standard key binding represented as text.
///
/// This payload is retained for a host that reports such a binding. The current
/// `winit` lowering does not produce it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StandardKeyBindingEvent {
    /// Identity of the target window.
    pub id: Id,
    /// Platform-reported binding text.
    pub binding: String,
}

/// An authored IME operation for a window.
#[derive(Clone, Debug, PartialEq)]
pub enum ImeRequest {
    /// Disable IME interaction.
    Disable,
    /// Enable IME interaction with this configuration.
    Enable(ImeConfig),
    /// Update the active IME configuration.
    Update(ImeConfig),
    /// Restart IME interaction with this configuration.
    Restart(ImeConfig),
}

/// Semantic configuration supplied to an IME request.
#[derive(Clone, Debug, PartialEq)]
pub struct ImeConfig {
    /// Intended text-entry purpose.
    pub purpose: ImePurpose,
    /// Requested text-entry hint.
    pub hint: ImeHint,
    /// Logical cursor rectangle, if known.
    pub cursor_area: Option<Rect>,
    /// Surrounding text and selection, if available.
    pub surrounding_text: Option<ImeSurroundingText>,
}

/// Semantic purpose of the text being entered through an IME.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum ImePurpose {
    /// General text entry.
    #[default]
    Normal,
    /// Password entry.
    Password,
    /// Numeric entry.
    Number,
    /// Email-address entry.
    Email,
    /// URL entry.
    Url,
    /// Terminal-style text entry.
    Terminal,
}

/// Text-entry hint for an IME.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum ImeHint {
    /// No explicit text-entry hint.
    #[default]
    None,
    /// Request spell checking.
    Spellcheck,
    /// Request that spell checking be disabled.
    NoSpellcheck,
}

/// Surrounding text and selection supplied to an IME.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ImeSurroundingText {
    /// Complete surrounding text supplied by the application.
    pub text: String,
    /// Cursor offset within `text`.
    pub cursor: usize,
    /// Selection anchor offset within `text`.
    pub anchor: usize,
}
