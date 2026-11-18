//! Message text and `Error` contract of the document scripting error types.
//!
//! Each expected message is written from the documented meaning of the variant
//! and from every place the crate raises it. The enums here are
//! `#[non_exhaustive]`, so an integration test cannot match them exhaustively:
//! each index function has a wildcard arm and runs only on the rows that exist,
//! so a variant added without a row passes this file. The exhaustive `Display`
//! match inside the crate is what forces a message for a new variant. A
//! wrapping variant states only its own context and returns the wrapped error
//! from `source()`.
#![cfg(feature = "document-scripting")]
#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt::Debug;
use std::sync::{Arc, atomic::AtomicBool};

use surgeist_dom::{Dom, DomBatch, DomError, limits::DomLimits};
use surgeist_runtime::{
    DomEventConstructionError, DomEventDispatchError, DomEventTimestampUnavailable,
    DomHostedRuntimeError, DomSession,
    document_scripting::{
        DocumentGuestEventError, DocumentScriptError, DocumentScriptNativeError,
        EnableDocumentGuestEventsFailure, EnableDocumentScriptFailure, PreparedDocumentScriptHost,
    },
};

/// Checks one type's table: row order against `index` (an exhaustive match where the type allows one), the exact
/// message, no trailing period, no `Debug` form, and `source()` present exactly
/// for the wrapping variants. `skipped` lists row numbers whose payload cannot be
/// built through the public API.
fn check_table<E>(
    rows: Vec<(E, &'static str)>,
    index: fn(&E) -> usize,
    wraps: fn(&E) -> bool,
    skipped: &[usize],
) where
    E: Error + 'static + Debug,
{
    let mut position = 0;
    for (value, expected) in &rows {
        while skipped.contains(&position) {
            position += 1;
        }
        assert_eq!(index(value), position, "row order for {value:?}");
        position += 1;
        let text = value.to_string();
        assert_eq!(&text, expected, "message for {value:?}");
        assert!(!text.ends_with('.'), "no trailing period: {text}");
        assert!(
            text.chars().next().unwrap().is_lowercase(),
            "a lowercase clause: {text}"
        );
        assert_ne!(text, format!("{value:?}"), "message is the Debug form");
        assert_eq!(
            value.source().is_some(),
            wraps(value),
            "source presence for {value:?}"
        );
    }
}

/// `source()` returns exactly the wrapped error and the message does not repeat
/// its text.
fn assert_chains<E: Error>(outer: &E, inner: &dyn Error) {
    let source = outer.source().expect("a wrapping variant has a source");
    assert_eq!(source.to_string(), inner.to_string());
    assert!(
        !outer.to_string().contains(&inner.to_string()),
        "the wrapped error is reported by source(), not repeated: {outer}"
    );
}

#[test]
fn guest_event_errors_state_the_refused_guest_operation() {
    check_table(
        vec![
            (
                DocumentGuestEventError::Document(DocumentScriptError::Native(
                    DocumentScriptNativeError::Busy,
                )),
                "the document script host or the bindings refused the operation",
            ),
            (
                DocumentGuestEventError::Construction(
                    DomEventConstructionError::UnsupportedInterface,
                ),
                "the Event or EventTarget construction was refused",
            ),
            (
                DocumentGuestEventError::Timestamp(
                    DomEventTimestampUnavailable::NoAuthenticatedClock,
                ),
                "no native creation timestamp is available for the Event",
            ),
            (
                DocumentGuestEventError::Dispatch(DomEventDispatchError::CyclicPath),
                "native event dispatch or registration was refused",
            ),
            (
                DocumentGuestEventError::CallbackRealmUnavailable,
                "the engine Realm has no authority in this attachment",
            ),
            (
                DocumentGuestEventError::RegistryState,
                "the guest event registry or its retained state is missing or already in use",
            ),
            (
                DocumentGuestEventError::RegistryUnbound,
                "the supplied guest registry has not claimed the native DOM",
            ),
            (
                DocumentGuestEventError::WorkLimit,
                "guest adapter work was refused",
            ),
            (
                DocumentGuestEventError::Accounting(surgeist_ledger::Error::Allocation),
                "guest adapter accounting failed",
            ),
            (
                DocumentGuestEventError::HostRouteUnavailable,
                "the guest operation requires a host route that is unavailable",
            ),
            (
                DocumentGuestEventError::SourcePending(pending_source_operation()),
                "the original Source operation blocked synchronous completion",
            ),
            (
                DocumentGuestEventError::HostedPending(hosted_operation()),
                "the original hosted operation blocked synchronous completion",
            ),
        ],
        |error| match error {
            DocumentGuestEventError::Document(_) => 0,
            DocumentGuestEventError::Construction(_) => 1,
            DocumentGuestEventError::Timestamp(_) => 2,
            DocumentGuestEventError::Dispatch(_) => 3,
            DocumentGuestEventError::CallbackRealmUnavailable => 4,
            DocumentGuestEventError::RegistryState => 5,
            DocumentGuestEventError::RegistryUnbound => 6,
            DocumentGuestEventError::WorkLimit => 7,
            DocumentGuestEventError::Accounting(_) => 8,
            DocumentGuestEventError::HostRouteUnavailable => 9,
            DocumentGuestEventError::SourcePending(_) => 10,
            DocumentGuestEventError::HostedPending(_) => 11,
            // `#[non_exhaustive]`: this arm is never reached by the rows above; the
            // in-crate exhaustive `Display` match forces a message for a new variant.
            _ => unreachable!("every constructed DocumentGuestEventError row is listed"),
        },
        |error| {
            matches!(
                error,
                DocumentGuestEventError::Document(_)
                    | DocumentGuestEventError::Construction(_)
                    | DocumentGuestEventError::Timestamp(_)
                    | DocumentGuestEventError::Dispatch(_)
                    | DocumentGuestEventError::Accounting(_)
            )
        },
        &[],
    );
    assert_chains(
        &DocumentGuestEventError::Document(DocumentScriptError::Native(
            DocumentScriptNativeError::Busy,
        )),
        &DocumentScriptError::Native(DocumentScriptNativeError::Busy),
    );
    assert_chains(
        &DocumentGuestEventError::Construction(DomEventConstructionError::UnsupportedInterface),
        &DomEventConstructionError::UnsupportedInterface,
    );
    assert_chains(
        &DocumentGuestEventError::Timestamp(DomEventTimestampUnavailable::NoAuthenticatedClock),
        &DomEventTimestampUnavailable::NoAuthenticatedClock,
    );
    assert_chains(
        &DocumentGuestEventError::Dispatch(DomEventDispatchError::CyclicPath),
        &DomEventDispatchError::CyclicPath,
    );
}

#[test]
fn native_scripting_errors_state_the_refused_native_operation() {
    check_table(
        vec![
            (
                DocumentScriptNativeError::Hosted(DomHostedRuntimeError::Busy),
                "the native hosted lifecycle refused the operation",
            ),
            (
                DocumentScriptNativeError::Dom(DomError::StaleRange),
                "the canonical DOM refused the operation",
            ),
            (
                DocumentScriptNativeError::StorageLimit,
                "the requested adapter storage or pending count exceeds the configured limit or cannot be sized",
            ),
            (
                DocumentScriptNativeError::Busy,
                "a native owner or retained record needed by the operation is already in use or unavailable",
            ),
            (
                DocumentScriptNativeError::ForeignPermit,
                "the permit, origin or Realm does not belong to this attachment",
            ),
            (
                DocumentScriptNativeError::Revoked,
                "the attachment has stopped, expired or no longer has scripting enabled, or a record it retained is gone",
            ),
            (
                DocumentScriptNativeError::SourcePending,
                "a native Source or hosted operation has an unfinished synchronous obligation",
            ),
            (
                DocumentScriptNativeError::SourceAccess,
                "the pending Source operation, event, retained frame or delegated node the operation needs is missing, unfinished or does not match this hook",
            ),
            (
                DocumentScriptNativeError::CreationGlobalUnavailable,
                "the creation Global is unavailable in this attachment",
            ),
            (
                DocumentScriptNativeError::CreationRealmAlreadyRegistered,
                "the native Global already has a creation realm",
            ),
            (
                DocumentScriptNativeError::CreationRealmWorkLimit,
                "guest binding work for the creation realm was refused or cannot be represented",
            ),
            (
                DocumentScriptNativeError::ProjectionCustody,
                "adapter custody state is missing, inconsistent with what was admitted, or already failed",
            ),
            (
                DocumentScriptNativeError::RepresentationExhausted,
                "a size, count or accounting total cannot be represented or does not balance in the supported service profile",
            ),
            (
                DocumentScriptNativeError::WindowTimersDisabled,
                "this attachment did not enable the Window timer profile",
            ),
            (
                DocumentScriptNativeError::TrustedTypesPolicyUnavailable,
                "the Trusted Types policy evaluation is unavailable for the Window's current CSP",
            ),
        ],
        |error| match error {
            DocumentScriptNativeError::Hosted(_) => 0,
            DocumentScriptNativeError::Dom(_) => 1,
            DocumentScriptNativeError::StorageLimit => 2,
            DocumentScriptNativeError::Busy => 3,
            DocumentScriptNativeError::ForeignPermit => 4,
            DocumentScriptNativeError::Revoked => 5,
            DocumentScriptNativeError::SourcePending => 6,
            DocumentScriptNativeError::SourceAccess => 7,
            DocumentScriptNativeError::CreationGlobalUnavailable => 8,
            DocumentScriptNativeError::CreationRealmAlreadyRegistered => 9,
            DocumentScriptNativeError::CustomElementRegistryCustodyUnavailable(_) => 10,
            DocumentScriptNativeError::CreationRealmWorkLimit => 11,
            DocumentScriptNativeError::ProjectionCustody => 12,
            DocumentScriptNativeError::RepresentationExhausted => 13,
            DocumentScriptNativeError::WindowTimersDisabled => 14,
            DocumentScriptNativeError::TrustedTypesPolicyUnavailable => 15,
            // `#[non_exhaustive]`: this arm is never reached by the rows above; the
            // in-crate exhaustive `Display` match forces a message for a new variant.
            _ => unreachable!("every constructed DocumentScriptNativeError row is listed"),
        },
        |error| {
            matches!(
                error,
                DocumentScriptNativeError::Hosted(_) | DocumentScriptNativeError::Dom(_)
            )
        },
        &[10],
    );
    // `CustomElementRegistryCustodyUnavailable` (row 10) carries a custom element
    // registry id that only a Document registry can issue.
    assert_chains(
        &DocumentScriptNativeError::Hosted(DomHostedRuntimeError::Busy),
        &DomHostedRuntimeError::Busy,
    );
    assert_chains(
        &DocumentScriptNativeError::Dom(DomError::StaleRange),
        &DomError::StaleRange,
    );
}

// End to end: errors returned by public operations.

fn session() -> DomSession {
    let mut dom = Dom::new(
        DomLimits::new(
            64,
            4096,
            128,
            0,
            0,
            16,
            65536,
            surgeist_dom::limits::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576)
                .unwrap(),
        )
        .unwrap(),
    )
    .unwrap();
    let mut batch = DomBatch::new();
    batch.create_document();
    dom.apply(batch, &AtomicBool::new(false)).unwrap();
    DomSession::new(dom, session_limits()).unwrap()
}

