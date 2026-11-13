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

#[test]
fn ordinary_containment_resolves_template_contents_without_crossing_the_host() {
    let mut dom = dom();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let template = batch.create_element_ns(&document, Namespace::html(), "template");
    batch.insert_before(&template, &document, None);
    let commit = apply(&mut dom, batch);
    let template = commit.created(&template).unwrap().unwrap();
    let snapshot = dom.snapshot();
    let contents = snapshot
        .node(template)
        .unwrap()
        .as_template()
        .unwrap()
        .contents()
        .unwrap();
    let inert = snapshot.node(contents).unwrap().owner_document().unwrap();
    let mut batch = DomBatch::new();
    let text = batch.create_text(inert, "template");
    batch.insert_before(&text, contents, None);
    let text = apply(&mut dom, batch).created(&text).unwrap().unwrap();
    let snapshot = dom.snapshot();
    let template = snapshot.node(template).unwrap();
    let contents = snapshot.node(contents).unwrap();
    assert!(
        contents
            .contains_with(text, || Ok::<_, DomError>(()))
            .unwrap()
    );
    assert!(
        !template
            .contains_with(text, || Ok::<_, DomError>(()))
            .unwrap()
    );
    let mut work = 0;
    let refused = contents.contains_with(text, || {
        work += 1;
        Err(DomError::Cancelled)
    });
    assert_eq!((refused, work), (Err(DomError::Cancelled), 1));
    let mut foreign = self::dom();
    let mut batch = DomBatch::new();
    let foreign_document = batch.create_document();
    let foreign_node = apply(&mut foreign, batch)
        .document(&foreign_document)
        .unwrap()
        .node();
    assert_eq!(
        contents.contains_with(foreign_node, || Ok(())),
        Err(DomError::ForeignOwner)
    );
}

#[test]
fn namespace_lookup_stops_at_inert_template_contents_and_preserves_refusal() {
    let mut dom = dom();
    let xmlns = Namespace::from_uri("http://www.w3.org/2000/xmlns/");
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let host = batch.create_element_ns(&document, Namespace::html(), "template");
    batch.set_attribute_ns(&host, xmlns, "xmlns:outside", "urn:outside");
    batch.insert_before(&host, &document, None);
    let commit = apply(&mut dom, batch);
    let host = commit.created(&host).unwrap().unwrap();
    let snapshot = dom.snapshot();
    let contents = snapshot
        .node(host)
        .unwrap()
        .as_template()
        .unwrap()
        .contents()
        .unwrap();
    let inert = snapshot.node(contents).unwrap().owner_document().unwrap();
    let mut batch = DomBatch::new();
    let element = batch.create_element_ns(inert, Namespace::none(), "inner");
    batch.insert_before(&element, contents, None);
    let element = apply(&mut dom, batch).created(&element).unwrap().unwrap();
    let snapshot = dom.snapshot();
    let host = snapshot.node(host).unwrap();
    let prefix = "outside".encode_utf16().collect::<Vec<_>>();
    let mut reached = 0;
    let uri = host
        .lookup_namespace_uri_with(Some(&prefix), || {
            reached += 1;
            Ok::<_, DomError>(())
        })
        .unwrap()
        .unwrap();
    assert!(
        uri.code_units()
            .iter()
            .copied()
            .eq("urn:outside".encode_utf16())
    );
    let inner = snapshot.node(element).unwrap();
    assert!(
        inner
            .lookup_namespace_uri_with(Some(&prefix), || Ok::<_, DomError>(()))
            .unwrap()
            .is_none()
    );
    for refused in 1..=reached {
        let mut attempted = 0;
        let result = host.lookup_namespace_uri_with(Some(&prefix), || {
            attempted += 1;
            if attempted == refused {
                Err(DomError::Cancelled)
            } else {
                Ok(())
            }
        });
        assert!(matches!(result, Err(DomError::Cancelled)));
        assert_eq!(attempted, refused);
    }
}

#[test]
fn structural_equality_admits_retained_sibling_frames_before_traversing() {
    let mut dom = dom();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let roots = ["root", "root"].map(|name| {
        let root = batch.create_element_ns(&document, Namespace::none(), name);
        let mut parent = root.clone();
        for _ in 0..3 {
            let child = batch.create_element_ns(&document, Namespace::none(), "child");
            let sibling = batch.create_text(&document, "sibling");
            batch.insert_before(&child, &parent, None);
            batch.insert_before(&sibling, &parent, None);
            parent = child;
        }
        root
    });
    let commit = apply(&mut dom, batch);
    let [left, right] = roots.map(|root| commit.created(&root).unwrap().unwrap());
    let snapshot = dom.snapshot();
    let node = snapshot.node(left).unwrap();
    let live_bytes = std::cell::Cell::new(0);
    let peak_bytes = std::cell::Cell::new(0);
    struct Scratch<'a> {
        live_bytes: &'a std::cell::Cell<usize>,
        bytes: usize,
    }
    impl Drop for Scratch<'_> {
        fn drop(&mut self) {
            self.live_bytes.set(self.live_bytes.get() - self.bytes);
        }
    }
    let compare = |limit| {
        node.is_equal_node_with(
            right,
            || Ok(()),
            |bytes| {
                let live = live_bytes.get() + bytes;
                if live > limit {
                    return Err(DomError::AllocationFailed);
                }
                live_bytes.set(live);
                peak_bytes.set(peak_bytes.get().max(live));
                Ok(Scratch {
                    live_bytes: &live_bytes,
                    bytes,
                })
            },
        )
    };
    assert_eq!(compare(usize::MAX), Ok(true));
    assert_eq!(live_bytes.get(), 0);
    let peak = peak_bytes.get();
    assert!(peak > 0);
    assert_eq!(compare(peak - 1), Err(DomError::AllocationFailed));
    assert_eq!(live_bytes.get(), 0);
    assert_eq!(compare(peak), Ok(true));
    assert_eq!(live_bytes.get(), 0);
    assert_eq!(dom.snapshot().stamp(), snapshot.stamp());
}

