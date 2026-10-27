//! Actual original Source and mutable-custody barriers, without Runtime delivery claims.
use super::*;
use crate::{
    DomSourceEdit, DomSourceEventOutcome, DomSourceLimits, DomSourceProgress, DomSourceRequest,
};
use surgeist_css::CssParserContext;
use surgeist_css::source::CssUtf16InputLimits;
use surgeist_cssom::{
    CssomDeclarationRequestLimits, CssomDeclarationSupport, CssomInputRole, CssomInputVersion,
    CssomLimits,
};

impl Dom {
    // Owning inspection keeps independent producer observations distinct from
    // the public coupled capture, which correctly rejects incomplete Source.
    pub(in crate::store) fn inline_snapshot_for_test(
        &self,
    ) -> Option<surgeist_cssom::CssomSnapshot> {
        self.inline_styles
            .as_ref()?
            .store
            .as_ref()
            .map(CssomStore::snapshot)
    }
}

fn cancel() -> AtomicBool {
    AtomicBool::new(false)
}
fn policy() -> DomInlineStyleLimits {
    DomInlineStyleLimits::new(CssomLimits::default(), 64, 64_000_000, 8_000_000, 4_000_000).unwrap()
}
fn source_limits() -> DomSourceLimits {
    DomSourceLimits::new(64_000_000, 1_000_000, 10_000_000, 64, 4096, 4096, 4)
}
fn request_limits() -> DomInlineStyleRequestLimits {
    DomInlineStyleRequestLimits {
        input: CssUtf16InputLimits::default(),
        declaration: CssomDeclarationRequestLimits::default(),
    }
}
fn fixture() -> (Dom, DocumentId, NodeId, NodeId) {
    let limits = DomLimits::new(
        128,
        100_000,
        256,
        128,
        128,
        32,
        4_000_000,
        crate::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
    )
    .unwrap()
    .with_dispatch_custody(
        crate::DomDispatchCustodyLimits::new(64, 4096, 16_000_000, 16_000_000, 4_000_000).unwrap(),
    );
    let mut dom = Dom::new(limits).unwrap();
    let mut b = DomBatch::new();
    let document = b.create_document();
    let root = b.create_element_ns(&document, crate::Namespace::html(), "main");
    let dialog = b.create_element_ns(&document, crate::Namespace::html(), "dialog");
    let target = b.create_element_ns(&document, crate::Namespace::html(), "div");
    b.insert_before(&root, &document, None);
    b.insert_before(&dialog, &root, None);
    b.insert_before(&target, &root, None);
    b.set_attribute(&target, "style", "--x:old");
    let c = dom.apply(b, &cancel()).unwrap();
    (
        dom,
        c.document(&document).unwrap(),
        c.created(&dialog).unwrap().unwrap(),
        c.created(&target).unwrap().unwrap(),
    )
}
fn inputs(dom: &Dom, document: DocumentId) -> DomInlineStyleInputs {
    DomInlineStyleInputs::new(
        dom.snapshot().document(document).unwrap().stamp(),
        CssParserContext::default(),
        CssomInputVersion {
            role: CssomInputRole::Support,
            identity: "explicit-empty".into(),
            revision: 1,
        },
        CssomDeclarationSupport::default(),
    )
    .unwrap()
}

#[test]
fn callback_first_binding_survives_original_completion_and_guards_a_later_source_notification() {
    let (mut dom, document, dialog, target) = fixture();
    let executor = dom.bind_source_event_executor(&cancel()).unwrap();
    // The Source predates both the policy and all inline membership.
    let request = {
        let mut lease = dom
            .begin_source_edit(
                DomSourceEdit::show_dialog(dialog),
                source_limits(),
                &cancel(),
            )
            .unwrap();
        let DomSourceProgress::Request(DomSourceRequest::Event(request)) =
            lease.advance(&cancel()).unwrap()
        else {
            panic!("actual BeforeToggle");
        };
        request
    };
    assert!(dom.inline_styles.is_none());
    let mut permit = Some(executor.begin_dispatch(&dom, &request, &cancel()).unwrap());
    let scope = executor
        .enter_dispatch(&dom, permit.as_ref().unwrap(), &cancel())
        .unwrap();
    dom.configure_inline_styles(policy(), &cancel()).unwrap();
    let i = inputs(&dom, document);
    let handle = dom
        .inline_style(
            target,
            &i,
            request_limits(),
            |_, _| panic!("custom declaration needs no decision"),
            &cancel(),
        )
        .unwrap();
    let captured = dom.capture_inline_style(target).unwrap().unwrap();
    drop(scope);
    let mut receipt = Some(
        executor
            .seal_completed(
                &dom,
                &mut permit,
                DomSourceEventOutcome::Canceled,
                &cancel(),
            )
            .unwrap(),
    );
    assert!(permit.is_none());
    let mut lease = dom
        .resume_source_edit(request.operation(), source_limits(), &cancel())
        .unwrap();
    lease
        .accept_source_event(&request, &mut receipt, &cancel())
        .unwrap();
    assert!(receipt.is_none());
    assert!(matches!(
        lease.advance(&cancel()).unwrap(),
        DomSourceProgress::Complete(_)
    ));
    drop(lease);
    assert!(dom.pending_source_edit().is_none());
    assert_eq!(
        dom.capture_inline_style(target)
            .unwrap()
            .unwrap()
            .properties()
            .handle(),
        &handle
    );
    // Source's public root admits one edit. Event observation discarded its
    // previous Draft; its genuine canceled tail reformed from current Dom.
    // A later real style invocation must use the callback-created membership.
    let mut lease = dom
        .begin_source_edit(
            DomSourceEdit::set_attribute(target, "style", "--x:old"),
            source_limits(),
            &cancel(),
        )
        .unwrap();
    let before = lease.snapshot();
    let error = lease.advance(&cancel()).unwrap_err();
    assert_eq!(
        error,
        DomError::InlineStyleContextRequired {
            element: target,
            change: DomInlineStyleChange::Attribute
        }
    );
    assert_eq!(
        lease
            .snapshot()
            .node(target)
            .unwrap()
            .as_element()
            .unwrap()
            .attributes(),
        before
            .node(target)
            .unwrap()
            .as_element()
            .unwrap()
            .attributes()
    );
    drop(lease);
    let after = dom.capture_inline_style(target).unwrap().unwrap();
    assert_eq!(after.properties().handle(), &handle);
    assert_eq!(
        after.properties().snapshot().revision(),
        captured.properties().snapshot().revision()
    );
    assert_eq!(after.dom().stamp(), before.stamp());
}

