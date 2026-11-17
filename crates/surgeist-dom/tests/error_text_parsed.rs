//! Parsed and native installation, pending association, Style and Meta error text.
//!
//! Each message is written from the documented meaning of its variant and from
//! every place that returns it (`docs/html-installation.md`, the pending
//! association and Style prefix sources), never from the `Debug` form. An
//! exhaustive `match` per type forces a row when a variant is added; a wrapping
//! variant states only its own context and returns the wrapped error from
//! `source()`.
use std::error::Error;
use std::fmt::{self, Debug, Display};
use std::sync::{Arc, atomic::AtomicBool};

use surgeist_cssom::{CssomError, CssomStore};
use surgeist_dom::*;
use surgeist_html::construction::{HtmlConstructionCause, HtmlNodeId};
use surgeist_html::security::csp::{
    HtmlCspError, HtmlCspLayout, HtmlCspOwner, HtmlCspProcessorAuthority, HtmlCspProcessorProfile,
    HtmlCspProtocolError, HtmlCspStorage, HtmlCspUnavailable, HtmlMetaCspOutcome,
    HtmlMetaCspProcessor, HtmlMetaCspRequest,
};
use surgeist_html::values::Namespace;
use surgeist_html_parser::{
    HtmlDocumentParseRequest, HtmlParseAssociationMissingKind, HtmlParseAssociationOccurrenceId,
    HtmlParseAssociationOccurrenceKind, HtmlParseLimits, HtmlParsePendingAssociationStep,
    HtmlParseReport, HtmlSource,
};
use surgeist_html_semantics::{
    HtmlBaseEncoding, HtmlBaseFactsRevision, HtmlBaseProducerId, HtmlBaseUrlRecord,
    HtmlBaseUrlScheme,
};

fn flag() -> AtomicBool {
    AtomicBool::new(false)
}
fn envelope() -> DomParsedHtmlAssociationLimits {
    DomParsedHtmlAssociationLimits::new(16_000_000, 1_000_000, 1_000_000, 1024).unwrap()
}
fn parse(text: &str) -> Arc<HtmlParseReport> {
    let limits = HtmlParseLimits::builder()
        .source_units(100_000)
        .buffered_utf16_units(100_000)
        .token_attributes(10_000)
        .auxiliary_entries(1_000_000)
        .active_formatting_entries(1_000_000)
        .managed_nodes(100_000)
        .managed_edges(100_000)
        .managed_attributes(100_000)
        .managed_utf16_units(100_000)
        .source_map_entries(100_000)
        .source_map_payload_units(100_000)
        .diagnostics(100_000)
        .diagnostic_payload_units(100_000)
        .context_effects(100_000)
        .open_elements(100)
        .tree_depth(100_000)
        .managed_capacity_bytes(100_000_000)
        .peak_transient_bytes(100_000_000)
        .atomic_layout_bytes(100_000_000)
        .work_credits(100_000_000)
        .finish()
        .unwrap();
    Arc::new(
        HtmlDocumentParseRequest::offline(HtmlSource::utf8(text), limits)
            .parse()
            .unwrap(),
    )
}
fn setup() -> (Dom, DomParsedDocumentEnvironment) {
    let policy = DomLimits::new(
        1024,
        1_000_000,
        1024,
        4096,
        4096,
        32,
        4_000_000,
        DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
    )
    .unwrap()
    .with_meta_processing_work(1_000_000)
    .with_meta_theme_color_scratch_bytes(262144)
    .unwrap();
    let mut dom = Dom::new(policy).unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let commit = dom.apply(batch, &flag()).unwrap();
    let origin = dom
        .snapshot()
        .document(commit.document(&document).unwrap())
        .unwrap()
        .metadata()
        .origin()
        .clone();
    let url = HtmlBaseUrlRecord::observed(
        HtmlBaseFactsRevision::new(HtmlBaseProducerId::fresh(), 1, 2, 3),
        "https://example.test/parsed/".into(),
        HtmlBaseUrlScheme::Other,
        false,
        None,
    )
    .unwrap();
    let environment = DomParsedDocumentEnvironment::headless(
        origin,
        url,
        HtmlBaseEncoding::fresh(),
        "UTF-8".into(),
        DomNativeInputPlatform::headless(),
    );
    (dom, environment)
}
fn install(
    dom: &mut Dom,
    report: Arc<HtmlParseReport>,
    environment: DomParsedDocumentEnvironment,
) -> (DocumentId, DomParsedHtmlAssociationId) {
    let installed = dom
        .install_parsed_html_document(
            dom.snapshot().stamp(),
            report,
            environment,
            DomParsedHtmlInstallLimits::new(1024, 10_000_000, 16_000_000, 100_000_000),
            &flag(),
        )
        .unwrap();
    (
        installed.document(),
        installed.pending_association().unwrap(),
    )
}
fn first_occurrence(report: &HtmlParseReport) -> HtmlParseAssociationOccurrenceId {
    let mut cursor = report.pending_associations();
    loop {
        match cursor.advance() {
            HtmlParsePendingAssociationStep::Occurrence(row) => return row.id(),
            HtmlParsePendingAssociationStep::Complete => panic!("no original occurrence"),
            _ => (),
        }
    }
}
fn root_cap(document: DocumentId) -> DomStyleCssomRoot {
    DomStyleCssomRoot::new(
        document.node(),
        CssomStore::new(Default::default(), Default::default()).unwrap(),
    )
}

/// Real identities that the public API never lets a caller fabricate.
struct Real {
    node: NodeId,
    document: DocumentId,
    stamp: DomStamp,
    document_stamp: DomDocumentStamp,
    contribution: DomParsedHtmlAssociationId,
    occurrence: HtmlParseAssociationOccurrenceId,
    subject: HtmlNodeId,
}
fn real() -> Real {
    let report = parse("<!doctype html><style>a { color: red }</style>");
    let occurrence = first_occurrence(&report);
    let subject = report.document().unwrap().root().id();
    let (mut dom, environment) = setup();
    let (document, contribution) = install(&mut dom, report, environment);
    Real {
        node: document.node(),
        document,
        stamp: dom.snapshot().stamp(),
        document_stamp: dom.snapshot().document(document).unwrap().stamp(),
        contribution,
        occurrence,
        subject,
    }
}
fn historical(real: &Real) -> DomParsedHtmlHistoricalDocument {
    DomParsedHtmlHistoricalDocument::Constructed(real.subject)
}
fn cssom() -> CssomError {
    CssomError::EffectAlreadyConsumed
}
fn csp_protocol() -> HtmlCspProtocolError {
    let mut admit = |_: HtmlCspLayout| Ok::<_, ()>(());
    let mut first = HtmlCspOwner::new_with(&mut admit).unwrap();
    let mut second = HtmlCspOwner::new_with(&mut admit).unwrap();
    let authority = HtmlCspProcessorAuthority::new_with(&mut admit).unwrap();
    let profile = authority.profile_with(1, &mut admit).unwrap();
    let association = first.initial_document_with(&mut admit).unwrap();
    match second.request_meta_insertion_with(&association, &profile, "x".into(), &mut admit) {
        Err(rejected) => match rejected.error {
            HtmlCspError::Protocol(error) => error,
            HtmlCspError::Interrupted(()) => panic!("admission never refuses"),
        },
        Ok(_) => panic!("a policy association of another owner is a protocol error"),
    }
}
fn invocation(real: &Real) -> DomParsedHtmlStyleNotificationRequirement {
    DomParsedHtmlStyleNotificationRequirement::Invocation {
        contribution: real.contribution,
        occurrence: real.occurrence.clone(),
        document: real.document,
        phase: DomParsedHtmlStylePhase::Unstarted,
    }
}
fn occurrence(real: &Real) -> DomParsedHtmlStyleNotificationRequirement {
    DomParsedHtmlStyleNotificationRequirement::Occurrence {
        contribution: real.contribution,
        occurrence: real.occurrence.clone(),
        document: real.document,
        kind: HtmlParseAssociationOccurrenceKind::ChangePreferred,
    }
}
/// Work that remains (an invocation or an occurrence) differs from an inventory
/// that no further work can complete (`Unavailable`).
fn requirement(real: &Real) -> DomParsedHtmlStyleNotificationRequirement {
    DomParsedHtmlStyleNotificationRequirement::Unavailable {
        contribution: real.contribution,
        reason: DomParsedHtmlStyleNotificationUnavailable::OriginalEntryMissing,
    }
}

