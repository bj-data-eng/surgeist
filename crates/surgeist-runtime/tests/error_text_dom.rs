//! Message text and `Error` contract of the DOM runtime owners' error types.
//!
//! Each expected message is written from the documented meaning of its variant
//! and from every place that raises it (see the raise-site list of the change
//! that introduced this file), never from the `Debug` form. For an enum that
//! is not `#[non_exhaustive]` an exhaustive `match` maps each variant to its
//! table row, so a new variant does not compile until it has a row. The one
//! `#[non_exhaustive]` enum here, `DomHostedRuntimeError`, cannot be matched
//! exhaustively from outside the crate: its index function has a wildcard arm
//! and runs only on the rows that exist, so a variant added without a row
//! passes this file. The exhaustive `Display` match inside the crate is what
//! forces a message for a new variant.
//! A variant that wraps an error states only its own context and returns the
//! wrapped error from `source()`.
#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt::Debug;
use std::sync::atomic::AtomicBool;

use surgeist_css::source::CssUtf16InputError;
use surgeist_cssom::CssomError;
use surgeist_dom::{
    Dom, DomBatch, DomError,
    hosted::{DomHostedError, sheet::DomHostedSheetConstructionError},
    identity::NodeId,
    installation::{
        DomNativeHtmlInstallError,
        style::{DomParsedHtmlFrontierError, DomParsedHtmlStyleError, DomParsedHtmlTerminalError},
    },
    limits::DomLimits,
    meta::DomSourceMetaCspError,
    style::{DomStyleCssomRootError, inline::DomInlineStyleError},
};
use surgeist_runtime::{
    DomConstructedSheetError, DomElementTaskRuntimeError, DomEventDispatchError, DomEventResource,
    DomHostedRuntimeError, DomSession, DomSessionCreateFailure, DomSourceEventResource,
    DomSourceEventRuntimeError, DomSourceRuntimeError, DomStyleParsedTerminalError,
    DomStyleRuntimeError, trusted_types_csp::PolicyListCapacity,
};

fn flag() -> AtomicBool {
    AtomicBool::new(false)
}

fn dom() -> (Dom, NodeId) {
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
    let element = batch.create_element(&document, "A");
    let commit = dom.apply(batch, &flag()).unwrap();
    let node = commit.created(&element).unwrap().unwrap();
    (dom, node)
}

fn node() -> NodeId {
    dom().1
}

fn some_dom_error() -> DomError {
    DomError::StaleRange
}

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
        assert!(
            !text.contains("{ ") && !text.contains("("),
            "no Debug structure in the message: {text}"
        );
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

fn assert_send_sync<E: Error + Send + Sync + 'static>() {}

#[test]
fn constructed_sheet_errors_state_the_refusing_owner() {
    let hosted = DomHostedRuntimeError::Busy;
    let style = DomStyleRuntimeError::Binding;
    let construction = DomHostedSheetConstructionError::Hosted(DomHostedError::ForeignSource);
    check_table(
        vec![
            (
                DomConstructedSheetError::Hosted(hosted.clone()),
                "the hosted runtime refused the constructed stylesheet operation",
            ),
            (
                DomConstructedSheetError::Style(DomStyleRuntimeError::Binding),
                "the Style runtime refused the constructed stylesheet operation",
            ),
            (
                DomConstructedSheetError::Construction(construction),
                "the native Document or CSSOM producer refused to construct the stylesheet",
            ),
        ],
        |error| match error {
            DomConstructedSheetError::Hosted(_) => 0,
            DomConstructedSheetError::Style(_) => 1,
            DomConstructedSheetError::Construction(_) => 2,
        },
        |_| true,
        &[],
    );
    assert_chains(&DomConstructedSheetError::Hosted(hosted.clone()), &hosted);
    assert_chains(
        &DomConstructedSheetError::Style(style),
        &DomStyleRuntimeError::Binding,
    );
}

