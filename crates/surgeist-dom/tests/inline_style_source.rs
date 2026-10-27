//! Authentic single-root Source synchronization and continuation ownership.
use std::{cell::Cell, sync::atomic::AtomicBool};
use surgeist_css::CssParserContext;
use surgeist_css::property::CssPropertyGrammar;
use surgeist_css::source::CssUtf16InputLimits;
use surgeist_cssom::*;
use surgeist_dom::*;
fn cancel() -> AtomicBool {
    AtomicBool::new(false)
}
fn limits() -> DomSourceLimits {
    DomSourceLimits::new(64_000_000, 1_000_000, 20_000_000, 128, 8192, 8192, 4)
}
fn requests() -> DomInlineStyleRequestLimits {
    DomInlineStyleRequestLimits {
        input: CssUtf16InputLimits::default(),
        declaration: CssomDeclarationRequestLimits::default(),
    }
}
fn support(decide: bool) -> CssomDeclarationSupport {
    CssomDeclarationSupport::try_new(vec![(
        CssPropertyGrammar::from_name("width").unwrap(),
        if decide {
            CssomUsabilityRequirement::WholeDeclarationDecision
        } else {
            CssomUsabilityRequirement::AllCheckedValues
        },
    )])
    .unwrap()
}
fn version(decide: bool) -> CssomInputVersion {
    CssomInputVersion {
        role: CssomInputRole::Support,
        identity: format!("source-width:{decide}"),
        revision: 1,
    }
}
fn row(element: NodeId, document: DocumentId, decide: bool) -> DomInlineStyleProfile {
    DomInlineStyleProfile::new(
        element,
        document,
        CssParserContext::default(),
        version(decide),
        support(decide),
    )
    .unwrap()
}
fn context(dom: &Dom, rows: Vec<DomInlineStyleProfile>) -> DomInlineStyleSourceContext {
    DomInlineStyleSourceContext::new(dom.snapshot().stamp(), rows).unwrap()
}
fn admission(
    c: &[CssomDeclarationCandidate],
    ctx: &CssomContext,
) -> Result<CssomAdmission, CssomError> {
    let support = ctx
        .inputs
        .iter()
        .find(|i| i.version.role == CssomInputRole::Support)
        .unwrap()
        .version
        .clone();
    Ok(CssomAdmission::new(
        c.iter()
            .map(|c| {
                c.decide(support.clone(), CssomDeclarationUsability::Usable)
                    .unwrap()
            })
            .collect(),
        ctx.inputs.iter().map(|i| i.version.clone()).collect(),
    ))
}
fn no_decisions(
    _: &[CssomDeclarationCandidate],
    _: &CssomContext,
) -> Result<CssomAdmission, CssomError> {
    panic!("no external declaration decision")
}
struct Fixture {
    dom: Dom,
    a: DocumentId,
    b: DocumentId,
    root: NodeId,
    first: NodeId,
    second: NodeId,
}
fn fixture() -> Fixture {
    fixture_with_bindings(true)
}
fn fixture_with_bindings(bind: bool) -> Fixture {
    let mut dom = Dom::new(
        DomLimits::new(
            128,
            100_000,
            256,
            128,
            128,
            32,
            4_000_000,
            DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
        )
        .unwrap()
        .with_base_processing(64, 1_000_000)
        .unwrap()
        .with_dispatch_custody(
            DomDispatchCustodyLimits::new(64, 4096, 16_000_000, 16_000_000, 4_000_000).unwrap(),
        ),
    )
    .unwrap();
    let mut batch = DomBatch::new();
    let a = batch.create_document();
    let b = batch.create_document();
    let root = batch.create_element_ns(&a, Namespace::html(), "main");
    let first = batch.create_element_ns(&a, Namespace::html(), "div");
    let second = batch.create_element_ns(&a, Namespace::html(), "span");
    batch.insert_before(&first, &root, None);
    batch.insert_before(&second, &root, None);
    let commit = dom.apply(batch, &cancel()).unwrap();
    let mut f = Fixture {
        a: commit.document(&a).unwrap(),
        b: commit.document(&b).unwrap(),
        root: commit.created(&root).unwrap().unwrap(),
        first: commit.created(&first).unwrap().unwrap(),
        second: commit.created(&second).unwrap().unwrap(),
        dom,
    };
    f.dom
        .configure_inline_styles(
            DomInlineStyleLimits::new(CssomLimits::default(), 32, 64_000_000, 8_000_000, 4_000_000)
                .unwrap(),
            &cancel(),
        )
        .unwrap();
    for element in [f.first, f.second].into_iter().filter(|_| bind) {
        let inputs = DomInlineStyleInputs::new(
            f.dom.snapshot().document(f.a).unwrap().stamp(),
            CssParserContext::default(),
            version(false),
            support(false),
        )
        .unwrap();
        f.dom
            .inline_style(element, &inputs, requests(), no_decisions, &cancel())
            .unwrap();
    }
    f
}
fn capture(dom: &Dom, node: NodeId) -> DomInlineStyleCapture {
    dom.capture_inline_style(node).unwrap().unwrap()
}
fn width(c: &DomInlineStyleCapture) -> String {
    c.properties()
        .get_property_value("width", requests().declaration)
        .unwrap()
}

