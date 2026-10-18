use super::*;
use std::cell::Cell;

#[derive(Clone, Copy, Default)]
struct Counts {
    calls: usize,
    visits: usize,
}
thread_local! { static ACTIVE: Cell<Option<Counts>> = const { Cell::new(None) }; }
pub(super) fn entered() {
    update(|counts| counts.calls += 1);
}
pub(super) fn visited() {
    update(|counts| counts.visits += 1);
}
fn update(change: impl FnOnce(&mut Counts)) {
    if !super::super::scale_metrics::ENABLED {
        return;
    }
    ACTIVE.with(|active| {
        if let Some(mut counts) = active.get() {
            change(&mut counts);
            active.set(Some(counts));
        }
    });
}
struct Probe;
impl Probe {
    fn start() -> Self {
        assert!(
            super::super::scale_metrics::ENABLED,
            "Details discovery requires enabled counters"
        );
        ACTIVE.with(|active| {
            assert!(active.get().is_none(), "Details discovery already active");
            active.set(Some(Counts::default()));
        });
        Self
    }
    fn finish(self) -> Counts {
        ACTIVE.with(|active| active.take().expect("active Details discovery"))
    }
}
impl Drop for Probe {
    fn drop(&mut self) {
        if super::super::scale_metrics::ENABLED {
            ACTIVE.with(|active| active.set(None));
        }
    }
}

fn case(unrelated: usize, matching_peer: bool) {
    let cancel = AtomicBool::new(false);
    let limits = DomLimits::new(256, 32768, 512, 0, 0, 8, 1_000_000)
        .unwrap()
        .with_details_toggle_intentions(16)
        .unwrap();
    let mut dom = Dom::new(limits).unwrap();
    let mut setup = DomBatch::new();
    let document = setup.create_document();
    let root = setup.create_element_ns(&document, crate::Namespace::html(), "div");
    for _ in 0..unrelated {
        let comment = setup.create_comment(&document, "unrelated");
        setup.insert_before(&comment, &root, None);
    }
    let peer = matching_peer.then(|| {
        let peer = setup.create_element_ns(&document, crate::Namespace::html(), "details");
        setup.set_attribute(&peer, "name", "exact");
        setup.set_attribute(&peer, "open", "");
        setup.insert_before(&peer, &root, None);
        peer
    });
    let target = setup.create_element_ns(&document, crate::Namespace::html(), "details");
    setup.set_attribute(&target, "name", "exact");
    setup.insert_before(&target, &root, None);
    let commit = dom.apply(setup, &cancel).unwrap();
    let target = commit.created(&target).unwrap().unwrap();
    let peer = peer
        .as_ref()
        .map(|peer| commit.created(peer).unwrap().unwrap());
    dom.take_details_toggle_intentions(16, &cancel).unwrap();
    let before = dom.snapshot();
    let probe = Probe::start();
    let mut edit = DomBatch::new();
    edit.set_attribute(target, "open", "");
    dom.apply(edit, &cancel).unwrap();
    let counts = probe.finish();
    use surgeist_html::HtmlDetailsState::{Closed, Open};
    assert_eq!(
        before.node(target).unwrap().as_details().unwrap().state(),
        Closed
    );
    assert_eq!(
        dom.snapshot()
            .node(target)
            .unwrap()
            .as_details()
            .unwrap()
            .state(),
        Open
    );
    let intentions = dom.take_details_toggle_intentions(16, &cancel).unwrap();
    let mut literal = vec![(target, Closed, Open)];
    if let Some(peer) = peer {
        assert_eq!(
            before.node(peer).unwrap().as_details().unwrap().state(),
            Open
        );
        assert_eq!(
            dom.snapshot()
                .node(peer)
                .unwrap()
                .as_details()
                .unwrap()
                .state(),
            Closed
        );
        literal.push((peer, Open, Closed));
    }
    assert_eq!(
        intentions
            .intentions()
            .iter()
            .map(|row| (row.target(), row.old_state(), row.new_state()))
            .collect::<Vec<_>>(),
        literal
    );
    assert_eq!(
        counts.calls, 1,
        "the original exclusivity operation actually ran"
    );
    assert!(
        counts.visits <= 8,
        "one exact Details group visited {} candidates with {unrelated} unrelated nodes",
        counts.visits
    );
}

#[test]
#[ignore = "requires SURGEIST_DOM_STORAGE_COUNTERS=1; original-path discovery regression"]
fn absent_details_name_peer_does_not_discover_unrelated_tree_nodes() {
    case(0, false);
    case(64, false);
}
#[test]
#[ignore = "requires SURGEIST_DOM_STORAGE_COUNTERS=1; original-path discovery regression"]
fn one_details_name_peer_does_not_discover_unrelated_tree_nodes() {
    case(0, true);
    case(64, true);
}
