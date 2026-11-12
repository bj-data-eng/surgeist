//! The DOM issuer boundary uses the same trusted native publication inputs as
//! the Fullscreen fixture. It executes the original child constructor and seals
//! real issuer receipts; it does not claim Runtime Window or event-loop service.
use super::*;
use crate::{
    DomHostedChildStep, DomInitialPermissionsPolicy, DomPermissionsPolicyValue, DomSourceEdit,
};
use std::cell::Cell;

thread_local! {
    static CANCEL_HISTORY: Cell<bool> = const { Cell::new(false) };
}

pub(super) fn after_history_assignment(cancel: &AtomicBool) {
    CANCEL_HISTORY.with(|armed| {
        if armed.replace(false) {
            cancel.store(true, Ordering::Release);
        }
    });
}

fn limits() -> DomSourceLimits {
    DomSourceLimits::new(64_000_000, 4_000_000, 100_000_000, 128, 4096, 4096, 4)
}

fn uuid(last: u16) -> [u16; 36] {
    let mut value = [0; 36];
    for (out, unit) in value
        .iter_mut()
        .zip("00000000-0000-4000-8000-000000000000".encode_utf16())
    {
        *out = unit;
    }
    value[35] = last;
    value
}

#[test]
fn child_history_publication_cancellation_restores_graph_and_retries_the_original_receipt() {
    let cancel = AtomicBool::new(false);
    let (mut dom, mut source, document, _) = super::super::fullscreen::native_fixture(
        DomLimits::new(
            128,
            1_000_000,
            256,
            128,
            128,
            32,
            4_000_000,
            crate::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
        )
        .unwrap()
        .with_fullscreen_exits(4)
        .unwrap()
        .with_fullscreen_events(4)
        .unwrap(),
    );
    let parent = dom
        .snapshot()
        .document(document)
        .unwrap()
        .document_element()
        .unwrap();
    let mut batch = DomBatch::new();
    // A closed HTML Document permits native construction without a hosted
    // initializer. Moving the Frame into the active tree then starts the real
    // child-construction protocol.
    let closed = batch.create_html_document(document, None);
    let iframe =
        batch.create_element_ns(&closed, surgeist_html::values::Namespace::html(), "frame");
    let iframe = dom
        .apply(batch, &cancel)
        .unwrap()
        .created(&iframe)
        .unwrap()
        .unwrap();
    let mut lease = dom
        .begin_source_edit(
            DomSourceEdit::insert_before(iframe, parent, None),
            limits(),
            &cancel,
        )
        .unwrap();
    let DomSourceProgress::Request(DomSourceRequest::HostedChild(child)) =
        lease.advance(&cancel).unwrap()
    else {
        panic!("actual connected Frame reaches its child constructor");
    };
    let preparation = lease
        .prepare_hosted_child(&mut source, &child, &cancel)
        .unwrap();
    let mut prepared = Some(
        source
            .seal_child_preparation(&child, &mut Some(preparation), &cancel)
            .unwrap(),
    );
    lease
        .accept_hosted_child_preparation(&mut source, &child, &mut prepared, &cancel)
        .unwrap();
    assert!(prepared.is_none());

    let mut publication = None;
    let mut operation = None;
    for _ in 0..8 {
        let value = match lease.hosted_child_step(&child).unwrap() {
            DomHostedChildStep::PermissionsPolicy(request) => {
                let mut policy = Some(
                    source
                        .seal_permissions_policy(
                            request,
                            DomInitialPermissionsPolicy::new(
                                DomPermissionsPolicyValue::Enabled,
                                DomPermissionsPolicyValue::Enabled,
                            ),
                            &cancel,
                        )
                        .unwrap(),
                );
                lease
                    .accept_hosted_child_policy(&mut source, &child, &mut policy, &cancel)
                    .unwrap();
                assert!(policy.is_none());
                continue;
            }
            DomHostedChildStep::NativeExecution(request) => {
                let global = source.issue_global(request, &cancel).unwrap();
                source
                    .seal_native_execution(request, global, 200, &cancel)
                    .unwrap()
            }
            DomHostedChildStep::MakeActive(request) => {
                source.seal_make_active(request, &cancel).unwrap()
            }
            DomHostedChildStep::FinishLoading(request) => {
                source.seal_finish_loading(request, 300, &cancel).unwrap()
            }
            DomHostedChildStep::InitialEntry(request) => {
                let entry = source.issue_entry(request, &cancel).unwrap();
                let navigable = source.issue_navigable(request, &cancel).unwrap();
                let mut initialization = Some(DomHostedEntryInitialization::new(
                    uuid(u16::from(b'3')),
                    uuid(u16::from(b'4')),
                ));
                source
                    .seal_initial_entry(request, entry, navigable, &mut initialization, &cancel)
                    .unwrap()
            }
            DomHostedChildStep::QueueHistory(request) => {
                operation = Some(request.operation());
                publication = Some(source.seal_child_history(request, &cancel).unwrap());
                break;
            }
            _ => panic!("unexpected child phase before its history handoff"),
        };
        let mut value = Some(value);
        lease
            .accept_hosted_child_publication(&mut source, &child, &mut value, &cancel)
            .unwrap();
        assert!(value.is_none());
    }
    let operation = operation.expect("original constructor reaches QueueHistory");
    assert!(publication.is_some());
    let before = lease.snapshot();
    let graph = lease
        .dom
        .source_operation
        .as_ref()
        .unwrap()
        .draft
        .hosted_history
        .clone()
        .unwrap();
    let revision = source.revision().unwrap();
    let spent = lease.status().spent_work();
    CANCEL_HISTORY.set(true);
    assert_eq!(
        lease
            .accept_hosted_child_history(&mut source, &child, &mut publication, &cancel)
            .unwrap_err(),
        DomHostedError::Dom(DomError::Cancelled),
    );
    assert!(cancel.load(Ordering::Acquire));
    assert!(
        publication.is_some(),
        "failed publication keeps its authentic receipt"
    );
    assert_eq!(source.revision().unwrap(), revision);
    assert_eq!(lease.snapshot().stamp(), before.stamp());
    assert!(Arc::ptr_eq(
        &graph,
        lease
            .dom
            .source_operation
            .as_ref()
            .unwrap()
            .draft
            .hosted_history
            .as_ref()
            .unwrap()
    ));
    assert!(matches!(
        lease.hosted_child_step(&child).unwrap(),
        DomHostedChildStep::QueueHistory(_)
    ));
    assert!(lease.status().spent_work() > spent);

    cancel.store(false, Ordering::Release);
    let queued = lease
        .accept_hosted_child_history(&mut source, &child, &mut publication, &cancel)
        .unwrap();
    assert_eq!(queued.operation(), operation);
    assert!(publication.is_none());
    assert_ne!(lease.snapshot().stamp(), before.stamp());
    let focused_snapshot = lease.snapshot();
    for id in focused_snapshot.nodes() {
        if let Some(document) = focused_snapshot.node(id).unwrap().as_document() {
            assert_eq!(
                document.focused_area_designation(),
                crate::DomFocusedAreaDesignation::KnownNull
            );
        }
    }
    crate::store::oracle_tests::assert_full_canonical(lease.dom);
    assert!(matches!(
        lease.hosted_child_step(&child).unwrap(),
        DomHostedChildStep::NeedsUrl(_)
    ));
}

