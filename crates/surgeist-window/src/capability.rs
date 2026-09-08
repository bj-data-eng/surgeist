use super::{CursorGrab, Error, ErrorCode, ImeHint, ImePurpose};

const ROLE_COUNT: usize = 4;
const FULLSCREEN_COUNT: usize = 3;
const CURSOR_COUNT: usize = 3;
const WINDOW_OPERATION_COUNT: usize = 16;
const CURSOR_GRAB_COUNT: usize = 3;
const IME_CAPABILITY_COUNT: usize = 12;

const fn cursor_grab_index(grab: CursorGrab) -> usize {
    match grab {
        CursorGrab::None => 0,
        CursorGrab::Confined => 1,
        CursorGrab::Locked => 2,
    }
}

/// A target-sensitive authored window-role capability dimension.
///
/// A report controls whether planning accepts the role intent; it does not
/// convert that intent into an observed native role. Current `winit` target
/// reporting supports only [`Self::Root`] and reports every non-root role
/// unsupported.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum RoleKind {
    /// Capability to create a top-level root window.
    Root,
    /// Capability to accept authored dialog intent with a parent.
    Dialog,
    /// Capability to accept authored tool-window intent.
    Tool,
    /// Capability to accept authored popup intent with a parent.
    Popup,
}

impl RoleKind {
    const fn index(self) -> usize {
        match self {
            Self::Root => 0,
            Self::Dialog => 1,
            Self::Tool => 2,
            Self::Popup => 3,
        }
    }
}

/// A target-sensitive fullscreen capability dimension.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum FullscreenMode {
    /// Capability to select non-fullscreen presentation.
    None,
    /// Capability to select borderless fullscreen presentation.
    Borderless,
    /// Capability to select exclusive fullscreen presentation.
    Exclusive,
}

impl FullscreenMode {
    const fn index(self) -> usize {
        match self {
            Self::None => 0,
            Self::Borderless => 1,
            Self::Exclusive => 2,
        }
    }
}

/// A target-sensitive cursor presentation capability dimension.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum CursorCapability {
    /// Capability to select a built-in cursor icon.
    Icon,
    /// Capability to hide the cursor.
    Hidden,
    /// Capability to accept an opaque authored custom-cursor identifier.
    ///
    /// The generic model permits a host to report this supported. Current `winit`
    /// target reporting declares it unsupported, so planning rejects it.
    Custom,
}

impl CursorCapability {
    const fn index(self) -> usize {
        match self {
            Self::Icon => 0,
            Self::Hidden => 1,
            Self::Custom => 2,
        }
    }
}

/// A window operation whose host support can differ by target.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum WindowOperation {
    /// Change a window title after creation.
    SetTitle,
    /// Change a window's logical outer position.
    SetOuterPosition,
    /// Change visibility after creation.
    SetVisible,
    /// Change user-resizability after creation.
    SetResizable,
    /// Change native title-bar controls after creation.
    SetControls,
    /// Change native decorations after creation.
    SetDecorations,
    /// Request transparent presentation while opening a window.
    InitialTransparency,
    /// Change transparent presentation after creation.
    SetTransparent,
    /// Request a new logical inner size.
    RequestInnerSize,
    /// Change logical minimum or maximum inner-size bounds.
    SetInnerSizeBounds,
    /// Change native stacking level.
    SetLevel,
    /// Set an explicit appearance preference.
    SetExplicitTheme,
    /// Clear an explicit appearance preference.
    ResetTheme,
    /// Request native user attention.
    RequestUserAttention,
    /// Request the next draw callback.
    RequestDraw,
    /// Begin window destruction.
    Destroy,
}