fn session_limits() -> surgeist_runtime::DomSessionLimits {
    use surgeist_css::source::CssUtf16InputLimits;
    use surgeist_dom::hosted::{DomHostedLimits, DomHostedRecordLimits};
    use surgeist_runtime::{
        DomElementTaskRuntimeLimits, DomHostedRuntimeLimits, DomSessionLimits,
        DomStyleRuntimeLimits,
    };
    DomSessionLimits {
        authored_events: surgeist_runtime::DomEventStorageLimits {
            storage_bytes: 16_000_000,
            work: 1_000_000,
        },
        source_events: surgeist_runtime::DomSourceEventRuntimeLimits {
            jobs: 4,
            storage_bytes: 16_000_000,
            work: 1_000_000,
        },
        style: DomStyleRuntimeLimits {
            roots: 4,
            completions: 32,
            reports: 32,
            storage_bytes: 16_000_000,
            work: 1_000_000,
            css_input: CssUtf16InputLimits::default(),
        },
        element_tasks: DomElementTaskRuntimeLimits {
            tasks: 32,
            storage_bytes: 16_000_000,
            work: 1_000_000,
        },
        hosted: DomHostedRuntimeLimits {
            canonical: DomHostedLimits::new(
                DomHostedRecordLimits::new(32, 32, 32, 32, 32, 32, 32),
                16_000_000,
                16_000_000,
                1_000_000,
            ),
            tasks: 32,
            storage_bytes: 16_000_000,
            scratch_bytes: 16_000_000,
            work: 1_000_000,
        },
    }
}

