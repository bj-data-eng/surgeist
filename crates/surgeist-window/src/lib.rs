#![deny(missing_docs)]
//! Native window boundary for Surgeist.
//!
//! This module owns the stable app-facing contract around `winit`: native window
//! identity, events, commands, metrics, draw scheduling, clipboard fallback,
//! and live native handles for renderer integration. It does not own renderers,
//! surfaces, UI semantics, layout, hit testing, or application behavior.
//!
//! ## Where things live
//!
//! The crate root holds the minimal program and its failure type: [`app`] and
//! [`App`] run a handler, [`open`] and [`Open`] author a window, [`Handler`]
//! receives the callbacks, [`Error`], [`ErrorCode`] and [`Result`] describe
//! failure, and [`Instant`] is the clock for deadlines. Every other public item
//! has exactly one path, in the module that owns its domain, and a root path to
//! a module item does not resolve.
//!
//! | Module | Domain |
//! | --- | --- |
//! | [`callback`] | Scope values handed to handler callbacks, window targets and context. |
//! | [`descriptor`] | Window requests, controls, level, theme, snapshot and metrics. |
//! | [`command`] | Authored commands and resolved host command plans. |
//! | [`event`] | Event, pointer, wheel, key, magnification, file-drag and input-method payloads. |
//! | [`geometry`] | Logical and physical points, sizes, rectangles and insets. |
//! | [`registry`] | Window identity, the live registry, handles, native handle access and the proxy. |
//! | [`capability`] | Host capability reports and decisions. |
//! | [`environment`] | Scale, viewport, screen, input and preference observations. |
//! | [`cursor`] | Cursor icons, custom cursors and grabs. |
//! | [`clipboard`] | The clipboard contract and its in-memory fallback. |
//! | [`host`] | The native event loop and native state transitions. |
//! | [`native_geometry`] | Host-authorized CSS-pixel geometry requests. |
//! | [`system_fonts`] | Host UI font observations. |
//! | `accessibility` | With the `accessibility` feature: typed events and the `accesskit` types. |
//! | [`testing`] | The display-free `Runner` and `Host`. |
//!
//! Four crates that appear in public signatures are re-exported where their
//! types are used, because the manifest pins each to an exact version:
//! `keyboard_types` in [`event`], `raw_window_handle` in [`registry`],
//! `CursorIcon` in [`cursor`] and, with the feature, `accesskit` in
//! `accessibility`.
//!
//! ## Migration
//!
//! `Open::modal` now directly authors a dialog. Replace
//! `.dialog(parent).modal(modality)` with `.modal(parent, modality)`.
//! The scope resizing-status accessor has been removed; observe resize callbacks instead.
//! Host capability reports are built explicitly with [`HostCapabilities::builder`](capability::HostCapabilities::builder),
//! and handlers inspect the resolved target report with [`Context::capabilities`](callback::Context::capabilities).
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
//!     Handler, Result,
//!     callback::{Close, Closed, Event, Frame, Ready},
//!     event::EventKind,
//!     open,
//!     registry::Id,
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
//! Semantic AccessKit types are available through the `accessibility::accesskit`
//! re-export; the native adapter remains an implementation detail.
//!
//! ```
//! # #[cfg(feature = "accessibility")]
//! # {
//! use surgeist_window::{
//!     Handler, Result,
//!     accessibility::{AccessibilityEvent, accesskit::{Node, NodeId, Role, Tree, TreeId, TreeUpdate}},
//!     callback::Event,
//!     capability::{CapabilitySupport, FullscreenMode, HostCapabilities, RoleKind},
//!     command::Command,
//!     event::EventKind,
//!     open,
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
//!     submitted_for: Option<surgeist_window::registry::Id>,
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

#[cfg(feature = "accessibility")]
pub mod accessibility;
pub mod callback;
pub mod capability;
pub mod clipboard;
pub mod command;
pub mod cursor;
pub mod descriptor;
pub mod environment;
pub mod event;
pub mod geometry;
pub mod host;
pub mod native_geometry;
pub mod registry;
pub mod system_fonts;
pub mod testing;

mod action;
mod context;
mod dsl;
mod environment_native;
mod error;
mod handler;
mod lease;
mod loop_;
mod native_input;
mod native_monitor;
mod native_preferences;
mod normalization;
mod planning;
mod pointer_coordinates;
mod pump;
mod scheduler;
mod transition;
mod winit_adapter;
mod winit_mapping;

pub use dsl::{App, Open, app, open};
pub use error::{Error, ErrorCode, Result};
pub use handler::Handler;

/// Platform monotonic clock shared by Window deadlines and input receipt times.
///
/// On native targets this is exactly [`std::time::Instant`]. Browser Wasm uses
/// the host context's `Performance.now()` clock and requires its `Performance`
/// object. This is not a DOM timestamp or a universal epoch shared by unrelated
/// contexts; atomics-enabled browser workers use the upstream time-origin
/// adjustment. Callers must create deadlines in the owning host clock domain.
///
/// The type is re-exported so that deadline callers on every target name one
/// type without a target-conditional dependency on `web-time`. The manifest
/// pins that dependency to an exact version, so a caller's own copy at another
/// version would be a different type on browser Wasm.
pub use web_time::Instant;

pub(crate) use registry::UserEvent;
pub(crate) use scheduler::DrawScheduler;

#[cfg(test)]
mod environment_tests;
#[cfg(test)]
mod magnification_tests;
#[cfg(test)]
mod tests;