#[test]
fn inherited_csp_association_is_parked_across_original_work_refusal_and_raised_retry() {
    use surgeist_html::{
        security::csp::{
            HtmlCspArtifactLayout, HtmlCspAssociationCause, HtmlCspDisposalWork, HtmlCspLayout,
            HtmlCspNativePolicy, HtmlCspPolicyArtifact, HtmlCspProcessorAuthority, HtmlCspStorage,
        },
        values::DocumentString,
    };
    let cancel = AtomicBool::new(false);
    let (mut dom, mut source, parent_document, _) = super::super::fullscreen::native_fixture(
        DomLimits::new(
            128,
            1_000_000,
            256,
            128,
            128,
            32,
            4_000_000,
            crate::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
        )
        .unwrap(),
    );
    // Qualify the fixture's actual initial container through the HTML
    // issuer, then use the normal canonical metadata publication/custody barrier.
    #[derive(Debug)]
    struct InheritedNative {
        text: DocumentString,
        padding: [u8; 1024],
    }
    impl HtmlCspPolicyArtifact for InheritedNative {
        fn retained_layout(&self) -> HtmlCspArtifactLayout {
            let units = self.text.retained_units();
            HtmlCspArtifactLayout::try_new(
                HtmlCspStorage::try_new(
                    size_of::<Self>() + size_of::<Vec<u16>>() + units * 2,
                    units,
                    2,
                )
                .unwrap(),
                HtmlCspDisposalWork::try_new(9, size_of::<Self>()).unwrap(),
            )
            .unwrap()
        }
    }
    let mut allow = |_: HtmlCspLayout| Ok::<_, ()>(());
    let mut parent_policy;
    let weak;
    {
        let mut owner = dom.csp_owner.value().borrow_mut();
        parent_policy = owner.initial_document_with(&mut allow).unwrap();
        let mut trusted = HtmlCspProcessorAuthority::new_with(&mut allow).unwrap();
        let profile = trusted.profile_with(1, &mut allow).unwrap();
        let request = owner
            .request_meta_insertion_with(
                &parent_policy,
                &profile,
                "fixture inherited policy".into(),
                &mut allow,
            )
            .unwrap();
        let native = Arc::new(InheritedNative {
            text: request.qualification().content().clone(),
            padding: [71; 1024],
        });
        weak = Arc::downgrade(&native);
        let prepared = trusted
            .attest_changed_with(&request, native, &mut allow)
            .unwrap();
        drop(
            owner
                .prepare_publication_with(&mut parent_policy, request, prepared, &mut allow)
                .unwrap()
                .publish(),
        );
    }
    let layout = parent_policy.retained_layout().unwrap();
    let mut draft = Draft::new(&dom, Arc::new(BatchIdentity), &cancel).unwrap();
    let units = draft
        .units
        .checked_add(layout.storage().utf16_units())
        .unwrap();
    crate::store::admit(units, dom.limits.units, DomResource::StringCodeUnits).unwrap();
    let mut metadata = draft
        .document_metadata_mut(Key::Existing(parent_document.node()), &cancel)
        .unwrap();
    metadata.policy_container_mut().csp = crate::meta::CspState::Associated(
        crate::meta::SelectedCsp::new(parent_policy.clone(), layout),
    );
    metadata.commit(&cancel).unwrap();
    draft.units = units;
    let prepared = draft.finish(&dom, &cancel).unwrap();
    dom.publish(prepared, &cancel).unwrap();
    let parent = dom
        .snapshot()
        .document(parent_document)
        .unwrap()
        .document_element()
        .unwrap();
    let mut batch = DomBatch::new();
    let closed = batch.create_html_document(parent_document, None);
    let frame = batch.create_element_ns(&closed, surgeist_html::values::Namespace::html(), "frame");
    let frame = dom
        .apply(batch, &cancel)
        .unwrap()
        .created(&frame)
        .unwrap()
        .unwrap();
    let small = DomSourceLimits::new(64_000_000, 4_000_000, 100_000, 128, 4096, 4096, 4);
    let mut lease = dom
        .begin_source_edit(
            DomSourceEdit::insert_before(frame, parent, None),
            small,
            &cancel,
        )
        .unwrap();
    let operation_id = lease.operation();
    let DomSourceProgress::Request(DomSourceRequest::HostedChild(child)) =
        lease.advance(&cancel).unwrap()
    else {
        panic!("real hosted child")
    };
    let preparation = lease
        .prepare_hosted_child(&mut source, &child, &cancel)
        .unwrap();
    let mut prepared = Some(
        source
            .seal_child_preparation(&child, &mut Some(preparation), &cancel)
            .unwrap(),
    );
    lease
        .accept_hosted_child_preparation(&mut source, &child, &mut prepared, &cancel)
        .unwrap();
    let DomHostedChildStep::PermissionsPolicy(request) = lease.hosted_child_step(&child).unwrap()
    else {
        panic!("policy before execution")
    };
    let mut policy = Some(
        source
            .seal_permissions_policy(
                request,
                DomInitialPermissionsPolicy::new(
                    DomPermissionsPolicyValue::Enabled,
                    DomPermissionsPolicyValue::Enabled,
                ),
                &cancel,
            )
            .unwrap(),
    );
    lease
        .accept_hosted_child_policy(&mut source, &child, &mut policy, &cancel)
        .unwrap();
    let DomHostedChildStep::NativeExecution(request) = lease.hosted_child_step(&child).unwrap()
    else {
        panic!("actual native execution")
    };
    let global = source.issue_global(request, &cancel).unwrap();
    let mut publication = Some(
        source
            .seal_native_execution(request, global, 200, &cancel)
            .unwrap(),
    );
    let before = lease.snapshot().stamp();
    let issued = {
        let operation = lease.dom.source_operation.as_mut().unwrap();
        let actual_child = operation.child.as_mut().unwrap();
        let Phase::Creating(inner) = &actual_child.phase else {
            panic!("real creation phase")
        };
        let inherited = operation
            .draft
            .inherit_child_policy(&lease.dom.csp_owner, &source, inner, &cancel)
            .unwrap();
        let crate::DomDocumentCspState::Associated(value) = inherited.csp.view() else {
            panic!("inheritance")
        };
        let issued = value.document().clone();
        assert_ne!(&issued, parent_policy.document());
        assert_eq!(value.policy(), parent_policy.policy());
        actual_child.inherited_policy = Some(inherited);
        // Spend this same real original counter; this is not a fake publication
        // failure or a public hook. The already formed destination must survive.
        loop {
            match operation.draft.source_step(&cancel) {
                Ok(()) => (),
                Err(DomError::SourceWorkLimit { .. }) => break,
                Err(error) => panic!("unexpected original admission: {error:?}"),
            }
        }
        issued
    };
    assert!(matches!(
        lease.accept_hosted_child_publication(&mut source, &child, &mut publication, &cancel),
        Err(DomHostedError::Dom(DomError::SourceWorkLimit { .. }))
    ));
    assert!(publication.is_some());
    assert_eq!(lease.snapshot().stamp(), before);
    let spent = lease.status().spent_work();
    drop(lease);
    let mut lease = dom
        .resume_source_edit(operation_id, limits(), &cancel)
        .unwrap();
    lease
        .accept_hosted_child_publication(&mut source, &child, &mut publication, &cancel)
        .unwrap();
    assert!(publication.is_none());
    assert!(lease.status().spent_work() > spent);
    let DomHostedChildStep::MakeActive(request) = lease.hosted_child_step(&child).unwrap() else {
        panic!("published child")
    };
    let snapshot = lease.snapshot();
    let metadata = snapshot.document(request.document()).unwrap();
    let crate::DomDocumentCspState::Associated(value) =
        metadata.metadata().policy_container().csp()
    else {
        panic!("canonical inherited policy")
    };
    assert_eq!(value.document(), &issued, "raised retry must not reissue");
    assert_eq!(value.policy(), parent_policy.policy());
    let policy = value.policy();
    let HtmlCspNativePolicy::External(native) = policy
        .borrow_native_with::<InheritedNative, _>(policy.profile().unwrap(), &mut allow)
        .unwrap()
    else {
        panic!("real inherited artifact")
    };
    assert_eq!(
        native.text,
        DocumentString::from("fixture inherited policy")
    );
    assert_eq!(native.padding, [71; 1024]);
    assert!(
        metadata.metadata().retained_payload_bytes().unwrap() >= 1024,
        "canonical admission includes native backing, not only its Arc handle"
    );
    assert!(weak.upgrade().is_some());
    assert_eq!(
        value.origin().transferred_from(),
        Some((
            parent_policy.document(),
            parent_policy.generation(),
            HtmlCspAssociationCause::Inheritance
        ))
    );
}

