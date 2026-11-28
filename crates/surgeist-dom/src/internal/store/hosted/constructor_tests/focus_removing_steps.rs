use super::*;
use crate::internal::{
    document::DomFocusedAreaDesignation::{Element, Viewport},
    source::{DomSourceEdit, DomSourceLimits, DomSourceProgress, DomSourceRequest},
};
use surgeist_html::{
    security::csp::{
        HtmlCspError, HtmlCspLayout, HtmlCspProcessorAuthority, HtmlCspProcessorProfile,
        HtmlCspStorage, HtmlMetaCspOutcome, HtmlMetaCspProcessor, HtmlMetaCspRequest,
    },
    values::Namespace,
};

struct Fixture {
    dom: Dom,
    _source: DomHostedSource,
    window: DomHostedWindowNavigation,
    document: DocumentId,
    parent: NodeId,
    designated: NodeId,
    unselected: NodeId,
}

fn fixture() -> Fixture {
    let cancel = cancel();
    let (mut dom, mut source, mut prepared) = prepared_top_level();
    let (mut lease, document, _, window) =
        publish_local_html_entry(&mut dom, &mut source, &mut prepared, true);
    lease.finish(&mut dom, &mut source, &cancel).unwrap();

    let mut batch = DomBatch::new();
    let parent = batch.create_element_ns(document, Namespace::html(), "head");
    let designated = batch.create_element_ns(document, Namespace::html(), "input");
    let unselected = batch.create_element_ns(document, Namespace::html(), "div");
    batch.insert_before(&parent, document, None);
    batch.insert_before(&designated, &parent, None);
    batch.insert_before(&unselected, &parent, None);
    let created = dom.apply(batch, &cancel).unwrap();
    let parent = created.created(&parent).unwrap().unwrap();
    let designated = created.created(&designated).unwrap().unwrap();
    let unselected = created.created(&unselected).unwrap().unwrap();
    // Seed only the target of an earlier focus update. Viewport provenance,
    // Window identity and all removing steps use the actual hosting path.
    dom.edit_native_state(&cancel, |draft| {
        let mut designation =
            draft.focused_area_designation_mut(Key::Existing(document.node()), &cancel)?;
        *designation = Element(designated);
        designation.commit(&cancel)
    })
    .unwrap();
    Fixture {
        dom,
        _source: source,
        window: window.unwrap(),
        document,
        parent,
        designated,
        unselected,
    }
}

#[test]
fn issuance_of_a_window_invalidates_the_earlier_native_seal_without_one() {
    let cancel = cancel();
    let (mut dom, mut source, mut prepared) = prepared_top_level();
    let mut lease = begin_local_html_execution(&mut dom, &mut source, &mut prepared, &cancel);
    let DomHostedCreationStep::NativeExecution(request) = lease.step(&dom).unwrap() else {
        panic!("expected execution request");
    };
    let global = source.issue_global(request, &cancel).unwrap();
    let mut outdated = Some(
        source
            .seal_native_execution(request, global, 100, None, &cancel)
            .unwrap(),
    );
    let window = source
        .create_window_navigation(request, global, &cancel)
        .unwrap();
    let mut current = Some(
        source
            .seal_native_execution(request, global, 100, Some(&window), &cancel)
            .unwrap(),
    );
    let before = dom.snapshot();
    assert_eq!(
        lease.accept_publication(&mut dom, &mut source, &mut outdated, &cancel),
        Err(DomHostedError::WrongPhase)
    );
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    assert!(outdated.is_some());
    lease
        .accept_publication(&mut dom, &mut source, &mut current, &cancel)
        .unwrap();
    assert!(current.is_none());
}

struct UnchangedPolicy {
    authority: HtmlCspProcessorAuthority,
    profile: HtmlCspProcessorProfile,
}

impl HtmlMetaCspProcessor for UnchangedPolicy {
    type Error = ();

    fn profile(&self) -> &HtmlCspProcessorProfile {
        &self.profile
    }