impl WindowOperation {
    const fn index(self) -> usize {
        match self {
            Self::SetTitle => 0,
            Self::SetOuterPosition => 1,
            Self::SetVisible => 2,
            Self::SetResizable => 3,
            Self::SetControls => 4,
            Self::SetDecorations => 5,
            Self::InitialTransparency => 6,
            Self::SetTransparent => 7,
            Self::RequestInnerSize => 8,
            Self::SetInnerSizeBounds => 9,
            Self::SetLevel => 10,
            Self::SetExplicitTheme => 11,
            Self::ResetTheme => 12,
            Self::RequestUserAttention => 13,
            Self::RequestDraw => 14,
            Self::Destroy => 15,
        }
    }
}

/// An IME semantic whose host support can differ by target.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ImeCapability {
    /// Enable, disable, update, or restart IME interaction.
    Enablement,
    /// Supply the logical text cursor area to the IME.
    CursorArea,
    /// Support a particular semantic input purpose.
    Purpose(ImePurpose),
    /// Support a particular semantic text-entry hint.
    Hint(ImeHint),
    /// Supply surrounding text and selection information.
    SurroundingText,
}

impl ImeCapability {
    const fn index(self) -> usize {
        match self {
            Self::Enablement => 0,
            Self::CursorArea => 1,
            Self::Purpose(ImePurpose::Normal) => 2,
            Self::Purpose(ImePurpose::Password) => 3,
            Self::Purpose(ImePurpose::Number) => 4,
            Self::Purpose(ImePurpose::Email) => 5,
            Self::Purpose(ImePurpose::Url) => 6,
            Self::Purpose(ImePurpose::Terminal) => 7,
            Self::Hint(ImeHint::None) => 8,
            Self::Hint(ImeHint::Spellcheck) => 9,
            Self::Hint(ImeHint::NoSpellcheck) => 10,
            Self::SurroundingText => 11,
        }
    }
}

/// The selected host's support contract for one capability-sensitive operation.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum CapabilitySupport {
    /// The selected host declares this dimension supported.
    Supported,
    /// The selected host declares this dimension unavailable and `require_*` rejects it.
    Unsupported,
    /// Support depends on runtime state, but `require_*` permits the request.
    RuntimeDependent,
}

/// A typed capability query that can be recorded with command diagnostics.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum CapabilityKind {
    /// A window-role dimension.
    Role(RoleKind),
    /// A fullscreen-mode dimension.
    Fullscreen(FullscreenMode),
    /// A cursor-presentation dimension.
    Cursor(CursorCapability),
    /// A mutable window-operation dimension.
    Window(WindowOperation),
    /// A cursor-grab-mode dimension.
    CursorGrab(CursorGrab),
    /// An IME semantic dimension.
    Ime(ImeCapability),
    /// The typed accessibility integration dimension.
    Accessibility,
}

/// The immutable result of one typed capability query.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct CapabilityDecision {
    kind: CapabilityKind,
    support: CapabilitySupport,
}

impl CapabilityDecision {
    pub(crate) const fn new(kind: CapabilityKind, support: CapabilitySupport) -> Self {
        Self { kind, support }
    }

    /// Returns the queried target-sensitive capability dimension.
    #[must_use]
    pub const fn kind(&self) -> CapabilityKind {
        self.kind
    }

    /// Returns the host's immutable support classification for that dimension.
    #[must_use]
    pub const fn support(&self) -> CapabilitySupport {
        self.support
    }
}

/// Immutable capability report for the currently selected host target.
///
/// Decision methods expose the target-sensitive classification without failing.
/// Matching `require_*` methods return `UnsupportedFeature` only for
/// [`CapabilitySupport::Unsupported`]; they permit `Supported` and
/// `RuntimeDependent` requests for the runtime host to resolve.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HostCapabilities {
    roles: [CapabilitySupport; ROLE_COUNT],
    fullscreens: [CapabilitySupport; FULLSCREEN_COUNT],
    cursors: [CapabilitySupport; CURSOR_COUNT],
    window_operations: [CapabilitySupport; WINDOW_OPERATION_COUNT],
    cursor_grabs: [CapabilitySupport; CURSOR_GRAB_COUNT],
    ime_capabilities: [CapabilitySupport; IME_CAPABILITY_COUNT],
    accessibility: CapabilitySupport,
}

