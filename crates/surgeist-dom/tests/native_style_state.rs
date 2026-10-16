//! Source-selected native Style intrinsic and atomic boundary behavior.
use std::sync::atomic::AtomicBool;
use surgeist_dom::{Dom, DomBatch, DomError, DomLimits, Namespace};
use surgeist_html::HtmlStyleUpdateAction;

#[test]
fn unstarted_command_payload_joins_the_simultaneous_utf16_allowance() {
    let cancel = AtomicBool::new(false);
    let mut dom = dom();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let style = batch.create_element_ns(&document, Namespace::html(), "style");
    let commit = dom.apply(batch, &cancel).unwrap();
    let document = commit.created(&document).unwrap().unwrap();
    let style = commit.created(&style).unwrap().unwrap();
    let before = dom.snapshot();
    let usage = dom.usage();
    let mut units = Vec::with_capacity(16_384);
    units.push(0xd800);
    let capacity = units.capacity();
    let text = surgeist_dom::DocumentString::from_utf16(units);
    let commands = || {
        let mut batch = DomBatch::new();
        batch.insert_before(style, document, None);
        // This command has not begun when the Style reaches its first boundary.
        batch.create_text(document, text.clone());
        batch
    };
    assert!(matches!(
        dom.begin_style_batch(
            commands(),
            surgeist_dom::DomStyleEditLimits::new(
                16_000_000,
                capacity - 1,
                10_000_000,
                128,
                4096,
                4096,
            ),
            &cancel,
        ),
        Err(DomError::ResourceLimit {
            resource: surgeist_dom::DomResource::StyleRetainedUnits,
            ..
        })
    ));
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    assert_eq!(dom.usage(), usage);
    assert!(dom.pending_style_edit().is_none());
    let mut lease = dom
        .begin_style_batch(commands(), source_limits(), &cancel)
        .unwrap();
    let request = match lease.advance(&cancel).unwrap() {
        surgeist_dom::DomStyleProgress::Request(request) => request,
        other => panic!("expected the first Style boundary, got {other:?}"),
    };
    assert_eq!(
        request.action(),
        surgeist_dom::DomStyleAction::Update(HtmlStyleUpdateAction::CheckInlineCsp)
    );
}

#[test]
fn first_external_phase_reserves_its_required_closing_publication() {
    let cancel = AtomicBool::new(false);
    let mut dom = dom();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let commit = dom.apply(batch, &cancel).unwrap();
    let document = commit.created(&document).unwrap().unwrap();
    let before = dom.snapshot();
    let usage = dom.usage();
    let mut batch = DomBatch::new();
    let style = batch.create_element_ns(document, Namespace::html(), "style");
    batch.insert_before(&style, document, None);
    let limits =
        surgeist_dom::DomStyleEditLimits::new(16_000_000, 1_000_000, 10_000_000, 1, 4096, 4096);
    let mut lease = dom.begin_style_batch(batch, limits, &cancel).unwrap();
    assert_eq!(
        lease.advance(&cancel).unwrap_err(),
        DomError::StylePublicationLimit
    );
    assert_eq!(lease.snapshot().stamp(), before.stamp());
    assert!(!lease.status().has_published());
    drop(lease);
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    assert_eq!(dom.usage(), usage);
    assert!(dom.pending_style_edit().is_none());
}

#[test]
fn unrelated_actual_attribute_backing_is_admitted_with_the_retained_source_operation() {
    fn minimum_bytes(spare: usize) -> (usize, usize) {
        let mut low = 0;
        let mut high = 1_000_000;
        let mut actual_capacity = 0;
        while low < high {
            let allowance = low + (high - low) / 2;
            let cancel = AtomicBool::new(false);
            let mut dom = dom();
            let mut batch = DomBatch::new();
            let document = batch.create_document();
            let parent = batch.create_element_ns(&document, Namespace::html(), "div");
            let style = batch.create_element_ns(&document, Namespace::html(), "style");
            let mut raw = Vec::with_capacity(spare);
            raw.push(b'x' as u16);
            batch.set_attribute(
                &parent,
                "data-retained",
                surgeist_dom::DocumentString::from_utf16(raw),
            );
            batch.insert_before(&parent, &document, None);
            let commit = dom.apply(batch, &cancel).unwrap();
            let parent = commit.created(&parent).unwrap().unwrap();
            let style = commit.created(&style).unwrap().unwrap();
            let before = dom.snapshot();
            actual_capacity = before
                .node(parent)
                .unwrap()
                .as_element()
                .unwrap()
                .attributes()
                .iter()
                .find(|attribute| {
                    attribute
                        .name()
                        .local_name()
                        .code_units()
                        .iter()
                        .copied()
                        .eq("data-retained".encode_utf16())
                })
                .unwrap()
                .value()
                .shared_code_units()
                .capacity();
            let mut batch = DomBatch::new();
            batch.insert_before(style, parent, None);
            let limits = surgeist_dom::DomStyleEditLimits::new(
                allowance, 1_000_000, 10_000_000, 128, 4096, 4096,
            );
            let admitted = match dom.begin_style_batch(batch, limits, &cancel) {
                Ok(mut lease) => match lease.advance(&cancel) {
                    Ok(surgeist_dom::DomStyleProgress::Request(_)) => true,
                    Err(DomError::ResourceLimit {
                        resource: surgeist_dom::DomResource::StyleRetainedBytes,
                        ..
                    }) => false,
                    other => panic!("unexpected admission: {other:?}"),
                },
                Err(DomError::ResourceLimit {
                    resource: surgeist_dom::DomResource::StyleRetainedBytes,
                    ..
                }) => false,
                Err(other) => panic!("unexpected admission: {other:?}"),
            };
            if admitted {
                high = allowance;
            } else {
                assert_eq!(dom.snapshot().stamp(), before.stamp());
                assert_eq!(dom.snapshot().node(style).unwrap().parent(), None);
                low = allowance + 1;
            }
        }
        (actual_capacity, low)
    }
    let (compact_capacity, compact) = minimum_bytes(1);
    let (padded_capacity, padded) = minimum_bytes(8192);
    assert!(padded_capacity > compact_capacity);
    assert!(padded - compact >= (padded_capacity - compact_capacity) * std::mem::size_of::<u16>());
}

fn dom() -> Dom {
    Dom::new(DomLimits::new(128, 32768, 512, 0, 0, 16, 1_000_000).unwrap()).unwrap()
}

#[test]
fn eligible_connected_style_refuses_atomically_at_actual_inline_policy_step() {
    let cancel = AtomicBool::new(false);
    let mut dom = dom();
    let mut b = DomBatch::new();
    let d = b.create_document();
    let parent = b.create_element_ns(&d, Namespace::html(), "div");
    let style = b.create_element_ns(&d, Namespace::html(), "style");
    b.insert_before(&parent, &d, None);
    let c = dom.apply(b, &cancel).unwrap();
    let parent = c.created(&parent).unwrap().unwrap();
    let style = c.created(&style).unwrap().unwrap();
    let before = dom.snapshot();
    let usage = dom.usage();
    let mut b = DomBatch::new();
    b.insert_before(style, parent, None);
    assert_eq!(
        dom.apply(b, &cancel).unwrap_err(),
        DomError::StyleCompositionRequired(HtmlStyleUpdateAction::CheckInlineCsp)
    );
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    assert_eq!(dom.usage(), usage);
    assert_eq!(dom.snapshot().node(style).unwrap().parent(), None);
    assert!(
        dom.changes_since(before.stamp())
            .unwrap()
            .changes()
            .is_empty()
    );
}

#[test]
fn equal_comment_data_reaches_style_but_nested_text_and_type_attribute_alone_do_not() {
    let cancel = AtomicBool::new(false);
    let mut dom = dom();
    let mut b = DomBatch::new();
    let d = b.create_document();
    let style = b.create_element_ns(&d, Namespace::html(), "style");
    b.set_attribute(&style, "type", "text/plain");
    let comment = b.create_comment(&d, "unchanged");
    let span = b.create_element_ns(&d, Namespace::html(), "span");
    let nested = b.create_text(&d, "nested");
    b.insert_before(&nested, &span, None);
    b.insert_before(&span, &style, None);
    b.insert_before(&comment, &style, None);
    b.insert_before(&style, &d, None);
    let c = dom.apply(b, &cancel).unwrap();
    let style = c.created(&style).unwrap().unwrap();
    let comment = c.created(&comment).unwrap().unwrap();
    let nested = c.created(&nested).unwrap().unwrap();
    let mut b = DomBatch::new();
    b.set_attribute(style, "type", "text/css");
    b.replace_data(nested, 0, 6, "changed nested");
    dom.apply(b, &cancel).unwrap();
    let before = dom.snapshot();
    let mut b = DomBatch::new();
    b.replace_data(comment, 0, 9, "unchanged");
    assert_eq!(
        dom.apply(b, &cancel).unwrap_err(),
        DomError::StyleCompositionRequired(HtmlStyleUpdateAction::CheckInlineCsp)
    );
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    // Removal's actual disconnected guard returns before policy or creation.
    let mut b = DomBatch::new();
    b.detach(style);
    dom.apply(b, &cancel).unwrap();
    assert!(!dom.snapshot().node(style).unwrap().is_connected());
}

