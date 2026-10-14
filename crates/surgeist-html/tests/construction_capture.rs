use std::sync::Arc;
use surgeist_html::{bounded::HtmlConstructionResult, *};

fn construction_limits() -> HtmlConstructionLimits {
    HtmlConstructionLimits::new(100, 200, 100, 10_000, 100)
}
fn capture_limits() -> HtmlCaptureLimits {
    HtmlCaptureLimits::new(200, 2000, 200, 100_000, 200, 10, 10)
}
fn adapter_limits() -> HtmlConstructionCaptureLimits {
    HtmlConstructionCaptureLimits::new(200, 20_000, 1_000_000)
}
fn name(local: &str) -> QualifiedName {
    QualifiedName::new(Namespace::html(), None, local)
}
fn properties() -> HtmlShadowRootProperties {
    HtmlShadowRootProperties::new(
        HtmlShadowRootMode::Closed,
        HtmlSlotAssignment::Manual,
        true,
        true,
        false,
    )
}
fn new_builder() -> (HtmlCaptureBuilder, HtmlHostFactsRevision) {
    let revision = HtmlHostFactsRevision::fresh();
    (
        HtmlCaptureBuilder::new(capture_limits(), revision.clone()),
        revision,
    )
}
fn external_document(builder: &mut HtmlCaptureBuilder) -> HtmlCaptureBuilderSubject {
    builder
        .add_document(
            HtmlDocumentMode::NoQuirks,
            HtmlScriptingStatus::Unavailable,
            HtmlCaptureRegistryInput::Unavailable,
        )
        .unwrap()
}
fn root(source: &HtmlConstructionResult) -> HtmlNodeId {
    match source {
        HtmlConstructionResult::Document(value) => value.root().id(),
        HtmlConstructionResult::Fragment(value) => value.root().id(),
    }
}
// Every external slot is explicitly unknown. This producer observes no runtime
// is/registry/history and attests only the supplied document assignment.
fn facts(
    source: &HtmlConstructionResult,
    revision: HtmlHostFactsRevision,
    document: HtmlConstructionCaptureDocument,
) -> HtmlConstructionCaptureFacts {
    let mut ids = Vec::new();
    let mut pending = vec![root(source)];
    while let Some(id) = pending.pop() {
        if ids.contains(&id) {
            continue;
        }
        let node = source.node(&id).unwrap();
        pending.extend(node.children());
        if let Some(id) = node.template_contents() {
            pending.push(id);
        }
        if let Some(shadow) = node.declarative_shadow_root() {
            pending.push(shadow.contents().clone());
        }
        if let Some(HtmlFragmentOwner::Template(owner) | HtmlFragmentOwner::ShadowHost(owner)) =
            node.fragment_owner()
        {
            pending.push(owner.clone());
        }
        ids.push(id);
    }
    HtmlConstructionCaptureFacts::new(
        revision,
        ids.into_iter()
            .map(|id| {
                let node = source.node(&id).unwrap();
                let doc = matches!(node.kind(), HtmlNodeKind::Document(_)).then(|| {
                    HtmlConstructionCaptureDocumentFacts::new(
                        HtmlScriptingStatus::Disabled,
                        HtmlCaptureRegistryInput::Unavailable,
                    )
                });
                let element = matches!(node.kind(), HtmlNodeKind::Element(_)).then(|| {
                    HtmlConstructionCaptureElementFacts::new(
                        HtmlCapturedIsValue::Unavailable,
                        HtmlCaptureRegistryInput::Unavailable,
                    )
                });
                let shadow = node.is_shadow_contents().then(|| {
                    HtmlConstructionCaptureShadowFacts::new(
                        HtmlCaptureRegistryInput::Unavailable,
                        true,
                        false,
                        true,
                    )
                });
                HtmlConstructionCaptureNodeFacts::new(id, document.clone(), doc, element, shadow)
            })
            .collect(),
    )
}
#[expect(clippy::result_large_err)] // Exercise the actual unboxed failure carrier.
fn import(
    builder: HtmlCaptureBuilder,
    source: HtmlConstructionResult,
    facts: HtmlConstructionCaptureFacts,
    limits: HtmlConstructionCaptureLimits,
    cancel: HtmlSemanticCancellation,
) -> Result<HtmlConstructionCaptureImport, HtmlConstructionCaptureFailure> {
    match source {
        HtmlConstructionResult::Document(source) => {
            HtmlConstructionCaptureAdapter::document(builder, source, facts, limits, cancel)
        }
        HtmlConstructionResult::Fragment(source) => {
            HtmlConstructionCaptureAdapter::fragment(builder, source, facts, limits, cancel)
        }
    }
}
fn node<'a>(result: &'a HtmlConstructionCapturedResult, id: &HtmlNodeId) -> HtmlCapturedNode<'a> {
    result
        .capture()
        .node(&result.mapping().subject(id, result.capture()).unwrap())
        .unwrap()
}
fn children(node: HtmlCapturedNode<'_>) -> Vec<HtmlCaptureSubjectId> {
    match node.children() {
        HtmlCapturedChildren::Complete(ids) => ids.collect(),
        _ => panic!("complete children"),
    }
}
#[test]
fn exact_utf16_native_document_shares_backing_and_preserves_order_and_identity() {
    let mut native =
        HtmlDocument::builder(construction_limits(), HtmlDocumentMode::Quirks).unwrap();
    let source_root = native.root();
    let element = native
        .append_element(&source_root, name("div"), OrderedAttributes::default())
        .unwrap();
    let raw = DocumentString::from_utf16(vec![0xd800, 0, 13, 10, 0xdc00]);
    native.append_text(&element, raw.clone()).unwrap();
    native.append_comment(&source_root, "tail").unwrap();
    let source = native.finish();
    let root = source.root().id();
    let ids: Vec<_> = source.root().children().collect();
    let text = source.node(&ids[0]).unwrap().children().next().unwrap();
    let source = HtmlConstructionResult::Document(source);
    let (builder, revision) = new_builder();
    let facts = facts(
        &source,
        revision,
        HtmlConstructionCaptureDocument::ResultDocument,
    );
    let result = import(
        builder,
        source,
        facts,
        adapter_limits(),
        HtmlSemanticCancellation::new(),
    )
    .unwrap()
    .finish()
    .unwrap();
    assert_eq!(result.mapping().root(), &root);
    assert_eq!(
        children(node(&result, &root)),
        ids.iter()
            .map(|id| result.mapping().subject(id, result.capture()).unwrap())
            .collect::<Vec<_>>()
    );
    let HtmlCapturedNodeKind::Text(value) = node(&result, &text).kind() else {
        panic!("text")
    };
    assert_eq!(value.code_units(), [0xd800, 0, 13, 10, 0xdc00]);
    assert!(Arc::ptr_eq(
        value.shared_code_units(),
        raw.shared_code_units()
    ));
    assert_eq!(result.source().node(&root).unwrap().id(), root);
    assert_eq!(
        node(&result, &ids[0]).shadow_root(),
        Some(HtmlCapturedShadowRoot::Unavailable)
    );
    let other = new_builder().0.finish().unwrap();
    assert_eq!(
        result.mapping().subject(&text, &other),
        Err(HtmlConstructionCaptureError::ForeignBuilder)
    );
}
#[test]
fn complete_children_leave_shadow_unknown_and_producer_can_explicitly_establish_absence() {
    let mut native = HtmlFragment::builder(construction_limits()).unwrap();
    let root_handle = native.root();
    native
        .append_element(&root_handle, name("span"), OrderedAttributes::default())
        .unwrap();
    let source = native.finish();
    let element = source.root().children().next().unwrap();
    let root = source.root().id();
    let source = HtmlConstructionResult::Fragment(source);
    let (mut builder, revision) = new_builder();
    let document = external_document(&mut builder);
    let facts = facts(
        &source,
        revision,
        HtmlConstructionCaptureDocument::Existing(document),
    );
    let mut imported = import(
        builder,
        source,
        facts,
        adapter_limits(),
        HtmlSemanticCancellation::new(),
    )
    .unwrap();
    let element_handle = imported
        .mapping()
        .builder_subject(&element)
        .unwrap()
        .clone();
    imported
        .builder_mut()
        .establish_no_shadow_root(&element_handle)
        .unwrap();
    let result = imported.finish().unwrap();
    assert!(children(node(&result, &element)).is_empty());
    assert_eq!(
        node(&result, &element).shadow_root(),
        Some(HtmlCapturedShadowRoot::Absent)
    );
    assert_eq!(node(&result, &root).host(), Some(HtmlCapturedHost::Absent));
}
#[test]
fn actual_shadow_root_imports_properties_and_rejects_conflicting_absence() {
    let mut native = HtmlFragment::builder(construction_limits()).unwrap();
    let root = native.root();
    let host = native
        .append_element(&root, name("div"), OrderedAttributes::default())
        .unwrap();
    let shadow = native
        .attach_declarative_shadow(&host, properties())
        .unwrap();
    native.append_text(&shadow, "shadow").unwrap();
    native.append_text(&host, "light").unwrap();
    let source = native.finish();
    let host = source.root().children().next().unwrap();
    let shadow = source
        .node(&host)
        .unwrap()
        .declarative_shadow_root()
        .unwrap()
        .contents()
        .clone();
    let source = HtmlConstructionResult::Fragment(source);
    let (mut builder, revision) = new_builder();
    let document = external_document(&mut builder);
    let facts = facts(
        &source,
        revision,
        HtmlConstructionCaptureDocument::Existing(document),
    );
    let mut imported = import(
        builder,
        source,
        facts,
        adapter_limits(),
        HtmlSemanticCancellation::new(),
    )
    .unwrap();
    let host_handle = imported.mapping().builder_subject(&host).unwrap().clone();
    assert_eq!(
        imported
            .builder_mut()
            .establish_no_shadow_root(&host_handle),
        Err(HtmlCaptureError::AssociationAlreadyEstablished)
    );
    let result = imported.finish().unwrap();
    assert_eq!(children(node(&result, &host)).len(), 1);
    assert_eq!(children(node(&result, &shadow)).len(), 1);
    assert!(matches!(
        node(&result, &host).shadow_root(),
        Some(HtmlCapturedShadowRoot::Present(_))
    ));
    let HtmlCapturedNodeKind::Fragment(value) = node(&result, &shadow).kind() else {
        panic!("fragment")
    };
    let HtmlCapturedFragmentKind::ShadowRoot(value) = value.kind() else {
        panic!("shadow")
    };
    assert_eq!(value.properties(), properties());
    assert!(value.declarative());
    assert!(!value.available_to_internals());
    assert!(value.keep_registry_null());
    assert_eq!(
        node(&result, &shadow).lookup_registry(),
        HtmlCapturedRegistry::Unavailable
    );
}
#[test]
fn selected_template_fragment_imports_only_its_owner_and_leaves_unrelated_relations_unknown() {
    let mut native =
        HtmlDocument::builder(construction_limits(), HtmlDocumentMode::NoQuirks).unwrap();
    let root = native.root();
    let template = native
        .append_element(&root, name("template"), OrderedAttributes::default())
        .unwrap();
    let contents = native.template_contents(&template).unwrap().unwrap();
    native.append_text(&contents, "inside").unwrap();
    native
        .append_element(&root, name("outside"), OrderedAttributes::default())
        .unwrap();
    let whole = native.finish();
    let ids: Vec<_> = whole.root().children().collect();
    let contents = whole.node(&ids[0]).unwrap().template_contents().unwrap();
    let source = whole.fragment(&contents).unwrap().unwrap();
    let source = HtmlConstructionResult::Fragment(source);
    let (mut builder, revision) = new_builder();
    let document = external_document(&mut builder);
    // Deliberately omit unrelated source document and sibling facts.
    let text = source.node(&contents).unwrap().children().next().unwrap();
    let node_facts = vec![
        HtmlConstructionCaptureNodeFacts::new(
            contents.clone(),
            HtmlConstructionCaptureDocument::Existing(document.clone()),
            None,
            None,
            None,
        ),
        HtmlConstructionCaptureNodeFacts::new(
            text,
            HtmlConstructionCaptureDocument::Existing(document.clone()),
            None,
            None,
            None,
        ),
        HtmlConstructionCaptureNodeFacts::new(
            ids[0].clone(),
            HtmlConstructionCaptureDocument::Existing(document),
            None,
            Some(HtmlConstructionCaptureElementFacts::new(
                HtmlCapturedIsValue::Unavailable,
                HtmlCaptureRegistryInput::Unavailable,
            )),
            None,
        ),
    ];
    let facts = HtmlConstructionCaptureFacts::new(revision, node_facts);
    let result = import(
        builder,
        source,
        facts,
        adapter_limits(),
        HtmlSemanticCancellation::new(),
    )
    .unwrap()
    .finish()
    .unwrap();
    let owner = node(&result, &ids[0]);
    assert!(matches!(
        owner.children(),
        HtmlCapturedChildren::Unavailable
    ));
    assert_eq!(owner.parent(), HtmlCapturedParent::Unavailable);
    assert_eq!(
        owner.shadow_root(),
        Some(HtmlCapturedShadowRoot::Unavailable)
    );
    assert_eq!(
        owner.template_contents_kind(),
        Some(HtmlTemplateContentsKind::Owned)
    );
    assert_eq!(
        result.mapping().builder_subject(&ids[1]),
        Err(HtmlConstructionCaptureError::OutsideImportedClosure)
    );
    assert_eq!(
        result.mapping().builder_subject(&whole.root().id()),
        Err(HtmlConstructionCaptureError::OutsideImportedClosure)
    );
}
#[test]
fn early_fact_refusals_and_each_admission_limit_return_original_builder_and_source() {
    let mut native = HtmlFragment::builder(construction_limits()).unwrap();
    let root = native.root();
    native
        .append_element(&root, name("p"), OrderedAttributes::default())
        .unwrap();
    let source = HtmlConstructionResult::Fragment(native.finish());
    let root = root_id(&source);
    let make = || {
        let (mut builder, revision) = new_builder();
        let document = external_document(&mut builder);
        let facts = facts(
            &source,
            revision,
            HtmlConstructionCaptureDocument::Existing(document.clone()),
        );
        (builder, document, facts)
    };
    let (builder, _, observations) = make();
    let full = import(
        builder,
        source.clone(),
        observations,
        adapter_limits(),
        HtmlSemanticCancellation::new(),
    )
    .unwrap();
    let usage = full.usage();
    for limits in [
        HtmlConstructionCaptureLimits::new(usage.mapping_entries() - 1, 20_000, 1_000_000),
        HtmlConstructionCaptureLimits::new(200, usage.scratch_entries() - 1, 1_000_000),
        HtmlConstructionCaptureLimits::new(200, 20_000, usage.work() - 1),
    ] {
        let (builder, document, facts) = make();
        let before = builder.usage();
        let refused = import(
            builder,
            source.clone(),
            facts,
            limits,
            HtmlSemanticCancellation::new(),
        )
        .unwrap_err();
        assert_eq!(
            refused.reason(),
            &HtmlConstructionCaptureError::LimitExceeded
        );
        assert_eq!(refused.builder().usage(), before);
        assert_eq!(root_id(refused.source()), root);
        let (_, builder, _, _, _) = refused.into_parts();
        let capture = builder.finish().unwrap();
        assert!(capture.subject_id(&document).is_ok());
    }
    let (builder, _, facts) = make();
    let cancellation = HtmlSemanticCancellation::new();
    cancellation.cancel();
    let refused = import(
        builder,
        source.clone(),
        facts,
        adapter_limits(),
        cancellation,
    )
    .unwrap_err();
    assert_eq!(refused.reason(), &HtmlConstructionCaptureError::Cancelled);
    assert_eq!(refused.usage(), HtmlConstructionCaptureUsage::default());
    let (builder, document, previous) = make();
    let facts = HtmlConstructionCaptureFacts::new(previous.host_revision().clone(), vec![]);
    let before = builder.usage();
    let refused = import(
        builder,
        source,
        facts,
        adapter_limits(),
        HtmlSemanticCancellation::new(),
    )
    .unwrap_err();
    assert_eq!(
        refused.reason(),
        &HtmlConstructionCaptureError::MissingNodeDocument
    );
    assert_eq!(refused.builder().usage(), before);
    let (_, builder, _, _, _) = refused.into_parts();
    assert!(builder.finish().unwrap().subject_id(&document).is_ok());
}
fn root_id(source: &HtmlConstructionResult) -> HtmlNodeId {
    root(source)
}
#[test]
fn refused_finish_keeps_caller_facts_mapping_and_original_source() {
    let mut native = HtmlFragment::builder(construction_limits()).unwrap();
    let root = native.root();
    native
        .append_element(&root, name("div"), OrderedAttributes::default())
        .unwrap();
    let source = HtmlConstructionResult::Fragment(native.finish());
    let root = root_id(&source);
    let element = source.node(&root).unwrap().children().next().unwrap();
    let (mut builder, revision) = new_builder();
    let doc = external_document(&mut builder);
    let facts = facts(
        &source,
        revision,
        HtmlConstructionCaptureDocument::Existing(doc),
    );
    let cancellation = HtmlSemanticCancellation::new();
    let mut imported = import(
        builder,
        source,
        facts,
        adapter_limits(),
        cancellation.clone(),
    )
    .unwrap();
    let handle = imported
        .mapping()
        .builder_subject(&element)
        .unwrap()
        .clone();
    imported
        .builder_mut()
        .establish_no_shadow_root(&handle)
        .unwrap();
    cancellation.cancel();
    let refused = imported.finish().unwrap_err();
    assert_eq!(refused.reason(), &HtmlConstructionCaptureError::Cancelled);
    assert_eq!(
        refused
            .import()
            .mapping()
            .builder_subject(&element)
            .unwrap(),
        &handle
    );
    assert_eq!(root_id(refused.import().source()), root);
    let (_, imported) = refused.into_parts();
    let (builder, _, mapping, _) = imported.into_parts();
    let capture = builder.finish().unwrap();
    let element = mapping.subject(&element, &capture).unwrap();
    assert_eq!(
        capture.node(&element).unwrap().shadow_root(),
        Some(HtmlCapturedShadowRoot::Absent)
    );
}