/// Where the construction of a hosted child stops for a measurement.
#[derive(Clone, Copy)]
enum Stop {
    /// Before the Permissions Policy answer, with the inputs of the embedder retained.
    Policy,
    /// After the initial entry and its history are accepted.
    History,
    /// At the Permissions Policy answer, after the declared-origin URL request of the iframe is
    /// issued.
    Url,
    /// At the same answer, after the host's observation of that request is accepted.
    Observed,
}

fn spare_block() -> surgeist_html::values::TextBlock {
    let mut units = Vec::with_capacity(512);
    units.extend([97_u16, 98, 99, 100, 101, 102, 103, 104]);
    surgeist_html::values::TextBlock::from_utf16(units)
}

fn exact_owner() -> surgeist_html::values::DocumentString {
    let mut units = Vec::new();
    units.try_reserve_exact(4).unwrap();
    units.extend([97_u16, 98, 99, 100]);
    surgeist_html::values::DocumentString::from_utf16(units)
}

/// The custody total of the store, and the number of blocks it charges, while the construction of a
/// hosted `iframe` with the given attributes is held at `stop`.
fn total_at(
    attributes: &[(&str, &surgeist_html::values::DocumentString)],
    stop: Stop,
) -> (crate::store::custody::Usage, usize) {
    run_to(attributes, stop, None, None)
}

