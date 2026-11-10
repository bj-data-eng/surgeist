use surgeist_html::{
    bounded::construction::HtmlConstructionResult,
    construction::{
        HtmlBuilderNode, HtmlConstructionLimits, HtmlDocument, HtmlDocumentMode, HtmlElement,
        HtmlFragment, HtmlFragmentBuilder, HtmlFragmentOwner, HtmlNodeId, HtmlNodeKind,
        HtmlShadowRootMode, HtmlShadowRootProperties, HtmlSlotAssignment, HtmlTemplateContentsKind,
    },
    values::{DocumentString, Namespace, OrderedAttributes, QualifiedName},
};
use surgeist_html_semantics::{
    HtmlCaptureBuilder, HtmlCaptureBuilderSubject, HtmlCaptureError, HtmlCaptureLimits,
    HtmlCaptureRegistryInput, HtmlCaptureSubjectId, HtmlCapturedChildren, HtmlCapturedFragmentKind,
    HtmlCapturedHost, HtmlCapturedIsValue, HtmlCapturedNode, HtmlCapturedNodeKind,
    HtmlCapturedParent, HtmlCapturedRegistry, HtmlCapturedShadowRoot,
    HtmlConstructionCaptureAdapter, HtmlConstructionCaptureDocument,
    HtmlConstructionCaptureDocumentFacts, HtmlConstructionCaptureElementFacts,
    HtmlConstructionCaptureError, HtmlConstructionCaptureFacts, HtmlConstructionCaptureFailure,
    HtmlConstructionCaptureImport, HtmlConstructionCaptureLimits, HtmlConstructionCaptureNodeFacts,
    HtmlConstructionCaptureShadowFacts, HtmlConstructionCaptureUsage,
    HtmlConstructionCapturedResult, HtmlHostFactsRevision, HtmlScriptingStatus,
    HtmlSemanticCancellation,
};