impl HostCapabilities {
    /// Starts an explicit report whose every target-sensitive dimension is unsupported.
    #[must_use]
    pub const fn builder() -> HostCapabilitiesBuilder {
        HostCapabilitiesBuilder::new()
    }

    /// Returns the support classification for one role.
    #[must_use]
    pub const fn role(&self, role: RoleKind) -> CapabilitySupport {
        self.roles[role.index()]
    }

    /// Requires one role, rejecting only a host-declared unsupported role.
    pub fn require_role(&self, role: RoleKind) -> Result<(), Error> {
        require_supported(self.role(role), || {
            Error::new(
                ErrorCode::UnsupportedFeature,
                format!("native host does not support {role:?} window role"),
            )
        })
    }

    /// Returns the support classification for one fullscreen mode.
    #[must_use]
    pub const fn fullscreen(&self, fullscreen: FullscreenMode) -> CapabilitySupport {
        self.fullscreens[fullscreen.index()]
    }

    /// Requires one fullscreen mode, rejecting only a host-declared unsupported mode.
    pub fn require_fullscreen(&self, fullscreen: FullscreenMode) -> Result<(), Error> {
        require_supported(self.fullscreen(fullscreen), || {
            Error::new(
                ErrorCode::UnsupportedFeature,
                format!("native host does not support {fullscreen:?} fullscreen"),
            )
        })
    }

    /// Returns the support classification for one cursor presentation mode.
    #[must_use]
    pub const fn cursor(&self, cursor: CursorCapability) -> CapabilitySupport {
        self.cursors[cursor.index()]
    }

    /// Requires one cursor mode, rejecting only a host-declared unsupported mode.
    pub fn require_cursor(&self, cursor: CursorCapability) -> Result<(), Error> {
        require_supported(self.cursor(cursor), || {
            Error::new(
                ErrorCode::UnsupportedFeature,
                format!("native host does not support {cursor:?} cursor"),
            )
        })
    }

    /// Returns the support classification for one mutable window operation.
    #[must_use]
    pub const fn window(&self, operation: WindowOperation) -> CapabilitySupport {
        self.window_operations[operation.index()]
    }

    /// Requires one window operation, rejecting only a host-declared unsupported operation.
    pub fn require_window(&self, operation: WindowOperation) -> Result<(), Error> {
        require_supported(self.window(operation), || {
            Error::new(
                ErrorCode::UnsupportedFeature,
                format!("native host does not support {operation:?} window operation"),
            )
        })
    }

    /// Returns the support classification for one cursor-grab mode.
    #[must_use]
    pub const fn cursor_grab(&self, grab: CursorGrab) -> CapabilitySupport {
        self.cursor_grabs[cursor_grab_index(grab)]
    }

    /// Requires one cursor-grab mode, rejecting only a host-declared unsupported mode.
    pub fn require_cursor_grab(&self, grab: CursorGrab) -> Result<(), Error> {
        require_supported(self.cursor_grab(grab), || {
            Error::new(
                ErrorCode::UnsupportedFeature,
                format!("native host does not support {grab:?} cursor grab"),
            )
        })
    }

    /// Returns the support classification for one IME semantic.
    #[must_use]
    pub const fn ime(&self, capability: ImeCapability) -> CapabilitySupport {
        self.ime_capabilities[capability.index()]
    }

    /// Requires one IME semantic, rejecting only a host-declared unsupported semantic.
    pub fn require_ime(&self, capability: ImeCapability) -> Result<(), Error> {
        require_supported(self.ime(capability), || {
            Error::new(
                ErrorCode::UnsupportedFeature,
                format!("native host does not support {capability:?} IME capability"),
            )
        })
    }

