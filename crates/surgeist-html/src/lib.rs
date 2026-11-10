//! HTML-owned construction values and language contracts.
//!
//! The selected WHATWG HTML edition defines language behavior. Immutable values
//! preserve exact UTF-16 code units; live document identity and mutation belong
//! to the consuming DOM subsystem.
//!
//! # Modules
//!
//! - [`accounting`]: the work and resource dimensions, conversion rates and cap
//!   scopes that the HTML crates charge through `surgeist-ledger`.
//! - [`values`]: document strings, names, attributes and other immutable
//!   construction values.
//! - [`vocabulary`]: exact name recognition and the specification's inventories.
//! - [`microsyntax`]: numbers, temporal values, colors and tokens.
//! - [`attributes`]: interpretation of individual attributes.
//! - [`elements`]: element-specific rules, by element family.
//! - [`focus`]: document-level focus algorithms.
//! - [`security`]: Content Security Policy, nonces and opener policy.
//! - [`parsing`]: decisions the HTML parser consumes.
//! - [`construction`]: native authoring and immutable construction output.
//! - [`bounded`]: controlled operations that a caller drives step by step
//!   under its own admission policy. Each submodule mirrors a domain module;
//!   [`bounded::lexical`] holds the token cursors and ASCII helpers that several
//!   domains share.
//!
//! # Errors
//!
//! Every type that a public function returns in an `Err` implements `Debug`,
//! `Display` and [`std::error::Error`]. `Display` states the cause in prose and
//! never prints `Debug`. A wrapper's text describes its own operation and
//! returns the wrapped failure from `source`, so a reporter that walks the chain
//! never prints a cause twice. A public error has one of four shapes. Apply the
//! questions in order to predict the shape of any error:
//!
//! 1. Does the refusal hand back recoverable owners? Then it is a carrier
//!    struct with the reason and an `into_parts` or accessor, such as
//!    [`bounded::construction::HtmlConstructionRefusal`].
//! 2. Is it a misuse of an owner or cursor protocol (a consumed, not-ready,
//!    stale, wrong-reply or already-complete state), or does one type merge the
//!    refusals of many operations? Then it is a struct with a private cause,
//!    queried through `is_*` predicates and fact accessors, such as
//!    [`security::csp::HtmlCspProtocolError`], the interpretation
//!    protocol errors and the cursor errors like [`focus::HtmlFocusingError`]. New
//!    causes never break a caller.
//! 3. Does a driver charge or branch on every variant, and does a consumer
//!    match it exhaustively? Then it is an exhaustive enum: the resource
//!    accounting enums such as [`bounded::numeric::HtmlIntegerUnitStepError`] and
//!    [`bounded::numeric::HtmlFloatingPointAllocationError`], and the two-source wrappers
//!    [`security::csp::HtmlCspError`] and [`bounded::rel::HtmlRelActionError`], which separate the
//!    caller's own refusal from this crate's. [`construction::HtmlConstructionError`] is a
//!    struct whose [`cause`](construction::HtmlConstructionError::cause) is the exhaustive enum
//!    [`construction::HtmlConstructionCause`], because the parser maps every cause.
//! 4. Otherwise it names a rule of the pinned standard, a syntax class, an
//!    unavailability reason or a vocabulary that a driver constructs. Then it is
//!    a `#[non_exhaustive]` enum, such as [`microsyntax::tokens::HtmlTokenError`],
//!    and [`attributes::HtmlAttributeConformanceError`].
//!
//! An owned cause converts with `From`, so `?` applies it. `map_err` remains
//! where the wrapped value is the caller's generic error, because a blanket
//! `From` for it would overlap the protocol conversion.
//!
//! Errors do not capture a [`std::backtrace::Backtrace`]. Most are small `Clone`
//! and `Eq` values that bounded operations return under exact allocation
//! accounting, and a capture allocates outside that accounting and cannot be
//! compared. A recoverable parse diagnostic is a value, not an error, and never
//! pays for a capture. A caller that wants a backtrace captures it where it
//! converts this crate's error into its own.
//!
//! # API evolution
//!
//! Public enums are closed or open by what their variants enumerate:
//!
//! - Closed by the pinned standard: keyword and state sets that the HTML
//!   Standard enumerates, such as input types and token comparison. They are
//!   exhaustive. A new edition of the standard is a deliberate breaking change
//!   that must fail to compile in every consumer.
//! - Closed by a controlled-operation protocol: the action, reply, outcome,
//!   step and phase enums that an owner and its driver exchange, and the
//!   resource vocabularies a driver charges. They are exhaustive, because
//!   matching every variant is what proves that a driver services every action
//!   and charges every resource.
//! - Open: failure causes, rule verdicts, conformance violations and
//!   unavailability reasons grow with the standard and its checks. They are
//!   `#[non_exhaustive]`, so a downstream `match` needs a wildcard arm.
//!
//! Before the root `surgeist` API reaches 1.0 no compatibility alias or
//! migration shim is kept for a changed enum.
//!
//! ```
//! use surgeist_html::construction::{
//!     HtmlConstructionCause, HtmlConstructionLimits, HtmlConstructionResource, HtmlFragment,
//! };
//!
//! // One UTF-16 unit of text fits; eight do not.
//! let mut builder = HtmlFragment::builder(HtmlConstructionLimits::new(3, 2, 0, 2, 2))?;
//! let root = builder.root();
//! let refusal = builder.append_text(&root, "too long").unwrap_err();
//!
//! // The cause names the violated rule and carries its facts.
//! assert_eq!(
//!     refusal.cause(),
//!     HtmlConstructionCause::LimitExceeded {
//!         resource: HtmlConstructionResource::Utf16Units,
//!         limit: 2,
//!         attempted: 8,
//!     }
//! );
//! assert_eq!(
//!     refusal.to_string(),
//!     "UTF-16 unit limit of 2 cannot admit a total of 8"
//! );
//! # Ok::<(), surgeist_html::construction::HtmlConstructionError>(())
//! ```
#![forbid(unsafe_code)]
#![deny(unfulfilled_lint_expectations)]
#![warn(missing_docs)]

pub mod accounting;
pub mod attributes;
pub mod bounded;
pub mod construction;
pub mod elements;
pub mod focus;
pub mod microsyntax;
pub mod parsing;
pub mod security;
pub mod values;
pub mod vocabulary;

mod internal;
