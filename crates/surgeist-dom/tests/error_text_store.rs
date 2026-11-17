//! `DomError` and `DomHistoryError` implement the standard error traits and say, in words,
//! why an operation was refused. Expected sentences come from each variant's documented
//! meaning and from the condition a public operation reports it for, never from `Debug`.
use std::error::Error;
use std::sync::atomic::AtomicBool;
use surgeist_css::color::CssColorUnavailable;
use surgeist_dom::{
    Dom, DomBaseFactsError, DomBatch, DomDocumentConstructorError, DomEncodingUrlFactsError,
    DomError, DomHistoryError, DomLimits, DomMetaError, DomNameLimits, DomResource,
    DomViewTransitionCssomError,
};
use surgeist_html::bounded::lexical::HtmlPiPseudoAttributeError;
use surgeist_html::bounded::numeric::HtmlRangeNumericError;
use surgeist_html::elements::form::HtmlOutputCursorError;
use surgeist_html_semantics::HtmlCaptureError;

fn assert_standard_error<E: Error + Send + Sync + 'static>() {}

#[test]
fn both_types_are_send_sync_standard_errors() {
    assert_standard_error::<DomError>();
    assert_standard_error::<DomHistoryError>();
}

fn limits() -> DomLimits {
    let names = DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap();
    DomLimits::new(32, 4096, 64, 0, 0, 8, 8192, names).unwrap()
}

#[test]
fn variants_without_payload_state_their_condition() {
    let rows: [(DomError, &str); 24] = [
        (DomError::StaleRange, "the Range is stale"),
        (DomError::StaleNodeIterator, "the NodeIterator is stale"),
        (
            DomError::StaleChildListObserver,
            "the child-list observer is stale",
        ),
        (DomError::StaleNode, "the node identity is stale"),
        (
            DomError::ForeignOwner,
            "the identity belongs to a different DOM owner",
        ),
        (
            DomError::ForeignBatch,
            "the creation ticket belongs to a different batch",
        ),
        (
            DomError::WrongNodeKind,
            "the node is not of the kind the operation requires",
        ),
        (
            DomError::WrongDocument,
            "the node belongs to a different Document",
        ),
        (
            DomError::HierarchyRequest,
            "the edit would produce an invalid node hierarchy",
        ),
        (
            DomError::InvalidCharacter,
            "the name contains a character that is not allowed",
        ),
        (
            DomError::Namespace,
            "the qualified name and namespace are inconsistent",
        ),
        (DomError::NotFound, "the requested node was not found"),
        (DomError::IndexSize, "the index is out of range"),
        (
            DomError::InUseAttribute,
            "the Attr is already in use by another Element",
        ),
        (DomError::Cancelled, "the operation was cancelled"),
        (DomError::AllocationFailed, "memory allocation failed"),
        (
            DomError::SourceOperationPending,
            "a Source operation is already pending",
        ),
        (
            DomError::IncompleteSourceOperation,
            "an incomplete Source operation must finish first",
        ),
        (DomError::ButtonInapplicable, "the Element is not a Button"),
        (DomError::SlotInapplicable, "the node is not a Slot"),
        (
            DomError::OwnerIdentityExhausted,
            "no DOM owner identities remain",
        ),
        (DomError::RevisionExhausted, "no store revisions remain"),
        (
            DomError::SourceLimitReduction,
            "a Source policy call tried to reduce an allowance, which may only be raised",
        ),
        (
            DomError::DuplicateViewTransitionName,
            "another view transition already uses this name",
        ),
    ];
    for (error, text) in rows {
        assert_eq!(error.to_string(), text);
        assert_ne!(error.to_string(), format!("{error:?}"));
        assert!(error.source().is_none(), "{text}");
    }
}

#[test]
fn scalar_payloads_are_appended_and_resources_are_named() {
    assert_eq!(
        DomError::SourceWorkLimit { limit: 41 }.to_string(),
        "the Source operation exhausted its work allowance of 41"
    );
    assert_eq!(
        DomError::ChildListWorkLimit { limit: 8 }.to_string(),
        "child-list observation exhausted its work allowance of 8"
    );
    assert_eq!(
        DomError::ResourceLimit {
            resource: DomResource::Ranges,
            limit: 4
        }
        .to_string(),
        "ranges would exceed its configured limit of 4"
    );
    assert_eq!(
        DomError::WorkLimit {
            resource: DomResource::StringCodeUnits,
            limit: 5
        }
        .to_string(),
        "string code units work exhausted its allowance of 5"
    );
    assert_eq!(
        DomError::InvalidLimit(DomResource::ManagedNodes).to_string(),
        "the configured limit for managed nodes exceeds the largest representable allocation"
    );
}