/// One table row: the value, its exact message and whether it wraps an error.
struct Row<T> {
    value: T,
    text: String,
    wraps: bool,
}
fn row<T>(value: T, text: &str, wraps: bool) -> Row<T> {
    Row {
        value,
        text: text.to_owned(),
        wraps,
    }
}
fn assert_semantic<T: Error + Send + Sync + 'static>(
    rows: Vec<Row<T>>,
    index: fn(&T) -> usize,
    variants: usize,
    unconstructible: &[usize],
    repeated: &[usize],
) {
    let mut covered = vec![false; variants];
    for index in unconstructible {
        covered[*index] = true;
    }
    // Every mismatch is collected, so that one run names all rows that differ.
    let mut mismatches = Vec::new();
    for Row { value, text, wraps } in &rows {
        let at = index(value);
        assert!(
            !covered[at] || repeated.contains(&at),
            "variant {at} appears twice: {text}"
        );
        covered[at] = true;
        let shown = value.to_string();
        if &shown != text {
            mismatches.push(format!("variant {at}: shows {shown:?}, expected {text:?}"));
        }
        let debug = format!("{value:?}");
        assert_ne!(shown, debug, "variant {at} writes its Debug form");
        assert!(
            !shown.contains(&debug),
            "variant {at} embeds its Debug form"
        );
        assert!(!shown.ends_with('.'), "variant {at} ends with a period");
        assert!(
            shown.chars().next().unwrap().is_lowercase(),
            "variant {at} starts upper case"
        );
        assert_eq!(value.source().is_some(), *wraps, "variant {at} source()");
    }
    assert!(mismatches.is_empty(), "{}", mismatches.join("\n"));
    assert!(
        covered.iter().all(|seen| *seen),
        "a variant has no row: {covered:?}"
    );
}

#[test]
fn admission_error_messages_state_the_refused_condition() {
    let real = real();
    type E = DomParsedHtmlAssociationAdmissionError;
    let rows = vec![
        row(
            E::Dom(DomError::Cancelled),
            "the association owner or canonical DOM refused the operation",
            true,
        ),
        row(
            E::NotPending {
                contribution: real.contribution,
            },
            "the contribution is not pending at its destination",
            false,
        ),
        row(
            E::LimitReduction {
                contribution: real.contribution,
            },
            "the original association limits cannot be lowered once bound",
            false,
        ),
        row(
            E::EarlierPending {
                document: real.document,
                pending: real.contribution,
            },
            "an earlier original contribution must complete first for this Document",
            false,
        ),
        row(
            E::Unsupported {
                contribution: real.contribution,
                occurrence: real.occurrence.clone(),
                kind: HtmlParseAssociationOccurrenceKind::UnsupportedLink,
            },
            "the contribution requires an unsupported influencing operation",
            false,
        ),
        row(
            E::Incomplete {
                contribution: real.contribution,
                missing: HtmlParseAssociationMissingKind::OriginalContext,
            },
            "the original capture lacks a qualification the contribution requires",
            false,
        ),
        row(
            E::DocumentMismatch {
                contribution: real.contribution,
                document: historical(&real),
            },
            "the original Document is not part of this destination's mapping",
            false,
        ),
        row(
            E::StaleDocument {
                document: real.document,
            },
            "the destination Document is no longer the current canonical generation",
            false,
        ),
        row(
            E::MissingPreferredName {
                contribution: real.contribution,
                occurrence: real.occurrence.clone(),
            },
            "the report seals a preferred-name change without its original preferred name",
            false,
        ),
    ];
    fn index(error: &E) -> usize {
        match error {
            E::Dom(_) => 0,
            E::NotPending { .. } => 1,
            E::LimitReduction { .. } => 2,
            E::EarlierPending { .. } => 3,
            E::Unsupported { .. } => 4,
            E::Incomplete { .. } => 5,
            E::DocumentMismatch { .. } => 6,
            E::StaleDocument { .. } => 7,
            E::MissingPreferredName { .. } => 8,
            // A defective published report cannot be built through the public API.
            E::UnresolvedReport(_) => 9,
        }
    }
    assert_semantic(rows, index, 10, &[9], &[]);
}

#[test]
fn admission_refuses_a_lowered_work_policy_with_the_limit_text() {
    let (mut dom, environment) = setup();
    let (_, id) = install(
        &mut dom,
        parse("<!doctype html><meta http-equiv=default-style content=B>"),
        environment,
    );
    let short = DomParsedHtmlAssociationLimits::new(16_000_000, 1_000_000, 2, 1024).unwrap();
    {
        let mut hold = dom.begin_parsed_html_meta(id, short, &flag()).unwrap();
        let error = hold.validate_meta(&flag()).unwrap_err();
        assert!(matches!(
            error,
            DomParsedHtmlAssociationAdmissionError::Dom(DomError::WorkLimit { .. })
        ));
        assert_eq!(
            error.to_string(),
            "the association owner or canonical DOM refused the operation"
        );
        assert!(error.source().is_some());
    }
    let lower = DomParsedHtmlAssociationLimits::new(16_000_000, 1_000_000, 1, 1024).unwrap();
    let Err(error) = dom.begin_parsed_html_meta(id, lower, &flag()) else {
        panic!("the original work policy cannot decrease");
    };
    assert_eq!(
        error.to_string(),
        "the original association limits cannot be lowered once bound"
    );
    assert!(error.source().is_none());
}

#[test]
fn meta_error_messages_state_their_own_context() {
    let real = real();
    type E = DomParsedHtmlMetaError;
    let admission = || DomParsedHtmlAssociationAdmissionError::Dom(DomError::Cancelled);
    let rows = vec![
        row(
            E::Admission(admission()),
            "the association owner or canonical DOM refused the operation",
            true,
        ),
        row(
            E::OwnerBusy { root: real.node },
            "the CSSOM owner of the destination root is already borrowed",
            false,
        ),
        row(
            E::OwnerPoisoned { root: real.node },
            "the CSSOM owner of the destination root was poisoned by an earlier failure",
            false,
        ),
        row(
            E::MissingOwner { root: real.node },
            "no CSSOM owner is available for the destination root",
            false,
        ),
        row(
            E::MismatchedOwner { root: real.node },
            "the supplied CSSOM owner does not match the owner of the destination root",
            false,
        ),
        row(
            E::StaleProducer { root: real.node },
            "the sheet is not in the prepared CSSOM snapshot of the destination root",
            false,
        ),
        row(
            E::Cssom {
                root: real.node,
                error: cssom(),
            },
            "the CSSOM producer refused a parsed Meta operation for the destination root",
            true,
        ),
    ];
    fn index(error: &E) -> usize {
        match error {
            E::Admission(_) => 0,
            E::OwnerBusy { .. } => 1,
            E::OwnerPoisoned { .. } => 2,
            E::MissingOwner { .. } => 3,
            E::MismatchedOwner { .. } => 4,
            E::StaleProducer { .. } => 5,
            E::Cssom { .. } => 6,
        }
    }
    assert_semantic(rows, index, 7, &[], &[]);
}

