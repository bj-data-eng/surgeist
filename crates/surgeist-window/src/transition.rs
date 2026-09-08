use super::{
    Error, ErrorCode, EventKind, Id, InputEvent, Metrics, ModifierState, PhysicalPoint, Point,
    PointerDeviceData, PointerEvent, PointerKind, PointerPhase, Theme, WindowSnapshot,
};

/// Validated state change reported by a native host.
///
/// Applying a patch first validates that its target identity matches the
/// snapshot, then updates a cloned snapshot and commits it atomically. Derived
/// events therefore observe committed state; validation failure changes neither
/// state nor callback delivery.
#[derive(Clone, Debug, PartialEq)]
pub enum WindowStatePatch {
    /// Replaces the title for one target window.
    Title {
        /// Target window identity.
        id: Id,
        /// Replacement title text.
        title: String,
    },
    /// Replaces the logical position for one target window.
    Position {
        /// Target window identity.
        id: Id,
        /// Replacement logical position.
        position: Point,
    },
    /// Replaces the visibility observation for one target window.
    Visible {
        /// Target window identity.
        id: Id,
        /// Replacement visibility state.
        visible: bool,
    },
    /// Replaces the complete metrics observation and selects its derived event.
    Metrics {
        /// Metrics whose identity must match the target snapshot.
        metrics: Metrics,
        /// Event kind derived after the metrics have committed.
        event: MetricsEvent,
    },
    /// Replaces the focus observation for one target window.
    Focused {
        /// Target window identity.
        id: Id,
        /// Replacement focus state.
        focused: bool,
    },
    /// Replaces the optional theme observation for one target window.
    Theme {
        /// Target window identity.
        id: Id,
        /// Replacement theme observation.
        theme: Option<Theme>,
    },
    /// Replaces the occlusion observation for one target window.
    Occluded {
        /// Target window identity.
        id: Id,
        /// Replacement occlusion state.
        occluded: bool,
    },
    /// Replaces the fullscreen observation for one target window.
    Fullscreen {
        /// Target window identity.
        id: Id,
        /// Replacement fullscreen state.
        fullscreen: bool,
    },
}

/// Event selected by a committed metrics patch.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MetricsEvent {
    /// Derives [`EventKind::Resized`] after applying the new metrics.
    Resized,
    /// Derives [`EventKind::ScaleFactorChanged`] after applying the new metrics.
    ScaleFactorChanged,
}

/// Canonical native input or state transition for one window.
///
/// Constructors pair a supported native fact with its validation and event
/// routing. There is no public arbitrary patch/event constructor. Applying a
/// transition rejects a mismatched identity before mutation and returns the
/// derived event only after state is coherent, so a failed transition reaches no
/// callback.
#[derive(Clone, Debug, PartialEq)]
pub struct NativeEventTransition {
    kind: NativeEventTransitionKind,
}

#[derive(Clone, Debug, PartialEq)]
enum NativeEventTransitionKind {
    Patch(WindowStatePatch),
    Input(InputEvent),
}

impl NativeEventTransition {
    const fn patch(patch: WindowStatePatch) -> Self {
        Self {
            kind: NativeEventTransitionKind::Patch(patch),
        }
    }

    #[must_use]
    /// Creates a focus-state transition for `id`.
    pub fn focused(id: Id, focused: bool) -> Self {
        let patch = WindowStatePatch::Focused { id, focused };
        Self::patch(patch)
    }

    #[must_use]
    /// Creates a logical-position transition for `id`.
    pub fn moved(id: Id, position: Point) -> Self {
        let patch = WindowStatePatch::Position { id, position };
        Self::patch(patch)
    }

    #[must_use]
    /// Creates a theme-observation transition for `id`.
    pub fn theme_changed(id: Id, theme: Option<Theme>) -> Self {
        let patch = WindowStatePatch::Theme { id, theme };
        Self::patch(patch)
    }

    #[must_use]
    /// Creates an occlusion-observation transition for `id`.
    pub fn occluded(id: Id, occluded: bool) -> Self {
        let patch = WindowStatePatch::Occluded { id, occluded };
        Self::patch(patch)
    }

    #[must_use]
    /// Creates a resize transition from complete metrics in logical and physical units.
    pub fn resized(metrics: Metrics) -> Self {
        let patch = WindowStatePatch::metrics(metrics, MetricsEvent::Resized);
        Self::patch(patch)
    }

    #[must_use]
    /// Creates a scale-factor transition from complete metrics in logical and physical units.
    pub fn scale_factor_changed(metrics: Metrics) -> Self {
        let patch = WindowStatePatch::metrics(metrics, MetricsEvent::ScaleFactorChanged);
        Self::patch(patch)
    }

    #[must_use]
    /// Creates a mouse-move input transition preserving both coordinate units.
    ///
    /// `position` and optional `delta` are logical points; `physical_position`
    /// is device pixels. This input does not mutate the window snapshot.
    pub fn mouse_moved(
        id: Id,
        position: Point,
        physical_position: PhysicalPoint,
        delta: Option<Point>,
        modifiers: ModifierState,
    ) -> Self {
        Self {
            kind: NativeEventTransitionKind::Input(InputEvent::Pointer(PointerEvent {
                id,
                phase: PointerPhase::Moved,
                kind: PointerKind::Mouse,
                pointer_id: None,
                position: Some(position),
                physical_position: Some(physical_position),
                delta,
                button: None,
                modifiers,
                device: PointerDeviceData::default(),
                timestamp: None,
            })),
        }
    }

