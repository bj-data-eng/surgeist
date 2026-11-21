//! HTML-owned pruning and disabled-state observations through public operations.
use std::sync::{Arc, atomic::AtomicBool};
use surgeist_dom::{
    Dom, DomBatch, DomError, DomSnapshot,
    edit::CreatedNode,
    identity::NodeId,
    limits::DomLimits,
    source::{DomSourceEdit, DomSourceLimits, DomSourceProgress, DomSourceRequest},
    style::DomStyleInlineCspObservation,
};
use surgeist_html::values::{DocumentString, Namespace};

fn limits() -> DomLimits {
    DomLimits::new(
        256,
        32768,
        1024,
        256,
        128,
        16,
        1_000_000,
        surgeist_dom::limits::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
    )
    .unwrap()
    .with_select_processing(128, 4_000_000, 10_000_000)
    .unwrap()
}
fn option(
    batch: &mut DomBatch,
    document: &CreatedNode,
    parent: &CreatedNode,
    value: &str,
) -> CreatedNode {
    let option = batch.create_element_ns(document, Namespace::html(), "option");
    batch.set_attribute(&option, "value", value);
    batch.insert_before(&option, parent, None);
    option
}
fn selected(dom: &Dom, option: NodeId) -> bool {
    dom.snapshot()
        .node(option)
        .unwrap()
        .as_option()
        .unwrap()
        .state()
        .flags()
        .selectedness()
}

fn selected_index(snapshot: &DomSnapshot, node: NodeId) -> Result<Option<usize>, DomError> {
    snapshot
        .node(node)?
        .as_select()
        .unwrap()
        .selected_index_with(|| Ok(()))
}

fn option_index(snapshot: &DomSnapshot, node: NodeId) -> Result<usize, DomError> {
    snapshot
        .node(node)?
        .as_option()
        .unwrap()
        .index_with(|| Ok(()))
}

fn select_value(snapshot: &DomSnapshot, node: NodeId) -> Vec<u16> {
    let mut value = Vec::new();
    snapshot
        .node(node)
        .unwrap()
        .as_select()
        .unwrap()
        .visit_value_with(
            || Ok::<_, DomError>(()),
            |_| Ok(()),
            |units| {
                value.extend_from_slice(units);
                Ok(())
            },
        )
        .unwrap();
    value
}

#[test]
fn option_membership_keeps_html_pruning_and_foreign_wrappers_distinct() {
    let cancel = AtomicBool::new(false);
    let mut dom = Dom::new(limits()).unwrap();
    let mut setup = DomBatch::new();
    let document = setup.create_document();
    let select = setup.create_element_ns(&document, Namespace::html(), "select");
    setup.set_attribute(&select, "multiple", "");
    let first = option(&mut setup, &document, &select, "first");
    let pruned_inside_option = option(&mut setup, &document, &first, "inside-option");
    let foreign = setup.create_element_ns(&document, Namespace::from_uri("urn:foreign"), "select");
    setup.insert_before(&foreign, &select, None);
    let second = option(&mut setup, &document, &foreign, "second");
    let group = setup.create_element_ns(&document, Namespace::html(), "optgroup");
    setup.insert_before(&group, &select, None);
    let third = option(&mut setup, &document, &group, "third");
    let nested_group = setup.create_element_ns(&document, Namespace::html(), "optgroup");
    setup.insert_before(&nested_group, &group, None);
    let pruned_nested_group = option(&mut setup, &document, &nested_group, "nested-group");
    let mut pruned = vec![pruned_inside_option, pruned_nested_group];
    for tag in ["hr", "datalist", "select"] {
        let barrier = setup.create_element_ns(&document, Namespace::html(), tag);
        setup.insert_before(&barrier, &select, None);
        pruned.push(option(&mut setup, &document, &barrier, tag));
    }
    let commit = dom.apply(setup, &cancel).unwrap();
    let id = |ticket: &CreatedNode| commit.created(ticket).unwrap().unwrap();
    let select = id(&select);
    let actual = [id(&first), id(&second), id(&third)];
    let pruned = pruned.iter().map(id).collect::<Vec<_>>();
    let original_pruned = pruned
        .iter()
        .map(|node| selected(&dom, *node))
        .collect::<Vec<_>>();
    let before = dom.snapshot();
    assert_eq!(
        actual.map(|node| option_index(&before, node)),
        [Ok(0), Ok(1), Ok(2)]
    );
    for &node in &pruned {
        assert_eq!(option_index(&before, node), Ok(0));
    }
    for ordinal in 0..3 {
        let mut edit = DomBatch::new();
        edit.set_select_selected_index(select, ordinal as i64);
        dom.apply(edit, &cancel).unwrap();
        assert_eq!(selected_index(&dom.snapshot(), select), Ok(Some(ordinal)));
        assert_eq!(
            actual.map(|node| selected(&dom, node)),
            [ordinal == 0, ordinal == 1, ordinal == 2]
        );
        assert_eq!(
            pruned
                .iter()
                .map(|node| selected(&dom, *node))
                .collect::<Vec<_>>(),
            original_pruned
        );
    }
    let mut edit = DomBatch::new();
    edit.set_select_selected_index(select, 3);
    dom.apply(edit, &cancel).unwrap();
    assert_eq!(selected_index(&dom.snapshot(), select), Ok(None));
    assert_eq!(actual.map(|node| selected(&dom, node)), [false; 3]);
}