#[test]
fn preparing_with_too_little_storage_returns_the_storage_refusal() {
    let failure =
        PreparedDocumentScriptHost::prepare(session(), 0, Arc::new(AtomicBool::new(false)))
            .unwrap_err();
    assert!(
        matches!(failure.error, DocumentScriptNativeError::StorageLimit),
        "{:?}",
        failure.error
    );
    assert_eq!(
        failure.error.to_string(),
        "the requested adapter storage or pending count exceeds the configured limit or cannot be sized"
    );
}

#[test]
fn a_failed_preparation_returns_the_session_and_chains_its_cause() {
    let failure =
        PreparedDocumentScriptHost::prepare(session(), 0, Arc::new(AtomicBool::new(false)))
            .unwrap_err();
    assert_eq!(
        failure.to_string(),
        "preparing document scripting failed and returned the original Session"
    );
    let source = failure.source().expect("the native refusal is the source");
    assert_eq!(
        source.to_string(),
        "the requested adapter storage or pending count exceeds the configured limit or cannot be sized"
    );
    assert!(!failure.to_string().contains("storage"));
    // The failure still owns the original Session, which the caller can reuse.
    let _session = failure.session;
}

fn prepared() -> PreparedDocumentScriptHost {
    PreparedDocumentScriptHost::prepare(session(), 1 << 20, Arc::new(AtomicBool::new(false)))
        .unwrap_or_else(|failure| panic!("preparation must succeed: {}", failure.error))
}

