//! Owning qualification fixtures; nonempty raw state is privately constructed,
//! without claiming public modal/top-layer execution can produce that state.
use super::*;
use crate::{Dom, DomBatch, DomLimits, Namespace};
use std::sync::Arc;
use surgeist_html_semantics::{
    HtmlInertFactError, HtmlSemanticFact, HtmlSemanticMissing, HtmlSemanticQuery, HtmlSemanticValue,
};

fn fixture() -> (DomSnapshot, DocumentId, NodeId) {
    let cancel = AtomicBool::new(false);
    let mut dom = Dom::new(DomLimits::new(32, 4096, 32, 32, 32, 4, 65536).unwrap()).unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let dialog = batch.create_element_ns(&document, Namespace::html(), "dialog");
    batch.insert_before(&dialog, &document, None);
    let commit = dom.apply(batch, &cancel).unwrap();
    (
        dom.snapshot(),
        commit.document(&document).unwrap(),
        commit.created(&dialog).unwrap().unwrap(),
    )
}

fn capture(source: &DomSnapshot, document: DocumentId) -> DomHtmlCapture {
    source
        .html_capture(
            document,
            None,
            HtmlCaptureLimits::new(64, 1024, 128, 16384, 64, 0, 0),
            DomHtmlCaptureLimits::new(64, 1000000),
            &AtomicBool::new(false),
        )
        .unwrap()
}

fn slot_limits() -> crate::DomSlotSemanticLimits {
    crate::DomSlotSemanticLimits::new(
        crate::DomSlotQueryLimits::new(64, 128).unwrap(),
        DomHtmlCaptureLimits::new(64, 1000000),
        surgeist_html_semantics::HtmlLanguageDirectionLimits::new(128, 128, 16384, 1000000),
        HtmlInertFactLimits::new(128, 1000000),
    )
}

fn inertness(capture: &DomHtmlCapture, node: NodeId) -> HtmlSemanticFact {
    capture
        .query(
            &[HtmlSemanticRequest::new(
                capture.subject(node).unwrap(),
                HtmlSemanticQuery::Inertness,
            )],
            HtmlValueInterpreter::new(8192),
            HtmlSemanticLimits::new(100000, 16, 1024, 64, 1000000, 1024),
            HtmlSemanticCancellation::new(),
        )
        .unwrap()
        .results()[0]
        .fact()
        .clone()
}

#[test]
fn unavailable_history_stays_missing_and_actual_raw_dialog_blocks_in_both_contexts() {
    for disposition in 0..3 {
        let (mut source, document, dialog) = fixture();
        // Mutate only an exclusively owned private fixture: unavailable history
        // and a positive raw blocker, without a public setter or modal success.
        let node = Arc::get_mut(&mut source.state).unwrap().slots[document.node().slot]
            .node
            .as_mut()
            .unwrap();
        node.interactive_document = match disposition {
            0 => Some(DocumentInteractiveState::Unavailable),
            1 => Some(DocumentInteractiveState::Known {
                open_dialogs: Vec::new(),
                top_layer: vec![dialog],
                pending_top_layer_removals: Vec::new(),
            }),
            _ => None,
        };
        let initial = capture(&source, document);
        let supplemented = capture(&source, document)
            .with_slot_semantics(slot_limits(), &AtomicBool::new(false))
            .unwrap();
        for captured in [initial, supplemented] {
            if disposition == 1 {
                assert_eq!(
                    inertness(&captured, document.node()),
                    HtmlSemanticFact::Inertness(HtmlSemanticValue::Known(true))
                );
                assert_eq!(
                    captured.context.inert().unwrap().documents()[0].blocking(),
                    &HtmlDocumentModalBlocking::BlockedBy(captured.subject(dialog).unwrap())
                );
            } else {
                assert_eq!(
                    inertness(&captured, document.node()),
                    HtmlSemanticFact::Inertness(HtmlSemanticValue::Missing(
                        HtmlSemanticMissing::ModalBlocking(
                            captured.subject(document.node()).unwrap()
                        )
                    ))
                );
                assert_eq!(
                    captured.context.inert().unwrap().documents()[0].blocking(),
                    &HtmlDocumentModalBlocking::Unavailable
                );
            }
        }
    }
}

