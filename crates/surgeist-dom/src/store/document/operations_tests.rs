//! DOM3071 Document constructor and live clone requirements. These new APIs
//! have no prior executable boundary; expectations are independent source facts.
use super::*;
use crate::*;

fn cancel() -> AtomicBool {
    AtomicBool::new(false)
}
fn bounds(nodes: usize, units: usize) -> DomLimits {
    DomLimits::new(
        nodes,
        units,
        4096,
        1024,
        512,
        32,
        8_000_000,
        DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
    )
    .unwrap()
    .with_dispatch_custody(
        DomDispatchCustodyLimits::new(32, 1024, 8_000_000, 8_000_000, 1_000_000).unwrap(),
    )
}
fn dom() -> Dom {
    Dom::new(bounds(1024, 1_000_000)).unwrap()
}
fn apply(dom: &mut Dom, batch: DomBatch) -> DomCommit {
    dom.apply(batch, &cancel()).unwrap()
}
fn source_limits(outputs: usize) -> DomSourceLimits {
    DomSourceLimits::new(64_000_000, 2_000_000, 20_000_000, 128, outputs, 4096, 8)
}
fn source(dom: &mut Dom, edit: DomSourceEdit) -> DomSourceCompletion {
    let mut lease = dom
        .begin_source_edit(edit, source_limits(4096), &cancel())
        .unwrap();
    let DomSourceProgress::Complete(done) = lease.advance(&cancel()).unwrap() else {
        panic!("actual operation completes without external service")
    };
    done
}

#[test]
fn split_text_creation_uses_current_owner_document_global_after_adoption() {
    let mut dom = dom();
    let mut issuer = issuer(&mut dom);
    let a = hosted(&mut dom, &mut issuer);
    let b = hosted(&mut dom, &mut issuer);
    let mut batch = DomBatch::new();
    let document = batch.construct_document(
        issuer
            .document_constructor_context(&dom, b.global(), &cancel())
            .unwrap(),
    );
    let text = batch.create_text(a.document(), "abcd");
    let commit = apply(&mut dom, batch);
    let text = commit.created(&text).unwrap().unwrap();
    let document = commit.document(&document).unwrap();
    let mut batch = DomBatch::new();
    batch.adopt(text, document);
    apply(&mut dom, batch);
    let done = source(&mut dom, DomSourceEdit::split_text(text, 2));
    let Some(DomSourceOutput::Node(new)) = done.output() else {
        panic!("new Text output")
    };
    let snapshot = dom.snapshot();
    assert_eq!(snapshot.relevant_global(text).unwrap(), Some(a.global()));
    assert_eq!(snapshot.relevant_global(new).unwrap(), Some(b.global()));
    assert_eq!(snapshot.node(new).unwrap().owner_document(), Some(document));
    detached_document(&snapshot, document, Some(b.global()));
}