#[test]
fn element_task_errors_state_the_refused_operation() {
    check_table(
        vec![
            (
                DomElementTaskRuntimeError::PretransferredSession,
                "standalone intake already transferred element-task obligations that this session cannot recover",
            ),
            (
                DomElementTaskRuntimeError::Binding,
                "the element-task owner, DOM or task binding is missing or does not match, or the retained task state does not match",
            ),
            (
                DomElementTaskRuntimeError::OwnerDisposed,
                "the session that owns the element-task state was disposed",
            ),
            (
                DomElementTaskRuntimeError::Busy,
                "the state needed by the element-task operation is already borrowed by another operation",
            ),
            (
                DomElementTaskRuntimeError::OutOfOrderTransfer,
                "the task transfer is older than a transfer from the same DOM that was already consumed",
            ),
            (
                DomElementTaskRuntimeError::Dom(some_dom_error()),
                "the canonical DOM refused the element-task operation",
            ),
            (
                DomElementTaskRuntimeError::Admission,
                "element-task admission refused the operation",
            ),
            (
                DomElementTaskRuntimeError::IdentityExhausted,
                "an element-task count, size or work total cannot be represented",
            ),
            (
                DomElementTaskRuntimeError::Dispatch(DomEventDispatchError::CyclicPath),
                "shared event dispatch refused the element task",
            ),
            (
                DomElementTaskRuntimeError::SourceEvent(DomSourceEventRuntimeError::Busy),
                "the Source event executor refused the element-task step",
            ),
            (
                DomElementTaskRuntimeError::Hosted(DomHostedRuntimeError::Busy),
                "the native hosted lifecycle refused the element-task operation",
            ),
        ],
        |error| match error {
            DomElementTaskRuntimeError::PretransferredSession => 0,
            DomElementTaskRuntimeError::Binding => 1,
            DomElementTaskRuntimeError::OwnerDisposed => 2,
            DomElementTaskRuntimeError::Busy => 3,
            DomElementTaskRuntimeError::OutOfOrderTransfer => 4,
            DomElementTaskRuntimeError::Dom(_) => 5,
            DomElementTaskRuntimeError::Admission => 6,
            DomElementTaskRuntimeError::IdentityExhausted => 7,
            DomElementTaskRuntimeError::Dispatch(_) => 8,
            DomElementTaskRuntimeError::SourceEvent(_) => 9,
            DomElementTaskRuntimeError::Hosted(_) => 10,
        },
        |error| {
            matches!(
                error,
                DomElementTaskRuntimeError::Dom(_)
                    | DomElementTaskRuntimeError::Dispatch(_)
                    | DomElementTaskRuntimeError::SourceEvent(_)
                    | DomElementTaskRuntimeError::Hosted(_)
            )
        },
        &[],
    );
    let inner = some_dom_error();
    assert_chains(&DomElementTaskRuntimeError::Dom(inner.clone()), &inner);
    assert_chains(
        &DomElementTaskRuntimeError::Dispatch(DomEventDispatchError::CyclicPath),
        &DomEventDispatchError::CyclicPath,
    );
    assert_chains(
        &DomElementTaskRuntimeError::SourceEvent(DomSourceEventRuntimeError::Busy),
        &DomSourceEventRuntimeError::Busy,
    );
    assert_chains(
        &DomElementTaskRuntimeError::Hosted(DomHostedRuntimeError::Busy),
        &DomHostedRuntimeError::Busy,
    );
}

