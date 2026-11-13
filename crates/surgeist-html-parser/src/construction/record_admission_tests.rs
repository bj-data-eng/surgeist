//! Tests of the session's side of the record operations: what the controller admits and records
//! for each, the report of every refusal, and the release of what the parse holds for the storage.
use super::*;
use crate::parser::control::{HtmlParseInvariant, HtmlParseStop, WalkSite};
use surgeist_html::{
    construction::{
        HtmlBuilderNode, HtmlConstructionLimits, HtmlDocument, HtmlDocumentMode,
        HtmlShadowRootMode, HtmlShadowRootProperties, HtmlSlotAssignment,
    },
    values::{Namespace, OrderedAttributes, QualifiedName},
};

fn invariant(invariant: HtmlParseInvariant) -> SessionError {
    SessionError::Stop(HtmlParseStop::InternalInvariant(invariant))
}

fn html_name(local: &str) -> QualifiedName {
    QualifiedName::new(Namespace::html(), None, local)
}

/// The report of `error` and the stop it latched.
fn reported(
    error: surgeist_html::construction::HtmlConstructionError,
) -> (SessionError, Option<HtmlParseStop>) {
    let mut control = crate::construction::tests::control();
    let session = construction_error(&mut control, error);
    (session, control.stop())
}

/// A session over a fragment root.
fn fresh() -> (ParseControl, ParserArenaSession) {
    let mut control = crate::construction::tests::control();
    let session = ParserArenaSession::new(&mut control, RootKind::Fragment).unwrap();
    (control, session)
}

/// Allocates a record the way the parser does, with the bookkeeping the session keeps for it.
fn allocate(
    storage: &mut SessionStorage,
    control: &mut ParseControl,
    kind: NewRecord<'_>,
) -> HtmlBuilderNode {
    let node = storage.allocate(control, kind, false, 0, 0).unwrap();
    node.handle
}

fn allocate_element(
    storage: &mut SessionStorage,
    control: &mut ParseControl,
    local: &str,
    template: bool,
) -> HtmlBuilderNode {
    let id = storage
        .intern_qualified_name(
            control,
            html_name(local).namespace(),
            None,
            html_name(local).local_name().code_units(),
        )
        .unwrap();
    let node = storage
        .allocate(control, NewRecord::NamedElement(id, None), template, 0, 0)
        .unwrap();
    node.handle
}

/// Inserts under the same limits the parser's own insertion uses and returns the report and the
/// stop it latched.
fn refused_insert(
    storage: &mut SessionStorage,
    control: &mut ParseControl,
    node: &HtmlBuilderNode,
    parent: &HtmlBuilderNode,
    before: Option<&HtmlBuilderNode>,
) -> (SessionError, Option<HtmlParseStop>) {
    let position = storage.construction.position(parent).unwrap();
    let error = storage
        .insert_record(
            control,
            node,
            (position, parent),
            before,
            false,
            &[(HtmlParseResource::ManagedEdges, 1)],
        )
        .unwrap_err();
    (error, control.stop())
}