#[test]
fn real_setter_returns_paired_original_publication_and_preserves_utf16() {
    let mut f = fixture();
    let old = capture(&f.dom, f.first);
    let before = f.dom.snapshot().stamp();
    let raw = DocumentString::from_utf16(vec![
        119, 105, 100, 116, 104, 58, 32, 51, 112, 120, 59, 13, 10, 45, 45, 120, 58, 0xd800,
    ]);
    let ctx = context(&f.dom, vec![row(f.first, f.a, true)]);
    let calls = Cell::new(0);
    let mut lease = f
        .dom
        .begin_source_edit_with_inline_styles(
            DomSourceEdit::set_attribute(f.first, "style", raw.clone()),
            ctx,
            limits(),
            requests(),
            &cancel(),
        )
        .unwrap();
    let id = lease.operation();
    let result = lease
        .advance_with_inline_styles(
            |c, ctx| {
                calls.set(calls.get() + 1);
                admission(c, ctx)
            },
            &cancel(),
        )
        .unwrap();
    let (source, pair) = result.into_parts();
    assert!(matches!(source, DomSourceProgress::Complete(ref c) if c.operation() == id));
    let pair = pair.unwrap();
    assert_eq!(pair.dom().from(), before);
    assert_eq!(pair.dom().to(), lease.snapshot().stamp());
    assert!(pair.inline().owner_effects().is_empty());
    assert_eq!(calls.get(), 1);
    let snapshot = lease.snapshot();
    let attr = snapshot
        .node(f.first)
        .unwrap()
        .as_element()
        .unwrap()
        .attributes()
        .iter()
        .find(|a| a.name().local_name().to_utf8().unwrap() == "style")
        .unwrap();
    assert_eq!(attr.value(), &raw);
    drop(lease);
    let current = capture(&f.dom, f.first);
    assert_eq!(current.properties().handle(), old.properties().handle());
    assert_eq!(width(&current), "3px");
    let source = current
        .properties()
        .snapshot()
        .block(current.properties().handle().id())
        .unwrap()
        .replacement_input()
        .unwrap()
        .parse_source()
        .unwrap()
        .source();
    assert_eq!(source.as_str(), "width: 3px;\n--x:�");
    assert_eq!(
        source.original_utf16().unwrap().as_slice(),
        raw.code_units()
    );
    assert_eq!(source.original_utf16_range(11..12), Some(11..13));
    assert_eq!(width(&old), "");
}

