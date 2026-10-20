//! Real Source controller discovery with formation outside the work window.
//! Unrelated detached nodes keep the same node-document while ordinary trees
//! and semantic participant sets stay fixed. No traversal/work counter is faked.
use super::*;
use std::cell::Cell;

const UNRELATED: usize = 4096;
const DISCOVERY_WORK: u64 = 4096;

fn limits() -> DomLimits {
    DomLimits::new(
        16_384,
        1_000_000,
        32_768,
        0,
        0,
        8,
        64_000_000,
        crate::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
    )
    .unwrap()
}

fn policy() -> crate::DomSourceLimits {
    crate::DomSourceLimits::new(
        500_000_000,
        10_000_000,
        1_000_000_000,
        64,
        65_536,
        65_536,
        1,
    )
}

fn source_draft(dom: &Dom, cancel: &AtomicBool) -> Draft {
    let batch = DomBatch::new();
    Draft::new_source(dom, batch.owner.clone(), &batch.commands, policy(), cancel)
        .unwrap()
        .0
}

fn limit_controller_work(draft: &mut Draft) -> u64 {
    let before = draft.source_work.load(Ordering::Relaxed);
    let mut bounded = policy();
    bounded.work = before.checked_add(DISCOVERY_WORK).unwrap();
    draft.source_limits = Some(bounded);
    before
}

fn form_fixture(unrelated: usize, same_tree: bool) -> (Dom, NodeId, NodeId) {
    let cancel = AtomicBool::new(false);
    let mut dom = Dom::new(limits()).unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let source = batch.create_element_ns(&document, crate::Namespace::html(), "div");
    batch.set_attribute(&source, "id", "event");
    let subject = batch.create_element_ns(&document, crate::Namespace::html(), "input");
    batch.set_attribute(&subject, "form", "");
    batch.insert_before(&source, &document, None);
    if same_tree {
        batch.insert_before(&subject, &source, None);
    }
    let mut last_unrelated = None;
    for _ in 0..unrelated {
        // Still assigned to this exact Document, but not in its ordinary tree.
        last_unrelated = Some(batch.create_element_ns(&document, crate::Namespace::html(), "div"));
    }
    let commit = dom.apply(batch, &cancel).unwrap();
    let document = commit.document(&document).unwrap();
    let source = commit.created(&source).unwrap().unwrap();
    let subject = commit.created(&subject).unwrap().unwrap();
    if let Some(ticket) = last_unrelated {
        let node = commit.created(&ticket).unwrap().unwrap();
        assert_eq!(
            dom.snapshot().node(node).unwrap().owner_document(),
            Some(document)
        );
        assert!(dom.snapshot().node(node).unwrap().parent().is_none());
    }
    let snapshot = dom.snapshot();
    let input = snapshot.node(subject).unwrap().as_input().unwrap();
    assert!(input.form_owner().unwrap().is_none());
    assert!(!input.state().parser_inserted());
    (dom, source, subject)
}

fn form_recipients_stay_bounded(same_tree: bool) {
    for unrelated in [0, UNRELATED] {
        let cancel = AtomicBool::new(false);
        let (dom, source, subject) = form_fixture(unrelated, same_tree);
        let before = dom.snapshot();
        let mut draft = source_draft(&dom, &cancel);
        let source = draft.resolve(source.into()).unwrap();
        let subject = draft.resolve(subject.into()).unwrap();
        let checkpoint = draft
            .source_checkpoint(draft.source_other_bytes, draft.source_other_units, &cancel)
            .unwrap();
        let spent = limit_controller_work(&mut draft);
        // The connected query's first-ID search has at most these three nodes;
        // the detached query skips it. Its cost cannot grow with `unrelated`.
        let result = draft.id_recipients(source, same_tree, &cancel);
        let after = draft.source_work.load(Ordering::Relaxed);
        if result.is_ok() {
            assert!(draft.node(subject).unwrap().control_owner().is_none());
            assert!(
                !state(draft.node(subject).unwrap().kind())
                    .unwrap()
                    .0
                    .parser_inserted
            );
        }
        checkpoint.restore(&mut draft);
        assert_eq!(draft.source_work.load(Ordering::Relaxed), after);
        assert_eq!(dom.snapshot().stamp(), before.stamp());
        assert_eq!(
            result,
            Ok(()),
            "same_tree={same_tree}, unrelated={unrelated}, actual controller work={}",
            after - spent
        );
    }
}

#[test]
fn source_id_change_discovery_does_not_spend_work_on_unrelated_same_document_nodes() {
    form_recipients_stay_bounded(true);
}

#[test]
fn source_document_id_event_discovery_does_not_spend_work_on_unrelated_same_document_nodes() {
    form_recipients_stay_bounded(false);
}