#[test]
fn a_failed_enable_returns_the_unpublished_owner_and_chains_its_cause() {
    let failure = EnableDocumentScriptFailure {
        owner: prepared(),
        error: DocumentScriptError::Native(DocumentScriptNativeError::Revoked),
    };
    assert_eq!(
        failure.to_string(),
        "enabling document scripting failed and returned the unpublished owner"
    );
    assert_chains(
        &failure,
        &DocumentScriptError::Native(DocumentScriptNativeError::Revoked),
    );

    let failure = EnableDocumentGuestEventsFailure {
        owner: prepared(),
        error: DocumentGuestEventError::RegistryUnbound,
    };
    assert_eq!(
        failure.to_string(),
        "enabling guest event registration failed and returned the unpublished owner"
    );
    assert_chains(&failure, &DocumentGuestEventError::RegistryUnbound);
}

#[test]
fn script_outcome_and_event_errors_state_their_own_context() {
    use surgeist_runtime::document_scripting::DocumentScriptEventError;
    use surgeist_runtime::{
        DomElementTaskRuntimeError, DomSourceRuntimeError, DomStyleRuntimeError,
    };
    let native = || DocumentScriptError::Native(DocumentScriptNativeError::Busy);
    check_table(
        vec![
            (native(), "a native entry or owner operation was refused"),
            (
                DocumentScriptError::Script(
                    surgeist_bindings_js::Error::PreparationAlreadyAdmitted,
                ),
                "the bindings execution outcome is an error",
            ),
        ],
        |error| match error {
            DocumentScriptError::Native(_) => 0,
            DocumentScriptError::Script(_) => 1,
            // `#[non_exhaustive]`: the in-crate exhaustive `Display` match forces a message.
            _ => unreachable!("every constructed DocumentScriptError row is listed"),
        },
        |_| true,
        &[],
    );
    assert_chains(&native(), &DocumentScriptNativeError::Busy);
    check_table(
        vec![
            (
                DocumentScriptEventError::Access(native()),
                "the script owner, its Source capability or the engine refused the operation",
            ),
            (
                DocumentScriptEventError::Service(DomSourceRuntimeError::Style(
                    DomStyleRuntimeError::Binding,
                )),
                "the native event adapter refused servicing",
            ),
            (
                DocumentScriptEventError::Task(DomElementTaskRuntimeError::Busy),
                "the element-task owner refused delivery or completion",
            ),
            (
                DocumentScriptEventError::Guest(DocumentGuestEventError::RegistryUnbound),
                "the attached registry or guest invocation refused delivery",
            ),
        ],
        |error| match error {
            DocumentScriptEventError::Access(_) => 0,
            DocumentScriptEventError::Service(_) => 1,
            DocumentScriptEventError::Task(_) => 2,
            DocumentScriptEventError::Guest(_) => 3,
            // `#[non_exhaustive]`: the in-crate exhaustive `Display` match forces a message.
            _ => unreachable!("every constructed DocumentScriptEventError row is listed"),
        },
        |_| true,
        &[],
    );
    assert_chains(
        &DocumentScriptEventError::Guest(DocumentGuestEventError::RegistryUnbound),
        &DocumentGuestEventError::RegistryUnbound,
    );
    // The guest event error that wraps the script error no longer prints the native text twice.
    assert_chains(&DocumentGuestEventError::Document(native()), &native());
}

