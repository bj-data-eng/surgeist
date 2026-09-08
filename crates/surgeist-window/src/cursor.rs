use super::{CursorCapability, CursorIcon};

/// Requested cursor presentation for a window.
///
/// The selected host's [`CursorCapability`] determines whether each form is
/// usable; this type expresses intent and does not guarantee backend behavior.
#[derive(Clone, Debug, PartialEq)]
pub enum Cursor {
    /// Select a built-in cursor icon.
    Icon(CursorIcon),
    /// Request that the cursor be hidden.
    Hidden,
    /// Request the supplied opaque authored custom-cursor identifier.
    ///
    /// A generic host capability report may support custom identifiers. The
    /// current `winit` target reports [`CursorCapability::Custom`] unsupported,
    /// so planning rejects this request before native execution.
    Custom(CustomCursorId),
}

impl Cursor {
    /// Returns the target-sensitive capability needed by this cursor request.
    #[must_use]
    pub const fn capability(&self) -> CursorCapability {
        match self {
            Self::Icon(_) => CursorCapability::Icon,
            Self::Hidden => CursorCapability::Hidden,
            Self::Custom(_) => CursorCapability::Custom,
        }
    }
}

/// Opaque raw authored identifier for a custom cursor.
///
/// This value has no registration relationship or native-resource meaning in
/// this crate. Whether a host can interpret it is determined by
/// [`CursorCapability::Custom`].
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct CustomCursorId(u64);

impl CustomCursorId {
    /// Creates a custom-cursor identifier from its raw numeric representation.
    #[must_use]
    pub const fn from_u64(value: u64) -> Self {
        Self(value)
    }
}

/// Requested pointer-confinement mode.
///
/// The host may report any mode unsupported. In particular, locked grabbing
/// does not imply relative-motion events or any backend-specific guarantee.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum CursorGrab {
    /// Do not request cursor confinement.
    #[default]
    None,
    /// Request confinement within the window bounds.
    Confined,
    /// Request locked cursor positioning.
    Locked,
}