    /// Returns the selected host's accessibility support classification.
    #[must_use]
    pub const fn accessibility(&self) -> CapabilitySupport {
        self.accessibility
    }

    /// Requires accessibility, rejecting only a host-declared unsupported request.
    pub fn require_accessibility(&self) -> Result<(), Error> {
        require_supported(self.accessibility(), || {
            Error::new(
                ErrorCode::UnsupportedFeature,
                "native host does not support accessibility",
            )
        })
    }
}

fn require_supported(
    support: CapabilitySupport,
    unsupported: impl FnOnce() -> Error,
) -> Result<(), Error> {
    match support {
        CapabilitySupport::Supported | CapabilitySupport::RuntimeDependent => Ok(()),
        CapabilitySupport::Unsupported => Err(unsupported()),
    }
}

/// Builder for one explicit immutable host capability report.
///
/// All dimensions begin as [`CapabilitySupport::Unsupported`]. Each setter
/// records one selected-target classification; [`Self::build`] returns that
/// complete immutable report.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HostCapabilitiesBuilder {
    capabilities: HostCapabilities,
}

impl HostCapabilitiesBuilder {
    const fn new() -> Self {
        Self {
            capabilities: HostCapabilities {
                roles: [CapabilitySupport::Unsupported; ROLE_COUNT],
                fullscreens: [CapabilitySupport::Unsupported; FULLSCREEN_COUNT],
                cursors: [CapabilitySupport::Unsupported; CURSOR_COUNT],
                window_operations: [CapabilitySupport::Unsupported; WINDOW_OPERATION_COUNT],
                cursor_grabs: [CapabilitySupport::Unsupported; CURSOR_GRAB_COUNT],
                ime_capabilities: [CapabilitySupport::Unsupported; IME_CAPABILITY_COUNT],
                accessibility: CapabilitySupport::Unsupported,
            },
        }
    }

    /// Records support for a role dimension.
    #[must_use]
    pub const fn role(mut self, role: RoleKind, support: CapabilitySupport) -> Self {
        self.capabilities.roles[role.index()] = support;
        self
    }

    /// Records support for a fullscreen-mode dimension.
    #[must_use]
    pub const fn fullscreen(
        mut self,
        fullscreen: FullscreenMode,
        support: CapabilitySupport,
    ) -> Self {
        self.capabilities.fullscreens[fullscreen.index()] = support;
        self
    }

    /// Records support for a cursor-presentation dimension.
    #[must_use]
    pub const fn cursor(mut self, cursor: CursorCapability, support: CapabilitySupport) -> Self {
        self.capabilities.cursors[cursor.index()] = support;
        self
    }

    /// Records support for one mutable window-operation dimension.
    #[must_use]
    pub const fn window(mut self, operation: WindowOperation, support: CapabilitySupport) -> Self {
        self.capabilities.window_operations[operation.index()] = support;
        self
    }

    /// Records support for one cursor-grab-mode dimension.
    #[must_use]
    pub const fn cursor_grab(mut self, grab: CursorGrab, support: CapabilitySupport) -> Self {
        self.capabilities.cursor_grabs[cursor_grab_index(grab)] = support;
        self
    }

    /// Records support for one IME semantic dimension.
    #[must_use]
    pub const fn ime(mut self, capability: ImeCapability, support: CapabilitySupport) -> Self {
        self.capabilities.ime_capabilities[capability.index()] = support;
        self
    }

    /// Records the selected target's accessibility support classification.
    #[must_use]
    pub const fn accessibility(mut self, support: CapabilitySupport) -> Self {
        self.capabilities.accessibility = support;
        self
    }

    /// Returns the completed immutable capability report.
    #[must_use]
    pub const fn build(self) -> HostCapabilities {
        self.capabilities
    }
}

pub(crate) trait CapabilityProvider {
    fn resolve(&mut self) -> HostCapabilities;
}
