//! DOM3071 #slot-assigned-nodes starts empty, including outside a shadow tree.
//! HTML877d dir=auto uses ordinary contained text for a non-shadow Slot.
use surgeist_html::{
    Attribute, HtmlDocumentMode, HtmlValueInterpreter, Namespace, OrderedAttributes, QualifiedName,
};
use surgeist_html_semantics::{
    HtmlCapture, HtmlCaptureBuilder, HtmlCaptureLimits, HtmlCaptureRegistryInput,
    HtmlCaptureSubjectId, HtmlCapturedIsValue, HtmlDirectionality, HtmlHostFactsRevision,
    HtmlLanguageDirectionFacts, HtmlLanguageDirectionLimits, HtmlScriptingStatus,
    HtmlSemanticCancellation, HtmlSemanticContext, HtmlSemanticContextInput, HtmlSemanticFact,
    HtmlSemanticLimits, HtmlSemanticQuery, HtmlSemanticRequest, HtmlSemanticSession,
    HtmlSemanticValue, HtmlSlotAssignedNodes,
};

fn complete_ordinary_slot(detached: bool) -> (HtmlCapture, HtmlCaptureSubjectId) {
    let mut builder = HtmlCaptureBuilder::new(
        HtmlCaptureLimits::new(3, 32, 1, 64, 4, 0, 0, 16_777_216),
        HtmlHostFactsRevision::fresh(),
    );
    let document = builder
        .add_document(
            HtmlDocumentMode::NoQuirks,
            HtmlScriptingStatus::Unavailable,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    let slot = builder
        .add_element(
            &document,
            QualifiedName::new(Namespace::html(), None, "slot"),
            OrderedAttributes::new([Attribute::new(
                QualifiedName::new(Namespace::none(), None, "dir"),
                "auto",
            )])
            .unwrap(),
            HtmlCapturedIsValue::Absent,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    builder.establish_no_shadow_root(&slot).unwrap();
    let text = builder.add_text(&document, "\u{05d0}".into()).unwrap();
    builder.establish_children(&slot, &[text]).unwrap();
    if detached {
        builder.establish_children(&document, &[]).unwrap();
        builder.establish_root(&slot).unwrap();
    } else {
        builder
            .establish_children(&document, std::slice::from_ref(&slot))
            .unwrap();
    }
    let capture = builder.finish().unwrap();
    let slot = capture.subject_id(&slot).unwrap();
    (capture, slot)
}

fn assert_empty_stored_assignment_and_ordinary_direction(
    capture: &HtmlCapture,
    slot: HtmlCaptureSubjectId,
) {
    let facts = HtmlLanguageDirectionFacts::new(
        capture,
        capture.host_facts_revision().clone(),
        vec![],
        vec![],
        vec![HtmlSlotAssignedNodes::new(slot.clone(), Some(vec![]))],
        HtmlLanguageDirectionLimits::new(1, 0, 0, 10000),
    )
    .expect("complete ordinary HTML Slot admits its established empty stored assignment");
    assert_eq!(facts.assignments()[0].slot(), &slot);
    assert_eq!(facts.assignments()[0].nodes(), Some([].as_slice()));
    let context = HtmlSemanticContext::new(
        capture,
        capture.host_facts_revision().clone(),
        HtmlSemanticContextInput::new(vec![]).with_language_direction(facts),
        0,
    )
    .unwrap();
    let report = HtmlSemanticSession::new(
        capture,
        &context,
        HtmlValueInterpreter::new(64),
        HtmlSemanticLimits::new(10000, 1, 32, 16, 10000, 64),
        HtmlSemanticCancellation::new(),
    )
    .unwrap()
    .query(&[HtmlSemanticRequest::new(
        slot,
        HtmlSemanticQuery::Directionality,
    )])
    .unwrap();
    assert!(matches!(
        report.results()[0].fact(),
        HtmlSemanticFact::Directionality(HtmlSemanticValue::Known(HtmlDirectionality::Rtl))
    ));
}

#[test]
fn known_empty_stored_assignment_is_admitted_for_complete_light_tree_slot() {
    let (capture, slot) = complete_ordinary_slot(false);
    assert_empty_stored_assignment_and_ordinary_direction(&capture, slot);
}

#[test]
fn known_empty_stored_assignment_is_admitted_for_complete_detached_slot() {
    let (capture, slot) = complete_ordinary_slot(true);
    assert_empty_stored_assignment_and_ordinary_direction(&capture, slot);
}