/// A handle, placement or cycle refusal describes the document's own shape: the session error
/// names it and nothing is latched.
#[test]
fn document_shaped_record_refusals_report_their_session_error_and_latch_nothing() {
    let (mut control, mut session) = fresh();
    let storage = session.active_mut().unwrap();
    let document = allocate(
        storage,
        &mut control,
        NewRecord::Document(HtmlDocumentMode::NoQuirks),
    );
    let comment = allocate(storage, &mut control, NewRecord::Comment(&[]));
    let outer = allocate_element(storage, &mut control, "div", false);
    let inner = allocate_element(storage, &mut control, "span", false);
    let stray = allocate_element(storage, &mut control, "p", false);
    let template = allocate_element(storage, &mut control, "template", true);
    let contents = storage
        .construction
        .node_at(storage.construction.len() - 1)
        .unwrap()
        .handle();
    let mut foreign_control = crate::construction::tests::control();
    let mut foreign_session =
        ParserArenaSession::new(&mut foreign_control, RootKind::Fragment).unwrap();
    let foreign_storage = foreign_session.active_mut().unwrap();
    let foreign = allocate_element(foreign_storage, &mut foreign_control, "b", false);

    assert_eq!(
        refused_insert(storage, &mut control, &foreign, &outer, None),
        (SessionError::ForeignNode, None),
        "a handle of another storage"
    );
    assert_eq!(
        storage.detach_record(&mut control, &document),
        Err(SessionError::RootMove),
        "moving a document"
    );
    assert_eq!(control.stop(), None);
    assert_eq!(
        refused_insert(storage, &mut control, &contents, &outer, None),
        (SessionError::AssociationMove, None),
        "moving template contents as an ordinary child"
    );
    assert_eq!(
        refused_insert(storage, &mut control, &inner, &comment, None),
        (SessionError::NotContainer, None),
        "a child under a comment"
    );
    assert_eq!(
        refused_insert(storage, &mut control, &inner, &template, None),
        (SessionError::TemplateContentsRequired, None),
        "a child directly under a template"
    );
    assert_eq!(
        refused_insert(storage, &mut control, &inner, &outer, Some(&stray)),
        (SessionError::WrongSibling, None),
        "a reference that is not a child of the parent"
    );
    let outer_position = storage.construction.position(&outer).unwrap();
    storage
        .insert_record(
            &mut control,
            &inner,
            (outer_position, &outer),
            None,
            false,
            &[(HtmlParseResource::ManagedEdges, 1)],
        )
        .unwrap();
    assert_eq!(
        refused_insert(storage, &mut control, &outer, &inner, None),
        (SessionError::Cycle, None),
        "a node under its own descendant"
    );
    session.release(&mut control).unwrap();
}

