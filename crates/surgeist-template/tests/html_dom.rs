#![forbid(unsafe_code)]

use std::sync::atomic::AtomicBool;

use surgeist_dom::{
    DocumentId, DocumentString, Dom, DomBatch, DomDocumentClass, DomError, DomLimits, DomResource,
    DomSnapshot, Namespace, NodeId,
};
use surgeist_html::{HtmlNativeFormAssociationKind, HtmlNativeInitializationRequirement};
use surgeist_template::{
    AttributeKind, AttributeRule, ComponentRegistry, ComponentSpec, NativeElementRegistry,
    SourcePos, SourceSpan, TemplateDomError, TemplateDomUnresolvedKind as Unresolved,
    ValidatedTemplate, ValidationErrorKind, append_static_html, parse_template, validate_template,
};

fn forms() -> AttributeRule {
    AttributeRule::any(
        AttributeKind::Static,
        [
            AttributeKind::Bool,
            AttributeKind::Expression,
            AttributeKind::Interpolated,
        ],
    )
}

fn validated(source: &str) -> ValidatedTemplate {
    let authored = parse_template(source).expect("strict template");
    validate_template(
        &authored,
        &NativeElementRegistry::from_html_metadata(forms()).unwrap(),
        &ComponentRegistry::default(),
    )
    .expect("HTML inventory validation")
}

fn dom() -> Dom {
    Dom::new(DomLimits::new(128, 32768, 512, 0, 0, 16, 262144).unwrap()).unwrap()
}

fn setup(dom: &mut Dom) -> (DocumentId, NodeId, NodeId) {
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let parent = batch.create_fragment(&document);
    let marker = batch.create_comment(&document, "before");
    batch.insert_before(&marker, &parent, None);
    let commit = dom.apply(batch, &AtomicBool::new(false)).unwrap();
    (
        commit.document(&document).unwrap(),
        commit.created(&parent).unwrap().unwrap(),
        commit.created(&marker).unwrap().unwrap(),
    )
}

fn span(start: usize, end: usize) -> SourceSpan {
    SourceSpan::try_new(
        SourcePos::try_new(1, start + 1, start).unwrap(),
        SourcePos::try_new(1, end + 1, end).unwrap(),
    )
    .unwrap()
}