#[test]
fn wrapping_variants_return_the_wrapped_error_as_their_source() {
    let wrapped: [(DomError, &str); 8] = [
        (
            DomError::Meta(DomMetaError::HistoryUnavailable),
            "the Document has no supplied Meta processing history",
        ),
        (
            DomError::BaseFacts(DomBaseFactsError::PolicyMismatch),
            "base URL facts were produced under a different policy",
        ),
        (
            DomError::EncodingUrlFacts(DomEncodingUrlFactsError::MissingFacts),
            "the document has no encoding and URL facts",
        ),
        (
            DomError::DocumentConstructor(DomDocumentConstructorError::Unavailable),
            "no hosted association is available for the Document constructor",
        ),
        (
            DomError::ViewTransitionCssom(DomViewTransitionCssomError::StaleRevision),
            "view transition CSSOM association rejected: the CSSOM object's revision is stale",
        ),
        (DomError::InputNumeric(HtmlRangeNumericError::Integrity), ""),
        (
            DomError::PseudoAttributeCursor(HtmlPiPseudoAttributeError::Integrity),
            "",
        ),
        (DomError::SelectCapture(HtmlCaptureError::NotHtmlLink), ""),
    ];
    for (error, inner) in wrapped {
        let source = error.source().expect("a wrapping variant has a source");
        if !inner.is_empty() {
            assert_eq!(source.to_string(), inner);
        } else {
            assert!(!source.to_string().is_empty());
        }
        assert!(!error.to_string().contains(&source.to_string()) || inner.is_empty());
    }
    assert!(
        DomError::OutputProtocol(HtmlOutputCursorError)
            .source()
            .is_some()
    );
}

#[test]
fn wrapping_variants_state_only_their_own_context() {
    assert_eq!(
        DomError::Meta(DomMetaError::HistoryUnavailable).to_string(),
        "native Meta processing refused the operation"
    );
    assert_eq!(
        DomError::DocumentConstructor(DomDocumentConstructorError::Unavailable).to_string(),
        "the Document constructor refused the request"
    );
    assert_eq!(
        DomError::BaseFacts(DomBaseFactsError::PolicyMismatch).to_string(),
        "the base URL facts were refused"
    );
}

#[test]
fn data_payloads_without_an_error_implementation_give_no_source() {
    let error = DomError::InputColorUnavailable(CssColorUnavailable::CurrentColor);
    assert_eq!(error.to_string(), "the Input color is unavailable");
    assert!(error.source().is_none());
}

#[test]
fn cancelling_a_batch_reports_a_cancellation_in_words() {
    let mut dom = Dom::new(limits()).unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let comment = batch.create_comment(&document, "c");
    batch.insert_before(&comment, &document, None);
    let error = dom.apply(batch, &AtomicBool::new(true)).unwrap_err();
    assert_eq!(error, DomError::Cancelled);
    assert_eq!(error.to_string(), "the operation was cancelled");
}

#[test]
fn inserting_a_document_under_a_comment_reports_an_invalid_hierarchy() {
    let mut dom = Dom::new(limits()).unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let comment = batch.create_comment(&document, "c");
    let other = batch.create_comment(&document, "d");
    batch.insert_before(&comment, &document, None);
    batch.insert_before(&other, &comment, None);
    let error = dom.apply(batch, &AtomicBool::new(false)).unwrap_err();
    assert_eq!(
        error.to_string(),
        "the edit would produce an invalid node hierarchy"
    );
}

#[test]
fn reading_changes_for_another_owner_reports_a_foreign_baseline() {
    let mut first = Dom::new(limits()).unwrap();
    let second = Dom::new(limits()).unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let comment = batch.create_comment(&document, "c");
    batch.insert_before(&comment, &document, None);
    first.apply(batch, &AtomicBool::new(false)).unwrap();
    let error = first.changes_since(second.snapshot().stamp()).unwrap_err();
    assert_eq!(error, DomHistoryError::ForeignOwner);
    assert_eq!(
        error.to_string(),
        "the baseline stamp belongs to a different DOM owner"
    );
    assert!(error.source().is_none());
}