#[test]
fn split_text_runs_as_nested_source_without_completing_the_original_event() {
    let mut dom = dom();
    let mut issuer = issuer(&mut dom);
    let original = hosted(&mut dom, &mut issuer);
    let executor = dom.bind_source_event_executor(&cancel()).unwrap();
    let mut batch = DomBatch::new();
    let dialog = batch.create_element_ns(original.document(), Namespace::html(), "dialog");
    let text = batch.create_text(original.document(), "abcd");
    let commit = apply(&mut dom, batch);
    let target = commit.created(&dialog).unwrap().unwrap();
    let text = commit.created(&text).unwrap().unwrap();
    let request = {
        let mut lease = dom
            .begin_source_edit(
                DomSourceEdit::show_dialog(target),
                source_limits(4096),
                &cancel(),
            )
            .unwrap();
        let DomSourceProgress::Request(DomSourceRequest::Event(request)) =
            lease.advance(&cancel()).unwrap()
        else {
            panic!("actual beforetoggle")
        };
        request
    };
    let permit = executor.begin_dispatch(&dom, &request, &cancel()).unwrap();
    let guard = executor.enter_dispatch(&dom, &permit, &cancel()).unwrap();
    let done = source(&mut dom, DomSourceEdit::split_text(text, 2));
    let Some(DomSourceOutput::Node(new)) = done.output() else {
        panic!("actual nested split result")
    };
    assert_eq!(
        dom.snapshot().node(text).unwrap().data(),
        Some(&"ab".into())
    );
    assert_eq!(dom.snapshot().node(new).unwrap().data(), Some(&"cd".into()));
    assert_eq!(
        dom.pending_source_edit().unwrap().operation(),
        request.operation()
    );
    drop(guard);
    assert_eq!(
        dom.pending_source_edit().unwrap().operation(),
        request.operation()
    );
    // The original event has not been dispatched or acknowledged by this split.
}
fn issuer(dom: &mut Dom) -> DomHostedSource {
    dom.register_hosted_source(
        DomHostedLimits::new(
            DomHostedRecordLimits::new(32, 32, 32, 32, 32, 32, 32),
            32_000_000,
            32_000_000,
            20_000_000,
        ),
        &cancel(),
    )
    .unwrap()
}
// Exercise the actual trusted DOM lifecycle protocol. This tests canonical
// Global authentication, not Runtime's external agent/clock/entropy execution.
fn hosted(dom: &mut Dom, source: &mut DomHostedSource) -> DomHostedCreation {
    let host = source.issue_host(&cancel()).unwrap();
    let mut preparation = Some(source.prepare_top_level(host, &cancel()).unwrap());
    let mut prepared = Some(source.seal_top_level(&mut preparation, &cancel()).unwrap());
    let mut lease = dom
        .begin_hosted_top_level(source, &mut prepared, &cancel())
        .unwrap();
    loop {
        let publication = match lease.step(dom).unwrap() {
            DomHostedCreationStep::PermissionsPolicy(request) => {
                let mut value = Some(
                    source
                        .seal_permissions_policy(
                            request,
                            DomInitialPermissionsPolicy::new(
                                DomPermissionsPolicyValue::Enabled,
                                DomPermissionsPolicyValue::Enabled,
                            ),
                            &cancel(),
                        )
                        .unwrap(),
                );
                lease
                    .accept_permissions_policy(dom, source, &mut value, &cancel())
                    .unwrap();
                continue;
            }
            DomHostedCreationStep::NativeExecution(request) => {
                let global = source.issue_global(request, &cancel()).unwrap();
                source
                    .seal_native_execution(request, global, 0, &cancel())
                    .unwrap()
            }
            DomHostedCreationStep::MakeActive(request) => {
                source.seal_make_active(request, &cancel()).unwrap()
            }
            DomHostedCreationStep::FinishLoading(request) => {
                source.seal_finish_loading(request, 100, &cancel()).unwrap()
            }
            DomHostedCreationStep::InitialEntry(request) => {
                let entry = source.issue_entry(request, &cancel()).unwrap();
                let nav = source.issue_navigable(request, &cancel()).unwrap();
                let uuid = |last| {
                    let mut out = [0; 36];
                    for (a, b) in out
                        .iter_mut()
                        .zip("00000000-0000-4000-8000-000000000000".encode_utf16())
                    {
                        *a = b;
                    }
                    out[35] = last;
                    out
                };
                let mut identity = Some(DomHostedEntryInitialization::new(uuid(49), uuid(50)));
                source
                    .seal_initial_entry(request, entry, nav, &mut identity, &cancel())
                    .unwrap()
            }
            DomHostedCreationStep::Complete(_) => break,
        };
        lease
            .accept_publication(dom, source, &mut Some(publication), &cancel())
            .unwrap();
    }
    lease.finish(dom, source, &cancel()).unwrap()
}
fn detached_document(
    snapshot: &DomSnapshot,
    document: DocumentId,
    global: Option<DomHostedGlobalId>,
) {
    let view = snapshot.document(document).unwrap();
    assert_eq!(
        snapshot.node(document.node()).unwrap().owner_document(),
        None
    );
    assert_eq!(
        snapshot
            .state
            .node(snapshot.owner(), document.node())
            .unwrap()
            .document(),
        document
    );
    assert_eq!(snapshot.relevant_global(document.node()).unwrap(), global);
    assert!(
        !view
            .metadata()
            .document_environment()
            .has_ever_had_navigable()
    );
    assert!(!view.is_fully_active().unwrap());
    assert_eq!(view.metadata().initial_permissions_policy(), None);
    assert_eq!(
        view.focused_area_designation(),
        DomFocusedAreaDesignation::KnownNull
    );
    assert_eq!(view.active_range(), DomActiveRangeState::Known(None));
}
#[test]
fn associated_global_constructor_copies_exact_origin_but_keeps_defaults_and_no_window_association()
{
    let mut dom = dom();
    let mut issuer = issuer(&mut dom);
    let a = hosted(&mut dom, &mut issuer);
    let b = hosted(&mut dom, &mut issuer);
    let old = dom.snapshot();
    assert_ne!(
        old.document(a.document()).unwrap().metadata().origin(),
        old.document(b.document()).unwrap().metadata().origin()
    );
    let mut batch = DomBatch::new();
    let ordinary = batch.create_document();
    let from_a = batch.construct_document(
        issuer
            .document_constructor_context(&dom, a.global(), &cancel())
            .unwrap(),
    );
    let from_b = batch.construct_document(
        issuer
            .document_constructor_context(&dom, b.global(), &cancel())
            .unwrap(),
    );
    let child = batch.create_element(&from_a, "MiXeD");
    let xml = batch.create_xml_document(&from_a, Namespace::none(), "root", None);
    let commit = apply(&mut dom, batch);
    let snapshot = dom.snapshot();
    let ordinary = commit.document(&ordinary).unwrap();
    let first = commit.document(&from_a).unwrap();
    let second = commit.document(&from_b).unwrap();
    for (copy, origin) in [(first, a.document()), (second, b.document())] {
        let metadata = snapshot.document(copy).unwrap().metadata();
        assert_eq!(metadata.interface(), DomDocumentInterface::Document);
        assert_eq!(metadata.class(), DomDocumentClass::Xml);
        assert_eq!(
            metadata.origin(),
            old.document(origin).unwrap().metadata().origin()
        );
        assert_eq!(metadata.url(), &DocumentString::from("about:blank"));
        assert_eq!(
            metadata.content_type(),
            &DocumentString::from("application/xml")
        );
        assert_eq!(metadata.encoding(), &DocumentString::from("UTF-8"));
        assert_eq!(metadata.mode(), HtmlDocumentMode::NoQuirks);
        assert!(!metadata.allow_declarative_shadow_roots());
        assert_eq!(metadata.custom_element_registry(), DomRegistryBinding::Null);
    }
    assert_ne!(
        snapshot.document(ordinary).unwrap().metadata().origin(),
        snapshot.document(first).unwrap().metadata().origin()
    );
    assert_eq!(
        snapshot.document(ordinary).unwrap().metadata().interface(),
        DomDocumentInterface::Document
    );
    assert_eq!(
        snapshot
            .document(commit.document(&xml).unwrap())
            .unwrap()
            .metadata()
            .interface(),
        DomDocumentInterface::XmlDocument
    );
    detached_document(&snapshot, first, Some(a.global()));
    detached_document(&snapshot, second, Some(b.global()));
    assert_eq!(
        snapshot
            .relevant_global(commit.created(&child).unwrap().unwrap())
            .unwrap(),
        Some(a.global())
    );
    assert_eq!(
        snapshot
            .node(commit.created(&child).unwrap().unwrap())
            .unwrap()
            .node_name(),
        DocumentString::from("MiXeD")
    );
    assert_eq!(dom.global_document(a.global()).unwrap(), a.document());
    assert_eq!(dom.global_document(b.global()).unwrap(), b.document());
}
#[test]
fn constructor_context_refuses_foreign_stale_and_expired_authority_without_publication() {
    let mut dom = dom();
    let mut issuer = issuer(&mut dom);
    let original = hosted(&mut dom, &mut issuer);
    let mut other = self::dom();
    let context = issuer
        .document_constructor_context(&dom, original.global(), &cancel())
        .unwrap();
    let mut batch = DomBatch::new();
    batch.construct_document(context);
    assert_eq!(
        other.apply(batch, &cancel()).unwrap_err(),
        DomError::ForeignOwner
    );
    assert!(other.snapshot().nodes().next().is_none());
    assert!(matches!(
        issuer.document_constructor_context(&other, original.global(), &cancel()),
        Err(DomHostedError::ForeignSource)
    ));
    let stale = issuer
        .document_constructor_context(&dom, original.global(), &cancel())
        .unwrap();
    issuer
        .retire(
            crate::hosted::Role::Global,
            original.global().source,
            original.global().slot,
            original.global().generation,
            &cancel(),
        )
        .unwrap();
    let before = dom.snapshot();
    let usage = dom.usage();
    let mut batch = DomBatch::new();
    batch.construct_document(stale);
    assert_eq!(
        dom.apply(batch, &cancel()).unwrap_err(),
        DomError::DocumentConstructor(DomDocumentConstructorError::StaleAssociation)
    );
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    assert_eq!(dom.usage(), usage);
    let live = hosted(&mut dom, &mut issuer);
    let expired = issuer
        .document_constructor_context(&dom, live.global(), &cancel())
        .unwrap();
    drop(issuer);
    let before = dom.snapshot();
    let mut batch = DomBatch::new();
    batch.construct_document(expired);
    assert_eq!(
        dom.apply(batch, &cancel()).unwrap_err(),
        DomError::DocumentConstructor(DomDocumentConstructorError::ExpiredIssuer)
    );
    assert_eq!(dom.snapshot().stamp(), before.stamp());
}
#[test]
fn real_source_dispatch_scope_allows_nested_constructor_and_leaves_original_event_pending() {
    let mut dom = dom();
    let mut issuer = issuer(&mut dom);
    let original = hosted(&mut dom, &mut issuer);
    let executor = dom.bind_source_event_executor(&cancel()).unwrap();
    let mut batch = DomBatch::new();
    let dialog = batch.create_element_ns(original.document(), Namespace::html(), "dialog");
    let target = apply(&mut dom, batch).created(&dialog).unwrap().unwrap();
    let request = {
        let mut lease = dom
            .begin_source_edit(
                DomSourceEdit::show_dialog(target),
                source_limits(4096),
                &cancel(),
            )
            .unwrap();
        let DomSourceProgress::Request(DomSourceRequest::Event(request)) =
            lease.advance(&cancel()).unwrap()
        else {
            panic!("actual beforetoggle")
        };
        request
    };
    let permit = executor.begin_dispatch(&dom, &request, &cancel()).unwrap();
    let guard = executor.enter_dispatch(&dom, &permit, &cancel()).unwrap();
    let context = issuer
        .document_constructor_context(&dom, original.global(), &cancel())
        .unwrap();
    let done = source(&mut dom, DomSourceEdit::construct_document(context));
    let Some(DomSourceOutput::Document(document)) = done.output() else {
        panic!("one actual Document result")
    };
    assert_eq!(done.published_node(), Some(document.node()));
    assert_eq!(
        dom.pending_source_edit().unwrap().operation(),
        request.operation()
    );
    assert_eq!(
        dom.snapshot()
            .document(document)
            .unwrap()
            .metadata()
            .origin(),
        dom.snapshot()
            .document(original.document())
            .unwrap()
            .metadata()
            .origin()
    );
    drop(guard);
    assert!(matches!(
        issuer.document_constructor_context(&dom, original.global(), &cancel()),
        Err(DomHostedError::Dom(DomError::IncompleteSourceOperation))
    ));
    // No completion attestation is invented for the unserviced original event.
}
#[test]
fn shallow_and_deep_document_clones_preserve_interface_fields_and_fresh_ownership() {
    let mut dom = dom();
    let mut issuer = issuer(&mut dom);
    let hosted = hosted(&mut dom, &mut issuer);
    let mut batch = DomBatch::new();
    let xml = batch.create_xml_document(hosted.document(), Namespace::none(), "root", None);
    let comment = batch.create_comment(&xml, "tail");
    batch.insert_before(&comment, &xml, None);
    let xml = apply(&mut dom, batch).document(&xml).unwrap();
    for original in [hosted.document(), xml] {
        for deep in [false, true] {
            let old = dom.snapshot();
            let metadata = old.document(original).unwrap().metadata().clone();
            let mut batch = DomBatch::new();
            let clone = batch.clone_node(original, deep);
            let next = batch.create_comment(&clone, "next ticket");
            let commit = apply(&mut dom, batch);
            let clone = commit.document(&clone).unwrap();
            let snapshot = dom.snapshot();
            let actual = snapshot.document(clone).unwrap().metadata();
            assert_ne!(clone, original);
            assert_eq!(actual.interface(), metadata.interface());
            assert_eq!(actual.class(), metadata.class());
            assert_eq!(actual.origin(), metadata.origin());
            assert_eq!(actual.url(), metadata.url());
            assert_eq!(actual.encoding(), metadata.encoding());
            assert_eq!(actual.content_type(), metadata.content_type());
            assert_eq!(actual.mode(), metadata.mode());
            assert_eq!(
                actual.allow_declarative_shadow_roots(),
                metadata.allow_declarative_shadow_roots()
            );
            assert_eq!(actual.custom_element_registry(), DomRegistryBinding::Null);
            detached_document(
                &snapshot,
                clone,
                old.relevant_global(original.node()).unwrap(),
            );
            let old_children = old.node(original.node()).unwrap().children();
            let new_children = snapshot.node(clone.node()).unwrap().children();
            assert_eq!(
                new_children.len(),
                if deep { old_children.len() } else { 0 }
            );
            for (a, b) in old_children.iter().zip(new_children) {
                assert_ne!(a, b);
                assert_eq!(snapshot.node(*b).unwrap().owner_document(), Some(clone));
                assert_eq!(
                    snapshot.node(*b).unwrap().node_name(),
                    old.node(*a).unwrap().node_name()
                );
            }
            assert_eq!(
                snapshot
                    .node(commit.created(&next).unwrap().unwrap())
                    .unwrap()
                    .owner_document(),
                Some(clone)
            );
            assert_eq!(old.node(original.node()).unwrap().children(), old_children);
        }
    }
}
#[test]
fn deep_live_clone_keeps_control_script_template_and_shadow_clone_rules() {
    use crate::{
        HtmlShadowRootMode as Mode, HtmlShadowRootProperties, HtmlSlotAssignment as Assignment,
    };
    let mut dom = dom();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let root = batch.create_element_ns(&document, Namespace::html(), "div");
    batch.insert_before(&root, &document, None);
    let input = batch.create_element_ns(&document, Namespace::html(), "input");
    batch.set_input_value(&input, "dirty");
    batch.set_input_checked(&input, true);
    batch.set_input_indeterminate(&input, true);
    let textarea = batch.create_element_ns(&document, Namespace::html(), "textarea");
    batch.set_textarea_value(&textarea, "raw");
    let script = batch.create_element_ns(&document, Namespace::html(), "script");
    batch.set_attribute(&script, "src", "old");
    let template = batch.create_element_ns(&document, Namespace::html(), "template");
    let host = batch.create_element_ns(&document, Namespace::html(), "div");
    let shadow = batch.attach_shadow(
        &host,
        HtmlShadowRootProperties::new(Mode::Closed, Assignment::Named, false, true, false),
    );
    let text = batch.create_text(&document, "shadow");
    batch.insert_before(&text, &shadow, None);
    for node in [&input, &textarea, &script, &template, &host] {
        batch.insert_before(node, &root, None);
    }
    let commit = apply(&mut dom, batch);
    let original = commit.document(&document).unwrap();
    let template = commit.created(&template).unwrap().unwrap();
    let contents = dom
        .snapshot()
        .node(template)
        .unwrap()
        .as_template()
        .unwrap()
        .contents()
        .unwrap();
    let inert = dom
        .snapshot()
        .node(contents)
        .unwrap()
        .owner_document()
        .unwrap();
    let mut batch = DomBatch::new();
    let text = batch.create_text(inert, "template");
    batch.insert_before(&text, contents, None);
    apply(&mut dom, batch);
    let done = source(&mut dom, DomSourceEdit::clone_node(original.node(), true));
    let Some(DomSourceOutput::Document(copy)) = done.output() else {
        panic!("Document clone result")
    };
    let snapshot = dom.snapshot();
    let root = snapshot.node(copy.node()).unwrap().children()[0];
    let children = snapshot.node(root).unwrap().children();
    let input = snapshot.node(children[0]).unwrap().as_input().unwrap();
    assert_eq!(input.value(), "dirty".encode_utf16().collect::<Vec<_>>());
    assert!(input.state().flags().dirty_value());
    assert!(input.state().flags().checkedness());
    assert!(input.state().flags().dirty_checkedness());
    assert!(input.state().flags().indeterminate());
    let textarea = snapshot.node(children[1]).unwrap().as_textarea().unwrap();
    assert_eq!(
        textarea
            .value(DomTextareaReadLimits::new(65536, 1_000_000), &cancel())
            .unwrap(),
        DocumentString::from("raw")
    );
    assert!(textarea.state().flags().dirty_value());
    let script = snapshot
        .node(children[2])
        .unwrap()
        .as_html_script()
        .unwrap();
    assert!(script.flags().already_started());
    assert_eq!(script.preparation_document(), None);
    let contents = snapshot
        .node(children[3])
        .unwrap()
        .as_template()
        .unwrap()
        .contents()
        .unwrap();
    let new_inert = snapshot.node(contents).unwrap().owner_document().unwrap();
    assert_ne!(new_inert, inert);
    assert_ne!(new_inert, copy);
    let template_text = snapshot.node(contents).unwrap().children()[0];
    assert_eq!(
        snapshot.node(template_text).unwrap().data(),
        Some(&DocumentString::from("template"))
    );
    let shadow = snapshot
        .node(children[4])
        .unwrap()
        .hosted_shadow_root()
        .unwrap();
    assert!(snapshot.shadow_root(shadow).unwrap().state().clonable());
    assert_eq!(snapshot.node(shadow).unwrap().owner_document(), Some(copy));
    let text = snapshot.node(shadow).unwrap().children()[0];
    assert_eq!(
        snapshot.node(text).unwrap().data(),
        Some(&DocumentString::from("shadow"))
    );
}
#[test]
fn unpublished_clone_refusal_rolls_back_and_import_refusals_remain_atomic() {
    let mut dom = dom();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let root = batch.create_element(&document, "root");
    batch.insert_before(&root, &document, None);
    let text = batch.create_text(&document, "text");
    batch.insert_before(&text, &root, None);
    let original = apply(&mut dom, batch).document(&document).unwrap();
    let old = dom.snapshot();
    let usage = dom.usage();
    {
        let mut lease = dom
            .begin_source_edit(
                DomSourceEdit::clone_node(original.node(), true),
                source_limits(0),
                &cancel(),
            )
            .unwrap();
        assert!(matches!(
            lease.advance(&cancel()),
            Err(DomError::SourceOutputLimit)
        ));
    }
    assert!(dom.pending_source_edit().is_none());
    assert_eq!(dom.snapshot().stamp(), old.stamp());
    assert_eq!(dom.usage(), usage);
    let done = source(&mut dom, DomSourceEdit::clone_node(original.node(), true));
    let Some(DomSourceOutput::Document(copy)) = done.output() else {
        panic!("one root output")
    };
    assert_eq!(done.published_node(), Some(copy.node()));
    assert_eq!(dom.usage().managed_nodes(), usage.managed_nodes() + 3);
    assert_eq!(old.node(original.node()).unwrap().children().len(), 1);
    let before = dom.snapshot();
    let usage = dom.usage();
    let mut batch = DomBatch::new();
    batch.import_node(&before, original.node(), copy, true);
    assert_eq!(
        dom.apply(batch, &cancel()).unwrap_err(),
        DomError::NotSupported
    );
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    assert_eq!(dom.usage(), usage);
    let mut batch = DomBatch::new();
    batch.clone_node(original, false);
    assert_eq!(
        dom.apply(batch, &AtomicBool::new(true)).unwrap_err(),
        DomError::Cancelled
    );
    assert_eq!(dom.snapshot().stamp(), before.stamp());
}

