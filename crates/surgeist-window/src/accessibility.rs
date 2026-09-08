use super::Id;

/// An accessibility lifecycle or typed action event emitted for a window.
///
/// This feature-gated event is owned by the window host boundary: application
/// handlers observe it through [`crate::EventKind::Accessibility`] and retain
/// the supplied [`Id`] when submitting tree updates or handling actions. Native
/// adapter delivery occurs only for a live window whose host capability report
/// permits accessibility; callback failure still follows the normal terminal
/// lifecycle and suppresses later callbacks.
#[derive(Clone, Debug, PartialEq)]
pub enum AccessibilityEvent {
    /// Requests the complete initial tree for this live window generation.
    ///
    /// The window is waiting for its initial tree when the callback receives
    /// this event. Submit one update through [`crate::Context::update_accessibility`]
    /// using this exact id; it must include `TreeUpdate::tree`, its root node,
    /// root-tree id, and focus. The tree becomes active only after the callback
    /// succeeds and the queued update is planned and applied.
    InitialTreeRequested(Id),
    /// Delivers one typed AccessKit action requested for a live window.
    ///
    /// The request carries both the target window identity and the complete
    /// AccessKit action payload. It does not itself change tree phase; handlers
    /// decide any corresponding application state change and later tree update.
    ActionRequested(AccessibilityActionRequest),
    /// Reports that the native host has deactivated this window's tree.
    ///
    /// The tree returns to its absent phase before the callback runs. A later
    /// [`Self::InitialTreeRequested`] for the same still-live window requires a
    /// new complete initial update rather than an incremental update.
    Deactivated(Id),
}

impl AccessibilityEvent {
    /// Returns the live window identity to which this event belongs.
    ///
    /// Use this identity when responding to an initial-tree request; it is not
    /// an authored name and becomes stale when the window leaves its live
    /// lifecycle.
    #[must_use]
    pub const fn id(&self) -> Id {
        match self {
            Self::InitialTreeRequested(id) | Self::Deactivated(id) => *id,
            Self::ActionRequested(request) => request.id,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
/// A typed accessibility action request for a window.
///
/// This wrapper preserves the target window identity and complete
/// `accesskit::ActionRequest` without lossy conversion. It is delivered only
/// through [`AccessibilityEvent::ActionRequested`] while the feature is
/// enabled; applications own the semantic response and any subsequent tree
/// update.
pub struct AccessibilityActionRequest {
    /// The live window generation that received the action request.
    ///
    /// This is the target identity for the application response, not the
    /// AccessKit tree or node identity stored in [`Self::request`].
    pub id: Id,
    /// The complete typed request supplied by AccessKit.
    ///
    /// Its target tree, node, action, and optional data are preserved exactly
    /// so the application can interpret the request in its own semantic model.
    pub request: accesskit::ActionRequest,
}
