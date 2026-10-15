use surgeist_html::*;
use surgeist_html_semantics::HtmlHostFactsRevision;
use surgeist_html_semantics::{
    HtmlCapture, HtmlCaptureBuilder, HtmlCaptureBuilderSubject, HtmlCaptureLimits,
    HtmlCaptureRegistryInput, HtmlCapturedIsValue, HtmlCapturedShadowProperties,
    HtmlDocumentFlatTreeRelation, HtmlDocumentModalBlocking, HtmlDocumentModalFact,
    HtmlFlatTreeProducerId, HtmlFlatTreeRevision, HtmlInertFactError, HtmlInertFactLimits,
    HtmlInertFacts, HtmlInertNodeFact, HtmlScriptingStatus, HtmlSemanticCancellation,
    HtmlSemanticContext, HtmlSemanticContextInput, HtmlSemanticDependencies, HtmlSemanticFact,
    HtmlSemanticFailureKind, HtmlSemanticLimits, HtmlSemanticMissing, HtmlSemanticQuery,
    HtmlSemanticReport, HtmlSemanticRequest, HtmlSemanticResource, HtmlSemanticSession,
    HtmlSemanticValue,
};

// Expectations follow HTML 877d6146487f §6.3.1–2 and the selected CSS Shadow
// 50337328484f §4.1 relation producer contract. These tests supply actual captured
// records; HTML consumes the supplied relations and never computes flattening.
fn builder() -> (HtmlCaptureBuilder, HtmlCaptureBuilderSubject) {
    let mut b = HtmlCaptureBuilder::new(
        HtmlCaptureLimits::new(1000, 10000, 1000, 100000, 1000, 0, 0),
        HtmlHostFactsRevision::fresh(),
    );
    let d = b
        .add_document(
            HtmlDocumentMode::NoQuirks,
            HtmlScriptingStatus::Unavailable,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    (b, d)
}
fn expanded(
    b: &mut HtmlCaptureBuilder,
    d: &HtmlCaptureBuilderSubject,
    namespace: Namespace,
    name: &str,
    attrs: &[(&str, &str)],
) -> HtmlCaptureBuilderSubject {
    b.add_element(
        d,
        QualifiedName::new(namespace, None, name),
        OrderedAttributes::new(attrs.iter().map(|(name, value)| {
            Attribute::new(QualifiedName::new(Namespace::none(), None, *name), *value)
        }))
        .unwrap(),
        HtmlCapturedIsValue::Unavailable,
        HtmlCaptureRegistryInput::Null,
    )
    .unwrap()
}
fn element(
    b: &mut HtmlCaptureBuilder,
    d: &HtmlCaptureBuilderSubject,
    name: &str,
    attrs: &[(&str, &str)],
) -> HtmlCaptureBuilderSubject {
    expanded(b, d, Namespace::html(), name, attrs)
}
fn key() -> HtmlFlatTreeRevision {
    HtmlFlatTreeRevision::new(HtmlFlatTreeProducerId::fresh(), 7, 12)
}
fn fact(
    c: &HtmlCapture,
    node: &HtmlCaptureBuilderSubject,
    connected: Option<bool>,
    relation: HtmlDocumentFlatTreeRelation,
) -> HtmlInertNodeFact {
    HtmlInertNodeFact::new(c.subject_id(node).unwrap(), connected, relation)
}
fn root(c: &HtmlCapture, node: &HtmlCaptureBuilderSubject) -> HtmlInertNodeFact {
    fact(c, node, Some(true), HtmlDocumentFlatTreeRelation::Root)
}
fn parent(
    c: &HtmlCapture,
    node: &HtmlCaptureBuilderSubject,
    parent: &HtmlCaptureBuilderSubject,
) -> HtmlInertNodeFact {
    fact(
        c,
        node,
        Some(true),
        HtmlDocumentFlatTreeRelation::Parent(c.subject_id(parent).unwrap()),
    )
}
fn modal(
    c: &HtmlCapture,
    d: &HtmlCaptureBuilderSubject,
    blocker: Option<&HtmlCaptureBuilderSubject>,
) -> HtmlDocumentModalFact {
    HtmlDocumentModalFact::new(
        c.subject_id(d).unwrap(),
        blocker.map_or(HtmlDocumentModalBlocking::Unblocked, |node| {
            HtmlDocumentModalBlocking::BlockedBy(c.subject_id(node).unwrap())
        }),
    )
}
fn supplement(
    c: &HtmlCapture,
    revision: HtmlFlatTreeRevision,
    nodes: Vec<HtmlInertNodeFact>,
    documents: Vec<HtmlDocumentModalFact>,
) -> HtmlInertFacts {
    HtmlInertFacts::new(
        c,
        c.host_facts_revision().clone(),
        revision,
        nodes,
        documents,
        HtmlInertFactLimits::new(100, 1000000),
    )
    .unwrap()
}
fn context(c: &HtmlCapture, facts: Option<HtmlInertFacts>) -> HtmlSemanticContext {
    let expected = facts
        .as_ref()
        .map_or_else(key, |f| f.flat_revision().clone());
    HtmlSemanticContext::new(
        c,
        c.host_facts_revision().clone(),
        HtmlSemanticContextInput::new(vec![]).with_inert(expected, facts),
        0,
    )
    .unwrap()
}
fn limits() -> HtmlSemanticLimits {
    HtmlSemanticLimits::new(1000000, 100, 1000, 1000, 1000000, 1000)
}
fn requests(c: &HtmlCapture, nodes: &[&HtmlCaptureBuilderSubject]) -> Vec<HtmlSemanticRequest> {
    nodes
        .iter()
        .map(|node| {
            HtmlSemanticRequest::new(c.subject_id(node).unwrap(), HtmlSemanticQuery::Inertness)
        })
        .collect()
}
fn query(
    c: &HtmlCapture,
    cx: &HtmlSemanticContext,
    nodes: &[&HtmlCaptureBuilderSubject],
) -> HtmlSemanticReport {
    HtmlSemanticSession::new(
        c,
        cx,
        HtmlValueInterpreter::new(10000),
        limits(),
        HtmlSemanticCancellation::new(),
    )
    .unwrap()
    .query(&requests(c, nodes))
    .unwrap()
}
fn value(report: &HtmlSemanticReport, index: usize) -> &HtmlSemanticValue<bool> {
    match report.results()[index].fact() {
        HtmlSemanticFact::Inertness(value) => value,
        other => panic!("expected inertness, got {other:?}"),
    }
}

#[test]
fn own_inert_presence_proves_true_without_any_host_supplement() {
    let (mut b, d) = builder();
    let e = element(&mut b, &d, "x-control", &[("inert", "false")]);
    let c = b.finish().unwrap();
    let cx = HtmlSemanticContext::new(
        &c,
        c.host_facts_revision().clone(),
        HtmlSemanticContextInput::new(vec![]),
        0,
    )
    .unwrap();
    let report = query(&c, &cx, &[&e]);
    assert_eq!(value(&report, 0), &HtmlSemanticValue::Known(true));
    assert_eq!(report.qualification().inert_revision(), cx.inert_revision());
    assert!(report.qualification().inert_revision().is_none());
    assert_eq!(report.capture(), c.identity());
    assert_eq!(
        report.dependencies(),
        HtmlSemanticDependencies::ConservativeRecompute
    );
}

#[test]
fn established_flat_ancestry_and_unblocked_document_decide_true_and_false() {
    let (mut b, d) = builder();
    let html = element(&mut b, &d, "html", &[]);
    let inert = element(&mut b, &d, "div", &[("inert", "")]);
    let child = b.add_text(&d, "child".into()).unwrap();
    let sibling = element(&mut b, &d, "button", &[]);
    b.establish_parent(&html, &d).unwrap();
    b.establish_children(&html, &[inert.clone(), sibling.clone()])
        .unwrap();
    b.establish_children(&inert, std::slice::from_ref(&child))
        .unwrap();
    let c = b.finish().unwrap();
    let cx = context(
        &c,
        Some(supplement(
            &c,
            key(),
            vec![
                root(&c, &html),
                parent(&c, &inert, &html),
                parent(&c, &child, &inert),
                parent(&c, &sibling, &html),
            ],
            vec![modal(&c, &d, None)],
        )),
    );
    let report = query(&c, &cx, &[&child, &sibling, &html]);
    assert_eq!(value(&report, 0), &HtmlSemanticValue::Known(true));
    assert_eq!(value(&report, 1), &HtmlSemanticValue::Known(false));
    assert_eq!(value(&report, 2), &HtmlSemanticValue::Known(false));
}

#[test]
fn assigned_node_inherits_from_actual_flat_slot_instead_of_ordinary_parent() {
    let (mut b, d) = builder();
    let html = element(&mut b, &d, "html", &[]);
    let host = element(&mut b, &d, "x-host", &[]);
    let light = element(&mut b, &d, "button", &[]);
    let shadow = b
        .add_shadow_root(
            &d,
            &host,
            HtmlCapturedShadowProperties::new(
                HtmlShadowRootProperties::new(
                    HtmlShadowRootMode::Closed,
                    HtmlSlotAssignment::Named,
                    false,
                    false,
                    false,
                ),
                false,
                false,
                false,
            ),
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    let slot = element(&mut b, &d, "slot", &[("inert", "")]);
    b.establish_parent(&html, &d).unwrap();
    b.establish_children(&html, std::slice::from_ref(&host))
        .unwrap();
    b.establish_children(&host, std::slice::from_ref(&light))
        .unwrap();
    b.establish_children(&shadow, std::slice::from_ref(&slot))
        .unwrap();
    let c = b.finish().unwrap();
    let cx = context(
        &c,
        Some(supplement(
            &c,
            key(),
            vec![
                root(&c, &html),
                parent(&c, &host, &html),
                parent(&c, &slot, &host),
                parent(&c, &light, &slot),
            ],
            vec![modal(&c, &d, None)],
        )),
    );
    assert_eq!(
        value(&query(&c, &cx, &[&light]), 0),
        &HtmlSemanticValue::Known(true)
    );
}

#[test]
fn only_current_topmost_blocker_escapes_ancestor_inertness() {
    let (mut b, d) = builder();
    let html = element(&mut b, &d, "html", &[]);
    let a = element(&mut b, &d, "dialog", &[("open", "")]);
    let container = element(&mut b, &d, "div", &[("inert", "")]);
    let lower = element(&mut b, &d, "dialog", &[("open", "")]);
    let child = element(&mut b, &d, "button", &[]);
    b.establish_parent(&html, &d).unwrap();
    b.establish_children(&html, std::slice::from_ref(&a))
        .unwrap();
    b.establish_children(&a, std::slice::from_ref(&container))
        .unwrap();
    b.establish_children(&container, std::slice::from_ref(&lower))
        .unwrap();
    b.establish_children(&lower, std::slice::from_ref(&child))
        .unwrap();
    let c = b.finish().unwrap();
    let records = || {
        vec![
            root(&c, &html),
            parent(&c, &a, &html),
            parent(&c, &container, &a),
            parent(&c, &lower, &container),
            parent(&c, &child, &lower),
        ]
    };
    let producer = HtmlFlatTreeProducerId::fresh();
    let revision = HtmlFlatTreeRevision::new(producer.clone(), 2, 1);
    let cx = context(
        &c,
        Some(supplement(
            &c,
            revision,
            records(),
            vec![modal(&c, &d, Some(&a))],
        )),
    );
    let report = query(&c, &cx, &[&a, &lower, &child]);
    assert_eq!(value(&report, 0), &HtmlSemanticValue::Known(false));
    assert_eq!(value(&report, 1), &HtmlSemanticValue::Known(true));
    assert_eq!(value(&report, 2), &HtmlSemanticValue::Known(true));
    let next = context(
        &c,
        Some(supplement(
            &c,
            HtmlFlatTreeRevision::new(producer, 2, 2),
            records(),
            vec![modal(&c, &d, Some(&lower))],
        )),
    );
    let next_report = query(&c, &next, &[&lower, &child, &a]);
    assert_eq!(value(&next_report, 0), &HtmlSemanticValue::Known(false));
    assert_eq!(value(&next_report, 1), &HtmlSemanticValue::Known(false));
    assert_eq!(value(&next_report, 2), &HtmlSemanticValue::Known(true));
    assert_ne!(report.qualification(), next_report.qualification());
}

#[test]
fn blocker_own_inert_wins_and_unknown_blocking_does_not_guess_escape() {
    let (mut b, d) = builder();
    let html = element(&mut b, &d, "html", &[]);
    let blocker = element(&mut b, &d, "dialog", &[("open", ""), ("inert", "")]);
    let child = element(&mut b, &d, "button", &[]);
    b.establish_parent(&html, &d).unwrap();
    b.establish_children(&html, std::slice::from_ref(&blocker))
        .unwrap();
    b.establish_children(&blocker, std::slice::from_ref(&child))
        .unwrap();
    let c = b.finish().unwrap();
    let records = || {
        vec![
            root(&c, &html),
            parent(&c, &blocker, &html),
            parent(&c, &child, &blocker),
        ]
    };
    let cx = context(
        &c,
        Some(supplement(
            &c,
            key(),
            records(),
            vec![modal(&c, &d, Some(&blocker))],
        )),
    );
    let report = query(&c, &cx, &[&blocker, &child]);
    assert_eq!(value(&report, 0), &HtmlSemanticValue::Known(true));
    assert_eq!(value(&report, 1), &HtmlSemanticValue::Known(true));
    let absent = context(&c, Some(supplement(&c, key(), records(), vec![])));
    assert_eq!(
        value(&query(&c, &absent, &[&child]), 0),
        &HtmlSemanticValue::Missing(HtmlSemanticMissing::ModalBlocking(
            c.subject_id(&d).unwrap()
        ))
    );
}

#[test]
fn connected_nodes_outside_blocker_including_known_excluded_light_are_forced_inert() {
    let (mut b, d) = builder();
    let html = element(&mut b, &d, "html", &[]);
    let blocker = element(&mut b, &d, "dialog", &[]);
    let outside = element(&mut b, &d, "div", &[]);
    let excluded = element(&mut b, &d, "span", &[]);
    b.establish_parent(&html, &d).unwrap();
    b.establish_children(&html, &[blocker.clone(), outside.clone(), excluded.clone()])
        .unwrap();
    let c = b.finish().unwrap();
    let cx = context(
        &c,
        Some(supplement(
            &c,
            key(),
            vec![
                root(&c, &html),
                parent(&c, &blocker, &html),
                parent(&c, &outside, &html),
                fact(
                    &c,
                    &excluded,
                    Some(true),
                    HtmlDocumentFlatTreeRelation::Excluded,
                ),
            ],
            vec![modal(&c, &d, Some(&blocker))],
        )),
    );
    let report = query(&c, &cx, &[&outside, &excluded]);
    assert_eq!(value(&report, 0), &HtmlSemanticValue::Known(true));
    assert_eq!(value(&report, 1), &HtmlSemanticValue::Known(true));
}

#[test]
fn detached_root_child_and_owner_document_blocker_do_not_fabricate_empty_ancestry() {
    let (mut b, d) = builder();
    let html = element(&mut b, &d, "html", &[]);
    let blocker = element(&mut b, &d, "dialog", &[]);
    let detached = element(&mut b, &d, "div", &[("inert", "")]);
    let child = element(&mut b, &d, "button", &[]);
    let empty_root = element(&mut b, &d, "span", &[]);
    b.establish_parent(&html, &d).unwrap();
    b.establish_children(&html, std::slice::from_ref(&blocker))
        .unwrap();
    b.establish_root(&detached).unwrap();
    b.establish_children(&detached, std::slice::from_ref(&child))
        .unwrap();
    b.establish_root(&empty_root).unwrap();
    let c = b.finish().unwrap();
    let records = || {
        vec![
            root(&c, &html),
            parent(&c, &blocker, &html),
            fact(
                &c,
                &detached,
                Some(false),
                HtmlDocumentFlatTreeRelation::Excluded,
            ),
            fact(
                &c,
                &child,
                Some(false),
                HtmlDocumentFlatTreeRelation::Excluded,
            ),
            fact(
                &c,
                &empty_root,
                Some(false),
                HtmlDocumentFlatTreeRelation::Excluded,
            ),
        ]
    };
    for blocking in [None, Some(&blocker)] {
        let cx = context(
            &c,
            Some(supplement(
                &c,
                key(),
                records(),
                vec![modal(&c, &d, blocking)],
            )),
        );
        let report = query(&c, &cx, &[&detached, &child, &empty_root]);
        assert_eq!(value(&report, 0), &HtmlSemanticValue::Known(true));
        assert_eq!(
            value(&report, 1),
            &HtmlSemanticValue::Missing(HtmlSemanticMissing::InertAncestry(
                c.subject_id(&child).unwrap()
            ))
        );
        assert_eq!(
            value(&report, 2),
            &HtmlSemanticValue::Missing(HtmlSemanticMissing::InertAncestry(
                c.subject_id(&empty_root).unwrap()
            ))
        );
    }
    let no_modal = context(&c, Some(supplement(&c, key(), records(), vec![])));
    assert_eq!(
        value(&query(&c, &no_modal, &[&empty_root]), 0),
        &HtmlSemanticValue::Missing(HtmlSemanticMissing::InertAncestry(
            c.subject_id(&empty_root).unwrap()
        ))
    );
}

#[test]
fn absent_records_unknown_relation_and_known_exclusion_stay_distinct() {
    let (mut b, d) = builder();
    let unknown = element(&mut b, &d, "div", &[]);
    let excluded = element(&mut b, &d, "div", &[]);
    let absent = element(&mut b, &d, "div", &[]);
    let c = b.finish().unwrap();
    let facts = supplement(
        &c,
        key(),
        vec![
            fact(
                &c,
                &unknown,
                None,
                HtmlDocumentFlatTreeRelation::Unavailable,
            ),
            fact(
                &c,
                &excluded,
                Some(false),
                HtmlDocumentFlatTreeRelation::Excluded,
            ),
        ],
        vec![modal(&c, &d, None)],
    );
    assert_eq!(
        facts.nodes()[0].relation(),
        &HtmlDocumentFlatTreeRelation::Unavailable
    );
    assert_eq!(
        facts.nodes()[1].relation(),
        &HtmlDocumentFlatTreeRelation::Excluded
    );
    let cx = context(&c, Some(facts));
    let report = query(&c, &cx, &[&unknown, &excluded, &absent]);
    assert_eq!(
        value(&report, 0),
        &HtmlSemanticValue::Missing(HtmlSemanticMissing::InertAncestry(
            c.subject_id(&unknown).unwrap()
        ))
    );
    assert_eq!(
        value(&report, 1),
        &HtmlSemanticValue::Missing(HtmlSemanticMissing::InertAncestry(
            c.subject_id(&excluded).unwrap()
        ))
    );
    assert_eq!(
        value(&report, 2),
        &HtmlSemanticValue::Missing(HtmlSemanticMissing::InertNodeFacts(
            c.subject_id(&absent).unwrap()
        ))
    );
}

#[test]
fn modal_unknown_and_connectedness_unknown_are_separate_missing_inputs() {
    let (mut b, d) = builder();
    let html = element(&mut b, &d, "html", &[]);
    let blocker = element(&mut b, &d, "dialog", &[("open", "")]);
    let c = b.finish().unwrap();
    let records = || vec![fact(&c, &html, None, HtmlDocumentFlatTreeRelation::Root)];
    let cx = context(
        &c,
        Some(supplement(
            &c,
            key(),
            records(),
            vec![modal(&c, &d, Some(&blocker))],
        )),
    );
    assert_eq!(
        value(&query(&c, &cx, &[&html]), 0),
        &HtmlSemanticValue::Missing(HtmlSemanticMissing::Connectedness(
            c.subject_id(&html).unwrap()
        ))
    );
    let unknown = HtmlDocumentModalFact::new(
        c.subject_id(&d).unwrap(),
        HtmlDocumentModalBlocking::Unavailable,
    );
    let cx = context(&c, Some(supplement(&c, key(), records(), vec![unknown])));
    assert_eq!(
        value(&query(&c, &cx, &[&blocker]), 0),
        &HtmlSemanticValue::Missing(HtmlSemanticMissing::ModalBlocking(
            c.subject_id(&d).unwrap()
        ))
    );
}

#[test]
fn foreign_own_attribute_has_no_html_effect_but_inherits_from_html_flat_parent() {
    let (mut b, d) = builder();
    let html = element(&mut b, &d, "html", &[]);
    let inert = element(&mut b, &d, "div", &[("inert", "")]);
    let svg = expanded(&mut b, &d, Namespace::svg(), "svg", &[("inert", "")]);
    let child = expanded(&mut b, &d, Namespace::svg(), "circle", &[]);
    b.establish_parent(&html, &d).unwrap();
    b.establish_children(&html, &[inert.clone(), svg.clone()])
        .unwrap();
    b.establish_children(&inert, std::slice::from_ref(&child))
        .unwrap();
    let c = b.finish().unwrap();
    let cx = context(
        &c,
        Some(supplement(
            &c,
            key(),
            vec![
                root(&c, &html),
                parent(&c, &inert, &html),
                parent(&c, &svg, &html),
                parent(&c, &child, &inert),
            ],
            vec![modal(&c, &d, None)],
        )),
    );
    let report = query(&c, &cx, &[&svg, &child]);
    assert_eq!(value(&report, 0), &HtmlSemanticValue::Known(false));
    assert_eq!(value(&report, 1), &HtmlSemanticValue::Known(true));
}

#[test]
fn duplicate_cycle_cross_document_and_disconnected_membership_are_rejected() {
    let (mut b, d) = builder();
    let a = element(&mut b, &d, "div", &[]);
    let z = element(&mut b, &d, "span", &[]);
    let detached = element(&mut b, &d, "span", &[]);
    b.establish_root(&detached).unwrap();
    let d2 = b
        .add_document(
            HtmlDocumentMode::NoQuirks,
            HtmlScriptingStatus::Unavailable,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    let foreign_document = element(&mut b, &d2, "div", &[]);
    let c = b.finish().unwrap();
    let make = |nodes| {
        HtmlInertFacts::new(
            &c,
            c.host_facts_revision().clone(),
            key(),
            nodes,
            vec![],
            HtmlInertFactLimits::new(100, 1000000),
        )
    };
    assert!(matches!(
        make(vec![
            fact(&c, &a, None, HtmlDocumentFlatTreeRelation::Unavailable),
            fact(&c, &a, None, HtmlDocumentFlatTreeRelation::Excluded)
        ]),
        Err(HtmlInertFactError::DuplicateSubject(_))
    ));
    assert!(matches!(
        make(vec![parent(&c, &a, &z), parent(&c, &z, &a)]),
        Err(HtmlInertFactError::FlatCycle(_))
    ));
    assert!(matches!(
        make(vec![parent(&c, &a, &foreign_document)]),
        Err(HtmlInertFactError::DocumentMismatch(_))
    ));
    assert!(matches!(
        make(vec![fact(
            &c,
            &detached,
            None,
            HtmlDocumentFlatTreeRelation::Parent(c.subject_id(&a).unwrap())
        )]),
        Err(HtmlInertFactError::ContradictoryMembership(_))
    ));
    assert!(matches!(
        make(vec![fact(
            &c,
            &a,
            Some(true),
            HtmlDocumentFlatTreeRelation::Parent(c.subject_id(&detached).unwrap())
        )]),
        Err(HtmlInertFactError::ContradictoryConnectedness(_))
    ));
}

#[test]
fn blocking_dialog_validation_rejects_wrong_kind_document_and_known_detachment() {
    let (mut b, d) = builder();
    let wrong = element(&mut b, &d, "div", &[]);
    let detached = element(&mut b, &d, "dialog", &[]);
    b.establish_root(&detached).unwrap();
    let d2 = b
        .add_document(
            HtmlDocumentMode::NoQuirks,
            HtmlScriptingStatus::Unavailable,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    let other = element(&mut b, &d2, "dialog", &[]);
    let c = b.finish().unwrap();
    let make = |blocker| {
        HtmlInertFacts::new(
            &c,
            c.host_facts_revision().clone(),
            key(),
            vec![],
            vec![modal(&c, &d, Some(blocker))],
            HtmlInertFactLimits::new(100, 1000000),
        )
    };
    assert!(matches!(
        make(&wrong),
        Err(HtmlInertFactError::InvalidBlockingDialog(_))
    ));
    assert!(matches!(
        make(&detached),
        Err(HtmlInertFactError::DisconnectedBlockingDialog(_))
    ));
    assert!(matches!(
        make(&other),
        Err(HtmlInertFactError::DocumentMismatch(_))
    ));
}

#[test]
fn facts_admission_checks_actual_moved_capacity_validation_work_and_host_identity() {
    let (mut b, d) = builder();
    let e = element(&mut b, &d, "div", &[]);
    let c = b.finish().unwrap();
    let mut nodes = Vec::with_capacity(9);
    nodes.push(fact(
        &c,
        &e,
        None,
        HtmlDocumentFlatTreeRelation::Unavailable,
    ));
    let capacity = nodes.capacity();
    let failure = HtmlInertFacts::new(
        &c,
        c.host_facts_revision().clone(),
        key(),
        nodes,
        vec![],
        HtmlInertFactLimits::new(capacity - 1, 1000000),
    )
    .unwrap_err();
    assert_eq!(
        failure,
        HtmlInertFactError::RecordCapacity {
            limit: capacity - 1,
            required: capacity
        }
    );
    let failure = HtmlInertFacts::new(
        &c,
        c.host_facts_revision().clone(),
        key(),
        vec![fact(
            &c,
            &e,
            None,
            HtmlDocumentFlatTreeRelation::Unavailable,
        )],
        vec![],
        HtmlInertFactLimits::new(1, 0),
    )
    .unwrap_err();
    assert_eq!(
        failure,
        HtmlInertFactError::ValidationWork {
            limit: 0,
            required: 1
        }
    );
    assert_eq!(
        HtmlInertFacts::new(
            &c,
            HtmlHostFactsRevision::fresh(),
            key(),
            vec![],
            vec![],
            HtmlInertFactLimits::new(0, 0)
        )
        .unwrap_err(),
        HtmlInertFactError::HostRevisionMismatch
    );
}

#[test]
fn producer_configuration_revision_and_capture_mismatches_cannot_join_context() {
    let (b, _) = builder();
    let c = b.finish().unwrap();
    let expected = key();
    for wrong in [
        HtmlFlatTreeRevision::new(HtmlFlatTreeProducerId::fresh(), 7, 12),
        HtmlFlatTreeRevision::new(expected.producer().clone(), 8, 12),
        HtmlFlatTreeRevision::new(expected.producer().clone(), 7, 13),
    ] {
        let input = HtmlSemanticContextInput::new(vec![]).with_inert(
            expected.clone(),
            Some(supplement(&c, wrong, vec![], vec![])),
        );
        assert!(HtmlSemanticContext::new(&c, c.host_facts_revision().clone(), input, 0).is_err());
    }
    let (other, _) = builder();
    let other = other.finish().unwrap();
    let input = HtmlSemanticContextInput::new(vec![]).with_inert(
        expected.clone(),
        Some(supplement(&other, expected, vec![], vec![])),
    );
    assert!(HtmlSemanticContext::new(&c, c.host_facts_revision().clone(), input, 0).is_err());
}

#[test]
fn cancelled_and_resource_failed_batches_keep_exact_qualification_and_sticky_stop() {
    let (mut b, d) = builder();
    let e = element(&mut b, &d, "div", &[("inert", "")]);
    let c = b.finish().unwrap();
    let cx = context(&c, None);
    let requests = requests(&c, &[&e, &e]);
    let report = query(&c, &cx, &[&e, &e]);
    let usage = report.usage();
    for (limits, expected) in [
        (
            HtmlSemanticLimits::new(usage.work() - 1, 100, 1000, 1000, 1000000, 1000),
            HtmlSemanticResource::Work,
        ),
        (
            HtmlSemanticLimits::new(1000000, 1, 1000, 1000, 1000000, 1000),
            HtmlSemanticResource::OutputRecords,
        ),
        (
            HtmlSemanticLimits::new(1000000, 100, 1000, 1000, usage.allocated_bytes() - 1, 1000),
            HtmlSemanticResource::AllocatedBytes,
        ),
    ] {
        let mut session = HtmlSemanticSession::new(
            &c,
            &cx,
            HtmlValueInterpreter::new(10000),
            limits,
            HtmlSemanticCancellation::new(),
        )
        .unwrap();
        let failure = session.query(&requests).unwrap_err();
        assert!(
            matches!(failure.kind(), HtmlSemanticFailureKind::LimitExceeded { resource, .. } if resource == &expected)
        );
        assert_eq!(
            failure.qualification().unwrap().inert_revision(),
            cx.inert_revision()
        );
        assert_eq!(failure.qualification().unwrap().capture(), c.identity());
        assert_eq!(session.query(&[]).unwrap_err(), failure);
        assert_eq!(session.usage().scratch_records(), 0);
    }
    let cancel = HtmlSemanticCancellation::new();
    let mut session = HtmlSemanticSession::new(
        &c,
        &cx,
        HtmlValueInterpreter::new(10000),
        limits(),
        cancel.clone(),
    )
    .unwrap();
    cancel.cancel();
    let failure = session.query(&requests).unwrap_err();
    assert_eq!(failure.kind(), &HtmlSemanticFailureKind::Cancelled);
    assert_eq!(
        failure.qualification().unwrap().inert_revision(),
        cx.inert_revision()
    );
    assert_eq!(session.query(&requests).unwrap_err(), failure);
}

#[test]
fn own_attribute_still_obeys_central_exact_utf16_input_admission() {
    let (mut b, d) = builder();
    let attrs = OrderedAttributes::new([Attribute::new(
        QualifiedName::new(Namespace::none(), None, "inert"),
        DocumentString::from_utf16(vec![0xd800, 0, 0xdfff]),
    )])
    .unwrap();
    let e = b
        .add_element(
            &d,
            QualifiedName::new(Namespace::html(), None, "div"),
            attrs,
            HtmlCapturedIsValue::Unavailable,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    let c = b.finish().unwrap();
    let cx = context(&c, None);
    let mut session = HtmlSemanticSession::new(
        &c,
        &cx,
        HtmlValueInterpreter::new(2),
        limits(),
        HtmlSemanticCancellation::new(),
    )
    .unwrap();
    let failure = session.query(&requests(&c, &[&e])).unwrap_err();
    assert!(matches!(
        failure.kind(),
        HtmlSemanticFailureKind::Attribute(_)
    ));
    assert_eq!(failure.qualification().unwrap().attribute_input_limit(), 2);
    assert_eq!(
        failure.qualification().unwrap().inert_revision(),
        cx.inert_revision()
    );
}

#[test]
fn only_exact_null_namespace_inert_attribute_is_intrinsic() {
    let (mut b, d) = builder();
    let html = element(&mut b, &d, "html", &[]);
    let attrs = OrderedAttributes::new([
        Attribute::new(QualifiedName::new(Namespace::none(), None, "INERT"), ""),
        Attribute::new(QualifiedName::new(Namespace::svg(), None, "inert"), ""),
    ])
    .unwrap();
    let e = b
        .add_element(
            &d,
            QualifiedName::new(Namespace::html(), None, "div"),
            attrs,
            HtmlCapturedIsValue::Unavailable,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    b.establish_parent(&html, &d).unwrap();
    b.establish_children(&html, std::slice::from_ref(&e))
        .unwrap();
    let c = b.finish().unwrap();
    let cx = context(
        &c,
        Some(supplement(
            &c,
            key(),
            vec![root(&c, &html), parent(&c, &e, &html)],
            vec![modal(&c, &d, None)],
        )),
    );
    assert_eq!(
        value(&query(&c, &cx, &[&e]), 0),
        &HtmlSemanticValue::Known(false)
    );
}

#[test]
fn known_capture_connection_and_excluded_parent_cannot_be_contradicted() {
    let (mut b, d) = builder();
    let html = element(&mut b, &d, "html", &[]);
    let child = element(&mut b, &d, "span", &[]);
    let other_root = element(&mut b, &d, "body", &[]);
    b.establish_parent(&html, &d).unwrap();
    b.establish_children(&html, std::slice::from_ref(&child))
        .unwrap();
    let c = b.finish().unwrap();
    let make = |nodes| {
        HtmlInertFacts::new(
            &c,
            c.host_facts_revision().clone(),
            key(),
            nodes,
            vec![],
            HtmlInertFactLimits::new(100, 1000000),
        )
    };
    assert!(matches!(
        make(vec![fact(
            &c,
            &child,
            Some(false),
            HtmlDocumentFlatTreeRelation::Excluded
        )]),
        Err(HtmlInertFactError::ContradictoryConnectedness(_))
    ));
    assert!(matches!(
        make(vec![
            fact(
                &c,
                &html,
                Some(true),
                HtmlDocumentFlatTreeRelation::Excluded
            ),
            parent(&c, &child, &html)
        ]),
        Err(HtmlInertFactError::ContradictoryMembership(_))
    ));
    assert!(matches!(
        make(vec![
            root(&c, &html),
            fact(&c, &other_root, None, HtmlDocumentFlatTreeRelation::Root)
        ]),
        Err(HtmlInertFactError::InvalidFlatRoot(_))
    ));
    assert!(matches!(
        make(vec![root(&c, &child)]),
        Err(HtmlInertFactError::InvalidFlatRoot(_))
    ));
}

#[test]
fn document_itself_needs_only_actual_modal_blocking_state() {
    let (mut b, d) = builder();
    let blocker = element(&mut b, &d, "dialog", &[]);
    b.establish_parent(&blocker, &d).unwrap();
    let c = b.finish().unwrap();
    for (blocking, expected) in [(None, false), (Some(&blocker), true)] {
        let cx = context(
            &c,
            Some(supplement(&c, key(), vec![], vec![modal(&c, &d, blocking)])),
        );
        assert_eq!(
            value(&query(&c, &cx, &[&d]), 0),
            &HtmlSemanticValue::Known(expected)
        );
    }
    assert_eq!(
        value(&query(&c, &context(&c, None), &[&d]), 0),
        &HtmlSemanticValue::Missing(HtmlSemanticMissing::ModalBlocking(
            c.subject_id(&d).unwrap()
        ))
    );
}

#[test]
fn template_host_association_does_not_establish_connection_or_inert_ancestry() {
    let (mut b, d) = builder();
    let template = element(&mut b, &d, "template", &[("inert", "")]);
    let fragment = b.add_fragment(&d).unwrap();
    let child = element(&mut b, &d, "button", &[]);
    b.establish_parent(&template, &d).unwrap();
    b.establish_template_contents(&template, &fragment).unwrap();
    b.establish_children(&fragment, std::slice::from_ref(&child))
        .unwrap();
    let c = b.finish().unwrap();
    assert!(matches!(
        HtmlInertFacts::new(
            &c,
            c.host_facts_revision().clone(),
            key(),
            vec![fact(
                &c,
                &child,
                Some(true),
                HtmlDocumentFlatTreeRelation::Excluded
            )],
            vec![],
            HtmlInertFactLimits::new(100, 1000000)
        ),
        Err(HtmlInertFactError::ContradictoryConnectedness(_))
    ));
    let cx = context(
        &c,
        Some(supplement(
            &c,
            key(),
            vec![fact(
                &c,
                &child,
                Some(false),
                HtmlDocumentFlatTreeRelation::Excluded,
            )],
            vec![],
        )),
    );
    assert_eq!(
        value(&query(&c, &cx, &[&child]), 0),
        &HtmlSemanticValue::Missing(HtmlSemanticMissing::InertAncestry(
            c.subject_id(&child).unwrap()
        ))
    );
}