#[test]
fn structural_equality_ignores_template_contents_and_refuses_foreign_nodes() {
    let mut dom = dom();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let left = batch.create_element_ns(&document, Namespace::html(), "template");
    let right = batch.create_element_ns(&document, Namespace::html(), "template");
    let commit = apply(&mut dom, batch);
    let left = commit.created(&left).unwrap().unwrap();
    let right = commit.created(&right).unwrap().unwrap();
    let snapshot = dom.snapshot();
    let contents = snapshot
        .node(left)
        .unwrap()
        .as_template()
        .unwrap()
        .contents()
        .unwrap();
    let inert = snapshot.node(contents).unwrap().owner_document().unwrap();
    let mut batch = DomBatch::new();
    let text = batch.create_text(inert, "inert");
    batch.insert_before(&text, contents, None);
    apply(&mut dom, batch);
    let snapshot = dom.snapshot();
    let node = snapshot.node(left).unwrap();
    let reserve = |_| Err::<(), _>(DomError::AllocationFailed);
    assert_eq!(node.is_equal_node_with(right, || Ok(()), reserve), Ok(true));
    let mut foreign = self::dom();
    let mut batch = DomBatch::new();
    let foreign_document = batch.create_document();
    let foreign = apply(&mut foreign, batch)
        .document(&foreign_document)
        .unwrap()
        .node();
    assert_eq!(
        node.is_equal_node_with(foreign, || Ok(()), reserve),
        Err(DomError::ForeignOwner)
    );
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

#[test]
fn registry_definition_commit_preserves_identity_history_and_hosted_quota() {
    use surgeist_html_semantics::{HtmlCaptureError, HtmlCustomElementDefinition};

    let mut dom = dom();
    let mut issuer = issuer(&mut dom);
    let owner = hosted(&mut dom, &mut issuer);
    let other = hosted(&mut dom, &mut issuer);
    let mut batch = DomBatch::new();
    let existing = batch.create_element(other.document(), "x-item");
    let unregistered = batch.create_element(owner.document(), "x-open");
    let null_host = batch.create_element(other.document(), "div");
    let null_child = batch.create_element(other.document(), "x-item");
    let created = apply(&mut dom, batch);
    let existing = created.created(&existing).unwrap().unwrap();
    let unregistered = created.created(&unregistered).unwrap().unwrap();
    let null_host = created.created(&null_host).unwrap().unwrap();
    let null_child = created.created(&null_child).unwrap().unwrap();
    let mut draft = Draft::new(&dom, DomBatch::new().owner, &cancel()).unwrap();
    let root = draft
        .allocate(
            crate::store::NewNode::ShadowRoot(DomShadowRootState::from_parsed(
                surgeist_html::construction::HtmlShadowRootProperties::new(
                    HtmlShadowRootMode::Open,
                    surgeist_html::construction::HtmlSlotAssignment::Named,
                    false,
                    false,
                    false,
                ),
                true,
                true,
                false,
            )),
            Key::Existing(other.document().node()),
            dom.limits(),
            &cancel(),
        )
        .unwrap();
    draft
        .attach_parsed_shadow(Key::Existing(null_host), root, dom.limits(), &cancel())
        .unwrap();
    draft
        .establish_element_registry(
            Key::Existing(null_child),
            DomRegistryBinding::Null,
            &cancel(),
        )
        .unwrap();
    let prepared = draft
        .finish_with_mapping(&dom, &cancel(), std::iter::once(root))
        .unwrap();
    let root = prepared.1[0];
    dom.publish(prepared.0, &cancel()).unwrap();
    let mut batch = DomBatch::new();
    batch.insert_before(null_child, root, None);
    apply(&mut dom, batch);
    let initial = dom.snapshot();
    let registry = |document| {
        let DomRegistryBinding::Registry(id) = initial
            .document(document)
            .unwrap()
            .metadata()
            .custom_element_registry()
        else {
            panic!("hosted Global has a registry");
        };
        id
    };
    let registry_id = registry(owner.document());
    let other_registry = registry(other.document());
    let original_limits = issuer.limits();
    issuer
        .set_limits(
            DomHostedLimits::new(
                DomHostedRecordLimits::new(32, 32, 32, 32, 32, 32, 2),
                original_limits.retained_bytes(),
                original_limits.scratch_bytes(),
                original_limits.work(),
            ),
            &cancel(),
        )
        .unwrap();

    let autonomous = HtmlCustomElementDefinition::new(
        DocumentString::from("x-item"),
        DocumentString::from("x-item"),
        false,
        true,
    );
    let custom = HtmlCustomElementDefinition::new(
        DocumentString::from("x-button"),
        DocumentString::from("button"),
        true,
        true,
    );
    let retained = issuer.retained_storage_bytes();
    let registry_limits = issuer.limits();
    issuer
        .set_limits(
            DomHostedLimits::new(
                registry_limits.records(),
                registry_limits.retained_bytes(),
                1,
                registry_limits.work(),
            ),
            &cancel(),
        )
        .unwrap();
    assert!(matches!(
        dom.prepare_global_registry_definition(
            &issuer,
            owner.global(),
            registry_id,
            &autonomous,
            source_limits(1),
            &cancel(),
        ),
        Err(DomHostedError::Limit {
            resource: DomHostedResource::ScratchBytes,
            limit: 1
        })
    ));
    assert_eq!(issuer.retained_storage_bytes(), retained);
    issuer.set_limits(registry_limits, &cancel()).unwrap();
    let input_units = autonomous.name().retained_units() + autonomous.local_name().retained_units();
    let canonical_units = dom.custody_registry.ledger().total().units();
    assert!(matches!(
        dom.prepare_global_registry_definition(
            &issuer,
            owner.global(),
            registry_id,
            &autonomous,
            DomSourceLimits::new(
                64_000_000,
                canonical_units + input_units,
                20_000_000,
                1,
                1,
                1,
                1,
            ),
            &cancel(),
        ),
        Err(DomHostedError::Dom(DomError::ResourceLimit {
            resource: DomResource::SourceRetainedUnits,
            ..
        }))
    ));
    assert_eq!(issuer.retained_storage_bytes(), retained);
    assert!(matches!(
        dom.prepare_global_registry_definition(
            &issuer,
            owner.global(),
            other_registry,
            &autonomous,
            source_limits(1),
            &cancel(),
        ),
        Err(DomHostedError::StaleIdentity)
    ));
    assert!(matches!(
        dom.prepare_global_registry_definition(
            &issuer,
            owner.global(),
            registry_id,
            &autonomous,
            source_limits(0),
            &cancel(),
        ),
        Err(DomHostedError::Dom(DomError::SourceOutputLimit))
    ));
    assert!(matches!(
        dom.prepare_global_registry_definition(
            &issuer,
            owner.global(),
            registry_id,
            &autonomous,
            DomSourceLimits::new(64_000_000, 2_000_000, 20_000_000, 1, 1, 1, 0),
            &cancel(),
        ),
        Err(DomHostedError::Dom(DomError::SourceNestedOperationLimit {
            limit: 0
        }))
    ));
    let staged = dom
        .prepare_global_registry_definition(
            &issuer,
            owner.global(),
            registry_id,
            &autonomous,
            source_limits(1),
            &cancel(),
        )
        .unwrap();
    drop(staged);
    assert_eq!(dom.snapshot().stamp(), initial.stamp());
    assert_eq!(issuer.retained_storage_bytes(), retained);

    let first = dom
        .prepare_global_registry_definition(
            &issuer,
            owner.global(),
            registry_id,
            &autonomous,
            source_limits(1),
            &cancel(),
        )
        .unwrap();
    let (changes, first_id) = first.commit();
    assert_eq!(first_id.registry(), registry_id);
    assert_eq!(first_id.ordinal(), 0);
    assert_eq!(
        changes.changes().changes(),
        &[DomChange::RegistryDefinitionAdded(first_id)]
    );
    let intermediate = dom.snapshot();
    assert!(matches!(
        dom.prepare_global_registry_definition(
            &issuer,
            owner.global(),
            registry_id,
            &autonomous,
            source_limits(1),
            &cancel(),
        ),
        Err(DomHostedError::Dom(DomError::SelectCapture(
            HtmlCaptureError::DuplicateDefinitionName
        )))
    ));
    assert_eq!(dom.snapshot().stamp(), intermediate.stamp());
    let second = dom
        .prepare_global_registry_definition(
            &issuer,
            owner.global(),
            registry_id,
            &custom,
            source_limits(1),
            &cancel(),
        )
        .unwrap();
    let (changes, second_id) = second.commit();
    assert_eq!(second_id.registry(), registry_id);
    assert_eq!(second_id.ordinal(), 1);
    assert_eq!(
        changes.changes().changes(),
        &[DomChange::RegistryDefinitionAdded(second_id)]
    );
    let current = dom.snapshot();
    let before = initial.registry(registry_id).unwrap();
    let after_first = intermediate.registry(registry_id).unwrap();
    let after_second = current.registry(registry_id).unwrap();
    assert_eq!(before.definition_count(), 0);
    assert_eq!(after_first.definition_count(), 1);
    assert_eq!(after_second.definition_count(), 2);
    assert_eq!(after_first.definition(first_id).unwrap(), &autonomous);
    assert_eq!(after_second.definition(first_id).unwrap(), &autonomous);
    assert_eq!(after_second.definition(second_id).unwrap(), &custom);
    for (snapshot, expected) in [(&initial, 0), (&intermediate, 1), (&current, 2)] {
        let exported = snapshot
            .html_capture(
                owner.document(),
                None,
                surgeist_html_semantics::HtmlCaptureLimits::new(
                    256, 4096, 1024, 65_536, 256, 16, 16, 1_000_000,
                ),
                DomHtmlCaptureLimits::new(256, 1_000_000),
                &cancel(),
            )
            .unwrap();
        let capture = exported.capture();
        let document = capture
            .node(&exported.subject(owner.document().node()).unwrap())
            .unwrap();
        let surgeist_html_semantics::HtmlCapturedRegistry::Registry(registry) =
            document.lookup_registry()
        else {
            panic!("exported document has its registry");
        };
        assert!(matches!(
            capture.definitions(&registry).unwrap(),
            surgeist_html_semantics::HtmlCapturedDefinitions::Complete(definitions)
                if definitions == &[autonomous.clone(), custom.clone()][..expected]
        ));
    }
    assert!(after_first.definition(second_id).is_err());
    assert!(before.definition(first_id).is_err());
    assert!(
        current
            .registry(other_registry)
            .unwrap()
            .definition(first_id)
            .is_err()
    );

    let mut adoption = DomBatch::new();
    adoption.adopt(existing, owner.document());
    adoption.adopt(null_host, owner.document());
    apply(&mut dom, adoption);
    let adopted = dom.snapshot();
    let DomNodeKind::Element(element) = adopted.node(existing).unwrap().kind() else {
        panic!("adopted element");
    };
    assert_eq!(
        element.custom_element_registry(),
        DomRegistryBinding::Registry(registry_id)
    );
    assert_eq!(
        element.custom_element_definition(),
        DomElementDefinition::Null
    );
    assert_eq!(
        adopted
            .shadow_root(root)
            .unwrap()
            .state()
            .custom_element_registry(),
        DomRegistryBinding::Null
    );
    let DomNodeKind::Element(element) = adopted.node(null_child).unwrap().kind() else {
        panic!("adopted element under declarative root");
    };
    assert_eq!(element.custom_element_registry(), DomRegistryBinding::Null);
    let init = DomShadowRootInit::new(HtmlShadowRootMode::Open);
    let mut refused = DomBatch::new();
    refused.attach_shadow(existing, init);
    assert!(matches!(
        dom.apply(refused, &cancel()),
        Err(DomError::NotSupported)
    ));
    let mut allowed = DomBatch::new();
    allowed.attach_shadow(unregistered, init);
    assert!(dom.apply(allowed, &cancel()).is_ok());
}

#[test]
fn document_factory_reaches_the_selected_definition_before_allocating_an_element() {
    use surgeist_html::values::Namespace;
    use surgeist_html_semantics::HtmlCustomElementDefinition;

    let mut dom = dom();
    let mut issuer = issuer(&mut dom);
    let owner = hosted(&mut dom, &mut issuer);
    let document = owner.document();
    let registry = {
        let snapshot = dom.snapshot();
        let DomNodeKind::Document(metadata) = snapshot.node(document.node()).unwrap().kind() else {
            panic!("hosted document");
        };
        let DomRegistryBinding::Registry(registry) = metadata.custom_element_registry() else {
            panic!("hosted document registry");
        };
        registry
    };
    let mut define = |name: &str, local: &str| {
        dom.prepare_global_registry_definition(
            &issuer,
            owner.global(),
            registry,
            &HtmlCustomElementDefinition::new(name.into(), local.into(), false, false),
            source_limits(1),
            &cancel(),
        )
        .unwrap()
        .commit()
        .1
    };
    let autonomous = define("x-widget", "x-widget");
    let customized = define("x-button", "button");
    let before = dom.snapshot().stamp();
    let watch_limits = DomNodeRetirementLimits::new(16_000_000, 1000);
    let inbox = dom.node_retirement_inbox(watch_limits, &cancel()).unwrap();
    let DomWatchedElementCreation::Constructor(watched_request) = dom
        .create_detached_element_with_node_retirement_watch(
            document,
            (
                DomElementCreationName::Local("x-widget".into()),
                DomElementCreationOptions::default(),
            ),
            (&inbox, 0),
            (source_limits(4096), watch_limits),
            &cancel(),
            |_| -> Result<(), DomError> { panic!("private wrapper was never prepared") },
        )
        .unwrap()
    else {
        panic!("watched constructor request");
    };
    assert_eq!(watched_request.definition(), autonomous);
    assert_eq!(
        dom.pending_source_edit().unwrap().operation(),
        watched_request.operation()
    );
    assert_eq!(dom.snapshot().stamp(), before);
    assert!(dom.node_retirements.is_none());
    dom.discard_unstarted_custom_element(watched_request)
        .unwrap();
    assert!(dom.pending_source_edit().is_none());
    assert_eq!(dom.snapshot().stamp(), before);

    for (name, options, definition, local, prefix) in [
        (
            DomElementCreationName::Qualified(Namespace::html(), "p:x-widget".into()),
            DomElementCreationOptions::default(),
            autonomous,
            "x-widget",
            Some("p"),
        ),
        (
            DomElementCreationName::Local("BUTTON".into()),
            DomElementCreationOptions {
                is: Some("x-button".into()),
                registry: None,
            },
            customized,
            "button",
            None,
        ),
    ] {
        let mut lease = dom
            .begin_source_edit(
                DomSourceEdit::create_element_with_options(document.node(), name, options),
                source_limits(4096),
                &cancel(),
            )
            .unwrap();
        let DomSourceProgress::Request(DomSourceRequest::CustomElementConstruction(request)) =
            lease.advance(&cancel()).unwrap()
        else {
            panic!("the selected definition must be serviced synchronously");
        };
        assert_eq!(request.operation(), lease.operation());
        assert_eq!(request.definition(), definition);
        assert_eq!(request.document(), document);
        assert_eq!(request.global(), owner.global());
        let name = lease
            .custom_element_creation_name(request, &cancel())
            .unwrap();
        assert_eq!(name.local_name(), &DocumentString::from(local));
        assert_eq!(name.prefix(), prefix.map(DocumentString::from).as_ref());
        assert!(!lease.status().has_published());
        assert_eq!(lease.snapshot().stamp(), before);
        drop(lease);
        let mut lease = dom
            .resume_source_edit(request.operation(), source_limits(4096), &cancel())
            .unwrap();
        let work = lease.status().spent_work();
        assert!(matches!(
            lease.advance(&cancel()).unwrap(),
            DomSourceProgress::Request(DomSourceRequest::CustomElementConstruction(same))
                if same == request
        ));
        assert_eq!(lease.status().spent_work(), work);
        lease.request_cancel();
        assert!(matches!(
            lease.advance(&cancel()).unwrap(),
            DomSourceProgress::Complete(done) if done.cancellation_requested()
        ));
        drop(lease);
        assert_eq!(dom.snapshot().stamp(), before);
    }

    for (name, options) in [
        (
            DomElementCreationName::Local("x-widget".into()),
            DomElementCreationOptions {
                is: None,
                registry: Some(DomRegistryBinding::Null),
            },
        ),
        (
            DomElementCreationName::Qualified(Namespace::svg(), "x-widget".into()),
            DomElementCreationOptions::default(),
        ),
    ] {
        let result = source(
            &mut dom,
            DomSourceEdit::create_element_with_options(document.node(), name, options),
        );
        let Some(DomSourceOutput::Node(node)) = result.output() else {
            panic!("ordinary detached element");
        };
        assert_eq!(
            dom.snapshot()
                .node(node)
                .unwrap()
                .as_element()
                .unwrap()
                .custom_element_definition(),
            DomElementDefinition::Null
        );
    }
}

#[test]
fn definition_candidates_include_closed_shadow_in_tree_order() {
    use surgeist_html::construction::HtmlShadowRootMode;
    use surgeist_html::values::Namespace;
    use surgeist_html_semantics::HtmlCustomElementDefinition;

    let mut dom = dom();
    let mut issuer = issuer(&mut dom);
    let owner = hosted(&mut dom, &mut issuer);
    let document = owner.document();
    let snapshot = dom.snapshot();
    let document_view = snapshot.document(document).unwrap();
    let DomRegistryBinding::Registry(registry) = document_view.metadata().custom_element_registry()
    else {
        panic!("hosted registry");
    };
    let root = document_view.document_element().unwrap();
    let mut batch = DomBatch::new();
    let host = batch.create_element_ns(document, Namespace::html(), "div");
    let shadow = batch.attach_shadow(&host, DomShadowRootInit::new(HtmlShadowRootMode::Closed));
    let inside = batch.create_element_ns(document, Namespace::html(), "x-upgrade");
    let light = batch.create_element_ns(document, Namespace::html(), "x-upgrade");
    let outside = batch.create_element_ns(document, Namespace::html(), "x-upgrade");
    let _detached = batch.create_element_ns(document, Namespace::html(), "x-upgrade");
    let other_namespace = batch.create_element_ns(document, Namespace::svg(), "x-upgrade");
    batch.insert_before(&inside, &shadow, None);
    batch.insert_before(&light, &host, None);
    batch.insert_before(&host, root, None);
    batch.insert_before(&other_namespace, root, None);
    batch.insert_before(&outside, root, None);
    let commit = dom.apply(batch, &cancel()).unwrap();
    let expected = [&inside, &light, &outside].map(|node| commit.created(node).unwrap().unwrap());
    let snapshot = dom.snapshot();
    let root = snapshot.node(document.node()).unwrap();
    let definition =
        HtmlCustomElementDefinition::new("x-upgrade".into(), "x-upgrade".into(), false, false);
    let work = std::cell::Cell::new(0usize);
    let mut actual = Vec::new();
    root.visit_custom_element_upgrade_candidates_with(
        (registry, &definition),
        || {
            work.set(work.get() + 1);
            Ok::<_, DomError>(())
        },
        |_| Ok(()),
        |node| {
            actual.push(node);
            Ok(())
        },
    )
    .unwrap();
    assert_eq!(actual, expected);
    let mut visits = 0;
    assert!(matches!(
        root.visit_custom_element_upgrade_candidates_with(
            (registry, &definition),
            || Ok(()),
            |_| Err::<(), _>(DomError::AllocationFailed),
            |_| {
                visits += 1;
                Ok(())
            },
        ),
        Err(DomError::AllocationFailed)
    ));
    assert_eq!(visits, 0);
    let remaining = std::cell::Cell::new(work.get() - 1);
    assert!(matches!(
        root.visit_custom_element_upgrade_candidates_with(
            (registry, &definition),
            || {
                let next = remaining.get().checked_sub(1).ok_or(DomError::Cancelled)?;
                remaining.set(next);
                Ok(())
            },
            |_| Ok(()),
            |_| Ok(()),
        ),
        Err(DomError::Cancelled)
    ));
    let mut buttons = Vec::new();
    for saved in [Some("x-button"), Some("x-other"), None] {
        let created = source(
            &mut dom,
            DomSourceEdit::create_element_with_options(
                document.node(),
                DomElementCreationName::Local("button".into()),
                DomElementCreationOptions {
                    is: saved.map(Into::into),
                    registry: None,
                },
            ),
        );
        let Some(DomSourceOutput::Node(button)) = created.output() else {
            panic!("button");
        };
        buttons.push(button);
    }
    let parent = dom
        .snapshot()
        .document(document)
        .unwrap()
        .document_element()
        .unwrap();
    let mut connect = DomBatch::new();
    for button in &buttons {
        connect.insert_before(*button, parent, None);
    }
    dom.apply(connect, &cancel()).unwrap();
    let snapshot = dom.snapshot();
    let customized =
        HtmlCustomElementDefinition::new("x-button".into(), "button".into(), false, false);
    let mut matching = Vec::new();
    snapshot
        .node(document.node())
        .unwrap()
        .visit_custom_element_upgrade_candidates_with(
            (registry, &customized),
            || Ok::<_, DomError>(()),
            |_| Ok(()),
            |node| {
                matching.push(node);
                Ok(())
            },
        )
        .unwrap();
    assert_eq!(matching, [buttons[0]]);
}

#[test]
fn insertion_reaction_receipt_preserves_shadow_order_and_refused_output() {
    use surgeist_html::scripting::{HtmlScriptingEngine, HtmlScriptingPolicy};
    use surgeist_html_semantics::HtmlCustomElementDefinition;

    let mut dom = dom();
    let mut issuer = issuer(&mut dom);
    let owner = hosted(&mut dom, &mut issuer);
    let document = owner.document();
    let snapshot = dom.snapshot();
    let document_view = snapshot.document(document).unwrap();
    let root = document_view.document_element().unwrap();
    let DomRegistryBinding::Registry(registry) = document_view.metadata().custom_element_registry()
    else {
        panic!("hosted registry");
    };
    let mut batch = DomBatch::new();
    let host = batch.create_element_ns(document, Namespace::html(), "div");
    let shadow = batch.attach_shadow(&host, DomShadowRootInit::new(HtmlShadowRootMode::Closed));
    let inside = batch.create_element_ns(document, Namespace::html(), "x-insertion");
    let outside = batch.create_element_ns(document, Namespace::html(), "x-insertion");
    let later = batch.create_element_ns(document, Namespace::html(), "x-insertion");
    batch.insert_before(&inside, &shadow, None);
    batch.insert_before(&outside, &host, None);
    let committed = dom.apply(batch, &cancel()).unwrap();
    let host = committed.created(&host).unwrap().unwrap();
    let inside = committed.created(&inside).unwrap().unwrap();
    let outside = committed.created(&outside).unwrap().unwrap();
    let later = committed.created(&later).unwrap().unwrap();
    let definition = dom
        .prepare_global_registry_definition(
            &issuer,
            owner.global(),
            registry,
            &HtmlCustomElementDefinition::new(
                "x-insertion".into(),
                "x-insertion".into(),
                false,
                false,
            ),
            source_limits(1),
            &cancel(),
        )
        .unwrap()
        .commit()
        .1;
    let backing = issuer
        .retain_runtime_payload(
            &mut dom,
            DomDocumentScriptRegistration::storage_bytes(),
            &cancel(),
        )
        .unwrap();
    let (binding, registration) = issuer
        .attach_document_script_host(
            &mut dom,
            owner.global(),
            document,
            DomDocumentScriptingFacts {
                engine: HtmlScriptingEngine::Supported,
                policy: HtmlScriptingPolicy {
                    user_allows: true,
                    automation_allows: true,
                },
            },
            backing,
            &cancel(),
        )
        .unwrap();
    registration.observe_custom_element_reactions().unwrap();
    let stamp = dom.snapshot().stamp();
    let mut lease = dom
        .begin_source_edit(
            DomSourceEdit::insert_before(host, root, None),
            source_limits(1),
            &cancel(),
        )
        .unwrap();
    assert!(matches!(
        lease.advance(&cancel()),
        Err(DomError::SourceOutputLimit)
    ));
    drop(lease);
    assert_eq!(dom.snapshot().stamp(), stamp);
    assert_eq!(registration.custom_element_reaction_segments(), 0);
    let done = source(&mut dom, DomSourceEdit::insert_before(host, root, None));
    let id = done.operation();
    assert!(matches!(
        registration.take_custom_element_reactions(id, 0),
        Err(DomError::SourceOutputLimit)
    ));
    let mut loan = registration
        .take_custom_element_reactions(id, 1)
        .unwrap()
        .unwrap();
    assert_eq!(
        loan.facts().collect::<Vec<_>>(),
        [
            crate::DomCustomElementReaction {
                element: inside,
                definition,
                kind: crate::DomCustomElementReactionKind::Upgrade,
            },
            crate::DomCustomElementReaction {
                element: outside,
                definition,
                kind: crate::DomCustomElementReactionKind::Upgrade,
            },
        ]
    );
    assert!(
        registration
            .take_custom_element_reactions(id, 1)
            .unwrap()
            .is_none()
    );
    assert!(dom.has_pending_custom_element_reactions(&binding));
    let later_operation =
        source(&mut dom, DomSourceEdit::insert_before(later, root, None)).operation();
    let mut later_loan = registration
        .take_custom_element_reactions(later_operation, 2)
        .unwrap()
        .unwrap();
    assert_eq!(
        later_loan.facts().collect::<Vec<_>>(),
        [crate::DomCustomElementReaction {
            element: later,
            definition,
            kind: crate::DomCustomElementReactionKind::Upgrade,
        }]
    );
    later_loan.acknowledge_front();
    drop(later_loan);
    loan.acknowledge_front();
    drop(loan);
    let loan = registration
        .take_custom_element_reactions(id, 2)
        .unwrap()
        .unwrap();
    assert_eq!(
        loan.facts().collect::<Vec<_>>(),
        [crate::DomCustomElementReaction {
            element: outside,
            definition,
            kind: crate::DomCustomElementReactionKind::Upgrade,
        }]
    );
    registration.revoke();
    assert!(matches!(
        registration.take_custom_element_reactions(id, 2),
        Err(DomError::HostedSourceUnavailable)
    ));
    assert!(
        dom.discard_stopped_custom_element_reactions(&binding)
            .is_err()
    );
    dom.release_document_script_registration(&binding);
    assert!(dom.has_pending_custom_element_reactions(&binding));
    drop(loan);
    dom.release_document_script_registration(&binding);
    assert!(dom.has_pending_custom_element_reactions(&binding));
    let replacement = issuer
        .retain_runtime_payload(
            &mut dom,
            DomDocumentScriptRegistration::storage_bytes(),
            &cancel(),
        )
        .unwrap();
    assert!(matches!(
        issuer.attach_document_script_host(
            &mut dom,
            owner.global(),
            document,
            DomDocumentScriptingFacts {
                engine: HtmlScriptingEngine::Supported,
                policy: HtmlScriptingPolicy {
                    user_allows: true,
                    automation_allows: true,
                },
            },
            replacement,
            &cancel(),
        ),
        Err(DomHostedError::WrongPhase)
    ));
    assert_eq!(
        dom.discard_stopped_custom_element_reactions(&binding)
            .unwrap(),
        1
    );
    assert!(!dom.has_pending_custom_element_reactions(&binding));
    dom.release_document_script_registration(&binding);
    let backing = issuer
        .retain_runtime_payload(
            &mut dom,
            DomDocumentScriptRegistration::storage_bytes(),
            &cancel(),
        )
        .unwrap();
    let (_, replacement) = issuer
        .attach_document_script_host(
            &mut dom,
            owner.global(),
            document,
            DomDocumentScriptingFacts {
                engine: HtmlScriptingEngine::Supported,
                policy: HtmlScriptingPolicy {
                    user_allows: true,
                    automation_allows: true,
                },
            },
            backing,
            &cancel(),
        )
        .unwrap();
    assert!(replacement.is_active());
    assert_eq!(replacement.custom_element_reaction_segments(), 0);
}

#[test]
fn customized_factory_publishes_one_receiver_and_retries_its_closing_phase() {
    use surgeist_html::{
        elements::native::HtmlCustomElementState,
        scripting::{HtmlScriptingEngine, HtmlScriptingPolicy},
    };
    use surgeist_html_semantics::HtmlCustomElementDefinition;

    let mut dom = Dom::new(bounds(1024, 1_000_000)).unwrap();
    let mut issuer = issuer(&mut dom);
    let owner = hosted(&mut dom, &mut issuer);
    let document = owner.document();
    let before = dom.snapshot();
    let DomRegistryBinding::Registry(registry) = before
        .document(document)
        .unwrap()
        .metadata()
        .custom_element_registry()
    else {
        panic!("hosted registry");
    };
    let definition = dom
        .prepare_global_registry_definition(
            &issuer,
            owner.global(),
            registry,
            &HtmlCustomElementDefinition::new("x-button".into(), "button".into(), false, false),
            source_limits(1),
            &cancel(),
        )
        .unwrap()
        .commit()
        .1;
    let backing = issuer
        .retain_runtime_payload(
            &mut dom,
            DomDocumentScriptRegistration::storage_bytes(),
            &cancel(),
        )
        .unwrap();
    let (_, registration) = issuer
        .attach_document_script_host(
            &mut dom,
            owner.global(),
            document,
            DomDocumentScriptingFacts {
                engine: HtmlScriptingEngine::Supported,
                policy: HtmlScriptingPolicy {
                    user_allows: true,
                    automation_allows: true,
                },
            },
            backing,
            &cancel(),
        )
        .unwrap();
    let bounds = source_limits(4096);
    let watch_limits = DomNodeRetirementLimits::new(16_000_000, 1000);
    let inbox = dom.node_retirement_inbox(watch_limits, &cancel()).unwrap();
    let mut lease = dom
        .begin_source_edit(
            DomSourceEdit::create_element_with_options(
                document.node(),
                DomElementCreationName::Local("button".into()),
                DomElementCreationOptions {
                    is: Some("x-button".into()),
                    registry: None,
                },
            ),
            bounds,
            &cancel(),
        )
        .unwrap();
    let DomSourceProgress::Request(DomSourceRequest::CustomElementConstruction(request)) =
        lease.advance(&cancel()).unwrap()
    else {
        panic!("customized constructor request");
    };
    assert_eq!(request.definition(), definition);
    let original = lease.snapshot().stamp();
    drop(lease);
    let mut lease = dom
        .resume_source_edit(
            request.operation(),
            DomSourceLimits {
                outputs: 3,
                ..bounds
            },
            &cancel(),
        )
        .unwrap();
    assert!(matches!(
        lease.open_customized_element_callback_with_watch(
            request,
            &registration,
            (&inbox, 0),
            watch_limits,
            &cancel(),
            |_| -> Result<(), DomError> {
                panic!("closing capacity must be admitted before wrapper formation")
            }
        ),
        Err(DomError::SourceOutputLimit)
    ));
    assert_eq!(lease.snapshot().stamp(), original);
    drop(lease);
    let mut lease = dom
        .resume_source_edit(
            request.operation(),
            DomSourceLimits {
                outputs: 4,
                ..bounds
            },
            &cancel(),
        )
        .unwrap();
    assert!(matches!(
        lease.open_customized_element_callback_with_watch(
            request,
            &registration,
            (&inbox, 0),
            watch_limits,
            &cancel(),
            |_| Err::<(), _>(DomError::AllocationFailed)
        ),
        Err(DomError::AllocationFailed)
    ));
    assert_eq!(lease.snapshot().stamp(), original);
    let unwind = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        lease.open_customized_element_callback_with_watch(
            request,
            &registration,
            (&inbox, 0),
            watch_limits,
            &cancel(),
            |_| -> Result<(), DomError> { panic!("wrapper preparation unwind") },
        )
    }));
    assert!(unwind.is_err());
    assert_eq!(lease.snapshot().stamp(), original);
    let (scope, watched, name) = lease
        .open_customized_element_callback_with_watch(
            request,
            &registration,
            (&inbox, 0),
            watch_limits,
            &cancel(),
            |name| Ok::<_, DomError>(name.local_name().clone()),
        )
        .unwrap();
    assert_eq!(name, "button".into());
    let receiver = watched.node();
    assert!(lease.status().has_published());
    let published = lease.snapshot();
    let element = published.node(receiver).unwrap().as_element().unwrap();
    assert_eq!(
        element.custom_element_state(),
        HtmlCustomElementState::Precustomized
    );
    assert_eq!(
        element.custom_element_definition(),
        DomElementDefinition::Definition(definition)
    );
    assert_eq!(element.saved_is(), Some(&"x-button".into()));
    drop(scope);
    let mut outcome = Some(DomSourceCustomElementOutcome::Failed);
    lease
        .accept_custom_element_callback(request, &registration, &mut outcome, &cancel())
        .unwrap();
    assert_eq!(outcome, None);
    assert!(matches!(
        lease.finish_custom_element_with_watch(
            request,
            (&inbox, 1),
            watch_limits,
            &cancel(),
            |_| -> Result<(), DomError> { panic!("existing watch must be reused") }
        ),
        Err(DomError::InvalidSourceOperation)
    ));
    let interrupted = AtomicBool::new(true);
    assert!(matches!(
        lease.advance(&interrupted),
        Err(DomError::Cancelled)
    ));
    assert_eq!(lease.snapshot().stamp(), published.stamp());
    let DomSourceProgress::Complete(done) = lease.advance(&cancel()).unwrap() else {
        panic!("original receiver must settle");
    };
    assert_eq!(done.output(), Some(DomSourceOutput::Node(receiver)));
    assert_eq!(
        done.custom_element_disposition(),
        Some(DomSourceCustomElementDisposition::FailedReceiverAbrupt)
    );
    let settled = lease.snapshot();
    let element = settled.node(receiver).unwrap().as_element().unwrap();
    assert_eq!(
        element.custom_element_state(),
        HtmlCustomElementState::Failed
    );
    assert_eq!(
        element.custom_element_definition(),
        DomElementDefinition::Null
    );
    assert_eq!(element.saved_is(), Some(&"x-button".into()));
    assert!(!element.has_failed_construction_interface());
    drop(lease);
    let mut lease = dom
        .begin_source_edit(
            DomSourceEdit::create_element_with_options(
                document.node(),
                DomElementCreationName::Local("button".into()),
                DomElementCreationOptions {
                    is: Some("x-button".into()),
                    registry: None,
                },
            ),
            bounds,
            &cancel(),
        )
        .unwrap();
    let DomSourceProgress::Request(DomSourceRequest::CustomElementConstruction(request)) =
        lease.advance(&cancel()).unwrap()
    else {
        panic!("customized constructor request");
    };
    let (scope, watched, ()) = lease
        .open_customized_element_callback_with_watch(
            request,
            &registration,
            (&inbox, 2),
            watch_limits,
            &cancel(),
            |_| Ok::<_, DomError>(()),
        )
        .unwrap();
    let retired = watched.node();
    drop(lease);
    let mut nested = dom
        .begin_source_edit(DomSourceEdit::destroy(retired), bounds, &cancel())
        .unwrap();
    assert!(matches!(
        nested.advance(&cancel()).unwrap(),
        DomSourceProgress::Complete(_)
    ));
    drop(nested);
    drop(scope);
    let mut lease = dom
        .resume_source_edit(request.operation(), bounds, &cancel())
        .unwrap();
    let mut outcome = Some(DomSourceCustomElementOutcome::Returned(retired));
    lease
        .accept_custom_element_callback(request, &registration, &mut outcome, &cancel())
        .unwrap();
    let DomSourceProgress::Complete(done) = lease.advance(&cancel()).unwrap() else {
        panic!("retired receiver must not strand the Source");
    };
    assert_eq!(done.published_node(), Some(retired));
    assert_eq!(done.output(), None);
    drop(lease);
    assert!(dom.pending_source_edit().is_none());
}