fn source_limits() -> surgeist_dom::DomStyleEditLimits {
    surgeist_dom::DomStyleEditLimits::new(16_000_000, 100_000, 10_000_000, 64, 4096, 4096)
}
fn cssom() -> surgeist_cssom::CssomStore {
    surgeist_cssom::CssomStore::new(Default::default(), Default::default()).unwrap()
}
fn block(request: &surgeist_dom::DomStyleRequest) -> surgeist_dom::DomStyleInlineCspObservation {
    // A supplied full CSP evaluation, not a local CSP interpreter or inference
    // from the native environment. This exact policy prohibits inline styles.
    surgeist_dom::DomStyleInlineCspObservation::evaluated(
        request.source(),
        std::sync::Arc::new(vec!["style-src 'none'".into()]),
        19,
        false,
    )
}

#[test]
fn lease_drop_before_publication_discards_only_the_private_candidate() {
    let cancel = AtomicBool::new(false);
    let mut dom = dom();
    let before = dom.snapshot();
    let mut b = DomBatch::new();
    b.create_document();
    let lease = dom.begin_style_batch(b, source_limits(), &cancel).unwrap();
    drop(lease);
    assert!(dom.pending_style_edit().is_none());
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    assert_eq!(dom.snapshot().nodes().count(), 0);
}

#[test]
fn published_source_parks_exact_tail_and_cancel_discards_only_later_commands() {
    use surgeist_dom::{DomStyleOperationId, DomStyleProgress};
    let cancel = AtomicBool::new(false);
    let mut dom = dom();
    let mut b = DomBatch::new();
    let document = b.create_document();
    let style = b.create_element_ns(&document, Namespace::html(), "style");
    b.insert_before(&style, &document, None);
    b.set_attribute(&style, "unstarted", "must not run");
    let operation: DomStyleOperationId;
    let request;
    {
        let mut lease = dom.begin_style_batch(b, source_limits(), &cancel).unwrap();
        operation = lease.operation();
        let DomStyleProgress::Request(r) = lease.advance(&cancel).unwrap() else {
            panic!("actual connected Style requires CSP");
        };
        request = r;
        assert_eq!(
            request.action(),
            surgeist_dom::DomStyleAction::Update(HtmlStyleUpdateAction::CheckInlineCsp)
        );
        assert!(lease.snapshot().incomplete_source_operation().is_some());
        assert!(matches!(
            lease
                .snapshot()
                .node(request.source().element())
                .unwrap()
                .as_style()
                .unwrap()
                .sheet(),
            Err(DomError::IncompleteSourceOperation)
        ));
    }
    assert!(dom.pending_style_edit().unwrap().has_published());
    assert!(matches!(
        dom.apply(DomBatch::new(), &cancel),
        Err(DomError::SourceOperationPending)
    ));
    let mut lease = dom
        .resume_style_edit(operation, source_limits(), &cancel)
        .unwrap();
    let DomStyleProgress::Request(reissued) = lease.advance(&cancel).unwrap() else {
        panic!("retained request");
    };
    assert_eq!(reissued.source().ordinal(), request.source().ordinal());
    assert_eq!(reissued.source().element(), request.source().element());
    lease.request_cancel();
    lease
        .accept_inline_csp(&reissued, &block(&reissued), &cancel)
        .unwrap();
    let DomStyleProgress::Complete(done) = lease.advance(&cancel).unwrap() else {
        panic!("blocked source completes");
    };
    assert!(done.cancellation_requested());
    let id = done.published_creation(&style).unwrap().unwrap();
    assert_eq!(id, request.source().element());
    drop(lease);
    assert!(dom.pending_style_edit().is_none());
    assert!(dom.snapshot().incomplete_source_operation().is_none());
    let snapshot = dom.snapshot();
    let element = snapshot.node(id).unwrap().as_element().unwrap();
    assert!(
        !element
            .attributes()
            .iter()
            .any(|a| a.name().local_name().code_units()
                == "unstarted".encode_utf16().collect::<Vec<_>>())
    );
    assert_eq!(
        snapshot
            .node(id)
            .unwrap()
            .as_style()
            .unwrap()
            .sheet()
            .unwrap(),
        None
    );
}

#[test]
fn same_parent_text_move_keeps_two_source_time_direct_text_captures() {
    use surgeist_dom::DomStyleProgress;
    let cancel = AtomicBool::new(false);
    let mut dom = dom();
    let mut b = DomBatch::new();
    let d = b.create_document();
    let s = b.create_element_ns(&d, Namespace::html(), "style");
    b.set_attribute(&s, "type", "text/plain");
    let a = b.create_text(&d, "A");
    let c = b.create_text(&d, "B");
    let span = b.create_element_ns(&d, Namespace::html(), "span");
    let nested = b.create_text(&d, "EXCLUDED");
    b.insert_before(&nested, &span, None);
    b.insert_before(&span, &s, None);
    b.insert_before(&a, &s, None);
    b.insert_before(&c, &s, None);
    b.insert_before(&s, &d, None);
    let commit = dom.apply(b, &cancel).unwrap();
    let s = commit.created(&s).unwrap().unwrap();
    let a = commit.created(&a).unwrap().unwrap();
    let mut b = DomBatch::new();
    b.set_attribute(s, "type", "text/css");
    b.insert_before(a, s, None);
    let mut lease = dom.begin_style_batch(b, source_limits(), &cancel).unwrap();
    let mut captured = Vec::new();
    while let DomStyleProgress::Request(r) = lease.advance(&cancel).unwrap() {
        captured.push(String::from_utf16(r.source().text()).unwrap());
        lease.accept_inline_csp(&r, &block(&r), &cancel).unwrap();
    }
    assert_eq!(captured, ["B", "BA"]);
}

