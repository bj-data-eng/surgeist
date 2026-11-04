//! The work of finished-capture lookups grows linearly with the lookups made.
//!
//! The public `DomHtmlCapture::subject` and `DomHtmlCapture::node` are not
//! charged to any adapter work bound, so a private counter of the entries they
//! visit measures them. The registry and modal lookups charge their own steps
//! to the capture's `Control`, which is read directly. Each count is compared
//! across doublings of the input: a lookup that scans every entry grows 4x per
//! doubling when every entry is looked up, a constant-step lookup 2x.
use std::{cell::Cell, sync::atomic::AtomicBool};

use super::*;
use crate::{
    Dom, DomBatch, DomLimits, DomNameLimits, DomRegistryId, Namespace,
    hosted::{DomHostedLimits, DomHostedRecordLimits, DomHostedSource, DomHostedSourceId},
    store::registry::Bank,
};
use surgeist_html::values::{OrderedAttributes, QualifiedName};

/// A linear count grows 2x per doubling; the slack covers per-document
/// constants.
const GROWTH_BOUND: f64 = 2.2;

thread_local! {
    static VISITS: Cell<usize> = const { Cell::new(0) };
}

/// One entry visited by a finished-capture lookup.
pub(super) fn visit() {
    VISITS.with(|visits| visits.set(visits.get() + 1));
}

fn visits() -> usize {
    VISITS.with(Cell::get)
}

fn assert_linear(what: &str, sizes: &[usize], counts: &[usize]) {
    for pair in 0..sizes.len() - 1 {
        let growth = counts[pair + 1] as f64 / counts[pair] as f64;
        assert!(
            growth <= GROWTH_BOUND,
            "{what}: {} -> {} grew {growth:.2}x ({counts:?} for {sizes:?})",
            sizes[pair],
            sizes[pair + 1],
        );
    }
}

fn names() -> DomNameLimits {
    DomNameLimits::new(16_777_216, 16_384, 4_000_000, 16_777_216).unwrap()
}

#[test]
fn mapping_every_node_to_its_subject_and_back_takes_steps_linear_in_the_nodes() {
    let sizes = [256usize, 512, 1024, 2048];
    let mut counts = Vec::new();
    for n in sizes {
        let limits =
            DomLimits::new(3 * n + 64, 1 << 22, 8 * n + 256, 0, 0, 16, 131072, names()).unwrap();
        let mut dom = Dom::new(limits).unwrap();
        let mut batch = DomBatch::new();
        let document = batch.create_document();
        let parent = batch.create_element_ns(&document, Namespace::html(), "div");
        for _ in 0..n {
            let child = batch.create_element_ns(&document, Namespace::html(), "span");
            batch.insert_before(&child, &parent, None);
        }
        batch.insert_before(&parent, &document, None);
        let commit = dom.apply(batch, &AtomicBool::new(false)).unwrap();
        let document = commit.document(&document).unwrap();
        let snapshot = dom.snapshot();
        let captured = snapshot
            .html_capture(
                document,
                None,
                HtmlCaptureLimits::new(
                    4 * n + 16,
                    16 * n + 64,
                    4 * n + 16,
                    100 * n + 100,
                    4 * n + 16,
                    0,
                    0,
                    1 << 30,
                ),
                DomHtmlCaptureLimits::new(4 * n + 16, 1 << 40),
                &AtomicBool::new(false),
            )
            .unwrap();
        let nodes: Vec<NodeId> = captured.entries.iter().map(|entry| entry.node).collect();
        assert!(nodes.len() > n);
        VISITS.with(|visits| visits.set(0));
        for node in nodes {
            let subject = captured.subject(node).unwrap();
            assert_eq!(captured.node(&subject).unwrap(), node);
        }
        counts.push(visits());
    }
    assert_linear("subject and node lookups", &sizes, &counts);
}