// Drive the real controlled construction owner to create topology the ordinary
// native authoring front door deliberately does not expose.
fn storage() -> bounded::HtmlConstructionStorage {
    use bounded::*;
    let mut value = Some(HtmlConstructionStorage::begin());
    while let Some(action) = value.as_mut().unwrap().next_action().unwrap() {
        action.perform().accept();
    }
    HtmlConstructionInitialization::try_ready(&mut value)
        .unwrap()
        .commit()
}
fn command(
    storage: bounded::HtmlConstructionStorage,
    command: bounded::HtmlConstructionCommand,
) -> bounded::HtmlConstructionCommit {
    use bounded::*;
    let mut value = Some(
        storage
            .prepare(command)
            .unwrap_or_else(|_| panic!("prepare")),
    );
    while let Some(action) = value.as_mut().unwrap().next_action().unwrap() {
        action.perform().unwrap().accept();
    }
    HtmlConstructionOperation::try_ready(&mut value)
        .unwrap()
        .commit()
}
fn change(
    storage: bounded::HtmlConstructionStorage,
    request: bounded::HtmlConstructionCommand,
) -> (bounded::HtmlConstructionStorage, Option<HtmlBuilderNode>) {
    match command(storage, request) {
        bounded::HtmlConstructionCommit::Changed {
            storage,
            created,
            mut retired_orders,
            ..
        } => {
            while let Some(action) = retired_orders.next_action() {
                action.perform();
            }
            (storage, created)
        }
        _ => panic!("change"),
    }
}
fn create(
    storage: bounded::HtmlConstructionStorage,
    request: bounded::HtmlConstructionCommand,
) -> (bounded::HtmlConstructionStorage, HtmlBuilderNode) {
    let (storage, created) = change(storage, request);
    (storage, created.unwrap())
}
fn publish(
    storage: bounded::HtmlConstructionStorage,
    root: HtmlBuilderNode,
) -> HtmlConstructionResult {
    match command(storage, bounded::HtmlConstructionCommand::Publish { root }) {
        bounded::HtmlConstructionCommit::Published { result, .. } => result,
        _ => panic!("publish"),
    }
}
fn create_element(local: &str) -> bounded::HtmlConstructionCommand {
    bounded::HtmlConstructionCommand::Create {
        kind: HtmlNodeKind::Element(HtmlElement::new(name(local), OrderedAttributes::default())),
    }
}
#[test]
fn nonzero_document_root_omits_detached_records_and_retains_actual_mode() {
    use bounded::HtmlConstructionCommand as C;
    let (owner, detached) = create(storage(), C::CreateFragment);
    let (owner, doc) = create(
        owner,
        C::CreateDocument {
            mode: HtmlDocumentMode::Quirks,
        },
    );
    let detached = owner.result_id(&detached).unwrap();
    let actual_root = owner.result_id(&doc).unwrap();
    let source = publish(owner, doc);
    let (builder, revision) = new_builder();
    let facts = facts(
        &source,
        revision,
        HtmlConstructionCaptureDocument::ResultDocument,
    );
    let result = import(
        builder,
        source,
        facts,
        adapter_limits(),
        HtmlSemanticCancellation::new(),
    )
    .unwrap()
    .finish()
    .unwrap();
    assert_eq!(result.mapping().root(), &actual_root);
    assert_eq!(
        result.mapping().builder_subject(&detached),
        Err(HtmlConstructionCaptureError::OutsideImportedClosure)
    );
    let HtmlCapturedNodeKind::Document(value) = node(&result, &actual_root).kind() else {
        panic!("document")
    };
    assert_eq!(value.mode(), HtmlDocumentMode::Quirks);
}
#[test]
fn unowned_shadow_reference_cycle_imports_once_without_fabricating_host_or_depth() {
    use bounded::HtmlConstructionCommand as C;
    let (owner, shadow) = create(
        storage(),
        C::CreateShadowContents {
            properties: properties(),
        },
    );
    let (owner, template) = create(owner, create_element("template"));
    let (owner, _) = change(
        owner,
        C::Insert {
            node: template.clone(),
            parent: shadow.clone(),
            before: None,
        },
    );
    let (owner, _) = change(
        owner,
        C::SetTemplateShadowContents {
            template: template.clone(),
            contents: shadow.clone(),
        },
    );
    let shadow_id = owner.result_id(&shadow).unwrap();
    let template_id = owner.result_id(&template).unwrap();
    let source = publish(owner, shadow);
    let (mut builder, revision) = new_builder();
    let document = external_document(&mut builder);
    let facts = facts(
        &source,
        revision,
        HtmlConstructionCaptureDocument::Existing(document),
    );
    let imported = import(
        builder,
        source,
        facts,
        adapter_limits(),
        HtmlSemanticCancellation::new(),
    )
    .unwrap();
    assert_eq!(imported.usage().mapping_entries(), 2);
    assert_eq!(imported.builder().usage().depth(), 1);
    let result = imported.finish().unwrap();
    assert_eq!(
        node(&result, &shadow_id).host(),
        Some(HtmlCapturedHost::Unavailable)
    );
    assert_eq!(
        node(&result, &template_id).template_contents_kind(),
        Some(HtmlTemplateContentsKind::ShadowReference)
    );
    assert_eq!(
        children(node(&result, &shadow_id)),
        [result
            .mapping()
            .subject(&template_id, result.capture())
            .unwrap()]
    );
}
#[test]
fn shadow_reference_reaches_external_subtree_and_only_its_supplemental_host() {
    use bounded::HtmlConstructionCommand as C;
    let (owner, selected) = create(storage(), C::CreateFragment);
    let (owner, template) = create(owner, create_element("template"));
    let (owner, host) = create(owner, create_element("div"));
    let (owner, shadow) = create(
        owner,
        C::CreateShadowContents {
            properties: properties(),
        },
    );
    let (owner, _) = change(
        owner,
        C::Insert {
            node: template.clone(),
            parent: selected.clone(),
            before: None,
        },
    );
    let (owner, _) = change(
        owner,
        C::AttachTemplateShadow {
            template: template.clone(),
            host: host.clone(),
            contents: shadow.clone(),
        },
    );
    let template = owner.result_id(&template).unwrap();
    let host = owner.result_id(&host).unwrap();
    let shadow = owner.result_id(&shadow).unwrap();
    let source = publish(owner, selected);
    let (mut builder, revision) = new_builder();
    let document = external_document(&mut builder);
    let facts = facts(
        &source,
        revision,
        HtmlConstructionCaptureDocument::Existing(document),
    );
    let imported = import(
        builder,
        source,
        facts,
        adapter_limits(),
        HtmlSemanticCancellation::new(),
    )
    .unwrap();
    assert_eq!(imported.usage().mapping_entries(), 4);
    let result = imported.finish().unwrap();
    assert!(matches!(
        node(&result, &host).children(),
        HtmlCapturedChildren::Unavailable
    ));
    assert_eq!(
        node(&result, &host).parent(),
        HtmlCapturedParent::Unavailable
    );
    assert_eq!(
        node(&result, &template).template_contents_kind(),
        Some(HtmlTemplateContentsKind::ShadowReference)
    );
    assert!(matches!(
        node(&result, &shadow).host(),
        Some(HtmlCapturedHost::Element(_))
    ));
    assert!(children(node(&result, &shadow)).is_empty());
}
#[test]
fn revision_foreign_ids_duplicate_roles_and_missing_shadow_facts_are_typed_refusals() {
    let source = HtmlConstructionResult::Fragment(
        HtmlFragment::builder(construction_limits())
            .unwrap()
            .finish(),
    );
    let id = root(&source);
    let run = |make: &dyn Fn(
        HtmlHostFactsRevision,
        HtmlCaptureBuilderSubject,
    ) -> HtmlConstructionCaptureFacts,
               expected| {
        let (mut builder, revision) = new_builder();
        let document = external_document(&mut builder);
        let before = builder.usage();
        let refused = import(
            builder,
            source.clone(),
            make(revision, document),
            adapter_limits(),
            HtmlSemanticCancellation::new(),
        )
        .unwrap_err();
        assert_eq!(refused.reason(), &expected);
        assert_eq!(refused.builder().usage(), before);
    };
    run(
        &|_, _| HtmlConstructionCaptureFacts::new(HtmlHostFactsRevision::fresh(), vec![]),
        HtmlConstructionCaptureError::HostRevisionMismatch,
    );
    let other = HtmlFragment::builder(construction_limits())
        .unwrap()
        .finish();
    run(
        &|revision, document| {
            HtmlConstructionCaptureFacts::new(
                revision,
                vec![HtmlConstructionCaptureNodeFacts::new(
                    other.root().id(),
                    HtmlConstructionCaptureDocument::Existing(document),
                    None,
                    None,
                    None,
                )],
            )
        },
        HtmlConstructionCaptureError::ForeignResult,
    );
    run(
        &|revision, document| {
            let entry = HtmlConstructionCaptureNodeFacts::new(
                id.clone(),
                HtmlConstructionCaptureDocument::Existing(document),
                None,
                None,
                None,
            );
            HtmlConstructionCaptureFacts::new(revision, vec![entry.clone(), entry])
        },
        HtmlConstructionCaptureError::DuplicateNodeFacts,
    );
    run(
        &|revision, document| {
            HtmlConstructionCaptureFacts::new(
                revision,
                vec![HtmlConstructionCaptureNodeFacts::new(
                    id.clone(),
                    HtmlConstructionCaptureDocument::Existing(document),
                    None,
                    Some(HtmlConstructionCaptureElementFacts::new(
                        HtmlCapturedIsValue::Absent,
                        HtmlCaptureRegistryInput::Null,
                    )),
                    None,
                )],
            )
        },
        HtmlConstructionCaptureError::WrongFactRole,
    );
    let (mut foreign, _) = new_builder();
    let document = external_document(&mut foreign);
    run(
        &|revision, _| {
            HtmlConstructionCaptureFacts::new(
                revision,
                vec![HtmlConstructionCaptureNodeFacts::new(
                    id.clone(),
                    HtmlConstructionCaptureDocument::Existing(document.clone()),
                    None,
                    None,
                    None,
                )],
            )
        },
        HtmlConstructionCaptureError::ForeignBuilder,
    );
    use bounded::HtmlConstructionCommand as C;
    let (owner, shadow) = create(
        storage(),
        C::CreateShadowContents {
            properties: properties(),
        },
    );
    let source = publish(owner, shadow);
    let (mut builder, revision) = new_builder();
    let document = external_document(&mut builder);
    let missing = HtmlConstructionCaptureFacts::new(
        revision,
        vec![HtmlConstructionCaptureNodeFacts::new(
            root(&source),
            HtmlConstructionCaptureDocument::Existing(document),
            None,
            None,
            None,
        )],
    );
    let refused = import(
        builder,
        source,
        missing,
        adapter_limits(),
        HtmlSemanticCancellation::new(),
    )
    .unwrap_err();
    assert_eq!(
        refused.reason(),
        &HtmlConstructionCaptureError::MissingShadowFacts
    );
}
#[test]
fn parsed_document_enters_the_same_adapter_with_original_result_identity() {
    let limits = HtmlParseLimits::builder()
        .source_units(10000)
        .buffered_utf16_units(10000)
        .token_attributes(1000)
        .auxiliary_entries(10000)
        .managed_nodes(1000)
        .managed_edges(2000)
        .managed_attributes(1000)
        .managed_utf16_units(10000)
        .source_map_entries(10000)
        .source_map_payload_units(10000)
        .diagnostics(1000)
        .diagnostic_payload_units(10000)
        .context_effects(1000)
        .open_elements(100)
        .tree_depth(1000)
        .managed_capacity_bytes(10_000_000)
        .peak_transient_bytes(10_000_000)
        .atomic_layout_bytes(10_000_000)
        .work_credits(10_000_000)
        .finish()
        .unwrap();
    let report =
        HtmlDocumentParseRequest::offline(HtmlSource::utf8("<!doctype html><p>x&amp;y"), limits)
            .parse()
            .unwrap();
    let document = report.document().unwrap().clone();
    let id = document.root().id();
    let source = HtmlConstructionResult::Document(document);
    let (builder, revision) = new_builder();
    let facts = facts(
        &source,
        revision,
        HtmlConstructionCaptureDocument::ResultDocument,
    );
    let result = import(
        builder,
        source,
        facts,
        adapter_limits(),
        HtmlSemanticCancellation::new(),
    )
    .unwrap()
    .finish()
    .unwrap();
    assert_eq!(result.mapping().root(), &id);
    assert_eq!(
        result.source().document().unwrap().root().id(),
        report.document().unwrap().root().id()
    );
    assert!(children(node(&result, &id)).len() >= 2);
}