#[test]
fn acknowledged_real_cssom_creation_survives_failed_advance_lease_drop_and_thread_handoff() {
    actual_creation_receipt(false, false, None);
}
#[test]
fn pending_cssom_owner_effects_return_the_actual_publication_without_acknowledging_source() {
    actual_creation_receipt(true, false, None);
}
#[test]
fn disabled_idl_uses_the_actual_existing_sheet_without_recreation_or_notification() {
    actual_creation_receipt(false, true, None);
}
#[derive(Clone, Copy, Debug)]
enum WrongCreationInput {
    Disabled,
    Title,
    Media,
    Base,
    Location,
    ConstructorDocument,
    OwnerMedia,
}
#[test]
fn actual_creation_rejects_wrong_disabled_without_consuming_the_publication() {
    actual_creation_receipt(false, false, Some(WrongCreationInput::Disabled));
}
#[test]
fn actual_creation_rejects_wrong_title_without_consuming_the_publication() {
    actual_creation_receipt(false, false, Some(WrongCreationInput::Title));
}
#[test]
fn actual_creation_rejects_wrong_media_without_consuming_the_publication() {
    actual_creation_receipt(false, false, Some(WrongCreationInput::Media));
}
#[test]
fn actual_creation_rejects_wrong_ordinary_constructor_fields() {
    for wrong in [
        WrongCreationInput::Base,
        WrongCreationInput::Location,
        WrongCreationInput::ConstructorDocument,
    ] {
        actual_creation_receipt(false, false, Some(wrong));
    }
}
#[test]
fn actual_owner_attributes_reject_wrong_media_and_preserve_the_publication() {
    actual_creation_receipt(false, false, Some(WrongCreationInput::OwnerMedia));
}
fn actual_creation_receipt(
    with_owner_effect: bool,
    exercise_disabled: bool,
    wrong: Option<WrongCreationInput>,
) {
    use std::sync::Arc;
    use surgeist_css::{CssParserContext, CssUtf16InputLimits};
    use surgeist_cssom::{CssomEditGuard, CssomInputRole, CssomInputVersion, CssomSheetInputs};
    use surgeist_dom::{DomStyleCssomOwnerBinding, DomStyleInlineCspObservation, DomStyleProgress};
    let cancel = AtomicBool::new(false);
    let mut dom = dom();
    let mut b = DomBatch::new();
    let document = b.create_document();
    let style = b.create_element_ns(&document, Namespace::html(), "style");
    if exercise_disabled {
        // The source's pre-insertion assignment has no sheet to mutate and
        // cannot become a stored flag for the later-created sheet.
        b.set_style_disabled(&style, true);
    }
    let text = b.create_text(&document, "a { color: red }");
    b.insert_before(&text, &style, None);
    b.insert_before(&style, &document, None);
    let mut lease = dom.begin_style_batch(b, source_limits(), &cancel).unwrap();
    let operation = lease.operation();
    let DomStyleProgress::Request(policy) = lease.advance(&cancel).unwrap() else {
        panic!("policy")
    };
    lease
        .accept_inline_csp(
            &policy,
            &DomStyleInlineCspObservation::complete_empty(policy.source(), 1),
            &cancel,
        )
        .unwrap();
    let DomStyleProgress::Request(create) = lease.advance(&cancel).unwrap() else {
        panic!("creation")
    };
    assert_eq!(
        create.action(),
        surgeist_dom::DomStyleAction::Update(HtmlStyleUpdateAction::CreateSheet)
    );
    let mut owner_store = cssom();
    let effect_block = if with_owner_effect {
        let mut setup = owner_store
            .batch(CssomEditGuard::from_snapshot(&owner_store.snapshot()))
            .unwrap();
        let ticket = setup
            .create_declarations(
                surgeist_css::CssDeclarationList::try_new(Vec::new()).unwrap(),
                false,
                false,
                Some(CssomInputVersion {
                    role: CssomInputRole::Owner,
                    identity: "actual-element-style".into(),
                    revision: 1,
                }),
            )
            .unwrap();
        Some(
            owner_store
                .commit(setup)
                .unwrap()
                .block(&ticket)
                .unwrap()
                .clone(),
        )
    } else {
        None
    };
    let owner = surgeist_dom::DomStyleCssomRoot::new(create.cssom_root().unwrap(), owner_store);
    assert_eq!(
        lease
            .qualify_cssom(&create, owner.root(), &owner.snapshot().unwrap())
            .unwrap_err(),
        DomError::StyleReceiptMismatch
    );
    cancel.store(true, std::sync::atomic::Ordering::Relaxed);
    assert_eq!(
        lease.bind_cssom_root(&create, &owner, &cancel).unwrap_err(),
        DomError::Cancelled
    );
    cancel.store(false, std::sync::atomic::Ordering::Relaxed);
    lease.bind_cssom_root(&create, &owner, &cancel).unwrap();
    let foreign = surgeist_dom::DomStyleCssomRoot::new(owner.root(), cssom());
    assert_eq!(
        lease
            .bind_cssom_root(&create, &foreign, &cancel)
            .unwrap_err(),
        DomError::StyleReceiptMismatch
    );
    let create = lease
        .qualify_cssom(
            &create,
            create.cssom_root().unwrap(),
            &owner.snapshot().unwrap(),
        )
        .unwrap();
    let version = CssomInputVersion {
        role: CssomInputRole::Owner,
        identity: "actual-style-owner".into(),
        revision: 1,
    };
    let binding =
        DomStyleCssomOwnerBinding::new(&create, version.clone(), "inline-owner".into()).unwrap();
    lease.bind_cssom_owner(&create, binding, &cancel).unwrap();
    let context = CssParserContext::default();
    let body = context
        .parse_sheet_utf16_with_limits(
            create.source().text().clone(),
            CssUtf16InputLimits::default(),
            || Ok::<_, ()>(()),
        )
        .unwrap();
    let media = context
        .parse_media_query_list_utf16_with_limits(
            Arc::new(if matches!(wrong, Some(WrongCreationInput::Media)) {
                "print".encode_utf16().collect()
            } else {
                Vec::new()
            }),
            CssUtf16InputLimits::default(),
            || Ok::<_, ()>(()),
        )
        .unwrap();
    let mut inputs = CssomSheetInputs::external(version, true);
    inputs.owner = Some("inline-owner".into());
    if matches!(wrong, Some(WrongCreationInput::Disabled)) {
        inputs.disabled = true;
    }
    if matches!(wrong, Some(WrongCreationInput::Title)) {
        inputs.title = Arc::new("wrong title".encode_utf16().collect());
    }
    match wrong {
        Some(WrongCreationInput::Base) => inputs.base_url = Some("wrong base".into()),
        Some(WrongCreationInput::Location) => inputs.location = Some("wrong location".into()),
        Some(WrongCreationInput::ConstructorDocument) => {
            inputs.constructor_document = Some("wrong document".into())
        }
        _ => (),
    }
    let mut edit = owner
        .batch(CssomEditGuard::from_snapshot(&owner.snapshot().unwrap()))
        .unwrap();
    let construction = edit
        .create_parsed_sheet_with_media_reports(body, media, inputs)
        .unwrap();
    if let Some(block) = &effect_block {
        let preparation = edit
            .prepare_css_text(block, "", Default::default())
            .unwrap();
        edit.apply_declaration(preparation, &[]).unwrap();
    }
    let commit = owner.commit(&mut Some(edit)).unwrap();
    let sheet = commit.sheet(construction.ticket()).unwrap().clone();
    let snapshot = owner.snapshot().unwrap();
    let revision = snapshot.revision().clone();
    if with_owner_effect {
        assert_eq!(commit.owner_effects().len(), 1);
        let rejected = lease
            .accept_cssom_publication(&create, commit, snapshot, Some(construction.ticket()))
            .unwrap_err();
        assert_eq!(rejected.error(), &DomError::StyleProtocol);
        let (_, commit, snapshot) = rejected.into_parts();
        assert_eq!(commit.owner_effects().len(), 1);
        assert_eq!(
            commit.owner_effects()[0].status(),
            surgeist_cssom::CssomOwnerEffectStatus::Pending
        );
        assert_eq!(snapshot.revision(), &revision);
        let DomStyleProgress::Request(retry) = lease.advance(&cancel).unwrap() else {
            panic!("pending owner output must not acknowledge creation")
        };
        assert_eq!(retry.association(), create.association());
        assert_eq!(retry.cssom_revision(), create.cssom_revision());
        drop(lease);
        assert!(dom.pending_style_edit().unwrap().has_published());
        assert!(dom.has_style_cssom_roots());
        return;
    }
    if let Some(wrong) = wrong.filter(|w| !matches!(w, WrongCreationInput::OwnerMedia)) {
        let rejected = lease
            .accept_cssom_publication(&create, commit, snapshot, Some(construction.ticket()))
            .expect_err(&format!(
                "wrong {wrong:?} must preserve and reject the actual publication"
            ));
        assert_eq!(rejected.error(), &DomError::StyleReceiptMismatch);
        let (_, returned_commit, returned_snapshot) = rejected.into_parts();
        assert_eq!(
            returned_commit.sheet(construction.ticket()).unwrap(),
            &sheet
        );
        assert_eq!(returned_snapshot.revision(), &revision);
        assert!(returned_snapshot.sheet(&sheet).is_ok());
        let DomStyleProgress::Request(retry) = lease.advance(&cancel).unwrap() else {
            panic!("rejected publication must leave the creation request outstanding")
        };
        assert_eq!(retry.association(), create.association());
        assert_eq!(retry.cssom_revision(), create.cssom_revision());
        return;
    }
    let rejected = lease
        .accept_cssom_publication(&policy, commit, snapshot, Some(construction.ticket()))
        .unwrap_err();
    assert_eq!(rejected.error(), &DomError::StyleReceiptMismatch);
    let (_, commit, snapshot) = rejected.into_parts();
    let mut token = lease
        .accept_cssom_publication(&create, commit, snapshot, Some(construction.ticket()))
        .unwrap()
        .unwrap();
    // The canonical owner, rather than this execution capability, now holds
    // the actual mutable store and acknowledged publication through recovery.
    drop(owner);
    cancel.store(true, std::sync::atomic::Ordering::Relaxed);
    assert_eq!(lease.advance(&cancel).unwrap_err(), DomError::Cancelled);
    drop(lease);
    fn require_send<T: Send>() {}
    require_send::<Dom>();
    require_send::<surgeist_dom::DomStyleCssomRoot>();
    let (moved, owner, id) = std::thread::spawn(move || {
        // The whole published owner and exact acknowledged receipt move, not a
        // snapshot or a reconstructed tree. Refusals on the destination leave
        // its original identity, work high-water and mutable CSSOM custody live.
        let destination_cancel = AtomicBool::new(true);
        let before = dom.snapshot().stamp();
        assert!(matches!(
            dom.resume_style_edit(operation, source_limits(), &destination_cancel),
            Err(DomError::Cancelled)
        ));
        assert_eq!(dom.snapshot().stamp(), before);
        destination_cancel.store(false, std::sync::atomic::Ordering::Relaxed);
        let insufficient =
            surgeist_dom::DomStyleEditLimits::new(16_000_000, 100_000, 0, 64, 4096, 4096);
        assert!(matches!(
            dom.resume_style_edit(operation, insufficient, &destination_cancel),
            Err(DomError::InvalidLimit(_))
        ));
        assert_eq!(dom.snapshot().stamp(), before);
        let mut lease = dom
            .resume_style_edit(operation, source_limits(), &destination_cancel)
            .unwrap();
        let owner = lease.cssom_root(&create).unwrap();
        let DomStyleProgress::Complete(done) = lease.advance(&destination_cancel).unwrap() else {
            panic!("external creation must not replay")
        };
        let id = done.published_creation(&style).unwrap().unwrap();
        drop(lease);
        (dom, owner, id)
    })
    .join()
    .unwrap();
    dom = moved;
    cancel.store(false, std::sync::atomic::Ordering::Relaxed);
    assert!(dom.has_style_cssom_roots());
    let snapshot = dom.snapshot();
    let style = snapshot.node(id).unwrap().as_style().unwrap();
    assert_eq!(style.sheet().unwrap().unwrap().id(), &sheet);
    assert_eq!(owner.snapshot().unwrap().revision(), &revision);
    assert_eq!(owner.snapshot().unwrap().sheets().len(), 1);
    assert!(matches!(
        dom.finish_style_notification(&mut token, &cancel).unwrap(),
        surgeist_dom::DomStyleNotificationCompletion::Complete(_)
    ));
    assert_eq!(
        dom.finish_style_notification(&mut token, &cancel)
            .unwrap_err(),
        DomError::StyleProtocol
    );
    if exercise_disabled {
        exercise_disabled_sheet(&mut dom, id, &owner, &sheet);
    }
    if matches!(wrong, Some(WrongCreationInput::OwnerMedia)) {
        exercise_wrong_owner_media(&mut dom, id, &owner, &sheet);
    }
}
fn exercise_wrong_owner_media(
    dom: &mut Dom,
    element: surgeist_dom::NodeId,
    owner: &surgeist_dom::DomStyleCssomRoot,
    sheet: &surgeist_cssom::CssomSheetId,
) {
    use surgeist_css::{CssParserContext, CssUtf16InputLimits};
    use surgeist_cssom::{
        CssomEditGuard, CssomInputRole, CssomInputVersion, CssomOwnerMediaInput,
        CssomSheetOwnerAttributes,
    };
    use surgeist_dom::{DomStyleCssomOwnerBinding, DomStyleProgress};
    let cancel = AtomicBool::new(false);
    let mut batch = DomBatch::new();
    batch.set_attribute(element, "media", "print");
    let mut lease = dom
        .begin_style_batch(batch, source_limits(), &cancel)
        .unwrap();
    let DomStyleProgress::Request(request) = lease.advance(&cancel).unwrap() else {
        panic!("actual owner attribute request")
    };
    assert_eq!(
        request.action(),
        surgeist_dom::DomStyleAction::OwnerAttributes
    );
    assert_eq!(request.source().media(), Some(&"print".into()));
    let request = lease
        .qualify_cssom(&request, owner.root(), &owner.snapshot().unwrap())
        .unwrap();
    let version = CssomInputVersion {
        role: CssomInputRole::Owner,
        identity: "actual-style-owner".into(),
        revision: 2,
    };
    lease
        .bind_cssom_owner(
            &request,
            DomStyleCssomOwnerBinding::new(&request, version.clone(), "inline-owner".into())
                .unwrap(),
            &cancel,
        )
        .unwrap();
    let wrong_media = CssParserContext::default()
        .parse_media_query_list_utf16_with_limits(
            std::sync::Arc::new("screen".encode_utf16().collect()),
            CssUtf16InputLimits::default(),
            || Ok::<_, ()>(()),
        )
        .unwrap();
    let mut batch = owner
        .batch(CssomEditGuard::from_snapshot(&owner.snapshot().unwrap()))
        .unwrap();
    batch
        .update_sheet_owner_attributes(
            sheet,
            CssomSheetOwnerAttributes {
                owner: "inline-owner",
                version,
                title: None,
                media: CssomOwnerMediaInput::Parsed(wrong_media),
            },
        )
        .unwrap();
    let commit = owner.commit(&mut Some(batch)).unwrap();
    let snapshot = owner.snapshot().unwrap();
    let revision = snapshot.revision().clone();
    let rejected = lease
        .accept_cssom_publication(&request, commit, snapshot, None)
        .expect_err("wrong owner media input must be rejected");
    assert_eq!(rejected.error(), &DomError::StyleReceiptMismatch);
    let (_, commit, snapshot) = rejected.into_parts();
    assert_eq!(snapshot.revision(), &revision);
    assert_eq!(
        match commit.publication() {
            surgeist_cssom::CssomPublication::Changed(p) => p.to(),
            surgeist_cssom::CssomPublication::Unchanged { revision } => revision,
        },
        &revision
    );
    let DomStyleProgress::Request(retry) = lease.advance(&cancel).unwrap() else {
        panic!("owner update still outstanding")
    };
    assert_eq!(retry.association(), request.association());
    assert_eq!(retry.cssom_revision(), request.cssom_revision());
}

