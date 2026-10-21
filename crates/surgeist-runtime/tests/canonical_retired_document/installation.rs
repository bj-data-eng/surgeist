//! Native installation through Runtime's actual Source entry. The retired
//! positive case lives in foundation; these controls retain the same installer.
use super::support::{HostedFixture, body, cancel, source_limits};
use std::sync::atomic::AtomicBool;
use surgeist_dom::*;
use surgeist_html::{HtmlConstructionLimits, HtmlDoctype, HtmlFragment};
use surgeist_runtime::*;

fn authoring() -> HtmlConstructionLimits {
    HtmlConstructionLimits::new(32, 32, 16, 4096, 16)
}

fn installation_limits() -> DomNativeHtmlInstallLimits {
    DomNativeHtmlInstallLimits::new(32, 100_000, 1_000_000)
}

fn fragment() -> HtmlFragment {
    let mut builder = HtmlFragment::builder(authoring()).unwrap();
    let root = builder.root();
    let element = builder
        .append_element(
            &root,
            QualifiedName::new(Namespace::html(), None, "div"),
            OrderedAttributes::default(),
        )
        .unwrap();
    builder
        .append_text(&element, DocumentString::from_utf16(vec![0, 0xd800, 10]))
        .unwrap();
    builder.append_comment(&root, "native control").unwrap();
    builder.finish()
}

fn target(
    fixture: &HostedFixture,
    document: DocumentId,
    parent: NodeId,
) -> DomNativeHtmlInstallTarget {
    DomNativeHtmlInstallTarget::new(
        fixture
            .session
            .snapshot()
            .document(document)
            .unwrap()
            .stamp(),
        parent,
        None,
    )
}

#[expect(
    clippy::result_large_err,
    reason = "the helper preserves the actual root and original producer failure for exact driving refusal assertions"
)]
fn install(
    fixture: &mut HostedFixture,
    source: &HtmlFragment,
    target: DomNativeHtmlInstallTarget,
    limits: DomNativeHtmlInstallLimits,
    cancelled: &AtomicBool,
) -> Result<DomSourceDriveProgress, DomSourceDriveError> {
    let mut runtime = fixture.session.source_execution();
    let mut policy = |_: &DomStyleSource| None;
    runtime.install_native_html(
        &mut fixture.session.dom(),
        target,
        source,
        DomSourceNativeHtmlInstallLimits::new(limits, source_limits()),
        &mut DomSourceServices {
            meta: None,
            execution_scope: None,
            style_policy: &mut policy,
            url: None,
        },
        cancelled,
    )
}

fn refused(
    fixture: &mut HostedFixture,
    source: &HtmlFragment,
    target: DomNativeHtmlInstallTarget,
    limits: DomNativeHtmlInstallLimits,
    cancelled: &AtomicBool,
) -> DomNativeHtmlInstallError {
    match install(fixture, source, target, limits, cancelled).unwrap_err() {
        surgeist_runtime::DomSourceDriveError::Source(
            DomSourceRuntimeError::NativeInstallation(error),
        ) => error,
        other => panic!("the native installer must retain its precise refusal: {other:?}"),
    }
}

fn unchanged(fixture: &mut HostedFixture, before: &DomSnapshot, usage: DomUsage) {
    let current = fixture.session.snapshot();
    assert_eq!(current.stamp(), before.stamp());
    assert_eq!(fixture.session.dom().usage(), usage);
    assert_eq!(
        current.nodes().collect::<Vec<_>>(),
        before.nodes().collect::<Vec<_>>()
    );
    for id in before.nodes() {
        let actual = current.node(id).unwrap();
        let original = before.node(id).unwrap();
        assert_eq!(actual.kind(), original.kind());
        assert_eq!(actual.parent(), original.parent());
        assert_eq!(actual.owner_document(), original.owner_document());
        assert_eq!(actual.children(), original.children());
    }
    assert!(fixture.session.dom().pending_source_edit().is_none());
    assert_eq!(current.incomplete_source_operation(), None);
    assert_eq!(current.incomplete_hosted_operation(), None);
    fixture.assert_retired();
}

