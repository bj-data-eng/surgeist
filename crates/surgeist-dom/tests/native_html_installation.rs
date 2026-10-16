//! Real native authoring -> canonical Draft installation, no parser or toy sink.
use std::sync::atomic::AtomicBool;
use surgeist_dom::*;
use surgeist_html::{
    HtmlAuthoredProcessingInstruction, HtmlConstructionLimits, HtmlDoctype, HtmlFragment,
    HtmlNodeKind,
};

fn authoring() -> HtmlConstructionLimits {
    HtmlConstructionLimits::new(1024, 2048, 1024, 131072, 512)
}
fn limits() -> DomNativeHtmlInstallLimits {
    DomNativeHtmlInstallLimits::new(1024, 1000000, 1048576)
}
fn dom() -> Dom {
    Dom::new(DomLimits::new(1024, 131072, 2048, 1024, 512, 16, 1048576).unwrap()).unwrap()
}
fn apply(dom: &mut Dom, batch: DomBatch) -> DomCommit {
    dom.apply(batch, &AtomicBool::new(false)).unwrap()
}
fn setup(dom: &mut Dom) -> (DocumentId, NodeId) {
    let mut b = DomBatch::new();
    let d = b.create_document();
    let root = b.create_element_ns(&d, Namespace::html(), "div");
    b.insert_before(&root, &d, None);
    let c = apply(dom, b);
    (c.document(&d).unwrap(), c.created(&root).unwrap().unwrap())
}
fn stamp(dom: &Dom, d: DocumentId) -> DomDocumentStamp {
    dom.snapshot().document(d).unwrap().stamp()
}
fn name(namespace: Namespace, local: &str) -> QualifiedName {
    QualifiedName::new(namespace, None, local)
}
fn fragment_with_element(name: QualifiedName, attributes: OrderedAttributes) -> HtmlFragment {
    let mut b = HtmlFragment::builder(authoring()).unwrap();
    b.append_element(&b.root(), name, attributes).unwrap();
    b.finish()
}
fn install(
    dom: &mut Dom,
    d: DocumentId,
    parent: NodeId,
    source: &HtmlFragment,
) -> DomNativeHtmlInstallation {
    dom.install_native_html_fragment(
        stamp(dom, d),
        parent,
        None,
        source,
        limits(),
        &AtomicBool::new(false),
    )
    .unwrap()
}
fn unchanged(dom: &Dom, before: &DomSnapshot, usage: DomUsage, parent: NodeId) {
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    assert_eq!(dom.usage(), usage);
    assert_eq!(
        dom.snapshot().node(parent).unwrap().children(),
        before.node(parent).unwrap().children()
    );
    assert!(
        dom.changes_since(before.stamp())
            .unwrap()
            .changes()
            .is_empty()
    );
}

