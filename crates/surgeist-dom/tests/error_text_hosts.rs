//! Message text and `Error` contract of the fact, protocol and host error types.
//!
//! Each table row is written from the variant's documented meaning and from the
//! operations that return it, not from the implementation's output. An
//! exhaustive `match` per type maps a variant to its row, so a new variant does
//! not compile until it has a row.
#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt::Debug;
use std::sync::atomic::AtomicBool;

use surgeist_css::CssParserContext;
use surgeist_css::property::CssPropertyGrammar;
use surgeist_css::source::{CssUtf16InputError, CssUtf16InputLimits, CssUtf16Resource};
use surgeist_cssom::{
    CssomAdmission, CssomContext, CssomDeclarationCandidate, CssomDeclarationRequestLimits,
    CssomDeclarationSupport, CssomError, CssomInputRole, CssomInputVersion, CssomLimits,
    CssomOwnerEffectStatus, CssomUsabilityRequirement,
};
use surgeist_dom::{
    DocumentId, Dom, DomBaseFactsError, DomBatch, DomDocumentConstructorError,
    DomEncodingUrlFactsError, DomError, DomHostDomains, DomHostError, DomHostObservations,
    DomHostPublisher, DomHostedError, DomHostedLimits, DomHostedRecordLimits, DomHostedResource,
    DomHostedSheetConstructionError, DomHtmlCaptureError, DomHtmlCaptureLimits,
    DomHtmlCaptureResource, DomInlineStyleError, DomInlineStyleInputs, DomInlineStyleLimits,
    DomInlineStyleRequestLimits, DomLimits, DomMetaAtomicError, DomMetaError,
    DomSourceEventClockFailure, DomStyleCssomRootError, DomViewTransitionCssomError, NodeId,
    Observed,
};
use surgeist_html::values::Namespace;
use surgeist_html_semantics::{
    HtmlAboutBaseUrl, HtmlBaseDocumentFacts, HtmlBaseEncoding, HtmlBaseFactsError,
    HtmlBaseFactsLimits, HtmlBaseFactsRevision, HtmlBaseMissing, HtmlBaseParseObservation,
    HtmlBaseParseOutcome, HtmlBaseProducerId, HtmlBaseUrlRecord, HtmlBaseUrlScheme,
    HtmlCaptureElementError, HtmlCaptureError, HtmlCaptureLimits, HtmlControlFactsError,
    HtmlElementReferenceFactsError, HtmlEncodingUrlFactsError, HtmlEncodingUrlParseFacts,
    HtmlGlobalInheritanceError, HtmlInertFactError, HtmlLanguageDirectionError,
    HtmlOutputDefaultError, HtmlParseBaseFacts, HtmlSemanticCancellation, HtmlSemanticContextError,
    HtmlSemanticLimits, HtmlSemanticQuery, HtmlSemanticRequest,
};

