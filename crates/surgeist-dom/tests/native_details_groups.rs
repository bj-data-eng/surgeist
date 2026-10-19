//! Literal HTML877d Details group outcomes through canonical attributes/trees.
//! Source: #details-name-group, the attribute/insertion steps, and the two
//! #ensure-details-exclusivity algorithms. No Runtime delivery is asserted.

use std::sync::atomic::AtomicBool;
use surgeist_dom::*;

fn dom() -> Dom {
    Dom::new(
        DomLimits::new(
            256,
            32768,
            1024,
            512,
            256,
            16,
            1_000_000,
            surgeist_dom::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
        )
        .unwrap()
        .with_details_toggle_intentions(512)
        .unwrap(),
    )
    .unwrap()
}

fn apply(dom: &mut Dom, batch: DomBatch) -> DomCommit {
    dom.apply(batch, &AtomicBool::new(false)).unwrap()
}

fn open(snapshot: &DomSnapshot, node: NodeId) -> bool {
    snapshot
        .node(node)
        .unwrap()
        .attribute(&DocumentString::from("open"))
        .is_some()
}

struct Pair {
    dom: Dom,
    document: DocumentId,
    root: NodeId,
    a: NodeId,
    b: NodeId,
}

fn pair() -> Pair {
    let mut dom = dom();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let root = batch.create_element_ns(&document, Namespace::html(), "div");
    let a = batch.create_element_ns(&document, Namespace::html(), "details");
    let b = batch.create_element_ns(&document, Namespace::html(), "details");
    batch.insert_before(&a, &root, None);
    batch.insert_before(&b, &root, None);
    let commit = apply(&mut dom, batch);
    Pair {
        document: commit.document(&document).unwrap(),
        root: commit.created(&root).unwrap().unwrap(),
        a: commit.created(&a).unwrap().unwrap(),
        b: commit.created(&b).unwrap().unwrap(),
        dom,
    }
}

fn assert_pair(pair: &Pair, expected: [bool; 2]) {
    let snapshot = pair.dom.snapshot();
    assert_eq!([open(&snapshot, pair.a), open(&snapshot, pair.b)], expected);
}

#[test]
fn only_equal_nonempty_exact_utf16_names_form_a_group_in_a_detached_tree() {
    let cases = [
        (None, None, false),
        (Some("".into()), Some("".into()), false),
        (Some("g".into()), None, false),
        (Some("g".into()), Some("g".into()), true),
        (Some(" ".into()), Some(" ".into()), true),
        (Some(" g ".into()), Some("g".into()), false),
        (Some("Group".into()), Some("group".into()), false),
        (Some("a\tb".into()), Some("a b".into()), false),
        (Some("a\0b".into()), Some("a\0b".into()), true),
        (
            Some(DocumentString::from_utf16(vec![0xd800, 0, 0xdfff])),
            Some(DocumentString::from_utf16(vec![0xd800, 0, 0xdfff])),
            true,
        ),
        (
            Some(DocumentString::from_utf16(vec![0xd800])),
            Some(DocumentString::from_utf16(vec![0xd801])),
            false,
        ),
    ];
    for (first_name, second_name, grouped) in cases {
        let mut f = pair();
        let mut batch = DomBatch::new();
        if let Some(name) = first_name {
            batch.set_attribute(f.a, "name", name);
        }
        if let Some(name) = second_name {
            batch.set_attribute(f.b, "name", name);
        }
        batch.set_attribute(f.a, "open", "false");
        batch.set_attribute(f.b, "open", "anything");
        apply(&mut f.dom, batch);
        assert_pair(&f, [!grouped, true]);
        assert!(!f.dom.snapshot().node(f.root).unwrap().is_connected());
    }
}

#[test]
fn opening_closes_the_peer_but_name_changes_close_the_given_details() {
    let mut f = pair();
    let mut batch = DomBatch::new();
    batch.set_attribute(f.a, "name", "g");
    batch.set_attribute(f.b, "name", "other");
    batch.set_attribute(f.a, "open", "false");
    batch.set_attribute(f.b, "open", "");
    apply(&mut f.dom, batch);
    assert_pair(&f, [true, true]);

    let mut batch = DomBatch::new();
    batch.set_attribute(f.b, "name", "g");
    apply(&mut f.dom, batch);
    assert_pair(&f, [true, false]);
    let mut batch = DomBatch::new();
    batch.set_attribute(f.b, "open", "false");
    apply(&mut f.dom, batch);
    assert_pair(&f, [false, true]);

    let mut batch = DomBatch::new();
    batch.set_attribute(f.a, "name", "other");
    batch.set_attribute(f.a, "open", "");
    batch.set_attribute(f.a, "name", "g");
    apply(&mut f.dom, batch);
    assert_pair(&f, [false, true]);
}