#[test]
fn template_contents_keep_their_explicit_distinct_document_assignment() {
    let mut native = HtmlFragment::builder(construction_limits()).unwrap();
    let root_handle = native.root();
    let template_handle = native
        .append_element(&root_handle, name("template"), OrderedAttributes::default())
        .unwrap();
    let contents_handle = native.template_contents(&template_handle).unwrap().unwrap();
    native.append_text(&contents_handle, "inert").unwrap();
    let source = HtmlConstructionResult::Fragment(native.finish());
    let root = root(&source);
    let template = source.node(&root).unwrap().children().next().unwrap();
    let contents = source.node(&template).unwrap().template_contents().unwrap();
    let text = source.node(&contents).unwrap().children().next().unwrap();
    let (mut builder, revision) = new_builder();
    let host_document = external_document(&mut builder);
    let contents_document = external_document(&mut builder);
    let observations = HtmlConstructionCaptureFacts::new(
        revision,
        vec![
            HtmlConstructionCaptureNodeFacts::new(
                root,
                HtmlConstructionCaptureDocument::Existing(host_document.clone()),
                None,
                None,
                None,
            ),
            HtmlConstructionCaptureNodeFacts::new(
                template.clone(),
                HtmlConstructionCaptureDocument::Existing(host_document.clone()),
                None,
                Some(HtmlConstructionCaptureElementFacts::new(
                    HtmlCapturedIsValue::Unavailable,
                    HtmlCaptureRegistryInput::Unavailable,
                )),
                None,
            ),
            HtmlConstructionCaptureNodeFacts::new(
                contents.clone(),
                HtmlConstructionCaptureDocument::Existing(contents_document.clone()),
                None,
                None,
                None,
            ),
            HtmlConstructionCaptureNodeFacts::new(
                text,
                HtmlConstructionCaptureDocument::Existing(contents_document.clone()),
                None,
                None,
                None,
            ),
        ],
    );
    let result = import(
        builder,
        source,
        observations,
        adapter_limits(),
        HtmlSemanticCancellation::new(),
    )
    .unwrap()
    .finish()
    .unwrap();
    assert_eq!(
        node(&result, &template).node_document().subject(),
        &result.capture().subject_id(&host_document).unwrap()
    );
    assert_eq!(
        node(&result, &contents).node_document().subject(),
        &result.capture().subject_id(&contents_document).unwrap()
    );
    assert_eq!(
        node(&result, &template).template_contents_kind(),
        Some(HtmlTemplateContentsKind::Owned)
    );
}