#[test]
fn mixed_fields_and_utf16_install_before_real_reference_with_complete_historical_map() {
    let mut dom = dom();
    let (d, parent) = setup(&mut dom);
    let mut batch = DomBatch::new();
    let marker = batch.create_comment(d, "marker");
    batch.insert_before(&marker, parent, None);
    let c = apply(&mut dom, batch);
    let marker = c.created(&marker).unwrap().unwrap();
    let before = dom.snapshot();
    let mut b = HtmlFragment::builder(authoring()).unwrap();
    let root = b.root();
    b.append_text(&root, DocumentString::from_utf16(vec![0, 0xd800, 13, 10]))
        .unwrap();
    b.append_comment(&root, "<!-- exact -->").unwrap();
    let attributes = OrderedAttributes::new([
        Attribute::new(
            name(Namespace::none(), "xml:lang"),
            DocumentString::from_utf16(vec![0xdc00]),
        ),
        Attribute::new(
            QualifiedName::new(Namespace::from_uri("urn:x"), Some("q:r".into()), "id"),
            "foreign",
        ),
        Attribute::new(name(Namespace::none(), "id"), "plain"),
    ])
    .unwrap();
    let element = b
        .append_element(
            &root,
            QualifiedName::new(
                Namespace::from_uri("urn:component"),
                Some("p:q".into()),
                "a:b",
            ),
            attributes.clone(),
        )
        .unwrap();
    b.append_text(&element, "child").unwrap();
    b.append_processing_instruction(
        &root,
        HtmlAuthoredProcessingInstruction::new("target", "x='y'").unwrap(),
    )
    .unwrap();
    let source = b.finish();
    let ids: Vec<_> = source.root().children().collect();
    let child = source.node(&ids[2]).unwrap().children().next().unwrap();
    let report = dom
        .install_native_html_fragment(
            stamp(&dom, d),
            parent,
            Some(marker),
            &source,
            limits(),
            &AtomicBool::new(false),
        )
        .unwrap();
    let mapped: Vec<_> = ids.iter().map(|id| report.node_for(id).unwrap()).collect();
    let s = report.snapshot();
    assert_eq!(s.stamp(), dom.snapshot().stamp());
    assert_eq!(report.changes().from(), before.stamp());
    assert_eq!(report.changes().to(), s.stamp());
    assert_eq!(report.document(), d);
    assert_eq!(report.bindings().len(), 6); //wrapper, four siblings, one child
    assert_eq!(report.node_for(&source.root().id()), Some(report.root()));
    assert_eq!(s.node(report.root()).unwrap().parent(), None);
    assert!(s.node(report.root()).unwrap().children().is_empty());
    assert_eq!(
        s.node(parent).unwrap().children(),
        &[mapped[0], mapped[1], mapped[2], mapped[3], marker]
    );
    assert_eq!(
        s.node(mapped[0]).unwrap().data().unwrap().code_units(),
        &[0, 0xd800, 13, 10]
    );
    assert_eq!(
        s.node(mapped[1]).unwrap().data(),
        Some(&"<!-- exact -->".into())
    );
    let e = s.node(mapped[2]).unwrap().as_element().unwrap();
    let HtmlNodeKind::Element(authored) = source.node(&ids[2]).unwrap().kind() else {
        panic!("authored Element")
    };
    assert_eq!(e.name(), authored.name());
    assert_eq!(e.name().prefix(), Some(&"p:q".into()));
    assert_eq!(e.name().local_name(), &"a:b".into());
    assert_eq!(e.attributes(), &attributes);
    assert_eq!(
        s.node(mapped[2]).unwrap().children(),
        &[report.node_for(&child).unwrap()]
    );
    let DomNodeKind::ProcessingInstruction(pi) = s.node(mapped[3]).unwrap().kind() else {
        panic!("installed PI")
    };
    assert_eq!(pi.attribute(&"x".into()), Some(&"y".into()));
    assert_eq!(before.node(parent).unwrap().children(), &[marker]);
    let source_id = ids[2].clone();
    drop(source);
    let mut batch = DomBatch::new();
    batch.detach(mapped[2]);
    batch.destroy(mapped[2]);
    apply(&mut dom, batch);
    assert_eq!(report.node_for(&source_id), Some(mapped[2]));
    assert_eq!(
        report.snapshot().node(mapped[2]).unwrap().node_name(),
        "p:q:a:b".into()
    );
    assert!(matches!(
        dom.snapshot().node(mapped[2]),
        Err(DomError::StaleNode)
    ));
}

#[test]
fn native_components_preserve_whole_spelling_and_do_not_replay_ns_api_conditions() {
    let mut dom = dom();
    let (d, parent) = setup(&mut dom);
    let cases = [
        QualifiedName::new(Namespace::html(), None, "DiV"),
        QualifiedName::new(Namespace::html(), None, "a:b"),
        QualifiedName::new(Namespace::none(), None, "xmlns"),
        QualifiedName::new(Namespace::none(), Some("p".into()), "x"),
        QualifiedName::new(Namespace::from_uri("urn:other"), Some("xml".into()), "x"),
        QualifiedName::new(
            Namespace::html(),
            None,
            DocumentString::from_utf16(vec![65, 0xc9, 0xd800, 90]),
        ),
    ];
    for exact in cases {
        let attributes = OrderedAttributes::new([Attribute::new(
            QualifiedName::new(Namespace::none(), Some("p:q".into()), "xml:lang"),
            "raw",
        )])
        .unwrap();
        let source = fragment_with_element(exact.clone(), attributes.clone());
        let id = source.root().children().next().unwrap();
        let report = install(&mut dom, d, parent, &source);
        let element = report
            .snapshot()
            .node(report.node_for(&id).unwrap())
            .unwrap()
            .as_element()
            .unwrap();
        assert_eq!(element.name(), &exact);
        assert_eq!(element.attributes(), &attributes);
    }
}