    fn prepare<E>(
        &mut self,
        request: &HtmlMetaCspRequest,
        allow: &mut impl FnMut(HtmlCspLayout) -> Result<(), E>,
        _release: &mut impl FnMut(HtmlCspStorage),
    ) -> Result<HtmlMetaCspOutcome<()>, HtmlCspError<E>> {
        self.authority
            .attest_unchanged_with(request, allow)
            .map(HtmlMetaCspOutcome::Completed)
    }
}

#[test]
fn private_replacement_revalidates_its_window_issuer_after_policy_processing() {
    let cancel = cancel();
    let Fixture {
        mut dom,
        _source,
        window,
        document,
        parent,
        designated,
        ..
    } = fixture();
    dom.set_limits(dom.limits().with_meta_processing_work(1_000_000))
        .unwrap();
    window.set_focus_changed_for_test();
    let mut setup = DomBatch::new();
    let meta = setup.create_element_ns(document, Namespace::html(), "meta");
    setup.set_attribute(&meta, "http-equiv", "content-security-policy");
    setup.set_attribute(&meta, "content", "default-src 'none'");
    let meta = dom
        .apply(setup, &cancel)
        .unwrap()
        .created(&meta)
        .unwrap()
        .unwrap();
    let before = dom.snapshot();
    let limits = DomSourceLimits::new(16_000_000, 200_000, 20_000, 64, 64, 64, 2);
    let mut lease = dom
        .begin_source_edit(
            DomSourceEdit::replace_child(meta, parent, designated),
            limits,
            &cancel,
        )
        .unwrap();
    let DomSourceProgress::Request(DomSourceRequest::MetaCsp(request)) =
        lease.advance(&cancel).unwrap()
    else {
        panic!("the inserted Meta reaches its policy processor after staging the removal");
    };
    let mut allow = |_: HtmlCspLayout| Ok::<_, ()>(());
    let authority = HtmlCspProcessorAuthority::new_with(&mut allow).unwrap();
    let profile = authority.profile_with(1, &mut allow).unwrap();
    let mut processor = UnchangedPolicy { authority, profile };
    drop(_source);
    lease
        .process_meta_csp(request, &mut processor, &cancel)
        .unwrap();
    assert_eq!(
        lease.advance(&cancel).unwrap_err(),
        DomError::HostedSourceUnavailable
    );
    let after = lease.snapshot();
    assert_eq!(after.stamp(), before.stamp());
    assert_eq!(after.node(designated).unwrap().parent(), Some(parent));
    assert!(after.node(meta).unwrap().parent().is_none());
    assert_eq!(
        after.document(document).unwrap().focused_area_designation(),
        Element(designated)
    );
    assert!(window.focus_changed());
}

#[test]
fn completed_hosted_teardown_preserves_the_retained_windows_true_flag() {
    let cancel = cancel();
    let (mut dom, mut source, mut prepared) = prepared_top_level();
    let (mut lease, document, global, window) =
        publish_local_html_entry(&mut dom, &mut source, &mut prepared, true);
    let creation = lease.finish(&mut dom, &mut source, &cancel).unwrap();
    let window = window.unwrap();
    window.set_focus_changed_for_test();
    let binding = |snapshot: &DomSnapshot| {
        snapshot
            .document(document)
            .unwrap()
            .metadata()
            .document_environment()
            .hosted
            .unwrap()
            .window_navigation
            .as_ref()
            .unwrap()
            .focus_changed
            .clone()
    };
    let original = binding(&dom.snapshot());
    let mut invocation = dom
        .begin_hosted_top_level_destruction(&mut source, creation.navigable(), &cancel)
        .unwrap();
    finish_local_html_destruction(&mut dom, &mut source, &mut invocation, &cancel);
    assert_eq!(
        dom.global_document(global),
        Err(DomHostedError::StaleIdentity)
    );
    assert!(original.ptr_eq(&binding(&dom.snapshot())));
    assert!(window.focus_changed());
    drop(window);
    assert!(original.upgrade().is_none());
}