fn radio_fixture(unrelated: usize, other_groups: usize) -> (Dom, NodeId, NodeId, Option<NodeId>) {
    let cancel = AtomicBool::new(false);
    let mut dom = Dom::new(limits()).unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let parent = batch.create_element_ns(&document, crate::Namespace::html(), "div");
    batch.insert_before(&parent, &document, None);
    let key = batch.create_element_ns(&document, crate::Namespace::html(), "input");
    let peer = batch.create_element_ns(&document, crate::Namespace::html(), "input");
    for radio in [&key, &peer] {
        batch.set_attribute(radio, "type", "radio");
        batch.set_attribute(radio, "name", "g");
        batch.insert_before(radio, &parent, None);
    }
    batch.set_input_checked(&peer, true);
    let mut other = None;
    for index in 0..other_groups {
        let radio = batch.create_element_ns(&document, crate::Namespace::html(), "input");
        batch.set_attribute(&radio, "type", "radio");
        batch.set_attribute(&radio, "name", format!("other-{index}"));
        batch.insert_before(&radio, &parent, None);
        other = Some(radio);
    }
    if let Some(radio) = &other {
        batch.set_input_checked(radio, true);
    }
    let mut last_unrelated = None;
    for _ in 0..unrelated {
        last_unrelated = Some(batch.create_element_ns(&document, crate::Namespace::html(), "div"));
    }
    let commit = dom.apply(batch, &cancel).unwrap();
    let document = commit.document(&document).unwrap();
    if let Some(ticket) = last_unrelated {
        let node = commit.created(&ticket).unwrap().unwrap();
        assert_eq!(
            dom.snapshot().node(node).unwrap().owner_document(),
            Some(document)
        );
        assert!(dom.snapshot().node(node).unwrap().parent().is_none());
    }
    (
        dom,
        commit.created(&key).unwrap().unwrap(),
        commit.created(&peer).unwrap().unwrap(),
        other
            .as_ref()
            .map(|radio| commit.created(radio).unwrap().unwrap()),
    )
}

thread_local! {
    static RADIO_DISCOVERY_ARMED: Cell<bool> = const { Cell::new(false) };
    static RADIO_DISCOVERY_START: Cell<Option<u64>> = const { Cell::new(None) };
}

struct RadioObservation;
impl RadioObservation {
    fn start() -> Self {
        assert!(!RADIO_DISCOVERY_ARMED.replace(true));
        RADIO_DISCOVERY_START.set(None);
        Self
    }
    fn finish(self) -> u64 {
        RADIO_DISCOVERY_START
            .get()
            .expect("actual radio discovery boundary reached")
    }
}
impl Drop for RadioObservation {
    fn drop(&mut self) {
        RADIO_DISCOVERY_ARMED.set(false);
        RADIO_DISCOVERY_START.set(None);
    }
}

/// Arm the real work policy at discovery, after the actual owned-name guard.
/// Old whole-root Source accounting during that guard is a separate #1261
/// concern and must not manufacture this participant-discovery RED.
pub(in crate::store) fn radio_discovery_started(draft: &mut Draft) {
    if RADIO_DISCOVERY_ARMED.replace(false) {
        RADIO_DISCOVERY_START.set(Some(limit_controller_work(draft)));
    }
}

fn radio_group_stays_bounded(unrelated: usize, other_groups: usize) {
    let cancel = AtomicBool::new(false);
    let (dom, key_id, peer_id, other_id) = radio_fixture(unrelated, other_groups);
    let before = dom.snapshot();
    let checked = |id| {
        before
            .node(id)
            .unwrap()
            .as_input()
            .unwrap()
            .state()
            .flags()
            .checkedness()
    };
    assert!(!checked(key_id));
    assert!(checked(peer_id));
    if let Some(other) = other_id {
        assert!(checked(other));
    }
    let mut draft = source_draft(&dom, &cancel);
    let key = draft.resolve(key_id.into()).unwrap();
    let peer = draft.resolve(peer_id.into()).unwrap();
    let other = other_id.map(|id| draft.resolve(id.into()).unwrap());
    let checkpoint = draft
        .source_checkpoint(draft.source_other_bytes, draft.source_other_units, &cancel)
        .unwrap();
    let before_alias = draft.source_work.load(Ordering::Relaxed);
    let observation = RadioObservation::start();
    let result = draft.set_input_checked(key, true, &cancel);
    let spent = observation.finish();
    eprintln!(
        "radio unrelated={unrelated} other_groups={other_groups}: prior name/accounting work={}, discovery budget={DISCOVERY_WORK}",
        spent - before_alias
    );
    let after = draft.source_work.load(Ordering::Relaxed);
    if result.is_ok() {
        assert!(draft.input_state(key).unwrap().flags.checkedness());
        assert!(!draft.input_state(peer).unwrap().flags.checkedness());
        if let Some(other) = other {
            assert!(draft.input_state(other).unwrap().flags.checkedness());
        }
    }
    checkpoint.restore(&mut draft);
    assert_eq!(draft.source_work.load(Ordering::Relaxed), after);
    assert!(!draft.input_state(key).unwrap().flags.checkedness());
    assert!(draft.input_state(peer).unwrap().flags.checkedness());
    if let Some(other) = other {
        assert!(draft.input_state(other).unwrap().flags.checkedness());
    }
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    assert_eq!(
        result,
        Ok(()),
        "unrelated={unrelated}, other_groups={other_groups}, actual controller work={}",
        after - spent
    );
}

#[test]
fn source_radio_discovery_does_not_spend_work_on_unrelated_same_document_nodes() {
    radio_group_stays_bounded(0, 0);
    radio_group_stays_bounded(UNRELATED, 0);
}

#[test]
fn source_radio_discovery_does_not_spend_work_on_other_named_groups() {
    radio_group_stays_bounded(0, 0);
    radio_group_stays_bounded(0, 1024);
}
