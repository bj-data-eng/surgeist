use super::support::{HostedFixture, cancel, source_limits};
use std::sync::atomic::AtomicBool;
use surgeist_dom::*;
use surgeist_html::{HtmlConstructionLimits, HtmlFragment};
use surgeist_runtime::*;

// The expected kinds, names, data and identities follow DOM's creation steps.
// Actual lifecycle teardown changes execution permission, not these operands.
#[test]
fn ordinary_creation_uses_retained_document_facts_after_real_teardown() {
    let mut fixture = HostedFixture::retired();
    create_ordinary_nodes(&mut fixture, true);
    fixture.assert_retired();
}

#[test]
fn ordinary_creation_has_active_and_never_hosted_controls() {
    let mut fixture = HostedFixture::active();
    create_ordinary_nodes(&mut fixture, true);
    create_ordinary_nodes(&mut fixture, false);
    assert!(
        fixture
            .session
            .snapshot()
            .document(fixture.top.document())
            .unwrap()
            .is_fully_active()
            .unwrap()
    );
}

fn create_ordinary_nodes(fixture: &mut HostedFixture, hosted: bool) {
    let document = if hosted {
        fixture.top.document()
    } else {
        fixture.closed
    };
    let global = hosted.then_some(fixture.top.global());
    let registry = if hosted {
        fixture.registry
    } else {
        DomRegistryBinding::Null
    };
    let exact = DocumentString::from_utf16(vec![0, 0xd800, 13, 10]);
    let mut batch = DomBatch::new();
    let text = batch.create_text(document, exact.clone());
    let comment = batch.create_comment(document, "comment");
    let fragment = batch.create_fragment(document);
    let pi = batch.create_processing_instruction(document, "probe", "a='b'");
    let doctype = batch.create_document_type(document, "html", "public", "system");
    let native = batch.create_element(document, "DiV");
    let namespaced = batch.create_element_ns(document, Namespace::from_uri("urn:fact"), "p:MiXeD");
    let commit = fixture
        .session
        .dom()
        .apply(batch, &cancel())
        .expect("canonical creation needs retained Document facts, not a live Global");
    let snapshot = fixture.session.snapshot();
    for ticket in [
        &text,
        &comment,
        &fragment,
        &pi,
        &doctype,
        &native,
        &namespaced,
    ] {
        let id = commit.created(ticket).unwrap().unwrap();
        assert_eq!(snapshot.node(id).unwrap().owner_document(), Some(document));
        assert_eq!(snapshot.node(id).unwrap().parent(), None);
        assert_eq!(snapshot.relevant_global(id).unwrap(), global);
    }
    let text = snapshot
        .node(commit.created(&text).unwrap().unwrap())
        .unwrap();
    assert!(matches!(text.kind(), DomNodeKind::Text(data) if data == &exact));
    assert_eq!(text.node_name(), DocumentString::from("#text"));
    assert!(
        matches!(snapshot.node(commit.created(&comment).unwrap().unwrap()).unwrap().kind(), DomNodeKind::Comment(data) if data == &DocumentString::from("comment"))
    );
    assert!(matches!(
        snapshot
            .node(commit.created(&fragment).unwrap().unwrap())
            .unwrap()
            .kind(),
        DomNodeKind::DocumentFragment
    ));
    match snapshot
        .node(commit.created(&pi).unwrap().unwrap())
        .unwrap()
        .kind()
    {
        DomNodeKind::ProcessingInstruction(value) => {
            assert_eq!(value.target(), &DocumentString::from("probe"));
            assert_eq!(value.data(), &DocumentString::from("a='b'"));
        }
        other => panic!("actual PI kind: {other:?}"),
    }
    match snapshot
        .node(commit.created(&doctype).unwrap().unwrap())
        .unwrap()
        .kind()
    {
        DomNodeKind::DocumentType(value) => {
            assert_eq!(value.name(), &DocumentString::from("html"));
            assert_eq!(value.public_id(), &DocumentString::from("public"));
            assert_eq!(value.system_id(), &DocumentString::from("system"));
        }
        other => panic!("actual DocumentType kind: {other:?}"),
    }
    for (ticket, namespace, prefix, local) in [
        (&native, Namespace::html(), None, "div"),
        (
            &namespaced,
            Namespace::from_uri("urn:fact"),
            Some(DocumentString::from("p")),
            "MiXeD",
        ),
    ] {
        let element = snapshot
            .node(commit.created(ticket).unwrap().unwrap())
            .unwrap()
            .as_element()
            .unwrap();
        assert_eq!(element.name().namespace(), &namespace);
        assert_eq!(element.name().prefix(), prefix.as_ref());
        assert_eq!(element.name().local_name(), &DocumentString::from(local));
        assert_eq!(element.custom_element_registry(), registry);
    }
    if let DomRegistryBinding::Registry(id) = registry {
        let actual = snapshot.registry(id).unwrap();
        assert_eq!(actual.definition_count(), 0);
        assert!(
            actual
                .lookup_definition(&Namespace::html(), &"div".into(), None)
                .unwrap()
                .is_none()
        );
    }
}