#[test]
fn separate_results_and_repeated_installs_never_alias_source_or_canonical_identities() {
    let mut dom = dom();
    let (d, parent) = setup(&mut dom);
    let source = fragment_with_element(
        name(Namespace::none(), "same"),
        OrderedAttributes::default(),
    );
    let other = fragment_with_element(
        name(Namespace::none(), "same"),
        OrderedAttributes::default(),
    );
    let id = source.root().children().next().unwrap();
    let foreign = other.root().children().next().unwrap();
    let a = install(&mut dom, d, parent, &source);
    let b = install(&mut dom, d, parent, &source);
    assert_ne!(a.root(), b.root());
    assert_ne!(a.node_for(&id), b.node_for(&id));
    assert_eq!(a.node_for(&foreign), None);
    assert_eq!(b.node_for(&foreign), None);
    assert_eq!(
        a.snapshot().node(parent).unwrap().children(),
        &[a.node_for(&id).unwrap()]
    );
    assert_eq!(
        b.snapshot().node(parent).unwrap().children(),
        &[a.node_for(&id).unwrap(), b.node_for(&id).unwrap()]
    );
}

#[test]
fn template_owned_contents_and_script_fresh_preparation_use_real_native_documents() {
    let mut dom = dom();
    let (d, parent) = setup(&mut dom);
    let mut b = HtmlFragment::builder(authoring()).unwrap();
    let root = b.root();
    let template = b
        .append_element(
            &root,
            name(Namespace::html(), "template"),
            OrderedAttributes::default(),
        )
        .unwrap();
    let contents = b.template_contents(&template).unwrap().unwrap();
    b.append_element(
        &contents,
        name(Namespace::html(), "script"),
        OrderedAttributes::new([Attribute::new(name(Namespace::none(), "src"), "")]).unwrap(),
    )
    .unwrap();
    let script = b
        .append_element(
            &root,
            name(Namespace::html(), "script"),
            OrderedAttributes::default(),
        )
        .unwrap();
    b.append_text(&script, " \n").unwrap();
    let source = b.finish();
    let ids: Vec<_> = source.root().children().collect();
    let contents = source.node(&ids[0]).unwrap().template_contents().unwrap();
    let inner = source.node(&contents).unwrap().children().next().unwrap();
    let report = install(&mut dom, d, parent, &source);
    let s = report.snapshot();
    let template = report.node_for(&ids[0]).unwrap();
    let contents = report.node_for(&contents).unwrap();
    assert_eq!(
        s.node(template)
            .unwrap()
            .as_template()
            .unwrap()
            .contents()
            .unwrap(),
        contents
    );
    let inert = s.node(contents).unwrap().owner_document().unwrap();
    assert_eq!(
        s.document(d).unwrap().associated_inert_template_document(),
        Some(inert)
    );
    assert_eq!(
        s.document(inert).unwrap().template_role(),
        DomTemplateDocumentRole::InertTemplate
    );
    let inner = report.node_for(&inner).unwrap();
    assert!(!s.node(inner).unwrap().is_connected());
    assert!(
        !s.node(inner)
            .unwrap()
            .as_html_script()
            .unwrap()
            .flags()
            .already_started()
    );
    assert_eq!(s.node(inner).unwrap().owner_document(), Some(inert));
    let outer = report.node_for(&ids[1]).unwrap();
    let state = s.node(outer).unwrap().as_html_script().unwrap();
    assert!(state.flags().already_started());
    assert_eq!(state.preparation_document(), Some(d));
    assert_eq!(
        state.script_type(),
        Some(surgeist_html::HtmlScriptKind::Classic)
    );
    let mut batch = DomBatch::new();
    batch.insert_before(inner, parent, None);
    apply(&mut dom, batch);
    assert_eq!(
        dom.snapshot()
            .node(inner)
            .unwrap()
            .as_html_script()
            .unwrap()
            .preparation_document(),
        Some(d)
    );
    assert!(
        !report
            .snapshot()
            .node(inner)
            .unwrap()
            .as_html_script()
            .unwrap()
            .flags()
            .already_started()
    );
}

