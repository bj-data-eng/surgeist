#![forbid(unsafe_code)]
use std::sync::atomic::AtomicBool;
use surgeist_css::CssCustomIdent;
use surgeist_cssom::*;
use surgeist_dom::*;

fn limits() -> DomLimits {
    DomLimits::new(64, 4096, 128, 0, 0, 32, 65536)
        .unwrap()
        .with_view_transitions(8, 16, 80)
        .unwrap()
}
fn apply(dom: &mut Dom, batch: DomBatch) -> DomCommit {
    dom.apply(batch, &AtomicBool::new(false)).unwrap()
}
fn basis(dom: &Dom, document: DocumentId) -> DomDocumentStamp {
    dom.snapshot().document(document).unwrap().stamp()
}
fn name(value: &str) -> CssViewTransitionIdent {
    CssViewTransitionIdent::try_new(CssCustomIdent::try_new(value).unwrap()).unwrap()
}
fn css() -> (CssomStore, CssomSheetId, Vec<CssomRuleId>) {
    let mut store = CssomStore::new(CssomLimits::default(), CssomContext::default()).unwrap();
    let mut batch = store
        .batch(CssomEditGuard::from_snapshot(&store.snapshot()))
        .unwrap();
    let sheet = batch
        .create_parsed_sheet(
            "a {opacity:0} b {opacity:1} @keyframes move {from {opacity:0}}",
            CssomSheetInputs::constructed(
                CssomInputVersion {
                    role: CssomInputRole::Document,
                    identity: "actual stylesheet fixture".into(),
                    revision: 1,
                },
                "actual stylesheet fixture".into(),
                None,
            ),
        )
        .unwrap();
    let sheet = store.commit(batch).unwrap().sheet(&sheet).unwrap().clone();
    let snapshot = store.snapshot();
    let rules = snapshot
        .rules(snapshot.sheet(&sheet).unwrap().rules())
        .unwrap()
        .to_vec();
    (store, sheet, rules)
}
struct Fixture {
    dom: Dom,
    document: DocumentId,
    root: NodeId,
    child: NodeId,
    css: CssomStore,
    sheet: CssomSheetId,
}
fn fixture() -> Fixture {
    let mut dom = Dom::new(limits()).unwrap();
    let mut batch = DomBatch::new();
    let doc = batch.create_document();
    let root = batch.create_element(&doc, "Root");
    let child = batch.create_element(&doc, "Child");
    batch.insert_before(&root, &doc, None);
    batch.insert_before(&child, &root, None);
    let result = apply(&mut dom, batch);
    let document = result.document(&doc).unwrap();
    let root = result.created(&root).unwrap().unwrap();
    let child = result.created(&child).unwrap().unwrap();
    let (css, sheet, _) = css();
    Fixture {
        dom,
        document,
        root,
        child,
        css,
        sheet,
    }
}
fn initialize(f: &mut Fixture) {
    let mut b = DomBatch::new();
    b.initialize_view_transitions(basis(&f.dom, f.document), &f.css.snapshot(), &f.sheet);
    apply(&mut f.dom, b);
}
fn begin(
    f: &mut Fixture,
    branches: DomViewTransitionBranches,
) -> (DomViewTransitionId, DomViewTransitionRecordId) {
    let mut b = DomBatch::new();
    let stamp = basis(&f.dom, f.document);
    let transition = b.begin_view_transition(stamp);
    let record = b.append_view_transition_record(stamp, &transition, name("one"));
    b.set_view_transition_branches(stamp, &record, branches);
    let result = apply(&mut f.dom, b);
    (
        result.view_transition(&transition).unwrap().unwrap(),
        result.view_transition_record(&record).unwrap().unwrap(),
    )
}
#[test]
fn explicit_real_sheet_initialization_has_literal_defaults_and_no_cssom_effect() {
    let mut f = fixture();
    assert!(
        f.dom
            .snapshot()
            .document(f.document)
            .unwrap()
            .view_transitions()
            .is_none()
    );
    let mut b = DomBatch::new();
    b.begin_view_transition(basis(&f.dom, f.document));
    assert_eq!(
        f.dom.apply(b, &AtomicBool::new(false)).unwrap_err(),
        DomError::ViewTransitionsUninitialized
    );
    let css_before = f.css.snapshot().revision().clone();
    initialize(&mut f);
    let snapshot = f.dom.snapshot();
    let state = snapshot
        .document(f.document)
        .unwrap()
        .view_transitions()
        .unwrap();
    assert_eq!(state.active(), None);
    assert!(!state.show_tree() && !state.rendering_suppressed());
    assert_eq!(state.sheet().id(), &f.sheet);
    assert_eq!(state.sheet().revision(), &css_before);
    assert_eq!(f.css.snapshot().revision(), &css_before);
    assert!(
        !snapshot
            .node(f.child)
            .unwrap()
            .as_element()
            .unwrap()
            .captured_in_view_transition()
    );
    let revision = f.dom.revision();
    initialize(&mut f);
    assert_eq!(f.dom.revision(), revision);
}
#[test]
fn ordered_pending_records_form_old_new_and_paired_pseudos_with_actual_root_origins() {
    use DomViewTransitionPseudoKind::*;
    let mut f = fixture();
    initialize(&mut f);
    let stamp = basis(&f.dom, f.document);
    let mut b = DomBatch::new();
    let t = b.begin_view_transition(stamp);
    let old = b.append_view_transition_record(stamp, &t, name("Old"));
    let new = b.append_view_transition_record(stamp, &t, name("new"));
    let pair = b.append_view_transition_record(stamp, &t, name("pair"));
    let result = apply(&mut f.dom, b);
    let t = result.view_transition(&t).unwrap().unwrap();
    let records = [&old, &new, &pair].map(|r| result.view_transition_record(r).unwrap().unwrap());
    let pending = f.dom.snapshot();
    let transition = pending.view_transition(t).unwrap();
    assert_eq!(
        transition
            .records()
            .map(|r| r.name().as_str())
            .collect::<Vec<_>>(),
        ["Old", "new", "pair"]
    );
    assert!(
        transition
            .records()
            .all(|r| r.branches() == DomViewTransitionBranches::Pending && r.rules().is_none())
    );
    assert!(
        !pending
            .view_transition_pseudo_captured(transition.root())
            .unwrap()
    );
    assert_eq!(
        pending
            .view_transition_pseudo(transition.root())
            .unwrap()
            .children()
            .count(),
        0
    );
    let mut b = DomBatch::new();
    b.setup_view_transition_pseudos(basis(&f.dom, f.document), t);
    assert_eq!(
        f.dom.apply(b, &AtomicBool::new(false)).unwrap_err(),
        DomError::InvalidViewTransitionState
    );
    let mut b = DomBatch::new();
    let stamp = basis(&f.dom, f.document);
    b.set_view_transition_branches(stamp, records[0], DomViewTransitionBranches::Old);
    b.set_view_transition_branches(stamp, records[1], DomViewTransitionBranches::New(f.child));
    b.set_view_transition_branches(
        stamp,
        records[2],
        DomViewTransitionBranches::OldAndNew(f.root),
    );
    b.setup_view_transition_pseudos(stamp, t);
    apply(&mut f.dom, b);
    let snapshot = f.dom.snapshot();
    let transition = snapshot.view_transition(t).unwrap();
    let root = snapshot.view_transition_pseudo(transition.root()).unwrap();
    assert_eq!(root.originating_element(), f.root);
    assert_eq!(root.parent(), None);
    let groups = root.children().collect::<Vec<_>>();
    assert_eq!(groups.len(), 3);
    for (index, expected) in [vec![Old], vec![New], vec![Old, New]].iter().enumerate() {
        let group = snapshot.view_transition_pseudo(groups[index]).unwrap();
        assert_eq!(group.id().kind(), Group);
        assert_eq!(group.parent(), Some(root.id()));
        assert_eq!(group.originating_pseudo(), group.parent());
        assert_eq!(group.originating_element(), f.root);
        let pair = snapshot
            .view_transition_pseudo(group.children().next().unwrap())
            .unwrap();
        assert_eq!(pair.id().kind(), ImagePair);
        assert_eq!(pair.parent(), Some(group.id()));
        let children = pair.children().collect::<Vec<_>>();
        assert_eq!(
            children.iter().map(|id| id.kind()).collect::<Vec<_>>(),
            *expected
        );
        for id in children {
            let value = snapshot.view_transition_pseudo(id).unwrap();
            assert_eq!(value.parent(), Some(pair.id()));
            assert_eq!(value.originating_element(), f.root);
            assert!(!value.captured());
            assert_eq!(
                value.stamp(),
                snapshot.document(f.document).unwrap().stamp()
            );
            assert_eq!(value.children().count(), 0);
        }
    }
    assert_eq!(snapshot.node(f.root).unwrap().children(), &[f.child]);
    assert_eq!(f.dom.usage().view_transition_pseudos(), 11);
}
#[test]
fn root_removal_and_replacement_derive_current_origin_without_restricting_tree_edits() {
    let mut f = fixture();
    initialize(&mut f);
    let (t, _) = begin(&mut f, DomViewTransitionBranches::Old);
    let mut b = DomBatch::new();
    b.setup_view_transition_pseudos(basis(&f.dom, f.document), t);
    apply(&mut f.dom, b);
    let old = f.dom.snapshot();
    let root = old.view_transition(t).unwrap().root();
    let mut b = DomBatch::new();
    b.detach(f.root);
    let changed = apply(&mut f.dom, b);
    assert!(
        changed
            .changes()
            .changes()
            .contains(&DomChange::ViewTransition(t))
    );
    assert_eq!(
        f.dom.snapshot().view_transition_pseudo(root).unwrap_err(),
        DomError::ViewTransitionOriginUnavailable
    );
    let mut b = DomBatch::new();
    b.set_view_transition_captured(
        basis(&f.dom, f.document),
        DomViewTransitionSubject::Pseudo(root),
        true,
    );
    apply(&mut f.dom, b);
    assert!(
        f.dom
            .snapshot()
            .view_transition_pseudo_captured(root)
            .unwrap()
    );
    let mut b = DomBatch::new();
    let new = b.create_element(f.document, "Replacement");
    b.insert_before(&new, f.document, None);
    let result = apply(&mut f.dom, b);
    let new = result.created(&new).unwrap().unwrap();
    assert_eq!(
        f.dom
            .snapshot()
            .view_transition_pseudo(root)
            .unwrap()
            .originating_element(),
        new
    );
    assert_eq!(
        old.view_transition_pseudo(root)
            .unwrap()
            .originating_element(),
        f.root
    );
}
#[test]
fn element_and_every_generated_pseudo_have_independent_flags_and_exact_change_domains() {
    let mut f = fixture();
    initialize(&mut f);
    let child = f.child;
    let (t, r) = begin(&mut f, DomViewTransitionBranches::OldAndNew(child));
    let mut b = DomBatch::new();
    b.setup_view_transition_pseudos(basis(&f.dom, f.document), t);
    apply(&mut f.dom, b);
    let snapshot = f.dom.snapshot();
    let transition = snapshot.view_transition(t).unwrap();
    let record = snapshot.view_transition_record(r).unwrap();
    let mut subjects = vec![
        DomViewTransitionSubject::Element(child),
        DomViewTransitionSubject::Pseudo(transition.root()),
    ];
    for kind in [
        DomViewTransitionPseudoKind::Group,
        DomViewTransitionPseudoKind::ImagePair,
        DomViewTransitionPseudoKind::Old,
        DomViewTransitionPseudoKind::New,
    ] {
        subjects.push(DomViewTransitionSubject::Pseudo(
            record.pseudo(kind).unwrap(),
        ));
    }
    let mut b = DomBatch::new();
    for subject in &subjects {
        b.set_view_transition_captured(basis(&f.dom, f.document), *subject, true);
    }
    let result = apply(&mut f.dom, b);
    assert!(
        result
            .changes()
            .changes()
            .contains(&DomChange::ViewTransitionCaptured(
                DomViewTransitionSubject::Element(child)
            ))
    );
    assert!(
        !result
            .changes()
            .changes()
            .contains(&DomChange::Attributes(child))
    );
    for subject in &subjects[1..] {
        assert!(
            result
                .changes()
                .changes()
                .contains(&DomChange::ViewTransitionCaptured(*subject))
        );
    }
    let current = f.dom.snapshot();
    assert!(
        current
            .node(child)
            .unwrap()
            .as_element()
            .unwrap()
            .captured_in_view_transition()
    );
    for subject in &subjects[1..] {
        let DomViewTransitionSubject::Pseudo(id) = subject else {
            unreachable!()
        };
        assert!(current.view_transition_pseudo_captured(*id).unwrap());
        assert!(!snapshot.view_transition_pseudo_captured(*id).unwrap());
    }
    let revision = f.dom.revision();
    let mut b = DomBatch::new();
    for subject in subjects {
        b.set_view_transition_captured(basis(&f.dom, f.document), subject, true);
    }
    apply(&mut f.dom, b);
    assert_eq!(f.dom.revision(), revision);
}
#[test]
fn same_new_element_two_names_and_foreign_stale_wrong_kind_or_document_assignments_reject_atomically()
 {
    let mut f = fixture();
    initialize(&mut f);
    let child = f.child;
    let (t, _) = begin(&mut f, DomViewTransitionBranches::New(child));
    let before = f.dom.snapshot();
    let usage = f.dom.usage();
    let mut b = DomBatch::new();
    let r = b.append_view_transition_record(basis(&f.dom, f.document), t, name("second"));
    b.set_view_transition_branches(
        basis(&f.dom, f.document),
        &r,
        DomViewTransitionBranches::New(child),
    );
    assert_eq!(
        f.dom.apply(b, &AtomicBool::new(false)).unwrap_err(),
        DomError::InvalidViewTransitionState
    );
    assert_eq!(f.dom.snapshot().stamp(), before.stamp());
    assert_eq!(f.dom.usage(), usage);
    let mut b = DomBatch::new();
    let other = b.create_document();
    let foreign_doc_element = b.create_element(&other, "Other");
    let text = b.create_text(f.document, "wrong kind");
    let stale = b.create_element(f.document, "gone");
    let result = apply(&mut f.dom, b);
    let other = result.created(&foreign_doc_element).unwrap().unwrap();
    let text = result.created(&text).unwrap().unwrap();
    let stale = result.created(&stale).unwrap().unwrap();
    let mut b = DomBatch::new();
    b.destroy(stale);
    apply(&mut f.dom, b);
    let foreign = fixture().root;
    for (node, expected) in [
        (other, DomError::WrongDocument),
        (text, DomError::WrongNodeKind),
        (stale, DomError::StaleNode),
        (foreign, DomError::ForeignOwner),
    ] {
        let mut b = DomBatch::new();
        let r = b.append_view_transition_record(basis(&f.dom, f.document), t, name("second"));
        b.set_view_transition_branches(
            basis(&f.dom, f.document),
            &r,
            DomViewTransitionBranches::New(node),
        );
        assert_eq!(
            f.dom.apply(b, &AtomicBool::new(false)).unwrap_err(),
            expected
        );
    }
    let mut b = DomBatch::new();
    b.append_view_transition_record(basis(&f.dom, f.document), t, name("one"));
    assert_eq!(
        f.dom.apply(b, &AtomicBool::new(false)).unwrap_err(),
        DomError::DuplicateViewTransitionName
    );
    let mut b = DomBatch::new();
    b.append_view_transition_record(basis(&f.dom, f.document), t, name("One"));
    apply(&mut f.dom, b);
}
#[test]
fn clear_retains_records_suppression_and_rules_and_cannot_clear_a_newer_capture() {
    let mut f = fixture();
    initialize(&mut f);
    let child = f.child;
    let (old, record) = begin(&mut f, DomViewTransitionBranches::New(child));
    let mut b = DomBatch::new();
    let stamp = basis(&f.dom, f.document);
    b.setup_view_transition_pseudos(stamp, old);
    b.set_view_transition_captured(stamp, DomViewTransitionSubject::Element(child), true);
    b.set_view_transition_rendering_suppression(stamp, true);
    b.set_view_transition_rules(
        stamp,
        record,
        &f.css.snapshot(),
        DomViewTransitionRuleAssociations::default(),
    );
    apply(&mut f.dom, b);
    let before = f.dom.snapshot();
    let mut b = DomBatch::new();
    b.clear_view_transition_state(basis(&f.dom, f.document), old);
    apply(&mut f.dom, b);
    let snapshot = f.dom.snapshot();
    let state = snapshot
        .document(f.document)
        .unwrap()
        .view_transitions()
        .unwrap();
    assert_eq!(state.active(), None);
    assert!(!state.show_tree());
    assert!(state.rendering_suppressed());
    assert_eq!(snapshot.view_transition(old).unwrap().records().count(), 1);
    assert!(
        snapshot
            .view_transition_record(record)
            .unwrap()
            .rules()
            .is_some()
    );
    assert!(
        !snapshot
            .node(child)
            .unwrap()
            .as_element()
            .unwrap()
            .captured_in_view_transition()
    );
    assert!(
        before
            .node(child)
            .unwrap()
            .as_element()
            .unwrap()
            .captured_in_view_transition()
    );
    let revision = f.dom.revision();
    let mut b = DomBatch::new();
    b.clear_view_transition_state(basis(&f.dom, f.document), old);
    apply(&mut f.dom, b);
    assert_eq!(f.dom.revision(), revision);
    let (new, _) = begin(&mut f, DomViewTransitionBranches::New(child));
    let mut b = DomBatch::new();
    b.set_view_transition_captured(
        basis(&f.dom, f.document),
        DomViewTransitionSubject::Element(child),
        true,
    );
    apply(&mut f.dom, b);
    let mut b = DomBatch::new();
    b.clear_view_transition_state(basis(&f.dom, f.document), old);
    assert_eq!(
        f.dom.apply(b, &AtomicBool::new(false)).unwrap_err(),
        DomError::InvalidViewTransitionState
    );
    assert_eq!(
        f.dom
            .snapshot()
            .document(f.document)
            .unwrap()
            .view_transitions()
            .unwrap()
            .active(),
        Some(new)
    );
    assert!(
        f.dom
            .snapshot()
            .node(child)
            .unwrap()
            .as_element()
            .unwrap()
            .captured_in_view_transition()
    );
    let mut b = DomBatch::new();
    b.release_view_transition(basis(&f.dom, f.document), old);
    apply(&mut f.dom, b);
    assert_eq!(
        f.dom.snapshot().view_transition(old).unwrap_err(),
        DomError::StaleViewTransition
    );
    assert!(before.view_transition(old).is_ok());
}