    /// Applies this transition and derives its event from the committed snapshot.
    ///
    /// A target mismatch or invalid patch returns [`ErrorCode::InvalidRequest`]
    /// without changing `snapshot`. Patch events are derived after the atomic
    /// state update; mouse input is returned as [`EventKind::Input`].
    pub fn apply(self, snapshot: &mut WindowSnapshot) -> Result<EventKind, Error> {
        let id = self.id();
        if snapshot.id() != id {
            return Err(Error::new(
                ErrorCode::InvalidRequest,
                "transition target does not match window",
            )
            .with_id(id));
        }

        match self.kind {
            NativeEventTransitionKind::Patch(patch) => patch.apply(snapshot)?.ok_or_else(|| {
                Error::new(
                    ErrorCode::InvalidRequest,
                    "native transition does not produce an event",
                )
                .with_id(id)
            }),
            NativeEventTransitionKind::Input(input) => Ok(EventKind::Input(input)),
        }
    }

    #[must_use]
    pub(crate) fn id(&self) -> Id {
        match &self.kind {
            NativeEventTransitionKind::Patch(patch) => patch.id(),
            NativeEventTransitionKind::Input(input) => input.id(),
        }
    }

    #[cfg(test)]
    #[must_use]
    pub(crate) const fn requires_specialized_dispatch(&self) -> bool {
        matches!(
            &self.kind,
            NativeEventTransitionKind::Patch(WindowStatePatch::Metrics { .. })
                | NativeEventTransitionKind::Input(_)
        )
    }
}

impl WindowStatePatch {
    #[must_use]
    /// Creates a title patch for `id`.
    pub fn title(id: Id, title: impl Into<String>) -> Self {
        Self::Title {
            id,
            title: title.into(),
        }
    }

    #[must_use]
    /// Creates a visibility patch for `id`.
    pub const fn visible(id: Id, visible: bool) -> Self {
        Self::Visible { id, visible }
    }

    #[must_use]
    /// Creates a metrics patch whose metrics identify the target window.
    pub const fn metrics(metrics: Metrics, event: MetricsEvent) -> Self {
        Self::Metrics { metrics, event }
    }

    #[must_use]
    /// Returns the identity this patch must match before it can commit.
    pub const fn id(&self) -> Id {
        match self {
            Self::Title { id, .. }
            | Self::Position { id, .. }
            | Self::Visible { id, .. }
            | Self::Focused { id, .. }
            | Self::Theme { id, .. }
            | Self::Occluded { id, .. }
            | Self::Fullscreen { id, .. } => *id,
            Self::Metrics { metrics, .. } => metrics.id(),
        }
    }

    /// Validates and atomically applies this patch to `snapshot`.
    ///
    /// On failure the snapshot is unchanged and no derived event is returned.
    /// A successful patch may intentionally have no generic event.
    pub fn apply(self, snapshot: &mut WindowSnapshot) -> Result<Option<EventKind>, Error> {
        let id = self.id();
        if snapshot.id() != id {
            return Err(Error::new(
                ErrorCode::InvalidRequest,
                "patch target does not match window",
            )
            .with_id(id));
        }

        let mut next = snapshot.clone();
        match &self {
            Self::Title { title, .. } => next.set_title(title.clone()),
            Self::Position { position, .. } => next.set_position(Some(*position))?,
            Self::Visible { visible, .. } => next.set_visible(Some(*visible)),
            Self::Metrics { metrics, .. } => next.set_metrics(metrics.clone())?,
            Self::Focused { focused, .. } => next.set_focused(*focused),
            Self::Theme { theme, .. } => next.set_theme(*theme),
            Self::Occluded { occluded, .. } => next.set_occluded(Some(*occluded)),
            Self::Fullscreen { fullscreen, .. } => next.set_fullscreen(*fullscreen),
        }

        *snapshot = next;
        Ok(self.derived_event(snapshot))
    }

    fn derived_event(&self, snapshot: &WindowSnapshot) -> Option<EventKind> {
        match self {
            Self::Title { .. } | Self::Visible { .. } | Self::Fullscreen { .. } => None,
            Self::Position { .. } => snapshot.position().map(|position| EventKind::Moved {
                id: snapshot.id(),
                position,
            }),
            Self::Metrics { event, .. } => match event {
                MetricsEvent::Resized => Some(EventKind::Resized(snapshot.metrics().clone())),
                MetricsEvent::ScaleFactorChanged => {
                    Some(EventKind::ScaleFactorChanged(snapshot.metrics().clone()))
                }
            },
            Self::Focused { .. } => Some(EventKind::Focused {
                id: snapshot.id(),
                focused: snapshot.is_focused(),
            }),
            Self::Theme { .. } => Some(EventKind::ThemeChanged {
                id: snapshot.id(),
                theme: snapshot.theme(),
            }),
            Self::Occluded { .. } => Some(EventKind::Occluded {
                id: snapshot.id(),
                occluded: snapshot.is_occluded(),
            }),
        }
    }
}