#[test]
fn completed_native_fragment_has_active_and_never_hosted_destination_controls() {
    let mut fixture = HostedFixture::active();
    let source = fragment();
    let source_children: Vec<_> = source.root().children().collect();
    let source_element = source.node(&source_children[0]).unwrap();
    let source_text = source_element.children().next().unwrap();
    for hosted in [true, false] {
        let document = if hosted {
            fixture.top.document()
        } else {
            fixture.closed
        };
        let before = fixture.session.snapshot();
        let parent = body(&before, document);
        let target = target(&fixture, document, parent);
        let expected_registry = if hosted {
            fixture.registry
        } else {
            DomRegistryBinding::Null
        };
        let expected_global = hosted.then_some(fixture.top.global());
        let DomSourceDriveProgress::Complete(completion) = install(
            &mut fixture,
            &source,
            target,
            installation_limits(),
            &cancel(),
        )
        .expect("ordinary native nodes need canonical facts, not external services") else {
            panic!("ordinary native installation must complete without a service request")
        };
        let report = completion.native_installation().unwrap();
        assert_eq!(report.destination(), target.basis());
        assert_eq!(report.document(), document);
        assert_eq!(report.source_root(), &source.root().id());
        assert_eq!(report.bindings().len(), 4);
        let element = report.node_for(&source_children[0]).unwrap();
        let text = report.node_for(&source_text).unwrap();
        let comment = report.node_for(&source_children[1]).unwrap();
        let current = fixture.session.snapshot();
        assert_eq!(
            current.node(parent).unwrap().children(),
            &[element, comment]
        );
        assert!(before.node(parent).unwrap().children().is_empty());
        assert_eq!(current.node(element).unwrap().children(), &[text]);
        assert_eq!(current.node(text).unwrap().parent(), Some(element));
        assert_eq!(current.node(comment).unwrap().parent(), Some(parent));
        let value = current.node(element).unwrap().as_element().unwrap();
        assert_eq!(value.name().namespace_uri(), Namespace::html().uri());
        assert_eq!(value.name().prefix(), None);
        assert_eq!(value.name().local_name(), &DocumentString::from("div"));
        assert_eq!(value.custom_element_registry(), expected_registry);
        assert!(matches!(
            current.node(text).unwrap().kind(),
            DomNodeKind::Text(data) if data.code_units() == [0, 0xd800, 10]
        ));
        assert!(matches!(
            current.node(comment).unwrap().kind(),
            DomNodeKind::Comment(data) if data == &DocumentString::from("native control")
        ));
        assert!(matches!(
            current.node(report.root()).unwrap().kind(),
            DomNodeKind::DocumentFragment
        ));
        assert!(current.node(report.root()).unwrap().children().is_empty());
        assert_eq!(current.node(report.root()).unwrap().parent(), None);
        for id in [report.root(), element, text, comment] {
            assert!(matches!(before.node(id), Err(DomError::StaleNode)));
            assert_eq!(current.node(id).unwrap().owner_document(), Some(document));
            assert_eq!(current.relevant_global(id).unwrap(), expected_global);
        }
        assert_eq!(
            current.document(document).unwrap().metadata(),
            before.document(document).unwrap().metadata()
        );
        assert_eq!(
            current
                .document(document)
                .unwrap()
                .is_fully_active()
                .unwrap(),
            hosted
        );
        assert_eq!(current.incomplete_source_operation(), None);
        assert!(fixture.session.dom().pending_source_edit().is_none());
    }
    assert!(fixture.retirement.is_none());
    let hosted = fixture.session.hosted_execution();
    assert!(
        hosted
            .settings(&fixture.session.dom(), fixture.top.global(), &cancel())
            .is_ok()
    );
}