#[test]
fn destination_template_contents_is_qualified_by_its_actual_inert_document() {
    let mut dom = dom();
    let (d, parent) = setup(&mut dom);
    let mut batch = DomBatch::new();
    let template = batch.create_element_ns(d, Namespace::html(), "template");
    batch.insert_before(&template, parent, None);
    let c = apply(&mut dom, batch);
    let template = c.created(&template).unwrap().unwrap();
    let s = dom.snapshot();
    let contents = s
        .node(template)
        .unwrap()
        .as_template()
        .unwrap()
        .contents()
        .unwrap();
    let inert = s.node(contents).unwrap().owner_document().unwrap();
    let source = fragment_with_element(
        name(Namespace::html(), "script"),
        OrderedAttributes::new([Attribute::new(name(Namespace::none(), "src"), "")]).unwrap(),
    );
    let id = source.root().children().next().unwrap();
    let report = install(&mut dom, inert, contents, &source);
    let node = report.node_for(&id).unwrap();
    assert_eq!(report.document(), inert);
    assert!(
        !report
            .snapshot()
            .node(node)
            .unwrap()
            .as_html_script()
            .unwrap()
            .flags()
            .already_started()
    );
    assert_eq!(
        report.snapshot().node(node).unwrap().owner_document(),
        Some(inert)
    );
}

#[test]
fn exact_target_revision_and_reference_validation_precede_source_work() {
    let mut dom = dom();
    let (d, parent) = setup(&mut dom);
    let source = fragment_with_element(
        QualifiedName::new(Namespace::none(), Some("".into()), "x"),
        OrderedAttributes::default(),
    );
    let old = stamp(&dom, d);
    let mut b = DomBatch::new();
    let text = b.create_text(d, "unrelated");
    let c = apply(&mut dom, b);
    let text = c.created(&text).unwrap().unwrap();
    assert!(matches!(
        dom.install_native_html_fragment(
            old,
            parent,
            None,
            &source,
            limits(),
            &AtomicBool::new(false)
        ),
        Err(DomNativeHtmlInstallError::StaleDestination { .. })
    ));
    let before = dom.snapshot();
    let usage = dom.usage();
    let basis = stamp(&dom, d);
    assert!(matches!(
        dom.install_native_html_fragment(
            basis,
            parent,
            Some(text),
            &source,
            DomNativeHtmlInstallLimits::new(0, 0, 0),
            &AtomicBool::new(false)
        ),
        Err(DomNativeHtmlInstallError::Target(DomError::NotFound))
    ));
    assert!(matches!(
        dom.install_native_html_fragment(
            basis,
            text,
            None,
            &source,
            limits(),
            &AtomicBool::new(false)
        ),
        Err(DomNativeHtmlInstallError::Target(
            DomError::HierarchyRequest
        ))
    ));
    let mut foreign = crate::dom();
    let (foreign_doc, foreign_parent) = setup(&mut foreign);
    assert!(matches!(
        dom.install_native_html_fragment(
            stamp(&foreign, foreign_doc),
            foreign_parent,
            None,
            &source,
            limits(),
            &AtomicBool::new(false)
        ),
        Err(DomNativeHtmlInstallError::Target(DomError::ForeignOwner))
    ));
    unchanged(&dom, &before, usage, parent);
}