#[test]
fn autonomous_factory_retains_a_single_completion_across_native_reentry_and_refusal() {
    use surgeist_html::{
        scripting::{HtmlScriptingEngine, HtmlScriptingPolicy},
        values::Namespace,
    };
    use surgeist_html_semantics::HtmlCustomElementDefinition;

    let mut dom = Dom::new(
        bounds(1024, 1_000_000).with_dispatch_custody(DomDispatchCustodyLimits::default()),
    )
    .unwrap();
    let mut issuer = issuer(&mut dom);
    let owner = hosted(&mut dom, &mut issuer);
    let document = owner.document();
    let registry = {
        let snapshot = dom.snapshot();
        let DomNodeKind::Document(metadata) = snapshot.node(document.node()).unwrap().kind() else {
            panic!("hosted document");
        };
        let DomRegistryBinding::Registry(registry) = metadata.custom_element_registry() else {
            panic!("hosted document registry");
        };
        registry
    };
    dom.prepare_global_registry_definition(
        &issuer,
        owner.global(),
        registry,
        &HtmlCustomElementDefinition::new("x-broken".into(), "x-broken".into(), false, false),
        source_limits(1),
        &cancel(),
    )
    .unwrap()
    .commit();
    let backing = issuer
        .retain_runtime_payload(
            &mut dom,
            DomDocumentScriptRegistration::storage_bytes(),
            &cancel(),
        )
        .unwrap();
    let (_, registration) = issuer
        .attach_document_script_host(
            &mut dom,
            owner.global(),
            document,
            DomDocumentScriptingFacts {
                engine: HtmlScriptingEngine::Supported,
                policy: HtmlScriptingPolicy {
                    user_allows: true,
                    automation_allows: true,
                },
            },
            backing,
            &cancel(),
        )
        .unwrap();
    let bounds = source_limits(4096);
    let watch_limits = DomNodeRetirementLimits::new(16_000_000, 1000);
    let mut inbox = dom.node_retirement_inbox(watch_limits, &cancel()).unwrap();
    let mut lease = dom
        .begin_source_edit(
            DomSourceEdit::create_element_ns(document.node(), Namespace::html(), "p:x-broken"),
            bounds,
            &cancel(),
        )
        .unwrap();
    let DomSourceProgress::Request(DomSourceRequest::CustomElementConstruction(request)) =
        lease.advance(&cancel()).unwrap()
    else {
        panic!("selected constructor");
    };
    let scope = lease
        .open_custom_element_callback(request, &registration, &cancel())
        .unwrap();
    assert!(matches!(
        lease.open_custom_element_callback(request, &registration, &cancel()),
        Err(DomError::InvalidSourceOperation)
    ));
    let mut result = Some(DomSourceCustomElementOutcome::Failed);
    assert_eq!(
        lease.accept_custom_element_callback(request, &registration, &mut result, &cancel()),
        Err(DomError::InvalidSourceOperation)
    );
    drop(lease);

    let before = dom.snapshot().stamp();
    assert!(matches!(
        dom.construct_autonomous_element_with_watch(
            (request, request),
            &registration,
            owner.global(),
            (&inbox, 0),
            DomNodeRetirementLimits::new(16_000_000, 0),
            &cancel(),
        ),
        Err(DomError::NodeRetirementWorkLimit { limit: 0 })
    ));
    assert_eq!(dom.snapshot().stamp(), before);
    assert!(dom.node_retirements.is_none());
    let created = dom
        .construct_autonomous_element_with_watch(
            (request, request),
            &registration,
            owner.global(),
            (&inbox, 0),
            watch_limits,
            &cancel(),
        )
        .unwrap();
    let constructed = created.node();
    let snapshot = dom.snapshot();
    let element = snapshot.node(constructed).unwrap().as_element().unwrap();
    assert_eq!(element.name().prefix(), None);
    assert_eq!(element.saved_is(), None);
    assert_eq!(
        element.custom_element_definition(),
        DomElementDefinition::Definition(request.definition())
    );
    assert_eq!(
        element.custom_element_state(),
        surgeist_html::elements::native::HtmlCustomElementState::Custom
    );

    let child = source(
        &mut dom,
        DomSourceEdit::create_element(document.node(), "span"),
    );
    let Some(DomSourceOutput::Node(child)) = child.output() else {
        panic!("nested detached element");
    };
    assert_eq!(
        dom.pending_source_edit().unwrap().operation(),
        request.operation()
    );
    drop(scope);

    assert!(matches!(
        dom.construct_autonomous_element_with_watch(
            (request, request),
            &registration,
            owner.global(),
            (&inbox, 1),
            watch_limits,
            &cancel(),
        ),
        Err(DomError::InvalidSourceOperation)
    ));
    let mut lease = dom
        .resume_source_edit(request.operation(), bounds, &cancel())
        .unwrap();
    assert_eq!(
        lease.accept_custom_element_callback(
            request,
            &registration,
            &mut result,
            &AtomicBool::new(true),
        ),
        Err(DomError::Cancelled)
    );
    assert_eq!(result, Some(DomSourceCustomElementOutcome::Failed));
    lease
        .accept_custom_element_callback(request, &registration, &mut result, &cancel())
        .unwrap();
    assert_eq!(result, None);
    lease.request_cancel();
    drop(lease);
    let short = DomSourceLimits {
        outputs: 0,
        ..bounds
    };
    let mut lease = dom
        .resume_source_edit(request.operation(), short, &cancel())
        .unwrap();
    let before_close = lease.snapshot().stamp();
    assert!(matches!(
        lease.advance(&cancel()),
        Err(DomError::SourceOutputLimit)
    ));
    assert_eq!(lease.snapshot().stamp(), before_close);
    drop(lease);
    let mut lease = dom
        .resume_source_edit(request.operation(), bounds, &cancel())
        .unwrap();
    let before_watch = lease.snapshot().stamp();
    assert_eq!(
        lease
            .finish_returned_custom_element(request, &cancel())
            .unwrap(),
        None
    );
    assert_eq!(lease.snapshot().stamp(), before_watch);
    assert_eq!(lease.status().operation(), request.operation());
    assert!(matches!(
        lease.finish_custom_element_with_watch(
            request,
            (&inbox, 1),
            watch_limits,
            &cancel(),
            |_| Err::<(), _>(DomError::NotSupported),
        ),
        Err(DomError::NotSupported)
    ));
    assert_eq!(lease.snapshot().stamp(), before_watch);
    let DomWatchedCustomElementResult::Fallback {
        node: fallback_node,
        watch: _fallback_watch,
        projection,
        disposition,
    } = lease
        .finish_custom_element_with_watch(request, (&inbox, 1), watch_limits, &cancel(), |name| {
            Ok::<_, DomError>(name.name().prefix().cloned())
        })
        .unwrap()
    else {
        panic!("failed factory returns its watched fallback");
    };
    assert_eq!(disposition, DomSourceCustomElementDisposition::Abrupt);
    assert_eq!(projection, Some("p".into()));
    assert_ne!(child, fallback_node);
    assert_ne!(constructed, fallback_node);
    let snapshot = lease.snapshot();
    assert!(snapshot.node(child).is_ok());
    assert!(snapshot.node(constructed).is_ok());
    let element = snapshot.node(fallback_node).unwrap().as_element().unwrap();
    assert!(element.has_failed_construction_interface());
    assert_eq!(element.name().prefix(), Some(&"p".into()));
    assert_eq!(element.name().local_name(), &"x-broken".into());
    assert_eq!(
        element.custom_element_state(),
        surgeist_html::elements::native::HtmlCustomElementState::Failed
    );
    assert_eq!(
        element.custom_element_definition(),
        DomElementDefinition::Null
    );
    assert_eq!(
        element.custom_element_registry(),
        DomRegistryBinding::Registry(registry)
    );
    drop(lease);
    let mut retire = DomBatch::new();
    retire.destroy(constructed);
    retire.destroy(fallback_node);
    dom.apply(retire, &cancel()).unwrap();
    let mut notices = [inbox.take().unwrap(), inbox.take().unwrap()];
    notices.sort_by_key(|notice| notice.custody_key());
    assert_eq!(
        (notices[0].node(), notices[0].custody_key()),
        (constructed, 0)
    );
    assert_eq!(
        (notices[1].node(), notices[1].custody_key()),
        (fallback_node, 1)
    );
    assert!(inbox.take().is_none());
    drop(created);

    // A constructor may return a different eligible native element. The
    // factory must preserve its identity and apply the requested prefix.
    let mut lease = dom
        .begin_source_edit(
            DomSourceEdit::create_element_ns(document.node(), Namespace::html(), "q:x-broken"),
            bounds,
            &cancel(),
        )
        .unwrap();
    let DomSourceProgress::Request(DomSourceRequest::CustomElementConstruction(request)) =
        lease.advance(&cancel()).unwrap()
    else {
        panic!("selected constructor");
    };
    let scope = lease
        .open_custom_element_callback(request, &registration, &cancel())
        .unwrap();
    drop(lease);
    dom.source_custom_element_callback_step(request, &registration, &cancel())
        .unwrap();
    let returned = source(
        &mut dom,
        DomSourceEdit::create_element_with_options(
            document.node(),
            DomElementCreationName::Local("x-broken".into()),
            DomElementCreationOptions {
                is: None,
                registry: Some(DomRegistryBinding::Null),
            },
        ),
    );
    let Some(DomSourceOutput::Node(returned)) = returned.output() else {
        panic!("nested eligible element");
    };
    drop(scope);
    let units = dom.usage().string_code_units();
    let mut lease = dom
        .resume_source_edit(request.operation(), bounds, &cancel())
        .unwrap();
    let mut outcome = Some(DomSourceCustomElementOutcome::Returned(returned));
    lease
        .accept_custom_element_callback(request, &registration, &mut outcome, &cancel())
        .unwrap();
    assert_eq!(
        lease
            .finish_returned_custom_element(request, &cancel())
            .unwrap(),
        Some(returned)
    );
    let snapshot = lease.snapshot();
    let element = snapshot.node(returned).unwrap().as_element().unwrap();
    assert!(!element.has_failed_construction_interface());
    assert_eq!(element.name().prefix(), Some(&"q".into()));
    assert_eq!(
        element.custom_element_registry(),
        DomRegistryBinding::Registry(registry)
    );
    drop(lease);
    assert_eq!(dom.usage().string_code_units(), units + 1);
}