#[test]
fn retired_native_installation_preserves_target_qualification_and_finite_refusals() {
    let mut fixture = HostedFixture::retired();
    let document = fixture.top.document();
    let source = fragment();
    let old = target(&fixture, document, fixture.body);
    let mut batch = DomBatch::new();
    let stale = batch.create_fragment(document);
    let text = batch.create_text(document, "not a container");
    let reference = batch.create_comment(document, "detached reference");
    let commit = fixture.session.dom().apply(batch, &cancel()).unwrap();
    let stale = commit.created(&stale).unwrap().unwrap();
    let text = commit.created(&text).unwrap().unwrap();
    let reference = commit.created(&reference).unwrap().unwrap();
    let mut batch = DomBatch::new();
    batch.destroy(stale);
    fixture.session.dom().apply(batch, &cancel()).unwrap();
    let before = fixture.session.snapshot();
    let usage = fixture.session.dom().usage();
    let current = target(&fixture, document, fixture.body);
    let zero = DomNativeHtmlInstallLimits::new(0, 0, 0);
    assert!(matches!(
        refused(&mut fixture, &source, old, zero, &cancel()),
        DomNativeHtmlInstallError::StaleDestination { expected, actual }
            if expected == old.basis() && actual == before.stamp()
    ));
    unchanged(&mut fixture, &before, usage);
    for (parent, reference, expected) in [
        (stale, None, DomError::StaleNode),
        (text, None, DomError::HierarchyRequest),
        (fixture.body, Some(reference), DomError::NotFound),
    ] {
        assert!(matches!(
            refused(
                &mut fixture,
                &source,
                DomNativeHtmlInstallTarget::new(current.basis(), parent, reference),
                zero,
                &cancel(),
            ),
            DomNativeHtmlInstallError::Target(error) if error == expected
        ));
        unchanged(&mut fixture, &before, usage);
    }
    let mut foreign = Dom::new(fixture.session.dom().limits()).unwrap();
    let mut batch = DomBatch::new();
    let foreign_document = batch.create_document();
    let commit = foreign.apply(batch, &cancel()).unwrap();
    let foreign_document = commit.document(&foreign_document).unwrap();
    let foreign_basis = foreign
        .snapshot()
        .document(foreign_document)
        .unwrap()
        .stamp();
    for foreign_target in [
        DomNativeHtmlInstallTarget::new(foreign_basis, foreign_document.node(), None),
        DomNativeHtmlInstallTarget::new(current.basis(), foreign_document.node(), None),
    ] {
        assert!(matches!(
            refused(&mut fixture, &source, foreign_target, zero, &cancel()),
            DomNativeHtmlInstallError::Target(DomError::ForeignOwner)
        ));
        unchanged(&mut fixture, &before, usage);
    }
    for (limits, resource, limit) in [
        (
            DomNativeHtmlInstallLimits::new(3, 100_000, 1_000_000),
            DomNativeHtmlInstallResource::MappedNodes,
            3,
        ),
        (
            DomNativeHtmlInstallLimits::new(4, 0, 1_000_000),
            DomNativeHtmlInstallResource::Work,
            0,
        ),
        (
            DomNativeHtmlInstallLimits::new(4, 100_000, 0),
            DomNativeHtmlInstallResource::ScratchBytes,
            0,
        ),
    ] {
        assert!(matches!(
            refused(&mut fixture, &source, current, limits, &cancel()),
            DomNativeHtmlInstallError::ResourceLimit { resource: actual_resource, limit: actual_limit }
                if actual_resource == resource && actual_limit == limit
        ));
        unchanged(&mut fixture, &before, usage);
    }
    assert!(matches!(
        refused(
            &mut fixture,
            &source,
            current,
            installation_limits(),
            &AtomicBool::new(true),
        ),
        DomNativeHtmlInstallError::Cancelled
    ));
    unchanged(&mut fixture, &before, usage);
}

#[test]
fn retired_native_installation_keeps_family_cause_and_late_failure_atomic() {
    let mut fixture = HostedFixture::retired();
    let target = target(&fixture, fixture.top.document(), fixture.body);
    let before = fixture.session.snapshot();
    let usage = fixture.session.dom().usage();
    let mut builder = HtmlFragment::builder(authoring()).unwrap();
    let root = builder.root();
    builder.append_text(&root, "private prefix").unwrap();
    builder
        .append_element(
            &root,
            QualifiedName::new(Namespace::html(), None, "audio"),
            OrderedAttributes::default(),
        )
        .unwrap();
    let source = builder.finish();
    let audio = source.root().children().nth(1).unwrap();
    assert!(matches!(
        refused(&mut fixture, &source, target, installation_limits(), &cancel()),
        DomNativeHtmlInstallError::Source {
            subject,
            error: DomError::NativeInitializationRequired(surgeist_html::HtmlElementClass::KnownHtml(
                surgeist_html::KnownHtmlElement::Audio,
            )),
        } if subject == audio
    ));
    unchanged(&mut fixture, &before, usage);

    let mut builder = HtmlFragment::builder(authoring()).unwrap();
    builder
        .append_element(
            &builder.root(),
            QualifiedName::new(Namespace::html(), None, "template"),
            OrderedAttributes::default(),
        )
        .unwrap();
    let source = builder.finish();
    let template = source.root().children().next().unwrap();
    let contents = source.node(&template).unwrap().template_contents().unwrap();
    let hosted_source = source.fragment(&contents).unwrap().unwrap();
    assert!(matches!(
        refused(&mut fixture, &hosted_source, target, installation_limits(), &cancel()),
        DomNativeHtmlInstallError::UnsupportedAssociation {
            subject,
            association: DomNativeHtmlAssociation::HostedSourceRoot,
        } if subject == contents
    ));
    unchanged(&mut fixture, &before, usage);

    let mut builder = HtmlFragment::builder(authoring()).unwrap();
    let root = builder.root();
    builder.append_text(&root, "private prefix").unwrap();
    builder
        .append_doctype(&root, HtmlDoctype::new("html", None, None))
        .unwrap();
    let source = builder.finish();
    let doctype = source.root().children().nth(1).unwrap();
    assert!(matches!(
        refused(&mut fixture, &source, target, installation_limits(), &cancel()),
        DomNativeHtmlInstallError::Source { subject, error: DomError::HierarchyRequest }
            if subject == doctype
    ));
    unchanged(&mut fixture, &before, usage);
}