/// A point of the construction before which a test drops the listing of a block, so that the
/// retention made there is the only path that lists it.
#[derive(Clone, Copy, PartialEq)]
enum Isolate {
    /// Before the inputs of the embedder are retained.
    Policy,
    /// Before the URL request of the iframe is issued.
    Request,
}

/// What the host's observation of the URL request of the iframe carries as its raw value, when
/// not the request's own.
type Raw<'a> = Option<&'a surgeist_html::values::DocumentString>;

/// [`total_at`], with the listing of a block dropped at a point of the construction, and the raw
/// value of the host's observation given.
fn run_to(
    attributes: &[(&str, &surgeist_html::values::DocumentString)],
    stop: Stop,
    isolate: Option<(Isolate, &surgeist_html::values::TextBlock)>,
    observed: Raw<'_>,
) -> (crate::store::custody::Usage, usize) {
    let cancel = AtomicBool::new(false);
    let (mut dom, mut source, document, _) = super::super::fullscreen::native_fixture(
        DomLimits::new(
            128,
            1_000_000,
            256,
            128,
            128,
            32,
            4_000_000,
            crate::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
        )
        .unwrap(),
    );
    let parent = dom
        .snapshot()
        .document(document)
        .unwrap()
        .document_element()
        .unwrap();
    let mut batch = DomBatch::new();
    let closed = batch.create_html_document(document, None);
    let iframe =
        batch.create_element_ns(&closed, surgeist_html::values::Namespace::html(), "iframe");
    for (name, value) in attributes {
        batch.set_attribute(&iframe, *name, (*value).clone());
    }
    let iframe = dom
        .apply(batch, &cancel)
        .unwrap()
        .created(&iframe)
        .unwrap()
        .unwrap();
    let mut lease = dom
        .begin_source_edit(
            DomSourceEdit::insert_before(iframe, parent, None),
            limits(),
            &cancel,
        )
        .unwrap();
    let DomSourceProgress::Request(DomSourceRequest::HostedChild(child)) =
        lease.advance(&cancel).unwrap()
    else {
        panic!("actual connected iframe reaches its child constructor");
    };
    let preparation = lease
        .prepare_hosted_child(&mut source, &child, &cancel)
        .unwrap();
    let mut prepared = Some(
        source
            .seal_child_preparation(&child, &mut Some(preparation), &cancel)
            .unwrap(),
    );
    if let Some((Isolate::Policy, block)) = isolate {
        lease.dom.custody_registry.unlist_block(block);
        assert_eq!(lease.dom.custody_registry.listed_blocks(), 0);
    }
    lease
        .accept_hosted_child_preparation(&mut source, &child, &mut prepared, &cancel)
        .unwrap();
    for _ in 0..12 {
        let at_policy = matches!(
            lease.hosted_child_step(&child).unwrap(),
            DomHostedChildStep::PermissionsPolicy(_)
        );
        if at_policy && matches!(stop, Stop::Url | Stop::Observed) {
            let revision = surgeist_html_semantics::HtmlBaseFactsRevision::new(
                surgeist_html_semantics::HtmlBaseProducerId::fresh(),
                21,
                22,
                23,
            );
            if let Some((Isolate::Request, block)) = isolate {
                lease.dom.custody_registry.unlist_block(block);
            }
            let issued = lease
                .source_url_request(
                    &child,
                    crate::DomSourceUrlStage::IframeDeclaredOrigin,
                    &revision,
                    &cancel,
                )
                .unwrap();
            if matches!(stop, Stop::Observed) {
                // The host observes the value the request carries, as the Runtime does: its raw
                // value is the clone of the request's.
                let actual = surgeist_html_semantics::HtmlBaseParseObservation::new(
                    observed.unwrap_or(issued.raw()).clone(),
                    issued.base_url().clone(),
                    issued.encoding().clone(),
                    revision.clone(),
                    surgeist_html_semantics::HtmlBaseParseOutcome::Failure,
                )
                .unwrap();
                let mut response = Some(issued.observe(actual, None).unwrap());
                lease
                    .accept_source_url_observation(&child, &issued, &mut response, &cancel)
                    .unwrap();
                assert!(response.is_none());
            }
            return (
                lease.dom.custody_registry.ledger().total(),
                lease.dom.custody_registry.listed_blocks(),
            );
        }
        let publication = match lease.hosted_child_step(&child).unwrap() {
            DomHostedChildStep::PermissionsPolicy(request) => {
                if matches!(stop, Stop::Policy) {
                    return (
                        lease.dom.custody_registry.ledger().total(),
                        lease.dom.custody_registry.listed_blocks(),
                    );
                }
                let mut policy = Some(
                    source
                        .seal_permissions_policy(
                            request,
                            DomInitialPermissionsPolicy::new(
                                DomPermissionsPolicyValue::Enabled,
                                DomPermissionsPolicyValue::Enabled,
                            ),
                            &cancel,
                        )
                        .unwrap(),
                );
                lease
                    .accept_hosted_child_policy(&mut source, &child, &mut policy, &cancel)
                    .unwrap();
                continue;
            }
            DomHostedChildStep::NativeExecution(request) => {
                let global = source.issue_global(request, &cancel).unwrap();
                source
                    .seal_native_execution(request, global, 200, &cancel)
                    .unwrap()
            }
            DomHostedChildStep::MakeActive(request) => {
                source.seal_make_active(request, &cancel).unwrap()
            }
            DomHostedChildStep::FinishLoading(request) => {
                source.seal_finish_loading(request, 300, &cancel).unwrap()
            }
            DomHostedChildStep::InitialEntry(request) => {
                let entry = source.issue_entry(request, &cancel).unwrap();
                let navigable = source.issue_navigable(request, &cancel).unwrap();
                let mut initialization = Some(DomHostedEntryInitialization::new(
                    uuid(u16::from(b'3')),
                    uuid(u16::from(b'4')),
                ));
                source
                    .seal_initial_entry(request, entry, navigable, &mut initialization, &cancel)
                    .unwrap()
            }
            DomHostedChildStep::QueueHistory(request) => {
                let mut publication = Some(source.seal_child_history(request, &cancel).unwrap());
                lease
                    .accept_hosted_child_history(&mut source, &child, &mut publication, &cancel)
                    .unwrap();
                assert!(publication.is_none());
                if matches!(stop, Stop::History) {
                    return (
                        lease.dom.custody_registry.ledger().total(),
                        lease.dom.custody_registry.listed_blocks(),
                    );
                }
                continue;
            }
            _ => panic!("unexpected child phase"),
        };
        let mut value = Some(publication);
        lease
            .accept_hosted_child_publication(&mut source, &child, &mut value, &cancel)
            .unwrap();
    }
    panic!("the construction did not reach the stop");
}

