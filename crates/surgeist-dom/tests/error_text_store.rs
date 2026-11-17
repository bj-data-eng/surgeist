//! `DomError` and `DomHistoryError` implement the standard error traits and say, in words,
//! why an operation was refused. Expected sentences come from each variant's documented
//! meaning and from the condition a public operation reports it for, never from `Debug`.
use std::error::Error;
use std::sync::atomic::AtomicBool;
use surgeist_css::color::CssColorUnavailable;
use surgeist_dom::{
    DocumentId, Dom, DomBaseFactsError, DomBatch, DomDocumentConstructorError,
    DomEncodingUrlFactsError, DomError, DomHistoryError, DomLimits, DomMetaError, DomNameLimits,
    DomResource, DomViewTransitionCssomError, NodeId,
};
use surgeist_html::bounded::lexical::HtmlPiPseudoAttributeError;
use surgeist_html::bounded::numeric::HtmlRangeNumericError;
use surgeist_html::construction::{
    HtmlShadowRootMode, HtmlShadowRootProperties, HtmlSlotAssignment,
};
use surgeist_html::elements::form::HtmlOutputCursorError;
use surgeist_html::values::Namespace;
use surgeist_html_semantics::HtmlCaptureError;

fn assert_standard_error<E: Error + Send + Sync + 'static>() {}

#[test]
fn both_types_are_send_sync_standard_errors() {
    assert_standard_error::<DomError>();
    assert_standard_error::<DomHistoryError>();
}