/// Checks every row of one type's table against the `Error` contract.
///
/// `index` is an exhaustive match returning each variant's row number, so the
/// table must hold one row per variant in that order. `wraps` states which
/// variants carry a source error.
fn check_table<E>(rows: Vec<(E, &'static str)>, index: fn(&E) -> usize, wraps: fn(&E) -> bool)
where
    E: Error + Send + Sync + 'static + Debug,
{
    check_partial_table(rows, index, wraps, &[]);
}

/// As `check_table`, for a type with variants that cannot be built outside the
/// crate that owns their payload; `skipped` lists the row numbers without a row.
fn check_partial_table<E>(
    rows: Vec<(E, &'static str)>,
    index: fn(&E) -> usize,
    wraps: fn(&E) -> bool,
    skipped: &[usize],
) where
    E: Error + Send + Sync + 'static + Debug,
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

fn cancel() -> AtomicBool {
    AtomicBool::new(false)
}

/// The error of a result whose success value has no `Debug`.
fn refusal<T, E>(result: Result<T, E>, expected: &str) -> E {
    match result {
        Err(error) => error,
        Ok(_) => panic!("{expected}"),
    }
}

const MISSING: [(HtmlBaseMissing, &str); 8] = [
    (HtmlBaseMissing::DocumentFacts, "document facts are missing"),
    (HtmlBaseMissing::DocumentUrl, "the document URL is missing"),
    (
        HtmlBaseMissing::AboutBaseUrl,
        "the about:base URL is missing",
    ),
    (
        HtmlBaseMissing::CharacterEncoding,
        "the character encoding is missing",
    ),
    (
        HtmlBaseMissing::ParseObservation,
        "the parse observation is missing",
    ),
    (
        HtmlBaseMissing::Csp,
        "the Content Security Policy is missing",
    ),
    (HtmlBaseMissing::FrozenBase, "no frozen base exists"),
    (
        HtmlBaseMissing::ResponseDomain,
        "the response domain is missing",
    ),
];

#[test]
fn base_facts_errors_state_the_refused_facts() {
    let rows = vec![
        (
            DomBaseFactsError::EnvironmentMismatch,
            "base URL facts do not describe this document's environment",
        ),
        (
            DomBaseFactsError::ForeignProducer,
            "base URL facts came from a different producer than the facts already bound to this document",
        ),
        (
            DomBaseFactsError::InputsMismatch,
            "base URL facts were computed from inconsistent inputs",
        ),
        (
            DomBaseFactsError::BasisRevisionMismatch,
            "base URL facts were captured at a different store revision",
        ),
        (
            DomBaseFactsError::Missing(HtmlBaseMissing::DocumentFacts),
            "the base URL cannot be established: document facts are missing",
        ),
        (
            DomBaseFactsError::PolicyMismatch,
            "base URL facts were produced under a Content Security Policy that differs from the document's",
        ),
    ];
    check_table(
        rows,
        |error| match error {
            DomBaseFactsError::EnvironmentMismatch => 0,
            DomBaseFactsError::ForeignProducer => 1,
            DomBaseFactsError::InputsMismatch => 2,
            DomBaseFactsError::BasisRevisionMismatch => 3,
            DomBaseFactsError::Missing(_) => 4,
            DomBaseFactsError::PolicyMismatch => 5,
        },
        |_| false,
    );
    for (missing, clause) in MISSING {
        assert_eq!(
            DomBaseFactsError::Missing(missing).to_string(),
            format!("the base URL cannot be established: {clause}")
        );
    }
}

#[test]
fn document_constructor_errors_state_the_refused_context() {
    check_table(
        vec![
            (
                DomDocumentConstructorError::ForeignIssuer,
                "the Document constructor context was issued by a different hosted source",
            ),
            (
                DomDocumentConstructorError::ExpiredIssuer,
                "the Document constructor context's issuer has expired",
            ),
            (
                DomDocumentConstructorError::PoisonedIssuer,
                "the Document constructor context's issuer was poisoned by an earlier failure",
            ),
            (
                DomDocumentConstructorError::StaleAssociation,
                "the Document constructor context's Global association is missing, stale or retired",
            ),
            (
                DomDocumentConstructorError::IncompleteAssociation,
                "the Document constructor context's association is not in the live initial-entry phase",
            ),
            (
                DomDocumentConstructorError::Unavailable,
                "no hosted association is available for the Document constructor",
            ),
        ],
        |error| match error {
            DomDocumentConstructorError::ForeignIssuer => 0,
            DomDocumentConstructorError::ExpiredIssuer => 1,
            DomDocumentConstructorError::PoisonedIssuer => 2,
            DomDocumentConstructorError::StaleAssociation => 3,
            DomDocumentConstructorError::IncompleteAssociation => 4,
            DomDocumentConstructorError::Unavailable => 5,
        },
        |_| false,
    );
}

#[test]
fn encoding_url_facts_errors_state_the_refused_facts_and_chain_the_producer() {
    check_table(
        vec![
            (
                DomEncodingUrlFactsError::BasisRevisionMismatch,
                "encoding and URL facts were captured at a different store revision",
            ),
            (
                DomEncodingUrlFactsError::EnvironmentMismatch,
                "encoding and URL facts do not describe this document's environment",
            ),
            (
                DomEncodingUrlFactsError::Base(HtmlBaseMissing::DocumentUrl),
                "the base URL needed for encoding and URL facts is unavailable: the document URL is missing",
            ),
            (
                DomEncodingUrlFactsError::MissingFacts,
                "the document has no encoding and URL facts",
            ),
            (
                DomEncodingUrlFactsError::Producer(HtmlEncodingUrlFactsError::ForeignProducer),
                "the HTML semantics producer refused the encoding and URL facts",
            ),
        ],
        |error| match error {
            DomEncodingUrlFactsError::BasisRevisionMismatch => 0,
            DomEncodingUrlFactsError::EnvironmentMismatch => 1,
            DomEncodingUrlFactsError::Base(_) => 2,
            DomEncodingUrlFactsError::MissingFacts => 3,
            DomEncodingUrlFactsError::Producer(_) => 4,
        },
        |error| matches!(error, DomEncodingUrlFactsError::Producer(_)),
    );
    for (missing, clause) in MISSING {
        assert_eq!(
            DomEncodingUrlFactsError::Base(missing).to_string(),
            format!("the base URL needed for encoding and URL facts is unavailable: {clause}")
        );
    }
    let inner = HtmlEncodingUrlFactsError::RevisionMismatch;
    let outer = DomEncodingUrlFactsError::Producer(inner.clone());
    let source = outer.source().expect("producer error is the source");
    assert_eq!(source.to_string(), inner.to_string());
    assert!(
        !outer.to_string().contains(&inner.to_string()),
        "the wrapped error is reported by source(), not repeated in the text"
    );
}

#[test]
fn meta_errors_state_the_refused_processing() {
    check_table(
        vec![
            (
                DomMetaError::HistoryUnavailable,
                "the Document has no supplied Meta processing history",
            ),
            (
                DomMetaError::DefaultStyleUnsupported,
                "the preferred stylesheet-set operation is not supported by this mutation route for this Document",
            ),
            (
                DomMetaError::ContentSecurityPolicyUnsupported,
                "this mutation route cannot process a Content Security Policy Meta insertion",
            ),
            (
                DomMetaError::ThemeColorMediaUnavailable,
                "theme-color media matching beyond an empty or ASCII-whitespace query needs matching facts that were not supplied",
            ),
            (
                DomMetaError::ThemeColorUnavailable(
                    surgeist_css::color::CssColorUnavailable::CurrentColor,
                ),
                "a fact needed to parse the theme-color value was not supplied",
            ),
            (
                DomMetaError::ThemeColorUnsupported(
                    surgeist_css::color::CssColorUnsupported::ProfileConversion,
                ),
                "the theme-color value needs a color capability that is not supported",
            ),
            (
                DomMetaError::ThemeColorFacts(surgeist_css::color::CssColorFactDefect::Foreign),
                "the supplied theme-color facts are defective",
            ),
            (
                DomMetaError::ThemeColorScratchLimit { limit: 4096 },
                "theme-color processing exceeded its scratch allowance of 4096",
            ),
            (
                DomMetaError::ColorSchemeProcessingUnavailable,
                "no finite color-scheme processing allowance was supplied",
            ),
            (
                DomMetaError::ColorSchemeLimit(surgeist_css::source::CssUtf16Resource::InputUnits),
                "color-scheme processing exceeded its limit on original UTF16 units",
            ),
            (
                DomMetaError::CssProcessingProtocol,
                "the CSS property processing protocol was violated",
            ),
            (
                DomMetaError::WorkLimit { limit: 7 },
                "Meta processing exhausted its cumulative work allowance of 7",
            ),
        ],
        |error| match error {
            DomMetaError::HistoryUnavailable => 0,
            DomMetaError::DefaultStyleUnsupported => 1,
            DomMetaError::ContentSecurityPolicyUnsupported => 2,
            DomMetaError::ThemeColorMediaUnavailable => 3,
            DomMetaError::ThemeColorUnavailable(_) => 4,
            DomMetaError::ThemeColorUnsupported(_) => 5,
            DomMetaError::ThemeColorFacts(_) => 6,
            DomMetaError::ThemeColorScratchLimit { .. } => 7,
            DomMetaError::ColorSchemeProcessingUnavailable => 8,
            DomMetaError::ColorSchemeLimit(_) => 9,
            DomMetaError::CssProcessingProtocol => 10,
            DomMetaError::WorkLimit { .. } => 11,
        },
        |_| false,
    );
    for (resource, name) in [
        (CssUtf16Resource::OutputBytes, "normalized UTF8 bytes"),
        (CssUtf16Resource::Work, "preparation work"),
        (CssUtf16Resource::ComponentDepth, "property component depth"),
    ] {
        assert_eq!(
            DomMetaError::ColorSchemeLimit(resource).to_string(),
            format!("color-scheme processing exceeded its limit on {name}")
        );
    }
}

#[test]
fn view_transition_cssom_errors_state_the_rejected_association() {
    check_table(
        vec![
            (
                DomViewTransitionCssomError::ForeignOwner,
                "view transition CSSOM association rejected: the CSSOM object or revision belongs to a different CSSOM owner",
            ),
            (
                DomViewTransitionCssomError::MissingObject,
                "view transition CSSOM association rejected: the CSSOM snapshot could not provide the selected rule",
            ),
            (
                DomViewTransitionCssomError::WrongKind,
                "view transition CSSOM association rejected: the CSSOM object is not the expected kind of rule",
            ),
            (
                DomViewTransitionCssomError::WrongSheet,
                "view transition CSSOM association rejected: the rule or captured input belongs to a different style sheet",
            ),
            (
                DomViewTransitionCssomError::DuplicateRule,
                "view transition CSSOM association rejected: the same rule is selected for more than one role",
            ),
            (
                DomViewTransitionCssomError::RuleOrder,
                "view transition CSSOM association rejected: the selected rules are out of the required order",
            ),
            (
                DomViewTransitionCssomError::StaleRevision,
                "view transition CSSOM association rejected: the CSSOM revision is older than the one already recorded",
            ),
        ],
        |error| match error {
            DomViewTransitionCssomError::ForeignOwner => 0,
            DomViewTransitionCssomError::MissingObject => 1,
            DomViewTransitionCssomError::WrongKind => 2,
            DomViewTransitionCssomError::WrongSheet => 3,
            DomViewTransitionCssomError::DuplicateRule => 4,
            DomViewTransitionCssomError::RuleOrder => 5,
            DomViewTransitionCssomError::StaleRevision => 6,
        },
        |_| false,
    );
}

#[test]
fn event_clock_failures_state_why_no_time_was_supplied() {
    check_table(
        vec![
            (
                DomSourceEventClockFailure::Unavailable,
                "the event clock could not supply a monotonic time",
            ),
            (
                DomSourceEventClockFailure::Regressed,
                "the event clock's native reading preceded its latest earlier reading",
            ),
            (
                DomSourceEventClockFailure::Overflow,
                "the event clock's native time exceeded the representable range",
            ),
        ],
        |error| match error {
            DomSourceEventClockFailure::Unavailable => 0,
            DomSourceEventClockFailure::Regressed => 1,
            DomSourceEventClockFailure::Overflow => 2,
        },
        |_| false,
    );
}

// End-to-end: public operations that return these errors.

fn limits(meta_work: u64) -> DomLimits {
    DomLimits::new(
        128,
        131_072,
        512,
        64,
        64,
        8,
        262_144,
        surgeist_dom::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
    )
    .unwrap()
    .with_base_processing(128, 1_000_000)
    .unwrap()
    .with_encoding_url_processing(128, 1_000_000)
    .unwrap()
    .with_meta_processing_work(meta_work)
    .with_meta_theme_color_scratch_bytes(262_144)
    .unwrap()
}

fn revision() -> HtmlBaseFactsRevision {
    HtmlBaseFactsRevision::new(HtmlBaseProducerId::fresh(), 1, 2, 3)
}

fn record(revision: &HtmlBaseFactsRevision, spelling: &str) -> HtmlBaseUrlRecord {
    HtmlBaseUrlRecord::observed(
        revision.clone(),
        spelling.into(),
        HtmlBaseUrlScheme::Other,
        spelling == "about:blank",
        None,
    )
    .unwrap()
}

fn base_facts(revision: &HtmlBaseFactsRevision, raw: &str) -> HtmlParseBaseFacts {
    let fallback = record(revision, "about:blank");
    let encoding = HtmlBaseEncoding::fresh();
    let observation = HtmlBaseParseObservation::new(
        raw.into(),
        fallback.clone(),
        encoding.clone(),
        revision.clone(),
        HtmlBaseParseOutcome::Failure,
    )
    .unwrap();
    let document = HtmlBaseDocumentFacts::new(
        revision.clone(),
        Some(fallback),
        HtmlAboutBaseUrl::KnownNull,
        false,
        Some(encoding),
    )
    .unwrap();
    HtmlParseBaseFacts::new(
        document,
        vec![observation],
        HtmlBaseFactsLimits::new(128, 65_536, 1_000_000),
    )
    .unwrap()
}

fn bind_base(
    dom: &mut Dom,
    document: DocumentId,
    facts: &HtmlParseBaseFacts,
) -> Result<(), DomError> {
    let mut batch = DomBatch::new();
    batch.bind_base_facts(dom.snapshot().document(document).unwrap().stamp(), facts);
    dom.apply(batch, &cancel()).map(|_| ())
}

fn encoding_facts(revision: &HtmlBaseFactsRevision) -> HtmlEncodingUrlParseFacts {
    HtmlEncodingUrlParseFacts::new(
        record(revision, "about:blank"),
        HtmlBaseEncoding::fresh(),
        revision.clone(),
        vec![],
        HtmlBaseFactsLimits::new(128, 65_536, 1_000_000),
    )
    .unwrap()
}

fn fixture(meta_work: u64) -> (Dom, DocumentId, NodeId, NodeId) {
    let mut dom = Dom::new(limits(meta_work)).unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let root = batch.create_element_ns(&document, Namespace::html(), "head");
    let meta = batch.create_element_ns(&document, Namespace::html(), "meta");
    batch.insert_before(&root, &document, None);
    let commit = dom.apply(batch, &cancel()).unwrap();
    (
        dom,
        commit.document(&document).unwrap(),
        commit.created(&root).unwrap().unwrap(),
        commit.created(&meta).unwrap().unwrap(),
    )
}

#[test]
fn rebinding_different_base_facts_returns_messages_naming_the_mismatch() {
    let (mut dom, document, _, _) = fixture(1_000_000);
    let r = revision();
    bind_base(&mut dom, document, &base_facts(&r, "x")).unwrap();

    let substitute = base_facts(&r, "x");
    let Err(DomError::BaseFacts(error)) = bind_base(&mut dom, document, &substitute) else {
        panic!("a substitute for the bound facts is refused as a base-facts error");
    };
    assert_eq!(error, DomBaseFactsError::InputsMismatch);
    assert_eq!(
        error.to_string(),
        "base URL facts were computed from inconsistent inputs"
    );

    let foreign = base_facts(&revision(), "x");
    let Err(DomError::BaseFacts(error)) = bind_base(&mut dom, document, &foreign) else {
        panic!("facts of another producer are refused as a base-facts error");
    };
    assert_eq!(error, DomBaseFactsError::ForeignProducer);
    assert_eq!(
        error.to_string(),
        "base URL facts came from a different producer than the facts already bound to this document"
    );
}

#[test]
fn binding_encoding_facts_without_base_facts_reports_the_missing_base() {
    let (mut dom, document, _, _) = fixture(1_000_000);
    let facts = encoding_facts(&revision());
    let mut batch = DomBatch::new();
    batch.bind_encoding_url_facts(dom.snapshot().document(document).unwrap().stamp(), &facts);
    let Err(DomError::EncodingUrlFacts(error)) = dom.apply(batch, &cancel()) else {
        panic!("binding without base facts is refused as an encoding-facts error");
    };
    assert_eq!(
        error,
        DomEncodingUrlFactsError::Base(HtmlBaseMissing::DocumentFacts)
    );
    assert_eq!(
        error.to_string(),
        "the base URL needed for encoding and URL facts is unavailable: document facts are missing"
    );
    assert!(error.source().is_none());
}

#[test]
fn exhausted_meta_work_allowance_reports_the_allowance() {
    let (mut dom, _, root, meta) = fixture(1_000_000);
    let mut batch = DomBatch::new();
    batch.set_attribute(meta, "http-equiv", "content-language");
    batch.set_attribute(meta, "content", "same");
    batch.insert_before(meta, root, None);
    dom.apply(batch, &cancel()).unwrap();
    dom.set_limits(limits(0)).unwrap();
    let mut batch = DomBatch::new();
    batch.detach(meta);
    batch.insert_before(meta, root, None);
    let Err(DomError::Meta(error)) = dom.apply(batch, &cancel()) else {
        panic!("reprocessing under a zero allowance is refused as a Meta error");
    };
    assert_eq!(error, DomMetaError::WorkLimit { limit: 0 });
    assert_eq!(
        error.to_string(),
        "Meta processing exhausted its cumulative work allowance of 0"
    );
}

// Hosted, host, capture, inline-style, Meta composition and style-root errors.

/// A real node of a real owner, for variants that carry an identity.
fn some_node() -> NodeId {
    fixture(1_000_000).2
}

fn some_dom_error() -> DomError {
    DomError::StaleRange
}

/// Checks that `source()` returns exactly the wrapped error and that the
/// message does not repeat its text.
fn assert_chains<E: Error>(outer: &E, inner: &dyn Error) {
    let source = outer.source().expect("a wrapping variant has a source");
    assert_eq!(source.to_string(), inner.to_string());
    assert!(
        !outer.to_string().contains(&inner.to_string()),
        "the wrapped error is reported by source(), not repeated: {outer}"
    );
}

#[test]
fn hosted_errors_state_the_refused_hosted_operation() {
    let rows = vec![
        (
            DomHostedError::Dom(some_dom_error()),
            "the canonical DOM refused the hosted operation",
        ),
        (
            DomHostedError::AlreadyRegistered,
            "the DOM owner already has a hosted source registered",
        ),
        (
            DomHostedError::PriorDetailsTransfer,
            "a hosted source cannot be registered after details task custody was transferred",
        ),
        (
            DomHostedError::ForeignSource,
            "the identity, request or binding belongs to a different hosted source, DOM owner or Global",
        ),
        (
            DomHostedError::SourceExpired,
            "the hosted source that issued the identity has expired",
        ),
        (
            DomHostedError::SourcePoisoned,
            "the hosted source's state was poisoned by an earlier panic",
        ),
        (
            DomHostedError::StaleIdentity,
            "the hosted identity is unknown, stale or retired",
        ),
        (
            DomHostedError::WrongPhase,
            "the hosted state does not permit this step",
        ),
        (
            DomHostedError::MissingPublication,
            "a hosted preparation, publication or history record that the operation needs is missing",
        ),
        (
            DomHostedError::StaleBasis,
            "the supplied basis does not match the current hosted or DOM state",
        ),
        (
            DomHostedError::IncompleteOperation,
            "a hosted operation, job or association has not completed",
        ),
        (
            DomHostedError::UnsupportedProfile,
            "the selected hosted profile does not support this operation or lacks the hosted state it needs",
        ),
        (
            DomHostedError::ScriptingDisabled,
            "scripting is disabled for the Document",
        ),
        (
            DomHostedError::PendingDocumentObligations,
            "the Document still has pending obligations",
        ),
        // `CspProtocol` has no row: `HtmlCspProtocolError` has no public constructor.
        (
            DomHostedError::CspIssuerBusy,
            "the Content Security Policy issuer is already borrowed",
        ),
        (
            DomHostedError::ClockRegression,
            "the finish-loading clock reading is from another source or precedes the navigation start",
        ),
        (
            DomHostedError::InvalidClockSample,
            "the clock sample is not a multiple of 100 microseconds",
        ),
        (
            DomHostedError::InvalidEntryInitialization,
            "the entry initialization's navigation key or id is not a canonical version 4 UUID",
        ),
        (
            DomHostedError::RepresentationExhausted,
            "a hosted counter, size or credit cannot represent or cover the requested amount",
        ),
        (
            DomHostedError::AllocationFailed,
            "an allocation required by the hosted operation failed",
        ),
        (
            DomHostedError::Limit {
                resource: DomHostedResource::BrowsingContexts,
                limit: 4,
            },
            "the hosted browsing contexts limit of 4 was exceeded",
        ),
        (
            DomHostedError::WorkLimit { limit: 9 },
            "hosted processing exhausted its work allowance of 9",
        ),
    ];
    check_partial_table(
        rows,
        |error| match error {
            DomHostedError::Dom(_) => 0,
            DomHostedError::AlreadyRegistered => 1,
            DomHostedError::PriorDetailsTransfer => 2,
            DomHostedError::ForeignSource => 3,
            DomHostedError::SourceExpired => 4,
            DomHostedError::SourcePoisoned => 5,
            DomHostedError::StaleIdentity => 6,
            DomHostedError::WrongPhase => 7,
            DomHostedError::MissingPublication => 8,
            DomHostedError::StaleBasis => 9,
            DomHostedError::IncompleteOperation => 10,
            DomHostedError::UnsupportedProfile => 11,
            DomHostedError::ScriptingDisabled => 12,
            DomHostedError::PendingDocumentObligations => 13,
            DomHostedError::CspProtocol(_) => 14,
            DomHostedError::CspIssuerBusy => 15,
            DomHostedError::ClockRegression => 16,
            DomHostedError::InvalidClockSample => 17,
            DomHostedError::InvalidEntryInitialization => 18,
            DomHostedError::RepresentationExhausted => 19,
            DomHostedError::AllocationFailed => 20,
            DomHostedError::Limit { .. } => 21,
            DomHostedError::WorkLimit { .. } => 22,
        },
        |error| {
            matches!(
                error,
                DomHostedError::Dom(_) | DomHostedError::CspProtocol(_)
            )
        },
        &[14],
    );
    // `CspProtocol` wraps the HTML crate's protocol error, which has no public
    // constructor; this proves the wrapped type is a `core::error::Error`, the
    // condition under which `source()` returns it.
    fn _csp_protocol_error_is_an_error(
        error: &surgeist_html::security::csp::HtmlCspProtocolError,
    ) -> &dyn Error {
        error
    }
    for (resource, name) in [
        (DomHostedResource::Hosts, "hosts"),
        (DomHostedResource::BrowsingContexts, "browsing contexts"),
        (DomHostedResource::Navigables, "navigables"),
        (DomHostedResource::Entries, "entries"),
        (DomHostedResource::Globals, "globals"),
        (DomHostedResource::Operations, "operations"),
        (DomHostedResource::Registries, "registries"),
        (DomHostedResource::RetainedBytes, "retained bytes"),
        (DomHostedResource::ScratchBytes, "scratch bytes"),
        (DomHostedResource::Work, "work"),
    ] {
        assert_eq!(
            DomHostedError::Limit { resource, limit: 3 }.to_string(),
            format!("the hosted {name} limit of 3 was exceeded")
        );
    }
    let inner = some_dom_error();
    assert_chains(&DomHostedError::Dom(inner.clone()), &inner);
}

fn hosted_limits(contexts: usize) -> DomHostedLimits {
    DomHostedLimits::new(
        DomHostedRecordLimits::new(16, contexts, 16, 16, 16, 16, 16),
        16_000_000,
        16_000_000,
        4_000_000,
    )
}

#[test]
fn registering_a_second_hosted_source_reports_the_existing_registration() {
    let mut dom = Dom::new(limits(1_000_000)).unwrap();
    let _source = dom
        .register_hosted_source(hosted_limits(16), &cancel())
        .unwrap();
    let error = refusal(
        dom.register_hosted_source(hosted_limits(16), &cancel()),
        "a second registration is refused",
    );
    assert_eq!(error, DomHostedError::AlreadyRegistered);
    assert_eq!(
        error.to_string(),
        "the DOM owner already has a hosted source registered"
    );
}

#[test]
fn host_errors_state_the_refused_publication_or_stamp() {
    let (dom, _, _, _) = fixture(1_000_000);
    let revision = dom.snapshot().revision();
    let node = some_node();
    let rows = vec![
        (
            DomHostError::ForeignOwner,
            "the compared document stamps belong to different DOM owners",
        ),
        (
            DomHostError::WrongDocument,
            "the compared document stamps refer to different Documents",
        ),
        (
            DomHostError::DomRevisionMismatch {
                expected: revision,
                actual: revision,
            },
            "the compared document stamps have different DOM revisions",
        ),
        (
            DomHostError::StaleDomRevision {
                current: revision,
                actual: revision,
            },
            "publication would move the canonical DOM revision backwards",
        ),
        (
            DomHostError::DomainMismatch {
                expected: DomHostDomains::Focus,
                actual: DomHostDomains::Hover,
            },
            "the observations do not supply exactly this source's configured domains",
        ),
        (
            DomHostError::ForeignSource,
            "the stamp belongs to a different publisher or configuration",
        ),
        (
            DomHostError::FutureHostRevision,
            "the stamp has a later external revision than the current publication",
        ),
        (
            DomHostError::FutureDomRevision,
            "the stamp has a later canonical revision than the current publication",
        ),
        (
            DomHostError::Unpublished,
            "the publisher has not published a capture yet",
        ),
        (
            DomHostError::Subject {
                node,
                error: some_dom_error(),
            },
            "a selected subject is not a current Element of the same Document",
        ),
        (
            DomHostError::HostRevisionExhausted,
            "the external revision cannot advance for the changed observations",
        ),
    ];
    check_table(
        rows,
        |error| match error {
            DomHostError::ForeignOwner => 0,
            DomHostError::WrongDocument => 1,
            DomHostError::DomRevisionMismatch { .. } => 2,
            DomHostError::StaleDomRevision { .. } => 3,
            DomHostError::DomainMismatch { .. } => 4,
            DomHostError::ForeignSource => 5,
            DomHostError::FutureHostRevision => 6,
            DomHostError::FutureDomRevision => 7,
            DomHostError::Unpublished => 8,
            DomHostError::Subject { .. } => 9,
            DomHostError::HostRevisionExhausted => 10,
        },
        |error| matches!(error, DomHostError::Subject { .. }),
    );
    let inner = some_dom_error();
    assert_chains(
        &DomHostError::Subject {
            node,
            error: inner.clone(),
        },
        &inner,
    );
}

#[test]
fn publisher_refusals_return_messages_naming_the_cause() {
    let (mut dom, document, root, _) = fixture(1_000_000);
    let snapshot = dom.snapshot();
    let current = snapshot.document(document).unwrap();
    let mut publisher = DomHostPublisher::new(DomHostDomains::Focus);

    // A stamp checked against another owner's or another Document's view.
    let focus_observations = DomHostObservations::Focus(Observed::Unsupported);
    let capture = publisher
        .publish(current, current.stamp(), focus_observations)
        .unwrap();
    let (other_dom, other_document, _, _) = fixture(1_000_000);
    let other_snapshot = other_dom.snapshot();
    let error = capture
        .capture()
        .validate_for(other_snapshot.document(other_document).unwrap())
        .unwrap_err();
    assert_eq!(error, DomHostError::ForeignOwner);
    assert_eq!(
        error.to_string(),
        "the compared document stamps belong to different DOM owners"
    );
    let mut batch = DomBatch::new();
    let second = batch.create_document();
    let commit = dom.apply(batch, &cancel()).unwrap();
    let second = commit.document(&second).unwrap();
    let latest = dom.snapshot();
    let error = capture
        .capture()
        .validate_for(latest.document(second).unwrap())
        .unwrap_err();
    assert_eq!(error, DomHostError::WrongDocument);
    assert_eq!(
        error.to_string(),
        "the compared document stamps refer to different Documents"
    );
    let mut publisher = DomHostPublisher::new(DomHostDomains::Focus);

    let error = publisher.changes_since(None).unwrap_err();
    assert_eq!(error, DomHostError::Unpublished);
    assert_eq!(
        error.to_string(),
        "the publisher has not published a capture yet"
    );

    let hover = DomHostObservations::Hover(Observed::NotApplicable);
    let error = publisher
        .publish(current, current.stamp(), hover)
        .unwrap_err();
    assert_eq!(
        error,
        DomHostError::DomainMismatch {
            expected: DomHostDomains::Focus,
            actual: DomHostDomains::Hover,
        }
    );
    assert_eq!(
        error.to_string(),
        "the observations do not supply exactly this source's configured domains"
    );

    let focus = DomHostObservations::Focus(Observed::Unsupported);
    publisher.publish(current, current.stamp(), focus).unwrap();
    let mut batch = DomBatch::new();
    batch.set_attribute(root, "key", "value");
    dom.apply(batch, &cancel()).unwrap();
    let newer = dom.snapshot();
    let newer = newer.document(document).unwrap();
    let error = publisher
        .publish(newer, current.stamp(), focus)
        .unwrap_err();
    assert!(matches!(error, DomHostError::DomRevisionMismatch { .. }));
    assert_eq!(
        error.to_string(),
        "the compared document stamps have different DOM revisions"
    );
}

#[test]
fn html_capture_errors_state_the_refused_capture_and_chain_their_cause() {
    let rows = |semantic: DomHtmlCaptureError| {
        vec![
            (
                DomHtmlCaptureError::Source(some_dom_error()),
                "the canonical DOM refused the HTML capture",
            ),
            (
                DomHtmlCaptureError::Host(DomHostError::Unpublished),
                "the host observations do not match the Document being captured",
            ),
            (
                DomHtmlCaptureError::StaleScriptingObservation,
                "the scripting observation was not made for this snapshot's exact state",
            ),
            (
                DomHtmlCaptureError::Capture(HtmlCaptureError::ForeignBuilder),
                "the HTML capture refused its input",
            ),
            (
                DomHtmlCaptureError::ElementValues(
                    HtmlCaptureElementError::InconsistentAttributes { index: 0 },
                ),
                "the HTML capture could not read an element's attributes",
            ),
            (
                DomHtmlCaptureError::Context(HtmlSemanticContextError::MediaCaptureMismatch),
                "the HTML semantic context could not be formed from the capture",
            ),
            (semantic, "the HTML semantic query failed"),
            (
                DomHtmlCaptureError::Language(HtmlLanguageDirectionError::HostRevisionMismatch),
                "the language and direction facts could not be derived",
            ),
            (
                DomHtmlCaptureError::Inert(HtmlInertFactError::HostRevisionMismatch),
                "the inertness facts could not be derived",
            ),
            (
                DomHtmlCaptureError::GlobalInheritance(
                    HtmlGlobalInheritanceError::HostRevisionMismatch,
                ),
                "the global attribute inheritance facts could not be derived",
            ),
            (
                DomHtmlCaptureError::Controls(HtmlControlFactsError::HostRevisionMismatch),
                "the control facts could not be derived",
            ),
            (
                DomHtmlCaptureError::BaseFacts(HtmlBaseFactsError::HostRevisionMismatch),
                "the base URL facts could not be captured",
            ),
            (
                DomHtmlCaptureError::OutputDefaults(HtmlOutputDefaultError::HostRevisionMismatch),
                "the output default facts could not be derived",
            ),
            (
                DomHtmlCaptureError::ElementReferences(
                    HtmlElementReferenceFactsError::HostRevisionMismatch,
                ),
                "the element reference facts could not be derived",
            ),
            (
                DomHtmlCaptureError::ResourceLimit {
                    resource: DomHtmlCaptureResource::Subjects,
                    limit: 12,
                },
                "the HTML capture exceeded its subject limit of 12",
            ),
            (
                DomHtmlCaptureError::Cancelled,
                "the HTML capture was cancelled",
            ),
            (
                DomHtmlCaptureError::AllocationFailed,
                "an allocation required by the HTML capture failed",
            ),
            (
                DomHtmlCaptureError::RepresentationExhausted,
                "an HTML capture count or size exceeded the representable range",
            ),
            (
                DomHtmlCaptureError::UnmappedSubject,
                "a node, subject or registry could not be mapped within the capture",
            ),
        ]
    };
    check_table(
        rows(semantic_failure()),
        |error| match error {
            DomHtmlCaptureError::Source(_) => 0,
            DomHtmlCaptureError::Host(_) => 1,
            DomHtmlCaptureError::StaleScriptingObservation => 2,
            DomHtmlCaptureError::Capture(_) => 3,
            DomHtmlCaptureError::ElementValues(_) => 4,
            DomHtmlCaptureError::Context(_) => 5,
            DomHtmlCaptureError::Semantic(_) => 6,
            DomHtmlCaptureError::Language(_) => 7,
            DomHtmlCaptureError::Inert(_) => 8,
            DomHtmlCaptureError::GlobalInheritance(_) => 9,
            DomHtmlCaptureError::Controls(_) => 10,
            DomHtmlCaptureError::BaseFacts(_) => 11,
            DomHtmlCaptureError::OutputDefaults(_) => 12,
            DomHtmlCaptureError::ElementReferences(_) => 13,
            DomHtmlCaptureError::ResourceLimit { .. } => 14,
            DomHtmlCaptureError::Cancelled => 15,
            DomHtmlCaptureError::AllocationFailed => 16,
            DomHtmlCaptureError::RepresentationExhausted => 17,
            DomHtmlCaptureError::UnmappedSubject => 18,
        },
        |error| {
            !matches!(
                error,
                DomHtmlCaptureError::StaleScriptingObservation
                    | DomHtmlCaptureError::ResourceLimit { .. }
                    | DomHtmlCaptureError::Cancelled
                    | DomHtmlCaptureError::AllocationFailed
                    | DomHtmlCaptureError::RepresentationExhausted
                    | DomHtmlCaptureError::UnmappedSubject
            )
        },
    );
    assert_eq!(
        DomHtmlCaptureError::ResourceLimit {
            resource: DomHtmlCaptureResource::Work,
            limit: 5,
        }
        .to_string(),
        "the HTML capture exceeded its work allowance of 5"
    );
    let inner = some_dom_error();
    assert_chains(&DomHtmlCaptureError::Source(inner.clone()), &inner);
}

fn capture_limits(subjects: usize, work: usize) -> (HtmlCaptureLimits, DomHtmlCaptureLimits) {
    (
        HtmlCaptureLimits::new(128, 2048, 256, 32768, 128, 0, 0, 16_777_216),
        DomHtmlCaptureLimits::new(subjects, work),
    )
}

/// A real semantic failure: a cancelled query against a real capture.
fn semantic_failure() -> DomHtmlCaptureError {
    let (dom, document, root, _) = fixture(1_000_000);
    let (html, dom_limits) = capture_limits(128, 1_000_000);
    let capture = dom
        .snapshot()
        .html_capture(document, None, html, dom_limits, &cancel())
        .unwrap();
    let cancellation = HtmlSemanticCancellation::new();
    cancellation.cancel();
    capture
        .query(
            &[HtmlSemanticRequest::new(
                capture.subject(root).unwrap(),
                HtmlSemanticQuery::TableModel,
            )],
            surgeist_html::values::HtmlValueInterpreter::new(8192),
            HtmlSemanticLimits::new(10_000_000, 16, 4096, 256, 10_000_000, 4096),
            cancellation,
        )
        .expect_err("a cancelled query is refused")
}

#[test]
fn html_capture_refusals_return_messages_naming_the_cause() {
    let (dom, document, _, _) = fixture(1_000_000);
    let snapshot = dom.snapshot();

    let (html, dom_limits) = capture_limits(128, 0);
    let error = refusal(
        snapshot.html_capture(document, None, html, dom_limits, &cancel()),
        "a zero work allowance is refused",
    );
    assert!(matches!(
        error,
        DomHtmlCaptureError::ResourceLimit {
            resource: DomHtmlCaptureResource::Work,
            limit: 0
        }
    ));
    assert_eq!(
        error.to_string(),
        "the HTML capture exceeded its work allowance of 0"
    );

    let (html, dom_limits) = capture_limits(0, 1_000_000);
    let error = refusal(
        snapshot.html_capture(document, None, html, dom_limits, &cancel()),
        "a zero subject allowance is refused",
    );
    assert_eq!(
        error.to_string(),
        "the HTML capture exceeded its subject limit of 0"
    );

    let error = semantic_failure();
    assert!(matches!(error, DomHtmlCaptureError::Semantic(_)));
    assert_eq!(error.to_string(), "the HTML semantic query failed");
    assert!(error.source().is_some());
}

#[test]
fn inline_style_errors_state_the_refused_operation_and_chain_their_cause() {
    let node = some_node();
    let rows = vec![
        (
            DomInlineStyleError::Dom(some_dom_error()),
            "the canonical DOM refused the inline style operation",
        ),
        (
            DomInlineStyleError::Cssom(CssomError::StaleRevision),
            "the declaration producer refused the inline style operation",
        ),
        (
            DomInlineStyleError::Input(CssUtf16InputError::limit(CssUtf16Resource::InputUnits)),
            "preparing the original CSS input for the inline style failed",
        ),
        (
            DomInlineStyleError::Unconfigured,
            "no finite inline style policy has been installed",
        ),
        (
            DomInlineStyleError::AlreadyConfigured,
            "the inline style owner already has its fixed policy",
        ),
        (
            DomInlineStyleError::Inapplicable,
            "the Element's namespace has no supported ElementCSSInlineStyle capability",
        ),
        (
            DomInlineStyleError::StaleContext,
            "the supplied observation or owner effect no longer matches the current state",
        ),
        (
            DomInlineStyleError::ParserModeMismatch,
            "the selected parser mode does not describe the Document",
        ),
        (
            DomInlineStyleError::InvalidSupportRole,
            "the supplied input version is not a Support qualification",
        ),
        (
            DomInlineStyleError::ContextUpdateRequired,
            "the bound inline style block already exists with different explicit external inputs",
        ),
        (
            DomInlineStyleError::Unbound,
            "the inline declaration block or parked feedback operation the operation needs is absent or does not match",
        ),
        (
            DomInlineStyleError::OperationPending { element: node },
            "another inline style owner-write outcome is still resident",
        ),
        (
            DomInlineStyleError::ReconciliationRequiresTerminal {
                status: CssomOwnerEffectStatus::Applied,
            },
            "reconciliation requires a failed terminal owner effect, not a live or applied one",
        ),
        (
            DomInlineStyleError::ReconciliationNotRequired,
            "an effect-free inline style operation has no owner synchronization to reconcile",
        ),
        (
            DomInlineStyleError::TerminalEffect {
                status: CssomOwnerEffectStatus::Canceled,
            },
            "the owner effect is terminal and cannot be retried as a pending write",
        ),
    ];
    check_table(
        rows,
        |error| match error {
            DomInlineStyleError::Dom(_) => 0,
            DomInlineStyleError::Cssom(_) => 1,
            DomInlineStyleError::Input(_) => 2,
            DomInlineStyleError::Unconfigured => 3,
            DomInlineStyleError::AlreadyConfigured => 4,
            DomInlineStyleError::Inapplicable => 5,
            DomInlineStyleError::StaleContext => 6,
            DomInlineStyleError::ParserModeMismatch => 7,
            DomInlineStyleError::InvalidSupportRole => 8,
            DomInlineStyleError::ContextUpdateRequired => 9,
            DomInlineStyleError::Unbound => 10,
            DomInlineStyleError::OperationPending { .. } => 11,
            DomInlineStyleError::ReconciliationRequiresTerminal { .. } => 12,
            DomInlineStyleError::ReconciliationNotRequired => 13,
            DomInlineStyleError::TerminalEffect { .. } => 14,
        },
        |error| {
            matches!(
                error,
                DomInlineStyleError::Dom(_)
                    | DomInlineStyleError::Cssom(_)
                    | DomInlineStyleError::Input(_)
            )
        },
    );
    let dom_error = some_dom_error();
    assert_chains(&DomInlineStyleError::Dom(dom_error.clone()), &dom_error);
    let cssom = CssomError::StaleRevision;
    assert_chains(&DomInlineStyleError::Cssom(cssom.clone()), &cssom);
    let input = CssUtf16InputError::limit(CssUtf16Resource::InputUnits);
    assert_chains(&DomInlineStyleError::Input(input.clone()), &input);
}

fn inline_policy() -> DomInlineStyleLimits {
    DomInlineStyleLimits::new(CssomLimits::default(), 64, 64_000_000, 8_000_000, 4_000_000).unwrap()
}

fn inline_requests() -> DomInlineStyleRequestLimits {
    DomInlineStyleRequestLimits {
        input: CssUtf16InputLimits::default(),
        declaration: CssomDeclarationRequestLimits::default(),
    }
}

fn inline_inputs(dom: &Dom, document: DocumentId) -> DomInlineStyleInputs {
    DomInlineStyleInputs::new(
        dom.snapshot().document(document).unwrap().stamp(),
        CssParserContext::default(),
        CssomInputVersion {
            role: CssomInputRole::Support,
            identity: "error-text-profile".into(),
            revision: 1,
        },
        CssomDeclarationSupport::try_new(vec![(
            CssPropertyGrammar::from_name("width").unwrap(),
            CssomUsabilityRequirement::AllCheckedValues,
        )])
        .unwrap(),
    )
    .unwrap()
}

fn no_decisions(
    _: &[CssomDeclarationCandidate],
    _: &CssomContext,
) -> Result<CssomAdmission, CssomError> {
    panic!("no whole-declaration candidate")
}

#[test]
fn inline_style_refusals_return_messages_naming_the_cause() {
    let (mut dom, document, _, element) = fixture(1_000_000);
    let inputs = inline_inputs(&dom, document);

    let error = refusal(
        dom.inline_style(element, &inputs, inline_requests(), no_decisions, &cancel()),
        "binding without a policy is refused",
    );
    assert!(matches!(error, DomInlineStyleError::Unconfigured));
    assert_eq!(
        error.to_string(),
        "no finite inline style policy has been installed"
    );

    dom.configure_inline_styles(inline_policy(), &cancel())
        .unwrap();
    let error = dom
        .configure_inline_styles(inline_policy(), &cancel())
        .unwrap_err();
    assert!(matches!(error, DomInlineStyleError::AlreadyConfigured));
    assert_eq!(
        error.to_string(),
        "the inline style owner already has its fixed policy"
    );

    let mut batch = DomBatch::new();
    let other = batch.create_element_ns(document, Namespace::from_uri("urn:test"), "e");
    let commit = dom.apply(batch, &cancel()).unwrap();
    let other = commit.created(&other).unwrap().unwrap();
    let error = refusal(
        dom.inline_style(other, &inputs, inline_requests(), no_decisions, &cancel()),
        "an unknown namespace has no inline style",
    );
    assert!(matches!(error, DomInlineStyleError::Inapplicable));
    assert_eq!(
        error.to_string(),
        "the Element's namespace has no supported ElementCSSInlineStyle capability"
    );
}

#[test]
fn meta_composition_errors_state_the_refused_root_operation_and_chain_their_cause() {
    let root = some_node();
    let rows = vec![
        (
            DomMetaAtomicError::Dom(some_dom_error()),
            "the canonical DOM refused the native Meta composition",
        ),
        (
            DomMetaAtomicError::Inline(DomInlineStyleError::Unbound),
            "inline style synchronization refused the native Meta composition",
        ),
        (
            DomMetaAtomicError::OwnerBusy { root },
            "the CSSOM owner of the root is already borrowed, so no producer operation was attempted",
        ),
        (
            DomMetaAtomicError::OwnerPoisoned { root },
            "the CSSOM owner of the root is poisoned, so no producer operation was attempted",
        ),
        (
            DomMetaAtomicError::Cssom {
                root,
                error: CssomError::StaleRevision,
            },
            "the CSSOM producer refused the Meta composition operation for the root",
        ),
        (
            DomMetaAtomicError::MissingOwner { root },
            "no CSSOM owner was supplied or held for the root's Document",
        ),
        (
            DomMetaAtomicError::MismatchedOwner { root },
            "the CSSOM owner supplied for the root's Document differs from the owner already bound or supplied for it",
        ),
        (
            DomMetaAtomicError::StaleProducer { root },
            "the root's producer snapshot is stale or does not contain the sheet",
        ),
    ];
    check_table(
        rows,
        |error| match error {
            DomMetaAtomicError::Dom(_) => 0,
            DomMetaAtomicError::Inline(_) => 1,
            DomMetaAtomicError::OwnerBusy { .. } => 2,
            DomMetaAtomicError::OwnerPoisoned { .. } => 3,
            DomMetaAtomicError::Cssom { .. } => 4,
            DomMetaAtomicError::MissingOwner { .. } => 5,
            DomMetaAtomicError::MismatchedOwner { .. } => 6,
            DomMetaAtomicError::StaleProducer { .. } => 7,
        },
        |error| {
            matches!(
                error,
                DomMetaAtomicError::Dom(_)
                    | DomMetaAtomicError::Inline(_)
                    | DomMetaAtomicError::Cssom { .. }
            )
        },
    );
    let dom_error = some_dom_error();
    assert_chains(&DomMetaAtomicError::Dom(dom_error.clone()), &dom_error);
    assert_chains(
        &DomMetaAtomicError::Inline(DomInlineStyleError::Unbound),
        &DomInlineStyleError::Unbound,
    );
    let cssom = CssomError::StaleRevision;
    assert_chains(
        &DomMetaAtomicError::Cssom {
            root,
            error: cssom.clone(),
        },
        &cssom,
    );
}

#[test]
fn style_root_errors_state_the_refused_access_and_chain_the_producer() {
    let rows = vec![
        (
            DomStyleCssomRootError::Busy,
            "the style CSSOM root is in use by another operation",
        ),
        (
            DomStyleCssomRootError::Poisoned,
            "the style CSSOM root was poisoned by an earlier panic",
        ),
        (
            DomStyleCssomRootError::MissingBatch,
            "the commit envelope holds no unattempted CSSOM batch",
        ),
        (
            DomStyleCssomRootError::Cssom(CssomError::Cancelled),
            "the CSSOM producer rejected an attempted batch operation",
        ),
    ];
    check_table(
        rows,
        |error| match error {
            DomStyleCssomRootError::Busy => 0,
            DomStyleCssomRootError::Poisoned => 1,
            DomStyleCssomRootError::MissingBatch => 2,
            DomStyleCssomRootError::Cssom(_) => 3,
        },
        |error| matches!(error, DomStyleCssomRootError::Cssom(_)),
    );
    let cssom = CssomError::Cancelled;
    assert_chains(&DomStyleCssomRootError::Cssom(cssom.clone()), &cssom);
}

#[test]
fn constructed_sheet_errors_state_the_refused_construction_and_chain_their_cause() {
    let rows = vec![
        (
            DomHostedSheetConstructionError::Hosted(DomHostedError::WrongPhase),
            "the hosted issuer, Document or custody refused the constructed stylesheet operation",
        ),
        (
            DomHostedSheetConstructionError::Root(DomStyleCssomRootError::Busy),
            "the style CSSOM root could not be loaned for the constructed stylesheet",
        ),
        (
            DomHostedSheetConstructionError::Cssom(CssomError::Cancelled),
            "CSSOM staging or preparation refused the constructed stylesheet",
        ),
        (
            DomHostedSheetConstructionError::InvalidBaseSerialization {
                offset: 3,
                unit: 0xD800,
            },
            "the base URL serialization contains an isolated surrogate 0xd800 at offset 3",
        ),
    ];
    check_table(
        rows,
        |error| match error {
            DomHostedSheetConstructionError::Hosted(_) => 0,
            DomHostedSheetConstructionError::Root(_) => 1,
            DomHostedSheetConstructionError::Cssom(_) => 2,
            DomHostedSheetConstructionError::InvalidBaseSerialization { .. } => 3,
        },
        |error| {
            !matches!(
                error,
                DomHostedSheetConstructionError::InvalidBaseSerialization { .. }
            )
        },
    );
    let hosted = DomHostedError::WrongPhase;
    assert_chains(
        &DomHostedSheetConstructionError::Hosted(hosted.clone()),
        &hosted,
    );
    let root = DomStyleCssomRootError::Busy;
    assert_chains(&DomHostedSheetConstructionError::Root(root.clone()), &root);
    let cssom = CssomError::Cancelled;
    assert_chains(
        &DomHostedSheetConstructionError::Cssom(cssom.clone()),
        &cssom,
    );
}