#[test]
fn real_cssom_zero_one_two_rules_preserve_handles_revision_and_old_before_new_order() {
    for selection in 0..3 {
        let mut f = fixture();
        initialize(&mut f);
        let child = f.child;
        let branches = [
            DomViewTransitionBranches::Old,
            DomViewTransitionBranches::New(child),
            DomViewTransitionBranches::OldAndNew(child),
        ][selection];
        let (_, record) = begin(&mut f, branches);
        let snapshot = f.css.snapshot();
        let roots = snapshot
            .rules(snapshot.sheet(&f.sheet).unwrap().rules())
            .unwrap();
        let image_animation = match selection {
            0 => DomViewTransitionImageRules::Old(roots[0].clone()),
            1 => DomViewTransitionImageRules::New(roots[1].clone()),
            _ => DomViewTransitionImageRules::OldThenNew {
                old: roots[0].clone(),
                new: roots[1].clone(),
            },
        };
        let request = DomViewTransitionRuleAssociations {
            group_keyframes: Some(roots[2].clone()),
            image_animation,
            ..Default::default()
        };
        let mut b = DomBatch::new();
        b.set_view_transition_rules(
            basis(&f.dom, f.document),
            record,
            &snapshot,
            request.clone(),
        );
        apply(&mut f.dom, b);
        let captured = f.dom.snapshot();
        let actual = captured
            .view_transition_record(record)
            .unwrap()
            .rules()
            .unwrap();
        assert_eq!(actual.revision(), snapshot.revision());
        assert_eq!(actual.associations(), &request);
        let mut b = DomBatch::new();
        b.clear_view_transition_rules(basis(&f.dom, f.document), record);
        apply(&mut f.dom, b);
        assert!(
            f.dom
                .snapshot()
                .view_transition_record(record)
                .unwrap()
                .rules()
                .is_none()
        );
        assert_eq!(f.css.snapshot().revision(), snapshot.revision());
        assert_eq!(
            snapshot
                .rules(snapshot.sheet(&f.sheet).unwrap().rules())
                .unwrap(),
            roots
        );
        assert_eq!(
            captured
                .view_transition_record(record)
                .unwrap()
                .rules()
                .unwrap()
                .associations(),
            &request
        );
    }
}
#[test]
fn cssom_identity_kind_membership_duplicates_order_and_revision_errors_preserve_dom() {
    let mut f = fixture();
    initialize(&mut f);
    let child = f.child;
    let (_, record) = begin(&mut f, DomViewTransitionBranches::OldAndNew(child));
    let old = f.css.snapshot();
    let roots = old
        .rules(old.sheet(&f.sheet).unwrap().rules())
        .unwrap()
        .to_vec();
    let (_, other_sheet, foreign) = css();
    let requests = [
        (
            DomViewTransitionRuleAssociations {
                image_animation: DomViewTransitionImageRules::OldThenNew {
                    old: roots[1].clone(),
                    new: roots[0].clone(),
                },
                ..Default::default()
            },
            DomViewTransitionCssomError::RuleOrder,
        ),
        (
            DomViewTransitionRuleAssociations {
                image_animation: DomViewTransitionImageRules::OldThenNew {
                    old: roots[0].clone(),
                    new: roots[0].clone(),
                },
                ..Default::default()
            },
            DomViewTransitionCssomError::DuplicateRule,
        ),
        (
            DomViewTransitionRuleAssociations {
                group_styles: Some(roots[2].clone()),
                ..Default::default()
            },
            DomViewTransitionCssomError::WrongKind,
        ),
        (
            DomViewTransitionRuleAssociations {
                group_keyframes: Some(roots[0].clone()),
                ..Default::default()
            },
            DomViewTransitionCssomError::WrongKind,
        ),
        (
            DomViewTransitionRuleAssociations {
                group_styles: Some(foreign[0].clone()),
                ..Default::default()
            },
            DomViewTransitionCssomError::ForeignOwner,
        ),
    ];
    for (request, expected) in requests {
        let stamp = f.dom.snapshot().stamp();
        let mut b = DomBatch::new();
        b.set_view_transition_rules(basis(&f.dom, f.document), record, &old, request);
        assert_eq!(
            f.dom.apply(b, &AtomicBool::new(false)).unwrap_err(),
            DomError::ViewTransitionCssom(expected)
        );
        assert_eq!(f.dom.snapshot().stamp(), stamp);
    }
    let mut edit = f.css.batch(CssomEditGuard::from_snapshot(&old)).unwrap();
    let detached = edit.delete_sheet_rule(&f.sheet, 0).unwrap();
    f.css.commit(edit).unwrap();
    let current = f.css.snapshot();
    assert!(current.rule(&detached).is_ok());
    let mut b = DomBatch::new();
    b.set_view_transition_rules(
        basis(&f.dom, f.document),
        record,
        &current,
        DomViewTransitionRuleAssociations {
            group_styles: Some(detached),
            ..Default::default()
        },
    );
    assert_eq!(
        f.dom.apply(b, &AtomicBool::new(false)).unwrap_err(),
        DomError::ViewTransitionCssom(DomViewTransitionCssomError::WrongSheet)
    );
    let mut b = DomBatch::new();
    b.set_view_transition_rules(
        basis(&f.dom, f.document),
        record,
        &current,
        DomViewTransitionRuleAssociations::default(),
    );
    apply(&mut f.dom, b);
    let mut b = DomBatch::new();
    b.set_view_transition_rules(
        basis(&f.dom, f.document),
        record,
        &old,
        DomViewTransitionRuleAssociations::default(),
    );
    assert_eq!(
        f.dom.apply(b, &AtomicBool::new(false)).unwrap_err(),
        DomError::ViewTransitionCssom(DomViewTransitionCssomError::StaleRevision)
    );
    let mut b = DomBatch::new();
    b.initialize_view_transitions(basis(&f.dom, f.document), &current, &other_sheet);
    assert_eq!(
        f.dom.apply(b, &AtomicBool::new(false)).unwrap_err(),
        DomError::ViewTransitionCssom(DomViewTransitionCssomError::ForeignOwner)
    );
}
#[test]
fn adopted_active_and_inactive_references_follow_actual_identity_and_clear_across_documents_atomically()
 {
    let mut f = fixture();
    initialize(&mut f);
    let child = f.child;
    let (t, record) = begin(&mut f, DomViewTransitionBranches::New(child));
    let mut b = DomBatch::new();
    let other = b.create_document();
    let second = b.create_element(&other, "Second");
    let result = apply(&mut f.dom, b);
    let other = result.document(&other).unwrap();
    let second = result.created(&second).unwrap().unwrap();
    let mut b = DomBatch::new();
    b.set_view_transition_captured(
        basis(&f.dom, f.document),
        DomViewTransitionSubject::Element(child),
        true,
    );
    b.adopt(child, other);
    apply(&mut f.dom, b);
    let adopted = f.dom.snapshot();
    assert_eq!(
        adopted
            .view_transition_record(record)
            .unwrap()
            .resolve_new_element()
            .unwrap()
            .unwrap()
            .owner_document(),
        Some(other)
    );
    let mut b = DomBatch::new();
    b.clear_view_transition_state(basis(&f.dom, f.document), t);
    b.insert_before(other, second, None);
    let before = f.dom.snapshot();
    assert_eq!(
        f.dom.apply(b, &AtomicBool::new(false)).unwrap_err(),
        DomError::HierarchyRequest
    );
    assert_eq!(f.dom.snapshot().stamp(), before.stamp());
    assert!(
        f.dom
            .snapshot()
            .node(child)
            .unwrap()
            .as_element()
            .unwrap()
            .captured_in_view_transition()
    );
    let mut b = DomBatch::new();
    b.clear_view_transition_state(basis(&f.dom, f.document), t);
    let result = apply(&mut f.dom, b);
    assert!(
        result
            .changes()
            .changes()
            .contains(&DomChange::ViewTransitionCaptured(
                DomViewTransitionSubject::Element(child)
            ))
    );
    assert!(
        result
            .changes()
            .changes()
            .contains(&DomChange::ViewTransitionDocument(f.document))
    );
    assert!(
        !f.dom
            .snapshot()
            .node(child)
            .unwrap()
            .as_element()
            .unwrap()
            .captured_in_view_transition()
    );
    assert!(
        adopted
            .node(child)
            .unwrap()
            .as_element()
            .unwrap()
            .captured_in_view_transition()
    );
    let mut b = DomBatch::new();
    b.adopt(child, f.document);
    apply(&mut f.dom, b);
    assert_eq!(
        f.dom
            .snapshot()
            .view_transition_record(record)
            .unwrap()
            .resolve_new_element()
            .unwrap()
            .unwrap()
            .owner_document(),
        Some(f.document)
    );
}
#[test]
fn active_and_inactive_native_destruction_retain_history_and_clear_never_follows_reused_slots() {
    for clear_first in [false, true] {
        let mut f = fixture();
        initialize(&mut f);
        let child = f.child;
        let (t, record) = begin(&mut f, DomViewTransitionBranches::New(child));
        let mut b = DomBatch::new();
        b.set_view_transition_captured(
            basis(&f.dom, f.document),
            DomViewTransitionSubject::Element(child),
            true,
        );
        apply(&mut f.dom, b);
        let old = f.dom.snapshot();
        if clear_first {
            let mut b = DomBatch::new();
            b.clear_view_transition_state(basis(&f.dom, f.document), t);
            apply(&mut f.dom, b);
        }
        let mut b = DomBatch::new();
        b.detach(child);
        b.destroy(child);
        let replacement = b.create_element(f.document, "Replacement");
        let result = apply(&mut f.dom, b);
        let replacement = result.created(&replacement).unwrap().unwrap();
        assert_ne!(replacement, child);
        let snapshot = f.dom.snapshot();
        assert_eq!(
            snapshot
                .view_transition_record(record)
                .unwrap()
                .branches()
                .new_element(),
            Some(child)
        );
        assert_eq!(
            snapshot
                .view_transition_record(record)
                .unwrap()
                .resolve_new_element()
                .unwrap_err(),
            DomError::StaleNode
        );
        assert_eq!(
            old.view_transition_record(record)
                .unwrap()
                .resolve_new_element()
                .unwrap()
                .unwrap()
                .id(),
            child
        );
        let mut b = DomBatch::new();
        b.set_view_transition_captured(
            basis(&f.dom, f.document),
            DomViewTransitionSubject::Element(replacement),
            true,
        );
        b.clear_view_transition_state(basis(&f.dom, f.document), t);
        apply(&mut f.dom, b);
        assert!(
            f.dom
                .snapshot()
                .node(replacement)
                .unwrap()
                .as_element()
                .unwrap()
                .captured_in_view_transition()
        );
        drop(f.dom);
        assert!(
            old.view_transition_record(record)
                .unwrap()
                .resolve_new_element()
                .unwrap()
                .is_some()
        );
    }
}
#[test]
fn generic_import_resets_the_target_capture_flag_and_preserves_the_captured_source() {
    let mut f = fixture();
    initialize(&mut f);
    let mut b = DomBatch::new();
    b.set_view_transition_captured(
        basis(&f.dom, f.document),
        DomViewTransitionSubject::Element(f.child),
        true,
    );
    apply(&mut f.dom, b);
    let source = f.dom.snapshot();
    let mut b = DomBatch::new();
    let imported = b.import_node(&source, f.child, f.document, true);
    let result = apply(&mut f.dom, b);
    let target = result.created(&imported).unwrap().unwrap();
    assert!(
        !f.dom
            .snapshot()
            .node(target)
            .unwrap()
            .as_element()
            .unwrap()
            .captured_in_view_transition()
    );
    assert!(
        source
            .node(f.child)
            .unwrap()
            .as_element()
            .unwrap()
            .captured_in_view_transition()
    );
}
#[test]
fn finite_feature_limits_names_pseudos_and_stale_basis_fail_without_identity_or_history_changes() {
    let mut f = fixture();
    initialize(&mut f);
    let base_limits = DomLimits::new(64, 4096, 128, 0, 0, 32, 65536).unwrap();
    f.dom.set_limits(base_limits).unwrap();
    let before = f.dom.snapshot();
    let mut b = DomBatch::new();
    b.begin_view_transition(basis(&f.dom, f.document));
    assert_eq!(
        f.dom.apply(b, &AtomicBool::new(false)).unwrap_err(),
        DomError::ResourceLimit {
            resource: DomResource::ViewTransitions,
            limit: 0
        }
    );
    assert_eq!(f.dom.snapshot().stamp(), before.stamp());
    f.dom
        .set_limits(base_limits.with_view_transitions(1, 1, 3).unwrap())
        .unwrap();
    let mut b = DomBatch::new();
    let stale_ticket = b.begin_view_transition(basis(&f.dom, f.document));
    let record =
        b.append_view_transition_record(basis(&f.dom, f.document), &stale_ticket, name("first"));
    b.set_view_transition_branches(
        basis(&f.dom, f.document),
        &record,
        DomViewTransitionBranches::OldAndNew(f.child),
    );
    b.setup_view_transition_pseudos(basis(&f.dom, f.document), &stale_ticket);
    assert_eq!(
        f.dom.apply(b, &AtomicBool::new(false)).unwrap_err(),
        DomError::ResourceLimit {
            resource: DomResource::ViewTransitionPseudos,
            limit: 3
        }
    );
    assert_eq!(f.dom.snapshot().stamp(), before.stamp());
    f.dom.set_limits(limits()).unwrap();
    let (t, record) = begin(&mut f, DomViewTransitionBranches::Old);
    let mut b = DomBatch::new();
    b.setup_view_transition_pseudos(basis(&f.dom, f.document), &stale_ticket);
    assert_eq!(
        f.dom.apply(b, &AtomicBool::new(false)).unwrap_err(),
        DomError::ForeignBatch
    );
    assert_eq!(f.dom.usage().view_transitions(), 1);
    assert_eq!(f.dom.usage().view_transition_records(), 1);
    assert_eq!(f.dom.usage().view_transition_pseudos(), 1);
    assert_eq!(
        f.dom.set_limits(base_limits),
        Err(DomError::ResourceLimit {
            resource: DomResource::ViewTransitions,
            limit: 0
        })
    );
    let stale = basis(&f.dom, f.document);
    let mut b = DomBatch::new();
    b.set_attribute(f.child, "k", "v");
    apply(&mut f.dom, b);
    let mut b = DomBatch::new();
    b.set_view_transition_rendering_suppression(stale, true);
    assert_eq!(
        f.dom.apply(b, &AtomicBool::new(false)).unwrap_err(),
        DomError::ViewTransitionRevisionMismatch
    );
    let mut b = DomBatch::new();
    b.clear_view_transition_state(basis(&f.dom, f.document), t);
    b.release_view_transition(basis(&f.dom, f.document), t);
    let committed = apply(&mut f.dom, b);
    assert!(
        committed
            .changes()
            .changes()
            .contains(&DomChange::ViewTransitionRecord(record))
    );
    assert_eq!(f.dom.usage().view_transition_records(), 0);
    let stamp = f.dom.snapshot().stamp();
    let usage = f.dom.usage();
    let mut b = DomBatch::new();
    let transient = b.begin_view_transition(basis(&f.dom, f.document));
    let record =
        b.append_view_transition_record(basis(&f.dom, f.document), &transient, name("temporary"));
    b.clear_view_transition_state(basis(&f.dom, f.document), &transient);
    b.release_view_transition(basis(&f.dom, f.document), &transient);
    let result = apply(&mut f.dom, b);
    assert_eq!(result.view_transition(&transient).unwrap(), None);
    assert_eq!(result.view_transition_record(&record).unwrap(), None);
    assert_eq!(f.dom.snapshot().stamp(), stamp);
    assert!(result.changes().changes().is_empty());
    assert_eq!(f.dom.usage(), usage);
    let mut b = DomBatch::new();
    b.begin_view_transition(basis(&f.dom, f.document));
    assert_eq!(
        f.dom.apply(b, &AtomicBool::new(true)).unwrap_err(),
        DomError::Cancelled
    );
    assert_eq!(f.dom.snapshot().stamp(), stamp);
}