#[test]
fn insertion_closes_the_entrant_before_a_later_detach_changes_the_final_tree() {
    let mut f = pair();
    let mut batch = DomBatch::new();
    batch.set_attribute(f.a, "name", "g");
    batch.set_attribute(f.b, "name", "g");
    batch.set_attribute(f.a, "open", "");
    batch.detach(f.b);
    batch.set_attribute(f.b, "open", "");
    apply(&mut f.dom, batch);
    assert_pair(&f, [true, true]);
    let before = f.dom.snapshot();

    let mut batch = DomBatch::new();
    batch.insert_before(f.b, f.root, None);
    batch.detach(f.a);
    apply(&mut f.dom, batch);
    assert_pair(&f, [true, false]);
    assert_eq!(f.dom.snapshot().node(f.a).unwrap().parent(), None);
    assert_eq!(f.dom.snapshot().node(f.b).unwrap().parent(), Some(f.root));
    assert!(open(&before, f.b));
    assert_eq!(before.node(f.b).unwrap().parent(), None);
}

#[test]
fn closed_earlier_peers_are_skipped_and_other_named_groups_remain_open() {
    let mut f = pair();
    let mut batch = DomBatch::new();
    let earlier = batch.create_element_ns(f.document, Namespace::html(), "details");
    let other = batch.create_element_ns(f.document, Namespace::html(), "details");
    batch.insert_before(&earlier, f.root, Some(f.a.into()));
    batch.insert_before(&other, f.root, None);
    for node in [f.a, f.b] {
        batch.set_attribute(node, "name", "g");
    }
    batch.set_attribute(&earlier, "name", "g");
    batch.set_attribute(&other, "name", "other");
    batch.set_attribute(f.a, "open", "");
    batch.set_attribute(&other, "open", "");
    let commit = apply(&mut f.dom, batch);
    let earlier = commit.created(&earlier).unwrap().unwrap();
    let other = commit.created(&other).unwrap().unwrap();

    let mut batch = DomBatch::new();
    batch.set_attribute(f.b, "open", "");
    apply(&mut f.dom, batch);
    let snapshot = f.dom.snapshot();
    assert_eq!(
        [earlier, f.a, f.b, other].map(|node| open(&snapshot, node)),
        [false, false, true, true]
    );
}

#[test]
fn namespaced_attributes_and_details_lookalikes_do_not_join_the_html_group() {
    let mut f = pair();
    let mut batch = DomBatch::new();
    batch.set_attribute(f.a, "name", "g");
    batch.set_attribute(f.b, "name", "g");
    batch.set_attribute(f.a, "open", "");
    batch.set_attribute_ns(f.b, Namespace::from_uri("urn:attribute"), "p:open", "");
    apply(&mut f.dom, batch);
    assert_pair(&f, [true, false]);

    let mut batch = DomBatch::new();
    batch.remove_attribute(f.b, "name");
    batch.set_attribute_ns(f.b, Namespace::from_uri("urn:attribute"), "p:name", "g");
    batch.set_attribute(f.b, "open", "");
    let foreign =
        batch.create_element_ns(f.document, Namespace::from_uri("urn:element"), "details");
    let generic = batch.create_element_ns(f.document, Namespace::none(), "details");
    let uppercase = batch.create_element_ns(f.document, Namespace::html(), "Details");
    for node in [&foreign, &generic, &uppercase] {
        batch.set_attribute(node, "name", "g");
        batch.set_attribute(node, "open", "");
        batch.insert_before(node, f.root, None);
    }
    let commit = apply(&mut f.dom, batch);
    assert_pair(&f, [true, true]);
    let snapshot = f.dom.snapshot();
    for ticket in [foreign, generic, uppercase] {
        assert!(open(&snapshot, commit.created(&ticket).unwrap().unwrap()));
    }
}