fn exercise_disabled_sheet(
    dom: &mut Dom,
    element: surgeist_dom::NodeId,
    owner: &surgeist_dom::DomStyleCssomRoot,
    sheet: &surgeist_cssom::CssomSheetId,
) {
    use surgeist_cssom::CssomEditGuard;
    use surgeist_dom::{DomStyleAction, DomStyleProgress};
    let cancel = AtomicBool::new(false);
    let historical_css = owner.snapshot().unwrap();
    assert!(!historical_css.sheet(sheet).unwrap().inputs().disabled);
    let original = dom.snapshot();
    let original_association = original
        .node(element)
        .unwrap()
        .as_style()
        .unwrap()
        .sheet()
        .unwrap()
        .unwrap()
        .clone();
    let ordinary = dom.style_cssom_root(owner.root()).unwrap().unwrap();
    assert_eq!(
        ordinary.snapshot().unwrap().revision(),
        historical_css.revision()
    );
    for (index, disabled) in [true, true, false].into_iter().enumerate() {
        let before = dom.snapshot();
        let mut batch = DomBatch::new();
        batch.set_style_disabled(element, disabled);
        assert_eq!(
            dom.apply(batch, &cancel).unwrap_err(),
            DomError::StyleDisabledCompositionRequired
        );
        assert_eq!(dom.snapshot().stamp(), before.stamp());
        let mut batch = DomBatch::new();
        batch.set_style_disabled(element, disabled);
        let mut lease = dom
            .begin_style_batch(batch, source_limits(), &cancel)
            .unwrap();
        let operation = lease.operation();
        let DomStyleProgress::Request(request) = lease.advance(&cancel).unwrap() else {
            panic!("associated sheet requires its real CSSOM owner")
        };
        assert_eq!(request.action(), DomStyleAction::SetDisabled(disabled));
        assert_eq!(request.sheet().unwrap().id(), sheet);
        assert_eq!(
            request.association(),
            Some(original_association.association())
        );
        let request = lease
            .qualify_cssom(&request, owner.root(), &owner.snapshot().unwrap())
            .unwrap();
        if index == 0 {
            let mut wrong = owner
                .batch(CssomEditGuard::from_snapshot(&owner.snapshot().unwrap()))
                .unwrap();
            assert!(!wrong.set_disabled(sheet, false).unwrap());
            let commit = owner.commit(&mut Some(wrong)).unwrap();
            let rejected = lease
                .accept_cssom_publication(&request, commit, owner.snapshot().unwrap(), None)
                .unwrap_err();
            assert_eq!(rejected.error(), &DomError::StyleReceiptMismatch);
            let (_, commit, snapshot) = rejected.into_parts();
            assert!(commit.owner_effects().is_empty());
            assert!(!snapshot.sheet(sheet).unwrap().inputs().disabled);
            let DomStyleProgress::Request(retry) = lease.advance(&cancel).unwrap() else {
                panic!("invalid reply retains actual request")
            };
            assert_eq!(retry.association(), request.association());
        }
        let mut edit = ordinary
            .batch(CssomEditGuard::from_snapshot(&ordinary.snapshot().unwrap()))
            .unwrap();
        assert_eq!(edit.set_disabled(sheet, disabled).unwrap(), index != 1);
        let commit = ordinary.commit(&mut Some(edit)).unwrap();
        assert!(
            lease
                .accept_cssom_publication(&request, commit, ordinary.snapshot().unwrap(), None)
                .unwrap()
                .is_none()
        );
        // The acknowledged flag mutation is not replayed after failed closing
        // admission, permission drop or reacquisition.
        cancel.store(true, std::sync::atomic::Ordering::Relaxed);
        assert_eq!(lease.advance(&cancel).unwrap_err(), DomError::Cancelled);
        drop(lease);
        assert!(matches!(
            dom.style_cssom_root(owner.root()),
            Err(DomError::IncompleteSourceOperation)
        ));
        cancel.store(false, std::sync::atomic::Ordering::Relaxed);
        let mut lease = dom
            .resume_style_edit(operation, source_limits(), &cancel)
            .unwrap();
        assert!(matches!(
            lease.advance(&cancel).unwrap(),
            DomStyleProgress::Complete(_)
        ));
        drop(lease);
        let current = dom.snapshot();
        let associated = current
            .node(element)
            .unwrap()
            .as_style()
            .unwrap()
            .sheet()
            .unwrap()
            .unwrap();
        assert_eq!(associated.id(), sheet);
        assert_eq!(associated.association(), original_association.association());
        assert_eq!(associated.root(), original_association.root());
        assert_eq!(
            associated.revision(),
            ordinary.snapshot().unwrap().revision()
        );
        assert_eq!(
            ordinary
                .snapshot()
                .unwrap()
                .sheet(sheet)
                .unwrap()
                .inputs()
                .disabled,
            disabled
        );
        assert_eq!(ordinary.snapshot().unwrap().sheets().len(), 1);
    }
    assert!(!historical_css.sheet(sheet).unwrap().inputs().disabled);
    assert_eq!(
        original
            .node(element)
            .unwrap()
            .as_style()
            .unwrap()
            .sheet()
            .unwrap()
            .unwrap(),
        &original_association
    );
}

#[test]
fn disabled_without_an_association_is_a_no_op_and_does_not_accept_other_families() {
    let cancel = AtomicBool::new(false);
    let mut dom = dom();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let style = batch.create_element_ns(&document, Namespace::html(), "style");
    let div = batch.create_element_ns(&document, Namespace::html(), "div");
    let commit = dom.apply(batch, &cancel).unwrap();
    let document = commit.document(&document).unwrap();
    let style = commit.created(&style).unwrap().unwrap();
    let div = commit.created(&div).unwrap().unwrap();
    let before = dom.snapshot();
    assert!(dom.style_cssom_root(document.node()).unwrap().is_none());
    assert!(matches!(
        dom.style_cssom_root(style),
        Err(DomError::WrongNodeKind)
    ));
    let mut batch = DomBatch::new();
    batch.set_style_disabled(style, true);
    dom.apply(batch, &cancel).unwrap();
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    assert!(
        dom.snapshot()
            .node(style)
            .unwrap()
            .as_style()
            .unwrap()
            .sheet()
            .unwrap()
            .is_none()
    );
    let mut batch = DomBatch::new();
    batch.set_style_disabled(div, true);
    assert_eq!(
        dom.apply(batch, &cancel).unwrap_err(),
        DomError::WrongNodeKind
    );
    assert_eq!(dom.snapshot().stamp(), before.stamp());
}