#[derive(Debug, PartialEq)]
enum ReadError {
    Dom(DomError),
    Refused,
}

impl From<DomError> for ReadError {
    fn from(error: DomError) -> Self {
        Self::Dom(error)
    }
}

fn measure<T: std::fmt::Debug + PartialEq>(
    expected: T,
    read: impl Fn(&mut dyn FnMut() -> Result<(), ReadError>) -> Result<T, ReadError>,
) -> usize {
    let mut steps = 0;
    let outcome = read(&mut || {
        steps += 1;
        Ok(())
    });
    assert_eq!(outcome.as_ref(), Ok(&expected));
    assert!(steps > 0);
    for allowance in [steps - 1, steps] {
        let mut used = 0;
        let outcome = read(&mut || {
            if used == allowance {
                return Err(ReadError::Refused);
            }
            used += 1;
            Ok(())
        });
        assert_eq!(used, allowance);
        if allowance == steps {
            assert_eq!(outcome.as_ref(), Ok(&expected));
        } else {
            assert_eq!(outcome, Err(ReadError::Refused));
        }
    }
    steps
}

#[test]
fn index_reads_admit_reached_members_and_ignore_unrelated_subtrees() {
    let cancel = AtomicBool::new(false);
    let mut dom = Dom::new(limits()).unwrap();
    let mut setup = DomBatch::new();
    let document = setup.create_document();
    let select = setup.create_element_ns(&document, Namespace::html(), "select");
    setup.set_attribute(&select, "multiple", "");
    let members =
        std::array::from_fn::<_, 17, _>(|_| option(&mut setup, &document, &select, "member"));
    setup.set_option_selected(&members[0], true);
    setup.set_option_selected(&members[16], true);
    let unrelated = (0..70)
        .map(|_| setup.create_element_ns(&document, Namespace::html(), "div"))
        .collect::<Vec<_>>();
    let done = dom.apply(setup, &cancel).unwrap();
    let id = |ticket: &CreatedNode| done.created(ticket).unwrap().unwrap();
    let select = id(&select);
    let [first, .., last] = members.each_ref().map(id);
    let work = |snapshot: &DomSnapshot| {
        let select = snapshot.node(select).unwrap().as_select().unwrap();
        let first = snapshot.node(first).unwrap().as_option().unwrap();
        let last = snapshot.node(last).unwrap().as_option().unwrap();
        [
            measure(Some(0), |step| select.selected_index_with(step)),
            measure(0, |step| first.index_with(step)),
            measure(16, |step| last.index_with(step)),
        ]
    };
    let initial = dom.snapshot();
    let before = work(&initial);
    assert!(before[1] < before[2]);

    let mut edit = DomBatch::new();
    // These existing nodes change real topology without growing the lookup banks.
    // Inserting before the first member exposes a subtree-walking read immediately.
    for node in &unrelated {
        edit.insert_before(id(node), select, Some(first.into()));
    }
    dom.apply(edit, &cancel).unwrap();
    let current = dom.snapshot();
    assert_ne!(current.stamp(), initial.stamp());
    assert_eq!(work(&current), before);
    assert_eq!(dom.snapshot().stamp(), current.stamp());

    let mut edit = DomBatch::new();
    edit.set_option_selected(first, false);
    dom.apply(edit, &cancel).unwrap();
    let current = dom.snapshot();
    let select_view = current.node(select).unwrap().as_select().unwrap();
    assert!(measure(Some(16), |step| select_view.selected_index_with(step)) > before[0]);
    assert_eq!(selected_index(&initial, select), Ok(Some(0)));
    assert_eq!(dom.snapshot().stamp(), current.stamp());

    let mut edit = DomBatch::new();
    edit.set_option_selected(last, false);
    dom.apply(edit, &cancel).unwrap();
    let current = dom.snapshot();
    let select_view = current.node(select).unwrap().as_select().unwrap();
    measure(None, |step| select_view.selected_index_with(step));
}