#[test]
fn resolver_refusal_and_unwind_restore_same_lease_before_retry() {
    let mut f = fixture();
    let before = f.dom.snapshot().stamp();
    let old = capture(&f.dom, f.first);
    let ctx = context(&f.dom, vec![row(f.first, f.a, true)]);
    let mut lease = f
        .dom
        .begin_source_edit_with_inline_styles(
            DomSourceEdit::set_attribute(f.first, "style", "width: 5px"),
            ctx,
            limits(),
            requests(),
            &cancel(),
        )
        .unwrap();
    let id = lease.operation();
    let error =
        lease.advance_with_inline_styles(|_, _| Err(CssomError::RevisionExhausted), &cancel());
    assert!(matches!(
        error,
        Err(DomInlineStyleError::Cssom(CssomError::RevisionExhausted))
    ));
    assert_eq!(lease.snapshot().stamp(), before);
    assert_eq!(lease.operation(), id);
    let spent = lease.status().spent_work();
    let panic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _ = lease.advance_with_inline_styles(|_, _| std::panic::panic_any(37u32), &cancel());
    }))
    .unwrap_err();
    assert_eq!(panic.downcast_ref::<u32>(), Some(&37));
    assert_eq!(lease.operation(), id);
    assert_eq!(lease.snapshot().stamp(), before);
    assert!(lease.status().spent_work() >= spent);
    let result = lease
        .advance_with_inline_styles(admission, &cancel())
        .unwrap();
    assert!(matches!(result.source(), DomSourceProgress::Complete(_)));
    assert!(result.inline().is_some());
    drop(lease);
    assert_eq!(width(&capture(&f.dom, f.first)), "5px");
    assert_eq!(width(&old), "");
}

#[test]
fn subtree_adoption_updates_two_packets_in_one_commit_without_reparse() {
    let mut f = fixture();
    let mut initial = DomBatch::new();
    initial.set_attribute(f.first, "style", "width: 3px");
    initial.set_attribute(f.second, "style", "width: 4px");
    let inputs = DomInlineStyleBatchContext::new(
        &initial,
        f.dom.snapshot().stamp(),
        vec![row(f.first, f.a, false), row(f.second, f.a, false)],
    )
    .unwrap();
    f.dom
        .apply_with_inline_styles(initial, inputs, requests(), no_decisions, &cancel())
        .unwrap();
    let a = capture(&f.dom, f.first);
    let b = capture(&f.dom, f.second);
    let empty = |element| {
        DomInlineStyleProfile::new(
            element,
            f.b,
            CssParserContext::default(),
            CssomInputVersion {
                role: CssomInputRole::Support,
                identity: "destination-empty".into(),
                revision: 1,
            },
            CssomDeclarationSupport::default(),
        )
        .unwrap()
    };
    let ctx = context(&f.dom, vec![empty(f.first), empty(f.second)]);
    let mut lease = f
        .dom
        .begin_source_edit_with_inline_styles(
            DomSourceEdit::adopt(f.root, f.b.node()),
            ctx,
            limits(),
            requests(),
            &cancel(),
        )
        .unwrap();
    let result = lease
        .advance_with_inline_styles(no_decisions, &cancel())
        .unwrap();
    assert!(matches!(result.source(), DomSourceProgress::Complete(_)));
    let CssomPublication::Changed(summary) = result.inline().unwrap().inline().publication() else {
        panic!("new Document packet")
    };
    assert_eq!(summary.from(), a.properties().snapshot().revision());
    let to = summary.to().clone();
    drop(lease);
    for (node, old, expected) in [(f.first, a, "3px"), (f.second, b, "4px")] {
        let c = capture(&f.dom, node);
        assert_eq!(c.properties().handle(), old.properties().handle());
        assert_eq!(width(&c), expected);
        assert_eq!(width(&old), expected);
        assert_eq!(c.dom().node(node).unwrap().owner_document(), Some(f.b));
        assert_eq!(c.properties().snapshot().revision(), &to);
    }
    let ctx = context(&f.dom, vec![empty(f.first)]);
    let mut lease = f
        .dom
        .begin_source_edit_with_inline_styles(
            DomSourceEdit::set_attribute(f.first, "style", "width: 10px"),
            ctx,
            limits(),
            requests(),
            &cancel(),
        )
        .unwrap();
    let result = lease
        .advance_with_inline_styles(no_decisions, &cancel())
        .unwrap();
    assert!(result.inline().is_some());
    drop(lease);
    assert_eq!(width(&capture(&f.dom, f.first)), "");
}