#[test]
fn source_input_one_short_work_or_storage_refuses_before_the_first_phase() {
    use surgeist_dom::{DomStyleEditLimits, DomStyleProgress};
    let cancel = AtomicBool::new(false);
    for work_bound in [false, true] {
        let attempt = |bound: usize| {
            let mut dom = dom();
            let mut batch = DomBatch::new();
            let document = batch.create_document();
            let style = batch.create_element_ns(&document, Namespace::html(), "style");
            let text = batch.create_text(&document, "a".repeat(2048));
            batch.insert_before(&text, &style, None);
            let commit = dom.apply(batch, &cancel).unwrap();
            let style = commit.created(&style).unwrap().unwrap();
            let document = commit.document(&document).unwrap();
            let before = dom.snapshot();
            let usage = dom.usage();
            let limits = DomStyleEditLimits::new(
                if work_bound { 16_000_000 } else { bound },
                100_000,
                if work_bound { bound as u64 } else { 10_000_000 },
                64,
                4096,
                4096,
            );
            let mut batch = DomBatch::new();
            batch.insert_before(style, document, None);
            let mut reached = false;
            if let Ok(mut lease) = dom.begin_style_batch(batch, limits, &cancel) {
                match lease.advance(&cancel) {
                    Ok(DomStyleProgress::Request(request)) => {
                        assert_eq!(
                            request.action(),
                            surgeist_dom::DomStyleAction::Update(
                                HtmlStyleUpdateAction::CheckInlineCsp
                            )
                        );
                        assert_eq!(request.source().text().len(), 2048);
                        assert_eq!(
                            request.source().document_metadata().url(),
                            before.document(document).unwrap().metadata().url()
                        );
                        reached = true;
                    }
                    Err(DomError::StyleWorkLimit { .. }) if work_bound => {}
                    Err(DomError::ResourceLimit {
                        resource: surgeist_dom::DomResource::StyleRetainedBytes,
                        ..
                    }) if !work_bound => {}
                    other => panic!("unexpected source admission outcome: {other:?}"),
                }
            }
            if !reached {
                assert_eq!(dom.snapshot().stamp(), before.stamp());
                assert_eq!(dom.usage(), usage);
                assert_eq!(dom.snapshot().node(style).unwrap().parent(), None);
                assert!(dom.pending_style_edit().is_none());
            }
            reached
        };
        // Find the actual required allowance, rather than fixing a layout-size
        // constant. EVERY rejected first invocation must preserve the old owner.
        let mut low = 0;
        let mut high = if work_bound { 10_000_000 } else { 16_000_000 };
        assert!(attempt(high));
        while high - low > 1 {
            let middle = low + (high - low) / 2;
            if attempt(middle) {
                high = middle;
            } else {
                low = middle;
            }
        }
        assert!(!attempt(high - 1));
        assert!(attempt(high));
    }
}

#[test]
fn transition_generation_and_original_tickets_survive_intermediate_style_phases() {
    use surgeist_cssom::{CssomEditGuard, CssomInputRole, CssomInputVersion, CssomSheetInputs};
    use surgeist_dom::{DomStyleAction, DomStyleProgress};
    let cancel = AtomicBool::new(false);
    let mut dom = Dom::new(
        DomLimits::new(64, 10000, 256, 0, 0, 16, 100000)
            .unwrap()
            .with_view_transitions(8, 16, 80)
            .unwrap(),
    )
    .unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let parent = batch.create_element_ns(&document, Namespace::html(), "div");
    let first_style = batch.create_element_ns(&document, Namespace::html(), "style");
    let second_style = batch.create_element_ns(&document, Namespace::html(), "style");
    batch.insert_before(&parent, &document, None);
    let commit = dom.apply(batch, &cancel).unwrap();
    let document = commit.document(&document).unwrap();
    let parent = commit.created(&parent).unwrap().unwrap();
    let first_style = commit.created(&first_style).unwrap().unwrap();
    let second_style = commit.created(&second_style).unwrap().unwrap();
    let mut css = cssom();
    let mut edit = css
        .batch(CssomEditGuard::from_snapshot(&css.snapshot()))
        .unwrap();
    let sheet = edit
        .create_parsed_sheet(
            "",
            CssomSheetInputs::constructed(
                CssomInputVersion {
                    role: CssomInputRole::Document,
                    identity: "actual VT stylesheet".into(),
                    revision: 1,
                },
                "actual VT stylesheet".into(),
                None,
            ),
        )
        .unwrap();
    let sheet = css.commit(edit).unwrap().sheet(&sheet).unwrap().clone();
    let mut batch = DomBatch::new();
    batch.initialize_view_transitions(
        dom.snapshot().document(document).unwrap().stamp(),
        surgeist_dom::DomPreparedViewTransitionSheet::capture(
            &css.snapshot(),
            &sheet,
            surgeist_dom::DomViewTransitionCaptureLimits::new(16_384, 1_000_000),
            &cancel,
        )
        .unwrap(),
    );
    dom.apply(batch, &cancel).unwrap();
    let basis = dom.snapshot().document(document).unwrap().stamp();
    let mut batch = DomBatch::new();
    let first = batch.begin_view_transition(basis);
    batch.insert_before(first_style, parent, None);
    batch.clear_view_transition_state(basis, &first);
    batch.release_view_transition(basis, &first);
    let second = batch.begin_view_transition(basis);
    batch.insert_before(second_style, parent, None);
    let mut lease = dom
        .begin_style_batch(batch, source_limits(), &cancel)
        .unwrap();
    let mut observed = Vec::new();
    let completion = loop {
        match lease.advance(&cancel).unwrap() {
            DomStyleProgress::Request(request) => {
                assert_eq!(
                    request.action(),
                    DomStyleAction::Update(HtmlStyleUpdateAction::CheckInlineCsp)
                );
                observed.push(
                    lease
                        .snapshot()
                        .document(document)
                        .unwrap()
                        .view_transitions()
                        .unwrap()
                        .active()
                        .unwrap(),
                );
                lease
                    .accept_inline_csp(&request, &block(&request), &cancel)
                    .unwrap();
            }
            DomStyleProgress::Complete(done) => break done,
        }
    };
    assert_eq!(observed.len(), 2);
    assert_ne!(
        observed[0], observed[1],
        "a publicly retired transition identity must never revive"
    );
    assert_eq!(
        completion.commit().view_transition(&first).unwrap(),
        Some(observed[0])
    );
    assert_eq!(
        completion.commit().view_transition(&second).unwrap(),
        Some(observed[1])
    );
}

#[test]
fn pending_source_refuses_dependent_base_and_image_reads_but_old_snapshots_stay_complete() {
    use surgeist_dom::DomStyleProgress;
    let cancel = AtomicBool::new(false);
    let mut dom = dom();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let image = batch.create_element_ns(&document, Namespace::html(), "img");
    let style = batch.create_element_ns(&document, Namespace::html(), "style");
    let commit = dom.apply(batch, &cancel).unwrap();
    let document = commit.document(&document).unwrap();
    let image = commit.created(&image).unwrap().unwrap();
    let style = commit.created(&style).unwrap().unwrap();
    let before = dom.snapshot();
    let old_wait = before
        .node(image)
        .unwrap()
        .as_img()
        .unwrap()
        .suspended_update()
        .unwrap()
        .unwrap()
        .id();
    let mut batch = DomBatch::new();
    batch.insert_before(style, document, None);
    let mut lease = dom
        .begin_style_batch(batch, source_limits(), &cancel)
        .unwrap();
    let DomStyleProgress::Request(request) = lease.advance(&cancel).unwrap() else {
        panic!("actual source boundary");
    };
    let pending = lease.snapshot();
    assert_eq!(
        pending.document(document).unwrap().first_base(),
        Err(DomError::IncompleteSourceOperation)
    );
    assert!(matches!(
        pending
            .node(image)
            .unwrap()
            .as_img()
            .unwrap()
            .suspended_update(),
        Err(DomError::IncompleteSourceOperation)
    ));
    assert_eq!(
        before.document(document).unwrap().first_base().unwrap(),
        None
    );
    assert_eq!(
        before
            .node(image)
            .unwrap()
            .as_img()
            .unwrap()
            .suspended_update()
            .unwrap()
            .unwrap()
            .id(),
        old_wait
    );
    lease
        .accept_inline_csp(&request, &block(&request), &cancel)
        .unwrap();
    assert!(matches!(
        lease.advance(&cancel).unwrap(),
        DomStyleProgress::Complete(_)
    ));
    drop(lease);
    assert_eq!(
        dom.snapshot()
            .document(document)
            .unwrap()
            .first_base()
            .unwrap(),
        None
    );
    assert_eq!(
        dom.snapshot()
            .node(image)
            .unwrap()
            .as_img()
            .unwrap()
            .suspended_update()
            .unwrap()
            .unwrap()
            .id(),
        old_wait
    );
    assert_eq!(
        pending.document(document).unwrap().first_base(),
        Err(DomError::IncompleteSourceOperation),
        "the old phase snapshot remains explicitly incomplete"
    );
}