/// A borrower of one block and an exact owner of the same four units give totals that differ by
/// the capacity of the block alone, and charge the block once.
fn assert_charged_by_length_and_the_block_once(
    names: &[&str],
    others: &[(&str, &surgeist_html::values::DocumentString)],
    stop: Stop,
) {
    let block = spare_block();
    assert!(block.capacity() >= 512);
    let borrowed = block.string(0..4).unwrap();
    let owner = exact_owner();
    let with = |value: &surgeist_html::values::DocumentString| {
        let mut attributes = others.to_vec();
        for name in names {
            attributes.push((*name, value));
        }
        total_at(&attributes, stop)
    };
    let (shared, listed) = with(&borrowed);
    let (owned, none) = with(&owner);
    assert_eq!(shared.units(), owned.units() + block.capacity());
    assert_eq!((listed, none), (1, 0));
}

#[test]
fn the_embedder_policy_inputs_of_an_iframe_that_borrow_a_block_are_charged_by_length() {
    assert_charged_by_length_and_the_block_once(&["sandbox", "srcdoc", "allow"], &[], Stop::Policy);
}

#[test]
fn the_name_an_iframe_history_entry_keeps_is_charged_by_length_when_it_borrows_a_block() {
    assert_charged_by_length_and_the_block_once(&["name"], &[], Stop::History);
}

