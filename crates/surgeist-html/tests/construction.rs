use surgeist_html::{
    Attribute, DocumentString, ForeignHtmlResult, HtmlConstructionError, HtmlConstructionLimits,
    HtmlConstructionResource, HtmlDoctype, HtmlDocument, HtmlDocumentMode, HtmlFragment,
    HtmlFragmentOwner, HtmlNodeKind, HtmlProcessingInstruction, HtmlProcessingInstructionError,
    HtmlShadowRootMode, HtmlShadowRootProperties, HtmlSlotAssignment, Namespace, OrderedAttributes,
    QualifiedName,
};

fn limits() -> HtmlConstructionLimits {
    HtmlConstructionLimits::new(100, 100, 100, 100_000, 100)
}
fn name(namespace: Namespace, local: &str) -> QualifiedName {
    QualifiedName::new(namespace, None, local)
}

#[test]
fn mixed_siblings_retain_order_and_exact_utf16_without_reparsing() {
    let mut builder = HtmlFragment::builder(limits()).unwrap();
    let root = builder.root();
    builder.append_text(&root, "before").unwrap();
    builder.append_comment(&root, "<!-- raw -->").unwrap();
    let element = builder
        .append_element(
            &root,
            name(Namespace::html(), "unknown-tag"),
            OrderedAttributes::default(),
        )
        .unwrap();
    let exact = DocumentString::from_utf16(vec![0, 0xD83D, 0xDE00, 0xD800, 32, 0xDC00, 13, 10]);
    builder.append_text(&element, exact.clone()).unwrap();
    builder.append_text(&root, "after").unwrap();
    let fragment = builder.finish();
    let children: Vec<_> = fragment.root().children().collect();
    assert_eq!(children.len(), 4);
    assert_eq!(
        fragment.node(&children[0]).unwrap().kind(),
        &HtmlNodeKind::Text("before".into())
    );
    assert_eq!(
        fragment.node(&children[1]).unwrap().kind(),
        &HtmlNodeKind::Comment("<!-- raw -->".into())
    );
    assert_eq!(
        fragment.node(&children[3]).unwrap().kind(),
        &HtmlNodeKind::Text("after".into())
    );
    let element_view = fragment.node(&children[2]).unwrap();
    assert_eq!(element_view.parent(), Some(fragment.root().id()));
    let text = element_view.children().next().unwrap();
    assert_eq!(
        fragment.node(&text).unwrap().kind(),
        &HtmlNodeKind::Text(exact)
    );
}

#[test]
fn same_local_name_in_foreign_namespaces_and_raw_attributes_stays_distinct() {
    let mut builder = HtmlFragment::builder(limits()).unwrap();
    let root = builder.root();
    let attributes = OrderedAttributes::new([
        Attribute::new(name(Namespace::none(), "z-last"), " invalid &amp; "),
        Attribute::new(
            QualifiedName::new(Namespace::from_uri("urn:custom"), Some("p".into()), "id"),
            DocumentString::from_utf16(vec![0xDFFF]),
        ),
        Attribute::new(name(Namespace::none(), "id"), "native"),
    ])
    .unwrap();
    for namespace in [
        Namespace::html(),
        Namespace::svg(),
        Namespace::mathml(),
        Namespace::from_uri("urn:other"),
    ] {
        builder
            .append_element(&root, name(namespace, "Title"), attributes.clone())
            .unwrap();
    }
    let fragment = builder.finish();
    let children: Vec<_> = fragment.root().children().collect();
    for (id, namespace) in children.iter().zip([
        Namespace::html(),
        Namespace::svg(),
        Namespace::mathml(),
        Namespace::from_uri("urn:other"),
    ]) {
        let HtmlNodeKind::Element(element) = fragment.node(id).unwrap().kind() else {
            panic!("expected element")
        };
        assert_eq!(element.name().namespace(), &namespace);
        assert_eq!(
            element.name().local_name().code_units(),
            &[84, 105, 116, 108, 101]
        );
        assert_eq!(element.attributes(), &attributes);
        assert_eq!(
            element.attributes().as_slice()[0].value().code_units(),
            " invalid &amp; ".encode_utf16().collect::<Vec<_>>()
        );
        assert_eq!(
            element.attributes().as_slice()[1].value().code_units(),
            &[0xDFFF]
        );
    }
}