#[test]
fn constructor_exact_metadata_admission_and_one_short_leave_no_partial_document() {
    for allowance in [31, 30] {
        let mut dom = dom();
        let mut issuer = issuer(&mut dom);
        let original = hosted(&mut dom, &mut issuer);
        let context = issuer
            .document_constructor_context(&dom, original.global(), &cancel())
            .unwrap();
        let before = dom.snapshot();
        let usage = dom.usage();
        dom.set_limits(bounds(
            usage.managed_nodes() + 1,
            usage.string_code_units() + allowance,
        ))
        .unwrap();
        let mut batch = DomBatch::new();
        let document = batch.construct_document(context);
        let result = dom.apply(batch, &cancel());
        if allowance == 31 {
            let document = result.unwrap().document(&document).unwrap();
            assert_eq!(dom.usage().managed_nodes(), usage.managed_nodes() + 1);
            assert_eq!(
                dom.usage().string_code_units(),
                usage.string_code_units() + 31
            );
            detached_document(&dom.snapshot(), document, Some(original.global()));
        } else {
            assert_eq!(
                result.unwrap_err(),
                DomError::ResourceLimit {
                    resource: DomResource::StringCodeUnits,
                    limit: usage.string_code_units() + 30
                }
            );
            assert_eq!(dom.snapshot().stamp(), before.stamp());
            assert_eq!(dom.usage(), usage);
        }
    }
}
#[test]
fn live_clone_refuses_direct_shadow_root_foreign_and_retired_source_but_import_keeps_history() {
    let mut dom = dom();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let host = batch.create_element_ns(&document, Namespace::html(), "div");
    let root = batch.attach_shadow(&host, DomShadowRootInit::new(HtmlShadowRootMode::Open));
    let leaf = batch.create_text(&document, "historical");
    let commit = apply(&mut dom, batch);
    let doc = commit.document(&document).unwrap();
    let shadow = commit.created(&root).unwrap().unwrap();
    let leaf = commit.created(&leaf).unwrap().unwrap();
    let old = dom.snapshot();
    for import in [false, true] {
        let mut batch = DomBatch::new();
        if import {
            batch.import_node(&old, shadow, doc, false);
        } else {
            batch.clone_node(shadow, false);
        }
        assert_eq!(
            dom.apply(batch, &cancel()).unwrap_err(),
            DomError::NotSupported
        );
        assert_eq!(dom.snapshot().stamp(), old.stamp());
    }
    let mut other = self::dom();
    let mut batch = DomBatch::new();
    let foreign = batch.create_document();
    let foreign = apply(&mut other, batch).document(&foreign).unwrap();
    let mut batch = DomBatch::new();
    batch.clone_node(foreign, false);
    assert_eq!(
        dom.apply(batch, &cancel()).unwrap_err(),
        DomError::ForeignOwner
    );
    let mut batch = DomBatch::new();
    batch.destroy(leaf);
    apply(&mut dom, batch);
    let mut batch = DomBatch::new();
    batch.clone_node(leaf, false);
    assert_eq!(
        dom.apply(batch, &cancel()).unwrap_err(),
        DomError::StaleNode
    );
    let mut batch = DomBatch::new();
    let imported = batch.import_node(&old, leaf, doc, false);
    let copy = apply(&mut dom, batch).created(&imported).unwrap().unwrap();
    assert_ne!(copy, leaf);
    assert_eq!(
        dom.snapshot().node(copy).unwrap().data(),
        Some(&DocumentString::from("historical"))
    );
}
fn service_style(
    lease: &mut DomSourceLease<'_>,
    request: &DomStyleRequest,
    roots: &mut Vec<DomStyleCssomRoot>,
) {
    use surgeist_css::{
        CssParser, CssParserContext, query::media::CssMediaQueryList, source::CssUtf16InputLimits,
    };
    use surgeist_cssom::{
        CssomEditGuard, CssomInputRole, CssomInputVersion, CssomSheetInputs, CssomStore,
    };
    use surgeist_html::elements::style::HtmlStyleUpdateAction;
    match request.action() {
        DomStyleAction::Update(HtmlStyleUpdateAction::CheckInlineCsp) => lease
            .accept_inline_csp(
                request,
                &DomStyleInlineCspObservation::complete_empty(request.source(), 1),
                &cancel(),
            )
            .unwrap(),
        DomStyleAction::Update(HtmlStyleUpdateAction::CreateSheet) => {
            let root = DomStyleCssomRoot::new(
                request.cssom_root().unwrap(),
                CssomStore::new(Default::default(), Default::default()).unwrap(),
            );
            lease.bind_cssom_root(request, &root, &cancel()).unwrap();
            let request = lease
                .qualify_cssom(request, root.root(), &root.snapshot().unwrap())
                .unwrap();
            let version = CssomInputVersion {
                role: CssomInputRole::Owner,
                identity: "live-clone-style".into(),
                revision: 1,
            };
            lease
                .bind_cssom_owner(
                    &request,
                    DomStyleCssomOwnerBinding::new(&request, version.clone(), "clone-owner".into())
                        .unwrap(),
                    &cancel(),
                )
                .unwrap();
            let parser = CssParser::new(CssParserContext::default(), Default::default());
            let body = parser
                .parse_sheet_utf16(
                    request.source().text().clone(),
                    CssUtf16InputLimits::default(),
                    || Ok::<_, ()>(()),
                )
                .unwrap();
            let media = CssMediaQueryList::parse_utf16(
                &parser,
                Arc::new(Vec::new()),
                CssUtf16InputLimits::default(),
                || Ok::<_, ()>(()),
            )
            .unwrap();
            let mut inputs = CssomSheetInputs::external(version, true);
            inputs.owner = Some("clone-owner".into());
            let mut edit = root
                .batch(CssomEditGuard::from_snapshot(&root.snapshot().unwrap()))
                .unwrap();
            let construction = edit
                .create_parsed_sheet_with_media_reports(body, media, inputs)
                .unwrap();
            let commit = root.commit(&mut Some(edit)).unwrap();
            let snapshot = root.snapshot().unwrap();
            let _notification = lease
                .accept_cssom_publication(&request, commit, snapshot, Some(construction.ticket()))
                .unwrap();
            roots.push(root);
        }
        action => panic!("actual empty Style creation, got {action:?}"),
    }
}
#[test]
fn document_clone_retains_real_style_tail_across_lease_drop_and_cancellation_and_returns_one_ticket()
 {
    let mut dom = dom();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let style = batch.create_element_ns(&document, Namespace::html(), "style");
    let commit = apply(&mut dom, batch);
    let original = commit.document(&document).unwrap();
    let style = commit.created(&style).unwrap().unwrap();
    let mut roots = Vec::new();
    {
        let mut lease = dom
            .begin_source_edit(
                DomSourceEdit::insert_before(style, original.node(), None),
                source_limits(4096),
                &cancel(),
            )
            .unwrap();
        loop {
            match lease.advance(&cancel()).unwrap() {
                DomSourceProgress::Complete(_) => break,
                DomSourceProgress::Request(DomSourceRequest::Style(request)) => {
                    service_style(&mut lease, &request, &mut roots)
                }
                other => panic!("actual Style service: {other:?}"),
            }
        }
    }
    let nodes = dom.usage().managed_nodes();
    let (operation, copy, copy_style) = {
        let mut lease = dom
            .begin_source_edit(
                DomSourceEdit::clone_node(original.node(), true),
                source_limits(4096),
                &cancel(),
            )
            .unwrap();
        let DomSourceProgress::Request(DomSourceRequest::Style(request)) =
            lease.advance(&cancel()).unwrap()
        else {
            panic!("connected clone's actual Style hook")
        };
        assert_ne!(request.source().element(), style);
        assert_ne!(request.source().document(), original);
        (
            lease.operation(),
            request.source().document(),
            request.source().element(),
        )
    };
    assert!(dom.pending_source_edit().unwrap().has_published());
    assert_eq!(dom.usage().managed_nodes(), nodes + 2);
    let mut lease = dom
        .resume_source_edit(operation, source_limits(4096), &cancel())
        .unwrap();
    let mut admitted = 0;
    loop {
        match lease.advance(&cancel()).unwrap() {
            DomSourceProgress::Complete(done) => {
                assert_eq!(done.operation(), operation);
                assert_eq!(done.output(), Some(DomSourceOutput::Document(copy)));
                assert_eq!(done.published_node(), Some(copy.node()));
                break;
            }
            DomSourceProgress::Request(DomSourceRequest::Style(request)) => {
                assert_eq!(request.source().element(), copy_style);
                assert_eq!(request.source().document(), copy);
                service_style(&mut lease, &request, &mut roots);
                admitted += 1;
                if admitted == 2 {
                    assert_eq!(
                        lease.advance(&AtomicBool::new(true)).unwrap_err(),
                        DomError::Cancelled
                    );
                }
            }
            other => panic!("actual clone service: {other:?}"),
        }
    }
    assert_eq!(
        admitted, 2,
        "original CSP/creation stages each serviced once"
    );
    drop(lease);
    assert_eq!(dom.usage().managed_nodes(), nodes + 2);
    assert_eq!(
        dom.snapshot().node(copy.node()).unwrap().children(),
        &[copy_style]
    );
    assert!(dom.pending_source_edit().is_none());
}