#[test]
fn meta_preparation_without_an_owner_reports_the_missing_root() {
    let (mut dom, environment) = setup();
    let (document, id) = install(
        &mut dom,
        parse("<!doctype html><meta http-equiv=default-style content=B>"),
        environment,
    );
    let mut hold = dom.begin_parsed_html_meta(id, envelope(), &flag()).unwrap();
    let Err(error) = hold.prepare(&[], &flag()) else {
        panic!("a Document that names a preferred sheet set needs its CSSOM owner");
    };
    assert!(
        matches!(error, DomParsedHtmlMetaError::MissingOwner { root } if root == document.node())
    );
    assert_eq!(
        error.to_string(),
        "no CSSOM owner is available for the destination root"
    );
}

#[test]
fn meta_occurrence_error_messages_state_their_own_context() {
    let real = real();
    type E = DomParsedHtmlMetaOccurrenceError;
    let style = || DomParsedHtmlStyleError::WrongRequest;
    let rows = vec![
        row(
            E::Style(style()),
            "the original Style prefix refused the Meta occurrence",
            true,
        ),
        row(
            E::Remaining(invocation(&real)),
            "other influencing original work must run before this Meta occurrence",
            false,
        ),
        row(
            E::Remaining(occurrence(&real)),
            "other influencing original work must run before this Meta occurrence",
            false,
        ),
        row(
            E::Remaining(requirement(&real)),
            "the original inventory cannot establish this Meta occurrence",
            false,
        ),
        row(
            E::WrongOccurrence,
            "the supplied occurrence is not the next supported original Meta occurrence",
            false,
        ),
        row(
            E::AlreadyConsumed,
            "the original Meta occurrence was already consumed",
            false,
        ),
        row(
            E::MissingOwner { root: real.node },
            "no CSSOM owner is available for the destination root",
            false,
        ),
        row(
            E::MismatchedOwner { root: real.node },
            "the supplied CSSOM owner does not match the owner of the destination root",
            false,
        ),
        row(
            E::OwnerBusy { root: real.node },
            "the CSSOM owner of the destination root is already borrowed",
            false,
        ),
        row(
            E::OwnerPoisoned { root: real.node },
            "the CSSOM owner of the destination root was poisoned by an earlier failure",
            false,
        ),
        row(
            E::Cssom(cssom()),
            "the CSSOM producer refused a step of the original Meta occurrence",
            true,
        ),
    ];
    fn index(error: &E) -> usize {
        match error {
            E::Style(_) => 0,
            E::Remaining(_) => 1,
            E::WrongOccurrence => 2,
            E::AlreadyConsumed => 3,
            E::MissingOwner { .. } => 4,
            E::MismatchedOwner { .. } => 5,
            E::OwnerBusy { .. } => 6,
            E::OwnerPoisoned { .. } => 7,
            E::Cssom(_) => 8,
        }
    }
    assert_semantic(rows, index, 9, &[], &[1]);
}

#[test]
fn style_error_messages_state_the_refused_command() {
    let real = real();
    type E = DomParsedHtmlStyleError;
    let rows = vec![
        row(
            E::Admission(DomParsedHtmlAssociationAdmissionError::Dom(
                DomError::Cancelled,
            )),
            "the association owner or canonical DOM refused the operation",
            true,
        ),
        row(
            E::Busy {
                contribution: real.contribution,
            },
            "the contribution's Style processing state is borrowed by another operation",
            false,
        ),
        row(
            E::Poisoned {
                contribution: real.contribution,
            },
            "the contribution's Style processing state was poisoned by an earlier failure",
            false,
        ),
        row(
            E::UnsupportedProfile {
                contribution: real.contribution,
            },
            "the original Style operation is outside the supported processing profile",
            false,
        ),
        row(
            E::WrongRequest,
            "the supplied inputs or the stored original Style state do not match what this command requires",
            false,
        ),
        row(
            E::ExecutionNotAdmitted {
                contribution: real.contribution,
            },
            "synchronous execution of the original contribution has not started",
            false,
        ),
        row(
            E::WrongExecutionProfile {
                profile:
                    surgeist_html::elements::style::HtmlStyleExecutionProfile::SourcePredicates,
            },
            "the Style execution profile or the Document's scripting mode does not allow execution without JavaScript",
            false,
        ),
        row(
            E::AlreadyBound,
            "this Style operation already has its initial inline policy",
            false,
        ),
        row(
            E::MissingPolicy,
            "no initial policy association was supplied",
            false,
        ),
        row(
            E::ReceiverOccupied,
            "the notification receiver already holds a continuation",
            false,
        ),
        row(
            E::NotificationTransferred,
            "the notification continuation was already transferred to its receiving owner",
            false,
        ),
        row(
            E::HandoffRequired {
                occurrence: real.occurrence.clone(),
            },
            "the notification continuation of the completed invocation must be transferred before the Style operation can advance",
            false,
        ),
        row(
            E::MissingObservation,
            "no inline-policy observation was supplied",
            false,
        ),
        row(
            E::InvalidSheetInput,
            "the sheet inputs, the order of sheet calls or the recorded Style state are not valid for this operation",
            false,
        ),
        row(
            E::WrongPhase {
                phase: DomParsedHtmlStylePhase::Unstarted,
            },
            "the original Style operation is not in the phase this command requires",
            false,
        ),
        row(
            E::DecisionAlreadyAccepted {
                contribution: real.contribution,
            },
            "the inline-policy decision for this Style operation was already accepted",
            false,
        ),
        row(
            E::PolicyContext {
                requirement:
                    surgeist_html_parser::HtmlParseAssociationContextRequirement::OfflineUnbound,
            },
            "the original inline-policy context cannot be used for this Style operation",
            false,
        ),
        row(
            E::Csp(csp_protocol()),
            "the HTML Content Security Policy protocol rejected a policy of the Style operation",
            true,
        ),
        row(
            E::Protocol(surgeist_html::elements::style::HtmlStyleProtocolError),
            "the HTML Style protocol rejected a reply for the original Style operation",
            true,
        ),
    ];
    fn index(error: &E) -> usize {
        match error {
            E::Admission(_) => 0,
            E::Busy { .. } => 1,
            E::Poisoned { .. } => 2,
            E::UnsupportedProfile { .. } => 3,
            E::WrongRequest => 4,
            E::ExecutionNotAdmitted { .. } => 5,
            E::WrongExecutionProfile { .. } => 6,
            E::AlreadyBound => 7,
            E::MissingPolicy => 8,
            E::ReceiverOccupied => 9,
            E::NotificationTransferred => 10,
            E::HandoffRequired { .. } => 11,
            E::MissingObservation => 12,
            E::InvalidSheetInput => 13,
            E::WrongPhase { .. } => 14,
            E::DecisionAlreadyAccepted { .. } => 15,
            E::PolicyContext { .. } => 16,
            E::Csp(_) => 17,
            E::Protocol(_) => 18,
        }
    }
    assert_semantic(rows, index, 19, &[], &[]);
}