#[test]
fn partial_document_coverage_uses_actual_lists_and_original_qualification() {
    let cancel = AtomicBool::new(false);
    let mut dom = Dom::new(DomLimits::new(32, 4096, 32, 32, 32, 4, 65536).unwrap()).unwrap();
    let mut batch = DomBatch::new();
    let first = batch.create_document();
    let second = batch.create_document();
    let omitted = batch.create_element_ns(&second, Namespace::html(), "dialog");
    batch.insert_before(&omitted, &second, None);
    let commit = dom.apply(batch, &cancel).unwrap();
    let documents = [
        commit.document(&first).unwrap(),
        commit.document(&second).unwrap(),
    ];
    let omitted = commit.created(&omitted).unwrap().unwrap();
    let mut source = dom.snapshot();
    drop(dom);
    Arc::get_mut(&mut source.state).unwrap().slots[documents[1].node().slot]
        .node
        .as_mut()
        .unwrap()
        .interactive_document = Some(DocumentInteractiveState::Known {
        open_dialogs: Vec::new(),
        top_layer: vec![omitted],
        pending_top_layer_removals: Vec::new(),
    });
    // Deliberately partial, but real, Document-only producer coverage. Omitting
    // the second Document's member does not qualify its raw list as empty.
    let mut builder = HtmlCaptureBuilder::new(
        HtmlCaptureLimits::new(8, 8, 0, 4096, 8, 0, 0),
        HtmlHostFactsRevision::fresh(),
    );
    let mut staged = Vec::new();
    for document in documents {
        staged.push((
            document.node(),
            builder
                .add_document(
                    source.document(document).unwrap().metadata().mode(),
                    HtmlScriptingStatus::Disabled,
                    HtmlCaptureRegistryInput::Null,
                )
                .unwrap(),
        ));
    }
    let captured = builder.finish().unwrap();
    let entries: Vec<_> = staged
        .iter()
        .map(|(node, subject)| Entry {
            node: *node,
            subject: captured.subject_id(subject).unwrap(),
        })
        .collect();
    let form = || {
        let input = with_facts(
            HtmlSemanticContextInput::new(Vec::new()),
            &source,
            &captured,
            &entries,
            Vec::new(),
            HtmlInertFactLimits::new(2, 3),
            &mut Control {
                limits: DomHtmlCaptureLimits::new(2, 1000),
                work: 0,
                cancellation: &cancel,
            },
        )
        .unwrap();
        HtmlSemanticContext::new(&captured, captured.host_facts_revision().clone(), input, 2)
            .unwrap()
    };
    let first = form();
    let facts = first.inert().unwrap();
    assert!(facts.nodes().is_empty());
    assert_eq!(facts.documents().len(), 2);
    assert_eq!(facts.documents()[0].document(), &entries[0].subject);
    assert_eq!(
        facts.documents()[0].blocking(),
        &HtmlDocumentModalBlocking::Unblocked
    );
    assert_eq!(facts.documents()[1].document(), &entries[1].subject);
    assert_eq!(
        facts.documents()[1].blocking(),
        &HtmlDocumentModalBlocking::Unavailable
    );
    assert_eq!(facts.capture(), captured.identity());
    assert_eq!(facts.host_revision(), captured.host_facts_revision());
    assert_eq!(facts.flat_revision().revision(), source.revision().get());
    assert_eq!(facts.flat_revision().configuration(), 0);
    let second = form();
    assert_ne!(
        facts.flat_revision().producer(),
        second.inert().unwrap().flat_revision().producer()
    );
}

#[test]
fn partial_modal_rows_obey_adapter_bounds_producer_capacity_work_and_cancellation() {
    let (source, document, _) = fixture();
    let captured = capture(&source, document);
    let run = |subjects, adapter_work, records, producer_work, cancel: &AtomicBool| {
        with_facts(
            HtmlSemanticContextInput::new(Vec::new()),
            &source,
            captured.capture(),
            &captured.entries,
            Vec::new(),
            HtmlInertFactLimits::new(records, producer_work),
            &mut Control {
                limits: DomHtmlCaptureLimits::new(subjects, adapter_work),
                work: 0,
                cancellation: cancel,
            },
        )
    };
    let cancel = AtomicBool::new(false);
    assert!(matches!(
        run(0, 1000, 1, 1, &cancel),
        Err(DomHtmlCaptureError::ResourceLimit {
            resource: DomHtmlCaptureResource::Subjects,
            limit: 0
        })
    ));
    assert!(matches!(
        run(2, 0, 1, 1, &cancel),
        Err(DomHtmlCaptureError::ResourceLimit {
            resource: DomHtmlCaptureResource::Work,
            limit: 0
        })
    ));
    assert!(matches!(
        run(2, 1000, 0, 1, &cancel),
        Err(DomHtmlCaptureError::Inert(
            HtmlInertFactError::RecordCapacity {
                limit: 0,
                required: 1
            }
        ))
    ));
    assert!(matches!(
        run(2, 1000, 1, 0, &cancel),
        Err(DomHtmlCaptureError::Inert(
            HtmlInertFactError::ValidationWork {
                limit: 0,
                required: 1
            }
        ))
    ));
    assert!(matches!(
        run(2, 1000, 1, 1, &AtomicBool::new(true)),
        Err(DomHtmlCaptureError::Cancelled)
    ));
    let input = run(2, 1000, 1, 1, &cancel).unwrap();
    let context = HtmlSemanticContext::new(
        captured.capture(),
        captured.capture().host_facts_revision().clone(),
        input,
        2,
    )
    .unwrap();
    assert!(context.inert().unwrap().nodes().is_empty());
    assert_eq!(context.inert().unwrap().documents().len(), 1);
    assert_eq!(source.document(document).unwrap().stamp(), captured.basis());
}