#[test]
fn document_mode_and_doctype_identifiers_are_explicit_and_exact() {
    let mut builder = HtmlDocument::builder(limits(), HtmlDocumentMode::LimitedQuirks).unwrap();
    let root = builder.root();
    let doctype = HtmlDoctype::new(
        "HTML",
        Some(DocumentString::from_utf16(vec![0xD800])),
        Some(DocumentString::default()),
    );
    builder.append_doctype(&root, doctype.clone()).unwrap();
    let document = builder.finish();
    assert_eq!(document.mode(), HtmlDocumentMode::LimitedQuirks);
    let child = document.root().children().next().unwrap();
    assert_eq!(
        document.node(&child).unwrap().kind(),
        &HtmlNodeKind::Doctype(doctype)
    );
    let HtmlNodeKind::Doctype(doctype) = document.node(&child).unwrap().kind() else {
        panic!("doctype")
    };
    assert_eq!(doctype.name().code_units(), &[72, 84, 77, 76]);
    assert_eq!(doctype.public_id().unwrap().code_units(), &[0xD800]);
    assert_eq!(doctype.system_id(), Some(&DocumentString::default()));
}

#[test]
fn strict_pi_authoring_follows_html_target_and_data_rules() {
    for target in ["", "9start", "-start", "a:b", "é", "a.b"] {
        assert_eq!(
            HtmlProcessingInstruction::new(target, "data"),
            Err(HtmlProcessingInstructionError::InvalidTarget)
        );
    }
    for target in ["xml", "XML", "Xml-StYlEsHeEt"] {
        assert_eq!(
            HtmlProcessingInstruction::new(target, "data"),
            Err(HtmlProcessingInstructionError::ReservedTarget)
        );
    }
    assert_eq!(
        HtmlProcessingInstruction::new("x", "a>b"),
        Err(HtmlProcessingInstructionError::DataContainsGreaterThan)
    );
    assert_eq!(
        HtmlProcessingInstruction::new("x", "a?"),
        Err(HtmlProcessingInstructionError::DataEndsWithQuestionMark)
    );
    let data = DocumentString::from_utf16(vec![32, 0xD800, 63, 97]);
    let pi = HtmlProcessingInstruction::new("_Target-9", data.clone()).unwrap();
    assert_eq!(
        pi.target().code_units(),
        "_Target-9".encode_utf16().collect::<Vec<_>>()
    );
    assert_eq!(pi.data(), &data);
    let mut builder = HtmlFragment::builder(limits()).unwrap();
    let root = builder.root();
    builder
        .append_processing_instruction(&root, pi.clone())
        .unwrap();
    let fragment = builder.finish();
    let id = fragment.root().children().next().unwrap();
    assert_eq!(
        fragment.node(&id).unwrap().kind(),
        &HtmlNodeKind::ProcessingInstruction(pi)
    );
}

#[test]
fn template_contents_are_a_separate_fragment_with_a_reverse_association() {
    let mut builder = HtmlDocument::builder(limits(), HtmlDocumentMode::NoQuirks).unwrap();
    let root = builder.root();
    let template = builder
        .append_element(
            &root,
            name(Namespace::html(), "template"),
            OrderedAttributes::default(),
        )
        .unwrap();
    let before = builder.usage();
    assert_eq!(
        builder.append_text(&template, "wrong parent").unwrap_err(),
        HtmlConstructionError::TemplateContentsRequired
    );
    assert_eq!(builder.usage(), before);
    let contents = builder.template_contents(&template).unwrap().unwrap();
    builder.append_text(&contents, "inside").unwrap();
    let document = builder.finish();
    let template_id = document.root().children().next().unwrap();
    let template_view = document.node(&template_id).unwrap();
    assert_eq!(template_view.children().len(), 0);
    let contents_id = template_view.template_contents().unwrap();
    let fragment = document.fragment(&contents_id).unwrap().unwrap();
    assert_eq!(fragment.root().parent(), None);
    assert_eq!(
        fragment.root().fragment_owner(),
        Some(&HtmlFragmentOwner::Template(template_id))
    );
    let text_id = fragment.root().children().next().unwrap();
    assert_eq!(
        fragment.node(&text_id).unwrap().kind(),
        &HtmlNodeKind::Text("inside".into())
    );
    assert_eq!(fragment.usage().nodes(), 4);
    assert_eq!(fragment.usage().edges(), 3);
    assert_eq!(fragment.usage().depth(), 3);
}

