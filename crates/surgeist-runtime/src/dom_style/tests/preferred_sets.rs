//! Current native Style associations apply CSSOM's add-sheet selection steps.
use super::*;

fn append_named_style(session: &mut DomSession, document: NodeId, title: &str) -> NodeId {
    let cancel = AtomicBool::new(false);
    let mut source = session.source_execution();
    let mut batch = DomBatch::new();
    let style = batch.create_element_ns(document, Namespace::html(), "style");
    batch.set_attribute(&style, "title", title);
    let setup = session.dom().apply(batch, &cancel).unwrap();
    let style = setup.created(&style).unwrap().unwrap();
    let container = session.snapshot().node(document).unwrap().children()[0];
    assert!(matches!(
        source
            .apply(
                &mut session.dom(),
                DomSourceEdit::insert_before(style, container, None),
                source_limits(),
                &mut DomSourceServices {
                    execution_scope: None,
                    style_policy: &mut policy,
                    url: None,
                },
                &cancel,
            )
            .unwrap(),
        DomSourceDriveProgress::Complete(_)
    ));
    style
}

#[test]
fn later_non_alternate_named_style_is_disabled_by_first_preferred_set() {
    let (mut session, document) = bound_session();
    let styles = session.style_execution();
    assert!(
        styles
            .snapshot(document)
            .unwrap()
            .unwrap()
            .sheets()
            .is_empty()
    );

    let a = append_named_style(&mut session, document, "A");
    let first_dom = session.snapshot();
    let first_association = first_dom
        .node(a)
        .unwrap()
        .as_style()
        .unwrap()
        .sheet()
        .unwrap()
        .unwrap();
    let a_sheet = first_association.id().clone();
    let first_cssom = styles.snapshot(document).unwrap().unwrap();
    assert_eq!(first_cssom.sheets(), std::slice::from_ref(&a_sheet));
    assert_eq!(
        first_cssom
            .sheet(&a_sheet)
            .unwrap()
            .inputs()
            .title
            .as_slice(),
        &[65]
    );
    assert!(!first_cssom.sheet(&a_sheet).unwrap().inputs().alternate);
    assert!(!styles.disabled(&session.dom(), a).unwrap().disabled());

    let b = append_named_style(&mut session, document, "B");
    let current_dom = session.snapshot();
    let current_a = current_dom
        .node(a)
        .unwrap()
        .as_style()
        .unwrap()
        .sheet()
        .unwrap()
        .unwrap();
    let current_b = current_dom
        .node(b)
        .unwrap()
        .as_style()
        .unwrap()
        .sheet()
        .unwrap()
        .unwrap();
    assert_eq!(current_a.id(), &a_sheet);
    assert_eq!(current_a.association(), first_association.association());
    assert_eq!(current_a.root(), document);
    assert_eq!(current_b.root(), document);
    assert_ne!(current_b.id(), &a_sheet);
    let current_cssom = styles.snapshot(document).unwrap().unwrap();
    assert_eq!(
        current_cssom.sheets(),
        &[a_sheet.clone(), current_b.id().clone()]
    );
    assert_eq!(
        current_cssom
            .sheet(current_b.id())
            .unwrap()
            .inputs()
            .title
            .as_slice(),
        &[66]
    );
    assert!(
        !current_cssom
            .sheet(current_b.id())
            .unwrap()
            .inputs()
            .alternate
    );
    assert!(!first_cssom.sheet(&a_sheet).unwrap().inputs().disabled);
    let a_disabled = styles.disabled(&session.dom(), a).unwrap();
    let b_disabled = styles.disabled(&session.dom(), b).unwrap();
    assert_eq!(a_disabled.association(), Some(current_a.association()));
    assert_eq!(b_disabled.association(), Some(current_b.association()));
    assert_eq!(a_disabled.cssom_revision(), Some(current_cssom.revision()));
    assert_eq!(b_disabled.cssom_revision(), Some(current_cssom.revision()));
    assert!(!a_disabled.disabled());
    // CSSOM add-sheet steps establish A as preferred, then disable a later
    // non-alternate named sheet whose title does not match that preferred name.
    assert!(
        b_disabled.disabled(),
        "named B must not join the initially enabled A set"
    );
}