fn construction_limits() -> HtmlConstructionLimits {
    HtmlConstructionLimits::new(100, 200, 100, 10_000, 100)
}
fn capture_limits() -> HtmlCaptureLimits {
    HtmlCaptureLimits::new(200, 2000, 200, 100_000, 200, 10, 10, 16_777_216)
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
    facts_with(source, revision, document, &[])
}
/// The facts of the closure of the source's root and of the `extra` nodes, which
/// the caller names whether or not the root reaches them.
fn facts_with(
    source: &HtmlConstructionResult,
    revision: HtmlHostFactsRevision,
    document: HtmlConstructionCaptureDocument,
    extra: &[HtmlNodeId],
) -> HtmlConstructionCaptureFacts {
    let mut ids = Vec::new();
    ids.extend(extra.iter().cloned());
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
            pending.push(shadow.contents());
        }
        if let Some(HtmlFragmentOwner::Template(owner) | HtmlFragmentOwner::ShadowHost(owner)) =
            node.fragment_owner()
        {
            pending.push(owner);
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
    let source = native.finish().unwrap();
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
    assert!(value.shares_storage_with(&raw));
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
    let source = native.finish().unwrap();
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
    let source = native.finish().unwrap();
    let host = source.root().children().next().unwrap();
    let shadow = source
        .node(&host)
        .unwrap()
        .declarative_shadow_root()
        .unwrap()
        .contents();
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
fn hosted_shadow_import_preserves_its_evidenced_document_independently_of_host() {
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
    let source = native.finish().unwrap();
    let host = source.root().children().next().unwrap();
    let shadow = source
        .node(&host)
        .unwrap()
        .declarative_shadow_root()
        .unwrap()
        .contents();
    let shadow_text = source.node(&shadow).unwrap().children().next().unwrap();
    let light = source.node(&host).unwrap().children().next().unwrap();
    let source = HtmlConstructionResult::Fragment(source);
    let (mut builder, revision) = new_builder();
    let host_document = external_document(&mut builder);
    let root_document = external_document(&mut builder);
    let observed = facts(
        &source,
        revision.clone(),
        HtmlConstructionCaptureDocument::Existing(host_document.clone()),
    );
    // DOM's internal replace/adopt path changes the root's node document while
    // retaining its host association. Each node's actual document is supplied
    // independently; ordinary shadow children still share their parent's one.
    let facts = HtmlConstructionCaptureFacts::new(
        revision,
        observed
            .nodes()
            .iter()
            .map(|entry| {
                let document = if entry.node() == &shadow || entry.node() == &shadow_text {
                    HtmlConstructionCaptureDocument::Existing(root_document.clone())
                } else {
                    entry.document().clone()
                };
                HtmlConstructionCaptureNodeFacts::new(
                    *entry.node(),
                    document,
                    entry.document_facts().cloned(),
                    entry.element_facts().cloned(),
                    entry.shadow_facts().cloned(),
                )
            })
            .collect(),
    );
    let result = import(
        builder,
        source,
        facts,
        adapter_limits(),
        HtmlSemanticCancellation::new(),
    )
    .unwrap_or_else(|failure| panic!("two-document shadow import: {:?}", failure.reason()))
    .finish()
    .unwrap();
    let capture = result.capture();
    let host_document = capture.subject_id(&host_document).unwrap();
    let root_document = capture.subject_id(&root_document).unwrap();
    assert_eq!(
        node(&result, &host).node_document().subject(),
        &host_document
    );
    assert_eq!(
        node(&result, &light).node_document().subject(),
        &host_document
    );
    assert_eq!(
        node(&result, &shadow).node_document().subject(),
        &root_document
    );
    assert_eq!(
        node(&result, &shadow_text).node_document().subject(),
        &root_document
    );
    let host_id = capture.element_id(&node(&result, &host).id()).unwrap();
    assert_eq!(
        node(&result, &shadow).host(),
        Some(HtmlCapturedHost::Element(host_id))
    );
    assert!(matches!(
        node(&result, &host).shadow_root(),
        Some(HtmlCapturedShadowRoot::Present(root)) if root.subject() == &node(&result, &shadow).id()
    ));
    assert_eq!(
        children(node(&result, &shadow)),
        [node(&result, &shadow_text).id()]
    );
    assert_eq!(children(node(&result, &host)), [node(&result, &light).id()]);
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
    let whole = native.finish().unwrap();
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
            contents,
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
            ids[0],
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
    let source = HtmlConstructionResult::Fragment(native.finish().unwrap());
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
    // The fragment root and its one appended element are the two mapped nodes,
    // each needs at least one traversal step and the worklist holds at least
    // one entry. The adapter's remaining step layout is private, so each
    // reported amount is checked to be the least admitting limit: it succeeds
    // exactly and, in the loop below, one less is refused.
    assert!(usage.mapping_entries() >= 2);
    assert!(usage.scratch_entries() >= 1);
    assert!(usage.work() >= usage.mapping_entries());
    let (builder, _, observations) = make();
    import(
        builder,
        source.clone(),
        observations,
        HtmlConstructionCaptureLimits::new(
            usage.mapping_entries(),
            usage.scratch_entries(),
            usage.work(),
        ),
        HtmlSemanticCancellation::new(),
    )
    .unwrap();
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
    let source = HtmlConstructionResult::Fragment(native.finish().unwrap());
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
fn storage() -> surgeist_html::bounded::construction::HtmlConstructionStorage {
    use surgeist_html::bounded::construction::HtmlConstructionStorage;
    let mut value = HtmlConstructionStorage::begin();
    while let Some(action) = value.next_action().unwrap() {
        action.perform().accept();
    }
    value.try_ready().unwrap().commit()
}
fn command(
    storage: surgeist_html::bounded::construction::HtmlConstructionStorage,
    command: surgeist_html::bounded::construction::HtmlConstructionCommand,
) -> surgeist_html::bounded::construction::HtmlConstructionCommit {
    let mut value = storage.prepare(command);
    while let Some(action) = value.next_action().unwrap() {
        action.perform().unwrap().accept();
    }
    value.try_ready().unwrap().commit().unwrap()
}
fn change(
    storage: surgeist_html::bounded::construction::HtmlConstructionStorage,
    request: surgeist_html::bounded::construction::HtmlConstructionCommand,
) -> (
    surgeist_html::bounded::construction::HtmlConstructionStorage,
    Option<HtmlBuilderNode>,
) {
    match command(storage, request) {
        surgeist_html::bounded::construction::HtmlConstructionCommit::Changed {
            storage,
            created,
        } => (storage, created),
        _ => panic!("change"),
    }
}
fn create(
    storage: surgeist_html::bounded::construction::HtmlConstructionStorage,
    request: surgeist_html::bounded::construction::HtmlConstructionCommand,
) -> (
    surgeist_html::bounded::construction::HtmlConstructionStorage,
    HtmlBuilderNode,
) {
    let (storage, created) = change(storage, request);
    (storage, created.unwrap())
}
fn publish(
    storage: surgeist_html::bounded::construction::HtmlConstructionStorage,
    root: HtmlBuilderNode,
) -> HtmlConstructionResult {
    match command(
        storage,
        surgeist_html::bounded::construction::HtmlConstructionCommand::Publish { root },
    ) {
        surgeist_html::bounded::construction::HtmlConstructionCommit::Published {
            result, ..
        } => result,
        _ => panic!("publish"),
    }
}
fn create_element(local: &str) -> surgeist_html::bounded::construction::HtmlConstructionCommand {
    surgeist_html::bounded::construction::HtmlConstructionCommand::Create {
        kind: HtmlNodeKind::Element(HtmlElement::new(name(local), OrderedAttributes::default())),
    }
}
#[test]
fn nonzero_document_root_omits_detached_records_and_retains_actual_mode() {
    use surgeist_html::bounded::construction::HtmlConstructionCommand as C;
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
/// Facts that name a node the import never reaches are a producer's mistake, for
/// example facts keyed to a different root, and are refused rather than ignored.
#[test]
fn facts_naming_a_node_outside_the_imported_closure_are_refused() {
    use surgeist_html::bounded::construction::HtmlConstructionCommand as C;
    let (owner, detached) = create(storage(), C::CreateFragment);
    let (owner, doc) = create(
        owner,
        C::CreateDocument {
            mode: HtmlDocumentMode::Quirks,
        },
    );
    let detached = owner.result_id(&detached).unwrap();
    let source = publish(owner, doc);
    let (builder, revision) = new_builder();
    let facts = facts_with(
        &source,
        revision,
        HtmlConstructionCaptureDocument::ResultDocument,
        std::slice::from_ref(&detached),
    );
    let refused = import(
        builder,
        source,
        facts,
        adapter_limits(),
        HtmlSemanticCancellation::new(),
    )
    .map(|_| ())
    .unwrap_err();
    assert_eq!(
        refused.reason(),
        &HtmlConstructionCaptureError::OutsideImportedClosure
    );
}
/// The first entry outside the closure, in the order of the facts, is the one the
/// import stops at: the work admitted by then is the same for either of two
/// entries outside when each comes first, and is more when the entry sits later
/// among the facts.
#[test]
fn the_first_fact_entry_outside_the_closure_is_where_the_import_stops() {
    use surgeist_html::bounded::construction::HtmlConstructionCommand as C;
    let (owner, first) = create(storage(), C::CreateFragment);
    let (owner, second) = create(owner, C::CreateFragment);
    let (owner, doc) = create(
        owner,
        C::CreateDocument {
            mode: HtmlDocumentMode::Quirks,
        },
    );
    let first = owner.result_id(&first).unwrap();
    let second = owner.result_id(&second).unwrap();
    let source = publish(owner, doc);
    let stopped_with = |outside: &[HtmlNodeId], last: bool| {
        let (builder, revision) = new_builder();
        let facts = facts_with(
            &source,
            revision.clone(),
            HtmlConstructionCaptureDocument::ResultDocument,
            outside,
        );
        let mut rows = facts.nodes().to_vec();
        if last {
            rows.rotate_left(outside.len());
        }
        let refused = import(
            builder,
            source.clone(),
            HtmlConstructionCaptureFacts::new(revision, rows),
            adapter_limits(),
            HtmlSemanticCancellation::new(),
        )
        .map(|_| ())
        .unwrap_err();
        assert_eq!(
            refused.reason(),
            &HtmlConstructionCaptureError::OutsideImportedClosure
        );
        refused.usage().work()
    };
    // Two entries outside, in either order, stop at the one that comes first.
    let two = stopped_with(&[first, second], false);
    let reversed = stopped_with(&[second, first], false);
    assert_eq!(reversed, two);
    // An entry outside is reached after the entries of the closure when it comes
    // last, so the import stops later, not at the end of a full scan.
    let early = stopped_with(std::slice::from_ref(&first), false);
    let late = stopped_with(std::slice::from_ref(&first), true);
    assert!(late > early, "{late} and {early}");
}
#[test]
fn unowned_shadow_reference_cycle_imports_once_without_fabricating_host_or_depth() {
    use surgeist_html::bounded::construction::HtmlConstructionCommand as C;
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
            node: template,
            parent: shadow,
            before: None,
        },
    );
    let (owner, _) = change(
        owner,
        C::SetTemplateShadowContents {
            template,
            contents: shadow,
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
    use surgeist_html::bounded::construction::HtmlConstructionCommand as C;
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
            node: template,
            parent: selected,
            before: None,
        },
    );
    let (owner, _) = change(
        owner,
        C::AttachTemplateShadow {
            template,
            host,
            contents: shadow,
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
// A template's shadow reference is independent of the template's ordinary
// position (html-source 147330-147332; the DOM lets a retained template be
// inserted under any element): the template below an element that is not the
// root's host still imports and finishes.
#[test]
fn a_template_below_an_unrelated_element_keeps_its_shadow_reference() {
    use surgeist_html::bounded::construction::HtmlConstructionCommand as C;
    let (owner, selected) = create(storage(), C::CreateFragment);
    let (owner, parent) = create(owner, create_element("p"));
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
            node: parent,
            parent: selected,
            before: None,
        },
    );
    let (owner, _) = change(
        owner,
        C::Insert {
            node: template,
            parent,
            before: None,
        },
    );
    let (owner, _) = change(
        owner,
        C::AttachTemplateShadow {
            template,
            host,
            contents: shadow,
        },
    );
    let parent = owner.result_id(&parent).unwrap();
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
    let subject = |id: &HtmlNodeId| result.mapping().subject(id, result.capture()).unwrap();
    assert_eq!(
        node(&result, &template).template_contents_kind(),
        Some(HtmlTemplateContentsKind::ShadowReference)
    );
    assert_eq!(
        node(&result, &template).parent(),
        HtmlCapturedParent::Subject(subject(&parent))
    );
    assert!(matches!(
        node(&result, &shadow).host(),
        Some(HtmlCapturedHost::Element(h)) if h.subject() == &subject(&host)
    ));
}
#[test]
fn revision_foreign_ids_duplicate_roles_and_missing_shadow_facts_are_typed_refusals() {
    let source = HtmlConstructionResult::Fragment(
        HtmlFragment::builder(construction_limits())
            .unwrap()
            .finish()
            .unwrap(),
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
        .finish()
        .unwrap();
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
                id,
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
                    id,
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
                    id,
                    HtmlConstructionCaptureDocument::Existing(document.clone()),
                    None,
                    None,
                    None,
                )],
            )
        },
        HtmlConstructionCaptureError::ForeignBuilder,
    );
    use surgeist_html::bounded::construction::HtmlConstructionCommand as C;
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
fn template_contents_keep_their_explicit_distinct_document_assignment() {
    let mut native = HtmlFragment::builder(construction_limits()).unwrap();
    let root_handle = native.root();
    let template_handle = native
        .append_element(&root_handle, name("template"), OrderedAttributes::default())
        .unwrap();
    let contents_handle = native.template_contents(&template_handle).unwrap().unwrap();
    native.append_text(&contents_handle, "inert").unwrap();
    let source = HtmlConstructionResult::Fragment(native.finish().unwrap());
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
                template,
                HtmlConstructionCaptureDocument::Existing(host_document.clone()),
                None,
                Some(HtmlConstructionCaptureElementFacts::new(
                    HtmlCapturedIsValue::Unavailable,
                    HtmlCaptureRegistryInput::Unavailable,
                )),
                None,
            ),
            HtmlConstructionCaptureNodeFacts::new(
                contents,
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
    let source = HtmlConstructionResult::Fragment(native.finish().unwrap());
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

/// Imports `source`, which has `nodes` nodes of at most `depth` levels, with a
/// fact for every node, into a fresh builder that never refuses.
#[expect(clippy::result_large_err)] // Exercise the actual unboxed failure carrier.
fn import_all_nodes(
    source: &HtmlFragment,
    nodes: usize,
    depth: usize,
    limits: HtmlConstructionCaptureLimits,
) -> Result<HtmlConstructionCaptureImport, HtmlConstructionCaptureFailure> {
    let source = HtmlConstructionResult::Fragment(source.clone());
    let revision = HtmlHostFactsRevision::fresh();
    let mut builder = HtmlCaptureBuilder::new(
        HtmlCaptureLimits::new(
            nodes + 8,
            8 * nodes + 64,
            0,
            64 * nodes + 256,
            depth + 8,
            4,
            4,
            1 << 20,
        ),
        revision.clone(),
    );
    let document = external_document(&mut builder);
    let facts = facts(
        &source,
        revision,
        HtmlConstructionCaptureDocument::Existing(document),
    );
    import(
        builder,
        source,
        facts,
        limits,
        HtmlSemanticCancellation::new(),
    )
}
/// Usage of importing `source` under allowances that never refuse.
fn usage_of_import(
    source: HtmlFragment,
    nodes: usize,
    depth: usize,
) -> HtmlConstructionCaptureUsage {
    let imported = import_all_nodes(
        &source,
        nodes,
        depth,
        HtmlConstructionCaptureLimits::new(nodes + 8, 1 << 24, 1 << 40),
    )
    .unwrap();
    assert_eq!(imported.usage().mapping_entries(), nodes);
    imported.usage()
}
fn fragment_builder(nodes: usize, depth: usize) -> HtmlFragmentBuilder {
    HtmlFragment::builder(HtmlConstructionLimits::new(
        nodes + 8,
        2 * nodes + 8,
        0,
        64 * nodes + 256,
        depth + 8,
    ))
    .unwrap()
}
/// A fragment root with `children` empty elements.
fn wide_fragment(children: usize) -> HtmlConstructionCaptureUsage {
    let mut native = fragment_builder(children + 1, 1);
    let root = native.root();
    for _ in 0..children {
        native
            .append_element(&root, name("p"), OrderedAttributes::default())
            .unwrap();
    }
    usage_of_import(native.finish().unwrap(), children + 1, 1)
}
/// A fragment root with `pairs` elements that each contain one element.
fn paired_fragment(pairs: usize) -> HtmlConstructionCaptureUsage {
    let mut native = fragment_builder(2 * pairs + 1, 2);
    let root = native.root();
    for _ in 0..pairs {
        let outer = native
            .append_element(&root, name("li"), OrderedAttributes::default())
            .unwrap();
        native
            .append_element(&outer, name("a"), OrderedAttributes::default())
            .unwrap();
    }
    usage_of_import(native.finish().unwrap(), 2 * pairs + 1, 2)
}
/// A fragment root with a single chain of `depth` nested elements.
fn chain_fragment(depth: usize) -> HtmlConstructionCaptureUsage {
    let mut native = fragment_builder(depth + 1, depth);
    let mut parent = native.root();
    for _ in 0..depth {
        parent = native
            .append_element(&parent, name("div"), OrderedAttributes::default())
            .unwrap();
    }
    usage_of_import(native.finish().unwrap(), depth + 1, depth)
}
/// A fragment root holding a template whose contents hold a template, and so on
/// `templates` times: the contents are attached to their template after their
/// own contents have been established.
fn nested_templates(templates: usize) -> HtmlConstructionCaptureUsage {
    let nodes = 2 * templates + 1;
    let mut native = fragment_builder(nodes, 2 * templates);
    let mut parent = native.root();
    for _ in 0..templates {
        let template = native
            .append_element(&parent, name("template"), OrderedAttributes::default())
            .unwrap();
        parent = native.template_contents(&template).unwrap().unwrap();
    }
    usage_of_import(native.finish().unwrap(), nodes, 2 * templates)
}
/// A fragment root holding a host whose declarative shadow root holds a host,
/// and so on `hosts` times.
fn nested_shadows(hosts: usize) -> HtmlConstructionCaptureUsage {
    let nodes = 2 * hosts + 1;
    let mut native = fragment_builder(nodes, 2 * hosts);
    let mut parent = native.root();
    for _ in 0..hosts {
        let host = native
            .append_element(&parent, name("div"), OrderedAttributes::default())
            .unwrap();
        parent = native
            .attach_declarative_shadow(&host, properties())
            .unwrap();
    }
    usage_of_import(native.finish().unwrap(), nodes, 2 * hosts)
}
/// A chain of `size` nested elements whose deepest element holds `size`
/// templates, each with one element in its contents.
fn chain_with_templates(size: usize) -> HtmlConstructionCaptureUsage {
    let nodes = 1 + size + 3 * size;
    let mut native = fragment_builder(nodes, size + 2);
    let mut parent = native.root();
    for _ in 0..size {
        parent = native
            .append_element(&parent, name("div"), OrderedAttributes::default())
            .unwrap();
    }
    for _ in 0..size {
        let template = native
            .append_element(&parent, name("template"), OrderedAttributes::default())
            .unwrap();
        let contents = native.template_contents(&template).unwrap().unwrap();
        native
            .append_element(&contents, name("p"), OrderedAttributes::default())
            .unwrap();
    }
    usage_of_import(native.finish().unwrap(), nodes, size + 2)
}
/// Every doubling of the size must at most double a charge that does not depend
/// on the other nodes. A little slack covers the amortized growth of arrays,
/// whose moves are charged when they happen; a charge that scans the earlier
/// nodes for every node roughly quadruples it.
fn assert_charge_is_linear(
    shape: &str,
    charge: &str,
    measure: impl Fn(usize) -> HtmlConstructionCaptureUsage,
    pick: impl Fn(HtmlConstructionCaptureUsage) -> usize,
) {
    let sizes = [50, 100, 200, 400];
    let amounts = sizes.map(|size| pick(measure(size)));
    for (pair, amounts) in sizes.windows(2).zip(amounts.windows(2)) {
        assert!(
            amounts[1] * 2 <= amounts[0] * 5,
            "{shape}: {} nodes charged {} {charge} but {} nodes charged {}: more than 2.5 times",
            pair[0],
            amounts[0],
            pair[1],
            amounts[1]
        );
    }
}
#[test]
fn charged_work_grows_linearly_with_the_number_of_imported_nodes() {
    assert_charge_is_linear(
        "wide",
        "work",
        wide_fragment,
        HtmlConstructionCaptureUsage::work,
    );
}
#[test]
fn charged_work_stays_linear_when_every_container_has_children() {
    assert_charge_is_linear(
        "paired",
        "work",
        paired_fragment,
        HtmlConstructionCaptureUsage::work,
    );
}
#[test]
fn charged_work_stays_linear_for_a_deep_chain() {
    assert_charge_is_linear(
        "chain",
        "work",
        chain_fragment,
        HtmlConstructionCaptureUsage::work,
    );
}
#[test]
fn charged_work_stays_linear_through_nested_template_contents() {
    assert_charge_is_linear(
        "nested templates",
        "work",
        nested_templates,
        HtmlConstructionCaptureUsage::work,
    );
}
#[test]
fn charged_work_stays_linear_for_templates_below_a_deep_chain() {
    assert_charge_is_linear(
        "chain with templates",
        "work",
        chain_with_templates,
        HtmlConstructionCaptureUsage::work,
    );
}
/// The work charged per imported node of a deeply nested source, at two sizes
/// eight times apart: finding the root below which an edge attaches stays a few
/// steps however deep the nesting, so the charge per node does not grow.
fn assert_work_per_node_is_constant(
    shape: &str,
    measure: impl Fn(usize) -> HtmlConstructionCaptureUsage,
    nodes: impl Fn(usize) -> usize,
) {
    let (small, large) = (200, 1600);
    let (small_work, large_work) = (measure(small).work(), measure(large).work());
    let (small_nodes, large_nodes) = (nodes(small), nodes(large));
    // Compare work / nodes of the two sizes without division, with 5% of slack
    // for the amortized moves of growing arrays.
    assert!(
        large_work * small_nodes * 100 <= small_work * large_nodes * 105,
        "{shape}: {small_work} work for {small_nodes} nodes but {large_work} for {large_nodes}: \
         the charge per node grew by more than 5%"
    );
}
#[test]
fn deeply_nested_sources_charge_constant_work_per_node() {
    assert_work_per_node_is_constant("chain", chain_fragment, |size| size + 1);
    assert_work_per_node_is_constant("nested templates", nested_templates, |size| 2 * size + 1);
    assert_work_per_node_is_constant("nested shadows", nested_shadows, |size| 2 * size + 1);
    assert_work_per_node_is_constant("chain with templates", chain_with_templates, |size| {
        1 + 4 * size
    });
}
#[test]
fn peak_scratch_grows_linearly_with_the_number_of_imported_nodes() {
    assert_charge_is_linear(
        "wide",
        "scratch entries",
        wide_fragment,
        HtmlConstructionCaptureUsage::scratch_entries,
    );
    assert_charge_is_linear(
        "paired",
        "scratch entries",
        paired_fragment,
        HtmlConstructionCaptureUsage::scratch_entries,
    );
    assert_charge_is_linear(
        "chain",
        "scratch entries",
        chain_fragment,
        HtmlConstructionCaptureUsage::scratch_entries,
    );
    assert_charge_is_linear(
        "nested templates",
        "scratch entries",
        nested_templates,
        HtmlConstructionCaptureUsage::scratch_entries,
    );
    assert_charge_is_linear(
        "chain with templates",
        "scratch entries",
        chain_with_templates,
        HtmlConstructionCaptureUsage::scratch_entries,
    );
}
#[test]
fn a_fragment_of_childless_siblings_peaks_at_seven_scratch_entries_per_sibling() {
    // The figure `docs/reference/html-crate-contracts.md` states for the
    // adapter's scratch: seven per sibling plus two, measured here.
    for siblings in [10, 100, 1000] {
        assert_eq!(
            wide_fragment(siblings).scratch_entries(),
            7 * siblings + 2,
            "{siblings} siblings"
        );
    }
}
#[test]
fn reported_usage_of_a_wide_import_is_its_least_admitting_allowance() {
    let children = 100;
    let mut native = fragment_builder(children + 1, 1);
    let root = native.root();
    for _ in 0..children {
        native
            .append_element(&root, name("p"), OrderedAttributes::default())
            .unwrap();
    }
    let source = native.finish().unwrap();
    let nodes = children + 1;
    let usage = usage_of_import(source.clone(), nodes, 1);
    let exact = HtmlConstructionCaptureLimits::new(
        usage.mapping_entries(),
        usage.scratch_entries(),
        usage.work(),
    );
    let imported = import_all_nodes(&source, nodes, 1, exact).unwrap();
    assert_eq!(imported.usage(), usage);
    // Each allowance is individually exact: one less is refused with the
    // original builder, which then still completes an unconstrained import.
    for refused in [
        HtmlConstructionCaptureLimits::new(exact.mapping_entries() - 1, 1 << 24, 1 << 40),
        HtmlConstructionCaptureLimits::new(nodes + 8, exact.scratch_entries() - 1, 1 << 40),
        HtmlConstructionCaptureLimits::new(nodes + 8, 1 << 24, exact.work() - 1),
    ] {
        let failure = import_all_nodes(&source, nodes, 1, refused).unwrap_err();
        assert_eq!(
            failure.reason(),
            &HtmlConstructionCaptureError::LimitExceeded
        );
        assert!(
            failure.usage().mapping_entries() <= refused.mapping_entries()
                && failure.usage().scratch_entries() <= refused.scratch_entries()
                && failure.usage().work() <= refused.work()
        );
        assert_eq!(failure.builder().usage().subjects(), 1);
    }
}