#[test]
fn output_refusal_precedes_resolver_and_preserves_original_invocation() {
    let mut f = fixture();
    let before = f.dom.snapshot().stamp();
    let ctx = context(&f.dom, vec![row(f.first, f.a, true)]);
    let none = DomSourceLimits::new(64_000_000, 1_000_000, 20_000_000, 128, 0, 8192, 4);
    let mut lease = f
        .dom
        .begin_source_edit_with_inline_styles(
            DomSourceEdit::set_attribute(f.first, "style", "width: 7px"),
            ctx,
            none,
            requests(),
            &cancel(),
        )
        .unwrap();
    let id = lease.operation();
    let error = lease.advance_with_inline_styles(no_decisions, &cancel());
    assert!(matches!(
        error,
        Err(DomInlineStyleError::Dom(DomError::SourceOutputLimit))
    ));
    assert_eq!(lease.snapshot().stamp(), before);
    assert_eq!(lease.operation(), id);
    assert!(!lease.status().has_published());
}

fn block(request: &DomStyleRequest) -> DomStyleInlineCspObservation {
    DomStyleInlineCspObservation::evaluated(
        request.source(),
        std::sync::Arc::new(vec!["style-src 'none'".into()]),
        19,
        false,
    )
}
#[test]
fn published_csp_prefix_parks_and_returns_inline_evidence_only_once() {
    let mut f = fixture();
    let old = capture(&f.dom, f.first);
    let mut b = DomBatch::new();
    let first = b.create_element_ns(f.a, Namespace::html(), "style");
    let second = b.create_element_ns(f.a, Namespace::html(), "style");
    b.insert_before(&first, f.root, None);
    b.insert_before(&second, f.root, None);
    let c = f.dom.apply(b, &cancel()).unwrap();
    let first = c.created(&first).unwrap().unwrap();
    let second = c.created(&second).unwrap().unwrap();
    let ctx = context(
        &f.dom,
        vec![row(f.first, f.a, false), row(f.second, f.a, false)],
    );
    let mut lease = f
        .dom
        .begin_source_edit_with_inline_styles(
            DomSourceEdit::insert_before(f.root, f.a.node(), None),
            ctx,
            limits(),
            requests(),
            &cancel(),
        )
        .unwrap();
    let id = lease.operation();
    let (source, pair) = lease
        .advance_with_inline_styles(no_decisions, &cancel())
        .unwrap()
        .into_parts();
    let DomSourceProgress::Request(DomSourceRequest::Style(request)) = source else {
        panic!("actual first CSP request")
    };
    assert_eq!(request.source().element(), first);
    let pair = pair.unwrap();
    assert_eq!(pair.dom().to(), lease.snapshot().stamp());
    assert!(pair.inline().owner_effects().is_empty());
    let CssomPublication::Changed(summary) = pair.inline().publication() else {
        panic!("scope changed at insertion")
    };
    assert_eq!(summary.from(), old.properties().snapshot().revision());
    let inline_revision = summary.to().clone();
    let phases = lease.status().published_phases();
    let work = lease.status().spent_work();
    assert!(lease.status().has_published());
    drop(lease);
    assert_eq!(f.dom.pending_source_edit().unwrap().operation(), id);
    let mut lease = f.dom.resume_source_edit(id, limits(), &cancel()).unwrap();
    let repeated = lease
        .advance_with_inline_styles(no_decisions, &cancel())
        .unwrap();
    assert!(repeated.inline().is_none());
    let DomSourceProgress::Request(DomSourceRequest::Style(repeated_request)) =
        repeated.into_parts().0
    else {
        panic!("same retained request")
    };
    assert_eq!(
        repeated_request.source().ordinal(),
        request.source().ordinal()
    );
    assert_eq!(lease.status().published_phases(), phases);
    assert!(lease.status().spent_work() >= work);
    lease.request_cancel();
    lease
        .accept_inline_csp(&repeated_request, &block(&repeated_request), &cancel())
        .unwrap();
    let tail = lease
        .advance_with_inline_styles(no_decisions, &cancel())
        .unwrap();
    assert!(tail.inline().is_none());
    let DomSourceProgress::Request(DomSourceRequest::Style(tail)) = tail.into_parts().0 else {
        panic!("second mandatory Style")
    };
    assert_eq!(tail.source().element(), second);
    assert!(tail.source().ordinal() > request.source().ordinal());
    lease
        .accept_inline_csp(&tail, &block(&tail), &cancel())
        .unwrap();
    let done = lease
        .advance_with_inline_styles(no_decisions, &cancel())
        .unwrap();
    assert!(done.inline().is_none());
    assert!(matches!(done.source(),DomSourceProgress::Complete(c) if c.cancellation_requested()));
    drop(lease);
    assert!(f.dom.pending_source_edit().is_none());
    let current = capture(&f.dom, f.first);
    assert_eq!(current.properties().handle(), old.properties().handle());
    assert_eq!(width(&current), width(&old));
    assert_eq!(current.properties().snapshot().revision(), &inline_revision);
    assert_eq!(current.dom().node(f.first).unwrap().parent(), Some(f.root));
}