#[test]
fn index_reads_follow_reorder_adoption_detach_and_retirement() {
    let cancel = AtomicBool::new(false);
    let mut dom = Dom::new(limits()).unwrap();
    let mut setup = DomBatch::new();
    let document = setup.create_document();
    let destination = setup.create_document();
    let select = setup.create_element_ns(&document, Namespace::html(), "select");
    let other = setup.create_element_ns(&destination, Namespace::html(), "select");
    setup.set_attribute(&select, "multiple", "");
    setup.set_attribute(&other, "multiple", "");
    let first = option(&mut setup, &document, &select, "first");
    let second = option(&mut setup, &document, &select, "second");
    let other_first = option(&mut setup, &destination, &other, "other");
    setup.set_option_selected(&second, true);
    let done = dom.apply(setup, &cancel).unwrap();
    let id = |ticket: &CreatedNode| done.created(ticket).unwrap().unwrap();
    let [select, other, first, second, other_first] =
        [&select, &other, &first, &second, &other_first].map(id);
    let destination = done.document(&destination).unwrap();
    let original = dom.snapshot();
    assert_eq!(selected_index(&original, select), Ok(Some(1)));
    assert_eq!(option_index(&original, second), Ok(1));

    let mut edit = DomBatch::new();
    edit.insert_before(second, select, Some(first.into()));
    dom.apply(edit, &cancel).unwrap();
    let reordered = dom.snapshot();
    assert_eq!(selected_index(&reordered, select), Ok(Some(0)));
    assert_eq!(option_index(&reordered, first), Ok(1));
    assert_eq!(option_index(&reordered, second), Ok(0));

    let mut edit = DomBatch::new();
    edit.adopt(second, destination);
    dom.apply(edit, &cancel).unwrap();
    let detached = dom.snapshot();
    assert_eq!(selected_index(&detached, select), Ok(None));
    assert_eq!(option_index(&detached, second), Ok(0));
    assert_eq!(
        detached.node(second).unwrap().owner_document(),
        Some(destination)
    );

    let mut edit = DomBatch::new();
    edit.insert_before(second, other, None);
    dom.apply(edit, &cancel).unwrap();
    let adopted = dom.snapshot();
    assert_eq!(selected_index(&adopted, other), Ok(Some(1)));
    assert_eq!(option_index(&adopted, second), Ok(1));
    assert_eq!(option_index(&adopted, other_first), Ok(0));

    let mut edit = DomBatch::new();
    edit.detach(second);
    edit.destroy(second);
    dom.apply(edit, &cancel).unwrap();
    let current = dom.snapshot();
    assert_eq!(option_index(&current, second), Err(DomError::StaleNode));
    assert_eq!(selected_index(&current, other), Ok(None));
    assert_eq!(option_index(&adopted, second), Ok(1));
    assert_eq!(option_index(&original, second), Ok(1));
}