#[test]
fn ordinary_light_shadow_and_template_trees_keep_separate_groups() {
    let mut f = pair();
    let mut batch = DomBatch::new();
    let host_a = batch.create_element_ns(f.document, Namespace::html(), "div");
    let host_b = batch.create_element_ns(f.document, Namespace::html(), "div");
    let shadow_a = batch.attach_shadow(&host_a, DomShadowRootInit::new(HtmlShadowRootMode::Open));
    let shadow_b = batch.attach_shadow(&host_b, DomShadowRootInit::new(HtmlShadowRootMode::Closed));
    let template = batch.create_element_ns(f.document, Namespace::html(), "template");
    for node in [&host_a, &host_b, &template] {
        batch.insert_before(node, f.root, None);
    }
    let commit = apply(&mut f.dom, batch);
    let shadow_a = commit.created(&shadow_a).unwrap().unwrap();
    let shadow_b = commit.created(&shadow_b).unwrap().unwrap();
    let template = commit.created(&template).unwrap().unwrap();
    let contents = f
        .dom
        .snapshot()
        .node(template)
        .unwrap()
        .as_template()
        .unwrap()
        .contents()
        .unwrap();
    let mut batch = DomBatch::new();
    let in_a = batch.create_element_ns(f.document, Namespace::html(), "details");
    let in_b = batch.create_element_ns(f.document, Namespace::html(), "details");
    let in_template = batch.create_element_ns(f.document, Namespace::html(), "details");
    for (node, parent) in [
        (&in_a, shadow_a),
        (&in_b, shadow_b),
        (&in_template, contents),
    ] {
        batch.set_attribute(node, "name", "g");
        batch.set_attribute(node, "open", "");
        batch.insert_before(node, parent, None);
    }
    batch.set_attribute(f.a, "name", "g");
    batch.set_attribute(f.b, "name", "g");
    batch.set_attribute(f.a, "open", "");
    batch.set_attribute(f.b, "open", "");
    let commit = apply(&mut f.dom, batch);
    let in_a = commit.created(&in_a).unwrap().unwrap();
    let in_b = commit.created(&in_b).unwrap().unwrap();
    let in_template = commit.created(&in_template).unwrap().unwrap();
    assert_pair(&f, [false, true]);
    let snapshot = f.dom.snapshot();
    assert_eq!(
        [in_a, in_b, in_template].map(|id| open(&snapshot, id)),
        [true; 3]
    );

    let mut batch = DomBatch::new();
    let entrant = batch.create_element_ns(f.document, Namespace::html(), "details");
    batch.set_attribute(&entrant, "name", "g");
    batch.set_attribute(&entrant, "open", "");
    batch.insert_before(&entrant, contents, None);
    let commit = apply(&mut f.dom, batch);
    let entrant = commit.created(&entrant).unwrap().unwrap();
    let snapshot = f.dom.snapshot();
    assert!(!open(&snapshot, entrant));
    assert_eq!(
        [in_a, in_b, in_template, f.b].map(|id| open(&snapshot, id)),
        [true; 4]
    );
}

#[test]
fn nested_group_members_follow_native_rules_even_when_author_nesting_is_nonconforming() {
    let mut f = pair();
    let mut batch = DomBatch::new();
    batch.insert_before(f.b, f.a, None);
    batch.set_attribute(f.a, "name", "g");
    batch.set_attribute(f.b, "name", "g");
    batch.set_attribute(f.a, "open", "");
    batch.set_attribute(f.b, "open", "");
    apply(&mut f.dom, batch);
    assert_pair(&f, [false, true]);
    let mut batch = DomBatch::new();
    batch.set_attribute(f.a, "open", "");
    apply(&mut f.dom, batch);
    assert_pair(&f, [true, false]);
}

#[test]
fn late_invalid_attribute_rolls_back_primary_and_secondary_group_edits() {
    let mut f = pair();
    let mut batch = DomBatch::new();
    batch.set_attribute(f.a, "name", "g");
    batch.set_attribute(f.b, "name", "g");
    batch.set_attribute(f.a, "open", "");
    apply(&mut f.dom, batch);
    let before = f.dom.snapshot();
    let mut batch = DomBatch::new();
    batch.set_attribute(f.b, "open", "");
    batch.set_attribute(f.b, "invalid name", "late failure");
    assert_eq!(
        f.dom.apply(batch, &AtomicBool::new(false)).unwrap_err(),
        DomError::InvalidCharacter
    );
    assert_eq!(f.dom.snapshot().stamp(), before.stamp());
    assert_pair(&f, [true, false]);
    let mut batch = DomBatch::new();
    batch.set_attribute(f.b, "open", "");
    apply(&mut f.dom, batch);
    assert_pair(&f, [false, true]);
    assert_eq!([open(&before, f.a), open(&before, f.b)], [true, false]);
}