#[test]
fn parsed_document_clone_copies_actual_url_encoding_and_mode_without_parser_or_policy_history() {
    use surgeist_html_parser::{HtmlDocumentParseRequest, HtmlParseLimits, HtmlSource};
    use surgeist_html_semantics::{
        HtmlBaseEncoding, HtmlBaseFactsRevision, HtmlBaseProducerId, HtmlBaseUrlRecord,
        HtmlBaseUrlScheme,
    };
    let mut dom = dom();
    let mut batch = DomBatch::new();
    let origin = batch.create_document();
    let origin = apply(&mut dom, batch).document(&origin).unwrap();
    let snapshot = dom.snapshot();
    let origin = snapshot
        .document(origin)
        .unwrap()
        .metadata()
        .origin()
        .clone();
    let parser = HtmlParseLimits::builder()
        .source_units(100)
        .buffered_utf16_units(100)
        .token_attributes(100)
        .auxiliary_entries(100_000)
        .active_formatting_entries(100_000)
        .managed_nodes(100)
        .managed_edges(100)
        .managed_attributes(100)
        .managed_utf16_units(1000)
        .source_map_entries(1000)
        .source_map_payload_units(1000)
        .diagnostics(100)
        .diagnostic_payload_units(1000)
        .context_effects(100)
        .open_elements(100)
        .tree_depth(100)
        .managed_capacity_bytes(1_000_000)
        .peak_transient_bytes(1_000_000)
        .atomic_layout_bytes(1_000_000)
        .work_credits(10_000_000)
        .finish()
        .unwrap();
    let report = Arc::new(
        HtmlDocumentParseRequest::offline(HtmlSource::utf8("<p>source</p>"), parser)
            .parse()
            .unwrap(),
    );
    let url = HtmlBaseUrlRecord::observed(
        HtmlBaseFactsRevision::new(HtmlBaseProducerId::fresh(), 1, 1, 1),
        "https://example.test/source/".into(),
        HtmlBaseUrlScheme::Other,
        false,
        None,
    )
    .unwrap();
    let encoding = HtmlBaseEncoding::fresh();
    let installed = dom
        .install_parsed_html_document(
            snapshot.stamp(),
            report,
            DomParsedDocumentEnvironment::headless(
                origin.clone(),
                url.clone(),
                encoding.clone(),
                "windows-1252".into(),
                DomNativeInputPlatform::headless(),
            ),
            DomParsedHtmlInstallLimits::new(128, 10_000_000, 10_000_000, 10_000_000),
            &cancel(),
        )
        .unwrap();
    let source_document = installed.document();
    let done = source(
        &mut dom,
        DomSourceEdit::clone_node(source_document.node(), true),
    );
    let Some(DomSourceOutput::Document(document)) = done.output() else {
        panic!("actual cloned Document")
    };
    let snapshot = dom.snapshot();
    let metadata = snapshot.document(document).unwrap().metadata();
    assert_eq!(metadata.interface(), DomDocumentInterface::Document);
    assert_eq!(metadata.class(), DomDocumentClass::Html);
    assert_eq!(
        metadata.url(),
        &DocumentString::from("https://example.test/source/")
    );
    assert_eq!(metadata.encoding(), &DocumentString::from("windows-1252"));
    assert_eq!(metadata.content_type(), &DocumentString::from("text/html"));
    assert_eq!(metadata.mode(), HtmlDocumentMode::Quirks);
    assert_eq!(metadata.origin(), &origin);
    assert_eq!(
        metadata.creation_cause(),
        &DomDocumentCreationCause::Cloned {
            url: Some(url),
            encoding: Some(encoding)
        }
    );
    assert!(metadata.meta_state().is_some());
    assert_eq!(metadata.initial_permissions_policy(), None);
    detached_document(&snapshot, document, None);
    let html = snapshot.node(document.node()).unwrap().children()[0];
    let body = snapshot.node(html).unwrap().children()[1];
    let p = snapshot.node(body).unwrap().children()[0];
    let text = snapshot.node(p).unwrap().children()[0];
    assert_eq!(
        snapshot.node(text).unwrap().data(),
        Some(&DocumentString::from("source"))
    );
    for node in [html, body, p, text] {
        assert_eq!(
            snapshot.node(node).unwrap().owner_document(),
            Some(document)
        );
    }
}
#[test]
fn clone_exact_node_and_string_admission_and_one_short_are_atomic() {
    for (extra_nodes, extra_units, resource) in [
        (3, 39, None),
        (2, 39, Some(DomResource::ManagedNodes)),
        (3, 38, Some(DomResource::StringCodeUnits)),
    ] {
        let mut dom = dom();
        let mut batch = DomBatch::new();
        let document = batch.create_document();
        let root = batch.create_element(&document, "root");
        let text = batch.create_text(&document, "text");
        batch.insert_before(&root, &document, None);
        batch.insert_before(&text, &root, None);
        let original = apply(&mut dom, batch).document(&document).unwrap();
        let before = dom.snapshot();
        let usage = dom.usage();
        // Literal storage: one Document's 31 units, four name units and four Text units.
        dom.set_limits(bounds(
            usage.managed_nodes() + extra_nodes,
            usage.string_code_units() + extra_units,
        ))
        .unwrap();
        let mut batch = DomBatch::new();
        let clone = batch.clone_node(original, true);
        let result = dom.apply(batch, &cancel());
        if let Some(resource) = resource {
            let limit = if resource == DomResource::ManagedNodes {
                usage.managed_nodes() + extra_nodes
            } else {
                usage.string_code_units() + extra_units
            };
            assert_eq!(
                result.unwrap_err(),
                DomError::ResourceLimit { resource, limit }
            );
            assert_eq!(dom.snapshot().stamp(), before.stamp());
            assert_eq!(dom.usage(), usage);
        } else {
            let document = result.unwrap().document(&clone).unwrap();
            assert_ne!(document, original);
            assert_eq!(dom.usage().managed_nodes(), usage.managed_nodes() + 3);
            assert_eq!(
                dom.usage().string_code_units(),
                usage.string_code_units() + 39
            );
        }
    }
}