#[test]
fn index_reads_share_the_published_source_prefix_with_node_records() {
    let cancel = AtomicBool::new(false);
    let mut dom = Dom::new(limits()).unwrap();
    let mut setup = DomBatch::new();
    let document = setup.create_document();
    let select = setup.create_element_ns(&document, Namespace::html(), "select");
    setup.set_attribute(&select, "multiple", "");
    option(&mut setup, &document, &select, "first");
    let group = setup.create_element_ns(&document, Namespace::html(), "optgroup");
    let chosen = option(&mut setup, &document, &group, "chosen");
    setup.set_option_selected(&chosen, true);
    let style = setup.create_element_ns(&document, Namespace::html(), "style");
    setup.insert_before(&style, &group, None);
    setup.insert_before(&select, &document, None);
    let done = dom.apply(setup, &cancel).unwrap();
    let id = |ticket: &CreatedNode| done.created(ticket).unwrap().unwrap();
    let [select, group, chosen, style] = [&select, &group, &chosen, &style].map(id);
    let original = dom.snapshot();
    let read = |snapshot: &DomSnapshot| {
        (
            selected_index(snapshot, select),
            option_index(snapshot, chosen),
        )
    };
    assert_eq!(read(&original), (Ok(None), Ok(0)));
    assert!(select_value(&original, select).is_empty());
    let mut lease = dom
        .begin_source_edit(
            DomSourceEdit::insert_before(group, select, None),
            DomSourceLimits::new(16_000_000, 1_000_000, 10_000_000, 128, 4096, 4096, 1),
            &cancel,
        )
        .unwrap();
    let DomSourceProgress::Request(DomSourceRequest::Style(request)) =
        lease.advance(&cancel).unwrap()
    else {
        panic!("connected Style requires CSP evidence");
    };
    assert_eq!(request.source().element(), style);
    let parked = lease.snapshot();
    assert_eq!(
        parked.incomplete_source_operation(),
        Some(lease.operation())
    );
    assert_eq!(parked.node(group).unwrap().parent(), Some(select));
    let chosen_view = parked.node(chosen).unwrap().as_option().unwrap();
    assert!(chosen_view.state().flags().selectedness());
    assert_eq!(
        chosen_view.cached_select(),
        Err(DomError::IncompleteSourceOperation)
    );
    let published = (Ok(Some(1)), Ok(1));
    assert_eq!(read(&parked), published);
    assert_eq!(read(&original), (Ok(None), Ok(0)));
    let selected_value = [99, 104, 111, 115, 101, 110];
    assert_eq!(select_value(&parked, select), selected_value);
    assert!(select_value(&original, select).is_empty());
    lease
        .accept_inline_csp(
            &request,
            &DomStyleInlineCspObservation::evaluated(
                request.source(),
                Arc::new(vec!["style-src 'none'".into()]),
                1,
                false,
            ),
            &cancel,
        )
        .unwrap();
    assert!(matches!(
        lease.advance(&cancel).unwrap(),
        DomSourceProgress::Complete(_)
    ));
    drop(lease);
    assert!(dom.pending_source_edit().is_none());
    let complete = dom.snapshot();
    assert_eq!(complete.incomplete_source_operation(), None);
    assert_eq!(read(&complete), published);
    assert_eq!(read(&parked), published);
    assert_eq!(select_value(&complete, select), selected_value);
    assert_eq!(select_value(&parked, select), selected_value);

    let mut edit = DomBatch::new();
    edit.set_attribute(chosen, "value", "changed");
    dom.apply(edit, &cancel).unwrap();
    assert_eq!(
        select_value(&dom.snapshot(), select),
        [99, 104, 97, 110, 103, 101, 100]
    );
    assert_eq!(select_value(&parked, select), selected_value);
}

#[test]
fn moved_option_membership_and_cached_select_restore_after_a_late_failure() {
    let cancel = AtomicBool::new(false);
    let mut dom = Dom::new(limits()).unwrap();
    let mut setup = DomBatch::new();
    let document = setup.create_document();
    let source = setup.create_element_ns(&document, Namespace::html(), "select");
    let destination = setup.create_element_ns(&document, Namespace::html(), "select");
    setup.set_attribute(&source, "multiple", "");
    setup.set_attribute(&destination, "multiple", "");
    let moved = option(&mut setup, &document, &source, "moved");
    let retained = option(&mut setup, &document, &destination, "retained");
    let commit = dom.apply(setup, &cancel).unwrap();
    let id = |ticket: &CreatedNode| commit.created(ticket).unwrap().unwrap();
    let source = id(&source);
    let destination = id(&destination);
    let moved = id(&moved);
    let retained = id(&retained);
    let mut edit = DomBatch::new();
    edit.set_select_selected_index(source, 0);
    edit.set_select_selected_index(destination, 0);
    dom.apply(edit, &cancel).unwrap();
    let before = dom.snapshot();
    let mut refused = DomBatch::new();
    refused.insert_before(moved, destination, None);
    refused.insert_before(source, source, None);
    assert_eq!(
        dom.apply(refused, &cancel).unwrap_err(),
        DomError::HierarchyRequest
    );
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    assert_eq!(
        dom.snapshot()
            .node(moved)
            .unwrap()
            .as_option()
            .unwrap()
            .cached_select()
            .unwrap(),
        Some(source)
    );
    let mut edit = DomBatch::new();
    edit.set_select_selected_index(source, -1);
    dom.apply(edit, &cancel).unwrap();
    assert!(!selected(&dom, moved));
    assert!(selected(&dom, retained));
    let mut edit = DomBatch::new();
    edit.insert_before(moved, destination, None);
    edit.set_select_selected_index(destination, 1);
    dom.apply(edit, &cancel).unwrap();
    assert!(selected(&dom, moved));
    assert!(!selected(&dom, retained));
    assert_eq!(
        dom.snapshot()
            .node(moved)
            .unwrap()
            .as_option()
            .unwrap()
            .cached_select()
            .unwrap(),
        Some(destination)
    );
    assert_eq!(
        before
            .node(moved)
            .unwrap()
            .as_option()
            .unwrap()
            .cached_select()
            .unwrap(),
        Some(source)
    );
}