#[test]
fn csp_fact_admission_counts_actual_policy_backing_and_failed_reply_keeps_request_retryable() {
    use surgeist_dom::{
        DocumentString, DomResource, DomStyleInlineCspObservation, DomStyleProgress,
    };
    let cancel = AtomicBool::new(false);
    let mut dom = dom();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let style = batch.create_element_ns(&document, Namespace::html(), "style");
    batch.insert_before(&style, &document, None);
    let mut lease = dom
        .begin_style_batch(batch, source_limits(), &cancel)
        .unwrap();
    let DomStyleProgress::Request(request) = lease.advance(&cancel).unwrap() else {
        panic!("policy boundary");
    };
    let phase = lease.snapshot().stamp();
    let mut raw = Vec::with_capacity(200_000);
    raw.push(b'x' as u16);
    let oversized = DomStyleInlineCspObservation::evaluated(
        request.source(),
        std::sync::Arc::new(vec![DocumentString::from_utf16(raw)]),
        3,
        false,
    );
    assert!(matches!(
        lease.accept_inline_csp(&request, &oversized, &cancel),
        Err(DomError::ResourceLimit {
            resource: DomResource::StyleRetainedUnits,
            ..
        })
    ));
    assert_eq!(lease.snapshot().stamp(), phase);
    cancel.store(true, std::sync::atomic::Ordering::Relaxed);
    assert_eq!(
        lease.accept_inline_csp(&request, &block(&request), &cancel),
        Err(DomError::Cancelled)
    );
    cancel.store(false, std::sync::atomic::Ordering::Relaxed);
    let DomStyleProgress::Request(retry) = lease.advance(&cancel).unwrap() else {
        panic!("same unacknowledged request");
    };
    assert_eq!(retry.source().stamp(), request.source().stamp());
    assert_eq!(retry.source().ordinal(), request.source().ordinal());
    lease
        .accept_inline_csp(&retry, &block(&retry), &cancel)
        .unwrap();
    assert!(matches!(
        lease.advance(&cancel).unwrap(),
        DomStyleProgress::Complete(_)
    ));
}

#[test]
fn issued_select_completion_parks_private_copy_and_reacquires_exact_fifo_head() {
    use surgeist_dom::{
        DomNativeCheckpoint, DomNativeCheckpointLimits, DomSlotQueryLimits, DomStyleProgress,
    };
    let cancel = AtomicBool::new(false);
    let checkpoint = DomNativeCheckpointLimits::new(
        4_000_000,
        10_000_000,
        DomSlotQueryLimits::new(256, 1024).unwrap(),
    )
    .unwrap();
    let mut dom = Dom::new(
        DomLimits::new(256, 100_000, 1024, 0, 0, 16, 1_000_000)
            .unwrap()
            .with_select_processing(32, 4_000_000, 10_000_000)
            .unwrap(),
    )
    .unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let select = batch.create_element_ns(&document, Namespace::html(), "select");
    let option = batch.create_element_ns(&document, Namespace::html(), "option");
    batch.set_attribute(&option, "selected", "");
    let original_style = batch.create_element_ns(&document, Namespace::html(), "style");
    batch.set_attribute(&original_style, "type", "text/plain");
    let text = batch.create_text(&document, "a { color: red }");
    batch.insert_before(&text, &original_style, None);
    let other = batch.create_element_ns(&document, Namespace::html(), "option");
    batch.insert_before(&original_style, &other, None);
    let content = batch.create_element_ns(&document, Namespace::html(), "selectedcontent");
    batch.insert_before(&option, &select, None);
    batch.insert_before(&other, &select, None);
    batch.insert_before(&content, &select, None);
    batch.insert_before(&select, &document, None);
    let commit = dom.apply(batch, &cancel).unwrap();
    let select = commit.created(&select).unwrap().unwrap();
    let option = commit.created(&option).unwrap().unwrap();
    let original_style = commit.created(&original_style).unwrap().unwrap();
    let content = commit.created(&content).unwrap().unwrap();
    let mut batch = DomBatch::new();
    // Removing the selected Option (not an unselected sibling) reaches the
    // source condition that actually queues the Select update intention.
    batch.detach(option);
    batch.set_attribute(original_style, "type", "text/css");
    dom.apply(batch, &cancel).unwrap();
    let DomNativeCheckpoint::Select(task) =
        dom.native_checkpoint_step(checkpoint, &cancel).unwrap()
    else {
        panic!("actual queued Select update");
    };
    let operation = task.operation();
    assert!(matches!(
        dom.complete_select_task(&task, checkpoint, &cancel),
        Err(DomError::StyleCompositionRequired(
            HtmlStyleUpdateAction::CheckInlineCsp
        ))
    ));
    assert_eq!(dom.pending_native_tasks(), 1);
    let mut lease = dom
        .begin_style_select_completion(&task, checkpoint, source_limits(), &cancel)
        .unwrap();
    let source_operation = lease.operation();
    let DomStyleProgress::Request(request) = lease.advance(&cancel).unwrap() else {
        panic!("copied connected Style boundary");
    };
    assert_ne!(request.source().element(), original_style);
    let partial = lease.snapshot();
    assert_eq!(
        partial.node(request.source().element()).unwrap().parent(),
        Some(content)
    );
    assert!(
        !partial.nodes().any(|id| matches!(
            partial.node(id).unwrap().kind(),
            surgeist_dom::DomNodeKind::DocumentFragment
        )),
        "private clone wrapper must never get a public identity"
    );
    drop(lease);
    drop(task);
    assert_eq!(dom.pending_native_tasks(), 1);
    let DomNativeCheckpoint::Select(reacquired) =
        dom.native_checkpoint_step(checkpoint, &cancel).unwrap()
    else {
        panic!("same canonical head remains owned");
    };
    assert_eq!(reacquired.operation(), operation);
    assert_eq!(reacquired.select(), select);
    let mut lease = dom
        .resume_style_select_completion(source_operation, &reacquired, source_limits(), &cancel)
        .unwrap();
    let DomStyleProgress::Request(request) = lease.advance(&cancel).unwrap() else {
        panic!("same exact unfinished source request");
    };
    lease
        .accept_inline_csp(&request, &block(&request), &cancel)
        .unwrap();
    assert!(matches!(
        lease.advance(&cancel).unwrap(),
        DomStyleProgress::Complete(_)
    ));
    drop(lease);
    assert_eq!(dom.pending_native_tasks(), 0);
    assert!(matches!(
        dom.complete_select_task(&reacquired, checkpoint, &cancel),
        Err(DomError::InvalidNativeTask)
    ));
    assert!(!dom.snapshot().nodes().any(|id| matches!(
        dom.snapshot().node(id).unwrap().kind(),
        surgeist_dom::DomNodeKind::DocumentFragment
    )));
}

