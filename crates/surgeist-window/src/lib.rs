#![deny(missing_docs)]
//! Native window boundary for Surgeist.
//!
//! This module owns the stable app-facing contract around `winit`: native window
//! identity, events, commands, metrics, draw scheduling, clipboard fallback,
//! and live native handles for renderer integration. It does not own renderers,
//! surfaces, UI semantics, layout, hit testing, or application behavior.
//!
//! ## Migration
//!
//! `Open::modal` now directly authors a dialog. Replace
//! `.dialog(parent).modal(modality)` with `.modal(parent, modality)`.
//! The scope resizing-status accessor has been removed; observe resize callbacks instead.
//! Host capability reports are built explicitly with [`HostCapabilities::builder`],
//! and handlers inspect the resolved target report with [`Context::capabilities`].
//!
//! ## Deterministic lifecycle
//!
//! [`testing::Runner`] drives the public lifecycle without opening a native display.
//! This complete example authors an open request, observes `Created` then `Ready`,
//! reads committed state, delivers a draw, accepts a close request, and exits from
//! `Closed` with a successful terminal result.
//!
//! ```
//! use surgeist_window::{
//!     Close, Closed, Event, EventKind, Frame, Handler, Id, Ready, Result, open,
//!     testing::Runner,
//! };
//!
//! #[derive(Default)]
//! struct Lifecycle {
//!     callbacks: Vec<&'static str>,
//!     id: Option<Id>,
//!     title: Option<String>,
//! }
//!
//! impl Handler for Lifecycle {
//!     fn event(&mut self, event: &mut Event<'_>) -> Result<()> {
//!         if matches!(event.event(), EventKind::Created(_)) {
//!             self.callbacks.push("created");
//!             self.id = Some(event.id());
//!             self.title = event.state().map(|state| state.title().to_owned());
//!         }
//!         Ok(())
//!     }
//!
//!     fn ready(&mut self, ready: &mut Ready<'_>) -> Result<()> {
//!         self.callbacks.push("ready");
//!         assert_eq!(ready.state().title(), "Lifecycle");
//!         Ok(())
//!     }
//!
//!     fn draw(&mut self, frame: &mut Frame<'_>) -> Result<()> {
//!         self.callbacks.push("draw");
//!         assert_eq!(frame.id(), self.id.expect("created window identity"));
//!         assert_eq!(frame.state().title(), "Lifecycle");
//!         Ok(())
//!     }
//!
//!     fn close(&mut self, close: &mut Close<'_>) -> Result<()> {
//!         self.callbacks.push("close");
//!         close.close();
//!         Ok(())
//!     }
//!
//!     fn closed(&mut self, closed: &mut Closed<'_>) -> Result<()> {
//!         self.callbacks.push("closed");
//!         assert_eq!(closed.id(), self.id.expect("created window identity"));
//!         assert_eq!(closed.state().title(), "Lifecycle");
//!         closed.exit();
//!         Ok(())
//!     }
//! }
//!
//! let mut runner = Runner::new(Lifecycle::default());
//! runner
//!     .startup(vec![open("main").title("Lifecycle").into()])
//!     .expect("startup is configured before the first resume");
//! runner.resume();
//!
//! let id = runner.handler().id.expect("created window identity");
//! assert_eq!(runner.handler().id, Some(id));
//! assert_eq!(runner.handler().title.as_deref(), Some("Lifecycle"));
//! assert_eq!(runner.handler().callbacks, ["created", "ready"]);
//!
//! runner.draw(id);
//! assert_eq!(runner.handler().callbacks, ["created", "ready", "draw"]);
//!
//! runner.request_close(id);
//! assert_eq!(
//!     runner.handler().callbacks,
//!     ["created", "ready", "draw", "close", "closed"]
//! );
//! assert!(matches!(runner.result(), Some(Ok(()))));
//! ```
//!
//! ## Accessibility
//!
//! Enable the `accessibility` feature to receive typed accessibility events and
//! submit AccessKit tree updates through `Context::update_accessibility`.
//! Semantic AccessKit types are available through the `accesskit` re-export; the native
//! adapter remains an implementation detail.
//!
//! ```
//! # #[cfg(feature = "accessibility")]
//! # {
//! use surgeist_window::{
//!     AccessibilityEvent, CapabilitySupport, Event, EventKind, FullscreenMode,
//!     Command, Handler, HostCapabilities, Result, RoleKind,
//!     accesskit::{Node, NodeId, Role, Tree, TreeId, TreeUpdate}, open,
//!     testing::Runner,
//! };
//!
//! let root_id = NodeId(1);
//! let mut root = Node::new(Role::Window);
//! root.set_label("Accessible window");
//! let initial_tree = TreeUpdate {
//!     nodes: vec![(root_id, root)],
//!     tree: Some(Tree::new(root_id)),
//!     tree_id: TreeId::ROOT,
//!     focus: root_id,
//! };
//!
//! struct AccessibilityHandler {
//!     initial_tree: TreeUpdate,
//!     submitted_for: Option<surgeist_window::Id>,
//! }
//!
//! impl Handler for AccessibilityHandler {
//!     fn event(&mut self, event: &mut Event<'_>) -> Result<()> {
//!         let EventKind::Accessibility(AccessibilityEvent::InitialTreeRequested(id)) = event.event() else {
//!             return Ok(());
//!         };
//!         let id = *id;
//!         event
//!             .context_mut()
//!             .update_accessibility(id, self.initial_tree.clone());
//!         self.submitted_for = Some(id);
//!         Ok(())
//!     }
//! }
//!
//! let capabilities = HostCapabilities::builder()
//!     .role(RoleKind::Root, CapabilitySupport::Supported)
//!     .fullscreen(FullscreenMode::None, CapabilitySupport::Supported)
//!     .accessibility(CapabilitySupport::Supported)
//!     .build();
//! let expected_update = initial_tree.clone();
//! let mut runner = Runner::with_capabilities(
//!     AccessibilityHandler { initial_tree, submitted_for: None },
//!     capabilities,
//! );
//! runner
//!     .startup(vec![open("accessible").into()])
//!     .expect("startup is configured before the first resume");
//! runner.resume();
//! let id = runner
//!     .host()
//!     .window_id("accessible")
//!     .expect("resume opens the live accessibility target");
//! runner.accessibility(AccessibilityEvent::InitialTreeRequested(id));
//! assert_eq!(runner.handler().submitted_for, Some(id));
//! assert_eq!(
//!     runner.host().commands().last(),
//!     Some(&Command::UpdateAccessibility {
//!         id,
//!         update: expected_update,
//!     })
//! );
//! assert!(runner.result().is_none());
//! # }
//! ```
//!
//! The native host delivers `InitialTreeRequested` only after the window is
//! live and ready to accept its first tree; use that event's id rather than an
//! authored name or a stored id. The example's [`testing::Runner`] lifecycle is
//! display-free; with this feature it can inject the matching typed parity event
//! after resume and observe the applied update without a native display. After
//! the initial update has been applied, subsequent updates for the active tree
//! may omit `TreeUpdate::tree`; after deactivation, wait for another initial-tree
//! request and submit a full update again.

