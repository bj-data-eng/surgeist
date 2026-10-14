use surgeist_html::*;

fn captured_shadow_properties() -> HtmlCapturedShadowProperties {
    HtmlCapturedShadowProperties::new(
        HtmlShadowRootProperties::new(
            HtmlShadowRootMode::Closed,
            HtmlSlotAssignment::Manual,
            true,
            false,
            true,
        ),
        true,
        false,
        true,
    )
}

#[test]
fn unhosted_shadow_fragments_preserve_properties_and_unknown_host() {
    let mut builder = HtmlCaptureBuilder::new(limits(), HtmlHostFactsRevision::fresh());
    let document = document(&mut builder, HtmlScriptingStatus::Disabled);
    let registry = builder.add_registry(None).unwrap();
    let properties = captured_shadow_properties();
    let shadow = builder
        .add_shadow_fragment(
            &document,
            properties.clone(),
            HtmlCaptureRegistryInput::Unavailable,
        )
        .unwrap();
    let plain = builder.add_fragment(&document).unwrap();
    let registered_shadow = builder
        .add_shadow_fragment(
            &document,
            captured_shadow_properties(),
            HtmlCaptureRegistryInput::Registry(registry.clone()),
        )
        .unwrap();
    let before = builder.usage();
    assert_eq!(
        builder.establish_unhosted(&shadow),
        Err(HtmlCaptureError::ShadowRootCannotBeUnhosted)
    );
    assert_eq!(builder.usage(), before);
    builder.establish_unhosted(&plain).unwrap();
    let capture = builder.finish().unwrap();
    let shadow_id = capture.subject_id(&shadow).unwrap();
    let node = capture.node(&shadow_id).unwrap();
    let HtmlCapturedNodeKind::Fragment(fragment) = node.kind() else {
        panic!("fragment")
    };
    assert_eq!(
        fragment.kind(),
        &HtmlCapturedFragmentKind::ShadowRoot(properties)
    );
    assert_eq!(node.host(), Some(HtmlCapturedHost::Unavailable));
    assert_eq!(node.lookup_registry(), HtmlCapturedRegistry::Unavailable);
    assert_eq!(node.parent(), HtmlCapturedParent::Root);
    assert!(matches!(node.children(), HtmlCapturedChildren::Unavailable));
    assert_eq!(
        capture
            .node(&capture.subject_id(&registered_shadow).unwrap())
            .unwrap()
            .lookup_registry(),
        HtmlCapturedRegistry::Registry(capture.registry_id(&registry).unwrap())
    );
    assert_eq!(
        capture
            .node(&capture.subject_id(&plain).unwrap())
            .unwrap()
            .host(),
        Some(HtmlCapturedHost::Absent)
    );
}

#[test]
fn shadow_fragment_refusals_preserve_builder_and_validate_document_before_registry() {
    let mut foreign = HtmlCaptureBuilder::new(limits(), HtmlHostFactsRevision::fresh());
    let foreign_document = document(&mut foreign, HtmlScriptingStatus::Disabled);
    let foreign_registry = foreign.add_registry(None).unwrap();
    let mut builder = HtmlCaptureBuilder::new(
        HtmlCaptureLimits::new(4, 1, 0, 0, 0, 0, 0),
        HtmlHostFactsRevision::fresh(),
    );
    let doc = document(&mut builder, HtmlScriptingStatus::Disabled);
    let fragment = builder.add_fragment(&doc).unwrap();
    let before = builder.usage();
    assert_eq!(
        builder.add_shadow_fragment(
            &foreign_document,
            captured_shadow_properties(),
            HtmlCaptureRegistryInput::Null
        ),
        Err(HtmlCaptureError::ForeignBuilder)
    );
    assert_eq!(
        builder.add_shadow_fragment(
            &fragment,
            captured_shadow_properties(),
            HtmlCaptureRegistryInput::Registry(foreign_registry.clone())
        ),
        Err(HtmlCaptureError::WrongKind {
            expected: HtmlCaptureSubjectKind::Document,
            actual: HtmlCaptureSubjectKind::Fragment
        })
    );
    assert_eq!(
        builder.add_shadow_fragment(
            &doc,
            captured_shadow_properties(),
            HtmlCaptureRegistryInput::Registry(foreign_registry)
        ),
        Err(HtmlCaptureError::ForeignBuilder)
    );
    assert_eq!(
        builder.add_shadow_fragment(
            &doc,
            captured_shadow_properties(),
            HtmlCaptureRegistryInput::Null
        ),
        Err(HtmlCaptureError::LimitExceeded {
            resource: HtmlCaptureResource::Relationships,
            limit: 1,
            required: 2
        })
    );
    assert_eq!(builder.usage(), before);
    let capture = builder.finish().unwrap();
    assert_eq!(capture.usage().subjects(), 2);
    assert_eq!(capture.usage().relationships(), 1);
    assert_eq!(
        capture
            .node(&capture.subject_id(&fragment).unwrap())
            .unwrap()
            .host(),
        Some(HtmlCapturedHost::Unavailable)
    );
}