#[test]
fn style_commands_report_their_refusal_before_and_after_a_policy_is_supplied() {
    let (mut dom, environment) = setup();
    let (_, id) = install(
        &mut dom,
        parse("<!doctype html><style nonce='n'>a { color: red }</style>"),
        environment,
    );
    {
        let mut hold = dom
            .begin_parsed_html_style(id, envelope(), &flag())
            .unwrap();
        assert_eq!(
            hold.advance(&flag()).unwrap(),
            DomParsedHtmlStylePhase::AwaitingInitialPolicy
        );
    }
    let request = dom
        .parsed_html_style_prefix(id)
        .unwrap()
        .initial_policy_request()
        .unwrap()
        .clone();
    let mut hold = dom
        .begin_parsed_html_style(id, envelope(), &flag())
        .unwrap();
    let error = hold
        .bind_initial_policy(&request, &mut None, &flag())
        .unwrap_err();
    assert!(matches!(error, DomParsedHtmlStyleError::MissingPolicy));
    assert_eq!(
        error.to_string(),
        "no initial policy association was supplied"
    );
    let error = hold.accept_inline_policy(&mut None, &flag()).unwrap_err();
    assert!(matches!(error, DomParsedHtmlStyleError::WrongRequest));
    assert_eq!(
        error.to_string(),
        "the supplied inputs or the stored original Style state do not match what this command requires"
    );
}

#[test]
fn style_notification_error_messages_state_their_own_context() {
    let real = real();
    type E = DomParsedHtmlStyleNotificationError;
    let rows = vec![
        row(
            E::Style(DomParsedHtmlStyleError::WrongRequest),
            "the original Style prefix refused the notification",
            true,
        ),
        row(
            E::MismatchedOwner { root: real.node },
            "the supplied CSSOM owner does not match the owner of the destination root",
            false,
        ),
        row(
            E::OwnerBusy { root: real.node },
            "the CSSOM owner of the destination root is already borrowed",
            false,
        ),
        row(
            E::OwnerPoisoned { root: real.node },
            "the CSSOM owner of the destination root was poisoned by an earlier failure",
            false,
        ),
        row(
            E::Cssom(cssom()),
            "the CSSOM producer refused the Style notification",
            true,
        ),
        row(
            E::MismatchedAssociation { target: real.node },
            "the current Style association of the target node does not match the notified sheet",
            false,
        ),
        row(
            E::MismatchedLabel { root: real.node },
            "the sheet's CSSOM owner label differs from the expected owner label",
            false,
        ),
        row(
            E::TargetDestroyed { target: real.node },
            "the Style element targeted by the notification no longer exists",
            false,
        ),
        row(
            E::MissingContinuation,
            "no notification continuation was supplied",
            false,
        ),
    ];
    fn index(error: &E) -> usize {
        match error {
            E::Style(_) => 0,
            E::MismatchedOwner { .. } => 1,
            E::OwnerBusy { .. } => 2,
            E::OwnerPoisoned { .. } => 3,
            E::Cssom(_) => 4,
            E::MismatchedAssociation { .. } => 5,
            E::MismatchedLabel { .. } => 6,
            E::TargetDestroyed { .. } => 7,
            E::MissingContinuation => 8,
        }
    }
    assert_semantic(rows, index, 9, &[], &[]);
}

#[test]
fn notification_preparation_without_a_continuation_names_the_missing_input() {
    let (mut dom, environment) = setup();
    let (document, id) = install(
        &mut dom,
        parse("<!doctype html><style>a { color: red }</style>"),
        environment,
    );
    let root = root_cap(document);
    let mut hold = dom
        .begin_parsed_html_style(id, envelope(), &flag())
        .unwrap();
    let Err(error) = hold.prepare_notification(&mut None, "owner", &root, &flag()) else {
        panic!("no continuation was supplied");
    };
    assert!(matches!(
        error,
        DomParsedHtmlStyleNotificationError::MissingContinuation
    ));
    assert_eq!(
        error.to_string(),
        "no notification continuation was supplied"
    );
}

#[test]
fn style_sheet_error_messages_state_their_own_context() {
    let real = real();
    type E = DomParsedHtmlStyleSheetError;
    let rows = vec![
        row(
            E::Style(DomParsedHtmlStyleError::WrongRequest),
            "the original Style prefix refused the first sheet publication",
            true,
        ),
        row(
            E::MismatchedOwner { root: real.node },
            "the supplied CSSOM owner does not match the owner of the destination root",
            false,
        ),
        row(
            E::OwnerBusy { root: real.node },
            "the CSSOM owner of the destination root is already borrowed",
            false,
        ),
        row(
            E::OwnerPoisoned { root: real.node },
            "the CSSOM owner of the destination root was poisoned by an earlier failure",
            false,
        ),
        row(
            E::Cssom {
                root: real.node,
                error: cssom(),
            },
            "the CSSOM producer refused a step of the first sheet publication for the destination root",
            true,
        ),
    ];
    fn index(error: &E) -> usize {
        match error {
            E::Style(_) => 0,
            E::MismatchedOwner { .. } => 1,
            E::OwnerBusy { .. } => 2,
            E::OwnerPoisoned { .. } => 3,
            E::Cssom { .. } => 4,
        }
    }
    assert_semantic(rows, index, 5, &[], &[]);
}

#[test]
fn sheet_preparation_before_execution_wraps_the_style_refusal() {
    let (mut dom, environment) = setup();
    let (document, id) = install(
        &mut dom,
        parse("<!doctype html><style>a { color: red }</style>"),
        environment,
    );
    let root = root_cap(document);
    let mut hold = dom
        .begin_parsed_html_style(id, envelope(), &flag())
        .unwrap();
    let Err(error) = hold.prepare_create_sheet(&root, &flag()) else {
        panic!("execution has not started");
    };
    assert_eq!(
        error.to_string(),
        "the original Style prefix refused the first sheet publication"
    );
    assert_eq!(
        error.source().unwrap().to_string(),
        "synchronous execution of the original contribution has not started"
    );
}

#[test]
fn style_subject_error_messages_state_their_own_context() {
    let real = real();
    type E = DomParsedHtmlStyleSubjectError;
    let rows = vec![
        row(
            E::Style(DomParsedHtmlStyleError::WrongRequest),
            "the original Style prefix refused the next subject",
            true,
        ),
        row(
            E::Remaining(invocation(&real)),
            "other influencing original work must run before the next Style subject",
            false,
        ),
        row(
            E::Remaining(occurrence(&real)),
            "other influencing original work must run before the next Style subject",
            false,
        ),
        row(
            E::Remaining(requirement(&real)),
            "the original inventory cannot establish the next Style subject",
            false,
        ),
        row(
            E::WrongOccurrence,
            "the supplied occurrence is not the next original Style subject",
            false,
        ),
        row(
            E::AlreadySelected,
            "the supplied occurrence was already selected or passed by the original sequence",
            false,
        ),
    ];
    fn index(error: &E) -> usize {
        match error {
            E::Style(_) => 0,
            E::Remaining(_) => 1,
            E::WrongOccurrence => 2,
            E::AlreadySelected => 3,
        }
    }
    assert_semantic(rows, index, 4, &[], &[1]);
}

#[test]
fn subject_advance_before_execution_wraps_the_style_refusal() {
    let report = parse("<!doctype html><style>a { color: red }</style>");
    let occurrence = first_occurrence(&report);
    let (mut dom, environment) = setup();
    let (_, id) = install(&mut dom, report, environment);
    let mut hold = dom
        .begin_parsed_html_style(id, envelope(), &flag())
        .unwrap();
    let Err(error) = hold.advance_subject(&occurrence, &flag()) else {
        panic!("execution has not started");
    };
    assert_eq!(
        error.to_string(),
        "the original Style prefix refused the next subject"
    );
    assert_eq!(
        error.source().unwrap().to_string(),
        "synchronous execution of the original contribution has not started"
    );
}

