#![forbid(unsafe_code)]
#![deny(unfulfilled_lint_expectations)]
#![warn(missing_docs)]

//! App runtime and authoring DSL boundary for Surgeist.
//!
//! This module coordinates deterministic app state, retained UI surfaces,
//! resources, tasks, services, native wakeups, and declared effects. Native
//! window mechanics stay with the host adapter.
//!
//! # Where things live
//!
//! Two items sit at the crate root: [`Runtime`], the deterministic owner of application state, surfaces, queues and diagnostics,
//! and [`DomSession`], the durable owner of the canonical DOM and its composed obligations, with the function [`crate_name`]. Every other public item has one path, in the module
//! that owns its domain; a root path to a module item does not resolve.
//!
//! | Module | Domain |
//! | --- | --- |
//! | [`manifest`] | Authored names, command and event values, descriptors and the app manifest. |
//! | [`reducer`], [`effect`] | State transitions and the effects they emit. |
//! | [`input`], [`drain`], [`proxy`] | Typed inputs and their queues, drain turns, producer wakeups. |
//! | [`identity`], [`task`], [`provenance`], [`diagnostic`] | Checked identities, task intents, input causality and diagnostics. |
//! | [`resource`], [`coordination`], [`service`], [`snapshot`] | Resource state, subscriptions, services and snapshots. |
//! | [`surface`], [`host_capture`] | UI surfaces and their binding to the canonical DOM. |
//! | [`session`], [`style`], [`source`], [`event`], [`hosted`], [`element_task`] | The DOM session and its composed owners. |
//! | [`trusted_types_csp`] | Trusted Types CSP decisions. |
//! | `document_scripting` | With the `document-scripting` feature: the document script host. |
//!
//! # Validated authoring names
//!
//! Command names, event names, and payload type names enter the public API through
//! fallible validation. Their text is preserved after validation:
//!
//! ```
//! use surgeist_runtime::manifest::{CommandName, EventName, PayloadTypeName};
//!
//! assert_eq!(CommandName::try_new("save")?.as_str(), "save");
//! assert_eq!(EventName::try_new("saved")?.as_str(), "saved");
//! assert_eq!(PayloadTypeName::try_new("Document")?.as_str(), "Document");
//! # Ok::<(), surgeist_runtime::manifest::NameError>(())
//! ```
//!
//! These types provide no unchecked `new` constructor:
//!
//! ```compile_fail
//! use surgeist_runtime::manifest::CommandName;
//! let _ = CommandName::new("save");
//! ```
//!
//! The same imports with the checked constructor compile:
//!
//! ```
//! use surgeist_runtime::manifest::CommandName;
//! let _ = CommandName::try_new("save");
//! ```
//!
//! ```compile_fail
//! use surgeist_runtime::manifest::EventName;
//! let _ = EventName::new("saved");
//! ```
//!
//! The same imports with the checked constructor compile:
//!
//! ```
//! use surgeist_runtime::manifest::EventName;
//! let _ = EventName::try_new("saved");
//! ```
//!
//! ```compile_fail
//! use surgeist_runtime::manifest::PayloadTypeName;
//! let _ = PayloadTypeName::new("Document");
//! ```
//!
//! The same imports with the checked constructor compile:
//!
//! ```
//! use surgeist_runtime::manifest::PayloadTypeName;
//! let _ = PayloadTypeName::try_new("Document");
//! ```
//!
//! Commands and events take validated names:
//!
//! ```
//! use surgeist_runtime::manifest::{AppCommand, AppEvent, CommandName, EventName};
//!
//! let command = AppCommand::named(CommandName::try_new("save")?);
//! let event = AppEvent::named(EventName::try_new("saved")?);
//! assert_eq!(command.name().as_str(), "save");
//! assert_eq!(event.name().as_str(), "saved");
//! # Ok::<(), surgeist_runtime::manifest::NameError>(())
//! ```
//!
//! A raw string cannot bypass that validation:
//!
//! ```compile_fail
//! use surgeist_runtime::manifest::AppCommand;
//! let _ = AppCommand::named("save");
//! ```
//!
//! The same imports with the checked constructor compile:
//!
//! ```
//! use surgeist_runtime::manifest::{AppCommand, CommandName};
//! let _ = AppCommand::named(CommandName::try_new("save")?);
//! # Ok::<(), surgeist_runtime::manifest::NameError>(())
//! ```
//!
//! ```compile_fail
//! use surgeist_runtime::manifest::AppEvent;
//! let _ = AppEvent::named("saved");
//! ```
//!
//! The same imports with the checked constructor compile:
//!
//! ```
//! use surgeist_runtime::manifest::{AppEvent, EventName};
//! let _ = AppEvent::named(EventName::try_new("saved")?);
//! # Ok::<(), surgeist_runtime::manifest::NameError>(())
//! ```
//!
//! Descriptor construction also validates names and semantic payload types:
//!
//! ```
//! use surgeist_runtime::{identity::ResourceId, manifest::{CommandDescriptor, EventDescriptor, ResourceDescriptor, TaskDescriptor}, task::TaskIntentName};
//!
//! let command = CommandDescriptor::try_new("save", "Document")?;
//! let event = EventDescriptor::try_new("saved", "Document")?;
//! let task = TaskDescriptor::try_new(TaskIntentName::new("load"), "Path")?;
//! let resource = ResourceDescriptor::try_new(ResourceId::new("document"), "Document")?;
//! assert_eq!(command.payload_type().as_str(), "Document");
//! assert_eq!(event.payload_type().as_str(), "Document");
//! assert_eq!(task.input_type().as_str(), "Path");
//! assert_eq!(resource.value_type().as_str(), "Document");
//! # Ok::<(), surgeist_runtime::manifest::NameError>(())
//! ```
//!
//! The former unchecked descriptor constructors are unavailable:
//!
//! ```compile_fail
//! use surgeist_runtime::manifest::CommandDescriptor;
//! let _ = CommandDescriptor::new("save", "Document");
//! ```
//!
//! The same imports with the checked constructor compile:
//!
//! ```
//! use surgeist_runtime::manifest::CommandDescriptor;
//! let _ = CommandDescriptor::try_new("save", "Document");
//! ```
//!
//! ```compile_fail
//! use surgeist_runtime::manifest::EventDescriptor;
//! let _ = EventDescriptor::new("saved", "Document");
//! ```
//!
//! The same imports with the checked constructor compile:
//!
//! ```
//! use surgeist_runtime::manifest::EventDescriptor;
//! let _ = EventDescriptor::try_new("saved", "Document");
//! ```
//!
//! ```compile_fail
//! use surgeist_runtime::{manifest::TaskDescriptor, task::TaskIntentName};
//! let name = TaskIntentName::new("load");
//! let _ = TaskDescriptor::new(name, "Path");
//! ```
//!
//! The same imports with the checked constructor compile:
//!
//! ```
//! use surgeist_runtime::{manifest::TaskDescriptor, task::TaskIntentName};
//! let name = TaskIntentName::new("load");
//! let _ = TaskDescriptor::try_new(name, "Path");
//! ```
//!
//! ```compile_fail
//! use surgeist_runtime::{identity::ResourceId, manifest::ResourceDescriptor};
//! let id = ResourceId::new("document");
//! let _ = ResourceDescriptor::new(id, "Document");
//! ```
//!
//! The same imports with the checked constructor compile:
//!
//! ```
//! use surgeist_runtime::{identity::ResourceId, manifest::ResourceDescriptor};
//! let id = ResourceId::new("document");
//! let _ = ResourceDescriptor::try_new(id, "Document");
//! ```
//!
//! # Host and test boundaries
//!
//! Hosts implement the public [`proxy::WakeBridge`] contract. Concrete retained-to-runtime
//! adapters belong to the composing application, so this crate has no public
//! `bridge` module:
//!
//! ```
//! use surgeist_runtime::proxy::{AppProxy, QueuePolicy, WakeBridge, WakeError};
//!
//! struct HostWake;
//! impl WakeBridge for HostWake {
//!     fn wake(&self) -> Result<(), WakeError> {
//!         Ok(())
//!     }
//! }
//! let _proxy = AppProxy::<()>::new(HostWake, QueuePolicy::default());
//! ```
//!
//! ```compile_fail
//! use surgeist_runtime::bridge;
//! ```
//!
//! External callers can exercise a runtime through its public reducer and input
//! contracts:
//!
//! ```
//! use surgeist_runtime::{Runtime, drain::RuntimeBudget, input::{AppInput, UiInput}, provenance::InputProvenance, reducer::{Reducer, ReducerCommit, ReducerResult}};
//!
//! struct KeepState;
//! impl Reducer<u8, ()> for KeepState {
//!     fn reduce(&mut self, _state: &u8, _input: &AppInput<()>) -> ReducerResult<u8> {
//!         ReducerResult::unchanged(ReducerCommit::new())
//!     }
//! }
//! let mut runtime = Runtime::new(7, KeepState);
//! runtime.enqueue_ui(UiInput::new((), InputProvenance::system()).unwrap()).unwrap();
//! runtime.drain_once(RuntimeBudget::default()).unwrap();
//! assert_eq!(runtime.state(), &7);
//! ```
//!
//! The crate's private test fixtures are unavailable to those callers:
//!
//! ```compile_fail
//! use surgeist_runtime::testing;
//! ```
//!
//! ```compile_fail
//! use surgeist_runtime::PrototypeApp;
//! ```
//!
//! These examples check the named public contracts. The root `surgeist`
//! repository owns the complete generated API audit.

pub mod coordination;
pub mod diagnostic;
#[cfg(feature = "document-scripting")]
pub mod document_scripting;
pub mod drain;
pub mod effect;
pub mod element_task;
pub mod event;
pub mod host_capture;
pub mod hosted;
pub mod identity;
pub mod input;
pub mod manifest;
pub mod provenance;
pub mod proxy;
pub mod reducer;
pub mod resource;
pub mod service;
pub mod session;
pub mod snapshot;
pub mod source;
pub mod style;
pub mod surface;
pub mod task;
pub mod trusted_types_csp;

mod internal;

pub use internal::dom_style::DomSession;
pub use internal::runtime::Runtime;

#[cfg(test)]
mod testing;

#[cfg(test)]
mod tests;

/// Returns the crate identity while the runtime API is being designed.
#[must_use]
pub const fn crate_name() -> &'static str {
    "surgeist-runtime"
}