#[test]
fn nonreentrant_parked_source_cannot_initialize_inline_policy_or_owner() {
    let (mut dom, _, dialog, _) = fixture();
    let _executor = dom.bind_source_event_executor(&cancel()).unwrap();
    let operation = {
        let mut lease = dom
            .begin_source_edit(
                DomSourceEdit::show_dialog(dialog),
                source_limits(),
                &cancel(),
            )
            .unwrap();
        let DomSourceProgress::Request(DomSourceRequest::Event(_)) =
            lease.advance(&cancel()).unwrap()
        else {
            panic!("actual parked event");
        };
        lease.status().operation()
    };
    let before = dom.custody_registry.ledger().total();
    assert!(matches!(
        dom.configure_inline_styles(policy(), &cancel()),
        Err(DomInlineStyleError::Dom(DomError::SourceOperationPending))
    ));
    assert!(dom.inline_styles.is_none());
    assert_eq!(dom.custody_registry.ledger().total(), before);
    assert_eq!(dom.pending_source_edit().unwrap().operation(), operation);
}

#[test]
fn zero_binding_policy_and_pre_cancel_refuse_without_constructing_a_cssom_store() {
    let (mut dom, document, _, target) = fixture();
    let mut limits = policy();
    limits.bindings = 0;
    dom.configure_inline_styles(limits, &cancel()).unwrap();
    let i = inputs(&dom, document);
    let before = dom.custody_registry.ledger().total();
    assert!(matches!(
        dom.inline_style(
            target,
            &i,
            request_limits(),
            |_, _| panic!("no admission"),
            &cancel()
        ),
        Err(DomInlineStyleError::Dom(DomError::ResourceLimit {
            resource: DomResource::InlineStyleBindings,
            limit: 0
        }))
    ));
    assert!(dom.inline_styles.as_ref().unwrap().store.is_none());
    assert_eq!(dom.custody_registry.ledger().total(), before);
    assert!(
        dom.inline_style(
            target,
            &i,
            request_limits(),
            |_, _| panic!("no admission"),
            &AtomicBool::new(true)
        )
        .is_err()
    );
    assert_eq!(dom.custody_registry.ledger().total(), before);
}

#[test]
fn failed_first_producer_admission_releases_temporary_packet_and_sparse_controls() {
    let (mut dom, document, _, target) = fixture();
    dom.configure_inline_styles(policy(), &cancel()).unwrap();
    let i = inputs(&dom, document);
    dom.custody_registry.reconcile_releases();
    let before = dom.custody_registry.ledger().total();
    let mut limits = request_limits();
    limits.input.max_input_units = 1;
    assert!(
        dom.inline_style(target, &i, limits, |_, _| panic!("no admission"), &cancel())
            .is_err()
    );
    dom.custody_registry.reconcile_releases();
    assert_eq!(dom.custody_registry.ledger().total(), before);
    assert!(dom.inline_styles.as_ref().unwrap().store.is_none());
    assert!(dom.inline_styles.as_ref().unwrap().objects.is_empty());
}

