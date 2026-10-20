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