#[test]
fn record_string_and_pseudo_capacity_boundaries_count_retained_state_until_release() {
    let mut f = fixture();
    initialize(&mut f);
    let initial_units = f.dom.usage().string_code_units();
    f.dom
        .set_limits(
            DomLimits::new(64, initial_units + 3, 128, 0, 0, 32, 65536)
                .unwrap()
                .with_view_transitions(2, 1, 5)
                .unwrap(),
        )
        .unwrap();
    let mut b = DomBatch::new();
    let stamp = basis(&f.dom, f.document);
    let t = b.begin_view_transition(stamp);
    let r = b.append_view_transition_record(stamp, &t, name("𐐀x"));
    b.set_view_transition_branches(stamp, &r, DomViewTransitionBranches::OldAndNew(f.child));
    b.setup_view_transition_pseudos(stamp, &t);
    let result = apply(&mut f.dom, b);
    let t = result.view_transition(&t).unwrap().unwrap();
    assert_eq!(f.dom.usage().string_code_units(), initial_units + 3);
    assert_eq!(f.dom.usage().view_transition_pseudos(), 5);
    let mut b = DomBatch::new();
    b.clear_view_transition_state(basis(&f.dom, f.document), t);
    apply(&mut f.dom, b);
    assert_eq!(f.dom.usage().view_transition_records(), 1);
    assert_eq!(f.dom.usage().view_transition_pseudos(), 5);
    let before = f.dom.snapshot();
    let mut b = DomBatch::new();
    let next = b.begin_view_transition(basis(&f.dom, f.document));
    b.append_view_transition_record(basis(&f.dom, f.document), &next, name("x"));
    assert_eq!(
        f.dom.apply(b, &AtomicBool::new(false)).unwrap_err(),
        DomError::ResourceLimit {
            resource: DomResource::ViewTransitionPseudos,
            limit: 5
        }
    );
    assert_eq!(f.dom.snapshot().stamp(), before.stamp());
    f.dom
        .set_limits(
            DomLimits::new(64, initial_units + 3, 128, 0, 0, 32, 65536)
                .unwrap()
                .with_view_transitions(2, 1, 8)
                .unwrap(),
        )
        .unwrap();
    let mut b = DomBatch::new();
    let next = b.begin_view_transition(basis(&f.dom, f.document));
    b.append_view_transition_record(basis(&f.dom, f.document), &next, name("x"));
    assert_eq!(
        f.dom.apply(b, &AtomicBool::new(false)).unwrap_err(),
        DomError::ResourceLimit {
            resource: DomResource::ViewTransitionRecords,
            limit: 1
        }
    );
    f.dom
        .set_limits(
            DomLimits::new(64, initial_units + 3, 128, 0, 0, 32, 65536)
                .unwrap()
                .with_view_transitions(2, 2, 8)
                .unwrap(),
        )
        .unwrap();
    let mut b = DomBatch::new();
    let next = b.begin_view_transition(basis(&f.dom, f.document));
    b.append_view_transition_record(basis(&f.dom, f.document), &next, name("x"));
    assert_eq!(
        f.dom.apply(b, &AtomicBool::new(false)).unwrap_err(),
        DomError::ResourceLimit {
            resource: DomResource::StringCodeUnits,
            limit: initial_units + 3
        }
    );
    let mut b = DomBatch::new();
    b.release_view_transition(basis(&f.dom, f.document), t);
    apply(&mut f.dom, b);
    assert_eq!(f.dom.usage().string_code_units(), initial_units);
    assert_eq!(f.dom.usage().view_transition_pseudos(), 0);
}