#[test]
fn selectedcontent_scope_uses_actual_disabled_hooks_and_move_rollback() {
    let cancel = AtomicBool::new(false);
    let mut dom = Dom::new(limits()).unwrap();
    let mut setup = DomBatch::new();
    let document = setup.create_document();
    let outer = setup.create_element_ns(&document, Namespace::html(), "select");
    let chosen = option(&mut setup, &document, &outer, "outer");
    let text = setup.create_text(&document, "outer");
    setup.insert_before(&text, &chosen, None);
    let datalist = setup.create_element_ns(&document, Namespace::html(), "datalist");
    setup.insert_before(&datalist, &outer, None);
    let enabled = setup.create_element_ns(&document, Namespace::html(), "selectedcontent");
    setup.insert_before(&enabled, &datalist, None);
    let nested = setup.create_element_ns(&document, Namespace::html(), "select");
    let nested_option = option(&mut setup, &document, &nested, "inner");
    let text = setup.create_text(&document, "inner");
    setup.insert_before(&text, &nested_option, None);
    let nested_target = setup.create_element_ns(&document, Namespace::html(), "selectedcontent");
    let sentinel = setup.create_text(&document, "not refreshed while disabled");
    setup.insert_before(&sentinel, &nested_target, None);
    setup.insert_before(&nested_target, &nested, None);
    setup.insert_before(&nested, &outer, None);
    let destination = setup.create_element_ns(&document, Namespace::html(), "div");
    let commit = dom.apply(setup, &cancel).unwrap();
    let id = |ticket: &CreatedNode| commit.created(ticket).unwrap().unwrap();
    let outer = id(&outer);
    let nested = id(&nested);
    let enabled = id(&enabled);
    let nested_target = id(&nested_target);
    let sentinel = id(&sentinel);
    let destination = id(&destination);
    assert!(
        dom.snapshot()
            .node(nested_target)
            .unwrap()
            .as_selectedcontent()
            .unwrap()
            .disabled()
    );
    let text_at = |dom: &Dom, target| {
        let snapshot = dom.snapshot();
        let children = snapshot.node(target).unwrap().children();
        assert_eq!(children.len(), 1);
        snapshot.node(children[0]).unwrap().data().unwrap().clone()
    };
    let mut edit = DomBatch::new();
    edit.set_select_selected_index(outer, 0);
    dom.apply(edit, &cancel).unwrap();
    assert_eq!(text_at(&dom, enabled), DocumentString::from("outer"));
    assert_eq!(
        dom.snapshot().node(nested_target).unwrap().children(),
        &[sentinel]
    );
    let old = dom.snapshot();
    let mut edit = DomBatch::new();
    edit.insert_before(nested, destination, None);
    edit.set_select_selected_index(nested, 0);
    dom.apply(edit, &cancel).unwrap();
    assert!(
        !dom.snapshot()
            .node(nested_target)
            .unwrap()
            .as_selectedcontent()
            .unwrap()
            .disabled()
    );
    assert_eq!(text_at(&dom, nested_target), DocumentString::from("inner"));
    assert!(
        old.node(nested_target)
            .unwrap()
            .as_selectedcontent()
            .unwrap()
            .disabled()
    );
    assert_eq!(old.node(nested_target).unwrap().children(), &[sentinel]);
    let before = dom.snapshot();
    let mut refused = DomBatch::new();
    refused.insert_before(nested, outer, None);
    refused.insert_before(destination, destination, None);
    assert_eq!(
        dom.apply(refused, &cancel).unwrap_err(),
        DomError::HierarchyRequest
    );
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    assert!(
        !dom.snapshot()
            .node(nested_target)
            .unwrap()
            .as_selectedcontent()
            .unwrap()
            .disabled()
    );
    let mut retry = DomBatch::new();
    retry.set_select_selected_index(nested, 0);
    dom.apply(retry, &cancel).unwrap();
    assert_eq!(text_at(&dom, nested_target), DocumentString::from("inner"));
}