#[test]
fn invalid_components_and_late_unadmitted_families_publish_no_partial_identities() {
    let mut dom = dom();
    let (d, parent) = setup(&mut dom);
    for invalid in [
        QualifiedName::new(Namespace::none(), Some("".into()), "x"),
        name(Namespace::none(), "a b"),
        name(Namespace::none(), "_a?"),
    ] {
        let source = fragment_with_element(invalid, OrderedAttributes::default());
        let id = source.root().children().next().unwrap();
        let before = dom.snapshot();
        let usage = dom.usage();
        let error = dom
            .install_native_html_fragment(
                stamp(&dom, d),
                parent,
                None,
                &source,
                limits(),
                &AtomicBool::new(false),
            )
            .unwrap_err();
        assert!(
            matches!(error,DomNativeHtmlInstallError::Source { subject,error:DomError::InvalidCharacter } if subject==id)
        );
        unchanged(&dom, &before, usage, parent);
    }
    let mut b = HtmlFragment::builder(authoring()).unwrap();
    let root = b.root();
    b.append_text(&root, "private prefix").unwrap();
    b.append_element(
        &root,
        name(Namespace::html(), "audio"),
        OrderedAttributes::default(),
    )
    .unwrap();
    let source = b.finish();
    let id = source.root().children().nth(1).unwrap();
    let before = dom.snapshot();
    let usage = dom.usage();
    assert!(
        matches!(dom.install_native_html_fragment(stamp(&dom,d),parent,None,&source,limits(),&AtomicBool::new(false)),Err(DomNativeHtmlInstallError::Source { subject,error:DomError::NativeInitializationRequired(surgeist_html::HtmlElementClass::KnownHtml(surgeist_html::KnownHtmlElement::Audio)) }) if subject==id)
    );
    unchanged(&dom, &before, usage, parent);
}

#[test]
fn precise_declarative_shadow_and_hosted_root_refusals_preserve_the_entire_batch() {
    let mut dom = dom();
    let (d, parent) = setup(&mut dom);
    let mut b = HtmlFragment::builder(authoring()).unwrap();
    let root = b.root();
    b.append_text(&root, "private prefix").unwrap();
    let host = b
        .append_element(
            &root,
            name(Namespace::html(), "div"),
            OrderedAttributes::default(),
        )
        .unwrap();
    b.attach_declarative_shadow(
        &host,
        HtmlShadowRootProperties::new(
            HtmlShadowRootMode::Closed,
            HtmlSlotAssignment::Manual,
            true,
            true,
            true,
        ),
    )
    .unwrap();
    let source = b.finish();
    let host = source.root().children().nth(1).unwrap();
    let before = dom.snapshot();
    let usage = dom.usage();
    assert!(
        matches!(dom.install_native_html_fragment(stamp(&dom,d),parent,None,&source,limits(),&AtomicBool::new(false)),Err(DomNativeHtmlInstallError::UnsupportedAssociation { subject,association:DomNativeHtmlAssociation::DeclarativeShadow }) if subject==host)
    );
    unchanged(&dom, &before, usage, parent);
    let mut b = HtmlFragment::builder(authoring()).unwrap();
    b.append_element(
        &b.root(),
        name(Namespace::html(), "template"),
        OrderedAttributes::default(),
    )
    .unwrap();
    let source = b.finish();
    let template = source.root().children().next().unwrap();
    let contents = source.node(&template).unwrap().template_contents().unwrap();
    let subview = source.fragment(&contents).unwrap().unwrap();
    assert!(
        matches!(dom.install_native_html_fragment(stamp(&dom,d),parent,None,&subview,limits(),&AtomicBool::new(false)),Err(DomNativeHtmlInstallError::UnsupportedAssociation { subject,association:DomNativeHtmlAssociation::HostedSourceRoot }) if subject==contents)
    );
    unchanged(&dom, &before, usage, parent);
}