#[test]
fn zero_binding_capability_survives_authentic_observer_reentry_and_original_completion() {
    let mut f = fixture_with_bindings(false);
    let mut b = DomBatch::new();
    let dialog = b.create_element_ns(f.a, Namespace::html(), "dialog");
    b.insert_before(&dialog, f.root, None);
    b.insert_before(f.root, f.a, None);
    let c = f.dom.apply(b, &cancel()).unwrap();
    let dialog = c.created(&dialog).unwrap().unwrap();
    let executor = f.dom.bind_source_event_executor(&cancel()).unwrap();
    let ctx = context(&f.dom, vec![row(f.first, f.a, false)]);
    let mut lease = f
        .dom
        .begin_source_edit_with_inline_styles(
            DomSourceEdit::show_dialog(dialog),
            ctx,
            limits(),
            requests(),
            &cancel(),
        )
        .unwrap();
    let id = lease.operation();
    let progress = lease
        .advance_with_inline_styles(no_decisions, &cancel())
        .unwrap();
    assert!(progress.inline().is_none());
    let DomSourceProgress::Request(DomSourceRequest::Event(request)) = progress.into_parts().0
    else {
        panic!("actual BeforeToggle")
    };
    drop(lease);
    let mut permit = Some(
        executor
            .begin_dispatch(&f.dom, &request, &cancel())
            .unwrap(),
    );
    let scope = executor
        .enter_dispatch(&f.dom, permit.as_ref().unwrap(), &cancel())
        .unwrap();
    let input = DomInlineStyleInputs::new(
        f.dom.snapshot().document(f.a).unwrap().stamp(),
        CssParserContext::default(),
        version(false),
        support(false),
    )
    .unwrap();
    let handle = f
        .dom
        .inline_style(f.first, &input, requests(), no_decisions, &cancel())
        .unwrap();
    let historical = capture(&f.dom, f.first);
    drop(scope);
    let mut receipt = Some(
        executor
            .seal_completed(
                &f.dom,
                &mut permit,
                DomSourceEventOutcome::Canceled,
                &cancel(),
            )
            .unwrap(),
    );
    let mut lease = f.dom.resume_source_edit(id, limits(), &cancel()).unwrap();
    lease
        .accept_source_event(&request, &mut receipt, &cancel())
        .unwrap();
    let done = lease
        .advance_with_inline_styles(no_decisions, &cancel())
        .unwrap();
    assert!(done.inline().is_none());
    assert!(matches!(done.source(), DomSourceProgress::Complete(_)));
    drop(lease);
    assert_eq!(capture(&f.dom, f.first).properties().handle(), &handle);
    let ctx = context(&f.dom, vec![row(f.first, f.a, false)]);
    let mut lease = f
        .dom
        .begin_source_edit_with_inline_styles(
            DomSourceEdit::set_attribute(f.first, "style", "width: 9px"),
            ctx,
            limits(),
            requests(),
            &cancel(),
        )
        .unwrap();
    let done = lease
        .advance_with_inline_styles(no_decisions, &cancel())
        .unwrap();
    assert!(done.inline().is_some());
    drop(lease);
    assert_eq!(width(&capture(&f.dom, f.first)), "9px");
    assert_eq!(width(&historical), "");
}

