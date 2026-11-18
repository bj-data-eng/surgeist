//! Context-only messages of the error types that wrap another error.
//!
//! Each of these types returns its wrapped error from `source()`. Its message
//! states only its own context and does not repeat the wrapped text, so that a
//! reporter which walks the chain prints each cause once. Each table row is
//! written from the variant's documented meaning; `source()` presence is
//! checked per row, and `assert_chains` checks that the wrapped text is neither
//! repeated nor replaced. Every enum here is matched exhaustively by its index
//! function, so a new variant does not compile until it has a row; rows whose
//! payload has no public constructor are listed per type as skipped.
#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt::Debug;

use surgeist_css::source::CssUtf16InputError;
use surgeist_dom::{
    DomError,
    installation::{
        DomParsedHtmlAssociationAdmissionError, DomParsedHtmlInstallError,
        meta::DomParsedHtmlMetaOccurrenceError,
        style::{
            DomParsedHtmlFrontierError, DomParsedHtmlStyleError,
            DomParsedHtmlStyleNotificationError, DomParsedHtmlStyleSheetError,
            DomParsedHtmlStyleSubjectError, DomParsedHtmlStyleSuccessorError,
        },
    },
    meta::DomMetaAtomicError,
};
use surgeist_runtime::{
    event::{DomEventConstructionError, DomEventDispatchError},
    hosted::DomHostedRuntimeError,
    input::TaskInput,
    provenance::InputProvenance,
    proxy::{AppProxy, QueuePolicy, WakeBridge, WakeError},
    session::DomSessionCreateFailure,
    source::{DomSourceDriveError, DomSourceEventRuntimeError, DomSourceRuntimeError},
    style::parsed::{
        DomStyleParsedAdmissionError, DomStyleParsedFrontierError, DomStyleParsedInstallError,
        DomStyleParsedMetaError, DomStyleParsedMetaOccurrenceError,
        DomStyleParsedNotificationError, DomStyleParsedPrefixError, DomStyleParsedSheetError,
        DomStyleParsedSubjectError, DomStyleParsedSuccessorError,
    },
    style::{DomStyleMetaError, DomStyleRootWithdrawalError, DomStyleRuntimeError},
    task::{TaskIntentAttemptId, TaskIntentId},
    trusted_types_csp::{AttributePolicyError, PolicyListCapacity, PolicyListError},
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

fn runtime() -> DomStyleRuntimeError {
    DomStyleRuntimeError::Binding
}

fn style() -> DomParsedHtmlStyleError {
    DomParsedHtmlStyleError::WrongRequest
}

#[test]
fn session_creation_failures_name_the_owner_that_refused() {
    check_table(
        vec![
            (
                DomSessionCreateFailure::Events(DomEventDispatchError::CyclicPath),
                "session creation was refused by the shared Event object storage",
            ),
            (
                DomSessionCreateFailure::SourceEvents(DomSourceEventRuntimeError::Busy),
                "session creation was refused by the Source event executor",
            ),
            (
                DomSessionCreateFailure::Style(runtime()),
                "session creation was refused by the Style owner",
            ),
            (
                DomSessionCreateFailure::Parsed(DomStyleParsedAdmissionError::Runtime(runtime())),
                "session creation was refused by the parsed association admission",
            ),
            (
                DomSessionCreateFailure::ElementTasks(
                    surgeist_runtime::element_task::DomElementTaskRuntimeError::Busy,
                ),
                "session creation was refused by the element-task owner",
            ),
            (
                DomSessionCreateFailure::Hosted(DomHostedRuntimeError::Busy),
                "session creation was refused by the hosted lifecycle owner",
            ),
        ],
        |error| match error {
            DomSessionCreateFailure::Events(_) => 0,
            DomSessionCreateFailure::SourceEvents(_) => 1,
            DomSessionCreateFailure::Style(_) => 2,
            DomSessionCreateFailure::Parsed(_) => 3,
            DomSessionCreateFailure::ElementTasks(_) => 4,
            DomSessionCreateFailure::Hosted(_) => 5,
        },
        |_| true,
        &[],
    );
    assert_chains(
        &DomSessionCreateFailure::Style(runtime()),
        &DomStyleRuntimeError::Binding,
    );
}

#[test]
fn drive_and_construction_errors_state_their_own_context() {
    check_table(
        vec![
            (
                DomSourceDriveError::Source(DomSourceRuntimeError::Style(runtime())),
                "the Source driver refused the operation",
            ),
            (
                DomSourceDriveError::Meta(DomStyleMetaError::Runtime(runtime())),
                "the native Meta composition of the Source driver refused the operation",
            ),
        ],
        |error| match error {
            DomSourceDriveError::Source(_) => 0,
            DomSourceDriveError::Meta(_) => 1,
        },
        |_| true,
        &[],
    );
    assert_chains(
        &DomSourceDriveError::Source(DomSourceRuntimeError::Style(runtime())),
        &DomSourceRuntimeError::Style(runtime()),
    );
    check_table(
        vec![
            (
                DomEventConstructionError::Hosted(DomHostedRuntimeError::Busy),
                "the hosted runtime, creation Global, Source owner or native clock refused the Event or EventTarget creation",
            ),
            (
                DomEventConstructionError::Event(DomEventDispatchError::CyclicPath),
                "the Event or EventTarget storage or type construction was refused",
            ),
            (
                DomEventConstructionError::UnsupportedInterface,
                "unsupported legacy Event interface",
            ),
            (
                DomEventConstructionError::MissingRelevantGlobal,
                "Document has no native relevant Global",
            ),
        ],
        |error| match error {
            DomEventConstructionError::Hosted(_) => 0,
            DomEventConstructionError::Event(_) => 1,
            DomEventConstructionError::UnsupportedInterface => 2,
            DomEventConstructionError::MissingRelevantGlobal => 3,
        },
        |error| {
            matches!(
                error,
                DomEventConstructionError::Hosted(_) | DomEventConstructionError::Event(_)
            )
        },
        &[],
    );
    assert_chains(
        &DomEventConstructionError::Event(DomEventDispatchError::CyclicPath),
        &DomEventDispatchError::CyclicPath,
    );
}

#[test]
fn trusted_types_wrappers_state_their_own_context() {
    // `AttributePolicyError::Csp` (row 3) wraps a CSP protocol error that has no
    // public constructor.
    check_table(
        vec![
            (
                AttributePolicyError::Dom(DomError::StaleRange),
                "the native lease, registration or Source controller refused the policy lookup",
            ),
            (
                AttributePolicyError::ContainerUnavailable,
                "the registered Global's CSP container is unavailable",
            ),
            (
                AttributePolicyError::ProcessorUnavailable,
                "the associated CSP processor is unavailable",
            ),
        ],
        |error| match error {
            AttributePolicyError::Dom(_) => 0,
            AttributePolicyError::ContainerUnavailable => 1,
            AttributePolicyError::ProcessorUnavailable => 2,
            AttributePolicyError::Csp(_) => 3,
        },
        |error| matches!(error, AttributePolicyError::Dom(_)),
        &[],
    );
    assert_chains(
        &AttributePolicyError::Dom(DomError::StaleRange),
        &DomError::StaleRange,
    );
    check_table(
        vec![
            (
                PolicyListError::<DomError>::Csp(
                    surgeist_html::security::csp::HtmlCspError::Interrupted(DomError::StaleRange),
                ),
                "the CSP protocol or the admission controller refused the policy list",
            ),
            (
                PolicyListError::<DomError>::Capacity(PolicyListCapacity::Allocation),
                "the native policy list could not be represented or reserved",
            ),
        ],
        |error| match error {
            PolicyListError::Csp(_) => 0,
            PolicyListError::Capacity(_) => 1,
        },
        |_| true,
        &[],
    );
    assert_chains(
        &PolicyListError::<DomError>::Capacity(PolicyListCapacity::Allocation),
        &PolicyListCapacity::Allocation,
    );
}

#[test]
fn parsed_style_wrappers_state_their_own_context() {
    check_table(
        vec![
            (
                DomStyleParsedPrefixError::Runtime(runtime()),
                "the Style runtime refused the parsed Style prefix command",
            ),
            (
                DomStyleParsedPrefixError::Style(style()),
                "canonical parsed Style input refused the prefix command",
            ),
        ],
        |error| match error {
            DomStyleParsedPrefixError::Runtime(_) => 0,
            DomStyleParsedPrefixError::Style(_) => 1,
            // The contribution id needs a parsed installation.
            DomStyleParsedPrefixError::ProcessingUnavailable { .. } => 2,
        },
        |_| true,
        &[],
    );
    check_table(
        vec![
            (
                DomStyleParsedNotificationError::Runtime(runtime()),
                "the Style runtime refused to queue the parsed notification",
            ),
            (
                DomStyleParsedNotificationError::Notification(
                    DomParsedHtmlStyleNotificationError::Style(style()),
                ),
                "canonical parsed Style notification qualification refused the queue",
            ),
        ],
        |error| match error {
            DomStyleParsedNotificationError::Runtime(_) => 0,
            DomStyleParsedNotificationError::Notification(_) => 1,
            // The next three carry parsed association or task ids.
            DomStyleParsedNotificationError::ProcessingUnavailable { .. } => 2,
            DomStyleParsedNotificationError::RenderingIncomplete { .. } => 3,
            DomStyleParsedNotificationError::AlreadyQueued { .. } => 4,
        },
        |_| true,
        &[],
    );
    check_table(
        vec![
            (
                DomStyleParsedSubjectError::Runtime(runtime()),
                "the Style runtime refused the parsed subject transition",
            ),
            (
                DomStyleParsedSubjectError::Subject(
                    DomParsedHtmlStyleSubjectError::WrongOccurrence,
                ),
                "canonical parsed Style subject transition was refused",
            ),
        ],
        |error| match error {
            DomStyleParsedSubjectError::Runtime(_) => 0,
            DomStyleParsedSubjectError::Subject(_) => 1,
        },
        |_| true,
        &[],
    );
    check_table(
        vec![
            (
                DomStyleParsedMetaOccurrenceError::Runtime(runtime()),
                "the Style runtime refused the parsed Meta occurrence",
            ),
            (
                DomStyleParsedMetaOccurrenceError::Meta(
                    DomParsedHtmlMetaOccurrenceError::WrongOccurrence,
                ),
                "canonical parsed Meta occurrence preparation was refused",
            ),
        ],
        |error| match error {
            DomStyleParsedMetaOccurrenceError::Runtime(_) => 0,
            DomStyleParsedMetaOccurrenceError::Meta(_) => 1,
        },
        |_| true,
        &[],
    );
    check_table(
        vec![
            (
                DomStyleParsedSheetError::Runtime(runtime()),
                "the Style runtime refused the parsed sheet processing",
            ),
            (
                DomStyleParsedSheetError::Sheet(DomParsedHtmlStyleSheetError::Style(style())),
                "canonical parsed sheet processing refused the sheet",
            ),
            (
                DomStyleParsedSheetError::Css(CssUtf16InputError::allocation()),
                "the CSS input of the parsed sheet was refused",
            ),
        ],
        |error| match error {
            DomStyleParsedSheetError::Runtime(_) => 0,
            DomStyleParsedSheetError::Sheet(_) => 1,
            DomStyleParsedSheetError::Css(_) => 2,
        },
        |_| true,
        &[],
    );
    check_table(
        vec![
            (
                DomStyleParsedFrontierError::Runtime(runtime()),
                "the Style runtime refused the parsed synchronous-end validation",
            ),
            (
                DomStyleParsedFrontierError::Frontier(DomParsedHtmlFrontierError::Style(style())),
                "the canonical parsed owner refused admission or reported remaining execution",
            ),
        ],
        |error| match error {
            DomStyleParsedFrontierError::Runtime(_) => 0,
            DomStyleParsedFrontierError::Frontier(_) => 1,
        },
        |_| true,
        &[],
    );
    check_table(
        vec![
            (
                DomStyleParsedSuccessorError::Runtime(runtime()),
                "the Style runtime refused the parsed successor",
            ),
            (
                DomStyleParsedSuccessorError::Successor(DomParsedHtmlStyleSuccessorError::Style(
                    style(),
                )),
                "the canonical parsed successor was refused",
            ),
        ],
        |error| match error {
            DomStyleParsedSuccessorError::Runtime(_) => 0,
            DomStyleParsedSuccessorError::Successor(_) => 1,
        },
        |_| true,
        &[],
    );
    check_table(
        vec![
            (
                DomStyleParsedMetaError::Runtime(runtime()),
                "the Style runtime refused the parsed Meta operation",
            ),
            (
                DomStyleParsedMetaError::Meta(
                    surgeist_dom::installation::meta::DomParsedHtmlMetaError::Admission(
                        DomParsedHtmlAssociationAdmissionError::Dom(DomError::Cancelled),
                    ),
                ),
                "the canonical parsed Meta qualification or producer preparation was refused",
            ),
        ],
        |error| match error {
            DomStyleParsedMetaError::Runtime(_) => 0,
            DomStyleParsedMetaError::Meta(_) => 1,
        },
        |_| true,
        &[],
    );
    check_table(
        vec![
            (
                DomStyleParsedAdmissionError::Runtime(runtime()),
                "the Style runtime refused the parsed Session intake",
            ),
            (
                DomStyleParsedAdmissionError::Association(
                    DomParsedHtmlAssociationAdmissionError::Dom(DomError::Cancelled),
                ),
                "the canonical parsed association classification or admission was refused",
            ),
        ],
        |error| match error {
            DomStyleParsedAdmissionError::Runtime(_) => 0,
            DomStyleParsedAdmissionError::Association(_) => 1,
        },
        |_| true,
        &[],
    );
    check_table(
        vec![
            (
                DomStyleParsedInstallError::Runtime(runtime()),
                "the Style runtime refused the parsed Session installation",
            ),
            (
                DomStyleParsedInstallError::Installation(DomParsedHtmlInstallError::Cancelled),
                "the canonical structural installation was refused",
            ),
        ],
        |error| match error {
            DomStyleParsedInstallError::Runtime(_) => 0,
            DomStyleParsedInstallError::Installation(_) => 1,
        },
        |_| true,
        &[],
    );
    assert_chains(
        &DomStyleParsedSheetError::Css(CssUtf16InputError::allocation()),
        &CssUtf16InputError::<DomStyleRuntimeError>::allocation(),
    );
    assert_chains(
        &DomStyleParsedInstallError::Runtime(runtime()),
        &DomStyleRuntimeError::Binding,
    );
}

#[test]
fn style_withdrawal_and_meta_errors_state_their_own_context() {
    check_table(
        vec![
            (
                DomStyleRootWithdrawalError::Style(runtime()),
                "the Style owner refused the root withdrawal",
            ),
            (
                DomStyleRootWithdrawalError::Hosted(DomHostedRuntimeError::Busy),
                "the hosted task, cleanup or canonical producer refused the root withdrawal",
            ),
        ],
        |error| match error {
            DomStyleRootWithdrawalError::Style(_) => 0,
            DomStyleRootWithdrawalError::Hosted(_) => 1,
        },
        |_| true,
        &[],
    );
    check_table(
        vec![
            (
                DomStyleMetaError::Runtime(runtime()),
                "the Style runtime refused the native Meta preparation",
            ),
            (
                DomStyleMetaError::Meta(DomMetaAtomicError::Dom(DomError::StaleRange)),
                "canonical or producer preparation refused the native Meta batch",
            ),
        ],
        |error| match error {
            DomStyleMetaError::Runtime(_) => 0,
            DomStyleMetaError::Meta(_) => 1,
        },
        |_| true,
        &[],
    );
    assert_chains(
        &DomStyleMetaError::Meta(DomMetaAtomicError::Dom(DomError::StaleRange)),
        &DomMetaAtomicError::Dom(DomError::StaleRange),
    );
}

// End to end.

struct FailingWake;

impl WakeBridge for FailingWake {
    fn wake(&self) -> Result<(), WakeError> {
        Err(WakeError::new("the host loop has stopped"))
    }
}

#[test]
fn a_failed_wake_is_reported_from_the_source_of_the_proxy_error() {
    let proxy = AppProxy::<()>::new(FailingWake, QueuePolicy::bounded(4));
    let provenance =
        InputProvenance::task(TaskIntentId::from_u64(1), TaskIntentAttemptId::from_u64(1));
    let error = proxy
        .send_task(TaskInput::new((), provenance).unwrap())
        .unwrap_err();
    assert_eq!(error.to_string(), "native wake bridge failed");
    assert_eq!(
        error.source().unwrap().to_string(),
        "the host loop has stopped"
    );
    // The queue overflow text is unchanged.
    let full = AppProxy::<()>::new(FailingWake, QueuePolicy::bounded(0));
    let provenance =
        InputProvenance::task(TaskIntentId::from_u64(1), TaskIntentAttemptId::from_u64(1));
    let error = full
        .send_task(TaskInput::new((), provenance).unwrap())
        .unwrap_err();
    assert_eq!(error.to_string(), "app proxy queue overflow at capacity 0");
}