#[test]
fn actual_html_body_gate_and_blocking_attribute_cleanup_use_the_canonical_set() {
    use std::sync::Arc;
    use surgeist_css::{CssParserContext, CssUtf16InputLimits};
    use surgeist_cssom::{CssomEditGuard, CssomInputRole, CssomInputVersion, CssomSheetInputs};
    use surgeist_dom::{
        DomChange, DomStyleCssomOwnerBinding, DomStyleCssomRoot, DomStyleInlineCspObservation,
        DomStyleProgress,
    };
    let cancel = AtomicBool::new(false);
    for body_present in [true, false] {
        let mut dom = dom();
        let mut batch = DomBatch::new();
        let source = batch.create_document();
        let document = batch.create_html_document(&source, None);
        let commit = dom.apply(batch, &cancel).unwrap();
        let document = commit.document(&document).unwrap();
        let before = dom.snapshot();
        let html = before.node(document.node()).unwrap().children()[1];
        let children = before.node(html).unwrap().children();
        let (head, body) = (children[0], children[1]);
        let mut batch = DomBatch::new();
        if !body_present {
            batch.detach(body);
        }
        let style = batch.create_element_ns(document, Namespace::html(), "style");
        batch.set_attribute(&style, "blocking", "other\trender\nrender");
        let text = batch.create_text(document, "a {}");
        batch.insert_before(&text, &style, None);
        batch.insert_before(&style, head, None);
        let mut lease = dom
            .begin_style_batch(batch, source_limits(), &cancel)
            .unwrap();
        let DomStyleProgress::Request(policy) = lease.advance(&cancel).unwrap() else {
            panic!("actual CSP request")
        };
        lease
            .accept_inline_csp(
                &policy,
                &DomStyleInlineCspObservation::complete_empty(policy.source(), 1),
                &cancel,
            )
            .unwrap();
        let DomStyleProgress::Request(create) = lease.advance(&cancel).unwrap() else {
            panic!("actual creation")
        };
        let root = DomStyleCssomRoot::new(document.node(), cssom());
        lease.bind_cssom_root(&create, &root, &cancel).unwrap();
        let create = lease
            .qualify_cssom(&create, document.node(), &root.snapshot().unwrap())
            .unwrap();
        let version = CssomInputVersion {
            role: CssomInputRole::Owner,
            identity: "blocking-style".into(),
            revision: 1,
        };
        lease
            .bind_cssom_owner(
                &create,
                DomStyleCssomOwnerBinding::new(&create, version.clone(), "inline".into()).unwrap(),
                &cancel,
            )
            .unwrap();
        let context = CssParserContext::default();
        let report = context
            .parse_sheet_utf16_with_limits(
                create.source().text().clone(),
                CssUtf16InputLimits::default(),
                || Ok::<_, ()>(()),
            )
            .unwrap();
        let media = context
            .parse_media_query_list_utf16_with_limits(
                Arc::new(Vec::new()),
                CssUtf16InputLimits::default(),
                || Ok::<_, ()>(()),
            )
            .unwrap();
        let mut inputs = CssomSheetInputs::external(version, true);
        inputs.owner = Some("inline".into());
        let mut edit = root
            .batch(CssomEditGuard::from_snapshot(&root.snapshot().unwrap()))
            .unwrap();
        let construction = edit
            .create_parsed_sheet_with_media_reports(report, media, inputs)
            .unwrap();
        let commit = root.commit(&mut Some(edit)).unwrap();
        let _notification = lease
            .accept_cssom_publication(
                &create,
                commit,
                root.snapshot().unwrap(),
                Some(construction.ticket()),
            )
            .unwrap();
        let DomStyleProgress::Complete(done) = lease.advance(&cancel).unwrap() else {
            panic!("completed synchronous source")
        };
        let style = done.published_creation(&style).unwrap().unwrap();
        drop(lease);
        let completed = dom.snapshot();
        assert_eq!(
            completed
                .node(style)
                .unwrap()
                .as_style()
                .unwrap()
                .is_render_blocking()
                .unwrap(),
            !body_present
        );
        assert!(
            completed
                .node(style)
                .unwrap()
                .as_style()
                .unwrap()
                .sheet()
                .unwrap()
                .is_some()
        );
        let mut batch = DomBatch::new();
        batch.set_attribute(style, "blocking", "other");
        let commit = dom.apply(batch, &cancel).unwrap();
        assert!(
            !dom.snapshot()
                .node(style)
                .unwrap()
                .as_style()
                .unwrap()
                .is_render_blocking()
                .unwrap()
        );
        assert_eq!(
            commit.changes().changes().iter().any(
                |change| matches!(change, DomChange::DocumentRenderBlocking(id) if *id == document)
            ),
            !body_present
        );
        assert_eq!(
            completed
                .node(style)
                .unwrap()
                .as_style()
                .unwrap()
                .is_render_blocking()
                .unwrap(),
            !body_present
        );
    }
}

#[test]
fn cancelled_unpublished_operation_id_cannot_resume_a_later_published_tail() {
    stale_operation_id_case(0);
}
#[test]
fn no_op_operation_id_cannot_resume_a_later_published_tail() {
    stale_operation_id_case(1);
}
#[test]
fn dropped_unpublished_permission_id_cannot_resume_a_later_published_tail() {
    stale_operation_id_case(2);
}
fn stale_operation_id_case(end: u8) {
    use surgeist_dom::DomStyleProgress;
    let cancel = AtomicBool::new(false);
    let mut dom = dom();
    let mut setup = DomBatch::new();
    let document = setup.create_document();
    let style = setup.create_element_ns(&document, Namespace::html(), "style");
    let commit = dom.apply(setup, &cancel).unwrap();
    let document = commit.document(&document).unwrap();
    let style = commit.created(&style).unwrap().unwrap();
    let mut first = dom
        .begin_style_batch(DomBatch::new(), source_limits(), &cancel)
        .unwrap();
    let old = first.operation();
    if end != 2 {
        if end == 0 {
            first.request_cancel();
        }
        assert!(matches!(
            first.advance(&cancel).unwrap(),
            DomStyleProgress::Complete(_)
        ));
    }
    drop(first);
    let mut batch = DomBatch::new();
    batch.insert_before(style, document, None);
    let mut current = dom
        .begin_style_batch(batch, source_limits(), &cancel)
        .unwrap();
    let id = current.operation();
    let DomStyleProgress::Request(request) = current.advance(&cancel).unwrap() else {
        panic!("actual published policy tail")
    };
    drop(current);
    assert!(
        matches!(
            dom.resume_style_edit(old, source_limits(), &cancel),
            Err(DomError::InvalidSourceOperation)
        ),
        "an old issued ID must not grant a different operation"
    );
    assert_ne!(old, id);
    let mut current = dom.resume_style_edit(id, source_limits(), &cancel).unwrap();
    current
        .accept_inline_csp(&request, &block(&request), &cancel)
        .unwrap();
    assert!(matches!(
        current.advance(&cancel).unwrap(),
        DomStyleProgress::Complete(_)
    ));
}

fn manual_slot_input_fixture(inputs: usize) -> (Dom, surgeist_dom::NodeId, surgeist_dom::NodeId) {
    use surgeist_dom::{HtmlShadowRootMode, HtmlShadowRootProperties, HtmlSlotAssignment};
    let mut dom = Dom::new(DomLimits::new(32, 4096, 64, 32, inputs, 8, 65536).unwrap()).unwrap();
    let mut setup = DomBatch::new();
    let document = setup.create_document();
    let host = setup.create_element_ns(&document, Namespace::html(), "div");
    setup.insert_before(&host, &document, None);
    let root = setup.attach_shadow(
        &host,
        HtmlShadowRootProperties::new(
            HtmlShadowRootMode::Open,
            HtmlSlotAssignment::Manual,
            false,
            true,
            false,
        ),
    );
    let slot = setup.create_element_ns(&document, Namespace::html(), "slot");
    let text = setup.create_text(&document, "actual slottable");
    setup.insert_before(&slot, &root, None);
    setup.insert_before(&text, &host, None);
    let commit = dom.apply(setup, &AtomicBool::new(false)).unwrap();
    (
        dom,
        commit.created(&slot).unwrap().unwrap(),
        commit.created(&text).unwrap().unwrap(),
    )
}

fn execute_source_batch(
    dom: &mut Dom,
    batch: DomBatch,
    cancel: &AtomicBool,
) -> Result<(), DomError> {
    match dom.begin_style_batch(batch, source_limits(), cancel) {
        Err(error) => Err(error),
        Ok(mut lease) => match lease.advance(cancel)? {
            surgeist_dom::DomStyleProgress::Complete(_) => Ok(()),
            surgeist_dom::DomStyleProgress::Request(request) => {
                panic!("manual Slot assignment has no external Style phase: {request:?}")
            }
        },
    }
}

#[test]
fn source_slot_inputs_preserve_the_canonical_zero_allowance_before_publication() {
    use surgeist_dom::DomResource;
    let cancel = AtomicBool::new(false);
    let (mut dom, slot, text) = manual_slot_input_fixture(0);
    let commands = || {
        let mut batch = DomBatch::new();
        batch.assign_slot(slot, vec![text.into()]);
        batch
    };
    let before = dom.snapshot();
    let usage = dom.usage();
    let expected = DomError::ResourceLimit {
        resource: DomResource::SlotAssignmentInputs,
        limit: 0,
    };
    assert_eq!(dom.apply(commands(), &cancel).unwrap_err(), expected);
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    assert_eq!(
        execute_source_batch(&mut dom, commands(), &cancel),
        Err(expected)
    );
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    assert_eq!(dom.usage(), usage);
    assert!(dom.pending_style_edit().is_none());
    assert!(
        dom.snapshot()
            .slot(slot)
            .unwrap()
            .manual_assignments()
            .is_empty()
    );
    assert!(
        dom.snapshot()
            .slot(slot)
            .unwrap()
            .assigned_nodes()
            .is_empty()
    );
    assert_eq!(
        dom.snapshot()
            .node(text)
            .unwrap()
            .manual_assignment()
            .unwrap(),
        None
    );
    assert!(
        dom.changes_since(before.stamp())
            .unwrap()
            .changes()
            .is_empty()
    );
}

#[test]
fn source_slot_inputs_admit_the_whole_raw_batch_at_exact_and_one_short() {
    use surgeist_dom::DomResource;
    let cancel = AtomicBool::new(false);
    for (limit, duplicate, raw_count) in [(1, false, 2), (2, false, 2), (2, true, 3), (3, true, 3)]
    {
        let (mut dom, slot, text) = manual_slot_input_fixture(limit);
        let commands = || {
            let mut batch = DomBatch::new();
            // Each command fits independently. Duplicates and the final
            // single-entry set cannot reduce aggregate raw admission.
            batch.assign_slot(
                slot,
                if duplicate {
                    vec![text.into(), text.into()]
                } else {
                    vec![text.into()]
                },
            );
            batch.assign_slot(slot, vec![text.into()]);
            batch
        };
        let before = dom.snapshot();
        let usage = dom.usage();
        if limit < raw_count {
            let expected = DomError::ResourceLimit {
                resource: DomResource::SlotAssignmentInputs,
                limit,
            };
            assert_eq!(dom.apply(commands(), &cancel).unwrap_err(), expected);
            assert_eq!(
                execute_source_batch(&mut dom, commands(), &cancel),
                Err(expected)
            );
            assert_eq!(dom.snapshot().stamp(), before.stamp());
            assert_eq!(dom.usage(), usage);
            assert!(
                dom.snapshot()
                    .slot(slot)
                    .unwrap()
                    .manual_assignments()
                    .is_empty()
            );
            assert!(
                dom.changes_since(before.stamp())
                    .unwrap()
                    .changes()
                    .is_empty()
            );
        } else {
            execute_source_batch(&mut dom, commands(), &cancel).unwrap();
            assert_eq!(
                dom.snapshot().slot(slot).unwrap().manual_assignments(),
                &[text]
            );
            assert_eq!(dom.snapshot().slot(slot).unwrap().assigned_nodes(), &[text]);
            assert_eq!(
                dom.snapshot()
                    .node(text)
                    .unwrap()
                    .manual_assignment()
                    .unwrap(),
                Some(slot)
            );
            assert_eq!(
                dom.snapshot()
                    .node(text)
                    .unwrap()
                    .stored_assigned_slot()
                    .unwrap(),
                Some(slot)
            );
            assert!(before.slot(slot).unwrap().manual_assignments().is_empty());
            assert_ne!(dom.snapshot().stamp(), before.stamp());
        }
        assert!(dom.pending_style_edit().is_none());
    }
}