/// The operation id of one started, unfinished Source edit.
fn pending_source_operation() -> surgeist_dom::source::DomSourceOperationId {
    use surgeist_dom::source::{DomSourceEdit, DomSourceLimits, DomSourceProgress};
    let mut dom = Dom::new(
        DomLimits::new(
            64,
            4096,
            128,
            0,
            0,
            16,
            65536,
            surgeist_dom::limits::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576)
                .unwrap(),
        )
        .unwrap(),
    )
    .unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let style =
        batch.create_element_ns(&document, surgeist_html::values::Namespace::html(), "style");
    let commit = dom.apply(batch, &AtomicBool::new(false)).unwrap();
    let document = commit.document(&document).unwrap();
    let style = commit.created(&style).unwrap().unwrap();
    let mut lease = dom
        .begin_source_edit(
            DomSourceEdit::insert_before(style, document.node(), None),
            DomSourceLimits::new(64_000_000, 1_000_000, 20_000_000, 128, 4096, 4096, 8),
            &AtomicBool::new(false),
        )
        .unwrap();
    assert!(matches!(
        lease.advance(&AtomicBool::new(false)).unwrap(),
        DomSourceProgress::Request(_)
    ));
    drop(lease);
    dom.pending_source_edit().unwrap().operation()
}

/// The operation id of one hosted top-level destruction.
fn hosted_operation() -> surgeist_dom::hosted::DomHostedOperationId {
    let dom = Dom::new(
        DomLimits::new(
            512,
            200_000,
            1024,
            1024,
            1024,
            64,
            4_000_000,
            surgeist_dom::limits::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576)
                .unwrap(),
        )
        .unwrap()
        .with_dispatch_custody(
            surgeist_dom::event::DomDispatchCustodyLimits::new(
                64, 4096, 16_000_000, 16_000_000, 4_000_000,
            )
            .unwrap(),
        )
        .with_details_toggle_intentions(32)
        .unwrap(),
    )
    .unwrap();
    let mut session = DomSession::new(dom, session_limits()).unwrap();
    let mut hosted = session.hosted_execution();
    let flag = AtomicBool::new(false);
    let host = hosted.create_host(&mut session.dom(), &flag).unwrap();
    let top = hosted
        .create_top_level(&mut session.dom(), host, &flag)
        .unwrap();
    hosted
        .destroy_top_level(&mut session.dom(), top.navigable(), &flag)
        .unwrap()
        .operation()
}