#[test]
fn mapped_node_work_storage_and_canonical_limits_are_independently_atomic() {
    let mut dom = dom();
    let (d, parent) = setup(&mut dom);
    let mut b = HtmlFragment::builder(authoring()).unwrap();
    b.append_text(&b.root(), "x").unwrap();
    let source = b.finish();
    for install_limits in [
        DomNativeHtmlInstallLimits::new(1, 1000, 65536),
        DomNativeHtmlInstallLimits::new(2, 0, 65536),
        DomNativeHtmlInstallLimits::new(2, 1000, 0),
    ] {
        let before = dom.snapshot();
        let usage = dom.usage();
        assert!(matches!(
            dom.install_native_html_fragment(
                stamp(&dom, d),
                parent,
                None,
                &source,
                install_limits,
                &AtomicBool::new(false)
            ),
            Err(DomNativeHtmlInstallError::ResourceLimit { .. })
        ));
        unchanged(&dom, &before, usage, parent);
    }
    let report = dom
        .install_native_html_fragment(
            stamp(&dom, d),
            parent,
            None,
            &source,
            DomNativeHtmlInstallLimits::new(2, 1000, 65536),
            &AtomicBool::new(false),
        )
        .unwrap();
    assert_eq!(report.bindings().len(), 2);
    let empty = HtmlFragment::builder(authoring()).unwrap().finish();
    let before = dom.snapshot();
    let usage = dom.usage();
    assert!(matches!(
        dom.install_native_html_fragment(
            stamp(&dom, d),
            parent,
            None,
            &empty,
            DomNativeHtmlInstallLimits::new(1, 8, 65536),
            &AtomicBool::new(false)
        ),
        Err(DomNativeHtmlInstallError::ResourceLimit {
            resource: DomNativeHtmlInstallResource::Work,
            limit: 8
        })
    ));
    unchanged(&dom, &before, usage, parent);
    let report = dom
        .install_native_html_fragment(
            stamp(&dom, d),
            parent,
            None,
            &empty,
            DomNativeHtmlInstallLimits::new(1, 9, 65536),
            &AtomicBool::new(false),
        )
        .unwrap();
    assert_eq!(report.bindings().len(), 1);
    let before = dom.snapshot();
    let usage = dom.usage();
    assert!(matches!(
        dom.install_native_html_fragment(
            stamp(&dom, d),
            parent,
            None,
            &source,
            limits(),
            &AtomicBool::new(true)
        ),
        Err(DomNativeHtmlInstallError::Cancelled)
    ));
    unchanged(&dom, &before, usage, parent);
}

#[test]
fn hierarchy_failure_never_changes_target_metadata_or_installs_partial_source() {
    let mut dom = dom();
    let (d, parent) = setup(&mut dom);
    let mut b = HtmlFragment::builder(authoring()).unwrap();
    b.append_doctype(&b.root(), HtmlDoctype::new("html", None, None))
        .unwrap();
    let source = b.finish();
    let before = dom.snapshot();
    let usage = dom.usage();
    let metadata = before.document(d).unwrap().metadata().clone();
    assert!(matches!(
        dom.install_native_html_fragment(
            stamp(&dom, d),
            parent,
            None,
            &source,
            limits(),
            &AtomicBool::new(false)
        ),
        Err(DomNativeHtmlInstallError::Source {
            error: DomError::HierarchyRequest,
            ..
        })
    ));
    unchanged(&dom, &before, usage, parent);
    assert_eq!(dom.snapshot().document(d).unwrap().metadata(), &metadata);
}

