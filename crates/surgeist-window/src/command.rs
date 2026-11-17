//! Authored window commands and the host command plan resolved from them.
//!
//! A [`Command`] preserves what the application asked for, and
//! [`Command::kind`] gives its [`CommandKind`] classification, which errors
//! carry. A [`HostCommandPlan`] is the opaque, capability-resolved form a
//! backend applies.

pub use super::planning::HostCommandPlan;

use crate::cursor::{Cursor, CursorGrab};
use crate::descriptor::{Controls, Fullscreen, Level, Theme, WindowRequest};
use crate::event::ImeRequest;
use crate::geometry::{Point, Size};
use crate::registry::Id;

/// App-authored request for a window operation.
///
/// Commands preserve caller intent. They are normalized, planned against host
/// capabilities and runtime state, then applied by a backend; callers do not
/// construct the resolved host plan directly.
#[derive(Clone, Debug, PartialEq)]
pub enum Command {
    /// Opens a window from an authored request.
    Open {
        /// Request whose authored options are normalized before host application.
        request: WindowRequest,
    },
    /// Changes a live window's title.
    SetTitle {
        /// Target live window identity.
        id: Id,
        /// New title text.
        title: String,
    },
    /// Changes a live window's logical position.
    SetPosition {
        /// Target live window identity.
        id: Id,
        /// Logical position to validate and apply.
        position: Point,
    },
    /// Changes whether a live window is visible.
    SetVisible {
        /// Target live window identity.
        id: Id,
        /// Requested visibility.
        visible: bool,
    },
    /// Changes whether a live window may be resized.
    SetResizable {
        /// Target live window identity.
        id: Id,
        /// Requested resizable state.
        resizable: bool,
    },
    /// Changes the live window's platform controls.
    SetControls {
        /// Target live window identity.
        id: Id,
        /// Requested controls configuration.
        controls: Controls,
    },
    /// Changes whether platform decorations are shown.
    SetDecorations {
        /// Target live window identity.
        id: Id,
        /// Requested decoration state.
        decorations: bool,
    },
    /// Changes whether the native window uses transparency.
    SetTransparent {
        /// Target live window identity.
        id: Id,
        /// Requested transparency state.
        transparent: bool,
    },
    /// Changes a live window's logical inner size.
    SetInnerSize {
        /// Target live window identity.
        id: Id,
        /// Requested logical inner size.
        size: Size,
    },
    /// Changes the optional lower logical inner-size bound.
    SetMinInnerSize {
        /// Target live window identity.
        id: Id,
        /// New lower bound, or `None` to clear it.
        size: Option<Size>,
    },
    /// Changes the optional upper logical inner-size bound.
    SetMaxInnerSize {
        /// Target live window identity.
        id: Id,
        /// New upper bound, or `None` to clear it.
        size: Option<Size>,
    },
    /// Changes the fullscreen mode of a live window.
    SetFullscreen {
        /// Target live window identity.
        id: Id,
        /// Requested fullscreen mode.
        fullscreen: Fullscreen,
    },
    /// Changes the platform stacking level of a live window.
    SetLevel {
        /// Target live window identity.
        id: Id,
        /// Requested level.
        level: Level,
    },
    /// Changes the optional application theme override.
    SetTheme {
        /// Target live window identity.
        id: Id,
        /// Theme override, or `None` to use host observation.
        theme: Option<Theme>,
    },
    /// Changes the cursor used while the pointer is over a live window.
    SetCursor {
        /// Target live window identity.
        id: Id,
        /// Requested cursor representation.
        cursor: Cursor,
    },
    /// Changes the cursor-grab behavior for a live window.
    SetCursorGrab {
        /// Target live window identity.
        id: Id,
        /// Requested grab mode.
        grab: CursorGrab,
    },
    /// Changes the input-method editor configuration for a live window.
    SetIme {
        /// Target live window identity.
        id: Id,
        /// Authored IME request, resolved against host capability during planning.
        request: ImeRequest,
    },
    /// Submits an AccessKit tree update for one window.
    #[cfg(feature = "accessibility")]
    UpdateAccessibility {
        /// Target live window identity.
        id: Id,
        /// Typed AccessKit tree update submitted through the optional adapter.
        update: accesskit::TreeUpdate,
    },
    /// Requests platform user attention for a live window.
    RequestUserAttention {
        /// Target live window identity.
        id: Id,
    },
    /// Requests a draw for a live window.
    RequestDraw {
        /// Target live window identity.
        id: Id,
    },
    /// Starts destruction of a live window.
    Destroy {
        /// Target live window identity.
        id: Id,
    },
}