#[test]
fn foreign_template_spelling_is_an_ordinary_exact_element() {
    let mut builder = HtmlFragment::builder(limits()).unwrap();
    let root = builder.root();
    let foreign = builder
        .append_element(
            &root,
            name(Namespace::svg(), "template"),
            OrderedAttributes::default(),
        )
        .unwrap();
    assert!(builder.template_contents(&foreign).unwrap().is_none());
    builder.append_text(&foreign, "ordinary").unwrap();
    let fragment = builder.finish();
    let id = fragment.root().children().next().unwrap();
    assert_eq!(fragment.node(&id).unwrap().children().len(), 1);
}

#[test]
fn shadow_host_and_contents_keep_properties_and_separate_ordinary_children() {
    let mut builder = HtmlFragment::builder(limits()).unwrap();
    let root = builder.root();
    let host = builder
        .append_element(
            &root,
            name(Namespace::html(), "div"),
            OrderedAttributes::default(),
        )
        .unwrap();
    builder.append_text(&host, "light").unwrap();
    let properties = HtmlShadowRootProperties::new(
        HtmlShadowRootMode::Closed,
        HtmlSlotAssignment::Manual,
        true,
        false,
        true,
    );
    let shadow = builder
        .attach_declarative_shadow(&host, properties)
        .unwrap();
    builder.append_text(&shadow, "shadow").unwrap();
    let before = builder.usage();
    assert_eq!(
        builder
            .attach_declarative_shadow(&host, properties)
            .unwrap_err(),
        HtmlConstructionError::ShadowAlreadyAttached
    );
    assert_eq!(builder.usage(), before);
    let fragment = builder.finish();
    let host_id = fragment.root().children().next().unwrap();
    let host_view = fragment.node(&host_id).unwrap();
    assert_eq!(host_view.children().len(), 1);
    let relation = host_view.declarative_shadow_root().unwrap();
    assert_eq!(relation.host(), &host_id);
    assert_eq!(relation.properties(), properties);
    assert_eq!(properties.mode(), HtmlShadowRootMode::Closed);
    assert_eq!(properties.slot_assignment(), HtmlSlotAssignment::Manual);
    assert!(properties.delegates_focus());
    assert!(!properties.clonable());
    assert!(properties.serializable());
    let shadow = fragment.fragment(relation.contents()).unwrap().unwrap();
    assert_eq!(shadow.root().parent(), None);
    assert_eq!(
        shadow.root().fragment_owner(),
        Some(&HtmlFragmentOwner::ShadowHost(host_id))
    );
    let text_id = shadow.root().children().next().unwrap();
    assert_eq!(
        shadow.node(&text_id).unwrap().kind(),
        &HtmlNodeKind::Text("shadow".into())
    );
}

#[test]
fn foreign_builder_handles_and_result_ids_are_rejected_without_changes() {
    let mut first = HtmlFragment::builder(limits()).unwrap();
    let mut second = HtmlFragment::builder(limits()).unwrap();
    let first_root = first.root();
    assert_eq!(
        second.append_text(&first_root, "foreign").unwrap_err(),
        HtmlConstructionError::ForeignBuilder
    );
    assert_eq!(second.usage().nodes(), 1);
    first.append_text(&first_root, "first").unwrap();
    let second_root = second.root();
    second.append_text(&second_root, "second").unwrap();
    let first = first.finish();
    let second = second.finish();
    let id = first.root().children().next().unwrap();
    assert_eq!(second.node(&id).unwrap_err(), ForeignHtmlResult);
    assert_eq!(second.fragment(&id).unwrap_err(), ForeignHtmlResult);
    assert_ne!(first.root().id(), second.root().id());
    let clone = first.clone();
    assert_eq!(clone.root().id(), first.root().id());
    assert_eq!(
        clone.node(&id).unwrap().kind(),
        &HtmlNodeKind::Text("first".into())
    );
}