#[test]
fn atomic_detach_resets_the_current_designation_and_its_original_window() {
    let cancel = cancel();
    let Fixture {
        mut dom,
        _source,
        window,
        document,
        parent,
        designated,
        unselected,
    } = fixture();
    let unrelated = fixture();
    unrelated.window.set_focus_changed_for_test();
    assert!(!window.focus_changed());
    window.set_focus_changed_for_test();
    let older = dom.snapshot();
    let mut other = DomBatch::new();
    other.detach(unselected);
    dom.apply(other, &cancel).unwrap();
    assert!(window.focus_changed());
    let mut removal = DomBatch::new();
    removal.detach(designated);
    let committed = dom.apply(removal, &cancel).unwrap();
    let after = dom.snapshot();
    assert!(after.node(designated).unwrap().parent().is_none());
    assert_eq!(
        after.document(document).unwrap().focused_area_designation(),
        Viewport
    );
    assert!(
        committed
            .changes()
            .changes()
            .contains(&DomChange::DocumentFocusDesignation(document))
    );
    assert!(!window.focus_changed());
    assert!(unrelated.window.focus_changed());
    assert_eq!(older.node(designated).unwrap().parent(), Some(parent));
    assert_eq!(
        older.document(document).unwrap().focused_area_designation(),
        Element(designated)
    );
}

#[test]
fn cancelled_publication_leaves_the_removal_and_both_focus_writes_private() {
    let cancel = cancel();
    let Fixture {
        mut dom,
        _source,
        window,
        document,
        parent,
        designated,
        ..
    } = fixture();
    window.set_focus_changed_for_test();
    let before = dom.snapshot();
    let mut removal = DomBatch::new();
    removal.detach(designated);
    let prepared = dom.prepare(removal, &cancel).unwrap();
    assert!(matches!(
        dom.publish(prepared, &AtomicBool::new(true)),
        Err(DomError::Cancelled)
    ));
    let after = dom.snapshot();
    assert_eq!(after.stamp(), before.stamp());
    assert_eq!(after.node(designated).unwrap().parent(), Some(parent));
    assert_eq!(
        after.document(document).unwrap().focused_area_designation(),
        Element(designated)
    );
    assert!(window.focus_changed());
    let mut retry = DomBatch::new();
    retry.detach(designated);
    dom.apply(retry, &cancel).unwrap();
    assert!(!window.focus_changed());
}

#[test]
fn source_detach_prepares_the_required_reset_even_when_the_flag_is_already_false() {
    let cancel = cancel();
    let Fixture {
        mut dom,
        _source,
        window,
        document,
        designated,
        ..
    } = fixture();
    assert!(!window.focus_changed());
    let limits = DomSourceLimits::new(16_000_000, 200_000, 20_000, 64, 64, 64, 2);
    let DomSourceProgress::Complete(done) = dom
        .begin_source_edit(DomSourceEdit::detach(designated), limits, &cancel)
        .unwrap()
        .advance(&cancel)
        .unwrap()
    else {
        panic!("focus fixup is synchronous and needs no event or service continuation");
    };
    assert!(
        done.commit()
            .changes()
            .changes()
            .contains(&DomChange::DocumentFocusDesignation(document))
    );
    assert_eq!(
        dom.snapshot()
            .document(document)
            .unwrap()
            .focused_area_designation(),
        Viewport
    );
    assert!(!window.focus_changed());
}

#[test]
fn dropping_the_original_window_refuses_the_whole_focus_removal() {
    let cancel = cancel();
    let Fixture {
        mut dom,
        _source,
        window,
        document,
        parent,
        designated,
        ..
    } = fixture();
    assert!(!window.focus_changed());
    let before = dom.snapshot();
    drop(window);
    let mut removal = DomBatch::new();
    removal.detach(designated);
    assert_eq!(
        dom.apply(removal, &cancel).unwrap_err(),
        DomError::HostedSourceUnavailable
    );
    let after = dom.snapshot();
    assert_eq!(after.stamp(), before.stamp());
    assert_eq!(after.node(designated).unwrap().parent(), Some(parent));
    assert_eq!(
        after.document(document).unwrap().focused_area_designation(),
        Element(designated)
    );
}