#[test]
fn finish_work_refusal_preserves_caller_additions_without_consuming_the_builder() {
    let mut native = HtmlFragment::builder(construction_limits()).unwrap();
    let root_handle = native.root();
    native
        .append_element(&root_handle, name("div"), OrderedAttributes::default())
        .unwrap();
    let source = HtmlConstructionResult::Fragment(native.finish());
    let element = source
        .node(&root(&source))
        .unwrap()
        .children()
        .next()
        .unwrap();
    let make = |limits| {
        let (mut builder, revision) = new_builder();
        let document = external_document(&mut builder);
        let observations = facts(
            &source,
            revision,
            HtmlConstructionCaptureDocument::Existing(document),
        );
        import(
            builder,
            source.clone(),
            observations,
            limits,
            HtmlSemanticCancellation::new(),
        )
        .unwrap()
    };
    let ordinary = make(adapter_limits());
    let work = ordinary.usage().work();
    drop(ordinary);
    let mut imported = make(HtmlConstructionCaptureLimits::new(200, 20_000, work + 5));
    let subject = imported
        .mapping()
        .builder_subject(&element)
        .unwrap()
        .clone();
    imported
        .builder_mut()
        .establish_no_shadow_root(&subject)
        .unwrap();
    let before = imported.builder().usage();
    let refused = imported.finish().unwrap_err();
    assert_eq!(
        refused.reason(),
        &HtmlConstructionCaptureError::LimitExceeded
    );
    assert_eq!(refused.import().builder().usage(), before);
    assert_eq!(
        refused
            .import()
            .mapping()
            .builder_subject(&element)
            .unwrap(),
        &subject
    );
    let (_, imported) = refused.into_parts();
    let (builder, source, mapping, _) = imported.into_parts();
    assert!(source.node(&element).is_ok());
    let capture = builder.finish().unwrap();
    let subject = mapping.subject(&element, &capture).unwrap();
    assert_eq!(
        capture.node(&subject).unwrap().shadow_root(),
        Some(HtmlCapturedShadowRoot::Absent)
    );
}