/// A snapshot whose registry bank holds `count` actual registries, and the
/// canonical identity of each.
fn registry_snapshot(count: usize) -> (DomSnapshot, Vec<DomRegistryId>) {
    let cancel = AtomicBool::new(false);
    let dom = Dom::new(DomLimits::new(8, 4096, 32, 0, 0, 1, 65536, names()).unwrap()).unwrap();
    let mut snapshot = dom.snapshot();
    drop(dom);
    let owner = snapshot.owner;
    let limits = DomHostedLimits::new(
        DomHostedRecordLimits::new(0, 0, 0, 0, 0, 0, count + 8),
        1 << 30,
        1 << 30,
        1 << 40,
    );
    let source =
        DomHostedSource::new(DomHostedSourceId { owner, serial: 0 }, limits, &cancel).unwrap();
    let work = source.work(&cancel).unwrap();
    let mut bank = None;
    let mut ids = Vec::new();
    for _ in 0..count {
        let (next, id) = Bank::form(bank.as_deref(), owner, &source, &work, None, &cancel).unwrap();
        bank = Some(next);
        ids.push(id);
    }
    std::sync::Arc::get_mut(&mut snapshot.state)
        .expect("the only holder of this state")
        .registries = bank;
    (snapshot, ids)
}

#[test]
fn resolving_each_registry_twice_takes_steps_linear_in_the_registries() {
    let sizes = [128usize, 256, 512, 1024];
    let mut counts = Vec::new();
    for n in sizes {
        let (snapshot, ids) = registry_snapshot(n);
        let mut builder = HtmlCaptureBuilder::new(
            HtmlCaptureLimits::new(8, 8, 0, 4096, 8, n + 8, 0, 1 << 30),
            HtmlHostFactsRevision::fresh(),
        );
        let cancel = AtomicBool::new(false);
        let mut control = Control {
            limits: DomHtmlCaptureLimits::new(n + 8, 1 << 40),
            work: 0,
            cancellation: &cancel,
        };
        let mut registries = StagedRegistries::default();
        // The first pass stages every registry, the second finds each staged.
        for _ in 0..2 {
            for id in &ids {
                registry_input(
                    &mut builder,
                    &snapshot,
                    DomRegistryBinding::Registry(*id),
                    &mut registries,
                    &mut control,
                )
                .unwrap();
            }
        }
        assert_eq!(registries.items.len(), n);
        counts.push(control.work);
    }
    assert_linear("registry resolution", &sizes, &counts);
}

#[test]
fn finding_the_modal_blocker_of_each_document_takes_steps_linear_in_the_documents() {
    let sizes = [128usize, 256, 512, 1024];
    let mut counts = Vec::new();
    for n in sizes {
        let cancel = AtomicBool::new(false);
        let limits =
            DomLimits::new(2 * n + 8, 1 << 20, 8 * n + 64, 0, 0, 1, 65536, names()).unwrap();
        let mut dom = Dom::new(limits).unwrap();
        let mut batch = DomBatch::new();
        let documents: Vec<_> = (0..n).map(|_| batch.create_document()).collect();
        let dialogs: Vec<_> = documents
            .iter()
            .map(|document| {
                let dialog = batch.create_element_ns(document, Namespace::html(), "dialog");
                batch.insert_before(&dialog, document, None);
                dialog
            })
            .collect();
        let commit = dom.apply(batch, &cancel).unwrap();
        let documents: Vec<_> = documents
            .iter()
            .map(|document| commit.document(document).unwrap())
            .collect();
        let dialogs: Vec<_> = dialogs
            .iter()
            .map(|dialog| commit.created(dialog).unwrap().unwrap())
            .collect();
        for (document, dialog) in documents.iter().zip(&dialogs) {
            crate::store::set_modal_fixture_state(
                &mut dom,
                *document,
                Some(crate::dialog::DocumentInteractiveState::Known {
                    open_dialogs: Vec::new(),
                    top_layer: vec![*dialog],
                    pending_top_layer_removals: Vec::new(),
                }),
                &cancel,
            )
            .unwrap();
        }
        let source = dom.snapshot();
        // Every Document and then every dialog, as a capture holding the
        // dialog of each Document it covers.
        let mut builder = HtmlCaptureBuilder::new(
            HtmlCaptureLimits::new(
                4 * n + 16,
                16 * n + 64,
                4 * n + 16,
                100 * n + 100,
                4 * n + 16,
                0,
                0,
                1 << 30,
            ),
            HtmlHostFactsRevision::fresh(),
        );
        let mut staged = Vec::new();
        for document in &documents {
            let subject = builder
                .add_document(
                    source.document(*document).unwrap().metadata().mode(),
                    HtmlScriptingStatus::Disabled,
                    HtmlCaptureRegistryInput::Null,
                )
                .unwrap();
            staged.push((document.node(), subject));
        }
        for (document, dialog) in staged.clone().iter().zip(&dialogs) {
            let subject = builder
                .add_element(
                    &document.1,
                    QualifiedName::new(Namespace::html(), None, "dialog"),
                    OrderedAttributes::default(),
                    HtmlCapturedIsValue::Absent,
                    HtmlCaptureRegistryInput::Null,
                )
                .unwrap();
            staged.push((*dialog, subject));
        }
        let captured = builder.finish().unwrap();
        let entries: Vec<_> = staged
            .iter()
            .map(|(node, subject)| Entry {
                node: *node,
                subject: captured.subject_id(subject).unwrap(),
            })
            .collect();
        let mut control = Control {
            limits: DomHtmlCaptureLimits::new(2 * n + 8, 1 << 40),
            work: 0,
            cancellation: &cancel,
        };
        let input = modal::with_facts(
            HtmlSemanticContextInput::new(Vec::new()),
            &source,
            &captured,
            &mut nodes::LazyNodes::new(&entries, None),
            Vec::new(),
            surgeist_html_semantics::HtmlInertFactLimits::new(n + 8, 1 << 30),
            &mut control,
        )
        .unwrap();
        let context = HtmlSemanticContext::new(
            &captured,
            captured.host_facts_revision().clone(),
            input,
            2 * n + 8,
        )
        .unwrap();
        let facts = context.inert().unwrap();
        assert_eq!(facts.documents().len(), n);
        for (fact, (_, dialog)) in facts.documents().iter().zip(&staged[n..]) {
            assert_eq!(
                fact.blocking(),
                &surgeist_html_semantics::HtmlDocumentModalBlocking::BlockedBy(
                    captured.subject_id(dialog).unwrap()
                )
            );
        }
        counts.push(control.work);
    }
    assert_linear("modal blocker lookups", &sizes, &counts);
}