#[test]
fn the_url_basis_of_an_iframe_source_that_borrows_a_block_is_charged_by_length() {
    let block = spare_block();
    assert!(block.capacity() >= 512);
    let borrowed = block.string(0..4).unwrap();
    let owner = exact_owner();
    // What the request adds beyond the construction held at the answer: the raw source and the
    // base the basis keeps.
    let added = |value: &surgeist_html::values::DocumentString| {
        let (policy, _) = total_at(&[("src", value)], Stop::Policy);
        let (url, listed) = total_at(&[("src", value)], Stop::Url);
        (url.units() - policy.units(), listed)
    };
    let (shared, listed) = added(&borrowed);
    let (owned, none) = added(&owner);
    assert_eq!(shared, owned);
    assert_eq!((listed, none), (1, 0));
}

#[test]
fn the_observation_the_host_returns_for_an_iframe_source_that_borrows_a_block_is_charged_by_length()
{
    let block = spare_block();
    assert!(block.capacity() >= 512);
    let borrowed = block.string(0..4).unwrap();
    let owner = exact_owner();
    // What the accepted observation adds beyond the request: the raw value it repeats.
    let added = |value: &surgeist_html::values::DocumentString| {
        let (url, _) = total_at(&[("src", value)], Stop::Url);
        let (observed, listed) = total_at(&[("src", value)], Stop::Observed);
        (observed.units() - url.units(), listed)
    };
    let (shared, listed) = added(&borrowed);
    let (owned, none) = added(&owner);
    assert_eq!(shared, owned);
    assert_eq!((listed, none), (1, 0));
}