#[test]
fn same_value_wrapper_requires_one_output_before_resolver_or_publication() {
    let mut f = fixture_with_bindings(false);
    let mut b = DomBatch::new();
    for id in [f.first, f.second] {
        b.set_attribute(id, "style", "width: 7px");
    }
    f.dom.apply(b, &cancel()).unwrap();
    let input = DomInlineStyleInputs::new(
        f.dom.snapshot().document(f.a).unwrap().stamp(),
        CssParserContext::default(),
        version(false),
        support(false),
    )
    .unwrap();
    f.dom
        .inline_style(f.first, &input, requests(), no_decisions, &cancel())
        .unwrap();
    let before = capture(&f.dom, f.first);
    let zero = DomSourceLimits::new(64_000_000, 1_000_000, 20_000_000, 128, 0, 8192, 4);
    // The identical setter on an unbound Element has no semantic DOM output.
    let mut control = f
        .dom
        .begin_source_edit(
            DomSourceEdit::set_attribute(f.second, "style", "width: 7px"),
            zero,
            &cancel(),
        )
        .unwrap();
    let DomSourceProgress::Complete(control_result) = control.advance(&cancel()).unwrap() else {
        panic!("ordinary no-op control")
    };
    assert!(control_result.commit().changes().changes().is_empty());
    drop(control);
    let ctx = context(&f.dom, vec![row(f.first, f.a, true)]);
    let mut lease = f
        .dom
        .begin_source_edit_with_inline_styles(
            DomSourceEdit::set_attribute(f.first, "style", "width: 7px"),
            ctx,
            zero,
            requests(),
            &cancel(),
        )
        .unwrap();
    assert!(matches!(
        lease.advance_with_inline_styles(no_decisions, &cancel()),
        Err(DomInlineStyleError::Dom(DomError::SourceOutputLimit))
    ));
    assert_eq!(lease.snapshot().stamp(), before.dom().stamp());
    assert!(!lease.status().has_published());
    drop(lease);
    assert_eq!(
        capture(&f.dom, f.first).properties().snapshot().revision(),
        before.properties().snapshot().revision()
    );
    // Exactly one available output admits the new original paired receipt.
    let one = DomSourceLimits::new(64_000_000, 1_000_000, 20_000_000, 128, 1, 8192, 4);
    let ctx = context(&f.dom, vec![row(f.first, f.a, true)]);
    let calls = Cell::new(0);
    let mut lease = f
        .dom
        .begin_source_edit_with_inline_styles(
            DomSourceEdit::set_attribute(f.first, "style", "width: 7px"),
            ctx,
            one,
            requests(),
            &cancel(),
        )
        .unwrap();
    let result = lease
        .advance_with_inline_styles(
            |c, ctx| {
                calls.set(calls.get() + 1);
                admission(c, ctx)
            },
            &cancel(),
        )
        .unwrap();
    assert!(result.inline().is_some());
    assert_eq!(calls.get(), 1);
    let DomSourceProgress::Complete(done) = result.source() else {
        panic!("same-value completion")
    };
    assert!(done.commit().changes().changes().is_empty());
    assert_eq!(lease.snapshot().stamp(), before.dom().stamp());
}

