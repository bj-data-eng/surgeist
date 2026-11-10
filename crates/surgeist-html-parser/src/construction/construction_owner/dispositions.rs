//! The parser's report of each refusal that HTML's construction owner returns, driven by
//! errors the owner itself produces.
use super::*;
use crate::parser::control::{HtmlParseInvariant, HtmlParseStop};
use surgeist_html::{
    bounded::construction::HtmlConstructionCommand,
    construction::{
        HtmlBuilderNode, HtmlConstructionLimits, HtmlDocument, HtmlDocumentMode, HtmlElement,
        HtmlFragment, HtmlNodeKind, HtmlShadowRootMode, HtmlShadowRootProperties,
        HtmlSlotAssignment,
    },
    values::{DocumentString, Namespace, OrderedAttributes, QualifiedName},
};

fn invariant(invariant: HtmlParseInvariant) -> SessionError {
    SessionError::Stop(HtmlParseStop::InternalInvariant(invariant))
}

fn html_name(local: &str) -> QualifiedName {
    QualifiedName::new(Namespace::html(), None, local)
}

/// The report of `error` and the stop it latched.
fn reported(error: HtmlConstructionError) -> (SessionError, Option<HtmlParseStop>) {
    let mut control = crate::construction::tests::control();
    let session = construction_error(&mut control, error);
    (session, control.stop())
}

fn create(
    owner: &mut ConstructionOwner,
    control: &mut ParseControl,
    command: HtmlConstructionCommand,
) -> HtmlBuilderNode {
    owner
        .apply(control, command, &[(HtmlParseResource::ManagedNodes, 1)])
        .unwrap()
        .unwrap()
}

fn create_element(
    owner: &mut ConstructionOwner,
    control: &mut ParseControl,
    local: &str,
) -> HtmlBuilderNode {
    create(
        owner,
        control,
        HtmlConstructionCommand::Create {
            kind: HtmlNodeKind::Element(HtmlElement::new(
                html_name(local),
                OrderedAttributes::default(),
            )),
        },
    )
}

/// Runs a command the owner refuses and returns its report and the stop it latched.
fn refused(
    owner: &mut ConstructionOwner,
    control: &mut ParseControl,
    command: HtmlConstructionCommand,
) -> (SessionError, Option<HtmlParseStop>) {
    let error = owner
        .apply(control, command, &[(HtmlParseResource::ManagedEdges, 1)])
        .unwrap_err();
    (error, control.stop())
}

/// A handle, placement or cycle refusal describes the document's own shape: the session error
/// names it and nothing is latched.
#[test]
fn document_shaped_owner_refusals_report_their_session_error_and_latch_nothing() {
    let mut control = crate::construction::tests::control();
    let mut owner = ConstructionOwner::new(&mut control).unwrap();
    let document = create(
        &mut owner,
        &mut control,
        HtmlConstructionCommand::CreateDocument {
            mode: HtmlDocumentMode::NoQuirks,
        },
    );
    let comment = create(
        &mut owner,
        &mut control,
        HtmlConstructionCommand::Create {
            kind: HtmlNodeKind::Comment(DocumentString::default()),
        },
    );
    let outer = create_element(&mut owner, &mut control, "div");
    let inner = create_element(&mut owner, &mut control, "span");
    let stray = create_element(&mut owner, &mut control, "p");
    let template = create(
        &mut owner,
        &mut control,
        HtmlConstructionCommand::CreateElement {
            name: html_name("template"),
            attributes: OrderedAttributes::default(),
            link_flags: None,
        },
    );
    let contents = owner
        .node_at(owner.test_storage().len() - 1)
        .unwrap()
        .handle();
    let insert =
        |node: &HtmlBuilderNode, parent: &HtmlBuilderNode, before: Option<&HtmlBuilderNode>| {
            HtmlConstructionCommand::Insert {
                node: *node,
                parent: *parent,
                before: before.cloned(),
            }
        };
    let mut foreign_owner_control = crate::construction::tests::control();
    let mut foreign_owner = ConstructionOwner::new(&mut foreign_owner_control).unwrap();
    let foreign = create_element(&mut foreign_owner, &mut foreign_owner_control, "b");

    assert_eq!(
        refused(&mut owner, &mut control, insert(&foreign, &outer, None)),
        (SessionError::ForeignNode, None),
        "a handle of another owner"
    );
    assert_eq!(
        refused(
            &mut owner,
            &mut control,
            HtmlConstructionCommand::Detach { node: document }
        ),
        (SessionError::RootMove, None),
        "moving a document"
    );
    assert_eq!(
        refused(&mut owner, &mut control, insert(&contents, &outer, None)),
        (SessionError::AssociationMove, None),
        "moving template contents as an ordinary child"
    );
    assert_eq!(
        refused(&mut owner, &mut control, insert(&inner, &comment, None)),
        (SessionError::NotContainer, None),
        "a child under a comment"
    );
    assert_eq!(
        refused(&mut owner, &mut control, insert(&inner, &template, None)),
        (SessionError::TemplateContentsRequired, None),
        "a child directly under a template"
    );
    assert_eq!(
        refused(
            &mut owner,
            &mut control,
            insert(&inner, &outer, Some(&stray))
        ),
        (SessionError::WrongSibling, None),
        "a reference that is not a child of the parent"
    );
    owner
        .apply(
            &mut control,
            insert(&inner, &outer, None),
            &[(HtmlParseResource::ManagedEdges, 1)],
        )
        .unwrap();
    assert_eq!(
        refused(&mut owner, &mut control, insert(&outer, &inner, None)),
        (SessionError::Cycle, None),
        "a node under its own descendant"
    );
}