#[test]
fn immutable_text_and_element_clones_get_fresh_retired_target_identities() {
    let mut fixture = HostedFixture::active();
    let exact = DocumentString::from_utf16(vec![0xd800, 0, 10]);
    let mut batch = DomBatch::new();
    let element = batch.create_element(fixture.closed, "div");
    batch.set_attribute(&element, "data-original", "retained");
    let text = batch.create_text(fixture.closed, exact.clone());
    batch.insert_before(&text, &element, None);
    let commit = fixture.session.dom().apply(batch, &cancel()).unwrap();
    let element = commit.created(&element).unwrap().unwrap();
    let text = commit.created(&text).unwrap().unwrap();
    let source = fixture.session.snapshot();
    fixture.retire();
    // import_node is the existing public clone algorithm, for both same-owner
    // immutable captures and a capture from a separate canonical owner.
    let mut foreign = Dom::new(DomLimits::new(32, 4096, 16, 0, 0, 4, 65536).unwrap()).unwrap();
    let mut batch = DomBatch::new();
    let foreign_doc = batch.create_document();
    let foreign_text = batch.create_text(&foreign_doc, exact.clone());
    let commit = foreign.apply(batch, &cancel()).unwrap();
    let foreign_text = commit.created(&foreign_text).unwrap().unwrap();
    let foreign_snapshot = foreign.snapshot();
    let mut batch = DomBatch::new();
    let copied = batch.import_node(&source, element, fixture.top.document(), true);
    let copied_text = batch.import_node(&source, text, fixture.top.document(), false);
    let imported = batch.import_node(
        &foreign_snapshot,
        foreign_text,
        fixture.top.document(),
        false,
    );
    let commit = fixture
        .session
        .dom()
        .apply(batch, &cancel())
        .expect("clone initialization consumes the destination's retained facts");
    let snapshot = fixture.session.snapshot();
    let copied = commit.created(&copied).unwrap().unwrap();
    let copied_text = commit.created(&copied_text).unwrap().unwrap();
    let imported = commit.created(&imported).unwrap().unwrap();
    assert_ne!(copied, element);
    assert_ne!(copied_text, text);
    assert_ne!(imported, foreign_text);
    assert_ne!(copied_text, imported);
    let copied_node = snapshot.node(copied).unwrap();
    assert_eq!(
        copied_node.attribute(&"data-original".into()),
        Some(&"retained".into())
    );
    assert_eq!(
        copied_node.as_element().unwrap().custom_element_registry(),
        fixture.registry
    );
    assert_eq!(copied_node.children().len(), 1);
    assert_ne!(copied_node.children()[0], text);
    for id in [copied_node.children()[0], copied_text, imported] {
        assert_eq!(snapshot.node(id).unwrap().data(), Some(&exact));
        assert_eq!(
            snapshot.node(id).unwrap().owner_document(),
            Some(fixture.top.document())
        );
        assert_eq!(
            snapshot.relevant_global(id).unwrap(),
            Some(fixture.top.global())
        );
    }
    assert_eq!(snapshot.node(element).unwrap().children(), &[text]);
    assert_eq!(
        snapshot
            .node(element)
            .unwrap()
            .attribute(&"data-original".into()),
        Some(&"retained".into())
    );
    assert_eq!(snapshot.node(text).unwrap().data(), Some(&exact));
    assert_eq!(
        snapshot.node(text).unwrap().owner_document(),
        Some(fixture.closed)
    );
    assert_eq!(source.node(element).unwrap().children(), &[text]);
    assert_eq!(
        source.node(text).unwrap().owner_document(),
        Some(fixture.closed)
    );
    fixture.assert_retired();
}