#[test]
fn the_embedder_policy_inputs_list_the_block_they_borrow_for_the_operation() {
    let block = spare_block();
    let borrowed = block.string(0..4).unwrap();
    let attributes = [
        ("sandbox", &borrowed),
        ("srcdoc", &borrowed),
        ("allow", &borrowed),
    ];
    let (listed, count) = run_to(
        &attributes,
        Stop::Policy,
        Some((Isolate::Policy, &block)),
        None,
    );
    // The block is charged once again, as when the attribute bank listed it.
    let (kept, kept_count) = total_at(&attributes, Stop::Policy);
    assert_eq!((count, kept_count), (1, 1));
    assert_eq!(listed.units(), kept.units());
}

#[test]
fn the_url_basis_of_an_iframe_source_lists_the_block_it_borrows_for_the_operation() {
    let block = spare_block();
    let borrowed = block.string(0..4).unwrap();
    let attributes = [("src", &borrowed)];
    let (listed, count) = run_to(
        &attributes,
        Stop::Url,
        Some((Isolate::Request, &block)),
        None,
    );
    let (kept, kept_count) = total_at(&attributes, Stop::Url);
    assert_eq!((count, kept_count), (1, 1));
    assert_eq!(listed.units(), kept.units());
}

#[test]
fn an_observation_whose_raw_value_borrows_another_block_lists_that_block_for_the_operation() {
    let block = spare_block();
    let borrowed = block.string(0..4).unwrap();
    // The host builds the observation from its own copy of the value, which is a range of a
    // block of its own that no earlier path of the store has listed.
    let other = spare_block();
    let echo = other.string(0..4).unwrap();
    assert_eq!(echo, borrowed);
    let attributes = [("src", &borrowed)];
    let (own, own_count) = total_at(&attributes, Stop::Observed);
    let (foreign, foreign_count) = run_to(&attributes, Stop::Observed, None, Some(&echo));
    assert_eq!((own_count, foreign_count), (1, 2));
    // The second block is charged once, at its capacity, and the observation by its length.
    assert_eq!(foreign.units(), own.units() + other.capacity());
}
