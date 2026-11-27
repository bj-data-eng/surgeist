//! Canonical ownership for a bounded native document tree.
//!
//! One [`Dom`] exclusively mutates its documents. Batches publish atomically,
//! and owned snapshots retain their exact identities and data after later edits.
//! Native XML and complete HTML document creation, exact namespaced import and
//! same-store adoption are supported. HTML API casing follows the current node
//! document while stored names and namespace-aware operations remain exact.
//! Explicitly initialized View Transitions retain canonical
//! structural state and real CSSOM associations, without rendering resources.
//! Namespace-aware common HTML/SVG/MathML elements retain actual shared common
//! state and nonce lifecycle. Native ShadowRoots retain distinct host/ordinary
//! relations and follow atomic adoption, clone/import and owned destruction.
//! Native Canvas-none and Marquee families retain HTML-owned state and execute
//! their actual attribute/value transitions without allocating pixels or animation.
//! Native Frame retains actual null child-content and consumes source-qualified
//! post-connection, src and removing decisions without navigation capability.
//! Native Slot retains manual intent and stored assignment separately from fresh
//! assignedSlot/flattened/CSS flat queries. An explicit intrinsic checkpoint takes
//! slotchange intentions; no callbacks run inside a transaction. Remaining special
//! families require their real adapters.
//! Native Fieldset retains actual association/custom-error state and bounded
//! source reset hooks. Custom messages normalize through HTML's shared rule;
//! semantic capture and Slot supplementation retain actual control facts.
//! Native Button shares those controls and retains independent weak reflected
//! targets, source-ordered setters, scoped semantic getters and retirement cleanup.
//! Native Input retains actual current/dirty/check/selection/file/Image state,
//! all shared type sanitizers, radio groups and one weak popover slot. Explicit
//! native platform facts and finite numeric/CSS processing remain independent.
//! Native Textarea retains exact raw value and shared flags/selection; bounded
//! API/default reads, setters/reset, immediate Text hooks and capture preserve
//! current/default distinctions without a second value owner or parser history.
//! Native Output stores one nullable default override and consumes the shared
//! ordered setter/reset cursor with real string-replace-all hooks. Existing
//! bounded semantic queries read exact ordinary descendant/default text, and
//! both initial and Slot captures preserve null versus established empty.
//! Native Template owns real separate contents and cached inert Documents;
//! clone/adopt/retirement preserve source ownership and document distinctions.
//! Native HTML Script retains complete supported state and real document history.
//! Source-ordered preparation reaches the actual Disabled return without fetching
//! or executing; attributes, children and CharacterData invoke their real hooks.
//! Native SVG Script separately retains the exact processed flag and Native/Clone
//! cause. The shared static shadow-including insertion walk consumes the actual
//! node Document's private acquisition/execution qualification and a fresh
//! operational recipient check. Processing refuses applicable Rust Load/Error
//! listeners without adding a loader, executor or SVG event output.
//! Explicit native HTML fragment installation preserves immutable component
//! fields and UTF-16 through that same canonical owner, including real native
//! Template contents and Script hooks. It returns a source-qualified historical
//! map and installed snapshot. Completed offline parser reports use separate
//! canonical installation entries that preserve actual retained history and
//! qualified contextual effects, with typed refusal for unsupported profiles.
//!
//! # Modules
//!
//! Five items sit at the crate root: [`Dom`] (the exclusive owner), [`DomBatch`]
//! (an ordered edit), [`DomCommit`] (the result of a published batch),
//! [`DomSnapshot`] (the owned read model) and [`DomError`] (the shared failure type).
//! Every other public item has exactly one path, in the module below that owns it.
//! Read the module documentation for the contract of each domain.
//!
//! | Module | Holds |
//! | --- | --- |
//! | [`capture`] | Immutable captures for semantic queries and host observations of external state. |
//! | [`custom_element`] | Custom element registries, definitions, reactions and the Source constructor protocol. |
//! | [`details`] | Details element state and its toggle intentions. |
//! | [`dialog`] | Dialog state, the dialog task queue, close watchers and the dialog Source requests. |
//! | [`document`] | Document class, interface, creation cause, environment and metadata. |
//! | [`edit`] | Operands and results of batch edits, change coverage, child-list observers and node-retirement watches. |
//! | [`event`] | Event parents, recipients, dispatch custody and the Source event firing protocol. |
//! | [`file`](mod@file) | Native selected File and FileList values and identities. |
//! | [`focus`] | Focus requests and observations, autofocus history and the focus-delegate protocol. |
//! | [`form`] | Form-associated element state: form, button, fieldset, input, output and textarea. |
//! | [`form::select`] | Select, option and selectedcontent state and the select task protocol. |
//! | [`fullscreen`] | Fullscreen model and exit protocol, and top-layer state. |
//! | [`hosted`] | Hosted lifecycle: identities, limits, creation and permissions policy. |
//! | [`hosted::child`] | Child navigables created and removed inside a hosted document. |
//! | [`hosted::destruction`] | Hosted document destruction and its proofs. |
//! | [`hosted::history`] | Session history entries, navigables and queued history jobs. |
//! | [`hosted::sheet`] | Constructed stylesheets and style-root withdrawal for a hosted document. |
//! | [`identity`] | Checked identities, revisions and stamps that qualify every read and change. |
//! | [`installation`] | Native HTML fragment installation and the completed parser-report installation. |
//! | [`installation::meta`] | Pending Meta associations of a parsed report. |
//! | [`installation::style`] | Pending Style stylesheet associations of a parsed report: prefix, notification, sheet, subject, successor and terminal. |
//! | [`limits`] | Finite admission policy for one DOM owner and its usage accounting. |
//! | [`media`] | Unloaded media observations and the stable-state removal continuation. |
//! | [`meta`] | Meta element state, document policy and CSP, and the native Meta composition. |
//! | [`range`] | Ranges, their boundaries and operands, and the designMode range transition. |
//! | [`script`] | HTML and SVG Script state, Document scripting registration and the Document constructor. |
//! | [`shadow`] | Shadow roots, slots, the flat tree and Template contents. |
//! | [`source`] | The domain-independent Source continuation: edit, request, input, limits, lease, status and cleanup. |
//! | [`source::attribute`] | Attribute write verification hooks of the Source continuation. |
//! | [`style`] | Style element association: sheets, sources, state, CSSOM roots and notifications. |
//! | [`style::inline`] | Inline-style identity bound to CSSOM, its capture, mutation and feedback. |
//! | [`task`] | Element task sources and the native checkpoint that releases queued tasks. |
//! | [`tree`] | The read model of the canonical tree: documents, nodes, elements, names, attributes and node iteration. |
//! | [`url`] | URL-bearing elements: Base, Link, hyperlinks and images, and the Source URL requests. |
//! | [`view_transition`] | View Transition structural state, pseudo-elements, rules and sheets. |
//!
//! ```
//! use std::sync::atomic::AtomicBool;
//! use surgeist_dom::limits::DomLimits;
//! use surgeist_dom::{Dom, DomBatch};
//!
//! let names = surgeist_dom::limits::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576)?;
//! let mut dom = Dom::new(DomLimits::new(32, 4096, 64, 0, 0, 8, 8192, names)?)?;
//! let mut batch = DomBatch::new();
//! let document = batch.create_document();
//! let comment = batch.create_comment(&document, "retained");
//! batch.insert_before(&comment, &document, None);
//! let committed = dom.apply(batch, &AtomicBool::new(false))?;
//! let comment = committed.created(&comment)?.expect("surviving creation");
//! let snapshot = dom.snapshot();
//! assert!(snapshot.node(comment)?.is_connected());
//! # Ok::<(), surgeist_dom::DomError>(())
//! ```
//!
//! ```
//! use std::sync::atomic::AtomicBool;
//! use surgeist_dom::limits::DomLimits;
//! use surgeist_dom::{Dom, DomBatch};
//! use surgeist_html::values::Namespace;
//! let names = surgeist_dom::limits::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576)?;
//! let mut dom = Dom::new(DomLimits::new(32, 4096, 64, 0, 0, 8, 8192, names)?)?;
//! let mut batch = DomBatch::new();
//! let document = batch.create_document();
//! let root = batch.create_element(&document, "Root");
//! batch.set_attribute_ns(&root, Namespace::from_uri("urn:example"), "p:key", "value");
//! batch.insert_before(&root, &document, None);
//! let committed = dom.apply(batch, &AtomicBool::new(false))?;
//! let snapshot = dom.snapshot();
//! let root = snapshot.node(committed.created(&root)?.unwrap())?;
//! assert_eq!(root.node_name().to_utf8()?, "Root");
//! assert_eq!(root.attributes().unwrap().len(), 1);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! ```
//! use std::sync::atomic::AtomicBool;
//! use surgeist_dom::limits::DomLimits;
//! use surgeist_dom::shadow::DomSlotQueryLimits;
//! use surgeist_dom::task::{DomNativeCheckpoint, DomNativeCheckpointLimits};
//! use surgeist_dom::{Dom, DomBatch};
//! use surgeist_html::construction::{
//!     HtmlShadowRootMode, HtmlShadowRootProperties, HtmlSlotAssignment};
//! use surgeist_html::values::Namespace;
//! let names = surgeist_dom::limits::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576)?;
//! let mut dom = Dom::new(DomLimits::new(32, 4096, 64, 64, 32, 8, 8192, names)?)?;
//! let mut batch = DomBatch::new();
//! let document = batch.create_document();
//! let host = batch.create_element_ns(&document, Namespace::html(), "div");
//! batch.insert_before(&host, &document, None);
//! let root = batch.attach_shadow(&host, HtmlShadowRootProperties::new(
//!     HtmlShadowRootMode::Open, HtmlSlotAssignment::Manual, false, false, false));
//! let slot = batch.create_element_ns(&document, Namespace::html(), "slot");
//! let text = batch.create_text(&document, "assigned");
//! batch.insert_before(&slot, &root, None);
//! batch.insert_before(&text, &host, None);
//! batch.assign_slot(&slot, vec![(&text).into()]);
//! let committed = dom.apply(batch, &AtomicBool::new(false))?;
//! let slot = committed.created(&slot)?.unwrap();
//! let text = committed.created(&text)?.unwrap();
//! assert_eq!(dom.snapshot().slot(slot)?.assigned_nodes(), &[text]);
//! let DomNativeCheckpoint::Slots(notifications) = dom.native_checkpoint_step(
//!     DomNativeCheckpointLimits::new(0, 0, DomSlotQueryLimits::new(32, 64)?)?, &AtomicBool::new(false))? else { panic!("reached Slot marker") };
//! assert_eq!(notifications.targets(), &[slot]);
//! # Ok::<(), surgeist_dom::DomError>(())
//! ```
#![forbid(unsafe_code)]
#![deny(unfulfilled_lint_expectations)]

pub mod capture;
pub mod custom_element;
pub mod details;
pub mod dialog;
pub mod document;
pub mod edit;
pub mod event;
pub mod file;
pub mod focus;
pub mod form;
pub mod fullscreen;
pub mod hosted;
pub mod identity;
pub mod installation;
pub mod limits;
pub mod media;
pub mod meta;
pub mod range;
pub mod script;
pub mod shadow;
pub mod source;
pub mod style;
pub mod task;
pub mod tree;
pub mod url;
pub mod view_transition;

mod internal;

pub use crate::internal::batch::DomBatch;
pub use crate::internal::snapshot::DomSnapshot;
pub use crate::internal::store::{Dom, DomCommit, DomError};