#[test]
fn installed_families_run_actual_attribute_and_control_hooks_then_remain_live() {
    let mut dom = dom();
    let (d, parent) = setup(&mut dom);
    let mut b = HtmlFragment::builder(authoring()).unwrap();
    let root = b.root();
    let form = b
        .append_element(
            &root,
            name(Namespace::html(), "form"),
            OrderedAttributes::new([Attribute::new(name(Namespace::none(), "id"), "chosen")])
                .unwrap(),
        )
        .unwrap();
    for local in ["button", "fieldset"] {
        b.append_element(
            &form,
            name(Namespace::html(), local),
            OrderedAttributes::default(),
        )
        .unwrap();
    }
    b.append_element(
        &root,
        name(Namespace::html(), "canvas"),
        OrderedAttributes::new([
            Attribute::new(name(Namespace::none(), "width"), "000400"),
            Attribute::new(name(Namespace::none(), "height"), "0"),
        ])
        .unwrap(),
    )
    .unwrap();
    for local in ["marquee", "slot", "frame", "iframe"] {
        b.append_element(
            &root,
            name(Namespace::html(), local),
            OrderedAttributes::default(),
        )
        .unwrap();
    }
    let source = b.finish();
    let ids: Vec<_> = source.root().children().collect();
    let controls: Vec<_> = source.node(&ids[0]).unwrap().children().collect();
    let report = install(&mut dom, d, parent, &source);
    let s = report.snapshot();
    let form = report.node_for(&ids[0]).unwrap();
    let button = report.node_for(&controls[0]).unwrap();
    let fieldset = report.node_for(&controls[1]).unwrap();
    assert_eq!(
        s.node(button)
            .unwrap()
            .as_button()
            .unwrap()
            .form_owner()
            .unwrap(),
        Some(form)
    );
    assert_eq!(
        s.node(fieldset)
            .unwrap()
            .as_fieldset()
            .unwrap()
            .form_owner()
            .unwrap(),
        Some(form)
    );
    assert_eq!(
        s.node(form).unwrap().as_form().unwrap().past_names().len(),
        0
    );
    let capture = s
        .html_capture(
            d,
            None,
            surgeist_html_semantics::HtmlCaptureLimits::new(256, 4096, 256, 32768, 256, 0, 0),
            DomHtmlCaptureLimits::new(256, 1000000),
            &AtomicBool::new(false),
        )
        .unwrap();
    for control in [button, fieldset] {
        use surgeist_html_semantics::{
            HtmlFormOwnerOrigin, HtmlSemanticCancellation, HtmlSemanticFact, HtmlSemanticLimits,
            HtmlSemanticQuery, HtmlSemanticRequest, HtmlSemanticValue,
        };
        let result = capture
            .query(
                &[HtmlSemanticRequest::new(
                    capture.subject(control).unwrap(),
                    HtmlSemanticQuery::FormOwner,
                )],
                surgeist_html::HtmlValueInterpreter::new(32768),
                HtmlSemanticLimits::new(1000000, 16, 1024, 512, 1000000, 1024),
                HtmlSemanticCancellation::new(),
            )
            .unwrap();
        let HtmlSemanticFact::FormOwner(HtmlSemanticValue::Known(owner)) =
            result.results()[0].fact()
        else {
            panic!("actual established control owner required")
        };
        assert_eq!(owner.owner(), Some(&capture.subject(form).unwrap()));
        assert_eq!(owner.origin(), HtmlFormOwnerOrigin::EstablishedCurrent);
    }
    let canvas = report.node_for(&ids[1]).unwrap();
    let state = s
        .node(canvas)
        .unwrap()
        .as_element()
        .unwrap()
        .canvas_none_state()
        .unwrap();
    assert!(
        matches!(state.width(),surgeist_html::HtmlCanvasBitmapDimension::Parsed(value) if value.decimal_digits()=="400")
    );
    assert!(
        matches!(state.height(),surgeist_html::HtmlCanvasBitmapDimension::Parsed(value) if value.decimal_digits()=="0")
    );
    let marquee = report.node_for(&ids[2]).unwrap();
    let state = s
        .node(marquee)
        .unwrap()
        .as_element()
        .unwrap()
        .marquee_state()
        .unwrap();
    assert!(state.turned_on());
    assert_eq!(state.current_loop_index(), 0);
    let slot = report.node_for(&ids[3]).unwrap();
    assert!(s.slot(slot).unwrap().manual_assignments().is_empty());
    assert!(
        s.node(report.node_for(&ids[4]).unwrap())
            .unwrap()
            .as_element()
            .unwrap()
            .child_navigable()
            .unwrap()
            .is_none()
    );
    assert!(
        s.node(report.node_for(&ids[5]).unwrap())
            .unwrap()
            .as_element()
            .unwrap()
            .iframe_initial_state()
            .unwrap()
            .lazy_load_resumption_is_null()
    );
    let mut batch = DomBatch::new();
    batch.set_custom_validity(button, "line\r\nnext");
    batch.stop_marquee(marquee);
    batch.set_attribute(canvas, "width", "20");
    batch.assign_slot(slot, vec![button.into()]);
    apply(&mut dom, batch);
    let current = dom.snapshot();
    assert_eq!(
        current
            .node(button)
            .unwrap()
            .as_button()
            .unwrap()
            .state()
            .custom_validity_message(),
        &"line\nnext".into()
    );
    assert!(
        !current
            .node(marquee)
            .unwrap()
            .as_element()
            .unwrap()
            .marquee_state()
            .unwrap()
            .turned_on()
    );
    assert_eq!(current.slot(slot).unwrap().manual_assignments(), &[button]);
    assert!(
        matches!(current.node(canvas).unwrap().as_element().unwrap().canvas_none_state().unwrap().width(),surgeist_html::HtmlCanvasBitmapDimension::Parsed(value) if value.decimal_digits()=="20")
    );
    assert!(
        report
            .snapshot()
            .node(button)
            .unwrap()
            .as_button()
            .unwrap()
            .state()
            .custom_validity_message()
            .is_empty()
    );
}

