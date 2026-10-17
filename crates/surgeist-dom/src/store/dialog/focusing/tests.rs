//! Native fact owners only. These tests do not fabricate a Window, active
//! Document, focusing receipt or a completed native Dialog method.
use super::*;

fn limits(work: u64) -> crate::DomSourceLimits {
    crate::DomSourceLimits::new(32_000_000, 1_000_000, work, 32, 4096, 4096, 2)
}
fn fixture(root_dialog: bool) -> (Dom, NodeId, NodeId) {
    let mut dom =
        Dom::new(DomLimits::new(64, 100_000, 128, 128, 128, 16, 1_000_000).unwrap()).unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let subject = batch.create_element_ns(&document, crate::Namespace::html(), "dialog");
    let other = batch.create_element_ns(&document, crate::Namespace::html(), "dialog");
    if root_dialog {
        batch.insert_before(&subject, &document, None);
    } else {
        let root = batch.create_element_ns(&document, crate::Namespace::html(), "div");
        batch.insert_before(&root, &document, None);
        batch.insert_before(&subject, &root, None);
    }
    batch.set_attribute(&subject, "inert", "noncanonical Boolean spelling");
    batch.set_attribute(&subject, "autofocus", "");
    batch.set_attribute_ns(
        &other,
        crate::Namespace::from_uri("http://www.w3.org/XML/1998/namespace"),
        "xml:inert",
        "",
    );
    batch.set_attribute_ns(
        &other,
        crate::Namespace::from_uri("http://www.w3.org/XML/1998/namespace"),
        "xml:autofocus",
        "",
    );
    let commit = dom.apply(batch, &AtomicBool::new(false)).unwrap();
    (
        dom,
        commit.created(&subject).unwrap().unwrap(),
        commit.created(&other).unwrap().unwrap(),
    )
}
fn draft(dom: &Dom) -> Draft {
    let batch = DomBatch::new();
    Draft::new_source(
        dom,
        batch.owner.clone(),
        &batch.commands,
        limits(10_000_000),
        &AtomicBool::new(false),
    )
    .unwrap()
    .0
}

#[test]
fn autofocus_and_own_inert_consume_real_boolean_intrinsics_and_exact_attribute_namespace() {
    let cancel = AtomicBool::new(false);
    let (dom, subject, other) = fixture(false);
    let staged = draft(&dom);
    let subject = staged.resolve(subject.into()).unwrap();
    let other = staged.resolve(other.into()).unwrap();
    for (selected, attribute) in [
        (
            surgeist_html::HtmlBooleanAttribute::Autofocus,
            KnownHtmlAttribute::Autofocus,
        ),
        (
            surgeist_html::HtmlBooleanAttribute::Inert,
            KnownHtmlAttribute::Inert,
        ),
    ] {
        assert!(
            staged
                .focus_boolean(subject, selected, attribute, &cancel)
                .unwrap()
        );
        assert!(
            !staged
                .focus_boolean(other, selected, attribute, &cancel)
                .unwrap()
        );
    }
    assert!(
        staged
            .dialog_subject_focus_no_change(subject, &cancel)
            .unwrap()
    );
    assert!(
        !staged
            .dialog_subject_focus_no_change(other, &cancel)
            .unwrap()
    );
    let (root_dom, root, _) = fixture(true);
    let root_staged = draft(&root_dom);
    assert!(
        !root_staged
            .dialog_subject_focus_no_change(root_staged.resolve(root.into()).unwrap(), &cancel)
            .unwrap()
    );
    assert_eq!(staged.source_formation_bytes.load(Ordering::Relaxed), 0);
    assert_eq!(staged.source_temporary_units.load(Ordering::Relaxed), 0);
}

#[test]
fn absent_window_context_never_becomes_permission_denial_or_a_top_document() {
    let (dom, subject, _) = fixture(false);
    let staged = draft(&dom);
    let subject = staged.resolve(subject.into()).unwrap();
    assert!(matches!(
        staged
            .dialog_allow_focus_permission(subject, &AtomicBool::new(false))
            .unwrap(),
        AllowFocusPermission::Unavailable
    ));
    assert_eq!(
        staged
            .dialog_control_top_document(subject, &AtomicBool::new(false))
            .unwrap(),
        None
    );
}

#[test]
fn actual_boolean_report_work_refusal_and_cancellation_release_the_original_formation() {
    let (dom, subject, _) = fixture(false);
    let cancel = AtomicBool::new(false);
    let measured = draft(&dom);
    let key = measured.resolve(subject.into()).unwrap();
    let before = measured.source_work.load(Ordering::Relaxed);
    assert!(
        measured
            .focus_boolean(
                key,
                surgeist_html::HtmlBooleanAttribute::Inert,
                KnownHtmlAttribute::Inert,
                &cancel
            )
            .unwrap()
    );
    // This is an observed owning admission boundary, not an independent
    // numerical tariff: the same input must refuse one short and retry intact.
    let work = measured.source_work.load(Ordering::Relaxed) - before;
    assert!(work > 1);
    let mut staged = draft(&dom);
    let key = staged.resolve(subject.into()).unwrap();
    let baseline = staged.source_work.load(Ordering::Relaxed);
    staged.source_limits = Some(limits(baseline + work - 1));
    assert_eq!(
        staged.focus_boolean(
            key,
            surgeist_html::HtmlBooleanAttribute::Inert,
            KnownHtmlAttribute::Inert,
            &cancel
        ),
        Err(DomError::SourceWorkLimit {
            limit: baseline + work - 1
        })
    );
    assert_eq!(staged.source_formation_bytes.load(Ordering::Relaxed), 0);
    assert_eq!(staged.source_temporary_units.load(Ordering::Relaxed), 0);
    let spent = staged.source_work.load(Ordering::Relaxed);
    assert_eq!(
        staged.focus_boolean(
            key,
            surgeist_html::HtmlBooleanAttribute::Inert,
            KnownHtmlAttribute::Inert,
            &AtomicBool::new(true)
        ),
        Err(DomError::Cancelled)
    );
    assert_eq!(staged.source_work.load(Ordering::Relaxed), spent);
    staged.source_limits = Some(limits(10_000_000));
    assert!(
        staged
            .focus_boolean(
                key,
                surgeist_html::HtmlBooleanAttribute::Inert,
                KnownHtmlAttribute::Inert,
                &cancel
            )
            .unwrap()
    );
    assert_eq!(staged.source_formation_bytes.load(Ordering::Relaxed), 0);
    assert_eq!(staged.source_temporary_units.load(Ordering::Relaxed), 0);
}
