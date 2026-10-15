use surgeist_html::{HtmlDocumentMode, Namespace, OrderedAttributes, QualifiedName};
use surgeist_html_semantics::{
    HtmlCapture, HtmlCaptureBuilder, HtmlCaptureLimits, HtmlCaptureRegistryInput,
    HtmlCaptureSubjectId, HtmlCapturedIsValue, HtmlHostFactsRevision, HtmlLanguageDirectionError,
    HtmlLanguageDirectionFacts, HtmlLanguageDirectionLimits, HtmlScriptingStatus,
    HtmlSlotAssignedNodes,
};

fn element(namespace: Namespace, name: &str) -> (HtmlCapture, HtmlCaptureSubjectId) {
    let mut builder = HtmlCaptureBuilder::new(
        HtmlCaptureLimits::new(2, 16, 0, 128, 2, 0, 0),
        HtmlHostFactsRevision::fresh(),
    );
    let document = builder
        .add_document(
            HtmlDocumentMode::NoQuirks,
            HtmlScriptingStatus::Unavailable,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    let element = builder
        .add_element(
            &document,
            QualifiedName::new(namespace, None, name),
            OrderedAttributes::default(),
            HtmlCapturedIsValue::Absent,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    builder.establish_children(&document, &[]).unwrap();
    builder.establish_root(&element).unwrap();
    builder.establish_children(&element, &[]).unwrap();
    let capture = builder.finish().unwrap();
    let subject = capture.subject_id(&element).unwrap();
    (capture, subject)
}

#[test]
fn empty_assignment_does_not_admit_other_elements_or_foreign_subjects() {
    for (namespace, name) in [
        (Namespace::html(), "div"),
        (Namespace::html(), "Slot"),
        (Namespace::from_uri("http://www.w3.org/2000/svg"), "slot"),
    ] {
        let (capture, subject) = element(namespace, name);
        let result = HtmlLanguageDirectionFacts::new(
            &capture,
            capture.host_facts_revision().clone(),
            vec![],
            vec![],
            vec![HtmlSlotAssignedNodes::new(subject.clone(), Some(vec![]))],
            HtmlLanguageDirectionLimits::new(1, 0, 0, 10000),
        );
        assert!(
            matches!(result, Err(HtmlLanguageDirectionError::InvalidSlot(id)) if id == subject)
        );
    }
    let (capture, _) = element(Namespace::html(), "slot");
    let (_, foreign) = element(Namespace::html(), "slot");
    assert!(matches!(
        HtmlLanguageDirectionFacts::new(
            &capture,
            capture.host_facts_revision().clone(),
            vec![],
            vec![],
            vec![HtmlSlotAssignedNodes::new(foreign, Some(vec![]))],
            HtmlLanguageDirectionLimits::new(1, 0, 0, 10000),
        ),
        Err(HtmlLanguageDirectionError::Capture(
            surgeist_html_semantics::HtmlCaptureError::ForeignCapture
        ))
    ));
}

#[test]
fn duplicate_empty_slot_facts_remain_rejected() {
    let (capture, slot) = element(Namespace::html(), "slot");
    assert!(matches!(
        HtmlLanguageDirectionFacts::new(
            &capture,
            capture.host_facts_revision().clone(),
            vec![],
            vec![],
            vec![
                HtmlSlotAssignedNodes::new(slot.clone(), Some(vec![])),
                HtmlSlotAssignedNodes::new(slot.clone(), Some(vec![])),
            ],
            HtmlLanguageDirectionLimits::new(2, 0, 0, 10000),
        ),
        Err(HtmlLanguageDirectionError::DuplicateSubject(id)) if id == slot
    ));
}

#[test]
fn empty_stored_vector_capacity_is_admitted_before_empty_topology_short_circuit() {
    let (capture, slot) = element(Namespace::html(), "slot");
    let nodes = Vec::with_capacity(1);
    assert!(matches!(
        HtmlLanguageDirectionFacts::new(
            &capture,
            capture.host_facts_revision().clone(),
            vec![],
            vec![],
            vec![HtmlSlotAssignedNodes::new(slot, Some(nodes))],
            HtmlLanguageDirectionLimits::new(1, 0, 0, 10000),
        ),
        Err(HtmlLanguageDirectionError::AssignedNodeCapacity {
            limit: 0,
            required: 1
        })
    ));
}

#[test]
fn unavailable_assignment_remains_distinct_from_known_empty() {
    let (capture, slot) = element(Namespace::html(), "slot");
    let facts = HtmlLanguageDirectionFacts::new(
        &capture,
        capture.host_facts_revision().clone(),
        vec![],
        vec![],
        vec![HtmlSlotAssignedNodes::new(slot, None)],
        HtmlLanguageDirectionLimits::new(1, 0, 0, 10000),
    )
    .unwrap();
    assert!(facts.assignments()[0].nodes().is_none());
}