#[test]
fn wrapper_and_every_installed_string_have_exact_canonical_admission() {
    // Existing XML Document31 + HTML div namespace28/local3 =62 units/two nodes.
    for (nodes, units, success) in [(3, 63, false), (4, 62, false), (4, 63, true)] {
        let mut dom = Dom::new(DomLimits::new(nodes, units, 32, 0, 0, 8, 65536).unwrap()).unwrap();
        let (d, parent) = setup(&mut dom);
        let mut b = HtmlFragment::builder(authoring()).unwrap();
        b.append_text(&b.root(), "x").unwrap();
        let source = b.finish();
        let before = dom.snapshot();
        let usage = dom.usage();
        let result = dom.install_native_html_fragment(
            stamp(&dom, d),
            parent,
            None,
            &source,
            limits(),
            &AtomicBool::new(false),
        );
        if success {
            let report = result.unwrap();
            assert_eq!(report.bindings().len(), 2);
            assert_eq!(dom.usage().managed_nodes(), 4);
            assert_eq!(dom.usage().string_code_units(), 63);
        } else {
            assert!(matches!(
                result,
                Err(DomNativeHtmlInstallError::Source {
                    error: DomError::ResourceLimit { .. },
                    ..
                })
            ));
            unchanged(&dom, &before, usage, parent);
        }
    }
}

#[test]
fn deep_native_authoring_installs_iteratively_without_a_second_live_tree() {
    let mut dom = dom();
    let (d, parent) = setup(&mut dom);
    let mut b = HtmlFragment::builder(authoring()).unwrap();
    let mut next = b.root();
    for _ in 0..200 {
        next = b
            .append_element(
                &next,
                name(Namespace::none(), "n"),
                OrderedAttributes::default(),
            )
            .unwrap();
    }
    b.append_text(&next, DocumentString::from_utf16(vec![0xd800, 0, 0xdc00]))
        .unwrap();
    let source = b.finish();
    let report = install(&mut dom, d, parent, &source);
    assert_eq!(report.bindings().len(), 202);
    let mut node = report.snapshot().node(parent).unwrap().children()[0];
    for _ in 0..199 {
        node = report.snapshot().node(node).unwrap().children()[0];
    }
    let text = report.snapshot().node(node).unwrap().children()[0];
    assert_eq!(
        report
            .snapshot()
            .node(text)
            .unwrap()
            .data()
            .unwrap()
            .code_units(),
        &[0xd800, 0, 0xdc00]
    );
}