fn limits() -> DomLimits {
    let names = DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap();
    DomLimits::new(32, 4096, 64, 64, 32, 8, 8192, names)
        .unwrap()
        .with_ranges(4)
        .unwrap()
        .with_select_processing(8, 4096, 100_000)
        .unwrap()
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
        (
            DomError::StaleNode,
            "the referenced node, registry or retained fact is stale or unavailable",
        ),
        (
            DomError::ForeignOwner,
            "the identity, basis or capability belongs to a different DOM owner",
        ),
        (
            DomError::ForeignBatch,
            "the ticket, operand or context belongs to a different batch",
        ),
        (
            DomError::WrongNodeKind,
            "the node is not of the kind the operation requires",
        ),
        (
            DomError::WrongDocument,
            "the node is not the expected Document or belongs to a different Document",
        ),
        (
            DomError::HierarchyRequest,
            "the node hierarchy does not allow this edit or position",
        ),
        (
            DomError::InvalidCharacter,
            "the name or value contains a character or sequence that is not allowed",
        ),
        (
            DomError::Namespace,
            "the qualified name and namespace are inconsistent",
        ),
        (
            DomError::NotFound,
            "the requested node or relation was not found",
        ),
        (DomError::IndexSize, "the index is out of range"),
        (
            DomError::InUseAttribute,
            "the Attr is already in use by another Element",
        ),
        (DomError::Cancelled, "the operation was cancelled"),
        (DomError::AllocationFailed, "memory allocation failed"),
        (
            DomError::SourceOperationPending,
            "a Source operation, callback or hold is still pending",
        ),
        (
            DomError::IncompleteSourceOperation,
            "a Source operation or its recorded inputs are incomplete and must finish first",
        ),
        (
            DomError::ButtonInapplicable,
            "the Element is not a Button with this reflected target",
        ),
        (DomError::SlotInapplicable, "the node is not a Slot"),
        (
            DomError::OwnerIdentityExhausted,
            "no DOM owner identities remain",
        ),
        (
            DomError::RevisionExhausted,
            "the revision counter cannot advance far enough for this operation",
        ),
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
        "the configured limit of 4 for ranges would be exceeded"
    );
    assert_eq!(
        DomError::WorkLimit {
            resource: DomResource::InlineStyleWork,
            limit: 5
        }
        .to_string(),
        "the allowance of 5 for inline style work was exhausted"
    );
    assert_eq!(
        DomError::InvalidLimit(DomResource::ManagedNodes).to_string(),
        "a configured limit is invalid"
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
        "the node hierarchy does not allow this edit or position"
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

// ---------------------------------------------------------------------------------------
// End-to-end cases. Each one obtains the error from a public operation and asserts a sentence
// written from that operation's contract (the DOM Standard's term for the failure, or the
// documented meaning of the limit), not from the message list in the crate.
// ---------------------------------------------------------------------------------------

fn no_cancel() -> AtomicBool {
    AtomicBool::new(false)
}

struct World {
    dom: Dom,
    document: DocumentId,
    div: NodeId,
    text: NodeId,
    comment: NodeId,
}

fn world_with(limits: DomLimits) -> World {
    let mut dom = Dom::new(limits).unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let div = batch.create_element_ns(&document, Namespace::html(), "div");
    batch.insert_before(&div, &document, None);
    let text = batch.create_text(&document, "hello");
    batch.insert_before(&text, &div, None);
    let comment = batch.create_comment(&document, "note");
    batch.insert_before(&comment, &div, None);
    let committed = dom.apply(batch, &no_cancel()).unwrap();
    World {
        dom,
        document: committed.document(&document).unwrap(),
        div: committed.created(&div).unwrap().unwrap(),
        text: committed.created(&text).unwrap().unwrap(),
        comment: committed.created(&comment).unwrap().unwrap(),
    }
}

fn world() -> World {
    world_with(limits())
}

fn refused(dom: &mut Dom, batch: DomBatch) -> DomError {
    match dom.apply(batch, &no_cancel()) {
        Err(error) => error,
        Ok(_) => panic!("the batch was accepted"),
    }
}

#[track_caller]
fn assert_refusal(error: &DomError, variant: &DomError, text: &str) {
    assert_eq!(error, variant);
    assert_eq!(error.to_string(), text);
    assert!(error.source().is_none());
}

#[test]
fn an_element_under_a_comment_is_a_hierarchy_request_error() {
    let mut w = world();
    let mut batch = DomBatch::new();
    let element = batch.create_element_ns(w.document, Namespace::html(), "span");
    batch.insert_before(&element, w.comment, None);
    let error = refused(&mut w.dom, batch);
    assert_refusal(
        &error,
        &DomError::HierarchyRequest,
        "the node hierarchy does not allow this edit or position",
    );
}

#[test]
fn a_second_document_element_is_a_hierarchy_request_error() {
    let mut w = world();
    let mut batch = DomBatch::new();
    let element = batch.create_element_ns(w.document, Namespace::html(), "p");
    batch.insert_before(&element, w.document, None);
    let error = refused(&mut w.dom, batch);
    assert_refusal(
        &error,
        &DomError::HierarchyRequest,
        "the node hierarchy does not allow this edit or position",
    );
}

#[test]
fn a_name_that_starts_with_a_digit_is_an_invalid_character_error() {
    let mut w = world();
    let mut batch = DomBatch::new();
    batch.create_element(w.document, "1bad");
    let error = refused(&mut w.dom, batch);
    assert_refusal(
        &error,
        &DomError::InvalidCharacter,
        "the name or value contains a character or sequence that is not allowed",
    );
}

#[test]
fn a_prefix_without_a_namespace_is_a_namespace_error() {
    let mut w = world();
    let mut batch = DomBatch::new();
    batch.create_element_ns(w.document, Namespace::none(), "p:a");
    let error = refused(&mut w.dom, batch);
    assert_refusal(
        &error,
        &DomError::Namespace,
        "the qualified name and namespace are inconsistent",
    );
}

#[test]
fn setting_an_attribute_on_a_text_node_names_the_wrong_node_kind() {
    let mut w = world();
    let mut batch = DomBatch::new();
    batch.set_attribute(w.text, "id", "x");
    let error = refused(&mut w.dom, batch);
    assert_refusal(
        &error,
        &DomError::WrongNodeKind,
        "the node is not of the kind the operation requires",
    );
}

#[test]
fn splitting_a_text_node_past_its_end_is_an_index_size_error() {
    let mut w = world();
    let mut batch = DomBatch::new();
    batch.split_text(w.text, 99);
    let error = refused(&mut w.dom, batch);
    assert_refusal(&error, &DomError::IndexSize, "the index is out of range");
}

#[test]
fn replacing_a_node_that_is_not_a_child_is_a_not_found_error() {
    let mut w = world();
    let mut batch = DomBatch::new();
    let replacement = batch.create_comment(w.document, "r");
    batch.replace_child(&replacement, w.div, w.document.node());
    let error = refused(&mut w.dom, batch);
    assert_refusal(
        &error,
        &DomError::NotFound,
        "the requested node or relation was not found",
    );
}

#[test]
fn a_second_shadow_root_on_a_host_is_a_not_supported_error() {
    let mut w = world();
    let properties = || {
        HtmlShadowRootProperties::new(
            HtmlShadowRootMode::Open,
            HtmlSlotAssignment::Manual,
            false,
            false,
            false,
        )
    };
    let mut first = DomBatch::new();
    first.attach_shadow(w.div, properties());
    w.dom.apply(first, &no_cancel()).unwrap();
    let mut second = DomBatch::new();
    second.attach_shadow(w.div, properties());
    let error = refused(&mut w.dom, second);
    assert_refusal(
        &error,
        &DomError::NotSupported,
        "the operation is not supported here",
    );
}

#[test]
fn a_document_cannot_be_destroyed_directly() {
    let mut w = world();
    let mut batch = DomBatch::new();
    batch.destroy(w.document.node());
    let error = refused(&mut w.dom, batch);
    assert_refusal(
        &error,
        &DomError::InvalidDestruction,
        "the node is attached or is a Document or ShadowRoot, so it cannot be destroyed directly",
    );
}

#[test]
fn a_ticket_from_another_batch_is_refused() {
    let mut w = world();
    let mut other = DomBatch::new();
    let foreign = other.create_comment(w.document, "x");
    let mut batch = DomBatch::new();
    batch.insert_before(&foreign, w.div, None);
    let error = refused(&mut w.dom, batch);
    assert_refusal(
        &error,
        &DomError::ForeignBatch,
        "the ticket, operand or context belongs to a different batch",
    );
}

#[test]
fn a_node_of_another_dom_is_refused_by_a_snapshot() {
    let w = world();
    let other = Dom::new(limits()).unwrap();
    let error = other.snapshot().node(w.div).map(|_| ()).unwrap_err();
    assert_refusal(
        &error,
        &DomError::ForeignOwner,
        "the identity, basis or capability belongs to a different DOM owner",
    );
}

#[test]
fn a_destroyed_node_is_stale_in_a_later_snapshot() {
    let mut w = world();
    let mut batch = DomBatch::new();
    batch.detach(w.comment);
    batch.destroy(w.comment);
    w.dom.apply(batch, &no_cancel()).unwrap();
    let error = w.dom.snapshot().node(w.comment).map(|_| ()).unwrap_err();
    assert_refusal(
        &error,
        &DomError::StaleNode,
        "the referenced node, registry or retained fact is stale or unavailable",
    );
}

#[test]
fn reading_a_slot_from_an_element_that_is_not_a_slot_is_refused() {
    let w = world();
    let error = w.dom.snapshot().slot(w.div).map(|_| ()).unwrap_err();
    assert_refusal(
        &error,
        &DomError::SlotInapplicable,
        "the node is not a Slot",
    );
}

#[test]
fn control_setters_on_a_div_name_the_element_they_need() {
    let cases: [(fn(&mut DomBatch, NodeId), DomError, &str); 7] = [
        (
            |b, n| b.set_input_checked(n, true),
            DomError::InputInapplicable,
            "the Element is not an Input",
        ),
        (
            |b, n| b.set_textarea_value(n, "x"),
            DomError::TextareaInapplicable,
            "the Element is not a Textarea",
        ),
        (
            |b, n| b.set_output_value(n, "x"),
            DomError::OutputInapplicable,
            "the Element is not an Output",
        ),
        (
            |b, n| b.set_select_value(n, "x"),
            DomError::SelectInapplicable,
            "the Element is not a Select",
        ),
        (
            |b, n| b.set_option_selected(n, true),
            DomError::OptionInapplicable,
            "the Element is not an Option",
        ),
        (
            |b, n| b.start_marquee(n),
            DomError::MarqueeInapplicable,
            "the Element is not a Marquee",
        ),
        (
            |b, n| b.set_custom_validity(n, "x"),
            DomError::CustomValidityInapplicable,
            "the Element has no custom validity",
        ),
    ];
    for (setter, variant, text) in cases {
        let mut w = world();
        let mut batch = DomBatch::new();
        setter(&mut batch, w.div);
        let error = refused(&mut w.dom, batch);
        assert_refusal(&error, &variant, text);
    }
}

#[test]
fn exceeding_the_node_limit_names_the_nodes_and_the_limit() {
    let names = DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap();
    let mut dom = Dom::new(DomLimits::new(2, 4096, 64, 0, 0, 8, 8192, names).unwrap()).unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    for _ in 0..3 {
        batch.create_comment(&document, "c");
    }
    let error = refused(&mut dom, batch);
    assert_eq!(
        error,
        DomError::ResourceLimit {
            resource: DomResource::ManagedNodes,
            limit: 2
        }
    );
    assert_eq!(
        error.to_string(),
        "the configured limit of 2 for managed nodes would be exceeded"
    );
}

#[test]
fn exceeding_the_command_limit_names_the_commands_and_the_limit() {
    let names = DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap();
    let mut dom = Dom::new(DomLimits::new(32, 4096, 2, 0, 0, 8, 8192, names).unwrap()).unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    for _ in 0..4 {
        batch.create_comment(&document, "c");
    }
    let error = refused(&mut dom, batch);
    assert_eq!(
        error,
        DomError::ResourceLimit {
            resource: DomResource::Commands,
            limit: 2
        }
    );
    assert_eq!(
        error.to_string(),
        "the configured limit of 2 for batch commands would be exceeded"
    );
}

#[test]
fn exceeding_the_string_limit_names_the_code_units_and_the_limit() {
    let names = DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap();
    let mut dom = Dom::new(DomLimits::new(32, 8, 64, 0, 0, 8, 8192, names).unwrap()).unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    batch.create_text(&document, "this text is longer than eight code units");
    let error = refused(&mut dom, batch);
    assert_eq!(
        error,
        DomError::ResourceLimit {
            resource: DomResource::StringCodeUnits,
            limit: 8
        }
    );
    assert_eq!(
        error.to_string(),
        "the configured limit of 8 for string code units would be exceeded"
    );
}

#[test]
fn exhausting_the_name_work_allowance_names_the_work_once() {
    let names = DomNameLimits::new(16_777_216, 16_384, 3, 1_048_576).unwrap();
    let mut dom = Dom::new(DomLimits::new(32, 4096, 64, 0, 0, 8, 8192, names).unwrap()).unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    batch.create_element_ns(&document, Namespace::html(), "a-rather-long-element-name");
    let error = refused(&mut dom, batch);
    assert_eq!(
        error,
        DomError::WorkLimit {
            resource: DomResource::NameWork,
            limit: 3
        }
    );
    assert_eq!(
        error.to_string(),
        "the allowance of 3 for name work was exhausted"
    );
}

#[test]
fn an_unrepresentable_limit_is_an_invalid_limit_without_naming_a_resource() {
    let names = DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap();
    let error = DomLimits::new(usize::MAX, 4096, 64, 0, 0, 8, 8192, names).unwrap_err();
    assert!(matches!(error, DomError::InvalidLimit(_)));
    assert_eq!(error.to_string(), "a configured limit is invalid");
    assert!(error.source().is_none());
}

#[test]
fn a_range_boundary_past_its_container_is_an_index_size_error() {
    let mut w = world();
    let mut batch = DomBatch::new();
    let range = batch.create_range(w.document);
    batch.set_range_start(&range, w.text, 99);
    let error = refused(&mut w.dom, batch);
    assert_refusal(&error, &DomError::IndexSize, "the index is out of range");
}

#[test]
fn reading_a_retired_range_is_a_stale_range_error() {
    let mut w = world();
    let mut batch = DomBatch::new();
    let created = batch.create_range(w.document);
    let committed = w.dom.apply(batch, &no_cancel()).unwrap();
    let id = committed.created_range(&created).unwrap().unwrap();
    let mut retire = DomBatch::new();
    retire.retire_range(id);
    w.dom.apply(retire, &no_cancel()).unwrap();
    let error = w.dom.snapshot().range(id).map(|_| ()).unwrap_err();
    assert_refusal(&error, &DomError::StaleRange, "the Range is stale");
}

#[test]
fn a_baseline_older_than_the_retained_history_must_be_recomputed() {
    let names = DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap();
    let mut dom = Dom::new(DomLimits::new(32, 4096, 64, 0, 0, 1, 8192, names).unwrap()).unwrap();
    let baseline = dom.snapshot().stamp();
    for _ in 0..3 {
        let mut batch = DomBatch::new();
        batch.create_document();
        dom.apply(batch, &no_cancel()).unwrap();
    }
    let error = dom.changes_since(baseline).unwrap_err();
    assert!(matches!(
        error,
        DomHistoryError::FullRecomputeRequired { .. }
    ));
    assert_eq!(
        error.to_string(),
        "the retained history no longer reaches back to the baseline; recompute from a snapshot"
    );
}

#[test]
fn creating_more_ranges_than_configured_names_the_ranges_and_the_limit() {
    let mut w = world_with(limits().with_ranges(1).unwrap());
    let mut batch = DomBatch::new();
    batch.create_range(w.document);
    batch.create_range(w.document);
    let error = refused(&mut w.dom, batch);
    assert_eq!(
        error,
        DomError::ResourceLimit {
            resource: DomResource::Ranges,
            limit: 1
        }
    );
    assert_eq!(
        error.to_string(),
        "the configured limit of 1 for ranges would be exceeded"
    );
}

#[test]
fn select_work_is_refused_when_no_work_is_allowed() {
    let names = DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap();
    let limits = DomLimits::new(32, 4096, 64, 0, 0, 8, 8192, names).unwrap();
    let mut w = world_with(limits);
    let mut batch = DomBatch::new();
    batch.set_option_selected(w.div, true);
    let error = refused(&mut w.dom, batch);
    assert_eq!(error, DomError::SelectWorkLimit { limit: 0 });
    assert_eq!(
        error.to_string(),
        "Select processing exhausted its work allowance of 0"
    );
}

#[test]
fn an_operand_of_another_dom_is_refused_by_a_batch() {
    let mut w = world();
    let other = world();
    let mut batch = DomBatch::new();
    batch.insert_before(other.comment, w.div, None);
    let error = refused(&mut w.dom, batch);
    assert_refusal(
        &error,
        &DomError::ForeignOwner,
        "the identity, basis or capability belongs to a different DOM owner",
    );
}

#[test]
fn adopting_into_a_node_that_is_not_a_document_is_refused() {
    let mut w = world();
    let mut batch = DomBatch::new();
    let comment = batch.create_comment(w.document, "x");
    batch.adopt(&comment, w.div);
    let error = refused(&mut w.dom, batch);
    assert_refusal(
        &error,
        &DomError::WrongNodeKind,
        "the node is not of the kind the operation requires",
    );
}