/// A snapshot holding a document with `n` elements, captured twice.
fn captured_twice(n: usize) -> (DomHtmlCapture, DomHtmlCapture) {
    let limits = DomLimits::new(n + 8, 1 << 20, 4 * n + 64, 0, 0, 1, 65536, names()).unwrap();
    let mut dom = Dom::new(limits).unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let parent = batch.create_element_ns(&document, Namespace::html(), "div");
    for _ in 0..n {
        let child = batch.create_element_ns(&document, Namespace::html(), "p");
        batch.insert_before(&child, &parent, None);
    }
    batch.insert_before(&parent, &document, None);
    let commit = dom.apply(batch, &AtomicBool::new(false)).unwrap();
    let document = commit.document(&document).unwrap();
    let snapshot = dom.snapshot();
    let capture = || {
        snapshot
            .html_capture(
                document,
                None,
                HtmlCaptureLimits::new(
                    4 * n + 16,
                    16 * n + 64,
                    4 * n + 16,
                    100 * n + 100,
                    4 * n + 16,
                    0,
                    0,
                    1 << 30,
                ),
                DomHtmlCaptureLimits::new(4 * n + 16, 1 << 40),
                &AtomicBool::new(false),
            )
            .unwrap()
    };
    (capture(), capture())
}

#[test]
fn a_subject_of_another_capture_is_refused_whatever_its_ordinal() {
    let (first, second) = captured_twice(8);
    let (small, _) = captured_twice(2);
    for (capture, foreign) in [(&first, &second), (&small, &first)] {
        for entry in &foreign.entries {
            assert!(matches!(
                capture.node(&entry.subject),
                Err(DomHtmlCaptureError::Capture(
                    surgeist_html_semantics::HtmlCaptureError::ForeignCapture
                ))
            ));
        }
    }
    // The smaller capture is refused for subjects beyond its own count too.
    assert!(first.entries.len() > small.entries.len());
    // Each capture still resolves its own subjects, which share ordinals.
    for (own, other) in first.entries.iter().zip(&second.entries) {
        assert_eq!(own.subject.ordinal(), other.subject.ordinal());
        assert_eq!(first.node(&own.subject).unwrap(), own.node);
        assert_eq!(second.node(&other.subject).unwrap(), other.node);
    }
}