#[test]
fn hosted_lifecycle_errors_state_the_refused_operation() {
    assert_send_sync::<DomHostedRuntimeError>();
    check_table(
        vec![
            (
                DomHostedRuntimeError::Dom(DomHostedError::ForeignSource),
                "a canonical DOM or hosted DOM error refused the hosted lifecycle operation",
            ),
            (
                DomHostedRuntimeError::OwnerDisposed,
                "the session that issued this hosted permission was disposed",
            ),
            (
                DomHostedRuntimeError::Busy,
                "the hosted lifecycle state, its event loop or the element-task state this operation needs is already in use",
            ),
            (
                DomHostedRuntimeError::Binding,
                "the hosted lifecycle permission, DOM view, clock source or target does not match the original binding, or is not ready",
            ),
            (
                DomHostedRuntimeError::Admission,
                "hosted lifecycle admission refused the operation",
            ),
            (
                DomHostedRuntimeError::ClockUnavailable,
                "this target has no native monotonic clock provider",
            ),
            (
                DomHostedRuntimeError::ClockRegressed,
                "the native monotonic clock regressed",
            ),
            (
                DomHostedRuntimeError::ClockOverflow,
                "the coarsened native time cannot be represented",
            ),
            (
                DomHostedRuntimeError::EntropyUnavailable,
                "native entropy is unavailable on this target",
            ),
            (
                DomHostedRuntimeError::EntropyFailed,
                "the native entropy operation failed",
            ),
            (
                DomHostedRuntimeError::Cancelled,
                "the hosted lifecycle operation was cancelled",
            ),
            (
                DomHostedRuntimeError::OperationPending,
                "a retained hosted operation, live hosted record or cleanup phase must be settled or reached before this operation can proceed",
            ),
            (
                DomHostedRuntimeError::TargetNotFound,
                "the live native host, Global, navigable or retained record this operation names was not found",
            ),
            (
                DomHostedRuntimeError::Protocol,
                "a retained hosted lifecycle record, phase or source identity is missing or disagrees with the request, or the element-task owner refused it",
            ),
            (
                DomHostedRuntimeError::Inactive,
                "the Document, Global or navigable this operation names is not currently active, ready or the expected top-level target",
            ),
            (
                DomHostedRuntimeError::CheckpointUnavailable,
                "native checkpoint work is unavailable while a Source or hosted operation or a slot signal is pending",
            ),
            (
                DomHostedRuntimeError::UrlUnavailable,
                "no qualified URL observation is available for the reached stage",
            ),
        ],
        |error| match error {
            DomHostedRuntimeError::Dom(_) => 0,
            DomHostedRuntimeError::OwnerDisposed => 1,
            DomHostedRuntimeError::Busy => 2,
            DomHostedRuntimeError::Binding => 3,
            DomHostedRuntimeError::Admission => 4,
            DomHostedRuntimeError::ClockUnavailable => 5,
            DomHostedRuntimeError::ClockRegressed => 6,
            DomHostedRuntimeError::ClockOverflow => 7,
            DomHostedRuntimeError::EntropyUnavailable => 8,
            DomHostedRuntimeError::EntropyFailed => 9,
            DomHostedRuntimeError::Cancelled => 10,
            DomHostedRuntimeError::OperationPending => 11,
            DomHostedRuntimeError::TargetNotFound => 12,
            DomHostedRuntimeError::Protocol => 13,
            DomHostedRuntimeError::Inactive => 14,
            DomHostedRuntimeError::CheckpointUnavailable => 15,
            DomHostedRuntimeError::UrlUnavailable => 16,
            // `#[non_exhaustive]`: this arm is never reached by the rows above; the
            // in-crate exhaustive `Display` match forces a message for a new variant.
            _ => unreachable!("every constructed DomHostedRuntimeError row is listed"),
        },
        |error| matches!(error, DomHostedRuntimeError::Dom(_)),
        &[],
    );
    let inner = DomHostedError::ForeignSource;
    assert_chains(&DomHostedRuntimeError::Dom(inner.clone()), &inner);
}