/// A request the parser's sequencing guarantees valid is a refusal of the storage, which latches
/// the construction-owner invariant.
#[test]
fn a_storage_refusal_of_a_sequenced_request_latches_the_owner_refusal_invariant() {
    let (mut control, mut session) = fresh();
    let storage = session.active_mut().unwrap();
    let comment = allocate(storage, &mut control, NewRecord::Comment(&[]));
    let error = storage
        .set_document_mode_record(&mut control, &comment, HtmlDocumentMode::Quirks)
        .unwrap_err();
    let expected = invariant(HtmlParseInvariant::ConstructionOwnerRefusal);
    assert_eq!(error, expected);
    assert_eq!(
        control.stop(),
        Some(HtmlParseStop::InternalInvariant(
            HtmlParseInvariant::ConstructionOwnerRefusal
        ))
    );
    session.release(&mut control).unwrap();

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

fn parser_insertion_pair(
    limits: crate::HtmlParseLimits,
) -> (ParseControl, ParserArenaSession, ParserNode, ParserNode) {
    let mut control = ParseControl::new(limits, crate::HtmlParseCancellation::new());
    let mut session = ParserArenaSession::new(&mut control, RootKind::Fragment).unwrap();
    let root = session.root(&mut control).unwrap();
    let child = session
        .allocate_element(
            &mut control,
            PreexistingElement::new(
                QualifiedName::new(Namespace::none(), None, "x"),
                OrderedAttributes::default(),
            ),
        )
        .unwrap();
    (control, session, root, child)
}

#[test]
fn parser_insertion_keeps_depth_and_edge_preflight_before_scratch_capacity() {
    for resource in [
        HtmlParseResource::TreeDepth,
        HtmlParseResource::ManagedEdges,
    ] {
        let limits = crate::construction::tests::limits();
        let (mut probe, mut session, _, _) = parser_insertion_pair(limits);
        let capacity = probe.usage().actual_capacity_bytes;
        session.release(&mut probe).unwrap();
        assert_eq!(probe.usage().actual_capacity_bytes, 0);
        let builder = limits.into_builder().managed_capacity_bytes(capacity);
        let limits = match resource {
            HtmlParseResource::TreeDepth => builder.tree_depth(0),
            HtmlParseResource::ManagedEdges => builder.managed_edges(0),
            _ => unreachable!(),
        }
        .finish()
        .unwrap();
        let (mut control, mut session, root, child) = parser_insertion_pair(limits);
        let auxiliary = control
            .usage()
            .resource(HtmlParseResource::AuxiliaryEntries);
        let result = session.insert(&mut control, &root, &child, None, None);
        let stop = control.stop().unwrap();
        assert!(
            matches!(stop, HtmlParseStop::LimitExceeded { resource: actual, .. } if actual == resource),
            "{stop:?}"
        );
        assert!(matches!(result, Err(SessionError::Stop(actual)) if actual == stop));
        let storage = session.active().unwrap();
        assert_eq!(
            storage
                .construction
                .node_at(root.index)
                .unwrap()
                .children()
                .count(),
            0
        );
        assert!(
            storage
                .construction
                .node_at(child.index)
                .unwrap()
                .parent()
                .is_none()
        );
        assert!(storage.effects.form.is_none());
        assert_eq!(control.usage().actual_capacity_bytes, capacity);
        assert_eq!(
            control
                .usage()
                .resource(HtmlParseResource::ManagedEdges)
                .current,
            0
        );
        assert_eq!(
            control
                .usage()
                .resource(HtmlParseResource::AuxiliaryEntries),
            auxiliary
        );
        let work = control.usage().resource(HtmlParseResource::WorkCredits);
        session.release(&mut control).unwrap();
        assert_eq!(control.usage().actual_capacity_bytes, 0);
        assert_eq!(
            control
                .usage()
                .resource(HtmlParseResource::AuxiliaryEntries)
                .current,
            0
        );
        assert_eq!(
            control.usage().resource(HtmlParseResource::WorkCredits),
            work
        );
        assert_eq!(control.stop(), Some(stop));
    }
}

fn registry_row() -> HtmlParseStop {
    HtmlParseStop::InternalInvariant(HtmlParseInvariant::RegistryRow)
}

fn assert_registry_row<T>(result: Result<T, SessionError>, control: &ParseControl) {
    assert_eq!(
        result.err(),
        Some(SessionError::Stop(registry_row())),
        "typed registry-row error"
    );
    assert_eq!(
        control.stop(),
        Some(registry_row()),
        "latched as the first stop"
    );
}

#[test]
fn a_refused_retirement_still_releases_everything_the_session_holds() {
    use crate::construction::effects::tests::{graph, request};
    let request = request("<body><div id=a>");
    let (mut c, mut session, _proof, _nodes) = graph(&request);
    c.refuse_next_construction_retire();
    assert_registry_row(session.release(&mut c), &c);
    crate::construction::tests::storage_zero(&c);
}

// Characterization: a cleanup walk that meets a refusal keeps going, so every other owner is still
// released or credited, and it reports the first refusal. A test hook makes the credit of the
// banks refuse after its work, reported as a wrong token.
#[test]
fn a_refused_bank_credit_still_releases_everything_the_session_holds() {
    use crate::construction::effects::tests::{graph, request};
    let request = request("<body><div id=a>");
    let (mut c, mut session, _proof, _nodes) = graph(&request);
    c.refuse_walk_steps(WalkSite::ConstructionBank, 1);
    let result = session.release(&mut c);
    assert_eq!(result.err(), Some(SessionError::WrongToken));
    crate::construction::tests::storage_zero(&c);
}

#[test]
fn a_refused_frozen_owner_still_releases_every_other_owner_and_reports_the_first_refusal() {
    use crate::construction::effects::tests::{graph, request};
    let request = request("<body><div id=a>");
    let (mut c, mut session, _proof, _nodes) = graph(&request);
    let mut frozen = session.freeze(&mut c).unwrap();
    c.refuse_walk_steps(WalkSite::FrozenOwner, 2);
    let result = frozen.release(&mut c);
    assert_eq!(result.err(), Some(SessionError::WrongToken));
    assert_eq!(c.stop(), Some(registry_row()));
    crate::construction::tests::storage_zero(&c);
}

/// A session whose root is followed by one element, the target of attribute edits.
fn with_element() -> (ParseControl, ParserArenaSession) {
    let (mut control, mut session) = fresh();
    allocate_element(session.active_mut().unwrap(), &mut control, "x", false);
    (control, session)
}

#[test]
fn an_edit_of_an_absent_record_is_a_foreign_node() {
    let (mut control, mut session) = with_element();
    let result = session
        .active_mut()
        .unwrap()
        .reserve_attributes(&mut control, 7, 1);
    assert_eq!(result, Err(SessionError::ForeignNode));
    assert_eq!(control.stop(), None);
    session.release(&mut control).unwrap();
    assert_eq!(control.usage().actual_capacity_bytes, 0);
}

fn units(text: &str) -> Vec<u16> {
    text.encode_utf16().collect()
}

fn held(storage: &SessionStorage) -> Vec<String> {
    match storage.construction.node_at(1).unwrap().kind() {
        surgeist_html::construction::ConstructionNodeKind::Element(element) => {
            test_support::attributes_of(element)
                .into_iter()
                .map(|held| held.name().local_name().to_utf8().unwrap())
                .collect()
        }
        _ => panic!("an element"),
    }
}

#[test]
fn an_append_that_the_capacity_limit_refuses_leaves_the_list_and_every_figure_as_it_was() {
    let (mut control, mut session) = with_element();
    let storage = session.active_mut().unwrap();
    let name = storage
        .intern_name(&mut control, &Namespace::none(), &units("a"))
        .unwrap();
    let before = control.usage().clone();
    let held_bytes = before
        .resource(HtmlParseResource::ManagedCapacityBytes)
        .current();
    let (banks, strings) = (storage.holds.bank_bytes, storage.holds.string_bytes);
    control.test_limit(HtmlParseResource::ManagedCapacityBytes, held_bytes);
    let result = storage.append_named_attribute(&mut control, 1, &name, &units("v"), None, 0);
    assert!(matches!(
        result,
        Err(SessionError::Stop(HtmlParseStop::LimitExceeded {
            resource: HtmlParseResource::ManagedCapacityBytes,
            ..
        }))
    ));
    assert!(held(storage).is_empty());
    assert_eq!(
        control
            .usage()
            .resource(HtmlParseResource::ManagedCapacityBytes)
            .current(),
        held_bytes,
        "no capacity was charged for the refused growth"
    );
    assert_eq!(
        (storage.holds.bank_bytes, storage.holds.string_bytes),
        (banks, strings)
    );
    session.release(&mut control).unwrap();
    assert_eq!(control.usage().actual_capacity_bytes, 0);
}

#[test]
fn appended_attributes_extend_the_run_of_their_element_and_are_released_with_the_storage() {
    let (mut control, mut session) = with_element();
    let storage = session.active_mut().unwrap();
    let banks = storage.holds.bank_bytes;
    for name in ["a", "b", "c", "d", "e"] {
        let id = storage
            .intern_name(&mut control, &Namespace::none(), &units(name))
            .unwrap();
        storage
            .append_named_attribute(&mut control, 1, &id, &units("v"), None, 0)
            .unwrap();
    }
    assert_eq!(held(storage), ["a", "b", "c", "d", "e"]);
    // The run is the last the table holds, so each append extends it in place by the one row it
    // needs.
    let dimensions = test_support::run_dimensions(storage, &mut control, 1);
    assert_eq!((dimensions.initialized_len, dimensions.capacity), (5, 5));
    assert!(
        storage.holds.bank_bytes
            >= banks + (dimensions.capacity * dimensions.element_size()) as u64,
        "the parse holds the bytes of the rows its banks reserved"
    );
    session.release(&mut control).unwrap();
    assert_eq!(control.usage().actual_capacity_bytes, 0);
}

#[test]
fn a_refused_record_after_a_bank_reserve_leaves_the_bank_credited_for_what_was_admitted() {
    let (mut control, mut session) = fresh();
    let held = control
        .usage()
        .resource(HtmlParseResource::ManagedCapacityBytes)
        .current;
    control.test_limit(HtmlParseResource::ManagedCapacityBytes, held + 16_000);
    control.overshoot_next_capacity_record(1_000_000);
    let result = session.active_mut().unwrap().create_record(
        &mut control,
        NewRecord::Fragment,
        &[(HtmlParseResource::ManagedNodes, 1)],
    );
    assert!(matches!(
        result,
        Err(SessionError::Stop(HtmlParseStop::LimitExceeded {
            resource: HtmlParseResource::ManagedCapacityBytes,
            ..
        }))
    ));
    session.release(&mut control).unwrap();
    crate::parser::tree_builder::tests::assert_released(&control);
}

#[test]
fn a_logical_limit_refuses_an_attribute_before_any_bank_grows() {
    use surgeist_html::values::Attribute;
    // An element of `count` attributes: `held` short ones and, when `count` exceeds that, one
    // whose value is longer than the room the text arena has. The limit on attributes admits
    // `held`, so the last one is refused by the limit. Its append would grow the text arena and,
    // when the run is full, the rows, so the capacity the parse holds after the refusal must be
    // what it holds for the admitted attributes and the interned name of the refused one.
    let long = "x".repeat(5000);
    let element = |held: usize, count: usize| {
        let mut attributes: Vec<Attribute> = (0..held)
            .map(|index| {
                Attribute::new(
                    QualifiedName::new(Namespace::none(), None, format!("a{index}")),
                    "v",
                )
            })
            .collect();
        if count > held {
            attributes.push(Attribute::new(
                QualifiedName::new(Namespace::none(), None, "z"),
                long.as_str(),
            ));
        }
        PreexistingElement::new(
            QualifiedName::new(Namespace::none(), None, "x"),
            OrderedAttributes::new(attributes).unwrap(),
        )
    };
    let mut full_runs = 0;
    for held in 1..=4_usize {
        let limits = crate::construction::tests::limits()
            .into_builder()
            .managed_attributes(held as u64)
            .finish()
            .unwrap();
        // The reference holds the admitted attributes and the name of the refused one.
        let mut reference = ParseControl::new(limits, crate::HtmlParseCancellation::new());
        let mut session = ParserArenaSession::new(&mut reference, RootKind::Fragment).unwrap();
        let node = session
            .allocate_element(&mut reference, element(held, held))
            .unwrap();
        let dimensions = crate::construction::test_support::run_dimensions(
            session.active_mut().unwrap(),
            &mut reference,
            node.index,
        );
        full_runs += usize::from(dimensions.initialized_len == dimensions.capacity);
        session
            .active_mut()
            .unwrap()
            .intern_qualified_name(&mut reference, &Namespace::none(), None, &[u16::from(b'z')])
            .unwrap();
        let expected = reference.usage().actual_capacity_bytes;
        session.release(&mut reference).unwrap();

        let mut control = ParseControl::new(limits, crate::HtmlParseCancellation::new());
        let mut session = ParserArenaSession::new(&mut control, RootKind::Fragment).unwrap();
        let refused = session.allocate_element(&mut control, element(held, held + 1));
        assert!(
            matches!(
                refused,
                Err(SessionError::Stop(HtmlParseStop::LimitExceeded {
                    resource: HtmlParseResource::ManagedAttributes,
                    ..
                }))
            ),
            "held {held}: {refused:?}"
        );
        assert_eq!(
            control.usage().actual_capacity_bytes,
            expected,
            "held {held}: a refused attribute grew a bank"
        );
        session.release(&mut control).unwrap();
    }
    assert!(full_runs > 0, "no count made the refused append need rows");
}

#[test]
fn a_units_limit_refuses_a_longer_replacement_value_before_the_text_arena_grows() {
    use surgeist_html::values::Attribute;
    let element = || {
        PreexistingElement::new(
            QualifiedName::new(Namespace::none(), None, "x"),
            OrderedAttributes::new([Attribute::new(
                QualifiedName::new(Namespace::none(), None, "n"),
                "v",
            )])
            .unwrap(),
        )
    };
    let value: Vec<u16> = "y".repeat(5000).encode_utf16().collect();
    let units = |control: &ParseControl| {
        control
            .usage()
            .resource(HtmlParseResource::ManagedUtf16Units)
            .current
    };
    // The units the element holds, with room for 10 more: the replacement adds 4999.
    let mut reference = ParseControl::new(
        crate::construction::tests::limits(),
        crate::HtmlParseCancellation::new(),
    );
    let mut session = ParserArenaSession::new(&mut reference, RootKind::Fragment).unwrap();
    session.allocate_element(&mut reference, element()).unwrap();
    let held = units(&reference);
    session.release(&mut reference).unwrap();

    let limits = crate::construction::tests::limits()
        .into_builder()
        .managed_utf16_units(held + 10)
        .finish()
        .unwrap();
    let mut control = ParseControl::new(limits, crate::HtmlParseCancellation::new());
    let mut session = ParserArenaSession::new(&mut control, RootKind::Fragment).unwrap();
    let node = session.allocate_element(&mut control, element()).unwrap();
    let id = session
        .active_mut()
        .unwrap()
        .intern_qualified_name(&mut control, &Namespace::none(), None, &[u16::from(b'n')])
        .unwrap();
    let capacity = control.usage().actual_capacity_bytes;
    let refused = session.active_mut().unwrap().replace_named_attribute(
        &mut control,
        node.index,
        0,
        &id,
        &value,
    );
    assert!(
        matches!(
            refused,
            Err(SessionError::Stop(HtmlParseStop::LimitExceeded {
                resource: HtmlParseResource::ManagedUtf16Units,
                ..
            }))
        ),
        "{refused:?}"
    );
    assert_eq!(units(&control), held, "a refused replacement kept units");
    assert_eq!(
        control.usage().actual_capacity_bytes,
        capacity,
        "a refused replacement grew a bank"
    );
    session.release(&mut control).unwrap();
}

#[test]
fn removing_or_replacing_a_namespaced_attribute_is_refused_and_changes_nothing() {
    use surgeist_html::values::{Attribute, DocumentString};
    let xlink = || {
        QualifiedName::new(
            Namespace::from_uri(DocumentString::from("http://www.w3.org/1999/xlink")),
            Some(DocumentString::from("xlink")),
            "href",
        )
    };
    let element = || {
        PreexistingElement::new(
            QualifiedName::new(Namespace::none(), None, "x"),
            OrderedAttributes::new([
                Attribute::new(xlink(), "value"),
                Attribute::new(QualifiedName::new(Namespace::none(), None, "id"), "i"),
            ])
            .unwrap(),
        )
    };
    let held = |control: &ParseControl| {
        let usage = control.usage();
        (
            usage.resource(HtmlParseResource::ManagedUtf16Units).current,
            usage.resource(HtmlParseResource::ManagedAttributes).current,
            usage.actual_capacity_bytes,
        )
    };
    for replace in [false, true] {
        let mut control = ParseControl::new(
            crate::construction::tests::limits(),
            crate::HtmlParseCancellation::new(),
        );
        let mut session = ParserArenaSession::new(&mut control, RootKind::Fragment).unwrap();
        let node = session.allocate_element(&mut control, element()).unwrap();
        let attributes = |session: &ParserArenaSession| {
            let storage = session.active().unwrap();
            let Some(ConstructionNodeKind::Element(element)) =
                storage.construction.node_at(node.index).map(|r| r.kind())
            else {
                panic!("an element record");
            };
            crate::construction::test_support::attribute_units(element)
        };
        let before = (held(&control), attributes(&session));
        let storage = session.active_mut().unwrap();
        let refused = if replace {
            let id = storage
                .intern_qualified_name(&mut control, &Namespace::none(), None, &[u16::from(b'n')])
                .unwrap();
            storage.replace_named_attribute(&mut control, node.index, 0, &id, &[u16::from(b'v')])
        } else {
            storage.remove_attribute(&mut control, node.index, 0)
        };
        assert_eq!(
            refused,
            Err(SessionError::Stop(HtmlParseStop::InternalInvariant(
                crate::parser::control::HtmlParseInvariant::RegistryRow
            ))),
            "replace: {replace}"
        );
        // The interned name of the replacement is held by the table of names, which is not an
        // edit of the attribute, so only the units and slots the attribute holds are compared.
        let after = (held(&control), attributes(&session));
        assert_eq!(after.0.0, before.0.0, "units, replace: {replace}");
        assert_eq!(after.0.1, before.0.1, "slots, replace: {replace}");
        assert_eq!(after.1, before.1, "attributes, replace: {replace}");
        session.release(&mut control).unwrap();
    }
    // An attribute without a namespace or a prefix is credited as before.
    let mut control = ParseControl::new(
        crate::construction::tests::limits(),
        crate::HtmlParseCancellation::new(),
    );
    let mut session = ParserArenaSession::new(&mut control, RootKind::Fragment).unwrap();
    let node = session.allocate_element(&mut control, element()).unwrap();
    let units = control
        .usage()
        .resource(HtmlParseResource::ManagedUtf16Units)
        .current;
    session
        .active_mut()
        .unwrap()
        .remove_attribute(&mut control, node.index, 1)
        .unwrap();
    // The attribute `id` with the value `i` counted 3 units.
    assert_eq!(
        control
            .usage()
            .resource(HtmlParseResource::ManagedUtf16Units)
            .current,
        units - 3
    );
    session.release(&mut control).unwrap();
}