#[test]
fn cancellation_during_resolver_rolls_back_then_retries_same_original_operation() {
    let mut f = fixture();
    let before = f.dom.snapshot().stamp();
    let ctx = context(&f.dom, vec![row(f.first, f.a, true)]);
    let flag = cancel();
    let mut lease = f
        .dom
        .begin_source_edit_with_inline_styles(
            DomSourceEdit::set_attribute(f.first, "style", "width: 11px"),
            ctx,
            limits(),
            requests(),
            &flag,
        )
        .unwrap();
    let id = lease.operation();
    let failed = lease.advance_with_inline_styles(
        |c, ctx| {
            flag.store(true, std::sync::atomic::Ordering::Relaxed);
            admission(c, ctx)
        },
        &flag,
    );
    assert!(matches!(
        failed,
        Err(DomInlineStyleError::Dom(DomError::Cancelled))
    ));
    assert_eq!(lease.snapshot().stamp(), before);
    let spent = lease.status().spent_work();
    assert_eq!(lease.operation(), id);
    flag.store(false, std::sync::atomic::Ordering::Relaxed);
    let result = lease.advance_with_inline_styles(admission, &flag).unwrap();
    assert!(result.inline().is_some());
    assert!(spent > 0);
    assert!(matches!(result.source(),DomSourceProgress::Complete(c) if c.operation()==id));
    drop(lease);
    assert_eq!(width(&capture(&f.dom, f.first)), "11px");
}

#[test]
fn missing_profiles_refuse_reached_setter_but_absent_removal_needs_no_resolver() {
    let mut f = fixture();
    let old = capture(&f.dom, f.first);
    let before = f.dom.snapshot().stamp();
    let ctx = context(&f.dom, vec![]);
    let mut lease = f
        .dom
        .begin_source_edit_with_inline_styles(
            DomSourceEdit::set_attribute(f.first, "style", "width: 13px"),
            ctx,
            limits(),
            requests(),
            &cancel(),
        )
        .unwrap();
    assert!(
        matches!(lease.advance_with_inline_styles(no_decisions,&cancel()),Err(DomInlineStyleError::Dom(DomError::InlineStyleProfileMissing{element})) if element==f.first)
    );
    assert_eq!(lease.snapshot().stamp(), before);
    drop(lease);
    assert_eq!(
        capture(&f.dom, f.first).properties().snapshot().revision(),
        old.properties().snapshot().revision()
    );
    let ctx = context(&f.dom, vec![]);
    let mut lease = f
        .dom
        .begin_source_edit_with_inline_styles(
            DomSourceEdit::remove_attribute(f.first, "style"),
            ctx,
            limits(),
            requests(),
            &cancel(),
        )
        .unwrap();
    let done = lease
        .advance_with_inline_styles(no_decisions, &cancel())
        .unwrap();
    assert!(done.inline().is_none());
    assert!(matches!(done.source(), DomSourceProgress::Complete(_)));
    assert_eq!(lease.snapshot().stamp(), before);
}

#[test]
fn plain_advance_refuses_then_borrowed_resolver_uses_same_retained_profiles() {
    let mut f = fixture();
    let ctx = context(&f.dom, vec![row(f.first, f.a, true)]);
    let before = f.dom.snapshot().stamp();
    let mut lease = f
        .dom
        .begin_source_edit_with_inline_styles(
            DomSourceEdit::set_attribute(f.first, "style", "width: 17px"),
            ctx,
            limits(),
            requests(),
            &cancel(),
        )
        .unwrap();
    let id = lease.operation();
    assert_eq!(
        lease.advance(&cancel()).unwrap_err(),
        DomError::InlineStyleContextRequired {
            element: f.first,
            change: DomInlineStyleChange::Attribute
        }
    );
    assert_eq!(lease.snapshot().stamp(), before);
    let spent = lease.status().spent_work();
    let done = lease
        .advance_with_inline_styles(admission, &cancel())
        .unwrap();
    assert!(done.inline().is_some());
    assert!(matches!(done.source(),DomSourceProgress::Complete(c) if c.operation()==id));
    assert!(spent > 0);
    drop(lease);
    assert_eq!(width(&capture(&f.dom, f.first)), "17px");
}