#[test]
fn source_event_errors_state_the_refused_step_and_name_the_resource() {
    check_table(
        vec![
            (
                DomSourceEventRuntimeError::Dom(some_dom_error()),
                "the canonical DOM refused the Source event step",
            ),
            (
                DomSourceEventRuntimeError::Dispatch(DomEventDispatchError::CyclicPath),
                "the native event dispatcher refused the Source event entry",
            ),
            (
                DomSourceEventRuntimeError::Admission(DomSourceEventResource::Jobs),
                "the Source event adapter refused admission for retained event jobs",
            ),
            (
                DomSourceEventRuntimeError::Admission(DomSourceEventResource::StorageBytes),
                "the Source event adapter refused admission for retained payload storage",
            ),
            (
                DomSourceEventRuntimeError::Admission(DomSourceEventResource::Work),
                "the Source event adapter refused admission for cumulative adapter work",
            ),
            (
                DomSourceEventRuntimeError::LimitReduction(DomSourceEventResource::Jobs),
                "a policy update tried to lower the Source event adapter limit for retained event jobs",
            ),
            (
                DomSourceEventRuntimeError::LimitReduction(DomSourceEventResource::StorageBytes),
                "a policy update tried to lower the Source event adapter limit for retained payload storage",
            ),
            (
                DomSourceEventRuntimeError::LimitReduction(DomSourceEventResource::Work),
                "a policy update tried to lower the Source event adapter limit for cumulative adapter work",
            ),
            (
                DomSourceEventRuntimeError::ExpiredOwner,
                "the session that owns the Source event adapter was dropped",
            ),
            (
                DomSourceEventRuntimeError::ForeignOwner,
                "the DOM view belongs to another canonical owner than the Source event adapter",
            ),
            (
                DomSourceEventRuntimeError::PreboundSession,
                "the DOM already has an executor bound to its Source event owner",
            ),
            (
                DomSourceEventRuntimeError::Busy,
                "the Source event owner is already borrowed by another operation",
            ),
            (
                DomSourceEventRuntimeError::Protocol,
                "a Source event request, receipt or retained phase is inconsistent",
            ),
        ],
        |error| match error {
            DomSourceEventRuntimeError::Dom(_) => 0,
            DomSourceEventRuntimeError::Dispatch(_) => 1,
            DomSourceEventRuntimeError::Admission(DomSourceEventResource::Jobs) => 2,
            DomSourceEventRuntimeError::Admission(DomSourceEventResource::StorageBytes) => 3,
            DomSourceEventRuntimeError::Admission(DomSourceEventResource::Work) => 4,
            DomSourceEventRuntimeError::LimitReduction(DomSourceEventResource::Jobs) => 5,
            DomSourceEventRuntimeError::LimitReduction(DomSourceEventResource::StorageBytes) => 6,
            DomSourceEventRuntimeError::LimitReduction(DomSourceEventResource::Work) => 7,
            DomSourceEventRuntimeError::ExpiredOwner => 8,
            DomSourceEventRuntimeError::ForeignOwner => 9,
            DomSourceEventRuntimeError::PreboundSession => 10,
            DomSourceEventRuntimeError::Busy => 11,
            DomSourceEventRuntimeError::Protocol => 12,
        },
        |error| {
            matches!(
                error,
                DomSourceEventRuntimeError::Dom(_) | DomSourceEventRuntimeError::Dispatch(_)
            )
        },
        &[],
    );
    let inner = some_dom_error();
    assert_chains(&DomSourceEventRuntimeError::Dom(inner.clone()), &inner);
    assert_chains(
        &DomSourceEventRuntimeError::Dispatch(DomEventDispatchError::CyclicPath),
        &DomEventDispatchError::CyclicPath,
    );
}

#[test]
fn source_operation_errors_state_the_refusing_owner() {
    check_table(
        vec![
            (
                DomSourceRuntimeError::SourceEvent(DomSourceEventRuntimeError::Busy),
                "the Source event adapter refused the Source operation",
            ),
            (
                DomSourceRuntimeError::Inline(DomInlineStyleError::Unconfigured),
                "the inline style producer refused the Source operation",
            ),
            (
                DomSourceRuntimeError::Dom(some_dom_error()),
                "the canonical DOM refused the Source operation",
            ),
            (
                DomSourceRuntimeError::Style(DomStyleRuntimeError::Binding),
                "the Style runtime refused the Source operation",
            ),
            (
                DomSourceRuntimeError::ElementTasks(DomElementTaskRuntimeError::Busy),
                "the element-task owner refused the Source operation",
            ),
            (
                DomSourceRuntimeError::Hosted(DomHostedRuntimeError::Busy),
                "the hosted lifecycle owner refused the Source operation",
            ),
            (
                DomSourceRuntimeError::NativeInstallation(DomNativeHtmlInstallError::Target(
                    some_dom_error(),
                )),
                "the native HTML installation refused the Source operation",
            ),
            (
                DomSourceRuntimeError::MetaCsp(
                    DomSourceMetaCspError::<PolicyListCapacity>::IssuerBusy,
                ),
                "the meta CSP pragma could not be processed",
            ),
        ],
        |error| match error {
            DomSourceRuntimeError::SourceEvent(_) => 0,
            DomSourceRuntimeError::Inline(_) => 1,
            DomSourceRuntimeError::Dom(_) => 2,
            DomSourceRuntimeError::Style(_) => 3,
            DomSourceRuntimeError::ElementTasks(_) => 4,
            DomSourceRuntimeError::Hosted(_) => 5,
            DomSourceRuntimeError::NativeInstallation(_) => 6,
            DomSourceRuntimeError::MetaCsp(_) => 7,
        },
        |_| true,
        &[],
    );
    assert_chains(
        &DomSourceRuntimeError::Inline(DomInlineStyleError::Unconfigured),
        &DomInlineStyleError::Unconfigured,
    );
    let inner = some_dom_error();
    assert_chains(&DomSourceRuntimeError::Dom(inner.clone()), &inner);
}