#[test]
fn xml_creator_reads_retained_origin_and_platform_but_creates_a_fresh_closed_document() {
    let platform = DomNativeInputPlatform::new(
        surgeist_html::HtmlInputEmptySelectionSupport::Unsupported,
        surgeist_html::HtmlInputNoneDirectionSupport::Supported,
    );
    let mut fixture = HostedFixture::active_with_platform(platform);
    fixture.retire();
    for (namespace, mime) in [
        (Namespace::none(), "application/xml"),
        (Namespace::html(), "application/xhtml+xml"),
        (Namespace::svg(), "image/svg+xml"),
    ] {
        let mut batch = DomBatch::new();
        let document =
            batch.create_xml_document(fixture.top.document(), namespace.clone(), "Root", None);
        let commit = fixture
            .session
            .dom()
            .apply(batch, &cancel())
            .expect("associated origin and fixed platform remain canonical facts");
        let document = commit.document(&document).unwrap();
        assert_ne!(document, fixture.top.document());
        let snapshot = fixture.session.snapshot();
        let actual = snapshot.document(document).unwrap();
        let metadata = actual.metadata();
        assert_eq!(metadata.class(), DomDocumentClass::Xml);
        assert_eq!(metadata.content_type(), &DocumentString::from(mime));
        assert_eq!(
            metadata.origin(),
            fixture
                .original
                .document(fixture.top.document())
                .unwrap()
                .metadata()
                .origin()
        );
        assert_eq!(metadata.document_environment().input_platform(), platform);
        assert_eq!(
            metadata.document_environment().browsing_context(),
            surgeist_html::HtmlBrowsingContextPresence::Absent
        );
        assert!(!metadata.document_environment().has_ever_had_navigable());
        assert!(!actual.is_fully_active().unwrap());
        assert_eq!(metadata.custom_element_registry(), DomRegistryBinding::Null);
        assert_eq!(snapshot.relevant_global(document.node()).unwrap(), None);
        let root = snapshot.node(actual.document_element().unwrap()).unwrap();
        assert_eq!(root.as_element().unwrap().name().namespace(), &namespace);
        assert_eq!(root.node_name(), DocumentString::from("Root"));
        assert_eq!(root.owner_document(), Some(document));
    }
    fixture.assert_retired();
}

#[test]
fn shadow_attachment_uses_retained_registry_with_reciprocal_links_and_atomic_refusals() {
    let mut fixture = HostedFixture::active();
    let mut batch = DomBatch::new();
    let host = batch.create_element(fixture.top.document(), "div");
    let second = batch.create_element(fixture.top.document(), "section");
    let invalid = batch.create_element(fixture.top.document(), "a");
    let commit = fixture.session.dom().apply(batch, &cancel()).unwrap();
    let host = commit.created(&host).unwrap().unwrap();
    let second = commit.created(&second).unwrap().unwrap();
    let invalid = commit.created(&invalid).unwrap().unwrap();
    fixture.retire();
    let mut batch = DomBatch::new();
    let root = batch.attach_shadow(host, DomShadowRootInit::new(HtmlShadowRootMode::Open));
    let commit = fixture
        .session
        .dom()
        .apply(batch, &cancel())
        .expect("attachShadow needs the real retained registry, not execution authority");
    let root = commit.created(&root).unwrap().unwrap();
    let snapshot = fixture.session.snapshot();
    let shadow = snapshot.shadow_root(root).unwrap();
    assert_eq!(shadow.host(), host);
    assert_eq!(shadow.state().custom_element_registry(), fixture.registry);
    assert_eq!(
        snapshot.node(host).unwrap().hosted_shadow_root(),
        Some(root)
    );
    assert_eq!(snapshot.node(host).unwrap().shadow_root(), Some(root));
    assert_eq!(snapshot.node(root).unwrap().parent(), None);
    assert_eq!(
        snapshot.node(root).unwrap().owner_document(),
        Some(fixture.top.document())
    );
    assert_eq!(
        snapshot.relevant_global(root).unwrap(),
        Some(fixture.top.global())
    );
    for target in [host, invalid] {
        let mut batch = DomBatch::new();
        batch.attach_shadow(target, DomShadowRootInit::new(HtmlShadowRootMode::Closed));
        assert_eq!(
            fixture.session.dom().apply(batch, &cancel()).unwrap_err(),
            DomError::NotSupported
        );
        assert_eq!(fixture.session.snapshot().stamp(), snapshot.stamp());
    }
    let before = fixture.session.snapshot();
    let usage = fixture.session.dom().usage();
    let mut batch = DomBatch::new();
    batch.attach_shadow(second, DomShadowRootInit::new(HtmlShadowRootMode::Open));
    // A later invalid attribute forces rollback after genuine root staging.
    batch.set_attribute(second, "bad name", "late");
    assert_eq!(
        fixture.session.dom().apply(batch, &cancel()).unwrap_err(),
        DomError::InvalidCharacter
    );
    assert_eq!(fixture.session.snapshot().stamp(), before.stamp());
    assert_eq!(fixture.session.dom().usage(), usage);
    assert_eq!(
        fixture
            .session
            .snapshot()
            .node(second)
            .unwrap()
            .hosted_shadow_root(),
        None
    );
    fixture.assert_retired();
}