#[test]
fn original_ledger_header_allowance_refuses_one_short_then_admits_exactly() {
    let (mut dom, _, _, _) = fixture();
    dom.custody_registry.reconcile_releases();
    let before = dom.custody_registry.ledger().total();
    let exact = before.bytes() + size_of::<Owner>();
    let mut limits = policy();
    limits.formation_bytes = exact - 1;
    assert!(matches!(
        dom.configure_inline_styles(limits, &cancel()),
        Err(DomInlineStyleError::Dom(DomError::ResourceLimit {
            resource: DomResource::InlineStyleStorage, limit
        })) if limit == exact - 1
    ));
    assert!(dom.inline_styles.is_none());
    assert_eq!(dom.custody_registry.ledger().total(), before);
    let mut limits = policy();
    limits.formation_bytes = exact;
    dom.configure_inline_styles(limits, &cancel()).unwrap();
    assert_eq!(
        dom.custody_registry.ledger().total(),
        custody::Usage::new(exact, before.units())
    );
    assert!(dom.inline_styles.as_ref().unwrap().store.is_none());
}

#[test]
fn cached_support_qualification_uses_the_new_observers_original_work() {
    let (mut dom, document, dialog, target) = fixture();
    dom.configure_inline_styles(policy(), &cancel()).unwrap();
    let mut i = inputs(&dom, document);
    i.support_version.identity = "s".repeat(131_072);
    let handle = dom
        .inline_style(
            target,
            &i,
            request_limits(),
            |_, _| panic!("no decision"),
            &cancel(),
        )
        .unwrap();
    let captured = dom.capture_inline_style(target).unwrap().unwrap();
    let executor = dom.bind_source_event_executor(&cancel()).unwrap();
    let mut limits = source_limits();
    limits.work = 100_000;
    let request = {
        let mut lease = dom
            .begin_source_edit(DomSourceEdit::show_dialog(dialog), limits, &cancel())
            .unwrap();
        let DomSourceProgress::Request(DomSourceRequest::Event(request)) =
            lease.advance(&cancel()).unwrap()
        else {
            panic!("actual BeforeToggle");
        };
        request
    };
    let permit = executor.begin_dispatch(&dom, &request, &cancel()).unwrap();
    let scope = executor.enter_dispatch(&dom, &permit, &cancel()).unwrap();
    assert!(matches!(
        dom.inline_style(
            target,
            &i,
            request_limits(),
            |_, _| panic!("no decision"),
            &cancel()
        ),
        Err(DomInlineStyleError::Dom(DomError::SourceWorkLimit {
            limit: 100_000
        }))
    ));
    assert_eq!(dom.pending_source_edit().unwrap().spent_work(), 100_000);
    let after = dom.capture_inline_style(target).unwrap().unwrap();
    assert_eq!(after.properties().handle(), &handle);
    assert_eq!(
        after.properties().snapshot().revision(),
        captured.properties().snapshot().revision()
    );
    assert_eq!(after.dom().stamp(), captured.dom().stamp());
    drop(scope);
}

#[test]
fn ordinary_inline_replay_uses_the_observing_sources_original_remaining_work() {
    let (mut dom, document, dialog, target) = fixture();
    dom.configure_inline_styles(policy(), &cancel()).unwrap();
    let i = inputs(&dom, document);
    dom.inline_style(
        target,
        &i,
        request_limits(),
        |_, _| panic!("empty support"),
        &cancel(),
    )
    .unwrap();
    let captured = dom.capture_inline_style(target).unwrap().unwrap();
    let executor = dom.bind_source_event_executor(&cancel()).unwrap();
    let mut limits = source_limits();
    limits.work = 100_000;
    let request = {
        let mut lease = dom
            .begin_source_edit(DomSourceEdit::show_dialog(dialog), limits, &cancel())
            .unwrap();
        let DomSourceProgress::Request(DomSourceRequest::Event(request)) =
            lease.advance(&cancel()).unwrap()
        else {
            panic!("actual BeforeToggle")
        };
        request
    };
    let permit = executor.begin_dispatch(&dom, &request, &cancel()).unwrap();
    let scope = executor.enter_dispatch(&dom, &permit, &cancel()).unwrap();
    let mut batch = DomBatch::new();
    batch.set_attribute(target, "style", "--x:new");
    let row = crate::DomInlineStyleProfile::new(
        target,
        document,
        CssParserContext::default(),
        CssomInputVersion {
            role: CssomInputRole::Support,
            identity: "s".repeat(131_072),
            revision: 1,
        },
        CssomDeclarationSupport::default(),
    )
    .unwrap();
    let context =
        crate::DomInlineStyleBatchContext::new(&batch, dom.snapshot().stamp(), vec![row]).unwrap();
    assert!(matches!(
        dom.apply_with_inline_styles(
            batch,
            context,
            request_limits(),
            |_, _| panic!("empty support"),
            &cancel()
        ),
        Err(DomInlineStyleError::Dom(DomError::SourceWorkLimit {
            limit: 100_000
        }))
    ));
    assert_eq!(dom.pending_source_edit().unwrap().spent_work(), 100_000);
    let after = dom.capture_inline_style(target).unwrap().unwrap();
    assert_eq!(after.dom().stamp(), captured.dom().stamp());
    assert_eq!(
        after.properties().snapshot().revision(),
        captured.properties().snapshot().revision()
    );
    drop(scope);
}