#[test]
fn html_registry_unions_global_and_contextual_metadata_with_explicit_forms() {
    let native = NativeElementRegistry::from_html_metadata(forms()).unwrap();
    // `for` and `type` occur in separate contextual rows. Selecting only their
    // first rows would reject output/ol/input, rather than unioning applicability.
    for source in [
        r#"<div class="one"><span title="two"/></div>"#,
        r#"<label for="one"/><output for="one two"/>"#,
        r#"<ol type="A"/><input type="number" value="7"/>"#,
        r#"<template for="items"/>"#,
    ] {
        assert!(
            validate_template(
                &parse_template(source).unwrap(),
                &native,
                &ComponentRegistry::default()
            )
            .is_ok(),
            "{source}"
        );
    }
    for (source, element, attribute) in [
        (r#"<div href="x"/>"#, "div", "href"),
        (r#"<span for="x"/>"#, "span", "for"),
        (r#"<div data-extra="x"/>"#, "div", "data-extra"),
    ] {
        let error = validate_template(
            &parse_template(source).unwrap(),
            &native,
            &ComponentRegistry::default(),
        )
        .unwrap_err();
        assert!(
            matches!(error.kind(), ValidationErrorKind::InvalidAttribute { element: actual, attribute: name } if actual == element && name == attribute)
        );
    }
    let unknown = validate_template(
        &parse_template("<x-widget/>").unwrap(),
        &native,
        &ComponentRegistry::default(),
    )
    .unwrap_err();
    assert!(
        matches!(unknown.kind(), ValidationErrorKind::UnknownNativeElement { name } if name == "x-widget")
    );
    let static_only =
        NativeElementRegistry::from_html_metadata(AttributeRule::one(AttributeKind::Static))
            .unwrap();
    let authored = parse_template("<div hidden/>").unwrap();
    assert!(validate_template(&authored, &native, &ComponentRegistry::default()).is_ok());
    assert!(
        matches!(validate_template(&authored, &static_only, &ComponentRegistry::default()).unwrap_err().kind(), ValidationErrorKind::InvalidAttributeValue { element, attribute } if element == "div" && attribute == "hidden")
    );
}

#[test]
fn static_html_appends_ordered_real_nodes_and_retains_the_old_snapshot() {
    let mut dom = dom();
    let (document, parent, marker) = setup(&mut dom);
    let old = dom.snapshot();
    let mut batch = DomBatch::new();
    let roots = append_static_html(
        &validated(
            r#"<div id="outer" class="one"><span title="Ω">A😀</span>tail</div><p hidden/>"#,
        ),
        &mut batch,
        document.into(),
        parent.into(),
    )
    .unwrap();
    assert_eq!(roots.len(), 2);
    let after = batch.create_comment(document, "after");
    batch.insert_before(&after, parent, None);
    let commit = dom.apply(batch, &AtomicBool::new(false)).unwrap();
    let div = commit.created(&roots[0]).unwrap().unwrap();
    let p = commit.created(&roots[1]).unwrap().unwrap();
    let after = commit.created(&after).unwrap().unwrap();
    let snapshot = dom.snapshot();
    assert_eq!(
        snapshot.document(document).unwrap().metadata().class(),
        DomDocumentClass::Xml
    );
    assert_ne!(snapshot.stamp(), old.stamp());
    assert_eq!(
        snapshot.node(parent).unwrap().children(),
        &[marker, div, p, after]
    );
    assert_eq!(
        snapshot.node(after).unwrap().data().unwrap(),
        &DocumentString::from("after")
    );
    let node = snapshot.node(div).unwrap();
    assert_eq!(node.owner_document(), Some(document));
    assert_eq!(node.node_name(), DocumentString::from("div"));
    let element = node.as_element().unwrap();
    assert_eq!(element.name().namespace(), &Namespace::html());
    let attributes = element.attributes().as_slice();
    assert_eq!(attributes.len(), 2);
    assert_eq!(
        attributes[0].name().local_name(),
        &DocumentString::from("id")
    );
    assert_eq!(attributes[0].value(), &DocumentString::from("outer"));
    assert_eq!(
        attributes[1].name().local_name(),
        &DocumentString::from("class")
    );
    assert_eq!(attributes[1].value(), &DocumentString::from("one"));
    assert!(
        attributes
            .iter()
            .all(|a| a.name().namespace().uri().is_none())
    );
    let [span, tail] = node.children() else {
        panic!("span then tail")
    };
    let child = snapshot.node(*span).unwrap();
    assert_eq!(child.node_name(), DocumentString::from("span"));
    let element = child.as_element().unwrap();
    assert_eq!(element.name().namespace(), &Namespace::html());
    assert_eq!(
        element.attributes().as_slice()[0].value(),
        &DocumentString::from("Ω")
    );
    assert_eq!(child.children().len(), 1);
    assert_eq!(
        snapshot
            .node(child.children()[0])
            .unwrap()
            .data()
            .unwrap()
            .code_units(),
        &[65, 0xd83d, 0xde00]
    );
    assert_eq!(
        snapshot.node(*tail).unwrap().data().unwrap(),
        &DocumentString::from("tail")
    );
    let paragraph = snapshot.node(p).unwrap();
    assert_eq!(paragraph.node_name(), DocumentString::from("p"));
    let attributes = paragraph.as_element().unwrap().attributes().as_slice();
    assert_eq!(attributes.len(), 1);
    assert_eq!(
        attributes[0].name().local_name(),
        &DocumentString::from("hidden")
    );
    assert!(attributes[0].value().is_empty());
    assert_eq!(old.node(parent).unwrap().children(), &[marker]);
    assert_eq!(
        old.node(marker).unwrap().data().unwrap(),
        &DocumentString::from("before")
    );
    assert_eq!(old.nodes().count(), 3);
}

#[test]
fn html_roots_attach_to_an_actual_connected_xml_element_parent() {
    let mut dom = dom();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let parent = batch.create_element(&document, "container");
    batch.insert_before(&parent, &document, None);
    let roots = append_static_html(
        &validated("<span>live</span>"),
        &mut batch,
        (&document).into(),
        (&parent).into(),
    )
    .unwrap();
    let commit = dom.apply(batch, &AtomicBool::new(false)).unwrap();
    let snapshot = dom.snapshot();
    let parent = commit.created(&parent).unwrap().unwrap();
    let root = commit.created(&roots[0]).unwrap().unwrap();
    assert_eq!(snapshot.node(parent).unwrap().children(), &[root]);
    assert!(
        snapshot
            .node(parent)
            .unwrap()
            .as_element()
            .unwrap()
            .name()
            .namespace()
            .uri()
            .is_none()
    );
    let root = snapshot.node(root).unwrap();
    assert!(root.is_connected());
    assert_eq!(
        root.owner_document(),
        Some(commit.document(&document).unwrap())
    );
    assert_eq!(
        root.as_element().unwrap().name().namespace(),
        &Namespace::html()
    );
    assert_eq!(
        snapshot.node(root.children()[0]).unwrap().data().unwrap(),
        &DocumentString::from("live")
    );
}

#[test]
fn document_and_parent_creation_tickets_compose_in_one_real_batch() {
    let mut dom = dom();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let parent = batch.create_fragment(&document);
    let before = batch.len();
    assert!(
        append_static_html(
            &validated(""),
            &mut batch,
            (&document).into(),
            (&parent).into()
        )
        .unwrap()
        .is_empty()
    );
    assert_eq!(batch.len(), before);
    let roots = append_static_html(
        &validated("<div/><span/>"),
        &mut batch,
        (&document).into(),
        (&parent).into(),
    )
    .unwrap();
    let commit = dom.apply(batch, &AtomicBool::new(false)).unwrap();
    let parent = commit.created(&parent).unwrap().unwrap();
    let first = commit.created(&roots[0]).unwrap().unwrap();
    let second = commit.created(&roots[1]).unwrap().unwrap();
    let snapshot = dom.snapshot();
    assert_eq!(snapshot.node(parent).unwrap().children(), &[first, second]);
    assert_eq!(
        snapshot.node(first).unwrap().owner_document(),
        Some(commit.document(&document).unwrap())
    );
    assert_eq!(
        snapshot.node(second).unwrap().node_name(),
        DocumentString::from("span")
    );
}

fn assert_unchanged(snapshot: &DomSnapshot, dom: &Dom, parent: NodeId, marker: NodeId) {
    assert_eq!(dom.snapshot().stamp(), snapshot.stamp());
    assert_eq!(dom.snapshot().node(parent).unwrap().children(), &[marker]);
    assert_eq!(
        dom.snapshot().nodes().collect::<Vec<_>>(),
        snapshot.nodes().collect::<Vec<_>>()
    );
}

#[test]
fn late_dom_failure_rolls_back_the_whole_composed_batch() {
    let mut dom = dom();
    let (document, parent, marker) = setup(&mut dom);
    let old = dom.snapshot();
    let mut batch = DomBatch::new();
    append_static_html(
        &validated("<div><span>fresh</span></div>"),
        &mut batch,
        document.into(),
        parent.into(),
    )
    .unwrap();
    batch.insert_before(parent, parent, None);
    assert_eq!(
        dom.apply(batch, &AtomicBool::new(false)).unwrap_err(),
        DomError::HierarchyRequest
    );
    assert_unchanged(&old, &dom, parent, marker);
}

#[test]
fn actual_dom_capacity_failure_preserves_the_existing_tree_and_revision() {
    let mut dom = Dom::new(DomLimits::new(3, 32768, 512, 0, 0, 16, 262144).unwrap()).unwrap();
    let (document, parent, marker) = setup(&mut dom);
    let old = dom.snapshot();
    let mut batch = DomBatch::new();
    append_static_html(
        &validated("<div/>"),
        &mut batch,
        document.into(),
        parent.into(),
    )
    .unwrap();
    assert_eq!(
        dom.apply(batch, &AtomicBool::new(false)).unwrap_err(),
        DomError::ResourceLimit {
            resource: DomResource::ManagedNodes,
            limit: 3
        }
    );
    assert_eq!(dom.usage().managed_nodes(), 3);
    assert_unchanged(&old, &dom, parent, marker);
}

#[test]
fn real_dom_native_family_admission_is_deferred_to_apply() {
    let mut dom = dom();
    let (document, parent, marker) = setup(&mut dom);
    let old = dom.snapshot();
    let mut batch = DomBatch::new();
    let roots = append_static_html(
        &validated("<div/><object/>"),
        &mut batch,
        document.into(),
        parent.into(),
    )
    .unwrap();
    assert_eq!(roots.len(), 2);
    assert_eq!(
        dom.apply(batch, &AtomicBool::new(false)).unwrap_err(),
        DomError::NativeInitializationRequired(HtmlNativeInitializationRequirement::NativeForm(
            HtmlNativeFormAssociationKind::Object
        ))
    );
    assert_unchanged(&old, &dom, parent, marker);
}

fn preflight_rejection(source: &str, expected: TemplateDomError) {
    let authored = parse_template(source).unwrap();
    let native = NativeElementRegistry::from_html_metadata(forms()).unwrap();
    let components =
        ComponentRegistry::try_from_specs(vec![ComponentSpec::try_new("Panel", vec![]).unwrap()])
            .unwrap();
    let template = validate_template(&authored, &native, &components).unwrap();
    let mut dom = dom();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let parent = batch.create_fragment(&document);
    let sentinel = batch.create_text(&document, "caller-owned");
    batch.insert_before(&sentinel, &parent, None);
    let length = batch.len();
    assert_eq!(
        append_static_html(&template, &mut batch, (&document).into(), (&parent).into())
            .unwrap_err(),
        expected
    );
    assert_eq!(batch.len(), length);
    let after = batch.create_comment(&document, "after");
    batch.insert_before(&after, &parent, None);
    let commit = dom.apply(batch, &AtomicBool::new(false)).unwrap();
    let snapshot = dom.snapshot();
    assert_eq!(snapshot.nodes().count(), 4);
    assert_eq!(
        snapshot
            .node(commit.created(&parent).unwrap().unwrap())
            .unwrap()
            .children(),
        &[
            commit.created(&sentinel).unwrap().unwrap(),
            commit.created(&after).unwrap().unwrap()
        ]
    );
    assert_eq!(
        snapshot
            .node(commit.created(&sentinel).unwrap().unwrap())
            .unwrap()
            .data()
            .unwrap(),
        &DocumentString::from("caller-owned")
    );
}

#[test]
fn symbolic_nodes_after_a_concrete_prefix_leave_the_caller_batch_untouched() {
    for (source, kind) in [
        ("<div/><Panel/>", Unresolved::Component),
        ("<div/>{$x}", Unresolved::Interpolation),
        ("<div/>{if true}ok{/if}", Unresolved::Conditional),
        (
            "<div/>{foreach $items as $item}ok{/foreach}",
            Unresolved::Iteration,
        ),
    ] {
        preflight_rejection(
            source,
            TemplateDomError::Unresolved {
                span: span(6, source.len()),
                kind,
            },
        );
    }
    for (source, kind) in [
        (
            "<div/><span title={$x}><Panel/></span>",
            Unresolved::AttributeExpression,
        ),
        (
            "<div/><span title={42}><Panel/></span>",
            Unresolved::AttributeExpression,
        ),
        (
            r#"<div/><span title="Hi {$x}"/>"#,
            Unresolved::AttributeInterpolation,
        ),
    ] {
        let start = source.find("title").unwrap();
        let end = if kind == Unresolved::AttributeExpression {
            source.find("><Panel").unwrap()
        } else {
            source.rfind("/>").unwrap()
        };
        preflight_rejection(
            source,
            TemplateDomError::Unresolved {
                span: span(start, end),
                kind,
            },
        );
    }
}

#[test]
fn html_template_contents_need_an_explicit_supported_target() {
    let source = "<div/><template><span/></template>";
    preflight_rejection(
        source,
        TemplateDomError::UnsupportedTemplateContentsTarget {
            span: span(6, source.len()),
        },
    );
}