#[test]
fn style_successor_error_messages_state_their_own_context() {
    let real = real();
    type E = DomParsedHtmlStyleSuccessorError;
    let rows = vec![
        row(
            E::Style(DomParsedHtmlStyleError::WrongRequest),
            "the original Style prefix refused the successor",
            true,
        ),
        row(
            E::MissingOwner { root: real.node },
            "no CSSOM owner was supplied for the sheet that the predecessor created",
            false,
        ),
        row(
            E::MismatchedOwner { root: real.node },
            "the supplied CSSOM owner does not match the owner of the destination root",
            false,
        ),
        row(
            E::OwnerBusy { root: real.node },
            "the CSSOM owner of the destination root is already borrowed",
            false,
        ),
        row(
            E::OwnerPoisoned { root: real.node },
            "the CSSOM owner of the destination root was poisoned by an earlier failure",
            false,
        ),
        row(
            E::Cssom(cssom()),
            "the CSSOM producer refused the Style successor",
            true,
        ),
        row(
            E::Remaining(invocation(&real)),
            "the next influencing original work is outside this ParserPopped successor",
            false,
        ),
        row(
            E::Remaining(occurrence(&real)),
            "the next influencing original work is outside this ParserPopped successor",
            false,
        ),
        row(
            E::Remaining(requirement(&real)),
            "the original inventory cannot establish the successor",
            false,
        ),
    ];
    fn index(error: &E) -> usize {
        match error {
            E::Style(_) => 0,
            E::MissingOwner { .. } => 1,
            E::MismatchedOwner { .. } => 2,
            E::OwnerBusy { .. } => 3,
            E::OwnerPoisoned { .. } => 4,
            E::Cssom(_) => 5,
            E::Remaining(_) => 6,
        }
    }
    assert_semantic(rows, index, 7, &[], &[6]);
}

#[test]
fn successor_preparation_before_execution_wraps_the_style_refusal() {
    let (mut dom, environment) = setup();
    let (_, id) = install(
        &mut dom,
        parse("<!doctype html><style>a { color: red }</style>"),
        environment,
    );
    let mut hold = dom
        .begin_parsed_html_style(id, envelope(), &flag())
        .unwrap();
    let Err(error) = hold.prepare_successor(None, &flag()) else {
        panic!("execution has not started");
    };
    assert_eq!(
        error.to_string(),
        "the original Style prefix refused the successor"
    );
    assert!(error.source().is_some());
}

#[test]
fn terminal_error_messages_state_the_unsettled_condition() {
    let real = real();
    type E = DomParsedHtmlTerminalError;
    let rows = vec![
        row(
            E::Frontier(DomParsedHtmlFrontierError::Remaining(requirement(&real))),
            "the original contribution settlement was refused",
            true,
        ),
        row(
            E::SynchronousEndRequired {
                contribution: real.contribution,
            },
            "the original synchronous end must be reached before the contribution settles",
            false,
        ),
        row(
            E::CriticalResourcesUnresolved {
                contribution: real.contribution,
            },
            "critical resources of the original contribution are unresolved",
            false,
        ),
        row(
            E::NotificationsUnsettled {
                contribution: real.contribution,
                remaining: std::num::NonZeroU64::new(3).unwrap(),
            },
            "unsettled original Style notifications remain: 3",
            false,
        ),
    ];
    fn index(error: &E) -> usize {
        match error {
            E::Frontier(_) => 0,
            E::SynchronousEndRequired { .. } => 1,
            E::CriticalResourcesUnresolved { .. } => 2,
            E::NotificationsUnsettled { .. } => 3,
        }
    }
    assert_semantic(rows, index, 4, &[], &[]);
}

#[test]
fn settlement_before_the_synchronous_end_names_the_missing_end() {
    let (mut dom, environment) = setup();
    let (_, id) = install(
        &mut dom,
        parse("<!doctype html><style>a { color: red }</style>"),
        environment,
    );
    let Err(error) = dom
        .begin_parsed_html_style(id, envelope(), &flag())
        .unwrap()
        .complete_contribution(&flag())
    else {
        panic!("the synchronous end was not reached");
    };
    assert!(
        matches!(error, DomParsedHtmlTerminalError::SynchronousEndRequired { contribution } if contribution == id)
    );
    assert_eq!(
        error.to_string(),
        "the original synchronous end must be reached before the contribution settles"
    );
}

#[test]
fn frontier_error_messages_state_their_own_context() {
    let real = real();
    type E = DomParsedHtmlFrontierError;
    let rows = vec![
        row(
            E::Style(DomParsedHtmlStyleError::WrongRequest),
            "the original Style prefix refused the original synchronous end or its settlement",
            true,
        ),
        row(
            E::Remaining(invocation(&real)),
            "influencing original work remains before the synchronous end",
            false,
        ),
        row(
            E::Remaining(occurrence(&real)),
            "influencing original work remains before the synchronous end",
            false,
        ),
        row(
            E::Remaining(requirement(&real)),
            "the original inventory cannot establish the synchronous end",
            false,
        ),
    ];
    fn index(error: &E) -> usize {
        match error {
            E::Style(_) => 0,
            E::Remaining(_) => 1,
        }
    }
    assert_semantic(rows, index, 2, &[], &[1]);
}

#[test]
fn synchronous_end_before_execution_wraps_the_style_refusal() {
    let (mut dom, environment) = setup();
    let (_, id) = install(
        &mut dom,
        parse("<!doctype html><style>a { color: red }</style>"),
        environment,
    );
    let mut hold = dom
        .begin_parsed_html_style(id, envelope(), &flag())
        .unwrap();
    let Err(error) = hold.complete_synchronous_execution(&flag()) else {
        panic!("execution has not started");
    };
    assert_eq!(
        error.to_string(),
        "the original Style prefix refused the original synchronous end or its settlement"
    );
    assert_eq!(
        error.source().unwrap().to_string(),
        "synchronous execution of the original contribution has not started"
    );
}