#[test]
fn shadow_reference_cycles_do_not_become_owning_edges_or_depth() {
    let mut builder = HtmlCaptureBuilder::new(
        HtmlCaptureLimits::new(5, 12, 0, 128, 1, 0, 0),
        HtmlHostFactsRevision::fresh(),
    );
    let doc = document(&mut builder, HtmlScriptingStatus::Disabled);
    let first = builder
        .add_shadow_fragment(
            &doc,
            captured_shadow_properties(),
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    let second = builder
        .add_shadow_fragment(
            &doc,
            captured_shadow_properties(),
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    let first_template = element(&mut builder, &doc, Namespace::html(), "template");
    let second_template = element(&mut builder, &doc, Namespace::html(), "template");
    builder
        .establish_children(&first, std::slice::from_ref(&first_template))
        .unwrap();
    builder
        .establish_children(&second, std::slice::from_ref(&second_template))
        .unwrap();
    builder
        .establish_template_shadow_reference(&first_template, &second)
        .unwrap();
    builder
        .establish_template_shadow_reference(&second_template, &first)
        .unwrap();
    let capture = builder.finish().unwrap();
    assert_eq!(capture.usage().relationships(), 12);
    assert_eq!(capture.usage().depth(), 1);
    for (template, target, parent) in [
        (&first_template, &second, &first),
        (&second_template, &first, &second),
    ] {
        let node = capture
            .node(&capture.subject_id(template).unwrap())
            .unwrap();
        assert_eq!(
            node.template_contents_kind(),
            Some(HtmlTemplateContentsKind::ShadowReference)
        );
        assert!(
            matches!(node.template_contents(), Some(HtmlCapturedTemplateContents::Present(id)) if id.subject() == &capture.subject_id(target).unwrap())
        );
        assert_eq!(
            node.parent(),
            HtmlCapturedParent::Subject(capture.subject_id(parent).unwrap())
        );
        let target = capture.node(&capture.subject_id(target).unwrap()).unwrap();
        assert_eq!(target.host(), Some(HtmlCapturedHost::Unavailable));
        assert_eq!(target.lookup_registry(), HtmlCapturedRegistry::Null);
        assert_eq!(target.parent(), HtmlCapturedParent::Root);
    }
}

#[test]
fn shadow_reference_preserves_existing_host_and_registry() {
    let mut builder = HtmlCaptureBuilder::new(limits(), HtmlHostFactsRevision::fresh());
    let doc = document(&mut builder, HtmlScriptingStatus::Disabled);
    let registry = builder.add_registry(None).unwrap();
    let host = element(&mut builder, &doc, Namespace::html(), "div");
    let template = element(&mut builder, &doc, Namespace::html(), "template");
    let shadow = builder
        .add_shadow_root(
            &host,
            captured_shadow_properties(),
            HtmlCaptureRegistryInput::Registry(registry.clone()),
        )
        .unwrap();
    let before = builder.usage();
    builder
        .establish_template_shadow_reference(&template, &shadow)
        .unwrap();
    assert_eq!(builder.usage().relationships(), before.relationships() + 1);
    assert_eq!(builder.usage().depth(), before.depth());
    assert_eq!(
        builder.establish_template_shadow_reference(&template, &shadow),
        Err(HtmlCaptureError::AssociationAlreadyEstablished)
    );
    let capture = builder.finish().unwrap();
    let host_id = capture
        .element_id(&capture.subject_id(&host).unwrap())
        .unwrap();
    let shadow_node = capture.node(&capture.subject_id(&shadow).unwrap()).unwrap();
    assert_eq!(
        shadow_node.host(),
        Some(HtmlCapturedHost::Element(host_id.clone()))
    );
    assert_eq!(
        shadow_node.lookup_registry(),
        HtmlCapturedRegistry::Registry(capture.registry_id(&registry).unwrap())
    );
    assert!(
        matches!(capture.node(host_id.subject()).unwrap().shadow_root(), Some(HtmlCapturedShadowRoot::Present(id)) if id.subject() == &capture.subject_id(&shadow).unwrap())
    );
}

#[test]
fn shadow_reference_foreign_kind_target_and_budget_refusals_are_atomic() {
    let mut foreign = HtmlCaptureBuilder::new(limits(), HtmlHostFactsRevision::fresh());
    let foreign_doc = document(&mut foreign, HtmlScriptingStatus::Disabled);
    let foreign_fragment = foreign.add_fragment(&foreign_doc).unwrap();
    let mut builder = HtmlCaptureBuilder::new(
        HtmlCaptureLimits::new(5, 4, 0, 128, 2, 0, 0),
        HtmlHostFactsRevision::fresh(),
    );
    let doc = document(&mut builder, HtmlScriptingStatus::Disabled);
    let template = element(&mut builder, &doc, Namespace::html(), "template");
    let div = element(&mut builder, &doc, Namespace::html(), "div");
    let plain = builder.add_fragment(&doc).unwrap();
    let shadow = builder
        .add_shadow_fragment(
            &doc,
            captured_shadow_properties(),
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    let before = builder.usage();
    assert_eq!(
        builder.establish_template_shadow_reference(&foreign_doc, &plain),
        Err(HtmlCaptureError::ForeignBuilder)
    );
    assert_eq!(
        builder.establish_template_shadow_reference(&template, &foreign_fragment),
        Err(HtmlCaptureError::ForeignBuilder)
    );
    assert_eq!(
        builder.establish_template_shadow_reference(&template, &doc),
        Err(HtmlCaptureError::WrongKind {
            expected: HtmlCaptureSubjectKind::Fragment,
            actual: HtmlCaptureSubjectKind::Document
        })
    );
    assert_eq!(
        builder.establish_template_shadow_reference(&doc, &shadow),
        Err(HtmlCaptureError::WrongKind {
            expected: HtmlCaptureSubjectKind::Element,
            actual: HtmlCaptureSubjectKind::Document,
        })
    );
    assert_eq!(
        builder.establish_template_shadow_reference(&div, &shadow),
        Err(HtmlCaptureError::NotHtmlTemplate)
    );
    assert_eq!(
        builder.establish_template_shadow_reference(&template, &plain),
        Err(HtmlCaptureError::InvalidTemplateContentsReference)
    );
    assert_eq!(
        builder.establish_template_shadow_reference(&template, &shadow),
        Err(HtmlCaptureError::LimitExceeded {
            resource: HtmlCaptureResource::Relationships,
            limit: 4,
            required: 5
        })
    );
    assert_eq!(builder.usage(), before);
    let capture = builder.finish().unwrap();
    let node = capture
        .node(&capture.subject_id(&template).unwrap())
        .unwrap();
    assert_eq!(node.template_contents_kind(), None);
    assert_eq!(
        node.template_contents(),
        Some(HtmlCapturedTemplateContents::Unavailable)
    );
}

#[test]
fn owned_template_contents_stay_reciprocal_and_reject_shadow_contents() {
    let mut builder = HtmlCaptureBuilder::new(limits(), HtmlHostFactsRevision::fresh());
    let doc = document(&mut builder, HtmlScriptingStatus::Disabled);
    let template = element(&mut builder, &doc, Namespace::html(), "template");
    let shadow = builder
        .add_shadow_fragment(
            &doc,
            captured_shadow_properties(),
            HtmlCaptureRegistryInput::Unavailable,
        )
        .unwrap();
    let plain = builder.add_fragment(&doc).unwrap();
    let before = builder.usage();
    assert_eq!(
        builder.establish_template_contents(&template, &shadow),
        Err(HtmlCaptureError::InvalidTemplateContentsReference)
    );
    assert_eq!(builder.usage(), before);
    builder
        .establish_template_contents(&template, &plain)
        .unwrap();
    assert_eq!(
        builder.establish_template_shadow_reference(&template, &shadow),
        Err(HtmlCaptureError::AssociationAlreadyEstablished)
    );
    let capture = builder.finish().unwrap();
    let template_id = capture.subject_id(&template).unwrap();
    let node = capture.node(&template_id).unwrap();
    assert_eq!(
        node.template_contents_kind(),
        Some(HtmlTemplateContentsKind::Owned)
    );
    let plain_node = capture.node(&capture.subject_id(&plain).unwrap()).unwrap();
    assert_eq!(
        plain_node.host(),
        Some(HtmlCapturedHost::Element(
            capture.element_id(&template_id).unwrap()
        ))
    );
}

#[test]
fn template_owner_facts_retain_explicit_class_and_one_qualified_cache_relationship() {
    let mut builder = HtmlCaptureBuilder::new(limits(), HtmlHostFactsRevision::fresh());
    let ordinary = document(&mut builder, HtmlScriptingStatus::Disabled);
    let inert = document(&mut builder, HtmlScriptingStatus::Disabled);
    builder
        .establish_document_template_facts(
            &inert,
            HtmlDocumentClass::NonHtml,
            HtmlTemplateDocumentRole::InertTemplate,
            HtmlCaptureTemplateCacheInput::Unavailable,
        )
        .unwrap();
    builder
        .establish_document_template_facts(
            &ordinary,
            HtmlDocumentClass::NonHtml,
            HtmlTemplateDocumentRole::Ordinary,
            HtmlCaptureTemplateCacheInput::Document(inert.clone()),
        )
        .unwrap();
    assert_eq!(
        builder.establish_document_template_facts(
            &ordinary,
            HtmlDocumentClass::Html,
            HtmlTemplateDocumentRole::Ordinary,
            HtmlCaptureTemplateCacheInput::Absent
        ),
        Err(HtmlCaptureError::DocumentFactsAlreadyEstablished)
    );
    let capture = builder.finish().unwrap();
    let ordinary = capture.subject_id(&ordinary).unwrap();
    let inert = capture
        .document_id(&capture.subject_id(&inert).unwrap())
        .unwrap();
    let node = capture.node(&ordinary).unwrap();
    let HtmlCapturedNodeKind::Document(facts) = node.kind() else {
        panic!("actual document")
    };
    assert_eq!(facts.document_class(), HtmlDocumentClass::NonHtml);
    assert_eq!(facts.template_role(), HtmlTemplateDocumentRole::Ordinary);
    assert_eq!(
        node.template_document_cache(),
        Some(HtmlCapturedTemplateCache::Document(inert))
    );
    assert_eq!(capture.usage().relationships(), 1);
}

#[test]
fn template_cache_rejects_foreign_ordinary_and_one_short_relationship_before_commit() {
    let mut foreign = HtmlCaptureBuilder::new(limits(), HtmlHostFactsRevision::fresh());
    let foreign_document = document(&mut foreign, HtmlScriptingStatus::Disabled);
    let mut builder = HtmlCaptureBuilder::new(
        HtmlCaptureLimits::new(3, 0, 0, 0, 0, 0, 0),
        HtmlHostFactsRevision::fresh(),
    );
    let ordinary = document(&mut builder, HtmlScriptingStatus::Disabled);
    let inert = document(&mut builder, HtmlScriptingStatus::Disabled);
    assert_eq!(
        builder.establish_document_template_facts(
            &ordinary,
            HtmlDocumentClass::Html,
            HtmlTemplateDocumentRole::Ordinary,
            HtmlCaptureTemplateCacheInput::Document(foreign_document)
        ),
        Err(HtmlCaptureError::ForeignBuilder)
    );
    assert_eq!(
        builder.establish_document_template_facts(
            &ordinary,
            HtmlDocumentClass::Html,
            HtmlTemplateDocumentRole::Ordinary,
            HtmlCaptureTemplateCacheInput::Document(inert.clone())
        ),
        Err(HtmlCaptureError::InvalidTemplateDocumentCache)
    );
    builder
        .establish_document_template_facts(
            &inert,
            HtmlDocumentClass::Html,
            HtmlTemplateDocumentRole::InertTemplate,
            HtmlCaptureTemplateCacheInput::Absent,
        )
        .unwrap();
    assert_eq!(
        builder.establish_document_template_facts(
            &ordinary,
            HtmlDocumentClass::Html,
            HtmlTemplateDocumentRole::Ordinary,
            HtmlCaptureTemplateCacheInput::Document(inert)
        ),
        Err(HtmlCaptureError::LimitExceeded {
            resource: HtmlCaptureResource::Relationships,
            limit: 0,
            required: 1
        })
    );
    let capture = builder.finish().unwrap();
    let node = capture
        .node(&capture.subject_id(&ordinary).unwrap())
        .unwrap();
    let HtmlCapturedNodeKind::Document(facts) = node.kind() else {
        panic!("actual document")
    };
    assert_eq!(facts.template_role(), HtmlTemplateDocumentRole::Unavailable);
    assert_eq!(
        node.template_document_cache(),
        Some(HtmlCapturedTemplateCache::Unavailable)
    );
    assert_eq!(capture.usage().relationships(), 0);
}

#[test]
fn registry_scope_is_independent_of_definition_completeness_and_capture_identity() {
    let mut builder = HtmlCaptureBuilder::new(limits(), HtmlHostFactsRevision::fresh());
    let registry = builder.add_registry(None).unwrap();
    builder
        .establish_registry_scope(&registry, HtmlRegistryScope::Scoped)
        .unwrap();
    assert_eq!(
        builder.establish_registry_scope(&registry, HtmlRegistryScope::Global),
        Err(HtmlCaptureError::RegistryScopeAlreadyEstablished)
    );
    let capture = builder.finish().unwrap();
    let registry = capture.registry_id(&registry).unwrap();
    assert_eq!(
        capture.registry_scope(&registry),
        Ok(HtmlRegistryScope::Scoped)
    );
    let foreign = HtmlCaptureBuilder::new(limits(), HtmlHostFactsRevision::fresh())
        .finish()
        .unwrap();
    assert_eq!(
        foreign.registry_scope(&registry),
        Err(HtmlCaptureError::ForeignCapture)
    );
}

fn limits() -> HtmlCaptureLimits {
    HtmlCaptureLimits::new(10_000, 100_000, 1_000, 1_000_000, 10_000, 100, 1_000)
}
fn name(namespace: Namespace, local: &str) -> QualifiedName {
    QualifiedName::new(namespace, None, DocumentString::from(local))
}
fn attrs(local: &str, value: &str) -> OrderedAttributes {
    OrderedAttributes::new([Attribute::new(name(Namespace::none(), local), value)]).unwrap()
}
fn document(
    b: &mut HtmlCaptureBuilder,
    scripting: HtmlScriptingStatus,
) -> HtmlCaptureBuilderSubject {
    b.add_document(
        HtmlDocumentMode::NoQuirks,
        scripting,
        HtmlCaptureRegistryInput::Null,
    )
    .unwrap()
}
fn element(
    b: &mut HtmlCaptureBuilder,
    d: &HtmlCaptureBuilderSubject,
    ns: Namespace,
    local: &str,
) -> HtmlCaptureBuilderSubject {
    b.add_element(
        d,
        name(ns, local),
        OrderedAttributes::default(),
        HtmlCapturedIsValue::Absent,
        HtmlCaptureRegistryInput::Null,
    )
    .unwrap()
}
fn context(
    ns: Namespace,
    local: &str,
    attributes: OrderedAttributes,
    scripting: HtmlScriptingStatus,
) -> HtmlFragmentContext {
    let mut b = HtmlCaptureBuilder::new(limits(), HtmlHostFactsRevision::fresh());
    let d = document(&mut b, scripting);
    let e = b
        .add_element(
            &d,
            name(ns, local),
            attributes,
            HtmlCapturedIsValue::Absent,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    b.establish_root(&e).unwrap();
    let c = b.finish().unwrap();
    HtmlFragmentContext::element(&c, &c.subject_id(&e).unwrap()).unwrap()
}
fn children(c: &HtmlCapture, id: &HtmlCaptureSubjectId) -> Vec<HtmlCaptureSubjectId> {
    match c.node(id).unwrap().children() {
        HtmlCapturedChildren::Complete(ids) => ids.collect(),
        HtmlCapturedChildren::Unavailable => panic!("children unavailable"),
    }
}

#[test]
fn exact_slots_order_and_all_retained_reference_budgets() {
    let mut b = HtmlCaptureBuilder::new(
        HtmlCaptureLimits::new(6, 22, 1, 25, 2, 1, 1),
        HtmlHostFactsRevision::fresh(),
    );
    let registry = b
        .add_registry(Some(vec![HtmlCustomElementDefinition::new(
            "x-a".into(),
            "e".into(),
            false,
            false,
        )]))
        .unwrap();
    let d = b
        .add_document(
            HtmlDocumentMode::Quirks,
            HtmlScriptingStatus::Unavailable,
            HtmlCaptureRegistryInput::Registry(registry.clone()),
        )
        .unwrap();
    let q = QualifiedName::new(Namespace::from_uri("u"), Some("p".into()), "e");
    let raw = DocumentString::from_utf16([0xd800, 0, 0xdc00].as_slice());
    let a = Attribute::new(
        QualifiedName::new(Namespace::from_uri("v"), Some("q".into()), "a"),
        raw.clone(),
    );
    let e = b
        .add_element(
            &d,
            q.clone(),
            OrderedAttributes::new([a.clone()]).unwrap(),
            HtmlCapturedIsValue::Value("xy".into()),
            HtmlCaptureRegistryInput::Registry(registry),
        )
        .unwrap();
    let t = b
        .add_text(&d, DocumentString::from_utf16([0xd801, 0xdc00].as_slice()))
        .unwrap();
    let comment = b.add_comment(&d, "c".into()).unwrap();
    let pi = b
        .add_processing_instruction(
            &d,
            "x".into(),
            DocumentString::from_utf16([0xd800, 0].as_slice()),
        )
        .unwrap();
    let dt = b
        .add_doctype(
            &d,
            HtmlDoctype::new("n", Some("ab".into()), Some("c".into())),
        )
        .unwrap();
    b.establish_children(&d, &[dt.clone(), e.clone()]).unwrap();
    b.establish_children(&e, &[t.clone(), comment.clone(), pi.clone()])
        .unwrap();
    assert_eq!(
        (
            b.usage().subjects(),
            b.usage().relationships(),
            b.usage().attributes(),
            b.usage().utf16_units(),
            b.usage().depth(),
            b.usage().registries(),
            b.usage().definitions()
        ),
        (6, 22, 1, 25, 2, 1, 1)
    );
    let c = b.finish().unwrap();
    let eid = c.subject_id(&e).unwrap();
    let HtmlCapturedNodeKind::Element(values) = c.node(&eid).unwrap().kind() else {
        panic!()
    };
    assert_eq!(values.name(), &q);
    assert_eq!(values.attributes().as_slice(), &[a]);
    assert_eq!(
        values.attributes().as_slice()[0].value().code_units(),
        &[0xd800, 0, 0xdc00]
    );
    assert_eq!(values.is_value(), &HtmlCapturedIsValue::Value("xy".into()));
    assert_eq!(
        children(&c, &c.subject_id(&d).unwrap()),
        vec![c.subject_id(&dt).unwrap(), eid.clone()]
    );
    assert_eq!(
        children(&c, &eid),
        vec![
            c.subject_id(&t).unwrap(),
            c.subject_id(&comment).unwrap(),
            c.subject_id(&pi).unwrap()
        ]
    );
    assert!(
        matches!(c.node(&c.subject_id(&d).unwrap()).unwrap().kind(), HtmlCapturedNodeKind::Document(d) if d.mode() == HtmlDocumentMode::Quirks)
    );
    assert!(
        matches!(c.node(&c.subject_id(&pi).unwrap()).unwrap().kind(), HtmlCapturedNodeKind::ProcessingInstruction(p) if p.data().code_units() == [0xd800, 0])
    );
}

#[test]
fn capture_and_builder_identities_are_checked_and_survive_later_builds() {
    let revision = HtmlHostFactsRevision::fresh();
    let mut b = HtmlCaptureBuilder::new(limits(), revision.clone());
    let d = document(&mut b, HtmlScriptingStatus::Disabled);
    let e = element(&mut b, &d, Namespace::html(), "div");
    b.establish_root(&e).unwrap();
    let c = b.finish().unwrap();
    let id = c.subject_id(&e).unwrap();
    let saved = HtmlFragmentContext::element(&c, &id).unwrap();
    let mut other = HtmlCaptureBuilder::new(limits(), HtmlHostFactsRevision::fresh());
    let od = document(&mut other, HtmlScriptingStatus::Enabled);
    assert_eq!(
        other.establish_parent(&e, &od),
        Err(HtmlCaptureError::ForeignBuilder)
    );
    let oe = element(&mut other, &od, Namespace::html(), "div");
    other.establish_root(&oe).unwrap();
    let other = other.finish().unwrap();
    assert_eq!(
        other.node(&id).unwrap_err(),
        HtmlCaptureError::ForeignCapture
    );
    assert_eq!(other.subject_id(&e), Err(HtmlCaptureError::ForeignBuilder));
    assert_eq!(c.clone().identity(), c.identity());
    assert_ne!(c.identity(), other.identity());
    assert_eq!(c.host_facts_revision(), &revision);
    assert_ne!(c.host_facts_revision(), other.host_facts_revision());
    assert_eq!(
        saved
            .resolve_scripting_mode(HtmlParserScriptingMode::Inert)
            .unwrap(),
        HtmlParserScriptingMode::Disabled
    );
    assert_eq!(saved.context_element().subject(), &id);
    assert_eq!(
        c.document_id(&id).unwrap_err(),
        HtmlCaptureError::WrongKind {
            expected: HtmlCaptureSubjectKind::Document,
            actual: HtmlCaptureSubjectKind::Element
        }
    );
}

#[test]
fn hosted_template_context_keeps_distinct_documents_and_lookup_routes() {
    let mut b = HtmlCaptureBuilder::new(limits(), HtmlHostFactsRevision::fresh());
    let reg = b.add_registry(Some(Vec::new())).unwrap();
    let normal = b
        .add_document(
            HtmlDocumentMode::LimitedQuirks,
            HtmlScriptingStatus::Enabled,
            HtmlCaptureRegistryInput::Registry(reg.clone()),
        )
        .unwrap();
    let inert = document(&mut b, HtmlScriptingStatus::Disabled);
    let template = b
        .add_element(
            &normal,
            name(Namespace::html(), "template"),
            attrs("id", "t"),
            HtmlCapturedIsValue::Absent,
            HtmlCaptureRegistryInput::Registry(reg),
        )
        .unwrap();
    b.establish_root(&template).unwrap();
    let fragment = b.add_fragment(&inert).unwrap();
    b.establish_template_contents(&template, &fragment).unwrap();
    let c = b.finish().unwrap();
    let tid = c.subject_id(&template).unwrap();
    let fid = c.subject_id(&fragment).unwrap();
    let ctx = HtmlFragmentContext::hosted_fragment(&c, &fid).unwrap();
    assert_eq!(ctx.context_element().subject(), &tid);
    assert_ne!(ctx.target_document(), ctx.context_document());
    assert_eq!(ctx.document_mode(), HtmlDocumentMode::LimitedQuirks);
    assert_eq!(ctx.scripting_status(), HtmlScriptingStatus::Enabled);
    assert_eq!(ctx.root_registry(), &HtmlCapturedRegistry::Null);
    assert!(matches!(
        HtmlFragmentContext::element(&c, &tid)
            .unwrap()
            .root_registry(),
        HtmlCapturedRegistry::Registry(_)
    ));
    assert!(ctx.is_template());
    assert_eq!(
        ctx.initial_insertion_mode(),
        HtmlFragmentInsertionMode::InTemplate
    );
    assert!(matches!(
        c.node(&fid).unwrap().parent(),
        HtmlCapturedParent::Root
    ));
    assert!(
        matches!(c.node(&fid).unwrap().host(), Some(HtmlCapturedHost::Element(host)) if host.subject() == &tid)
    );
    assert!(
        matches!(c.node(&tid).unwrap().template_contents(), Some(HtmlCapturedTemplateContents::Present(f)) if f.subject() == &fid)
    );
    assert!(matches!(
        c.node(&tid).unwrap().children(),
        HtmlCapturedChildren::Unavailable
    ));
}

#[test]
fn fragment_host_absent_unavailable_and_wrong_kind_are_distinct() {
    let mut b = HtmlCaptureBuilder::new(limits(), HtmlHostFactsRevision::fresh());
    let d = document(&mut b, HtmlScriptingStatus::Disabled);
    let unknown = b.add_fragment(&d).unwrap();
    let absent = b.add_fragment(&d).unwrap();
    b.establish_unhosted(&absent).unwrap();
    let c = b.finish().unwrap();
    assert!(matches!(
        HtmlFragmentContext::hosted_fragment(&c, &c.subject_id(&unknown).unwrap()).unwrap_err(),
        HtmlCaptureError::MissingFact {
            fact: HtmlCaptureFact::FragmentHost,
            ..
        }
    ));
    assert_eq!(
        HtmlFragmentContext::hosted_fragment(&c, &c.subject_id(&absent).unwrap()).unwrap_err(),
        HtmlCaptureError::UnhostedFragment
    );
    assert_eq!(
        HtmlFragmentContext::element(&c, &c.subject_id(&absent).unwrap()).unwrap_err(),
        HtmlCaptureError::WrongKind {
            expected: HtmlCaptureSubjectKind::Element,
            actual: HtmlCaptureSubjectKind::Fragment
        }
    );
}

#[test]
fn nearest_form_requires_only_its_ordinary_dependency_closure() {
    let mut b = HtmlCaptureBuilder::new(limits(), HtmlHostFactsRevision::fresh());
    let d = document(&mut b, HtmlScriptingStatus::Disabled);
    let outer = element(&mut b, &d, Namespace::html(), "form");
    let inner = element(&mut b, &d, Namespace::html(), "form");
    let foreign = element(&mut b, &d, Namespace::svg(), "form");
    let target = element(&mut b, &d, Namespace::html(), "span");
    b.establish_parent(&inner, &outer).unwrap();
    b.establish_parent(&foreign, &inner).unwrap();
    b.establish_parent(&target, &foreign).unwrap();
    let unknown = element(&mut b, &d, Namespace::html(), "div");
    let known_absent = element(&mut b, &d, Namespace::html(), "div");
    b.establish_root(&known_absent).unwrap();
    let c = b.finish().unwrap();
    let inner_id = c.element_id(&c.subject_id(&inner).unwrap()).unwrap();
    assert_eq!(c.nearest_form(&inner_id).unwrap(), Some(inner_id.clone()));
    assert_eq!(
        HtmlFragmentContext::element(&c, &c.subject_id(&target).unwrap())
            .unwrap()
            .nearest_form(),
        Some(&inner_id)
    );
    assert!(matches!(
        HtmlFragmentContext::element(&c, &c.subject_id(&unknown).unwrap()).unwrap_err(),
        HtmlCaptureError::MissingFact {
            fact: HtmlCaptureFact::Parent,
            ..
        }
    ));
    assert!(
        HtmlFragmentContext::element(&c, &c.subject_id(&known_absent).unwrap())
            .unwrap()
            .nearest_form()
            .is_none()
    );
}

#[test]
fn host_associations_are_not_ordinary_form_ancestors() {
    let mut b = HtmlCaptureBuilder::new(limits(), HtmlHostFactsRevision::fresh());
    let d = document(&mut b, HtmlScriptingStatus::Disabled);
    let form = element(&mut b, &d, Namespace::html(), "form");
    let template = element(&mut b, &d, Namespace::html(), "template");
    b.establish_parent(&template, &form).unwrap();
    let f = b.add_fragment(&d).unwrap();
    b.establish_template_contents(&template, &f).unwrap();
    let target = element(&mut b, &d, Namespace::html(), "span");
    b.establish_parent(&target, &f).unwrap();
    let c = b.finish().unwrap();
    let ctx = HtmlFragmentContext::element(&c, &c.subject_id(&target).unwrap()).unwrap();
    assert!(ctx.nearest_form().is_none());
    assert!(!ctx.is_template());
    assert_eq!(
        ctx.initial_insertion_mode(),
        HtmlFragmentInsertionMode::InBody
    );
}

#[test]
fn scripting_status_mode_override_and_unavailable_execution_are_source_derived() {
    for status in [
        HtmlScriptingStatus::Disabled,
        HtmlScriptingStatus::Enabled,
        HtmlScriptingStatus::Unavailable,
    ] {
        let c = context(
            Namespace::html(),
            "noscript",
            OrderedAttributes::default(),
            status,
        );
        for wrong in [
            HtmlParserScriptingMode::Normal,
            HtmlParserScriptingMode::Disabled,
        ] {
            assert_eq!(
                c.resolve_scripting_mode(wrong),
                Err(HtmlCaptureError::InvalidFragmentScriptingMode(wrong))
            );
        }
        match status {
            HtmlScriptingStatus::Disabled => {
                for requested in [
                    HtmlParserScriptingMode::Inert,
                    HtmlParserScriptingMode::Fragment,
                ] {
                    assert_eq!(
                        c.resolve_scripting_mode(requested).unwrap(),
                        HtmlParserScriptingMode::Disabled
                    );
                    assert_eq!(
                        c.initial_tokenizer_state(requested).unwrap(),
                        HtmlFragmentTokenizerState::Data
                    );
                }
            }
            HtmlScriptingStatus::Enabled => {
                assert_eq!(
                    c.resolve_scripting_mode(HtmlParserScriptingMode::Inert)
                        .unwrap(),
                    HtmlParserScriptingMode::Inert
                );
                assert_eq!(
                    c.initial_tokenizer_state(HtmlParserScriptingMode::Inert)
                        .unwrap(),
                    HtmlFragmentTokenizerState::Rawtext
                );
                assert_eq!(
                    c.resolve_scripting_mode(HtmlParserScriptingMode::Fragment),
                    Err(HtmlCaptureError::UnsupportedExecution(
                        HtmlParserScriptingMode::Fragment
                    ))
                );
            }
            HtmlScriptingStatus::Unavailable => assert!(matches!(
                c.resolve_scripting_mode(HtmlParserScriptingMode::Inert),
                Err(HtmlCaptureError::MissingFact {
                    fact: HtmlCaptureFact::ScriptingStatus,
                    ..
                })
            )),
        }
    }
}

#[test]
fn tokenizer_context_names_have_exact_namespace_and_case_semantics() {
    for (local, expected) in [
        ("title", HtmlFragmentTokenizerState::Rcdata),
        ("textarea", HtmlFragmentTokenizerState::Rcdata),
        ("style", HtmlFragmentTokenizerState::Rawtext),
        ("xmp", HtmlFragmentTokenizerState::Rawtext),
        ("iframe", HtmlFragmentTokenizerState::Rawtext),
        ("noembed", HtmlFragmentTokenizerState::Rawtext),
        ("noframes", HtmlFragmentTokenizerState::Rawtext),
        ("script", HtmlFragmentTokenizerState::ScriptData),
        ("plaintext", HtmlFragmentTokenizerState::Plaintext),
        ("div", HtmlFragmentTokenizerState::Data),
        ("TEXTAREA", HtmlFragmentTokenizerState::Data),
    ] {
        assert_eq!(
            context(
                Namespace::html(),
                local,
                OrderedAttributes::default(),
                HtmlScriptingStatus::Disabled
            )
            .initial_tokenizer_state(HtmlParserScriptingMode::Inert)
            .unwrap(),
            expected
        );
        assert_eq!(
            context(
                Namespace::svg(),
                local,
                OrderedAttributes::default(),
                HtmlScriptingStatus::Disabled
            )
            .initial_tokenizer_state(HtmlParserScriptingMode::Inert)
            .unwrap(),
            HtmlFragmentTokenizerState::Data
        );
    }
}

#[test]
fn initial_insertion_modes_use_context_not_captured_ancestor_stack() {
    for (local, expected) in [
        ("tr", HtmlFragmentInsertionMode::InRow),
        ("thead", HtmlFragmentInsertionMode::InTableBody),
        ("tbody", HtmlFragmentInsertionMode::InTableBody),
        ("tfoot", HtmlFragmentInsertionMode::InTableBody),
        ("caption", HtmlFragmentInsertionMode::InCaption),
        ("colgroup", HtmlFragmentInsertionMode::InColumnGroup),
        ("table", HtmlFragmentInsertionMode::InTable),
        ("template", HtmlFragmentInsertionMode::InTemplate),
        ("frameset", HtmlFragmentInsertionMode::InFrameset),
        ("html", HtmlFragmentInsertionMode::BeforeHead),
        ("td", HtmlFragmentInsertionMode::InBody),
        ("th", HtmlFragmentInsertionMode::InBody),
        ("head", HtmlFragmentInsertionMode::InBody),
    ] {
        assert_eq!(
            context(
                Namespace::html(),
                local,
                OrderedAttributes::default(),
                HtmlScriptingStatus::Disabled
            )
            .initial_insertion_mode(),
            expected
        );
        assert_eq!(
            context(
                Namespace::mathml(),
                local,
                OrderedAttributes::default(),
                HtmlScriptingStatus::Disabled
            )
            .initial_insertion_mode(),
            HtmlFragmentInsertionMode::InBody
        );
    }
}

#[test]
fn foreign_integration_derives_fake_token_from_exact_captured_attributes() {
    for local in ["mi", "mo", "mn", "ms", "mtext"] {
        assert!(
            context(
                Namespace::mathml(),
                local,
                OrderedAttributes::default(),
                HtmlScriptingStatus::Disabled
            )
            .is_mathml_text_integration_point()
        );
        assert!(
            !context(
                Namespace::html(),
                local,
                OrderedAttributes::default(),
                HtmlScriptingStatus::Disabled
            )
            .is_mathml_text_integration_point()
        );
    }
    for value in ["text/html", "TEXT/HTML", "Application/XhTmL+XmL"] {
        let a = attrs("encoding", value);
        let c = context(
            Namespace::mathml(),
            "annotation-xml",
            a.clone(),
            HtmlScriptingStatus::Disabled,
        );
        assert!(c.is_html_integration_point());
        let HtmlCapturedNodeKind::Element(e) = c
            .capture()
            .node(c.context_element().subject())
            .unwrap()
            .kind()
        else {
            panic!()
        };
        assert_eq!(e.attributes(), &a);
    }
    for value in [" text/html", "text/html ", "text/xml", ""] {
        assert!(
            !context(
                Namespace::mathml(),
                "annotation-xml",
                attrs("encoding", value),
                HtmlScriptingStatus::Disabled
            )
            .is_html_integration_point()
        );
    }
    let foreign_attr = OrderedAttributes::new([Attribute::new(
        name(Namespace::svg(), "encoding"),
        "text/html",
    )])
    .unwrap();
    assert!(
        !context(
            Namespace::mathml(),
            "annotation-xml",
            foreign_attr,
            HtmlScriptingStatus::Disabled
        )
        .is_html_integration_point()
    );
    for local in ["foreignObject", "desc", "title"] {
        assert!(
            context(
                Namespace::svg(),
                local,
                OrderedAttributes::default(),
                HtmlScriptingStatus::Disabled
            )
            .is_html_integration_point()
        );
    }
    assert!(
        !context(
            Namespace::svg(),
            "foreignobject",
            OrderedAttributes::default(),
            HtmlScriptingStatus::Disabled
        )
        .is_html_integration_point()
    );
}

#[test]
fn registry_complete_empty_null_unavailable_and_foreign_are_distinct() {
    let mut b = HtmlCaptureBuilder::new(limits(), HtmlHostFactsRevision::fresh());
    let empty = b.add_registry(Some(Vec::new())).unwrap();
    let missing = b.add_registry(None).unwrap();
    let d = document(&mut b, HtmlScriptingStatus::Disabled);
    let e = b
        .add_element(
            &d,
            name(Namespace::html(), "div"),
            OrderedAttributes::default(),
            HtmlCapturedIsValue::Absent,
            HtmlCaptureRegistryInput::Registry(empty.clone()),
        )
        .unwrap();
    b.establish_root(&e).unwrap();
    let c = b.finish().unwrap();
    let empty_id = c.registry_id(&empty).unwrap();
    let missing_id = c.registry_id(&missing).unwrap();
    assert!(
        matches!(c.definitions(&empty_id).unwrap(), HtmlCapturedDefinitions::Complete(d) if d.is_empty())
    );
    assert!(matches!(
        c.definitions(&missing_id).unwrap(),
        HtmlCapturedDefinitions::Unavailable
    ));
    let captured = HtmlCapturedRegistry::Registry(empty_id.clone());
    let ctx = HtmlFragmentContext::element(&c, &c.subject_id(&e).unwrap()).unwrap();
    assert_eq!(ctx.root_registry(), &captured);
    assert!(
        c.lookup_definition(&captured, &Namespace::html(), &"x-a".into(), None)
            .unwrap()
            .is_none()
    );
    assert!(
        c.lookup_definition(
            &HtmlCapturedRegistry::Null,
            &Namespace::html(),
            &"x-a".into(),
            None
        )
        .unwrap()
        .is_none()
    );
    assert_eq!(
        c.lookup_definition(
            &HtmlCapturedRegistry::Registry(missing_id.clone()),
            &Namespace::html(),
            &"x-a".into(),
            None
        ),
        Err(HtmlCaptureError::UnavailableDefinitions(missing_id))
    );
    assert!(matches!(
        c.lookup_definition(
            &HtmlCapturedRegistry::Unavailable,
            &Namespace::html(),
            &"x-a".into(),
            None
        ),
        Err(HtmlCaptureError::MissingFact {
            fact: HtmlCaptureFact::Registry,
            ..
        })
    ));
    assert!(
        c.lookup_definition(
            &HtmlCapturedRegistry::Unavailable,
            &Namespace::svg(),
            &"x-a".into(),
            None
        )
        .unwrap()
        .is_none()
    );
    let other = HtmlCaptureBuilder::new(limits(), HtmlHostFactsRevision::fresh())
        .finish()
        .unwrap();
    assert_eq!(
        other.definitions(&empty_id).unwrap_err(),
        HtmlCaptureError::ForeignCapture
    );
    assert_eq!(
        other.lookup_definition(&captured, &Namespace::html(), &"x-a".into(), None),
        Err(HtmlCaptureError::ForeignCapture)
    );
}

#[test]
fn registry_lookup_is_exact_and_autonomous_match_precedes_is_match() {
    let mut b = HtmlCaptureBuilder::new(limits(), HtmlHostFactsRevision::fresh());
    let reg = b
        .add_registry(Some(vec![
            HtmlCustomElementDefinition::new("x-host".into(), "x-host".into(), true, true),
            HtmlCustomElementDefinition::new("x-other".into(), "x-host".into(), false, false),
            HtmlCustomElementDefinition::new("x-div".into(), "div".into(), false, true),
        ]))
        .unwrap();
    let c = b.finish().unwrap();
    let reg = HtmlCapturedRegistry::Registry(c.registry_id(&reg).unwrap());
    let d = c
        .lookup_definition(
            &reg,
            &Namespace::html(),
            &"x-host".into(),
            Some(&"x-other".into()),
        )
        .unwrap()
        .unwrap();
    assert_eq!(d.name(), &DocumentString::from("x-host"));
    assert!(d.form_associated());
    assert!(d.disable_shadow());
    let d = c
        .lookup_definition(
            &reg,
            &Namespace::html(),
            &"div".into(),
            Some(&"x-div".into()),
        )
        .unwrap()
        .unwrap();
    assert_eq!(d.name(), &DocumentString::from("x-div"));
    for (ns, local, is_value) in [
        (Namespace::html(), "DIV", Some("x-div")),
        (Namespace::html(), "div", Some("X-DIV")),
        (Namespace::html(), "div", None),
        (Namespace::mathml(), "div", Some("x-div")),
    ] {
        let is_value = is_value.map(DocumentString::from);
        assert!(
            c.lookup_definition(&reg, &ns, &local.into(), is_value.as_ref())
                .unwrap()
                .is_none()
        );
    }
}

#[test]
fn shadow_status_uses_internal_is_slot_and_definition_without_execution() {
    let mut b = HtmlCaptureBuilder::new(limits(), HtmlHostFactsRevision::fresh());
    let reg = b
        .add_registry(Some(vec![HtmlCustomElementDefinition::new(
            "x-block".into(),
            "div".into(),
            false,
            true,
        )]))
        .unwrap();
    let d = document(&mut b, HtmlScriptingStatus::Disabled);
    let blocked = b
        .add_element(
            &d,
            name(Namespace::html(), "div"),
            attrs("is", "x-free"),
            HtmlCapturedIsValue::Value("x-block".into()),
            HtmlCaptureRegistryInput::Registry(reg.clone()),
        )
        .unwrap();
    let allowed = b
        .add_element(
            &d,
            name(Namespace::html(), "div"),
            attrs("is", "x-block"),
            HtmlCapturedIsValue::Absent,
            HtmlCaptureRegistryInput::Unavailable,
        )
        .unwrap();
    let unknown_is = b
        .add_element(
            &d,
            name(Namespace::html(), "div"),
            OrderedAttributes::default(),
            HtmlCapturedIsValue::Unavailable,
            HtmlCaptureRegistryInput::Registry(reg),
        )
        .unwrap();
    let null = b
        .add_element(
            &d,
            name(Namespace::html(), "div"),
            OrderedAttributes::default(),
            HtmlCapturedIsValue::Unavailable,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    for e in [&blocked, &allowed, &unknown_is, &null] {
        b.establish_no_shadow_root(e).unwrap();
    }
    let c = b.finish().unwrap();
    let id = |h| c.element_id(&c.subject_id(h).unwrap()).unwrap();
    assert_eq!(
        c.shadow_attachment_status(&id(&blocked)).unwrap(),
        HtmlShadowAttachmentStatus::DefinitionDisallows
    );
    assert_eq!(
        c.shadow_attachment_status(&id(&allowed)).unwrap(),
        HtmlShadowAttachmentStatus::Allowed
    );
    assert_eq!(
        c.shadow_attachment_status(&id(&null)).unwrap(),
        HtmlShadowAttachmentStatus::Allowed
    );
    assert!(matches!(
        c.shadow_attachment_status(&id(&unknown_is)),
        Err(HtmlCaptureError::MissingFact {
            fact: HtmlCaptureFact::IsValue,
            ..
        })
    ));
    let HtmlCapturedNodeKind::Element(e) = c.node(id(&blocked).subject()).unwrap().kind() else {
        panic!()
    };
    assert_eq!(
        e.attributes().as_slice()[0].value(),
        &DocumentString::from("x-free")
    );
}

#[test]
fn closed_shadow_presence_and_hosted_registry_require_no_captured_children() {
    let mut b = HtmlCaptureBuilder::new(limits(), HtmlHostFactsRevision::fresh());
    let reg = b.add_registry(Some(Vec::new())).unwrap();
    let d = document(&mut b, HtmlScriptingStatus::Enabled);
    let host = element(&mut b, &d, Namespace::html(), "div");
    b.establish_root(&host).unwrap();
    let properties = HtmlShadowRootProperties::new(
        HtmlShadowRootMode::Closed,
        HtmlSlotAssignment::Manual,
        true,
        true,
        true,
    );
    let shadow = b
        .add_shadow_root(
            &host,
            HtmlCapturedShadowProperties::new(properties, true, true, false),
            HtmlCaptureRegistryInput::Registry(reg.clone()),
        )
        .unwrap();
    let c = b.finish().unwrap();
    let host_id = c.element_id(&c.subject_id(&host).unwrap()).unwrap();
    assert_eq!(
        c.shadow_attachment_status(&host_id).unwrap(),
        HtmlShadowAttachmentStatus::ExistingRoot
    );
    let sid = c.subject_id(&shadow).unwrap();
    assert!(matches!(
        c.node(&sid).unwrap().children(),
        HtmlCapturedChildren::Unavailable
    ));
    assert!(
        matches!(c.node(host_id.subject()).unwrap().shadow_root(), Some(HtmlCapturedShadowRoot::Present(f)) if f.subject() == &sid)
    );
    let HtmlCapturedNodeKind::Fragment(f) = c.node(&sid).unwrap().kind() else {
        panic!()
    };
    let HtmlCapturedFragmentKind::ShadowRoot(p) = f.kind() else {
        panic!()
    };
    assert_eq!(p.properties(), properties);
    assert!(p.declarative());
    assert!(p.available_to_internals());
    assert!(!p.keep_registry_null());
    let ctx = HtmlFragmentContext::hosted_fragment(&c, &sid).unwrap();
    assert_eq!(ctx.target_document(), ctx.context_document());
    assert_eq!(
        ctx.root_registry(),
        &HtmlCapturedRegistry::Registry(c.registry_id(&reg).unwrap())
    );
    assert_eq!(ctx.context_element(), &host_id);
}

#[test]
fn missing_shadow_facts_are_not_absence_and_irrelevant_facts_are_not_required() {
    let mut b = HtmlCaptureBuilder::new(limits(), HtmlHostFactsRevision::fresh());
    let d = document(&mut b, HtmlScriptingStatus::Disabled);
    let unknown = element(&mut b, &d, Namespace::html(), "div");
    b.establish_root(&unknown).unwrap();
    let foreign = b
        .add_element(
            &d,
            name(Namespace::svg(), "div"),
            OrderedAttributes::default(),
            HtmlCapturedIsValue::Unavailable,
            HtmlCaptureRegistryInput::Unavailable,
        )
        .unwrap();
    let invalid = b
        .add_element(
            &d,
            name(Namespace::html(), "input"),
            OrderedAttributes::default(),
            HtmlCapturedIsValue::Unavailable,
            HtmlCaptureRegistryInput::Unavailable,
        )
        .unwrap();
    let c = b.finish().unwrap();
    let id = |h| c.element_id(&c.subject_id(h).unwrap()).unwrap();
    assert!(matches!(
        c.shadow_attachment_status(&id(&unknown)),
        Err(HtmlCaptureError::MissingFact {
            fact: HtmlCaptureFact::ShadowRoot,
            ..
        })
    ));
    assert_eq!(
        c.shadow_attachment_status(&id(&foreign)).unwrap(),
        HtmlShadowAttachmentStatus::InvalidHost
    );
    assert_eq!(
        c.shadow_attachment_status(&id(&invalid)).unwrap(),
        HtmlShadowAttachmentStatus::InvalidHost
    );
    // Inert context selection does not need a shadow decision or complete children.
    assert_eq!(
        HtmlFragmentContext::element(&c, &c.subject_id(&unknown).unwrap())
            .unwrap()
            .resolve_scripting_mode(HtmlParserScriptingMode::Inert)
            .unwrap(),
        HtmlParserScriptingMode::Disabled
    );
}

#[test]
fn cycles_document_conflicts_and_complete_children_fail_atomically() {
    let mut b = HtmlCaptureBuilder::new(limits(), HtmlHostFactsRevision::fresh());
    let d = document(&mut b, HtmlScriptingStatus::Disabled);
    let a = element(&mut b, &d, Namespace::html(), "div");
    let p = element(&mut b, &d, Namespace::html(), "div");
    b.establish_parent(&a, &p).unwrap();
    let usage = b.usage();
    assert_eq!(b.establish_parent(&p, &a), Err(HtmlCaptureError::Cycle));
    assert_eq!(b.usage(), usage);
    assert_eq!(
        b.establish_children(&p, &[a.clone(), a.clone()]),
        Err(HtmlCaptureError::DuplicateChild)
    );
    assert_eq!(b.usage(), usage);
    assert_eq!(
        b.establish_children(&p, &[]),
        Err(HtmlCaptureError::ParentChildConflict)
    );
    assert_eq!(b.usage(), usage);
    assert_eq!(
        b.establish_children(&p, &[a.clone(), p.clone()]),
        Err(HtmlCaptureError::Cycle)
    );
    assert_eq!(b.usage(), usage);
    b.establish_children(&p, std::slice::from_ref(&a)).unwrap();
    let extra = element(&mut b, &d, Namespace::html(), "span");
    assert_eq!(
        b.establish_parent(&extra, &p),
        Err(HtmlCaptureError::ParentChildConflict)
    );
    let other_doc = document(&mut b, HtmlScriptingStatus::Disabled);
    let other = element(&mut b, &other_doc, Namespace::html(), "span");
    assert_eq!(
        b.establish_parent(&other, &p),
        Err(HtmlCaptureError::DocumentMismatch)
    );
    assert_eq!(
        b.establish_parent(&d, &p),
        Err(HtmlCaptureError::ParentAlreadyEstablished)
    );
    assert_eq!(
        b.add_fragment(&a).unwrap_err(),
        HtmlCaptureError::WrongKind {
            expected: HtmlCaptureSubjectKind::Document,
            actual: HtmlCaptureSubjectKind::Element
        }
    );
    let c = b.finish().unwrap();
    assert!(matches!(
        c.node(&c.subject_id(&p).unwrap()).unwrap().parent(),
        HtmlCapturedParent::Unavailable
    ));
    assert_eq!(
        children(&c, &c.subject_id(&p).unwrap()),
        vec![c.subject_id(&a).unwrap()]
    );
}

#[test]
fn association_cycles_and_non_html_template_are_checked() {
    let mut b = HtmlCaptureBuilder::new(limits(), HtmlHostFactsRevision::fresh());
    let d = document(&mut b, HtmlScriptingStatus::Disabled);
    let t = element(&mut b, &d, Namespace::html(), "template");
    let f = b.add_fragment(&d).unwrap();
    b.establish_parent(&t, &f).unwrap();
    let usage = b.usage();
    assert_eq!(
        b.establish_template_contents(&t, &f),
        Err(HtmlCaptureError::Cycle)
    );
    assert_eq!(b.usage(), usage);
    let foreign = element(&mut b, &d, Namespace::svg(), "template");
    assert_eq!(
        b.establish_template_contents(&foreign, &f),
        Err(HtmlCaptureError::NotHtmlTemplate)
    );
    let c = b.finish().unwrap();
    assert!(matches!(
        c.node(&c.subject_id(&t).unwrap())
            .unwrap()
            .template_contents(),
        Some(HtmlCapturedTemplateContents::Unavailable)
    ));
    assert!(matches!(
        c.node(&c.subject_id(&f).unwrap()).unwrap().host(),
        Some(HtmlCapturedHost::Unavailable)
    ));
}

#[test]
fn each_capture_resource_has_a_typed_atomic_limit() {
    let zero = HtmlCaptureLimits::new(0, 0, 0, 0, 0, 0, 0);
    let mut b = HtmlCaptureBuilder::new(zero, HtmlHostFactsRevision::fresh());
    assert!(matches!(
        b.add_document(
            HtmlDocumentMode::NoQuirks,
            HtmlScriptingStatus::Disabled,
            HtmlCaptureRegistryInput::Null
        ),
        Err(HtmlCaptureError::LimitExceeded {
            resource: HtmlCaptureResource::Subjects,
            ..
        })
    ));
    assert!(matches!(
        b.add_registry(None),
        Err(HtmlCaptureError::LimitExceeded {
            resource: HtmlCaptureResource::Registries,
            ..
        })
    ));
    assert_eq!(b.usage(), HtmlCaptureUsage::default());
    for (selected, resource) in [
        (
            HtmlCaptureLimits::new(3, 0, 1, 1000, 5, 1, 1),
            HtmlCaptureResource::Relationships,
        ),
        (
            HtmlCaptureLimits::new(3, 20, 0, 1000, 5, 1, 1),
            HtmlCaptureResource::Attributes,
        ),
        (
            HtmlCaptureLimits::new(3, 20, 1, 0, 5, 1, 1),
            HtmlCaptureResource::Utf16Units,
        ),
    ] {
        let mut b = HtmlCaptureBuilder::new(selected, HtmlHostFactsRevision::fresh());
        let d = document(&mut b, HtmlScriptingStatus::Disabled);
        let before = b.usage();
        assert!(
            matches!(b.add_element(&d, name(Namespace::html(), "div"), attrs("id", "a"), HtmlCapturedIsValue::Absent,
            HtmlCaptureRegistryInput::Null), Err(HtmlCaptureError::LimitExceeded { resource: r, .. }) if r == resource)
        );
        assert_eq!(b.usage(), before);
        assert_eq!(b.finish().unwrap().usage().subjects(), 1);
    }
    let mut b = HtmlCaptureBuilder::new(
        HtmlCaptureLimits::new(3, 20, 0, 1000, 0, 1, 0),
        HtmlHostFactsRevision::fresh(),
    );
    let before = b.usage();
    assert!(matches!(
        b.add_registry(Some(vec![HtmlCustomElementDefinition::new(
            "x-a".into(),
            "x-a".into(),
            false,
            false
        )])),
        Err(HtmlCaptureError::LimitExceeded {
            resource: HtmlCaptureResource::Definitions,
            ..
        })
    ));
    assert_eq!(b.usage(), before);
    let d = document(&mut b, HtmlScriptingStatus::Disabled);
    let e = element(&mut b, &d, Namespace::html(), "div");
    let before = b.usage();
    assert!(matches!(
        b.establish_parent(&e, &d),
        Err(HtmlCaptureError::LimitExceeded {
            resource: HtmlCaptureResource::Depth,
            ..
        })
    ));
    assert_eq!(b.usage(), before);
}

#[test]
fn association_and_descendant_depth_admission_preserves_failed_relations() {
    let mut b = HtmlCaptureBuilder::new(
        HtmlCaptureLimits::new(5, 100, 0, 1000, 1, 0, 0),
        HtmlHostFactsRevision::fresh(),
    );
    let d = document(&mut b, HtmlScriptingStatus::Disabled);
    let t = element(&mut b, &d, Namespace::html(), "template");
    let f = b.add_fragment(&d).unwrap();
    let child = element(&mut b, &d, Namespace::html(), "div");
    b.establish_parent(&child, &f).unwrap();
    b.establish_parent(&t, &d).unwrap();
    let before = b.usage();
    assert!(matches!(
        b.establish_template_contents(&t, &f),
        Err(HtmlCaptureError::LimitExceeded {
            resource: HtmlCaptureResource::Depth,
            required: 2,
            ..
        })
    ));
    assert_eq!(b.usage(), before);
    let c = b.finish().unwrap();
    assert!(matches!(
        c.node(&c.subject_id(&f).unwrap()).unwrap().host(),
        Some(HtmlCapturedHost::Unavailable)
    ));
    assert!(matches!(
        c.node(&c.subject_id(&t).unwrap())
            .unwrap()
            .template_contents(),
        Some(HtmlCapturedTemplateContents::Unavailable)
    ));
    assert_eq!(c.usage().depth(), 1);
}

#[test]
fn definition_admission_validates_the_selected_modern_name_rules() {
    let mut b = HtmlCaptureBuilder::new(limits(), HtmlHostFactsRevision::fresh());
    for invalid in [
        "x",
        "X-host",
        "x-Host",
        "x- host",
        "x-/host",
        "x->",
        "annotation-xml",
    ] {
        let before = b.usage();
        assert_eq!(
            b.add_registry(Some(vec![HtmlCustomElementDefinition::new(
                invalid.into(),
                "div".into(),
                false,
                false
            )])),
            Err(HtmlCaptureError::InvalidDefinitionName)
        );
        assert_eq!(b.usage(), before);
    }
    // Pinned DOM's ASCII-first branch permits '<' and exact isolated surrogates.
    b.add_registry(Some(vec![HtmlCustomElementDefinition::new(
        DocumentString::from_utf16([b'x' as u16, b'-' as u16, b'<' as u16, 0xd800].as_slice()),
        "div".into(),
        false,
        false,
    )]))
    .unwrap();
    let before = b.usage();
    assert_eq!(
        b.add_registry(Some(vec![
            HtmlCustomElementDefinition::new("x-a".into(), "div".into(), false, false),
            HtmlCustomElementDefinition::new("x-a".into(), "span".into(), true, true)
        ])),
        Err(HtmlCaptureError::DuplicateDefinitionName)
    );
    assert_eq!(b.usage(), before);
}

#[test]
fn captured_processing_instructions_do_not_apply_native_authoring_checks() {
    let mut b = HtmlCaptureBuilder::new(limits(), HtmlHostFactsRevision::fresh());
    let d = document(&mut b, HtmlScriptingStatus::Disabled);
    let target = DocumentString::from("xml:é");
    let data = DocumentString::from_utf16([0xd800, b'>' as u16, b'?' as u16].as_slice());
    assert!(HtmlAuthoredProcessingInstruction::new(target.clone(), data.clone()).is_err());
    let pi = b
        .add_processing_instruction(&d, target.clone(), data.clone())
        .unwrap();
    let c = b.finish().unwrap();
    let HtmlCapturedNodeKind::ProcessingInstruction(p) =
        c.node(&c.subject_id(&pi).unwrap()).unwrap().kind()
    else {
        panic!()
    };
    assert_eq!(p.target(), &target);
    assert_eq!(p.data(), &data);
}

#[test]
fn deep_capture_validation_and_cleanup_use_no_recursive_tree_drop() {
    std::thread::Builder::new()
        .stack_size(64 * 1024)
        .spawn(|| {
            let mut b = HtmlCaptureBuilder::new(
                HtmlCaptureLimits::new(20_001, 100_000, 0, 1_000_000, 20_000, 0, 0),
                HtmlHostFactsRevision::fresh(),
            );
            let d = document(&mut b, HtmlScriptingStatus::Disabled);
            let mut parent = element(&mut b, &d, Namespace::html(), "div");
            b.establish_root(&parent).unwrap();
            for _ in 1..20_000 {
                let child = element(&mut b, &d, Namespace::html(), "div");
                b.establish_parent(&child, &parent).unwrap();
                parent = child;
            }
            let c = b.finish().unwrap();
            assert_eq!(c.usage().depth(), 19_999);
            let id = c.element_id(&c.subject_id(&parent).unwrap()).unwrap();
            assert!(c.nearest_form(&id).unwrap().is_none());
            drop(c);
        })
        .unwrap()
        .join()
        .unwrap();
}

#[test]
fn template_cache_rejects_known_class_conflict_but_preserves_unused_unavailable_class() {
    let mut builder = HtmlCaptureBuilder::new(limits(), HtmlHostFactsRevision::fresh());
    let ordinary = document(&mut builder, HtmlScriptingStatus::Disabled);
    let inert = document(&mut builder, HtmlScriptingStatus::Disabled);
    builder
        .establish_document_template_facts(
            &inert,
            HtmlDocumentClass::NonHtml,
            HtmlTemplateDocumentRole::InertTemplate,
            HtmlCaptureTemplateCacheInput::Absent,
        )
        .unwrap();
    let before = builder.usage();
    assert_eq!(
        builder.establish_document_template_facts(
            &ordinary,
            HtmlDocumentClass::Html,
            HtmlTemplateDocumentRole::Ordinary,
            HtmlCaptureTemplateCacheInput::Document(inert.clone())
        ),
        Err(HtmlCaptureError::InvalidTemplateDocumentCache)
    );
    assert_eq!(builder.usage(), before);
    builder
        .establish_document_template_facts(
            &ordinary,
            HtmlDocumentClass::Unavailable,
            HtmlTemplateDocumentRole::Ordinary,
            HtmlCaptureTemplateCacheInput::Document(inert.clone()),
        )
        .unwrap();
    let capture = builder.finish().unwrap();
    let ordinary = capture.subject_id(&ordinary).unwrap();
    let inert = capture
        .document_id(&capture.subject_id(&inert).unwrap())
        .unwrap();
    let node = capture.node(&ordinary).unwrap();
    assert_eq!(
        node.template_document_cache(),
        Some(HtmlCapturedTemplateCache::Document(inert))
    );
    let HtmlCapturedNodeKind::Document(facts) = node.kind() else {
        panic!("actual document")
    };
    assert_eq!(facts.document_class(), HtmlDocumentClass::Unavailable);
}