#[test]
fn exact_document_basis_and_typed_transition_subjects_reject_foreign_stale_and_missing_origins() {
    let mut f = fixture();
    initialize(&mut f);
    let (t, record) = begin(&mut f, DomViewTransitionBranches::Old);
    let mut other = fixture();
    initialize(&mut other);
    let (foreign, foreign_record) = begin(&mut other, DomViewTransitionBranches::Old);
    let foreign_root = other
        .dom
        .snapshot()
        .view_transition(foreign)
        .unwrap()
        .root();
    assert_eq!(
        f.dom.snapshot().view_transition(foreign).unwrap_err(),
        DomError::ForeignOwner
    );
    assert_eq!(
        f.dom
            .snapshot()
            .view_transition_record(foreign_record)
            .unwrap_err(),
        DomError::ForeignOwner
    );
    let before = f.dom.snapshot();
    let mut b = DomBatch::new();
    b.set_view_transition_captured(
        basis(&f.dom, f.document),
        DomViewTransitionSubject::Pseudo(foreign_root),
        true,
    );
    assert_eq!(
        f.dom.apply(b, &AtomicBool::new(false)).unwrap_err(),
        DomError::ForeignOwner
    );
    assert_eq!(f.dom.snapshot().stamp(), before.stamp());
    let mut b = DomBatch::new();
    b.detach(f.root);
    apply(&mut f.dom, b);
    let mut b = DomBatch::new();
    b.setup_view_transition_pseudos(basis(&f.dom, f.document), t);
    assert_eq!(
        f.dom.apply(b, &AtomicBool::new(false)).unwrap_err(),
        DomError::ViewTransitionOriginUnavailable
    );
    let mut b = DomBatch::new();
    b.clear_view_transition_state(basis(&f.dom, f.document), t);
    b.release_view_transition(basis(&f.dom, f.document), t);
    apply(&mut f.dom, b);
    let mut b = DomBatch::new();
    b.set_view_transition_branches(
        basis(&f.dom, f.document),
        record,
        DomViewTransitionBranches::Old,
    );
    assert_eq!(
        f.dom.apply(b, &AtomicBool::new(false)).unwrap_err(),
        DomError::StaleViewTransitionRecord
    );
    assert!(
        before
            .view_transition_record(record)
            .unwrap()
            .resolve_new_element()
            .unwrap()
            .is_none()
    );
}
