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

use surgeist_dom::{
    DocumentId, Dom, DomBaseFactsError, DomBatch, DomDocumentConstructorError,
    DomEncodingUrlFactsError, DomError, DomLimits, DomMetaError, DomSourceEventClockFailure,
    DomViewTransitionCssomError, NodeId,
};
use surgeist_html::values::Namespace;
use surgeist_html_semantics::{
    HtmlAboutBaseUrl, HtmlBaseDocumentFacts, HtmlBaseEncoding, HtmlBaseFactsLimits,
    HtmlBaseFactsRevision, HtmlBaseMissing, HtmlBaseParseObservation, HtmlBaseParseOutcome,
    HtmlBaseProducerId, HtmlBaseUrlRecord, HtmlBaseUrlScheme, HtmlEncodingUrlFactsError,
    HtmlEncodingUrlParseFacts, HtmlParseBaseFacts,
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
    for (position, (value, expected)) in rows.iter().enumerate() {
        assert_eq!(index(value), position, "row order for {value:?}");
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
            "base URL facts were captured for a different document environment",
        ),
        (
            DomBaseFactsError::ForeignProducer,
            "base URL facts came from a producer that does not belong to this document",
        ),
        (
            DomBaseFactsError::InputsMismatch,
            "base URL facts were computed from inputs that differ from the document's current inputs",
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
            "base URL facts were produced under a different policy",
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
                "the Document constructor context was issued for a different DOM owner",
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
                "the Document constructor context's Global association is stale or in the wrong phase",
            ),
            (
                DomDocumentConstructorError::IncompleteAssociation,
                "the Document constructor context's association has an incomplete operation",
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
                "encoding and URL facts were computed for a different document environment",
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
                "the preferred stylesheet-set operation requires its CSSOM owner",
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
                "the theme-color value is unavailable",
            ),
            (
                DomMetaError::ThemeColorUnsupported(
                    surgeist_css::color::CssColorUnsupported::ProfileConversion,
                ),
                "the theme-color value is not supported",
            ),
            (
                DomMetaError::ThemeColorFacts(surgeist_css::color::CssColorFactDefect::Foreign),
                "the theme-color facts are defective",
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
                "color-scheme processing exceeded an input limit",
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
}

#[test]
fn view_transition_cssom_errors_state_the_rejected_association() {
    check_table(
        vec![
            (
                DomViewTransitionCssomError::ForeignOwner,
                "view transition CSSOM association rejected: the CSSOM object belongs to a different DOM owner",
            ),
            (
                DomViewTransitionCssomError::MissingObject,
                "view transition CSSOM association rejected: the CSSOM object does not exist",
            ),
            (
                DomViewTransitionCssomError::WrongKind,
                "view transition CSSOM association rejected: the CSSOM object is not the expected kind of rule",
            ),
            (
                DomViewTransitionCssomError::WrongSheet,
                "view transition CSSOM association rejected: the rule belongs to a different style sheet",
            ),
            (
                DomViewTransitionCssomError::DuplicateRule,
                "view transition CSSOM association rejected: the style sheet already contains this rule",
            ),
            (
                DomViewTransitionCssomError::RuleOrder,
                "view transition CSSOM association rejected: the rule is out of the required order",
            ),
            (
                DomViewTransitionCssomError::StaleRevision,
                "view transition CSSOM association rejected: the CSSOM object's revision is stale",
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
        "base URL facts were computed from inputs that differ from the document's current inputs"
    );

    let foreign = base_facts(&revision(), "x");
    let Err(DomError::BaseFacts(error)) = bind_base(&mut dom, document, &foreign) else {
        panic!("facts of another producer are refused as a base-facts error");
    };
    assert_eq!(error, DomBaseFactsError::ForeignProducer);
    assert_eq!(
        error.to_string(),
        "base URL facts came from a producer that does not belong to this document"
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