#[test]
fn adopted_custom_element_exports_the_definition_from_its_original_registry() {
    use surgeist_html::elements::native::HtmlCustomElementState;
    use surgeist_html_semantics::{
        HtmlCaptureLimits, HtmlCapturedCustomElementDefinition, HtmlCapturedNodeKind,
        HtmlCapturedRegistry, HtmlCustomElementDefinition,
    };

    let mut dom = dom();
    let mut issuer = issuer(&mut dom);
    let original = hosted(&mut dom, &mut issuer);
    let destination = hosted(&mut dom, &mut issuer);
    let mut batch = DomBatch::new();
    let ticket = batch.create_element(original.document(), "x-widget");
    let node = apply(&mut dom, batch).created(&ticket).unwrap().unwrap();
    let initial = dom.snapshot();
    let registry_of = |document| {
        let DomRegistryBinding::Registry(id) = initial
            .document(document)
            .unwrap()
            .metadata()
            .custom_element_registry()
        else {
            panic!("hosted document registry");
        };
        id
    };
    let source_registry = registry_of(original.document());
    let destination_registry = registry_of(destination.document());
    let other = HtmlCustomElementDefinition::new("x-other".into(), "x-other".into(), false, false);
    dom.prepare_global_registry_definition(
        &issuer,
        original.global(),
        source_registry,
        &other,
        source_limits(1),
        &cancel(),
    )
    .unwrap()
    .commit();
    let competing =
        HtmlCustomElementDefinition::new("x-widget".into(), "x-widget".into(), false, true);
    let (_, wrong) = dom
        .prepare_global_registry_definition(
            &issuer,
            destination.global(),
            destination_registry,
            &competing,
            source_limits(1),
            &cancel(),
        )
        .unwrap()
        .commit();
    let facts =
        HtmlCustomElementDefinition::new("x-widget".into(), "x-widget".into(), false, false);
    let (_, definition) = dom
        .prepare_global_registry_definition(
            &issuer,
            original.global(),
            source_registry,
            &facts,
            source_limits(1),
            &cancel(),
        )
        .unwrap()
        .commit();

    // This fixture starts at the native state publication boundary after a
    // successful constructor. No guest completion is fabricated by the API.
    let mut draft = Draft::new(&dom, DomBatch::new().owner, &cancel()).unwrap();
    let key = Key::Existing(node);
    let binding = DomRegistryBinding::Registry(source_registry);
    assert_eq!(
        draft.complete_element_upgrade(key, wrong, &cancel()),
        Err(DomError::InvalidSourceOperation)
    );
    assert_eq!(
        draft
            .lookup_element_definition(key, binding, &cancel())
            .unwrap(),
        Some(definition)
    );
    draft
        .complete_element_upgrade(key, definition, &cancel())
        .unwrap();
    let prepared = draft.finish(&dom, &cancel()).unwrap();
    let commit = dom.publish(prepared, &cancel()).unwrap();
    assert!(
        commit
            .changes()
            .changes()
            .contains(&DomChange::ElementState(node))
    );
    let before_adoption = dom.snapshot();
    let mut batch = DomBatch::new();
    batch.adopt(node, destination.document());
    apply(&mut dom, batch);

    let snapshot = dom.snapshot();
    let view = snapshot.node(node).unwrap().as_element().unwrap();
    assert_eq!(
        view.custom_element_registry(),
        DomRegistryBinding::Registry(destination_registry)
    );
    assert_eq!(
        view.custom_element_definition(),
        DomElementDefinition::Definition(definition)
    );
    assert_eq!(view.custom_element_state(), HtmlCustomElementState::Custom);
    assert_eq!(
        before_adoption
            .node(node)
            .unwrap()
            .as_element()
            .unwrap()
            .custom_element_registry(),
        binding
    );
    assert_eq!(
        initial
            .node(node)
            .unwrap()
            .as_element()
            .unwrap()
            .custom_element_definition(),
        DomElementDefinition::Null
    );

    let export = snapshot
        .html_capture(
            destination.document(),
            None,
            HtmlCaptureLimits::new(256, 4096, 1024, 65_536, 256, 16, 16, 1_000_000),
            DomHtmlCaptureLimits::new(256, 1_000_000),
            &cancel(),
        )
        .unwrap();
    let capture = export.capture();
    let node = capture.node(&export.subject(node).unwrap()).unwrap();
    let HtmlCapturedRegistry::Registry(current_registry) = node.lookup_registry() else {
        panic!("adopted registry");
    };
    let HtmlCapturedNodeKind::Element(element) = node.kind() else {
        panic!("captured element");
    };
    let HtmlCapturedCustomElementDefinition::Definition(associated) =
        element.custom_element_definition()
    else {
        panic!("captured definition");
    };
    assert_ne!(associated.registry(), &current_registry);
    assert_eq!(definition.ordinal(), 1);
    assert_eq!(associated.ordinal(), 1);
    assert_eq!(capture.definition(associated).unwrap(), &facts);
    let captured = HtmlCapturedRegistry::Registry(current_registry);
    assert_eq!(
        capture
            .lookup_definition(
                &captured,
                &surgeist_html::values::Namespace::html(),
                &"x-widget".into(),
                None,
            )
            .unwrap(),
        Some(&competing)
    );
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
fn implementation_attachment_refuses_stale_factory_authority() {
    use surgeist_html::scripting::{HtmlScriptingEngine, HtmlScriptingPolicy};

    let mut dom = dom();
    let mut issuer = issuer(&mut dom);
    let hosted = hosted(&mut dom, &mut issuer);
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let document = apply(&mut dom, batch).document(&document).unwrap();
    let backing = issuer
        .retain_runtime_payload(
            &mut dom,
            DomDocumentScriptRegistration::storage_bytes(),
            &cancel(),
        )
        .unwrap();
    let (_, registration) = issuer
        .attach_document_script_host(
            &mut dom,
            hosted.global(),
            hosted.document(),
            DomDocumentScriptingFacts {
                engine: HtmlScriptingEngine::Supported,
                policy: HtmlScriptingPolicy {
                    user_allows: true,
                    automation_allows: true,
                },
            },
            backing,
            &cancel(),
        )
        .unwrap();
    let fallback = registration.creation_realm_fallback();
    registration.revoke();
    let before = dom.snapshot().stamp();
    for receiver in [document, hosted.document()] {
        let mut batch = DomBatch::new();
        batch.create_detached(
            receiver,
            DomDetachedNodeCreation::DocumentType {
                name: "root".into(),
                public_id: "".into(),
                system_id: "".into(),
                fallback,
            },
        );
        assert!(matches!(
            dom.apply(batch, &cancel()),
            Err(DomError::HostedSourceUnavailable)
        ));
        let edit = DomSourceEdit::create_html_document_in_attachment(receiver, None, fallback);
        let mut lease = dom
            .begin_source_edit(edit, source_limits(1), &cancel())
            .unwrap();
        assert!(matches!(
            lease.advance(&cancel()),
            Err(DomError::HostedSourceUnavailable)
        ));
    }
    assert_eq!(dom.snapshot().stamp(), before);
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
    let shadow = commit.attached_shadow_root(&root).unwrap().unwrap();
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