#[test]
fn style_runtime_errors_state_the_refused_operation() {
    let root = node();
    // Rows without a public constructor for their payload: `PendingParsedAssociations`
    // (row 0, a parsed association id) and `RootReleasedForHostedDestruction` (row 13, a
    // hosted operation id and a CSSOM revision) have no constructed row.
    check_table(
        vec![
            (
                DomStyleRuntimeError::Dom(some_dom_error()),
                "the canonical DOM refused the Style runtime operation",
            ),
            (
                DomStyleRuntimeError::Cssom(CssomError::EffectAlreadyConsumed),
                "the CSSOM owner refused the operation",
            ),
            (
                DomStyleRuntimeError::Css(CssUtf16InputError::allocation()),
                "the CSS input could not be prepared or parsed",
            ),
            (
                DomStyleRuntimeError::Admission,
                "the Style operation was refused admission or an accounting step failed",
            ),
            (
                DomStyleRuntimeError::Binding,
                "an owner needed by the Style operation is borrowed, replaced or conflicts with the supplied binding",
            ),
            (
                DomStyleRuntimeError::Protocol,
                "a Source reply, retained operation or permit is inconsistent with the original request",
            ),
            (
                DomStyleRuntimeError::Dispatch(DomEventDispatchError::CyclicPath),
                "shared DOM event dispatch refused the Style task",
            ),
            (
                DomStyleRuntimeError::SessionUnavailable,
                "the session that owns the Style runtime has ended",
            ),
            (
                DomStyleRuntimeError::WrongSession,
                "the DOM view belongs to a different session than the Style runtime",
            ),
            (
                DomStyleRuntimeError::PreboundSession,
                "the DOM already has Style CSSOM roots that a new session cannot reconstruct",
            ),
            (
                DomStyleRuntimeError::SheetSetRootUnavailable { root },
                "the root has no bound stylesheet association owner",
            ),
            (
                DomStyleRuntimeError::SheetSetDocumentRequired { root },
                "the root is not a Document, which preference history requires",
            ),
            (
                DomStyleRuntimeError::RootWorkPending { root },
                "retained processing, notification or cleanup work for the root still needs its execution custody",
            ),
            (
                DomStyleRuntimeError::StaleSheetSetObservation,
                "the canonical, preference or producer state changed since the sheet-set observation",
            ),
            (
                DomStyleRuntimeError::SheetSetPublicationPending,
                "a staged or committed Source publication holds exclusive custody",
            ),
            (
                DomStyleRuntimeError::SheetSetOutputLimit,
                "the preference or producer output could not be admitted",
            ),
            (
                DomStyleRuntimeError::SourcePending(dom_with_pending_edit().1),
                "a canonical Source operation is already pending",
            ),
            (
                DomStyleRuntimeError::Resource(DomStyleCssomRootError::Busy),
                "the retained CSSOM root resource is unavailable or refused the work",
            ),
            (
                DomStyleRuntimeError::EventExecutor(DomSourceEventRuntimeError::Busy),
                "the Source event executor refused the Style event dispatch",
            ),
            (
                DomStyleRuntimeError::EventClock(DomHostedRuntimeError::Busy),
                "the hosted runtime refused the Style operation",
            ),
        ],
        |error| match error {
            DomStyleRuntimeError::PendingParsedAssociations(_) => 0,
            DomStyleRuntimeError::Dom(_) => 1,
            DomStyleRuntimeError::Cssom(_) => 2,
            DomStyleRuntimeError::Css(_) => 3,
            DomStyleRuntimeError::Admission => 4,
            DomStyleRuntimeError::Binding => 5,
            DomStyleRuntimeError::Protocol => 6,
            DomStyleRuntimeError::Dispatch(_) => 7,
            DomStyleRuntimeError::SessionUnavailable => 8,
            DomStyleRuntimeError::WrongSession => 9,
            DomStyleRuntimeError::PreboundSession => 10,
            DomStyleRuntimeError::SheetSetRootUnavailable { .. } => 11,
            DomStyleRuntimeError::SheetSetDocumentRequired { .. } => 12,
            DomStyleRuntimeError::RootReleasedForHostedDestruction { .. } => 13,
            DomStyleRuntimeError::RootWorkPending { .. } => 14,
            DomStyleRuntimeError::StaleSheetSetObservation => 15,
            DomStyleRuntimeError::SheetSetPublicationPending => 16,
            DomStyleRuntimeError::SheetSetOutputLimit => 17,
            DomStyleRuntimeError::SourcePending(_) => 18,
            DomStyleRuntimeError::Resource(_) => 19,
            DomStyleRuntimeError::EventExecutor(_) => 20,
            DomStyleRuntimeError::EventClock(_) => 21,
        },
        |error| {
            matches!(
                error,
                DomStyleRuntimeError::Dom(_)
                    | DomStyleRuntimeError::Cssom(_)
                    | DomStyleRuntimeError::Css(_)
                    | DomStyleRuntimeError::Dispatch(_)
                    | DomStyleRuntimeError::Resource(_)
                    | DomStyleRuntimeError::EventExecutor(_)
                    | DomStyleRuntimeError::EventClock(_)
            )
        },
        &[0, 13],
    );
    let inner = some_dom_error();
    assert_chains(&DomStyleRuntimeError::Dom(inner.clone()), &inner);
    let inner = CssomError::EffectAlreadyConsumed;
    assert_chains(&DomStyleRuntimeError::Cssom(inner.clone()), &inner);
    let inner = DomStyleCssomRootError::Busy;
    assert_chains(&DomStyleRuntimeError::Resource(inner.clone()), &inner);
    let inner = DomHostedRuntimeError::Busy;
    assert_chains(&DomStyleRuntimeError::EventClock(inner.clone()), &inner);
}