/// AccessKit semantic types used by the optional accessibility contract.
///
/// This re-export is available only with the `accessibility` feature; it keeps
/// application tree construction independent from the crate's private native adapter.
#[cfg(feature = "accessibility")]
pub use accesskit;
pub use cursor_icon::CursorIcon;
pub use keyboard_types;
pub use keyboard_types::Code;
pub use raw_window_handle;

#[cfg(feature = "accessibility")]
mod accessibility;
mod capability;
mod clipboard;
mod command;
mod context;
mod cursor;
mod descriptor;
mod dsl;
mod error;
mod event;
mod geometry;
mod handler;
mod lease;
mod loop_;
mod normalization;
mod planning;
mod pump;
mod registry;
mod scheduler;
pub mod testing;
mod transition;
mod winit_adapter;
mod winit_mapping;

#[cfg(feature = "accessibility")]
pub use accessibility::{AccessibilityActionRequest, AccessibilityEvent};
pub use capability::{
    CapabilityDecision, CapabilityKind, CapabilitySupport, CursorCapability, FullscreenMode,
    HostCapabilities, HostCapabilitiesBuilder, ImeCapability, RoleKind, WindowOperation,
};
pub use clipboard::{Clipboard, ClipboardImage, ClipboardImageRef, MemoryClipboard};
pub use command::{Command, CommandKind};
pub use context::Context;
pub use cursor::{Cursor, CursorGrab, CustomCursorId};
pub use descriptor::{
    Controls, Fullscreen, Level, Metrics, Modality, Role, Theme, WindowRequest,
    WindowRequestBuilder, WindowSnapshot,
};
pub use dsl::{
    App, Close, Closed, ControlsBuilder, Event, Frame, Input, Open, Ready, Resize, Scope, Selector,
    Target, app, controls, open, point, rect, size,
};
pub use error::{Error, ErrorCode, Result};
pub use event::{
    EventKind, FileDragEvent, ImeConfig, ImeEvent, ImeHint, ImePurpose, ImeRequest,
    ImeSurroundingText, InputEvent, KeyEvent, KeyState, ModifierState, PointerButton,
    PointerDeviceData, PointerEvent, PointerKind, PointerPhase, StandardKeyBindingEvent,
    TouchPhase, WheelDelta, WheelEvent,
};
pub use geometry::{Id, Insets, PhysicalPoint, PhysicalSize, Point, Rect, Size};
pub use handler::Handler;
pub use loop_::Loop;
pub use planning::HostCommandPlan;
pub use registry::{Access, Handle, Instance, Proxy, Ref, Registry};
pub use transition::{MetricsEvent, NativeEventTransition, WindowStatePatch};

pub(crate) use registry::UserEvent;
pub(crate) use scheduler::DrawScheduler;

#[cfg(test)]
mod tests;