use surgeist_cssom::{CssomError, CssomLimits, CssomPublication};
use surgeist_dom::DocumentString;
fn flag() -> AtomicBool {
    AtomicBool::new(false)
}
fn names(session: &mut DomSession, document: NodeId) -> DomStyleSheetSetObservation {
    session
        .style_execution()
        .sheet_sets(&session.dom(), document, &flag())
        .unwrap()
}
fn disabled(session: &mut DomSession, element: NodeId) -> bool {
    session
        .style_execution()
        .disabled(&session.dom(), element)
        .unwrap()
        .disabled()
}
fn drive(session: &mut DomSession, edit: DomSourceEdit) {
    let mut source = session.source_execution();
    assert!(matches!(
        source
            .apply(
                &mut session.dom(),
                edit,
                source_limits(),
                &mut DomSourceServices {
                    execution_scope: None,
                    style_policy: &mut policy,
                    url: None,
                },
                &flag()
            )
            .unwrap(),
        DomSourceDriveProgress::Complete(_)
    ));
}
fn append_title_value(session: &mut DomSession, document: NodeId, title: DocumentString) -> NodeId {
    let mut batch = DomBatch::new();
    let style = batch.create_element_ns(document, Namespace::html(), "style");
    batch.set_attribute(&style, "title", title);
    let commit = session.dom().apply(batch, &flag()).unwrap();
    let style = commit.created(&style).unwrap().unwrap();
    let container = session.snapshot().node(document).unwrap().children()[0];
    drive(
        session,
        DomSourceEdit::insert_before(style, container, None),
    );
    style
}
fn bind_document(session: &mut DomSession, label: &str, producer_limits: CssomLimits) -> NodeId {
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let container = batch.create_element_ns(&document, Namespace::html(), "div");
    batch.insert_before(&container, &document, None);
    let commit = session.dom().apply(batch, &flag()).unwrap();
    let document = commit.created(&document).unwrap().unwrap();
    session
        .style_execution()
        .bind_root(
            &session.dom(),
            label.into(),
            &mut Some(DomStyleCssomRoot::new(
                document,
                CssomStore::new(producer_limits, Default::default()).unwrap(),
            )),
            &flag(),
        )
        .unwrap();
    document
}
#[test]
fn preferred_name_without_sheets_controls_later_real_style_additions() {
    let (mut session, document) = bound_session();
    let mut styles = session.style_execution();
    let initial = names(&mut session, document);
    assert!(initial.preferred_name().is_empty());
    assert!(initial.last_selected_name().is_none());
    assert_eq!(initial.revision(), 0);
    let changed = styles
        .change_preferred_sheet_set_name(&session.dom(), &initial, &"B".into(), 1, &flag())
        .unwrap();
    assert!(
        matches!(changed.cssom().publication(),CssomPublication::Unchanged {revision} if revision==initial.cssom_revision())
    );
    assert_eq!(
        changed.observation().preferred_name(),
        &DocumentString::from("B")
    );
    assert_eq!(changed.observation().revision(), 1);
    assert!(
        styles
            .snapshot(document)
            .unwrap()
            .unwrap()
            .sheets()
            .is_empty()
    );
    let a = append_named_style(&mut session, document, "A");
    let b = append_named_style(&mut session, document, "B");
    assert!(disabled(&mut session, a));
    assert!(!disabled(&mut session, b));
    let current = names(&mut session, document);
    assert_eq!(current.preferred_name(), &DocumentString::from("B"));
    assert!(current.last_selected_name().is_none());
    assert_eq!(
        current.revision(),
        1,
        "sheet identity issuance does not revise preference history"
    );
    assert!(initial.preferred_name().is_empty());
}
#[test]
fn enable_select_and_preferred_change_preserve_distinct_history_and_persistent_flags() {
    let (mut session, document) = bound_session();
    let mut styles = session.style_execution();
    let a = append_named_style(&mut session, document, "A");
    let b = append_named_style(&mut session, document, "B");
    let persistent = append_style(&mut session, document, None);
    let original = names(&mut session, document);
    let enabled = styles
        .enable_sheet_set(&session.dom(), &original, &"B".into(), 1, &flag())
        .unwrap();
    assert_eq!(enabled.observation().revision(), original.revision());
    assert!(enabled.observation().last_selected_name().is_none());
    assert!(disabled(&mut session, a));
    assert!(!disabled(&mut session, b));
    assert!(!disabled(&mut session, persistent));
    let same = styles
        .change_preferred_sheet_set_name(
            &session.dom(),
            enabled.observation(),
            &"A".into(),
            1,
            &flag(),
        )
        .unwrap();
    assert!(matches!(
        same.cssom().publication(),
        CssomPublication::Unchanged { .. }
    ));
    assert!(
        disabled(&mut session, a),
        "same preferred name must not re-enable the set"
    );
    let changed = styles
        .change_preferred_sheet_set_name(
            &session.dom(),
            same.observation(),
            &"B".into(),
            1,
            &flag(),
        )
        .unwrap();
    assert_eq!(
        changed.observation().preferred_name(),
        &DocumentString::from("B")
    );
    let selected = styles
        .select_sheet_set(
            &session.dom(),
            changed.observation(),
            &"A".into(),
            1,
            &flag(),
        )
        .unwrap();
    assert_eq!(
        selected.observation().last_selected_name(),
        Some(&DocumentString::from("A"))
    );
    assert!(!disabled(&mut session, a));
    assert!(disabled(&mut session, b));
    let changed = styles
        .change_preferred_sheet_set_name(
            &session.dom(),
            selected.observation(),
            &"C".into(),
            1,
            &flag(),
        )
        .unwrap();
    assert_eq!(
        changed.observation().preferred_name(),
        &DocumentString::from("C")
    );
    assert!(!disabled(&mut session, a));
    assert!(disabled(&mut session, b));
    let c = append_named_style(&mut session, document, "C");
    assert!(disabled(&mut session, c));
    let a2 = append_named_style(&mut session, document, "A");
    assert!(!disabled(&mut session, a2));
    let basis = names(&mut session, document);
    let empty = styles
        .select_sheet_set(
            &session.dom(),
            &basis,
            &DocumentString::default(),
            1,
            &flag(),
        )
        .unwrap();
    assert!(empty.observation().last_selected_name().unwrap().is_empty());
    for target in [a, b, c, a2] {
        assert!(disabled(&mut session, target));
    }
    assert!(!disabled(&mut session, persistent));
    let changed = styles
        .change_preferred_sheet_set_name(
            &session.dom(),
            empty.observation(),
            &"A".into(),
            1,
            &flag(),
        )
        .unwrap();
    assert!(disabled(&mut session, a));
    let enabled = styles
        .enable_sheet_set(
            &session.dom(),
            changed.observation(),
            &"B".into(),
            1,
            &flag(),
        )
        .unwrap();
    assert!(
        enabled
            .observation()
            .last_selected_name()
            .unwrap()
            .is_empty()
    );
    assert!(!disabled(&mut session, b));
    let later = append_named_style(&mut session, document, "B");
    assert!(disabled(&mut session, later));
}
#[test]
fn raw_utf16_names_are_case_sensitive_without_preprocessing_or_loss() {
    let (mut session, document) = bound_session();
    let mut styles = session.style_execution();
    let upper = DocumentString::from_utf16(vec![0xd800, 0, 65]);
    let lower = DocumentString::from_utf16(vec![0xd800, 0, 97]);
    let initial = names(&mut session, document);
    let changed = styles
        .change_preferred_sheet_set_name(&session.dom(), &initial, &upper, 1, &flag())
        .unwrap();
    assert_eq!(
        changed.observation().preferred_name().code_units(),
        [0xd800, 0, 65]
    );
    let a = append_title_value(&mut session, document, upper.clone());
    let b = append_title_value(&mut session, document, lower.clone());
    assert!(!disabled(&mut session, a));
    assert!(disabled(&mut session, b));
    let current = names(&mut session, document);
    let selected = styles
        .select_sheet_set(&session.dom(), &current, &lower, 1, &flag())
        .unwrap();
    assert_eq!(selected.observation().last_selected_name().unwrap(), &lower);
    assert_eq!(selected.observation().preferred_name(), &upper);
    assert!(disabled(&mut session, a));
    assert!(!disabled(&mut session, b));
}
#[test]
fn explicit_selection_without_sheets_and_last_removal_preserve_document_history() {
    let (mut session, document) = bound_session();
    let mut styles = session.style_execution();
    let initial = names(&mut session, document);
    styles
        .select_sheet_set(&session.dom(), &initial, &"X".into(), 1, &flag())
        .unwrap();
    let a = append_named_style(&mut session, document, "A");
    let x = append_named_style(&mut session, document, "X");
    assert!(disabled(&mut session, a));
    assert!(!disabled(&mut session, x));
    let before = names(&mut session, document);
    assert_eq!(before.preferred_name(), &DocumentString::from("A"));
    drive(&mut session, DomSourceEdit::detach(a));
    drive(&mut session, DomSourceEdit::detach(x));
    assert!(
        styles
            .snapshot(document)
            .unwrap()
            .unwrap()
            .sheets()
            .is_empty()
    );
    let empty = names(&mut session, document);
    assert_eq!(empty.preferred_name(), before.preferred_name());
    assert_eq!(empty.last_selected_name(), before.last_selected_name());
    assert_eq!(empty.revision(), before.revision());
    let container = session.snapshot().node(document).unwrap().children()[0];
    drive(
        &mut session,
        DomSourceEdit::insert_before(a, container, None),
    );
    assert!(disabled(&mut session, a));
}
#[test]
fn direct_flag_edits_stale_the_producer_basis_without_resetting_preference_history() {
    let (mut session, document) = bound_session();
    let mut styles = session.style_execution();
    let a = append_named_style(&mut session, document, "A");
    let before = names(&mut session, document);
    let owner = session.dom().style_cssom_root(document).unwrap().unwrap();
    let snapshot = owner.snapshot().unwrap();
    let mut batch = owner
        .batch(CssomEditGuard::from_snapshot(&snapshot))
        .unwrap();
    batch.set_disabled(&snapshot.sheets()[0], true).unwrap();
    owner.commit(&mut Some(batch)).unwrap();
    assert!(matches!(
        styles.select_sheet_set(&session.dom(), &before, &"A".into(), 1, &flag()),
        Err(DomStyleRuntimeError::StaleSheetSetObservation)
    ));
    let fresh = names(&mut session, document);
    assert_eq!(fresh.preferred_name(), before.preferred_name());
    assert_eq!(fresh.revision(), before.revision());
    assert!(fresh.last_selected_name().is_none());
    assert!(disabled(&mut session, a));
    let same = styles
        .change_preferred_sheet_set_name(&session.dom(), &fresh, &"A".into(), 1, &flag())
        .unwrap();
    assert!(disabled(&mut session, a));
    styles
        .enable_sheet_set(&session.dom(), same.observation(), &"A".into(), 1, &flag())
        .unwrap();
    assert!(!disabled(&mut session, a));
}
#[test]
fn late_producer_refusal_preserves_preference_and_all_affected_flags() {
    let mut session = DomSession::new(dom(), session_limits(limits())).unwrap();
    let document = bind_document(
        &mut session,
        "finite-producer",
        CssomLimits {
            max_revision: 2,
            ..Default::default()
        },
    );
    let mut styles = session.style_execution();
    let a = append_named_style(&mut session, document, "A");
    let b = append_named_style(&mut session, document, "B");
    let before = names(&mut session, document);
    let snapshot = styles.snapshot(document).unwrap().unwrap();
    assert!(matches!(
        styles.change_preferred_sheet_set_name(&session.dom(), &before, &"B".into(), 1, &flag()),
        Err(DomStyleRuntimeError::Resource(
            DomStyleCssomRootError::Cssom(CssomError::RevisionExhausted)
        ))
    ));
    let after = names(&mut session, document);
    assert_eq!(after.preferred_name(), before.preferred_name());
    assert_eq!(after.last_selected_name(), before.last_selected_name());
    assert_eq!(after.revision(), before.revision());
    assert_eq!(after.cssom_revision(), before.cssom_revision());
    assert_eq!(
        styles.snapshot(document).unwrap().unwrap().revision(),
        snapshot.revision()
    );
    assert!(!disabled(&mut session, a));
    assert!(disabled(&mut session, b));
}
#[test]
fn output_storage_work_and_cancellation_refuse_before_preference_publication() {
    let (mut session, document) = bound_session();
    let mut styles = session.style_execution();
    let before = names(&mut session, document);
    assert!(matches!(
        styles.change_preferred_sheet_set_name(&session.dom(), &before, &"B".into(), 0, &flag()),
        Err(DomStyleRuntimeError::SheetSetOutputLimit)
    ));
    let canceled = AtomicBool::new(true);
    assert!(matches!(
        styles.select_sheet_set(&session.dom(), &before, &"B".into(), 1, &canceled),
        Err(DomStyleRuntimeError::Dom(DomError::Cancelled))
    ));
    let current_storage = styles
        .original()
        .unwrap()
        .borrow()
        .storage(None, &flag(), u64::MAX)
        .unwrap();
    let mut low = limits();
    low.storage_bytes = current_storage + 512;
    styles.set_limits(&session.dom(), low, &flag()).unwrap();
    let long = DocumentString::from_utf16(vec![65; 10_000]);
    assert!(matches!(
        styles.change_preferred_sheet_set_name(&session.dom(), &before, &long, 1, &flag()),
        Err(DomStyleRuntimeError::Admission)
    ));
    styles
        .set_limits(&session.dom(), limits(), &flag())
        .unwrap();
    let unchanged = names(&mut session, document);
    assert!(unchanged.preferred_name().is_empty());
    assert_eq!(unchanged.revision(), 0);
    assert_eq!(unchanged.cssom_revision(), before.cssom_revision());
    let changed = styles
        .change_preferred_sheet_set_name(&session.dom(), &before, &long, 1, &flag())
        .unwrap();
    let spent = styles.original().unwrap().borrow().work.get();
    let mut low = limits();
    low.work = spent + 1024;
    styles.set_limits(&session.dom(), low, &flag()).unwrap();
    let mut different = vec![65; 10_000];
    different[9999] = 66;
    let different = DocumentString::from_utf16(different);
    assert!(matches!(
        styles.change_preferred_sheet_set_name(
            &session.dom(),
            changed.observation(),
            &different,
            1,
            &flag()
        ),
        Err(DomStyleRuntimeError::Admission)
    ));
    assert!(styles.original().unwrap().borrow().work.get() > spent);
    styles
        .set_limits(&session.dom(), limits(), &flag())
        .unwrap();
    let unchanged = names(&mut session, document);
    assert_eq!(unchanged.preferred_name(), &long);
    assert_eq!(unchanged.revision(), changed.observation().revision());
    styles
        .change_preferred_sheet_set_name(&session.dom(), &unchanged, &different, 1, &flag())
        .unwrap();
}
#[test]
fn parked_actual_style_source_refuses_independent_preference_calls_until_its_tail_finishes() {
    let (mut session, document) = bound_session();
    let mut styles = session.style_execution();
    let before = names(&mut session, document);
    let mut batch = DomBatch::new();
    let style = batch.create_element_ns(document, Namespace::html(), "style");
    batch.set_attribute(&style, "title", "A");
    let created = session.dom().apply(batch, &flag()).unwrap();
    let style = created.created(&style).unwrap().unwrap();
    let container = session.snapshot().node(document).unwrap().children()[0];
    let mut source = session.source_execution();
    assert!(matches!(
        source
            .apply(
                &mut session.dom(),
                DomSourceEdit::insert_before(style, container, None),
                source_limits(),
                &mut DomSourceServices {
                    execution_scope: None,
                    style_policy: &mut |_| None,
                    url: None
                },
                &flag()
            )
            .unwrap(),
        DomSourceDriveProgress::Pending { .. }
    ));
    let operation = session.dom().pending_source_edit().unwrap().operation();
    let cssom = styles.snapshot(document).unwrap().unwrap();
    assert!(
        matches!(styles.sheet_sets(&session.dom(),document,&flag()),Err(DomStyleRuntimeError::SourcePending(id)) if id==operation)
    );
    assert!(
        matches!(styles.change_preferred_sheet_set_name(&session.dom(),&before,&"B".into(),1,&flag()),Err(DomStyleRuntimeError::SourcePending(id)) if id==operation)
    );
    assert_eq!(
        styles.snapshot(document).unwrap().unwrap().revision(),
        cssom.revision()
    );
    assert!(matches!(
        source
            .resume(
                &mut session.dom(),
                operation,
                source_limits(),
                &mut DomSourceServices {
                    execution_scope: None,
                    style_policy: &mut policy,
                    url: None
                },
                &flag()
            )
            .unwrap(),
        DomSourceDriveProgress::Complete(_)
    ));
    let after = names(&mut session, document);
    assert_eq!(after.preferred_name(), &DocumentString::from("A"));
    assert!(after.last_selected_name().is_none());
    assert!(!disabled(&mut session, style));
}
#[test]
fn document_adoption_keeps_each_preference_and_shadow_lists_have_no_document_history() {
    let (mut session, first) = bound_session();
    let mut styles = session.style_execution();
    let second = bind_document(&mut session, "second-sets", Default::default());
    let first_style = append_named_style(&mut session, first, "A");
    let second_initial = names(&mut session, second);
    styles
        .change_preferred_sheet_set_name(&session.dom(), &second_initial, &"B".into(), 1, &flag())
        .unwrap();
    let second_container = session.snapshot().node(second).unwrap().children()[0];
    drive(
        &mut session,
        DomSourceEdit::insert_before(first_style, second_container, None),
    );
    assert!(disabled(&mut session, first_style));
    assert_eq!(
        names(&mut session, first).preferred_name(),
        &DocumentString::from("A")
    );
    assert_eq!(
        names(&mut session, second).preferred_name(),
        &DocumentString::from("B")
    );
    assert!(styles.snapshot(first).unwrap().unwrap().sheets().is_empty());
    let b = append_named_style(&mut session, second, "B");
    assert!(!disabled(&mut session, b));
    let first_b = append_named_style(&mut session, first, "B");
    assert!(disabled(&mut session, first_b));
    let mut batch = DomBatch::new();
    let host = batch.create_element_ns(first, Namespace::html(), "div");
    let shadow = batch.attach_shadow(
        &host,
        surgeist_dom::DomShadowRootInit::new(surgeist_html::HtmlShadowRootMode::Closed),
    );
    batch.insert_before(
        &host,
        session.snapshot().node(first).unwrap().children()[0],
        None,
    );
    let style = batch.create_element_ns(first, Namespace::html(), "style");
    batch.set_attribute(&style, "title", "ShadowTitle");
    let made = session.dom().apply(batch, &flag()).unwrap();
    let shadow = made.created(&shadow).unwrap().unwrap();
    let style = made.created(&style).unwrap().unwrap();
    styles
        .bind_root(
            &session.dom(),
            "shadow-sets".into(),
            &mut Some(DomStyleCssomRoot::new(
                shadow,
                CssomStore::new(Default::default(), Default::default()).unwrap(),
            )),
            &flag(),
        )
        .unwrap();
    drive(
        &mut session,
        DomSourceEdit::insert_before(style, shadow, None),
    );
    assert!(
        matches!(styles.sheet_sets(&session.dom(),shadow,&flag()),Err(DomStyleRuntimeError::SheetSetDocumentRequired {root}) if root==shadow)
    );
    let shadow_capture = styles.snapshot(shadow).unwrap().unwrap();
    let sheet = shadow_capture.sheet(&shadow_capture.sheets()[0]).unwrap();
    assert!(sheet.inputs().title.is_empty());
    assert!(!sheet.inputs().disabled);
    let basis = names(&mut session, first);
    styles
        .select_sheet_set(
            &session.dom(),
            &basis,
            &DocumentString::default(),
            1,
            &flag(),
        )
        .unwrap();
    assert!(!disabled(&mut session, style));
}
#[test]
fn missing_owner_foreign_session_and_unsupported_document_retirement_preserve_history() {
    let (mut session, document) = bound_session();
    let mut styles = session.style_execution();
    let mut batch = DomBatch::new();
    let unbound = batch.create_document();
    let made = session.dom().apply(batch, &flag()).unwrap();
    let unbound = made.created(&unbound).unwrap().unwrap();
    assert!(
        matches!(styles.sheet_sets(&session.dom(),unbound,&flag()),Err(DomStyleRuntimeError::SheetSetRootUnavailable {root}) if root==unbound)
    );
    let (mut other, other_doc) = bound_session();
    assert!(matches!(
        styles.sheet_sets(&other.dom(), other_doc, &flag()),
        Err(DomStyleRuntimeError::WrongSession)
    ));
    let basis = names(&mut session, document);
    let mut other_styles = other.style_execution();
    assert!(matches!(
        other_styles.select_sheet_set(&other.dom(), &basis, &"A".into(), 1, &flag()),
        Err(DomStyleRuntimeError::Dom(DomError::ForeignOwner))
    ));
    let temporary = bind_document(&mut session, "retire-sets", Default::default());
    let initial = names(&mut session, temporary);
    let changed = styles
        .select_sheet_set(&session.dom(), &initial, &"Old".into(), 1, &flag())
        .unwrap();
    let owner_before = styles.snapshot(temporary).unwrap().unwrap();
    let mut batch = DomBatch::new();
    batch.destroy(temporary);
    assert!(matches!(
        session.dom().apply(batch, &flag()),
        Err(DomError::InvalidDestruction)
    ));
    let retained = names(&mut session, temporary);
    assert_eq!(
        retained.last_selected_name(),
        changed.observation().last_selected_name()
    );
    assert_eq!(retained.revision(), changed.observation().revision());
    assert_eq!(
        styles.snapshot(temporary).unwrap().unwrap().revision(),
        owner_before.revision()
    );
    let another = bind_document(&mut session, "another-sets", Default::default());
    assert_ne!(another, temporary);
    let fresh = names(&mut session, another);
    assert!(fresh.preferred_name().is_empty());
    assert!(fresh.last_selected_name().is_none());
    assert_eq!(fresh.revision(), 0);
    assert!(matches!(
        styles.select_sheet_set(
            &session.dom(),
            changed.observation(),
            &"Wrong".into(),
            1,
            &flag()
        ),
        Err(DomStyleRuntimeError::StaleSheetSetObservation)
    ));
    assert_eq!(
        names(&mut session, temporary).last_selected_name(),
        Some(&DocumentString::from("Old"))
    );
}
// Real external sheet construction exercises add-sheet branches for alternate
// and initially disabled producer flags. It grants no Link/loader capability.
fn add_external_sheet(
    session: &mut DomSession,
    document: NodeId,
    title: &str,
    alternate: bool,
    disabled: bool,
) -> CssomSheetId {
    let styles = session.style_execution();
    let owner = styles.original().unwrap();
    let mut state = owner.borrow_mut();
    let index = state
        .roots
        .iter()
        .position(|root| root.node == document)
        .unwrap();
    let root = state.roots[index].store.clone();
    let snapshot = root.snapshot().unwrap();
    let mut batch = root
        .batch(CssomEditGuard::from_snapshot(&snapshot))
        .unwrap();
    let mut inputs = CssomSheetInputs::external(
        CssomInputVersion {
            role: CssomInputRole::Owner,
            identity: "actual-external-set-owner".into(),
            revision: 1,
        },
        true,
    );
    inputs.title = Arc::new(title.encode_utf16().collect());
    inputs.alternate = alternate;
    inputs.disabled = disabled;
    let next = state
        .prepare_added_sheet_sets(index, &snapshot, &mut batch, &mut inputs, &flag())
        .unwrap();
    let ticket = batch
        .create_parsed_sheet_report(CssParserContext::default().parse_sheet(""), inputs)
        .unwrap();
    let commit = root.commit(&mut Some(batch)).unwrap();
    if let Some(next) = next {
        state.roots[index].sheet_sets.publish(next);
    }
    commit.sheet(&ticket).unwrap().clone()
}
#[test]
fn alternate_and_initially_disabled_additions_follow_actual_producer_flags() {
    let (mut session, document) = bound_session();
    let mut styles = session.style_execution();
    let alternate = add_external_sheet(&mut session, document, "A", true, false);
    assert!(names(&mut session, document).preferred_name().is_empty());
    assert!(
        styles
            .snapshot(document)
            .unwrap()
            .unwrap()
            .sheet(&alternate)
            .unwrap()
            .inputs()
            .disabled
    );
    let predisabled = add_external_sheet(&mut session, document, "Other", false, true);
    assert!(names(&mut session, document).preferred_name().is_empty());
    let preferred = add_external_sheet(&mut session, document, "A", false, false);
    let current = styles.snapshot(document).unwrap().unwrap();
    assert!(
        !current.sheet(&alternate).unwrap().inputs().disabled,
        "preferred selection enables the existing matching alternate"
    );
    assert!(!current.sheet(&preferred).unwrap().inputs().disabled);
    assert!(current.sheet(&predisabled).unwrap().inputs().disabled);
    let persistent = add_external_sheet(&mut session, document, "", false, false);
    let basis = names(&mut session, document);
    styles
        .select_sheet_set(
            &session.dom(),
            &basis,
            &DocumentString::default(),
            1,
            &flag(),
        )
        .unwrap();
    let current = styles.snapshot(document).unwrap().unwrap();
    assert!(!current.sheet(&persistent).unwrap().inputs().disabled);
    assert!(current.sheet(&alternate).unwrap().inputs().disabled);
    assert!(current.sheet(&preferred).unwrap().inputs().disabled);
}
#[test]
fn retired_shadow_generation_releases_its_root_without_changing_document_preference() {
    let (mut session, document) = bound_session();
    let mut styles = session.style_execution();
    let a = append_named_style(&mut session, document, "A");
    let before = names(&mut session, document);
    let mut batch = DomBatch::new();
    let host = batch.create_element_ns(document, Namespace::html(), "div");
    let shadow = batch.attach_shadow(
        &host,
        surgeist_dom::DomShadowRootInit::new(surgeist_html::HtmlShadowRootMode::Closed),
    );
    batch.insert_before(
        &host,
        session.snapshot().node(document).unwrap().children()[0],
        None,
    );
    let made = session.dom().apply(batch, &flag()).unwrap();
    let host = made.created(&host).unwrap().unwrap();
    let shadow = made.created(&shadow).unwrap().unwrap();
    styles
        .bind_root(
            &session.dom(),
            "retired-set-root".into(),
            &mut Some(DomStyleCssomRoot::new(
                shadow,
                CssomStore::new(Default::default(), Default::default()).unwrap(),
            )),
            &flag(),
        )
        .unwrap();
    drive(&mut session, DomSourceEdit::detach(host));
    drive(&mut session, DomSourceEdit::destroy(host));
    assert!(styles.snapshot(shadow).unwrap().is_none());
    assert!(matches!(
        styles.sheet_sets(&session.dom(), shadow, &flag()),
        Err(DomStyleRuntimeError::Dom(DomError::StaleNode))
    ));
    let mut batch = DomBatch::new();
    let host = batch.create_element_ns(document, Namespace::html(), "div");
    let replacement = batch.attach_shadow(
        &host,
        surgeist_dom::DomShadowRootInit::new(surgeist_html::HtmlShadowRootMode::Closed),
    );
    let made = session.dom().apply(batch, &flag()).unwrap();
    let replacement = made.created(&replacement).unwrap().unwrap();
    assert_ne!(shadow, replacement);
    styles
        .bind_root(
            &session.dom(),
            "retired-set-root".into(),
            &mut Some(DomStyleCssomRoot::new(
                replacement,
                CssomStore::new(Default::default(), Default::default()).unwrap(),
            )),
            &flag(),
        )
        .unwrap();
    assert!(
        matches!(styles.sheet_sets(&session.dom(),replacement,&flag()),Err(DomStyleRuntimeError::SheetSetDocumentRequired {root}) if root==replacement)
    );
    let after = names(&mut session, document);
    assert_eq!(after.preferred_name(), before.preferred_name());
    assert_eq!(after.last_selected_name(), before.last_selected_name());
    assert_eq!(after.revision(), before.revision());
    assert!(!disabled(&mut session, a));
}
#[test]
fn actual_networking_callback_can_select_a_set_without_bypassing_source_custody() {
    let (mut session, document) = bound_dispatch_session();
    let a = append_named_style(&mut session, document, "A");
    let b = append_named_style(&mut session, document, "B");
    let mut context = Context {
        styles: session.style_execution(),
        source: session.source_execution(),
        events: Vec::new(),
    };
    let mut dispatcher = DomEventDispatcher::new(DomEventRegistryLimits {
        registrations: 4,
        pending_events: 2,
        nested_dispatches: 2,
    })
    .unwrap();
    dispatcher
        .add_listener(
            &mut session.dom(),
            a,
            DomEventType::Load,
            DomEventCallback::new(move |dom, _, context: &mut Context, event| {
                let basis = context.styles.sheet_sets(dom, document, &flag()).unwrap();
                context
                    .styles
                    .select_sheet_set(dom, &basis, &"B".into(), 1, &flag())
                    .unwrap();
                context.events.push((
                    event.target().unwrap(),
                    event.current_target().unwrap().node().unwrap(),
                ));
                Ok(())
            }),
            DomEventListenerOptions::default(),
        )
        .unwrap();
    let DomStyleNetworkingProgress::Complete { receipt, .. } = DomStyleRuntime::run_networking(
        &mut session.dom(),
        &mut dispatcher,
        &mut context,
        dispatch_limits(),
        &flag(),
    )
    .unwrap() else {
        panic!("actual first Load must complete")
    };
    assert_eq!(receipt.callbacks(), 1);
    assert_eq!(context.events, [(a, a)]);
    assert!(disabled(&mut session, a));
    assert!(!disabled(&mut session, b));
    let current = names(&mut session, document);
    assert_eq!(current.preferred_name(), &DocumentString::from("A"));
    assert_eq!(
        current.last_selected_name(),
        Some(&DocumentString::from("B"))
    );
}