fn fragment() -> HtmlFragment {
    let mut builder =
        HtmlFragment::builder(HtmlConstructionLimits::new(32, 32, 16, 4096, 16)).unwrap();
    let element = builder
        .append_element(
            &builder.root(),
            QualifiedName::new(Namespace::html(), None, "div"),
            OrderedAttributes::default(),
        )
        .unwrap();
    builder
        .append_text(&element, DocumentString::from_utf16(vec![0, 0xd800, 10]))
        .unwrap();
    builder
        .append_comment(&builder.root(), "native comment")
        .unwrap();
    builder.finish()
}

fn install(
    fixture: &mut HostedFixture,
    source: &HtmlFragment,
    target: DomNativeHtmlInstallTarget,
    limits: DomNativeHtmlInstallLimits,
    cancelled: &AtomicBool,
) -> Result<DomSourceDriveProgress, DomSourceRuntimeError> {
    let mut runtime = fixture.session.source_execution();
    let mut policy = |_: &DomStyleSource| None;
    runtime.install_native_html(
        &mut fixture.session.dom(),
        target,
        source,
        DomSourceNativeHtmlInstallLimits::new(limits, source_limits()),
        &mut DomSourceServices {
            style_policy: &mut policy,
            url: None,
        },
        cancelled,
    )
}

#[test]
fn completed_native_fragment_installs_into_retired_document_with_actual_fresh_mapping() {
    let mut fixture = HostedFixture::retired();
    let source = fragment();
    let source_children: Vec<_> = source.root().children().collect();
    let mut prior = None;
    for _ in 0..2 {
        let basis = fixture
            .session
            .snapshot()
            .document(fixture.top.document())
            .unwrap()
            .stamp();
        let target = DomNativeHtmlInstallTarget::new(basis, fixture.body, None);
        let DomSourceDriveProgress::Complete(completion) = install(
            &mut fixture,
            &source,
            target,
            DomNativeHtmlInstallLimits::new(32, 100_000, 1_000_000),
            &cancel(),
        )
        .expect("the original native installer consumes retained target facts") else {
            panic!("ordinary completed native nodes need no external service")
        };
        let report = completion.native_installation().unwrap();
        assert_eq!(report.document(), fixture.top.document());
        assert_eq!(report.destination(), basis);
        assert_eq!(report.source_root(), &source.root().id());
        assert_eq!(report.bindings().len(), 4);
        let element = report.node_for(&source_children[0]).unwrap();
        let comment = report.node_for(&source_children[1]).unwrap();
        let snapshot = fixture.session.snapshot();
        let node = snapshot.node(element).unwrap();
        assert_eq!(node.parent(), Some(fixture.body));
        assert_eq!(node.owner_document(), Some(fixture.top.document()));
        assert_eq!(
            node.as_element().unwrap().name(),
            &QualifiedName::new(Namespace::html(), None, "div")
        );
        assert_eq!(
            node.as_element().unwrap().custom_element_registry(),
            fixture.registry
        );
        assert_eq!(
            snapshot.relevant_global(element).unwrap(),
            Some(fixture.top.global())
        );
        assert_eq!(node.children().len(), 1);
        assert_eq!(
            snapshot.node(node.children()[0]).unwrap().data(),
            Some(&DocumentString::from_utf16(vec![0, 0xd800, 10]))
        );
        assert!(
            matches!(snapshot.node(comment).unwrap().kind(), DomNodeKind::Comment(data) if data == &DocumentString::from("native comment"))
        );
        assert!(snapshot.node(report.root()).unwrap().children().is_empty());
        assert!(matches!(
            snapshot.node(report.root()).unwrap().kind(),
            DomNodeKind::DocumentFragment
        ));
        if let Some((old_element, old_comment)) = prior {
            assert_ne!(element, old_element);
            assert_ne!(comment, old_comment);
        }
        prior = Some((element, comment));
    }
    fixture.assert_retired();
}