#[test]
fn old_immutable_artifact_survives_later_builds_and_associated_view_owns_arena() {
    let mut builder = HtmlDocument::builder(limits(), HtmlDocumentMode::Quirks).unwrap();
    let root = builder.root();
    let template = builder
        .append_element(
            &root,
            name(Namespace::html(), "template"),
            OrderedAttributes::default(),
        )
        .unwrap();
    let contents = builder.template_contents(&template).unwrap().unwrap();
    builder.append_text(&contents, "historical").unwrap();
    let old = builder.finish();
    let template_id = old.root().children().next().unwrap();
    let contents_id = old.node(&template_id).unwrap().template_contents().unwrap();
    let retained = old.fragment(&contents_id).unwrap().unwrap();
    let text_id = retained.root().children().next().unwrap();
    for _ in 0..10 {
        let later = HtmlFragment::builder(limits()).unwrap().finish();
        assert_eq!(later.node(&text_id).unwrap_err(), ForeignHtmlResult);
    }
    assert_eq!(old.mode(), HtmlDocumentMode::Quirks);
    drop(old);
    assert_eq!(
        retained.node(&text_id).unwrap().kind(),
        &HtmlNodeKind::Text("historical".into())
    );
}

#[test]
fn each_finite_budget_rejects_atomically_and_later_smaller_append_succeeds() {
    for (limits, expected) in [
        (
            HtmlConstructionLimits::new(1, 10, 10, 1000, 10),
            HtmlConstructionResource::Nodes,
        ),
        (
            HtmlConstructionLimits::new(10, 0, 10, 1000, 10),
            HtmlConstructionResource::Edges,
        ),
        (
            HtmlConstructionLimits::new(10, 10, 0, 1000, 10),
            HtmlConstructionResource::Attributes,
        ),
        (
            HtmlConstructionLimits::new(10, 10, 10, 0, 10),
            HtmlConstructionResource::Utf16Units,
        ),
        (
            HtmlConstructionLimits::new(10, 10, 10, 1000, 0),
            HtmlConstructionResource::Depth,
        ),
    ] {
        let mut builder = HtmlFragment::builder(limits).unwrap();
        let root = builder.root();
        let before = builder.usage();
        let attrs =
            OrderedAttributes::new([Attribute::new(name(Namespace::none(), "id"), "value")])
                .unwrap();
        let failure = builder
            .append_element(&root, name(Namespace::none(), "x"), attrs)
            .unwrap_err();
        assert!(
            matches!(failure, HtmlConstructionError::LimitExceeded { resource, .. } if resource == expected)
        );
        assert_eq!(builder.usage(), before);
        assert_eq!(builder.finish().root().children().len(), 0);
    }
    let mut builder = HtmlFragment::builder(HtmlConstructionLimits::new(3, 2, 0, 2, 2)).unwrap();
    let root = builder.root();
    assert!(matches!(
        builder.append_text(&root, "too long"),
        Err(HtmlConstructionError::LimitExceeded {
            resource: HtmlConstructionResource::Utf16Units,
            attempted: 8,
            limit: 2
        })
    ));
    builder
        .append_text(&root, DocumentString::from_utf16(vec![0xD800, 0xDC00]))
        .unwrap();
    assert_eq!(builder.usage().utf16_units(), 2);
    assert_eq!(builder.finish().root().children().len(), 1);
}

#[test]
fn all_stored_string_slots_consume_exact_unit_budget_and_zero_nodes_rejects_root() {
    assert!(matches!(
        HtmlFragment::builder(HtmlConstructionLimits::new(0, 0, 0, 0, 0)),
        Err(HtmlConstructionError::LimitExceeded {
            resource: HtmlConstructionResource::Nodes,
            attempted: 1,
            limit: 0,
        })
    ));
    let mut builder =
        HtmlFragment::builder(HtmlConstructionLimits::new(20, 20, 1, 31, 20)).unwrap();
    let root = builder.root();
    let attribute = Attribute::new(
        QualifiedName::new(Namespace::from_uri("urn:a"), Some("q".into()), "k"),
        "val",
    );
    builder
        .append_element(
            &root,
            QualifiedName::new(Namespace::from_uri("urn:x"), Some("p".into()), "tag"),
            OrderedAttributes::new([attribute]).unwrap(),
        )
        .unwrap();
    // Element: URI 5 + prefix 1 + local 3; attribute: 5 + 1 + 1 + 3.
    assert_eq!(builder.usage().utf16_units(), 19);
    builder
        .append_doctype(
            &root,
            HtmlDoctype::new("h", Some("ab".into()), Some("xyz".into())),
        )
        .unwrap();
    assert_eq!(builder.usage().utf16_units(), 25);
    builder
        .append_processing_instruction(&root, HtmlProcessingInstruction::new("x", "ab").unwrap())
        .unwrap();
    builder.append_comment(&root, "c").unwrap();
    builder
        .append_text(&root, DocumentString::from_utf16(vec![0xD83D, 0xDE00]))
        .unwrap();
    assert_eq!(builder.usage().utf16_units(), 31);
    let before = builder.usage();
    assert!(matches!(
        builder.append_text(&root, "x"),
        Err(HtmlConstructionError::LimitExceeded {
            resource: HtmlConstructionResource::Utf16Units,
            attempted: 32,
            limit: 31,
        })
    ));
    assert_eq!(builder.usage(), before);
    assert_eq!(builder.finish().root().children().len(), 5);
}