#[test]
fn source_slot_inputs_preserve_cancellation_and_command_limit_precedence() {
    use surgeist_dom::DomResource;
    let (mut dom, slot, text) = manual_slot_input_fixture(0);
    let commands = || {
        let mut batch = DomBatch::new();
        batch.assign_slot(slot, vec![text.into()]);
        batch
    };
    let before = dom.snapshot();
    let cancelled = AtomicBool::new(true);
    assert_eq!(
        dom.apply(commands(), &cancelled).unwrap_err(),
        DomError::Cancelled
    );
    assert_eq!(
        execute_source_batch(&mut dom, commands(), &cancelled),
        Err(DomError::Cancelled)
    );
    dom.set_limits(DomLimits::new(32, 4096, 0, 32, 0, 8, 65536).unwrap())
        .unwrap();
    let cancel = AtomicBool::new(false);
    let expected = DomError::ResourceLimit {
        resource: DomResource::Commands,
        limit: 0,
    };
    assert_eq!(dom.apply(commands(), &cancel).unwrap_err(), expected);
    assert!(
        matches!(dom.begin_style_batch(commands(), surgeist_dom::DomStyleEditLimits::new(0, 0, 0, 0, 0, 0), &cancel), Err(error) if error == expected)
    );
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    assert!(dom.pending_style_edit().is_none());
}

#[test]
fn source_slot_inputs_are_admitted_before_an_earlier_style_can_publish() {
    use surgeist_dom::DomResource;
    let cancel = AtomicBool::new(false);
    let (mut dom, slot, text) = manual_slot_input_fixture(0);
    let before = dom.snapshot();
    let node = before.node(text).unwrap();
    let document = node.owner_document().unwrap();
    let host = node.parent().unwrap();
    let commands = || {
        let mut batch = DomBatch::new();
        let style = batch.create_element_ns(document, Namespace::html(), "style");
        batch.insert_before(&style, host, None);
        // This later raw input must be checked before even the preceding
        // Style insertion can publish a policy request or creation identity.
        batch.assign_slot(slot, vec![text.into()]);
        batch
    };
    let expected = DomError::ResourceLimit {
        resource: DomResource::SlotAssignmentInputs,
        limit: 0,
    };
    assert_eq!(dom.apply(commands(), &cancel).unwrap_err(), expected);
    assert!(
        matches!(dom.begin_style_batch(commands(), source_limits(), &cancel), Err(error) if error == expected)
    );
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    assert!(dom.pending_style_edit().is_none());
    assert_eq!(dom.snapshot().node(host).unwrap().children().len(), 1);
    assert!(
        dom.changes_since(before.stamp())
            .unwrap()
            .changes()
            .is_empty()
    );
}

#[test]
fn raised_independent_meta_budget_reaches_the_exact_published_source_tail() {
    // SETUP: the same-command headroom scan spends original Meta work. Locate
    // a budget reaching real publication, with too little left for the later
    // real Meta phase. This search is not an independent Meta tariff oracle;
    // the recovery, lowering, cancellation and metadata assertions below are.
    let mut left = 0;
    let mut right = 1_000_000;
    assert!(!meta_budget_reaches_first_style_request(left));
    assert!(meta_budget_reaches_first_style_request(right));
    while left < right {
        let middle = left.checked_add((right - left) / 2).unwrap();
        if meta_budget_reaches_first_style_request(middle) {
            right = middle;
        } else {
            left = middle + 1;
        }
    }
    let scan = left;
    assert!(scan > 1);
    for low in [scan, scan.checked_add(scan / 2).unwrap()] {
        resume_after_meta_work_refusal(low);
    }
}
fn meta_budget_reaches_first_style_request(low: u64) -> bool {
    use surgeist_dom::{DomMetaError, DomStyleProgress};
    let cancel = AtomicBool::new(false);
    let (mut dom, _, _, batch) = meta_recovery_fixture(low);
    let before = dom.snapshot();
    let mut lease = dom
        .begin_style_batch(batch, source_limits(), &cancel)
        .unwrap();
    match lease.advance(&cancel) {
        Ok(DomStyleProgress::Request(_)) => {
            assert!(lease.status().has_published());
            true
        }
        Err(DomError::Meta(DomMetaError::WorkLimit { limit })) if limit == low => {
            assert!(!lease.status().has_published());
            assert_eq!(lease.snapshot().stamp(), before.stamp());
            false
        }
        other => panic!("actual same-command Meta preflight: {other:?}"),
    }
}
fn meta_recovery_fixture(low: u64) -> (Dom, DomLimits, surgeist_dom::DocumentId, DomBatch) {
    let cancel = AtomicBool::new(false);
    let high = DomLimits::new(128, 32768, 512, 0, 0, 16, 1_000_000)
        .unwrap()
        .with_meta_processing_work(1_000_000);
    let mut dom = Dom::new(high).unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let container = batch.create_element_ns(&document, Namespace::html(), "div");
    let style = batch.create_element_ns(&document, Namespace::html(), "style");
    let meta = batch.create_element_ns(&document, Namespace::html(), "meta");
    batch.set_attribute(&meta, "http-equiv", "content-language");
    batch.set_attribute(&meta, "content", "he");
    batch.insert_before(&style, &container, None);
    batch.insert_before(&meta, &container, None);
    let created = dom.apply(batch, &cancel).unwrap();
    let document = created.document(&document).unwrap();
    let container = created.created(&container).unwrap().unwrap();
    dom.set_limits(high.with_meta_processing_work(low)).unwrap();
    let mut batch = DomBatch::new();
    batch.insert_before(container, document.node(), None);
    (dom, high, document, batch)
}
fn resume_after_meta_work_refusal(low: u64) {
    use surgeist_dom::{DomMetaError, DomStyleProgress};
    let cancel = AtomicBool::new(false);
    let (mut dom, high, document, batch) = meta_recovery_fixture(low);
    let before = dom.snapshot();
    let mut lease = dom
        .begin_style_batch(batch, source_limits(), &cancel)
        .unwrap();
    let operation = lease.operation();
    let DomStyleProgress::Request(request) = lease.advance(&cancel).unwrap() else {
        panic!("first Style policy boundary")
    };
    assert!(lease.status().has_published());
    lease
        .accept_inline_csp(&request, &block(&request), &cancel)
        .unwrap();
    assert_eq!(
        lease.advance(&cancel).unwrap_err(),
        DomError::Meta(DomMetaError::WorkLimit { limit: low })
    );
    let spent = lease.status().spent_work();
    drop(lease);
    let published = dom.snapshot().stamp();
    let old_limits = dom.limits();
    if low != 0 {
        assert_eq!(
            dom.set_limits(high.with_meta_processing_work(0))
                .unwrap_err(),
            DomError::Meta(DomMetaError::WorkLimit { limit: 0 })
        );
        assert_eq!(dom.limits(), old_limits);
        assert_eq!(dom.snapshot().stamp(), published);
    }
    cancel.store(true, std::sync::atomic::Ordering::Relaxed);
    assert!(matches!(
        dom.resume_style_edit(operation, source_limits(), &cancel),
        Err(DomError::Cancelled)
    ));
    assert_eq!(dom.limits(), old_limits);
    assert_eq!(dom.snapshot().stamp(), published);
    cancel.store(false, std::sync::atomic::Ordering::Relaxed);
    dom.set_limits(high).unwrap();
    assert_eq!(dom.snapshot().stamp(), published);
    let mut lease = dom
        .resume_style_edit(operation, source_limits(), &cancel)
        .unwrap();
    assert!(lease.status().spent_work() >= spent);
    assert!(
        matches!(
            lease.advance(&cancel).unwrap(),
            DomStyleProgress::Complete(_)
        ),
        "raising the independent bound must finish the retained command"
    );
    drop(lease);
    assert_eq!(
        dom.snapshot()
            .document(document)
            .unwrap()
            .metadata()
            .meta_state()
            .unwrap()
            .pragma_language(),
        Some(&"he".into())
    );
    assert_eq!(
        before
            .document(document)
            .unwrap()
            .metadata()
            .meta_state()
            .unwrap()
            .pragma_language(),
        None
    );
}