#[test]
fn malformed_foreign_stale_cancelled_and_late_failed_edits_leave_retired_document_unchanged() {
    let mut fixture = HostedFixture::retired();
    let before = fixture.session.snapshot();
    let usage = fixture.session.dom().usage();
    let mut foreign = Dom::new(DomLimits::new(8, 1024, 8, 0, 0, 1, 65536).unwrap()).unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let commit = foreign.apply(batch, &cancel()).unwrap();
    let foreign_document = commit.document(&document).unwrap();
    let mut batch = DomBatch::new();
    batch.create_text(foreign_document, "foreign");
    assert_eq!(
        fixture.session.dom().apply(batch, &cancel()).unwrap_err(),
        DomError::ForeignOwner
    );
    let mut batch = DomBatch::new();
    batch.create_element_ns(fixture.top.document(), Namespace::none(), "p:bad");
    assert_eq!(
        fixture.session.dom().apply(batch, &cancel()).unwrap_err(),
        DomError::Namespace
    );
    let mut batch = DomBatch::new();
    batch.create_text(fixture.top.document(), "cancelled");
    assert_eq!(
        fixture
            .session
            .dom()
            .apply(batch, &AtomicBool::new(true))
            .unwrap_err(),
        DomError::Cancelled
    );
    let mut batch = DomBatch::new();
    let staged = batch.create_element(fixture.top.document(), "div");
    batch.set_attribute(&staged, "bad name", "late");
    assert_eq!(
        fixture.session.dom().apply(batch, &cancel()).unwrap_err(),
        DomError::InvalidCharacter
    );
    assert_eq!(fixture.session.snapshot().stamp(), before.stamp());
    assert_eq!(fixture.session.dom().usage(), usage);
    let source = fragment();
    let old_basis = before.document(fixture.top.document()).unwrap().stamp();
    let mut batch = DomBatch::new();
    let removed = batch.create_comment(fixture.top.document(), "retire this exact generation");
    let commit = fixture.session.dom().apply(batch, &cancel()).unwrap();
    let removed = commit.created(&removed).unwrap().unwrap();
    let mut batch = DomBatch::new();
    batch.destroy(removed);
    fixture.session.dom().apply(batch, &cancel()).unwrap();
    let before = fixture.session.snapshot();
    let usage = fixture.session.dom().usage();
    let mut batch = DomBatch::new();
    batch.replace_data(removed, 0, 0, "stale");
    assert_eq!(
        fixture.session.dom().apply(batch, &cancel()).unwrap_err(),
        DomError::StaleNode
    );
    let old_target = DomNativeHtmlInstallTarget::new(old_basis, fixture.body, None);
    assert!(matches!(
        install(
            &mut fixture,
            &source,
            old_target,
            DomNativeHtmlInstallLimits::new(32, 100_000, 1_000_000),
            &cancel()
        ),
        Err(DomSourceRuntimeError::NativeInstallation(
            DomNativeHtmlInstallError::StaleDestination { .. }
        ))
    ));
    let target = DomNativeHtmlInstallTarget::new(
        before.document(fixture.top.document()).unwrap().stamp(),
        fixture.body,
        None,
    );
    assert!(matches!(
        install(
            &mut fixture,
            &source,
            target,
            DomNativeHtmlInstallLimits::new(0, 100_000, 1_000_000),
            &cancel()
        ),
        Err(DomSourceRuntimeError::NativeInstallation(
            DomNativeHtmlInstallError::ResourceLimit {
                resource: DomNativeHtmlInstallResource::MappedNodes,
                limit: 0
            }
        ))
    ));
    assert_eq!(fixture.session.snapshot().stamp(), before.stamp());
    assert_eq!(fixture.session.dom().usage(), usage);
    assert_eq!(
        fixture
            .session
            .snapshot()
            .node(fixture.body)
            .unwrap()
            .children(),
        before.node(fixture.body).unwrap().children()
    );
    fixture.assert_retired();
}

#[test]
fn retired_history_does_not_supply_the_independent_never_hosted_svg_script_proof() {
    let mut fixture = HostedFixture::retired();
    let before = fixture.session.snapshot();
    let mut batch = DomBatch::new();
    batch.create_element_ns(fixture.top.document(), Namespace::svg(), "script");
    assert!(matches!(
        fixture.session.dom().apply(batch, &cancel()),
        Err(DomError::NativeInitializationRequired(
            surgeist_html::HtmlElementClass::SvgScript
        ))
    ));
    assert_eq!(fixture.session.snapshot().stamp(), before.stamp());
    let mut batch = DomBatch::new();
    let script = batch.create_element_ns(fixture.closed, Namespace::svg(), "script");
    let commit = fixture.session.dom().apply(batch, &cancel()).unwrap();
    assert_eq!(
        fixture
            .session
            .snapshot()
            .node(commit.created(&script).unwrap().unwrap())
            .unwrap()
            .owner_document(),
        Some(fixture.closed)
    );
}