/// Stable semantic diagnostic for an authored window command.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum CommandKind {
    /// Classification for [`Command::Open`].
    Open,
    /// Classification for [`Command::SetTitle`].
    SetTitle,
    /// Classification for [`Command::SetPosition`].
    SetPosition,
    /// Classification for [`Command::SetVisible`].
    SetVisible,
    /// Classification for [`Command::SetResizable`].
    SetResizable,
    /// Classification for [`Command::SetControls`].
    SetControls,
    /// Classification for [`Command::SetDecorations`].
    SetDecorations,
    /// Classification for [`Command::SetTransparent`].
    SetTransparent,
    /// Classification for [`Command::SetInnerSize`].
    SetInnerSize,
    /// Classification for [`Command::SetMinInnerSize`].
    SetMinInnerSize,
    /// Classification for [`Command::SetMaxInnerSize`].
    SetMaxInnerSize,
    /// Classification for [`Command::SetFullscreen`].
    SetFullscreen,
    /// Classification for [`Command::SetLevel`].
    SetLevel,
    /// Classification for [`Command::SetTheme`].
    SetTheme,
    /// Classification for [`Command::SetCursor`].
    SetCursor,
    /// Classification for [`Command::SetCursorGrab`].
    SetCursorGrab,
    /// Classification for [`Command::SetIme`].
    SetIme,
    /// Semantic diagnostic for an AccessKit tree update.
    #[cfg(feature = "accessibility")]
    UpdateAccessibility,
    /// Classification for [`Command::RequestUserAttention`].
    RequestUserAttention,
    /// Classification for [`Command::RequestDraw`].
    RequestDraw,
    /// Classification for [`Command::Destroy`].
    Destroy,
}

impl Command {
    /// Returns this command's stable semantic classification.
    #[must_use]
    pub const fn kind(&self) -> CommandKind {
        match self {
            Self::Open { .. } => CommandKind::Open,
            Self::SetTitle { .. } => CommandKind::SetTitle,
            Self::SetPosition { .. } => CommandKind::SetPosition,
            Self::SetVisible { .. } => CommandKind::SetVisible,
            Self::SetResizable { .. } => CommandKind::SetResizable,
            Self::SetControls { .. } => CommandKind::SetControls,
            Self::SetDecorations { .. } => CommandKind::SetDecorations,
            Self::SetTransparent { .. } => CommandKind::SetTransparent,
            Self::SetInnerSize { .. } => CommandKind::SetInnerSize,
            Self::SetMinInnerSize { .. } => CommandKind::SetMinInnerSize,
            Self::SetMaxInnerSize { .. } => CommandKind::SetMaxInnerSize,
            Self::SetFullscreen { .. } => CommandKind::SetFullscreen,
            Self::SetLevel { .. } => CommandKind::SetLevel,
            Self::SetTheme { .. } => CommandKind::SetTheme,
            Self::SetCursor { .. } => CommandKind::SetCursor,
            Self::SetCursorGrab { .. } => CommandKind::SetCursorGrab,
            Self::SetIme { .. } => CommandKind::SetIme,
            #[cfg(feature = "accessibility")]
            Self::UpdateAccessibility { .. } => CommandKind::UpdateAccessibility,
            Self::RequestUserAttention { .. } => CommandKind::RequestUserAttention,
            Self::RequestDraw { .. } => CommandKind::RequestDraw,
            Self::Destroy { .. } => CommandKind::Destroy,
        }
    }

    /// Returns the target identity, or `None` for [`Command::Open`].
    #[must_use]
    pub const fn target(&self) -> Option<Id> {
        match self {
            Self::Open { .. } => None,
            #[cfg(feature = "accessibility")]
            Self::UpdateAccessibility { id, .. } => Some(*id),
            Self::SetTitle { id, .. }
            | Self::SetPosition { id, .. }
            | Self::SetVisible { id, .. }
            | Self::SetResizable { id, .. }
            | Self::SetControls { id, .. }
            | Self::SetDecorations { id, .. }
            | Self::SetTransparent { id, .. }
            | Self::SetInnerSize { id, .. }
            | Self::SetMinInnerSize { id, .. }
            | Self::SetMaxInnerSize { id, .. }
            | Self::SetFullscreen { id, .. }
            | Self::SetLevel { id, .. }
            | Self::SetTheme { id, .. }
            | Self::SetCursor { id, .. }
            | Self::SetCursorGrab { id, .. }
            | Self::SetIme { id, .. }
            | Self::RequestUserAttention { id }
            | Self::RequestDraw { id }
            | Self::Destroy { id } => Some(*id),
        }
    }
}