#[test]
fn parsed_install_error_messages_state_the_refused_report() {
    let real = real();
    type E = DomParsedHtmlInstallError;
    type R = DomParsedHtmlInstallResource;
    let subject = || real.subject;
    let rows = vec![
        row(
            E::Target(DomError::Cancelled),
            "the canonical DOM refused the completed HTML installation",
            true,
        ),
        row(
            E::StaleDestination {
                expected: real.stamp,
                actual: real.stamp,
            },
            "the destination store changed since the supplied stamp",
            false,
        ),
        row(
            E::NotCompletedOrdinaryReport,
            "the report is not a completed ordinary-document report of the supported profile",
            false,
        ),
        row(
            E::NotCompletedFragmentReport,
            "the report is not a completed fragment report of the supported profile",
            false,
        ),
        row(
            E::ForeignContext,
            "the report's captured context does not match the supplied HTML capture",
            false,
        ),
        row(
            E::CurrentPolicyUnavailable,
            "the report or the destination Document is outside the policy profile that completed HTML installation supports",
            false,
        ),
        row(
            E::CspProtocol(csp_protocol()),
            "the HTML Content Security Policy protocol rejected the report's policy association",
            true,
        ),
        row(
            E::CspIssuerBusy,
            "the Document policy issuer is already borrowed",
            false,
        ),
        row(
            E::CapturedStateUnavailable,
            "the captured historical state the report relies on cannot be matched in the destination",
            false,
        ),
        row(
            E::EnvironmentMismatch,
            "the supplied environment is missing or differs from the report's original Document facts",
            false,
        ),
        row(
            E::MissingMetadata { subject: subject() },
            "installation found no report metadata or source mapping that it requires for a source node",
            false,
        ),
        row(
            E::UnsupportedState { subject: subject() },
            "a source node cannot be installed because its state or installation mapping is unsupported",
            false,
        ),
        row(
            E::FamilyStateUnavailable {
                subject: subject(),
                class: surgeist_html::elements::native::HtmlElementClass::GenericElement,
            },
            "the original element family state of a source node is unavailable",
            false,
        ),
        row(
            E::BaseStateUnavailable { subject: subject() },
            "the original state of a Base element is unavailable",
            false,
        ),
        row(
            E::MetaStateUnavailable {
                document: subject(),
                subject: subject(),
                reason: surgeist_html_parser::HtmlParseMetaMissingKind::Csp,
            },
            "the original Meta state of a Document is unavailable",
            false,
        ),
        row(
            E::AssociationCaptureUnavailable {
                document: historical(&real),
                reason: HtmlParseAssociationMissingKind::OriginalContext,
            },
            "the original pending association capture of a Document is unavailable",
            false,
        ),
        row(
            E::HyperlinkUnavailable {
                subject: subject(),
                reason: surgeist_html_parser::HtmlParseHyperlinkUnavailable::DocumentEnvironment,
            },
            "the original URL observation of a hyperlink is unavailable",
            false,
        ),
        row(
            E::ImageActivityUnavailable { subject: subject() },
            "the original update activity of an image is unavailable",
            false,
        ),
        row(
            E::DialogStateUnavailable {
                subject: subject(),
                action: surgeist_html::elements::dialog::HtmlDialogAction::ReadOpen,
            },
            "the original Dialog state that installation requires is unavailable",
            false,
        ),
        row(
            E::SourceLookup {
                subject: subject(),
                error: surgeist_html::construction::ForeignHtmlResult,
            },
            "a source node identity does not belong to the report's HTML result",
            true,
        ),
        row(
            E::UnreadableSource {
                subject: subject(),
                cause: HtmlConstructionCause::UnknownNode,
            },
            "a source node holds a reference that its result does not resolve",
            false,
        ),
        row(
            E::ResourceLimit {
                resource: R::MappedNodes,
                limit: 7,
            },
            "completed HTML installation exceeded its mapped-node allowance of 7",
            false,
        ),
        row(
            E::ResourceLimit {
                resource: R::Work,
                limit: 8,
            },
            "completed HTML installation exceeded its adapter work allowance of 8",
            false,
        ),
        row(
            E::ResourceLimit {
                resource: R::ScratchBytes,
                limit: 9,
            },
            "completed HTML installation exceeded its scratch byte allowance of 9",
            false,
        ),
        row(
            E::ResourceLimit {
                resource: R::HistoricalBytes,
                limit: 10,
            },
            "completed HTML installation exceeded its historical byte allowance of 10",
            false,
        ),
        row(
            E::Cancelled,
            "the completed HTML installation was cancelled",
            false,
        ),
        row(
            E::AllocationFailed,
            "an allocation required by the completed HTML installation failed",
            false,
        ),
        row(
            E::RepresentationExhausted,
            "the completed HTML installation met a count, size or position that its representation cannot hold",
            false,
        ),
    ];
    fn index(error: &E) -> usize {
        match error {
            E::Target(_) => 0,
            E::StaleDestination { .. } => 1,
            E::NotCompletedOrdinaryReport => 2,
            E::NotCompletedFragmentReport => 3,
            // A defective published report cannot be built through the public API.
            E::UnresolvedReport(_) => 4,
            E::ForeignContext => 5,
            E::CurrentPolicyUnavailable => 6,
            E::CspProtocol(_) => 7,
            E::CspIssuerBusy => 8,
            E::CapturedStateUnavailable => 9,
            E::EnvironmentMismatch => 10,
            E::MissingMetadata { .. } => 11,
            E::UnsupportedState { .. } => 12,
            E::FamilyStateUnavailable { .. } => 13,
            E::BaseStateUnavailable { .. } => 14,
            E::MetaStateUnavailable { .. } => 15,
            E::AssociationCaptureUnavailable { .. } => 16,
            E::HyperlinkUnavailable { .. } => 17,
            E::ImageActivityUnavailable { .. } => 18,
            E::DialogStateUnavailable { .. } => 19,
            E::SourceLookup { .. } => 20,
            E::UnreadableSource { .. } => 21,
            E::ResourceLimit { .. } => 22,
            E::Cancelled => 23,
            E::AllocationFailed => 24,
            E::RepresentationExhausted => 25,
        }
    }
    assert_semantic(rows, index, 26, &[4], &[22]);
}

#[test]
fn parsed_install_refuses_with_text_that_names_the_cause() {
    let report = parse("<!doctype html><p>text");
    let (mut dom, environment) = setup();
    let stale = dom.snapshot().stamp();
    let mut batch = DomBatch::new();
    batch.create_document();
    dom.apply(batch, &flag()).unwrap();
    let error = dom
        .install_parsed_html_document(
            stale,
            report.clone(),
            environment.clone(),
            DomParsedHtmlInstallLimits::new(1024, 10_000_000, 16_000_000, 100_000_000),
            &flag(),
        )
        .unwrap_err();
    assert!(matches!(
        error,
        DomParsedHtmlInstallError::StaleDestination { .. }
    ));
    assert_eq!(
        error.to_string(),
        "the destination store changed since the supplied stamp"
    );
    let error = dom
        .install_parsed_html_document(
            dom.snapshot().stamp(),
            report.clone(),
            environment.clone(),
            DomParsedHtmlInstallLimits::new(0, 10_000_000, 16_000_000, 100_000_000),
            &flag(),
        )
        .unwrap_err();
    assert!(matches!(
        error,
        DomParsedHtmlInstallError::ResourceLimit { limit: 0, .. }
    ));
    assert_eq!(
        error.to_string(),
        "completed HTML installation exceeded its mapped-node allowance of 0"
    );
    let cancelled = AtomicBool::new(true);
    let error = dom
        .install_parsed_html_document(
            dom.snapshot().stamp(),
            report,
            environment,
            DomParsedHtmlInstallLimits::new(1024, 10_000_000, 16_000_000, 100_000_000),
            &cancelled,
        )
        .unwrap_err();
    assert!(matches!(error, DomParsedHtmlInstallError::Cancelled));
    assert_eq!(
        error.to_string(),
        "the completed HTML installation was cancelled"
    );
}