/// A request the parser's sequencing guarantees valid is an owner refusal, which latches the
/// construction-owner invariant.
#[test]
fn an_owner_refusal_of_a_sequenced_request_latches_the_owner_refusal_invariant() {
    let mut control = crate::construction::tests::control();
    let mut owner = ConstructionOwner::new(&mut control).unwrap();
    let refusal = refused(
        &mut owner,
        &mut control,
        HtmlConstructionCommand::Create {
            kind: HtmlNodeKind::Document(HtmlDocumentMode::NoQuirks),
        },
    );
    let expected = invariant(HtmlParseInvariant::ConstructionOwnerRefusal);
    assert_eq!(
        refusal,
        (
            expected,
            Some(HtmlParseStop::InternalInvariant(
                HtmlParseInvariant::ConstructionOwnerRefusal
            ))
        )
    );

    let limits = HtmlConstructionLimits::new(100, 100, 100, 100_000, 100);
    let properties = HtmlShadowRootProperties::new(
        HtmlShadowRootMode::Open,
        HtmlSlotAssignment::Named,
        false,
        false,
        false,
    );
    let mut builder = HtmlDocument::builder(limits, HtmlDocumentMode::NoQuirks).unwrap();
    let root = builder.root();
    let leaf = builder.append_comment(&root, "").unwrap();
    let host = builder
        .append_element(&root, html_name("div"), OrderedAttributes::default())
        .unwrap();
    builder
        .attach_declarative_shadow(&host, properties)
        .unwrap();
    for error in [
        builder
            .attach_declarative_shadow(&leaf, properties)
            .unwrap_err(),
        builder
            .attach_declarative_shadow(&host, properties)
            .unwrap_err(),
    ] {
        assert_eq!(
            reported(error),
            (
                expected,
                Some(HtmlParseStop::InternalInvariant(
                    HtmlParseInvariant::ConstructionOwnerRefusal
                ))
            )
        );
    }
}

/// A handle of another builder and a child under a leaf are document-shaped, as the builder
/// reports them.
#[test]
fn builder_handle_and_parent_refusals_report_document_shaped_session_errors() {
    let limits = HtmlConstructionLimits::new(100, 100, 100, 100_000, 100);
    let other = HtmlFragment::builder(limits).unwrap();
    let mut builder = HtmlFragment::builder(limits).unwrap();
    let foreign = builder.append_text(&other.root(), "x").unwrap_err();
    assert_eq!(reported(foreign), (SessionError::ForeignNode, None));

    let mut builder = HtmlDocument::builder(limits, HtmlDocumentMode::NoQuirks).unwrap();
    let root = builder.root();
    let leaf = builder.append_comment(&root, "").unwrap();
    let under_leaf = builder.append_text(&leaf, "x").unwrap_err();
    assert_eq!(reported(under_leaf), (SessionError::NotContainer, None));
    let template = builder
        .append_element(&root, html_name("template"), OrderedAttributes::default())
        .unwrap();
    let under_template = builder.append_text(&template, "x").unwrap_err();
    assert_eq!(
        reported(under_template),
        (SessionError::TemplateContentsRequired, None)
    );
}

/// A limit the owner reports disagrees with what the controller charged, since the controller
/// admits every resource before the owner is asked.
#[test]
fn an_owner_limit_refusal_latches_the_capacity_mismatch_invariant() {
    let mut builder = HtmlFragment::builder(HtmlConstructionLimits::new(3, 2, 0, 2, 2)).unwrap();
    let root = builder.root();
    let error = builder
        .append_text(&root, DocumentString::from("too long"))
        .unwrap_err();
    let mismatch = HtmlParseInvariant::CapacityMismatch;
    assert_eq!(
        reported(error),
        (
            invariant(mismatch),
            Some(HtmlParseStop::InternalInvariant(mismatch))
        )
    );
}