#[test]
fn template_and_shadow_associations_obey_depth_and_edge_budgets_atomically() {
    let mut builder =
        HtmlFragment::builder(HtmlConstructionLimits::new(10, 1, 10, 1000, 10)).unwrap();
    let root = builder.root();
    let before = builder.usage();
    assert!(matches!(
        builder.append_element(
            &root,
            name(Namespace::html(), "template"),
            OrderedAttributes::default()
        ),
        Err(HtmlConstructionError::LimitExceeded {
            resource: HtmlConstructionResource::Edges,
            attempted: 2,
            limit: 1
        })
    ));
    assert_eq!(builder.usage(), before);
    let host = builder
        .append_element(
            &root,
            name(Namespace::none(), "host"),
            OrderedAttributes::default(),
        )
        .unwrap();
    let before = builder.usage();
    let properties = HtmlShadowRootProperties::new(
        HtmlShadowRootMode::Open,
        HtmlSlotAssignment::Named,
        false,
        true,
        false,
    );
    assert!(matches!(
        builder.attach_declarative_shadow(&host, properties),
        Err(HtmlConstructionError::LimitExceeded {
            resource: HtmlConstructionResource::Edges,
            attempted: 2,
            limit: 1
        })
    ));
    assert_eq!(builder.usage(), before);
    let result = builder.finish();
    let host_id = result.root().children().next().unwrap();
    assert!(
        result
            .node(&host_id)
            .unwrap()
            .declarative_shadow_root()
            .is_none()
    );
    let mut builder =
        HtmlFragment::builder(HtmlConstructionLimits::new(10, 10, 10, 1000, 1)).unwrap();
    let root = builder.root();
    assert!(matches!(
        builder.append_element(
            &root,
            name(Namespace::html(), "template"),
            OrderedAttributes::default()
        ),
        Err(HtmlConstructionError::LimitExceeded {
            resource: HtmlConstructionResource::Depth,
            attempted: 2,
            limit: 1
        })
    ));
    assert_eq!(builder.usage().nodes(), 1);
}

#[test]
fn leaf_parents_are_rejected_and_deep_artifacts_drop_without_recursive_ownership() {
    let mut builder = HtmlFragment::builder(HtmlConstructionLimits::new(
        20_002, 20_001, 0, 20_001, 20_001,
    ))
    .unwrap();
    let root = builder.root();
    let leaf = builder.append_comment(&root, "").unwrap();
    assert_eq!(
        builder.append_text(&leaf, "invalid").unwrap_err(),
        HtmlConstructionError::ParentCannotHaveChildren
    );
    assert_eq!(
        builder
            .attach_declarative_shadow(
                &leaf,
                HtmlShadowRootProperties::new(
                    HtmlShadowRootMode::Open,
                    HtmlSlotAssignment::Named,
                    false,
                    false,
                    false
                )
            )
            .unwrap_err(),
        HtmlConstructionError::ShadowHostMustBeElement
    );
    let mut parent = root;
    for _ in 0..20_000 {
        parent = builder
            .append_element(
                &parent,
                name(Namespace::none(), "x"),
                OrderedAttributes::default(),
            )
            .unwrap();
    }
    let artifact = builder.finish();
    assert_eq!(artifact.usage().depth(), 20_000);
    assert_eq!(artifact.usage().nodes(), 20_002);
    drop(artifact);
}