#[test]
fn native_install_error_messages_state_the_refused_fragment() {
    let real = real();
    type E = DomNativeHtmlInstallError;
    type R = DomNativeHtmlInstallResource;
    type A = DomNativeHtmlAssociation;
    let subject = || real.subject;
    let rows = vec![
        row(
            E::Target(DomError::Cancelled),
            "the canonical DOM refused the native HTML installation",
            true,
        ),
        row(
            E::StaleDestination {
                expected: real.document_stamp,
                actual: real.stamp,
            },
            "the destination changed since the supplied basis stamp",
            false,
        ),
        row(
            E::Source {
                subject: subject(),
                error: DomError::Cancelled,
            },
            "the canonical DOM refused to build a node of the source result",
            true,
        ),
        row(
            E::SourceLookup {
                subject: subject(),
                error: surgeist_html::construction::ForeignHtmlResult,
            },
            "a source node identity does not belong to the supplied HTML result",
            true,
        ),
        row(
            E::UnreadableSource {
                subject: subject(),
                cause: HtmlConstructionCause::UnknownNode,
            },
            "a source node holds a reference that its result does not resolve",
            false,
        ),
        row(
            E::UnsupportedAssociation {
                subject: subject(),
                association: A::HostedSourceRoot,
            },
            "native HTML installation cannot convert a hosted source fragment root into fresh native ownership",
            false,
        ),
        row(
            E::UnsupportedAssociation {
                subject: subject(),
                association: A::DeclarativeShadow,
            },
            "native HTML installation cannot convert a declarative shadow relationship into fresh native ownership",
            false,
        ),
        row(
            E::UnsupportedAssociation {
                subject: subject(),
                association: A::TemplateShadowReference,
            },
            "native HTML installation cannot convert a Template shadow reference into fresh native ownership",
            false,
        ),
        row(
            E::UnsupportedAssociation {
                subject: subject(),
                association: A::TemplateOwnership,
            },
            "native HTML installation cannot convert a Template ownership relationship into fresh native ownership",
            false,
        ),
        row(
            E::ResourceLimit {
                resource: R::MappedNodes,
                limit: 7,
            },
            "native HTML installation exceeded its mapped-node allowance of 7",
            false,
        ),
        row(
            E::ResourceLimit {
                resource: R::Work,
                limit: 8,
            },
            "native HTML installation exceeded its adapter work allowance of 8",
            false,
        ),
        row(
            E::ResourceLimit {
                resource: R::ScratchBytes,
                limit: 9,
            },
            "native HTML installation exceeded its scratch byte allowance of 9",
            false,
        ),
        row(
            E::Cancelled,
            "the native HTML installation was cancelled",
            false,
        ),
        row(
            E::AllocationFailed,
            "an allocation required by the native HTML installation failed",
            false,
        ),
        row(
            E::RepresentationExhausted,
            "the native HTML installation exhausted a checked representation",
            false,
        ),
    ];
    fn index(error: &E) -> usize {
        match error {
            E::Target(_) => 0,
            E::StaleDestination { .. } => 1,
            E::Source { .. } => 2,
            E::SourceLookup { .. } => 3,
            E::UnreadableSource { .. } => 4,
            E::UnsupportedAssociation { .. } => 5,
            E::ResourceLimit { .. } => 6,
            E::Cancelled => 7,
            E::AllocationFailed => 8,
            E::RepresentationExhausted => 9,
        }
    }
    assert_semantic(rows, index, 10, &[], &[5, 6]);
}

mod native_fragment {
    use super::*;
    use surgeist_html::construction::{HtmlConstructionLimits, HtmlFragment};
    use surgeist_html::values::{OrderedAttributes, QualifiedName};

    fn dom() -> Dom {
        Dom::new(
            DomLimits::new(
                1024,
                131072,
                2048,
                1024,
                512,
                16,
                1048576,
                DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
            )
            .unwrap(),
        )
        .unwrap()
    }
    fn fragment() -> HtmlFragment {
        let mut builder =
            HtmlFragment::builder(HtmlConstructionLimits::new(1024, 2048, 1024, 131072, 512))
                .unwrap();
        builder
            .append_element(
                &builder.root(),
                QualifiedName::new(Namespace::html(), None, "p"),
                OrderedAttributes::new([]).unwrap(),
            )
            .unwrap();
        builder.finish().unwrap()
    }
    fn target(dom: &mut Dom) -> (DomDocumentStamp, NodeId) {
        let mut batch = DomBatch::new();
        let document = batch.create_document();
        let root = batch.create_element_ns(&document, Namespace::html(), "div");
        batch.insert_before(&root, &document, None);
        let commit = dom.apply(batch, &flag()).unwrap();
        let document = commit.document(&document).unwrap();
        (
            dom.snapshot().document(document).unwrap().stamp(),
            commit.created(&root).unwrap().unwrap(),
        )
    }

    #[test]
    fn refusals_name_the_cause() {
        let mut dom = dom();
        let (stamp, parent) = target(&mut dom);
        let source = fragment();
        let limits = DomNativeHtmlInstallLimits::new(1024, 1_000_000, 1_048_576);
        let mut batch = DomBatch::new();
        batch.create_document();
        dom.apply(batch, &flag()).unwrap();
        let error = dom
            .install_native_html_fragment(stamp, parent, None, &source, limits, &flag())
            .unwrap_err();
        assert!(matches!(
            error,
            DomNativeHtmlInstallError::StaleDestination { .. }
        ));
        assert_eq!(
            error.to_string(),
            "the destination changed since the supplied basis stamp"
        );
        let (stamp, parent) = target(&mut dom);
        let error = dom
            .install_native_html_fragment(
                stamp,
                parent,
                None,
                &source,
                DomNativeHtmlInstallLimits::new(0, 1_000_000, 1_048_576),
                &flag(),
            )
            .unwrap_err();
        assert!(matches!(
            error,
            DomNativeHtmlInstallError::ResourceLimit { limit: 0, .. }
        ));
        assert_eq!(
            error.to_string(),
            "native HTML installation exceeded its mapped-node allowance of 0"
        );
        let cancelled = AtomicBool::new(true);
        let error = dom
            .install_native_html_fragment(stamp, parent, None, &source, limits, &cancelled)
            .unwrap_err();
        assert!(matches!(error, DomNativeHtmlInstallError::Cancelled));
        assert_eq!(
            error.to_string(),
            "the native HTML installation was cancelled"
        );
    }
}

fn empty_policy() -> surgeist_html::security::csp::HtmlCspAssociation {
    let mut admit = |_: HtmlCspLayout| Ok::<_, ()>(());
    let mut owner = HtmlCspOwner::new_with(&mut admit).unwrap();
    owner.initial_document_with(&mut admit).unwrap()
}
/// A contribution whose first inline decision blocked, so that its parser-popped
/// successor and everything after it remain as original work.
fn blocked(
    dom: &mut Dom,
    report: Arc<HtmlParseReport>,
    environment: DomParsedDocumentEnvironment,
) -> DomParsedHtmlAssociationId {
    let (_, id) = install(dom, report, environment);
    {
        let mut hold = dom
            .begin_parsed_html_style(id, envelope(), &flag())
            .unwrap();
        assert_eq!(
            hold.advance(&flag()).unwrap(),
            DomParsedHtmlStylePhase::AwaitingInitialPolicy
        );
    }
    let request = dom
        .parsed_html_style_prefix(id)
        .unwrap()
        .initial_policy_request()
        .unwrap()
        .clone();
    {
        let mut hold = dom
            .begin_parsed_html_style(id, envelope(), &flag())
            .unwrap();
        hold.bind_initial_policy(&request, &mut Some(empty_policy()), &flag())
            .unwrap();
        assert_eq!(
            hold.advance(&flag()).unwrap(),
            DomParsedHtmlStylePhase::InlinePolicyPending
        );
    }
    let mut observation = {
        let view = dom.parsed_html_style_prefix(id).unwrap();
        Some(DomParsedHtmlStyleInlinePolicyObservation::evaluated(
            view.inline_policy_request().unwrap(),
            false,
        ))
    };
    dom.begin_parsed_html_style(id, envelope(), &flag())
        .unwrap()
        .accept_inline_policy(&mut observation, &flag())
        .unwrap();
    id
}

#[test]
fn synchronous_end_with_original_work_remaining_names_that_work() {
    let (mut dom, environment) = setup();
    let id = blocked(
        &mut dom,
        parse("<!doctype html><style>blocked</style>"),
        environment,
    );
    let Err(error) = dom
        .begin_parsed_html_style(id, envelope(), &flag())
        .unwrap()
        .complete_synchronous_execution(&flag())
    else {
        panic!("the parser-popped return of the blocked Style is still original work");
    };
    assert!(matches!(
        error,
        DomParsedHtmlFrontierError::Remaining(
            DomParsedHtmlStyleNotificationRequirement::Occurrence { .. }
        )
    ));
    assert_eq!(
        error.to_string(),
        "influencing original work remains before the synchronous end"
    );
    assert!(error.source().is_none());
}