#[test]
fn parsed_contribution_settlement_errors_state_the_refusing_owner() {
    fn terminal() -> DomParsedHtmlTerminalError {
        DomParsedHtmlTerminalError::Frontier(DomParsedHtmlFrontierError::Style(
            DomParsedHtmlStyleError::WrongRequest,
        ))
    }
    check_table(
        vec![
            (
                DomStyleParsedTerminalError::Runtime(DomStyleRuntimeError::Binding),
                "the Style runtime refused to settle the parsed contribution",
            ),
            (
                DomStyleParsedTerminalError::Terminal(terminal()),
                "canonical parsed HTML settlement refused the contribution",
            ),
        ],
        |error| match error {
            DomStyleParsedTerminalError::Runtime(_) => 0,
            DomStyleParsedTerminalError::Terminal(_) => 1,
            // `TailPending` carries a parsed association id that only a parsed
            // installation can issue.
            DomStyleParsedTerminalError::TailPending { .. } => 2,
        },
        |error| !matches!(error, DomStyleParsedTerminalError::TailPending { .. }),
        &[],
    );
    assert_chains(
        &DomStyleParsedTerminalError::Terminal(terminal()),
        &terminal(),
    );
}

// End to end: errors returned by public operations.

fn limits() -> surgeist_runtime::DomSessionLimits {
    use surgeist_css::source::CssUtf16InputLimits;
    use surgeist_dom::hosted::{DomHostedLimits, DomHostedRecordLimits};
    use surgeist_runtime::{
        DomElementTaskRuntimeLimits, DomHostedRuntimeLimits, DomStyleRuntimeLimits,
    };
    surgeist_runtime::DomSessionLimits {
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

/// The refusal of a session over a fresh DOM with the given limits.
fn creation_refusal(
    limits: surgeist_runtime::DomSessionLimits,
) -> surgeist_runtime::DomSessionCreateError {
    match DomSession::new(dom().0, limits) {
        Err(error) => error,
        Ok(_) => panic!("the session must be refused"),
    }
}

/// Checks a session creation refusal: the context of the failure, the context of
/// the error that returns the DOM, and the exhausted resource from the chain.
fn assert_refusal(
    limits: surgeist_runtime::DomSessionLimits,
    check: fn(&DomSessionCreateFailure) -> bool,
    context: &str,
    cause: &str,
) {
    let refusal = creation_refusal(limits);
    let failure = refusal.error();
    assert!(check(failure), "{failure:?}");
    assert_eq!(failure.to_string(), context);
    assert_eq!(
        failure.source().expect("the owner's refusal").to_string(),
        cause
    );
    assert_eq!(
        refusal.to_string(),
        "creating the DOM session failed and returned the original DOM"
    );
    assert_eq!(refusal.source().expect("the failure").to_string(), context);
}

#[test]
fn session_creation_refusals_name_the_exhausted_owner_resource() {
    let mut refused = limits();
    refused.source_events.work = 0;
    assert_refusal(
        refused,
        |failure| {
            matches!(
                failure,
                DomSessionCreateFailure::SourceEvents(DomSourceEventRuntimeError::Admission(
                    DomSourceEventResource::Work
                ))
            )
        },
        "session creation was refused by the Source event executor",
        "the Source event adapter refused admission for cumulative adapter work",
    );

    let mut refused = limits();
    refused.element_tasks.storage_bytes = 0;
    assert_refusal(
        refused,
        |failure| {
            matches!(
                failure,
                DomSessionCreateFailure::ElementTasks(DomElementTaskRuntimeError::Admission)
            )
        },
        "session creation was refused by the element-task owner",
        "element-task admission refused the operation",
    );

    let mut refused = limits();
    refused.style.storage_bytes = 0;
    assert_refusal(
        refused,
        |failure| {
            matches!(
                failure,
                DomSessionCreateFailure::Style(DomStyleRuntimeError::Admission)
            )
        },
        "session creation was refused by the Style owner",
        "the Style operation was refused admission or an accounting step failed",
    );

    let mut refused = limits();
    refused.hosted.storage_bytes = 0;
    assert_refusal(
        refused,
        |failure| {
            matches!(
                failure,
                DomSessionCreateFailure::Hosted(DomHostedRuntimeError::Admission)
            )
        },
        "session creation was refused by the hosted lifecycle owner",
        "hosted lifecycle admission refused the operation",
    );

    let mut refused = limits();
    refused.authored_events.storage_bytes = 0;
    assert_refusal(
        refused,
        |failure| {
            matches!(
                failure,
                DomSessionCreateFailure::Events(DomEventDispatchError::Admission(
                    DomEventResource::Storage
                ))
            )
        },
        "session creation was refused by the shared Event object storage",
        "native DOM event dispatch admission refused event, target, listener and invocation storage",
    );
}

#[test]
fn operations_on_a_disposed_session_report_the_disposed_owner() {
    let session = DomSession::new(dom().0, limits()).unwrap();
    let tasks = session.element_task_execution();
    let styles = session.style_execution();
    let mut hosted = session.hosted_execution();
    let (control_dom, node) = dom();
    drop(session);

    let error = tasks.pending_total_tasks().unwrap_err();
    assert!(matches!(error, DomElementTaskRuntimeError::OwnerDisposed));
    assert_eq!(
        error.to_string(),
        "the session that owns the element-task state was disposed"
    );

    let error = styles.snapshot(node).unwrap_err();
    assert!(matches!(error, DomStyleRuntimeError::SessionUnavailable));
    assert_eq!(
        error.to_string(),
        "the session that owns the Style runtime has ended"
    );

    let mut control = DomSession::new(control_dom, limits()).unwrap();
    let error = hosted
        .set_limits(&mut control.dom(), limits().hosted, &flag())
        .unwrap_err();
    assert!(matches!(error, DomHostedRuntimeError::OwnerDisposed));
    assert_eq!(
        error.to_string(),
        "the session that issued this hosted permission was disposed"
    );
}

/// A session whose DOM can hold hosted top-level Documents.
fn hosted_session() -> DomSession {
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
    DomSession::new(dom, limits()).unwrap()
}

#[test]
fn hosted_lifecycle_refusals_of_public_operations_state_their_cause() {
    let mut session = hosted_session();
    let mut hosted = session.hosted_execution();

    let error = hosted
        .create_host(&mut session.dom(), &AtomicBool::new(true))
        .unwrap_err();
    assert!(matches!(error, DomHostedRuntimeError::Cancelled));
    assert_eq!(
        error.to_string(),
        "the hosted lifecycle operation was cancelled"
    );

    // A DOM view of another session does not match the permission's binding.
    let mut foreign = hosted_session();
    let error = hosted.create_host(&mut foreign.dom(), &flag()).unwrap_err();
    assert!(matches!(error, DomHostedRuntimeError::Binding));
    assert_eq!(
        error.to_string(),
        "the hosted lifecycle permission, DOM view, clock source or target does not match the original binding, or is not ready"
    );

    // A host that still has a live top-level cannot be destroyed first.
    let host = hosted.create_host(&mut session.dom(), &flag()).unwrap();
    hosted
        .create_top_level(&mut session.dom(), host, &flag())
        .unwrap();
    let error = hosted
        .destroy_host(&mut session.dom(), host, &flag())
        .unwrap_err();
    assert!(matches!(error, DomHostedRuntimeError::OperationPending));
    assert_eq!(
        error.to_string(),
        "a retained hosted operation, live hosted record or cleanup phase must be settled or reached before this operation can proceed"
    );

    // A host that was already destroyed is no longer found.
    let empty = hosted.create_host(&mut session.dom(), &flag()).unwrap();
    hosted
        .destroy_host(&mut session.dom(), empty, &flag())
        .unwrap();
    let error = hosted
        .destroy_host(&mut session.dom(), empty, &flag())
        .unwrap_err();
    assert!(matches!(error, DomHostedRuntimeError::TargetNotFound));
    assert_eq!(
        error.to_string(),
        "the live native host, Global, navigable or retained record this operation names was not found"
    );
}

/// A DOM with one started, unfinished Source edit, and that edit's operation id.
fn dom_with_pending_edit() -> (Dom, surgeist_dom::source::DomSourceOperationId) {
    use surgeist_dom::source::{DomSourceEdit, DomSourceLimits, DomSourceProgress};
    let (mut dom, _) = dom();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let style =
        batch.create_element_ns(&document, surgeist_html::values::Namespace::html(), "style");
    let commit = dom.apply(batch, &flag()).unwrap();
    let document = commit.document(&document).unwrap();
    let style = commit.created(&style).unwrap().unwrap();
    let mut lease = dom
        .begin_source_edit(
            DomSourceEdit::insert_before(style, document.node(), None),
            DomSourceLimits::new(64_000_000, 1_000_000, 20_000_000, 128, 4096, 4096, 8),
            &flag(),
        )
        .unwrap();
    assert!(matches!(
        lease.advance(&flag()).unwrap(),
        DomSourceProgress::Request(_)
    ));
    drop(lease);
    let operation = dom.pending_source_edit().unwrap().operation();
    (dom, operation)
}

#[test]
fn a_session_over_a_dom_with_a_pending_source_edit_is_refused_for_it() {
    let (dom, operation) = dom_with_pending_edit();
    let error = match DomSession::new(dom, limits()) {
        Err(error) => error,
        Ok(_) => panic!("a pending Source edit must refuse the session"),
    };
    assert!(
        matches!(
            error.error(),
            DomSessionCreateFailure::Style(DomStyleRuntimeError::SourcePending(pending))
                if *pending == operation
        ),
        "{:?}",
        error.error()
    );
    assert_eq!(
        DomStyleRuntimeError::SourcePending(operation).to_string(),
        "a canonical Source operation is already pending"
    );
}

#[test]
fn operations_with_the_dom_view_of_another_session_state_the_mismatch() {
    let session = DomSession::new(dom().0, limits()).unwrap();
    let mut tasks = session.element_task_execution();
    let styles = session.style_execution();
    let (other_dom, node) = dom();
    let mut other = DomSession::new(other_dom, limits()).unwrap();

    // The hosted owner is checked first, so the refusal is the hosted one.
    let error = tasks.ingest(&mut other.dom(), &flag()).unwrap_err();
    assert!(
        matches!(
            error,
            DomElementTaskRuntimeError::Hosted(DomHostedRuntimeError::Binding)
        ),
        "{error:?}"
    );
    assert_eq!(
        error.to_string(),
        "the native hosted lifecycle refused the element-task operation"
    );
    assert_eq!(
        error.source().unwrap().to_string(),
        "the hosted lifecycle permission, DOM view, clock source or target does not match the original binding, or is not ready"
    );

    let error = styles.sheet_sets(&other.dom(), node, &flag()).unwrap_err();
    assert!(
        matches!(error, DomStyleRuntimeError::WrongSession),
        "{error:?}"
    );
    assert_eq!(
        error.to_string(),
        "the DOM view belongs to a different session than the Style runtime"
    );
}