#[test]
fn meta_occurrence_error_returns_wrap_and_work_remaining_text() {
    let report =
        parse("<!doctype html><style>blocked</style><meta http-equiv=default-style content=A>");
    let meta = {
        let mut cursor = report.pending_associations();
        loop {
            match cursor.advance() {
                HtmlParsePendingAssociationStep::Occurrence(row)
                    if row.kind() == HtmlParseAssociationOccurrenceKind::ChangePreferred =>
                {
                    break row.id();
                }
                HtmlParsePendingAssociationStep::Complete => panic!("no Meta occurrence"),
                _ => (),
            }
        }
    };
    let (mut dom, environment) = setup();
    let (_, id) = install(&mut dom, report.clone(), environment);
    {
        let hold = dom
            .begin_parsed_html_style(id, envelope(), &flag())
            .unwrap();
        let Err(error) = hold.meta_occurrence_document(&meta, &flag()) else {
            panic!("execution has not started");
        };
        assert_eq!(
            error.to_string(),
            "the original Style prefix refused the Meta occurrence"
        );
        assert_eq!(
            error.source().unwrap().to_string(),
            "synchronous execution of the original contribution has not started"
        );
    }
    let (mut dom, environment) = setup();
    let id = blocked(&mut dom, report, environment);
    let hold = dom
        .begin_parsed_html_style(id, envelope(), &flag())
        .unwrap();
    let Err(error) = hold.meta_occurrence_document(&meta, &flag()) else {
        panic!("the blocked Style's parser-popped return comes first");
    };
    assert!(matches!(
        error,
        DomParsedHtmlMetaOccurrenceError::Remaining(
            DomParsedHtmlStyleNotificationRequirement::Occurrence { .. }
        )
    ));
    assert_eq!(
        error.to_string(),
        "other influencing original work must run before this Meta occurrence"
    );
    assert!(error.source().is_none());
}

#[derive(Debug, PartialEq, Eq)]
struct Veto;
impl Display for Veto {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("the processor said no")
    }
}
impl Error for Veto {}

struct Processor {
    _authority: HtmlCspProcessorAuthority,
    profile: HtmlCspProcessorProfile,
    outcome: fn() -> HtmlMetaCspOutcome<Veto>,
}
impl Processor {
    fn new(outcome: fn() -> HtmlMetaCspOutcome<Veto>) -> Self {
        let mut allow = |_: HtmlCspLayout| Ok::<_, ()>(());
        let authority = HtmlCspProcessorAuthority::new_with(&mut allow).unwrap();
        let profile = authority.profile_with(1, &mut allow).unwrap();
        Self {
            _authority: authority,
            profile,
            outcome,
        }
    }
}
impl HtmlMetaCspProcessor for Processor {
    type Error = Veto;
    fn profile(&self) -> &HtmlCspProcessorProfile {
        &self.profile
    }
    fn prepare<E>(
        &mut self,
        _request: &HtmlMetaCspRequest,
        _admit: &mut impl FnMut(HtmlCspLayout) -> Result<(), E>,
        _release: &mut impl FnMut(HtmlCspStorage),
    ) -> Result<HtmlMetaCspOutcome<Veto>, HtmlCspError<E>> {
        Ok((self.outcome)())
    }
}

/// The error of a real native Meta CSP insertion whose processor answers `outcome`.
fn delivered(outcome: fn() -> HtmlMetaCspOutcome<Veto>) -> DomSourceMetaCspError<Veto> {
    let cancel = flag();
    let mut dom = Dom::new(
        DomLimits::new(
            32,
            1_000_000,
            64,
            0,
            0,
            8,
            1_000_000,
            DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
        )
        .unwrap()
        .with_meta_processing_work(1_000_000),
    )
    .unwrap();
    let mut setup = DomBatch::new();
    let document = setup.create_document();
    let head = setup.create_element_ns(&document, Namespace::html(), "head");
    let meta = setup.create_element_ns(&document, Namespace::html(), "meta");
    setup.insert_before(&head, &document, None);
    setup.set_attribute(&meta, "http-equiv", "content-security-policy");
    setup.set_attribute(&meta, "content", "require-trusted-types-for 'script'");
    let created = dom.apply(setup, &cancel).unwrap();
    let head = created.created(&head).unwrap().unwrap();
    let meta = created.created(&meta).unwrap().unwrap();
    let limits = DomSourceLimits::new(64_000_000, 4_000_000, 1_000_000, 64, 4096, 4096, 1);
    let mut lease = dom
        .begin_source_edit(
            DomSourceEdit::insert_before(meta, head, None),
            limits,
            &cancel,
        )
        .unwrap();
    let Ok(DomSourceProgress::Request(DomSourceRequest::MetaCsp(request))) = lease.advance(&cancel)
    else {
        panic!("a reached Meta insertion waits for its policy processor");
    };
    let mut processor = Processor::new(outcome);
    let Err(error) = lease.process_meta_csp(request, &mut processor, &cancel) else {
        panic!("the processor does not complete");
    };
    lease.request_cancel();
    error
}

#[test]
fn source_meta_csp_error_messages_state_the_refused_delivery() {
    type E = DomSourceMetaCspError<Veto>;
    let rows = vec![
        row(
            E::Dom(DomError::Cancelled),
            "the canonical Source refused the native Meta Content Security Policy delivery",
            true,
        ),
        row(
            E::Protocol(csp_protocol()),
            "the HTML Content Security Policy protocol rejected the native Meta delivery",
            true,
        ),
        row(
            E::IssuerBusy,
            "the Document's Content Security Policy issuer is already borrowed",
            false,
        ),
        row(
            E::PolicyUnavailable,
            "the Document has no qualified prior Content Security Policy container",
            false,
        ),
        row(
            E::ProcessorUnavailable(HtmlCspUnavailable::Processor),
            "the Meta Content Security Policy processor lacks a capability the occurrence requires",
            false,
        ),
        row(
            delivered(|| HtmlMetaCspOutcome::Refused(Veto)),
            "the Meta Content Security Policy processor refused the occurrence",
            true,
        ),
        row(
            delivered(|| HtmlMetaCspOutcome::Failed(Veto)),
            "the Meta Content Security Policy processor failed while processing the occurrence",
            true,
        ),
    ];
    fn index(error: &E) -> usize {
        match error {
            E::Dom(_) => 0,
            E::Protocol(_) => 1,
            E::IssuerBusy => 2,
            E::PolicyUnavailable => 3,
            E::ProcessorUnavailable(_) => 4,
            E::ProcessorRefused(_) => 5,
            E::ProcessorFailed(_) => 6,
        }
    }
    assert_semantic(rows, index, 7, &[], &[]);
}

#[test]
fn source_meta_csp_processor_outcomes_return_the_processor_error_as_source() {
    let refused = delivered(|| HtmlMetaCspOutcome::Refused(Veto));
    assert!(matches!(
        refused,
        DomSourceMetaCspError::ProcessorRefused(_)
    ));
    assert_eq!(
        refused.to_string(),
        "the Meta Content Security Policy processor refused the occurrence"
    );
    assert_eq!(
        refused.source().unwrap().to_string(),
        "the processor said no"
    );
    let failed = delivered(|| HtmlMetaCspOutcome::Failed(Veto));
    assert!(matches!(failed, DomSourceMetaCspError::ProcessorFailed(_)));
    assert_eq!(
        failed.source().unwrap().to_string(),
        "the processor said no"
    );
}
