use surgeist_html_semantics::HtmlCaptureDocumentId;
use surgeist_html_semantics::{
    HtmlCapture, HtmlCaptureBuilder, HtmlCaptureBuilderSubject, HtmlCaptureRegistryInput,
    HtmlCaptureSubjectId, HtmlCapturedIsValue, HtmlCapturedShadowProperties, HtmlMicrodataAuthor,
    HtmlMicrodataCondition, HtmlMicrodataDocumentAuthor, HtmlMicrodataFacts,
    HtmlMicrodataFactsError, HtmlMicrodataItem, HtmlMicrodataProperty, HtmlMicrodataPropertyFact,
    HtmlMicrodataPropertyPermission, HtmlMicrodataProviderId, HtmlMicrodataProviderRevision,
    HtmlMicrodataRawBinding, HtmlMicrodataRawSource, HtmlMicrodataRevision, HtmlMicrodataTypeFact,
    HtmlMicrodataTyping, HtmlMicrodataUrlFact, HtmlMicrodataUrlOperation, HtmlMicrodataUrlResult,
    HtmlMicrodataValue, HtmlMicrodataVocabularyId, HtmlSemanticCancellation, HtmlSemanticContext,
    HtmlSemanticContextError, HtmlSemanticContextInput, HtmlSemanticFact, HtmlSemanticFailureKind,
    HtmlSemanticLimits, HtmlSemanticMissing, HtmlSemanticQuery, HtmlSemanticReport,
    HtmlSemanticRequest, HtmlSemanticResource, HtmlSemanticSession, HtmlSemanticUnsupported,
    HtmlSemanticValue,
};
#[path = "common/builder.rs"]
mod shared_builder;
use shared_builder::builder;
#[path = "common/context.rs"]
mod shared_context;
use shared_context::context;
use surgeist_html::*;

// Literal expectations are selected from HTML877d §5.2, not from a second
// crawler or from implementation-derived resource totals.

fn attrs(a: &[(&str, &str)]) -> OrderedAttributes {
    OrderedAttributes::new(
        a.iter()
            .map(|(n, v)| Attribute::new(QualifiedName::new(Namespace::none(), None, *n), *v)),
    )
    .unwrap()
}
fn el(
    b: &mut HtmlCaptureBuilder,
    d: &HtmlCaptureBuilderSubject,
    name: &str,
    a: &[(&str, &str)],
) -> HtmlCaptureBuilderSubject {
    b.add_element(
        d,
        QualifiedName::new(Namespace::html(), None, name),
        attrs(a),
        HtmlCapturedIsValue::Unavailable,
        HtmlCaptureRegistryInput::Null,
    )
    .unwrap()
}
fn cs(b: &mut HtmlCaptureBuilder, p: &HtmlCaptureBuilderSubject, c: &[HtmlCaptureBuilderSubject]) {
    b.establish_children(p, c).unwrap();
}

fn limits() -> HtmlSemanticLimits {
    HtmlSemanticLimits::new(20_000_000, 100, 10000, 0, 10_000_000, 100000)
}
fn run(
    c: &HtmlCapture,
    cx: &HtmlSemanticContext,
    id: &HtmlCaptureSubjectId,
    q: HtmlSemanticQuery,
) -> HtmlSemanticReport {
    HtmlSemanticSession::new(
        c,
        cx,
        HtmlValueInterpreter::new(1000),
        limits(),
        HtmlSemanticCancellation::new(),
    )
    .unwrap()
    .query(&[HtmlSemanticRequest::new(id.clone(), q)])
    .unwrap()
}
fn item(r: &HtmlSemanticReport) -> &HtmlMicrodataItem {
    match r.results()[0].fact() {
        HtmlSemanticFact::MicrodataItem(HtmlSemanticValue::Known(v)) => v,
        x => panic!("{x:?}"),
    }
}
fn prop(r: &HtmlSemanticReport) -> &HtmlMicrodataProperty {
    match r.results()[0].fact() {
        HtmlSemanticFact::MicrodataProperty(HtmlSemanticValue::Known(v)) => v,
        x => panic!("{x:?}"),
    }
}
fn author(r: &HtmlSemanticReport) -> &HtmlMicrodataAuthor {
    match r.results()[0].fact() {
        HtmlSemanticFact::MicrodataAuthor(HtmlSemanticValue::Known(v)) => v,
        x => panic!("{x:?}"),
    }
}
fn document(r: &HtmlSemanticReport) -> &HtmlMicrodataDocumentAuthor {
    match r.results()[0].fact() {
        HtmlSemanticFact::MicrodataDocumentAuthor(HtmlSemanticValue::Known(v)) => v,
        x => panic!("{x:?}"),
    }
}
fn known<T>(v: &HtmlSemanticValue<T>) -> &T {
    match v {
        HtmlSemanticValue::Known(v) => v,
        _ => panic!("not Known"),
    }
}
fn single(name: &str, a: &[(&str, &str)]) -> (HtmlCapture, HtmlCaptureSubjectId) {
    let (mut b, d) = builder();
    let n = el(&mut b, &d, name, a);
    cs(&mut b, &n, &[]);
    b.establish_root(&n).unwrap();
    let c = b.finish().unwrap();
    let id = c.subject_id(&n).unwrap();
    (c, id)
}

#[test]
fn property_precedence_and_raw_attribute_values_do_not_parse_meter_or_time() {
    for (name, a, expected) in [
        ("meta", vec![("content", " A\rB ")], " A\rB "),
        ("data", vec![("value", "12junk")], "12junk"),
        ("meter", vec![("value", "1e999")], "1e999"),
        (
            "time",
            vec![("datetime", "invalid temporal")],
            "invalid temporal",
        ),
        ("meta", vec![], ""),
    ] {
        let mut a = a;
        a.push(("itemprop", "p"));
        let (c, id) = single(name, &a);
        let r = run(&c, &context(&c), &id, HtmlSemanticQuery::MicrodataProperty);
        assert_eq!(
            prop(&r).value(),
            &HtmlSemanticValue::Known(HtmlMicrodataValue::String(DocumentString::from(expected)))
        );
    }
    let (c, id) = single(
        "img",
        &[("itemscope", ""), ("itemprop", "p"), ("src", "unknown")],
    );
    let r = run(&c, &context(&c), &id, HtmlSemanticQuery::MicrodataProperty);
    assert_eq!(
        prop(&r).value(),
        &HtmlSemanticValue::Known(HtmlMicrodataValue::Item(id))
    );
}
#[test]
fn time_child_text_and_generic_descendant_text_keep_exact_utf16() {
    for name in ["time", "span"] {
        let (mut b, d) = builder();
        let n = el(&mut b, &d, name, &[("itemprop", "p")]);
        let inner = el(&mut b, &d, "b", &[]);
        let t = b
            .add_text(&d, DocumentString::from_utf16(vec![0xd800, 0x20, 0x41]))
            .unwrap();
        let inside = b.add_text(&d, "B".into()).unwrap();
        cs(&mut b, &inner, &[inside]);
        cs(&mut b, &n, &[t, inner]);
        b.establish_root(&n).unwrap();
        let c = b.finish().unwrap();
        let r = run(
            &c,
            &context(&c),
            &c.subject_id(&n).unwrap(),
            HtmlSemanticQuery::MicrodataProperty,
        );
        let expected = if name == "time" {
            vec![0xd800, 0x20, 0x41]
        } else {
            vec![0xd800, 0x20, 0x41, 0x42]
        };
        assert_eq!(
            prop(&r).value(),
            &HtmlSemanticValue::Known(HtmlMicrodataValue::String(DocumentString::from_utf16(
                expected
            )))
        );
    }
}
#[test]
fn properties_sort_in_tree_order_and_nested_items_prune() {
    let (mut b, d) = builder();
    let root = el(
        &mut b,
        &d,
        "div",
        &[("itemscope", ""), ("itemref", "outside")],
    );
    let first = el(
        &mut b,
        &d,
        "meta",
        &[("itemprop", "x x y"), ("content", "first")],
    );
    let nested = el(
        &mut b,
        &d,
        "section",
        &[("itemscope", ""), ("itemprop", "nested")],
    );
    let ignored = el(&mut b, &d, "meta", &[("itemprop", "hidden")]);
    let outside = el(
        &mut b,
        &d,
        "meta",
        &[("id", "outside"), ("itemprop", "x"), ("content", "last")],
    );
    cs(&mut b, &first, &[]);
    cs(&mut b, &ignored, &[]);
    cs(&mut b, &nested, &[ignored]);
    cs(&mut b, &root, &[first.clone(), nested.clone()]);
    cs(&mut b, &outside, &[]);
    cs(&mut b, &d, &[root.clone(), outside.clone()]);
    let c = b.finish().unwrap();
    let r = run(
        &c,
        &context(&c),
        &c.subject_id(&root).unwrap(),
        HtmlSemanticQuery::MicrodataItem,
    );
    let p = known(item(&r).properties());
    assert_eq!(
        p.iter().map(|p| p.subject().clone()).collect::<Vec<_>>(),
        vec![
            c.subject_id(&first).unwrap(),
            c.subject_id(&nested).unwrap(),
            c.subject_id(&outside).unwrap()
        ]
    );
    assert_eq!(p[0].names().processing_tokens().count(), 2);
    assert_eq!(
        p[1].value(),
        &HtmlSemanticValue::Known(HtmlMicrodataValue::Item(c.subject_id(&nested).unwrap()))
    );
    assert!(matches!(
        item(&r).type_context().typing(),
        HtmlMicrodataTyping::Untyped
    ));
}
#[test]
fn duplicate_itemref_is_a_revisit_and_first_foreign_id_blocks_later_html() {
    let (mut b, d) = builder();
    let root = el(
        &mut b,
        &d,
        "div",
        &[("itemscope", ""), ("id", "self"), ("itemref", "self x x")],
    );
    let foreign = b
        .add_element(
            &d,
            QualifiedName::new(Namespace::svg(), None, "g"),
            attrs(&[("id", "x"), ("itemscope", ""), ("itemprop", "wrong")]),
            HtmlCapturedIsValue::Unavailable,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    let nested = el(
        &mut b,
        &d,
        "meta",
        &[("itemprop", "p"), ("content", "through foreign")],
    );
    let later = el(&mut b, &d, "meta", &[("id", "x"), ("itemprop", "later")]);
    cs(&mut b, &nested, &[]);
    cs(&mut b, &foreign, std::slice::from_ref(&nested));
    cs(&mut b, &later, &[]);
    cs(&mut b, &root, &[]);
    cs(&mut b, &d, &[root.clone(), foreign, later]);
    let c = b.finish().unwrap();
    let r = run(
        &c,
        &context(&c),
        &c.subject_id(&root).unwrap(),
        HtmlSemanticQuery::MicrodataItem,
    );
    assert_eq!(
        known(item(&r).properties())
            .iter()
            .map(|p| p.subject().clone())
            .collect::<Vec<_>>(),
        vec![c.subject_id(&nested).unwrap()]
    );
    assert!(matches!(
        item(&r).revisits(),
        HtmlMicrodataCondition::Violated(_)
    ));
}
#[test]
fn empty_itemprop_suppresses_top_level_and_is_an_orphan() {
    let (mut b, d) = builder();
    let n = el(&mut b, &d, "div", &[("itemscope", ""), ("itemprop", "")]);
    cs(&mut b, &n, &[]);
    cs(&mut b, &d, std::slice::from_ref(&n));
    let c = b.finish().unwrap();
    let id = c.subject_id(&n).unwrap();
    let r = run(&c, &context(&c), &id, HtmlSemanticQuery::MicrodataItem);
    assert!(!item(&r).top_level());
    let r = run(
        &c,
        &context(&c),
        &c.subject_id(&d).unwrap(),
        HtmlSemanticQuery::MicrodataDocumentAuthor,
    );
    assert_eq!(
        document(&r).orphans(),
        &HtmlMicrodataCondition::Violated(id)
    );
}
#[test]
fn receiving_types_are_item_value_paths_not_dom_ancestors() {
    let (mut b, d) = builder();
    let a = el(
        &mut b,
        &d,
        "div",
        &[
            ("itemscope", ""),
            ("itemtype", "TypeA"),
            ("itemref", "child"),
        ],
    );
    let z = el(
        &mut b,
        &d,
        "div",
        &[
            ("itemscope", ""),
            ("itemtype", "TypeZ"),
            ("itemref", "child"),
        ],
    );
    let child = el(
        &mut b,
        &d,
        "section",
        &[("itemscope", ""), ("id", "child"), ("itemprop", "child")],
    );
    cs(&mut b, &a, &[]);
    cs(&mut b, &z, &[]);
    cs(&mut b, &child, &[]);
    cs(&mut b, &d, &[a.clone(), z.clone(), child.clone()]);
    let c = b.finish().unwrap();
    let id = c.subject_id(&child).unwrap();
    let r = run(&c, &context(&c), &id, HtmlSemanticQuery::MicrodataItem);
    assert_eq!(
        item(&r).type_context().typing(),
        &HtmlMicrodataTyping::Typed
    );
    let paths = known(item(&r).type_context().paths());
    assert_eq!(paths.len(), 2);
    assert_eq!(paths[0].items(), &[id.clone(), c.subject_id(&a).unwrap()]);
    assert_eq!(paths[1].items(), &[id, c.subject_id(&z).unwrap()]);
    assert_eq!(
        paths[0].types().token_units(0),
        Some(DocumentString::from("TypeA").code_units())
    );
}
#[test]
fn item_graph_cycle_is_distinct_from_crawl_revisit() {
    let (mut b, d) = builder();
    let a = el(
        &mut b,
        &d,
        "div",
        &[
            ("itemscope", ""),
            ("id", "a"),
            ("itemprop", "p"),
            ("itemref", "z"),
        ],
    );
    let z = el(
        &mut b,
        &d,
        "div",
        &[
            ("itemscope", ""),
            ("id", "z"),
            ("itemprop", "p"),
            ("itemref", "a"),
        ],
    );
    cs(&mut b, &a, &[]);
    cs(&mut b, &z, &[]);
    cs(&mut b, &d, &[a.clone(), z.clone()]);
    let c = b.finish().unwrap();
    let r = run(
        &c,
        &context(&c),
        &c.subject_id(&d).unwrap(),
        HtmlSemanticQuery::MicrodataDocumentAuthor,
    );
    assert_eq!(document(&r).revisits(), &HtmlMicrodataCondition::Satisfied);
    assert!(matches!(
        document(&r).cycles(),
        HtmlMicrodataCondition::Violated(_)
    ));
    assert_eq!(document(&r).orphans(), &HtmlMicrodataCondition::Satisfied);
    let r = run(
        &c,
        &context(&c),
        &c.subject_id(&a).unwrap(),
        HtmlSemanticQuery::MicrodataItem,
    );
    assert_eq!(
        item(&r).type_context().typing(),
        &HtmlMicrodataTyping::Cyclic
    );
}
#[test]
fn complete_empty_and_missing_children_are_distinct() {
    let (c, id) = single("div", &[("itemscope", "")]);
    let r = run(&c, &context(&c), &id, HtmlSemanticQuery::MicrodataItem);
    assert!(known(item(&r).properties()).is_empty());
    let (mut b, d) = builder();
    let n = el(&mut b, &d, "div", &[("itemscope", "")]);
    b.establish_root(&n).unwrap();
    let c = b.finish().unwrap();
    let r = run(
        &c,
        &context(&c),
        &c.subject_id(&n).unwrap(),
        HtmlSemanticQuery::MicrodataItem,
    );
    assert!(matches!(
        item(&r).properties(),
        HtmlSemanticValue::Missing(HtmlSemanticMissing::OrderedChildren(_))
    ));
}
#[test]
fn local_eligibility_and_unknown_external_checks_are_independent() {
    let (c, id) = single(
        "span",
        &[
            ("itemtype", "not-url not-url"),
            ("itemid", "bad"),
            ("itemref", "absent"),
        ],
    );
    let r = run(&c, &context(&c), &id, HtmlSemanticQuery::MicrodataAuthor);
    let a = author(&r);
    assert_eq!(a.type_eligibility(), &HtmlMicrodataCondition::Violated(()));
    assert_eq!(a.id_eligibility(), &HtmlMicrodataCondition::Violated(()));
    assert_eq!(a.ref_eligibility(), &HtmlMicrodataCondition::Violated(()));
    assert!(!a.types().local_author_conforming());
    assert!(matches!(a.type_urls(), HtmlMicrodataCondition::Missing(_)));
    assert_eq!(a.references(), &HtmlMicrodataCondition::Violated(0));
}

fn revision() -> HtmlMicrodataProviderRevision {
    HtmlMicrodataProviderRevision::new(HtmlMicrodataProviderId::fresh(), 1, 7)
}
fn facts_context(
    c: &HtmlCapture,
    revision: HtmlMicrodataRevision,
    urls: Vec<HtmlMicrodataUrlFact>,
    types: Vec<HtmlMicrodataTypeFact>,
    properties: Vec<HtmlMicrodataPropertyFact>,
) -> HtmlSemanticContext {
    let facts =
        HtmlMicrodataFacts::new(c, revision.clone(), urls, types, properties, 1000, 1000).unwrap();
    HtmlSemanticContext::new(
        c,
        c.host_facts_revision().clone(),
        HtmlSemanticContextInput::new(vec![]).with_microdata(revision, Some(facts)),
        0,
    )
    .unwrap()
}
#[test]
fn property_url_uses_qualified_historical_operation_not_current_parse() {
    let (c, id) = single("img", &[("itemprop", "p"), ("src", "relative")]);
    let r = run(&c, &context(&c), &id, HtmlSemanticQuery::MicrodataProperty);
    assert!(matches!(prop(&r).value(), HtmlSemanticValue::Missing(_)));
    let rev = revision();
    let binding =
        HtmlMicrodataRawBinding::new(&c, id.clone(), HtmlMicrodataRawSource::Src, 8).unwrap();
    let document = c.node(&id).unwrap().node_document();
    let op = HtmlMicrodataUrlOperation::PropertyAttributeSet {
        revision: rev.clone(),
        event: 3,
        document,
        base_revision: 11,
        encoding_revision: 2,
    };
    let fact = HtmlMicrodataUrlFact::new(
        binding,
        op.clone(),
        Some(op),
        Some(HtmlMicrodataUrlResult::Parsed(Some(
            "https://old-base/relative".into(),
        ))),
    )
    .unwrap();
    let cx = facts_context(
        &c,
        HtmlMicrodataRevision::new(Some(rev), None),
        vec![fact],
        vec![],
        vec![],
    );
    let r = run(&c, &cx, &id, HtmlSemanticQuery::MicrodataProperty);
    assert_eq!(
        prop(&r).value(),
        &HtmlSemanticValue::Known(HtmlMicrodataValue::String(
            "https://old-base/relative".into()
        ))
    );
}
#[test]
fn actual_failed_url_parse_is_empty_and_missing_attribute_needs_no_fact() {
    let (c, id) = single("a", &[("itemprop", "p"), ("href", "bad")]);
    let rev = revision();
    let binding =
        HtmlMicrodataRawBinding::new(&c, id.clone(), HtmlMicrodataRawSource::Href, 3).unwrap();
    let op = HtmlMicrodataUrlOperation::PropertyAttributeSet {
        revision: rev.clone(),
        event: 0,
        document: c.node(&id).unwrap().node_document(),
        base_revision: 0,
        encoding_revision: 0,
    };
    let fact = HtmlMicrodataUrlFact::new(
        binding,
        op.clone(),
        Some(op),
        Some(HtmlMicrodataUrlResult::Parsed(None)),
    )
    .unwrap();
    let cx = facts_context(
        &c,
        HtmlMicrodataRevision::new(Some(rev), None),
        vec![fact],
        vec![],
        vec![],
    );
    assert_eq!(
        prop(&run(&c, &cx, &id, HtmlSemanticQuery::MicrodataProperty)).value(),
        &HtmlSemanticValue::Known(HtmlMicrodataValue::String("".into()))
    );
    let (c, id) = single("a", &[("itemprop", "p")]);
    assert_eq!(
        prop(&run(
            &c,
            &context(&c),
            &id,
            HtmlSemanticQuery::MicrodataProperty
        ))
        .value(),
        &HtmlSemanticValue::Known(HtmlMicrodataValue::String("".into()))
    );
}
#[test]
fn raw_caps_terminal_and_atomic_batch_do_not_publish_earlier_results() {
    let (c, id) = single("meta", &[("itemprop", "p"), ("content", "oversized")]);
    let cx = context(&c);
    let mut s = HtmlSemanticSession::new(
        &c,
        &cx,
        HtmlValueInterpreter::new(1),
        limits(),
        HtmlSemanticCancellation::new(),
    )
    .unwrap();
    let err = s
        .query(&[HtmlSemanticRequest::new(
            id.clone(),
            HtmlSemanticQuery::MicrodataProperty,
        )])
        .unwrap_err();
    assert!(matches!(err.kind(), HtmlSemanticFailureKind::Attribute(_)));
    assert_eq!(s.query(&[]).unwrap_err(), err);
    let cancel = HtmlSemanticCancellation::new();
    cancel.cancel();
    let mut s = HtmlSemanticSession::new(&c, &cx, HtmlValueInterpreter::new(100), limits(), cancel)
        .unwrap();
    assert!(matches!(
        s.query(&[HtmlSemanticRequest::new(
            id,
            HtmlSemanticQuery::MicrodataProperty
        )])
        .unwrap_err()
        .kind(),
        HtmlSemanticFailureKind::Cancelled
    ));
}

#[test]
fn known_type_url_definition_vocabulary_and_identifier_failures_are_separate() {
    let (c, id) = single(
        "div",
        &[("itemscope", ""), ("itemtype", "A B"), ("itemid", "id")],
    );
    let url = revision();
    let vocabulary = revision();
    let mut urls = Vec::new();
    let mut types = Vec::new();
    for (i, syntax) in [true, false].into_iter().enumerate() {
        let binding = HtmlMicrodataRawBinding::new(
            &c,
            id.clone(),
            HtmlMicrodataRawSource::ItemTypeToken(i),
            3,
        )
        .unwrap();
        let op = HtmlMicrodataUrlOperation::AbsoluteAuthorToken(url.clone());
        urls.push(
            HtmlMicrodataUrlFact::new(
                binding.clone(),
                op.clone(),
                Some(op),
                Some(HtmlMicrodataUrlResult::AuthorSyntax(syntax)),
            )
            .unwrap(),
        );
        types.push(
            HtmlMicrodataTypeFact::new(
                binding,
                vocabulary.clone(),
                Some(i == 0),
                Some(HtmlMicrodataVocabularyId::fresh()),
                Some(i == 0),
            )
            .unwrap(),
        );
    }
    let binding =
        HtmlMicrodataRawBinding::new(&c, id.clone(), HtmlMicrodataRawSource::ItemId, 3).unwrap();
    let op = HtmlMicrodataUrlOperation::ItemIdAuthor(url.clone());
    urls.push(
        HtmlMicrodataUrlFact::new(
            binding.clone(),
            op.clone(),
            Some(op),
            Some(HtmlMicrodataUrlResult::AuthorSyntax(false)),
        )
        .unwrap(),
    );
    let op = HtmlMicrodataUrlOperation::CurrentItemIdentifier {
        revision: url.clone(),
        document: c.node(&id).unwrap().node_document(),
    };
    urls.push(
        HtmlMicrodataUrlFact::new(
            binding,
            op.clone(),
            Some(op),
            Some(HtmlMicrodataUrlResult::Parsed(None)),
        )
        .unwrap(),
    );
    let cx = facts_context(
        &c,
        HtmlMicrodataRevision::new(Some(url), Some(vocabulary)),
        urls,
        types,
        vec![],
    );
    let r = run(&c, &cx, &id, HtmlSemanticQuery::MicrodataAuthor);
    let a = author(&r);
    assert_eq!(a.type_urls(), &HtmlMicrodataCondition::Violated(1));
    assert_eq!(a.type_definitions(), &HtmlMicrodataCondition::Violated(1));
    assert_eq!(a.same_vocabulary(), &HtmlMicrodataCondition::Violated(1));
    assert_eq!(
        a.identifier_permission(),
        &HtmlMicrodataCondition::Violated(1)
    );
    assert_eq!(a.identifier_syntax(), &HtmlMicrodataCondition::Violated(()));
    assert_eq!(
        item(&run(&c, &cx, &id, HtmlSemanticQuery::MicrodataItem)).identifier(),
        &HtmlSemanticValue::Known(None)
    );
}
#[test]
fn four_property_name_alternatives_and_url_value_requirement_are_independent() {
    for (typed, name, defined_name, defined_url, proprietary) in [
        (true, "title", true, false, false),
        (false, "https://vocab/p", false, true, false),
        (true, "https://private/p", false, false, true),
        (false, "private", false, false, true),
    ] {
        let (mut b, d) = builder();
        let a = if typed {
            vec![("itemscope", ""), ("itemtype", "Type")]
        } else {
            vec![("itemscope", "")]
        };
        let root = el(&mut b, &d, "div", &a);
        let p = el(
            &mut b,
            &d,
            "meta",
            &[("itemprop", name), ("content", "looks:https://url")],
        );
        cs(&mut b, &p, &[]);
        cs(&mut b, &root, std::slice::from_ref(&p));
        cs(&mut b, &d, std::slice::from_ref(&root));
        let c = b.finish().unwrap();
        let id = c.subject_id(&p).unwrap();
        let root = c.subject_id(&root).unwrap();
        let url = revision();
        let vocab = revision();
        let binding = HtmlMicrodataRawBinding::new(
            &c,
            id.clone(),
            HtmlMicrodataRawSource::ItemPropToken(0),
            100,
        )
        .unwrap();
        let op = HtmlMicrodataUrlOperation::AbsoluteAuthorToken(url.clone());
        let url_fact = HtmlMicrodataUrlFact::new(
            binding.clone(),
            op.clone(),
            Some(op),
            Some(HtmlMicrodataUrlResult::AuthorSyntax(
                name.starts_with("https:"),
            )),
        )
        .unwrap();
        let permission = HtmlMicrodataPropertyPermission::new(
            Some(defined_name),
            Some(defined_url),
            Some(false),
            Some(proprietary),
        );
        let property = HtmlMicrodataPropertyFact::new(
            &c,
            binding,
            vocab.clone(),
            root.clone(),
            typed.then(|| root.clone()),
            permission,
            100,
        )
        .unwrap();
        let cx = facts_context(
            &c,
            HtmlMicrodataRevision::new(Some(url), Some(vocab)),
            vec![url_fact],
            vec![],
            vec![property],
        );
        let r = run(&c, &cx, &id, HtmlSemanticQuery::MicrodataAuthor);
        let rows = known(author(&r).receivers());
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].name(), &HtmlMicrodataCondition::Satisfied);
        assert_eq!(rows[0].value_kind(), &HtmlMicrodataCondition::Satisfied);
        assert_eq!(rows[0].origin(), typed.then_some(&root));
    }
}
#[test]
fn defined_absolute_url_value_uses_role_not_string_spelling() {
    let (mut b, d) = builder();
    let root = el(
        &mut b,
        &d,
        "div",
        &[("itemscope", ""), ("itemtype", "Type")],
    );
    let p = el(
        &mut b,
        &d,
        "meta",
        &[("itemprop", "url"), ("content", "https://looks-valid")],
    );
    cs(&mut b, &p, &[]);
    cs(&mut b, &root, std::slice::from_ref(&p));
    b.establish_root(&root).unwrap();
    let c = b.finish().unwrap();
    let id = c.subject_id(&p).unwrap();
    let rid = c.subject_id(&root).unwrap();
    let vocab = revision();
    let binding =
        HtmlMicrodataRawBinding::new(&c, id.clone(), HtmlMicrodataRawSource::ItemPropToken(0), 3)
            .unwrap();
    let fact = HtmlMicrodataPropertyFact::new(
        &c,
        binding,
        vocab.clone(),
        rid.clone(),
        Some(rid),
        HtmlMicrodataPropertyPermission::new(Some(true), Some(false), Some(true), Some(false)),
        10,
    )
    .unwrap();
    let cx = facts_context(
        &c,
        HtmlMicrodataRevision::new(None, Some(vocab)),
        vec![],
        vec![],
        vec![fact],
    );
    let r = run(&c, &cx, &id, HtmlSemanticQuery::MicrodataAuthor);
    let row = &known(author(&r).receivers())[0];
    assert_eq!(row.name(), &HtmlMicrodataCondition::Satisfied);
    assert_eq!(row.value_kind(), &HtmlMicrodataCondition::Violated(0));
}
#[test]
fn qualified_facts_reject_foreign_capture_revision_duplicate_and_retained_capacity() {
    let (c, id) = single("img", &[("itemprop", "p"), ("src", "abc")]);
    let rev = revision();
    let binding = HtmlMicrodataRawBinding::new(&c, id, HtmlMicrodataRawSource::Src, 3).unwrap();
    let op = HtmlMicrodataUrlOperation::PropertyAttributeSet {
        revision: rev.clone(),
        event: 1,
        document: binding_document(&c, &binding),
        base_revision: 1,
        encoding_revision: 1,
    };
    let fact = HtmlMicrodataUrlFact::new(
        binding,
        op.clone(),
        Some(op),
        Some(HtmlMicrodataUrlResult::Parsed(None)),
    )
    .unwrap();
    let expected = HtmlMicrodataRevision::new(Some(rev), None);
    assert!(matches!(
        HtmlMicrodataFacts::new(
            &c,
            expected.clone(),
            vec![fact.clone(), fact.clone()],
            vec![],
            vec![],
            10,
            3
        ),
        Err(HtmlMicrodataFactsError::Duplicate)
    ));
    let mut stored = Vec::with_capacity(10);
    stored.push(fact.clone());
    assert!(matches!(
        HtmlMicrodataFacts::new(&c, expected.clone(), stored, vec![], vec![], 1, 3),
        Err(HtmlMicrodataFactsError::Limit)
    ));
    let (other, _) = single("img", &[]);
    assert!(matches!(
        HtmlMicrodataFacts::new(
            &other,
            expected.clone(),
            vec![fact.clone()],
            vec![],
            vec![],
            1,
            3
        ),
        Err(HtmlMicrodataFactsError::OperationMismatch)
    ));
    let facts = HtmlMicrodataFacts::new(&c, expected, vec![fact], vec![], vec![], 1, 3).unwrap();
    assert!(matches!(
        HtmlSemanticContext::new(
            &c,
            c.host_facts_revision().clone(),
            HtmlSemanticContextInput::new(vec![]).with_microdata(
                HtmlMicrodataRevision::new(Some(revision()), None),
                Some(facts)
            ),
            0
        ),
        Err(HtmlSemanticContextError::MicrodataFactsMismatch)
    ));
}
fn binding_document(c: &HtmlCapture, b: &HtmlMicrodataRawBinding) -> HtmlCaptureDocumentId {
    c.node(b.subject()).unwrap().node_document()
}
#[test]
fn foreign_subject_and_attribute_namespace_case_do_not_enter_html_microdata() {
    let (mut b, d) = builder();
    let n = b
        .add_element(
            &d,
            QualifiedName::new(Namespace::svg(), None, "g"),
            attrs(&[("itemscope", "")]),
            HtmlCapturedIsValue::Unavailable,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    cs(&mut b, &n, &[]);
    b.establish_root(&n).unwrap();
    let c = b.finish().unwrap();
    let r = run(
        &c,
        &context(&c),
        &c.subject_id(&n).unwrap(),
        HtmlSemanticQuery::MicrodataItem,
    );
    assert!(matches!(
        r.results()[0].fact(),
        HtmlSemanticFact::MicrodataItem(HtmlSemanticValue::Unsupported(
            HtmlSemanticUnsupported::ForeignNamespace
        ))
    ));
    let (c, id) = single("div", &[("ITEMSCOPE", ""), ("itemprop", "")]);
    assert!(matches!(
        run(&c, &context(&c), &id, HtmlSemanticQuery::MicrodataItem).results()[0].fact(),
        HtmlSemanticFact::MicrodataItem(HtmlSemanticValue::Inapplicable)
    ));
}
#[test]
fn influencing_first_id_unknown_before_match_prevents_later_choice() {
    for unknown_first in [true, false] {
        let (mut b, d) = builder();
        let root = el(&mut b, &d, "div", &[("itemscope", ""), ("itemref", "p")]);
        let unknown = el(&mut b, &d, "section", &[]);
        let p = el(&mut b, &d, "meta", &[("id", "p"), ("itemprop", "x")]);
        cs(&mut b, &root, &[]);
        cs(&mut b, &p, &[]);
        let order = if unknown_first {
            vec![root.clone(), unknown, p.clone()]
        } else {
            vec![root.clone(), p.clone(), unknown]
        };
        cs(&mut b, &d, &order);
        let c = b.finish().unwrap();
        let r = run(
            &c,
            &context(&c),
            &c.subject_id(&root).unwrap(),
            HtmlSemanticQuery::MicrodataItem,
        );
        if unknown_first {
            assert!(matches!(
                item(&r).properties(),
                HtmlSemanticValue::Missing(_)
            ));
        } else {
            assert_eq!(known(item(&r).properties()).len(), 1);
        }
    }
}
#[test]
fn empty_itemprop_known_noncontribution_ignores_unknown_ancestry() {
    let (mut b, d) = builder();
    let n = el(&mut b, &d, "span", &[("itemprop", " \t")]);
    let c = b.finish().unwrap();
    let r = run(
        &c,
        &context(&c),
        &c.subject_id(&n).unwrap(),
        HtmlSemanticQuery::MicrodataAuthor,
    );
    assert_eq!(
        author(&r).contribution(),
        &HtmlMicrodataCondition::Violated(())
    );
    assert!(known(author(&r).receivers()).is_empty());
}
#[test]
fn raw_first_caps_exact_and_one_short_even_for_duplicate_or_scope_precedence() {
    let (c, id) = single(
        "meta",
        &[
            ("itemprop", "p p"),
            ("itemscope", "xx"),
            ("content", "unread unrelated"),
        ],
    );
    let cx = context(&c);
    let mut s = HtmlSemanticSession::new(
        &c,
        &cx,
        HtmlValueInterpreter::new(3),
        limits(),
        HtmlSemanticCancellation::new(),
    )
    .unwrap();
    let r = s
        .query(&[HtmlSemanticRequest::new(
            id.clone(),
            HtmlSemanticQuery::MicrodataProperty,
        )])
        .unwrap();
    assert_eq!(
        prop(&r).value(),
        &HtmlSemanticValue::Known(HtmlMicrodataValue::Item(id.clone()))
    );
    let mut s = HtmlSemanticSession::new(
        &c,
        &cx,
        HtmlValueInterpreter::new(2),
        limits(),
        HtmlSemanticCancellation::new(),
    )
    .unwrap();
    assert!(matches!(
        s.query(&[HtmlSemanticRequest::new(
            id,
            HtmlSemanticQuery::MicrodataProperty
        )])
        .unwrap_err()
        .kind(),
        HtmlSemanticFailureKind::Attribute(_)
    ));
}
#[test]
fn independent_output_scratch_and_nested_limits_stop_atomically_and_reclaim_scratch() {
    let (c, id) = single("div", &[("itemscope", ""), ("itemtype", "T")]);
    let cx = context(&c);
    for (l, resource) in [
        (
            HtmlSemanticLimits::new(1000000, 0, 100, 0, 100000, 100),
            HtmlSemanticResource::OutputRecords,
        ),
        (
            HtmlSemanticLimits::new(1000000, 10, 0, 0, 100000, 100),
            HtmlSemanticResource::ScratchRecords,
        ),
        (
            HtmlSemanticLimits::new(1000000, 10, 100, 0, 100000, 0),
            HtmlSemanticResource::NestedOutputRecords,
        ),
        (
            HtmlSemanticLimits::new(1000000, 10, 100, 0, 0, 100),
            HtmlSemanticResource::AllocatedBytes,
        ),
    ] {
        let mut s = HtmlSemanticSession::new(
            &c,
            &cx,
            HtmlValueInterpreter::new(1),
            l,
            HtmlSemanticCancellation::new(),
        )
        .unwrap();
        let err = s
            .query(&[HtmlSemanticRequest::new(
                id.clone(),
                HtmlSemanticQuery::MicrodataItem,
            )])
            .unwrap_err();
        assert!(
            matches!(err.kind(),HtmlSemanticFailureKind::LimitExceeded{resource:r,..}if r==&resource)
        );
        assert_eq!(s.usage().scratch_records(), 0);
        let stopped = s.usage();
        assert_eq!(s.query(&[]).unwrap_err(), err);
        assert_eq!(s.usage(), stopped);
    }
}
#[test]
fn later_batch_failure_does_not_publish_known_first_property() {
    let (c, id) = single("meta", &[("itemprop", "p"), ("content", "v")]);
    let cx = context(&c);
    let (mut b, d) = builder();
    let n = el(&mut b, &d, "meta", &[]);
    let foreign = b.finish().unwrap().subject_id(&n).unwrap();
    let mut s = HtmlSemanticSession::new(
        &c,
        &cx,
        HtmlValueInterpreter::new(1),
        limits(),
        HtmlSemanticCancellation::new(),
    )
    .unwrap();
    let err = s
        .query(&[
            HtmlSemanticRequest::new(id, HtmlSemanticQuery::MicrodataProperty),
            HtmlSemanticRequest::new(foreign, HtmlSemanticQuery::MicrodataProperty),
        ])
        .unwrap_err();
    assert!(matches!(err.kind(), HtmlSemanticFailureKind::Capture(_)));
    assert_eq!(s.usage().scratch_records(), 0);
    assert_eq!(s.query(&[]).unwrap_err(), err);
}
#[test]
fn document_and_item_walks_exclude_hosted_template_and_shadow_contents() {
    for shadow in [false, true] {
        let (mut b, d) = builder();
        let root = el(&mut b, &d, "div", &[("itemscope", "")]);
        let host = el(&mut b, &d, if shadow { "div" } else { "template" }, &[]);
        let f = if shadow {
            b.add_shadow_root(
                &d,
                &host,
                HtmlCapturedShadowProperties::new(
                    HtmlShadowRootProperties::new(
                        HtmlShadowRootMode::Open,
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
            .unwrap()
        } else {
            let f = b.add_fragment(&d).unwrap();
            b.establish_template_contents(&host, &f).unwrap();
            f
        };
        let hidden = el(
            &mut b,
            &d,
            "meta",
            &[("itemprop", "orphan"), ("content", "not ordinary")],
        );
        cs(&mut b, &hidden, &[]);
        cs(&mut b, &f, &[hidden]);
        cs(&mut b, &host, &[]);
        cs(&mut b, &root, &[host]);
        cs(&mut b, &d, std::slice::from_ref(&root));
        let c = b.finish().unwrap();
        let r = run(
            &c,
            &context(&c),
            &c.subject_id(&root).unwrap(),
            HtmlSemanticQuery::MicrodataItem,
        );
        assert!(known(item(&r).properties()).is_empty());
        let r = run(
            &c,
            &context(&c),
            &c.subject_id(&d).unwrap(),
            HtmlSemanticQuery::MicrodataDocumentAuthor,
        );
        assert_eq!(document(&r).orphans(), &HtmlMicrodataCondition::Satisfied);
    }
}
#[test]
fn exact_work_on_inapplicable_text_is_three_and_one_short_denies_result_write() {
    let (mut b, d) = builder();
    let n = b.add_text(&d, "x".into()).unwrap();
    let c = b.finish().unwrap();
    let id = c.subject_id(&n).unwrap();
    let cx = context(&c);
    for work in [3, 2] {
        let mut s = HtmlSemanticSession::new(
            &c,
            &cx,
            HtmlValueInterpreter::new(0),
            HtmlSemanticLimits::new(work, 1, 0, 0, 100000, 0),
            HtmlSemanticCancellation::new(),
        )
        .unwrap();
        let result = s.query(&[HtmlSemanticRequest::new(
            id.clone(),
            HtmlSemanticQuery::MicrodataItem,
        )]);
        if work == 3 {
            assert!(matches!(
                result.unwrap().results()[0].fact(),
                HtmlSemanticFact::MicrodataItem(HtmlSemanticValue::Inapplicable)
            ));
            assert_eq!(s.usage().work(), 3);
        } else {
            let err = result.unwrap_err();
            assert_eq!(
                err.kind(),
                &HtmlSemanticFailureKind::LimitExceeded {
                    resource: HtmlSemanticResource::Work,
                    limit: 2,
                    required: 3
                }
            );
            assert_eq!(s.usage().work(), 2);
            assert_eq!(s.query(&[]).unwrap_err(), err);
        }
    }
}
#[test]
fn own_type_override_does_not_consult_unavailable_children_or_ancestry() {
    let (mut b, d) = builder();
    let n = el(&mut b, &d, "div", &[("itemscope", ""), ("itemtype", "T T")]);
    let c = b.finish().unwrap();
    let r = run(
        &c,
        &context(&c),
        &c.subject_id(&n).unwrap(),
        HtmlSemanticQuery::MicrodataItem,
    );
    assert_eq!(
        item(&r).type_context().typing(),
        &HtmlMicrodataTyping::Typed
    );
    assert_eq!(
        known(item(&r).type_context().paths())[0]
            .types()
            .processing_tokens()
            .count(),
        2
    );
    assert!(matches!(
        item(&r).properties(),
        HtmlSemanticValue::Missing(_)
    ));
}
#[test]
fn known_contribution_and_cycle_survive_unrelated_unknown_coverage() {
    let (mut b, d) = builder();
    let a = el(
        &mut b,
        &d,
        "div",
        &[
            ("itemscope", ""),
            ("id", "a"),
            ("itemprop", "p"),
            ("itemref", "z"),
        ],
    );
    let z = el(
        &mut b,
        &d,
        "div",
        &[
            ("itemscope", ""),
            ("id", "z"),
            ("itemprop", "p"),
            ("itemref", "a"),
        ],
    );
    let unknown = el(&mut b, &d, "span", &[]);
    cs(&mut b, &a, &[]);
    cs(&mut b, &z, &[]);
    cs(&mut b, &d, &[a.clone(), z, unknown]);
    let c = b.finish().unwrap();
    let r = run(
        &c,
        &context(&c),
        &c.subject_id(&d).unwrap(),
        HtmlSemanticQuery::MicrodataDocumentAuthor,
    );
    assert!(matches!(
        document(&r).cycles(),
        HtmlMicrodataCondition::Violated(_)
    ));
    assert!(matches!(
        document(&r).orphans(),
        HtmlMicrodataCondition::Missing(_)
    ));
    let r = run(
        &c,
        &context(&c),
        &c.subject_id(&a).unwrap(),
        HtmlSemanticQuery::MicrodataAuthor,
    );
    assert_eq!(
        author(&r).contribution(),
        &HtmlMicrodataCondition::Satisfied
    );
    assert!(matches!(
        author(&r).receivers(),
        HtmlSemanticValue::Missing(_)
    ));
}

#[test]
fn every_url_property_role_selects_only_its_source_attribute() {
    for (name, source, local) in [
        ("audio", HtmlMicrodataRawSource::Src, "src"),
        ("embed", HtmlMicrodataRawSource::Src, "src"),
        ("iframe", HtmlMicrodataRawSource::Src, "src"),
        ("img", HtmlMicrodataRawSource::Src, "src"),
        ("source", HtmlMicrodataRawSource::Src, "src"),
        ("track", HtmlMicrodataRawSource::Src, "src"),
        ("video", HtmlMicrodataRawSource::Src, "src"),
        ("a", HtmlMicrodataRawSource::Href, "href"),
        ("area", HtmlMicrodataRawSource::Href, "href"),
        ("link", HtmlMicrodataRawSource::Href, "href"),
        ("object", HtmlMicrodataRawSource::Data, "data"),
    ] {
        let (c, id) = single(name, &[("itemprop", "p"), (local, "raw")]);
        let rev = revision();
        let binding = HtmlMicrodataRawBinding::new(&c, id.clone(), source, 3).unwrap();
        let op = HtmlMicrodataUrlOperation::PropertyAttributeSet {
            revision: rev.clone(),
            event: 0,
            document: binding_document(&c, &binding),
            base_revision: 0,
            encoding_revision: 0,
        };
        let fact = HtmlMicrodataUrlFact::new(
            binding,
            op.clone(),
            Some(op),
            Some(HtmlMicrodataUrlResult::Parsed(Some("serialized".into()))),
        )
        .unwrap();
        let cx = facts_context(
            &c,
            HtmlMicrodataRevision::new(Some(rev), None),
            vec![fact],
            vec![],
            vec![],
        );
        let r = run(&c, &cx, &id, HtmlSemanticQuery::MicrodataProperty);
        assert_eq!(
            prop(&r).value(),
            &HtmlSemanticValue::Known(HtmlMicrodataValue::String("serialized".into()))
        );
    }
}
#[test]
fn no_namespace_local_is_exact_while_prefix_has_no_semantic_role() {
    for (prefix, namespace, local, applies) in [
        (Some("p"), Namespace::none(), "itemprop", true),
        (None, Namespace::svg(), "itemprop", false),
        (None, Namespace::none(), "ITEMPROP", false),
    ] {
        let (mut b, d) = builder();
        let n = b
            .add_element(
                &d,
                QualifiedName::new(Namespace::html(), Some("h".into()), "meta"),
                OrderedAttributes::new([Attribute::new(
                    QualifiedName::new(namespace, prefix.map(DocumentString::from), local),
                    "p",
                )])
                .unwrap(),
                HtmlCapturedIsValue::Unavailable,
                HtmlCaptureRegistryInput::Null,
            )
            .unwrap();
        let c = b.finish().unwrap();
        let r = run(
            &c,
            &context(&c),
            &c.subject_id(&n).unwrap(),
            HtmlSemanticQuery::MicrodataProperty,
        );
        if applies {
            assert_eq!(
                prop(&r).value(),
                &HtmlSemanticValue::Known(HtmlMicrodataValue::String("".into()))
            );
        } else {
            assert!(matches!(
                r.results()[0].fact(),
                HtmlSemanticFact::MicrodataProperty(HtmlSemanticValue::Inapplicable)
            ));
        }
    }
}
#[test]
fn unavailable_and_wrong_operation_url_facts_cannot_be_relabelled() {
    let (c, id) = single("img", &[("itemprop", "p"), ("src", "x")]);
    let rev = revision();
    let b = HtmlMicrodataRawBinding::new(&c, id.clone(), HtmlMicrodataRawSource::Src, 1).unwrap();
    assert!(matches!(
        HtmlMicrodataUrlFact::new(
            b.clone(),
            HtmlMicrodataUrlOperation::ItemIdAuthor(rev.clone()),
            None,
            None
        ),
        Err(HtmlMicrodataFactsError::OperationMismatch)
    ));
    let op = HtmlMicrodataUrlOperation::PropertyAttributeSet {
        revision: rev.clone(),
        event: 7,
        document: binding_document(&c, &b),
        base_revision: 2,
        encoding_revision: 1,
    };
    let wrong = HtmlMicrodataUrlOperation::PropertyAttributeSet {
        revision: rev.clone(),
        event: 8,
        document: binding_document(&c, &b),
        base_revision: 2,
        encoding_revision: 1,
    };
    assert!(matches!(
        HtmlMicrodataUrlFact::new(
            b.clone(),
            op.clone(),
            Some(wrong),
            Some(HtmlMicrodataUrlResult::Parsed(None))
        ),
        Err(HtmlMicrodataFactsError::OperationMismatch)
    ));
    let fact = HtmlMicrodataUrlFact::new(b, op, None, None).unwrap();
    let cx = facts_context(
        &c,
        HtmlMicrodataRevision::new(Some(rev), None),
        vec![fact],
        vec![],
        vec![],
    );
    assert!(matches!(
        prop(&run(&c, &cx, &id, HtmlSemanticQuery::MicrodataProperty)).value(),
        HtmlSemanticValue::Missing(_)
    ));
}
#[test]
fn work_prefix_cuts_leave_no_scratch_and_do_not_resume_after_stop() {
    let (mut b, d) = builder();
    let root = el(&mut b, &d, "div", &[("itemscope", "")]);
    let p = el(&mut b, &d, "meta", &[("itemprop", "p p"), ("content", "v")]);
    cs(&mut b, &p, &[]);
    cs(&mut b, &root, &[p]);
    b.establish_root(&root).unwrap();
    let c = b.finish().unwrap();
    let id = c.subject_id(&root).unwrap();
    let cx = context(&c);
    let mut reached_scratch = false;
    for work in [0, 1, 2, 10, 100, 200, 500, 1000, 2000, 4000, 8000, 12000] {
        let mut s = HtmlSemanticSession::new(
            &c,
            &cx,
            HtmlValueInterpreter::new(3),
            HtmlSemanticLimits::new(work, 10, 1000, 0, 1000000, 10000),
            HtmlSemanticCancellation::new(),
        )
        .unwrap();
        match s.query(&[HtmlSemanticRequest::new(
            id.clone(),
            HtmlSemanticQuery::MicrodataItem,
        )]) {
            Err(err) => {
                assert!(matches!(
                    err.kind(),
                    HtmlSemanticFailureKind::LimitExceeded {
                        resource: HtmlSemanticResource::Work,
                        ..
                    }
                ));
                reached_scratch |= s.usage().peak_scratch_records() > 0;
                assert_eq!(s.usage().scratch_records(), 0);
                let usage = s.usage();
                assert_eq!(s.query(&[]).unwrap_err(), err);
                assert_eq!(s.usage(), usage);
            }
            Ok(r) => assert_eq!(known(item(&r).properties()).len(), 1),
        }
    }
    assert!(reached_scratch);
}
#[test]
fn completed_report_is_not_revoked_by_later_cancellation() {
    let (c, id) = single("meta", &[("itemprop", "p"), ("content", "value")]);
    let cx = context(&c);
    let cancel = HtmlSemanticCancellation::new();
    let mut s = HtmlSemanticSession::new(
        &c,
        &cx,
        HtmlValueInterpreter::new(5),
        limits(),
        cancel.clone(),
    )
    .unwrap();
    let report = s
        .query(&[HtmlSemanticRequest::new(
            id,
            HtmlSemanticQuery::MicrodataProperty,
        )])
        .unwrap();
    cancel.cancel();
    assert_eq!(
        prop(&report).value(),
        &HtmlSemanticValue::Known(HtmlMicrodataValue::String("value".into()))
    );
    assert!(matches!(
        s.query(&[]).unwrap_err().kind(),
        HtmlSemanticFailureKind::Cancelled
    ));
}

#[test]
fn multiple_paths_to_one_type_origin_remain_distinct() {
    let (mut b, d) = builder();
    let a = el(
        &mut b,
        &d,
        "div",
        &[("itemscope", ""), ("itemtype", "T"), ("itemref", "b c")],
    );
    let left = el(
        &mut b,
        &d,
        "div",
        &[
            ("itemscope", ""),
            ("id", "b"),
            ("itemprop", "p"),
            ("itemref", "x"),
        ],
    );
    let right = el(
        &mut b,
        &d,
        "div",
        &[
            ("itemscope", ""),
            ("id", "c"),
            ("itemprop", "p"),
            ("itemref", "x"),
        ],
    );
    let x = el(
        &mut b,
        &d,
        "div",
        &[("itemscope", ""), ("id", "x"), ("itemprop", "p")],
    );
    for n in [&a, &left, &right, &x] {
        cs(&mut b, n, &[]);
    }
    cs(
        &mut b,
        &d,
        &[a.clone(), left.clone(), right.clone(), x.clone()],
    );
    let c = b.finish().unwrap();
    let xid = c.subject_id(&x).unwrap();
    let aid = c.subject_id(&a).unwrap();
    let r = run(&c, &context(&c), &xid, HtmlSemanticQuery::MicrodataItem);
    let paths = known(item(&r).type_context().paths());
    assert_eq!(paths.len(), 2);
    assert_eq!(
        paths[0].items(),
        &[xid.clone(), c.subject_id(&left).unwrap(), aid.clone()]
    );
    assert_eq!(paths[1].items(), &[xid, c.subject_id(&right).unwrap(), aid]);
}
#[test]
fn seeded_typing_is_known_while_full_path_coverage_remains_missing() {
    let (mut b, d) = builder();
    let root = el(
        &mut b,
        &d,
        "div",
        &[("itemscope", ""), ("itemtype", "T"), ("itemref", "x")],
    );
    let x = el(
        &mut b,
        &d,
        "div",
        &[("itemscope", ""), ("id", "x"), ("itemprop", "p")],
    );
    let unknown = el(&mut b, &d, "span", &[]);
    cs(&mut b, &root, &[]);
    cs(&mut b, &x, &[]);
    cs(&mut b, &d, &[root, x.clone(), unknown]);
    let c = b.finish().unwrap();
    let r = run(
        &c,
        &context(&c),
        &c.subject_id(&x).unwrap(),
        HtmlSemanticQuery::MicrodataItem,
    );
    assert_eq!(
        item(&r).type_context().typing(),
        &HtmlMicrodataTyping::Typed
    );
    assert!(matches!(
        item(&r).type_context().paths(),
        HtmlSemanticValue::Missing(_)
    ));
}
#[test]
fn missing_url_value_does_not_discard_a_complete_property_collection() {
    let (mut b, d) = builder();
    let root = el(&mut b, &d, "div", &[("itemscope", "")]);
    let p = el(
        &mut b,
        &d,
        "img",
        &[("itemprop", "reserved.name"), ("src", "relative")],
    );
    cs(&mut b, &p, &[]);
    cs(&mut b, &root, std::slice::from_ref(&p));
    cs(&mut b, &d, std::slice::from_ref(&root));
    let c = b.finish().unwrap();
    let id = c.subject_id(&p).unwrap();
    let r = run(
        &c,
        &context(&c),
        &c.subject_id(&root).unwrap(),
        HtmlSemanticQuery::MicrodataItem,
    );
    let props = known(item(&r).properties());
    assert_eq!(props.len(), 1);
    assert_eq!(props[0].subject(), &id);
    assert!(matches!(props[0].value(), HtmlSemanticValue::Missing(_)));
    let r = run(
        &c,
        &context(&c),
        &c.subject_id(&d).unwrap(),
        HtmlSemanticQuery::MicrodataDocumentAuthor,
    );
    assert_eq!(document(&r).orphans(), &HtmlMicrodataCondition::Satisfied);
}

#[test]
fn itemscope_boolean_author_spelling_does_not_turn_presence_false() {
    let (c, id) = single("div", &[("itemscope", "false")]);
    let cx = context(&c);
    let r = run(&c, &cx, &id, HtmlSemanticQuery::MicrodataAuthor);
    assert!(matches!(
        author(&r).itemscope_author(),
        HtmlAttributeAuthorState::Nonconforming(_)
    ));
    assert_eq!(
        author(&r).itemscope().unwrap().value(),
        &DocumentString::from("false")
    );
    let r = run(&c, &cx, &id, HtmlSemanticQuery::MicrodataItem);
    assert!(item(&r).top_level());
    assert!(known(item(&r).properties()).is_empty());
}

#[test]
fn partial_receiving_crawl_preserves_proved_type_seed_and_contribution() {
    // HTML56965–68 and57249–51: A receives B even when X's unknown subtree
    // prevents claiming a complete property collection or all receiving paths.
    let (mut b, d) = builder();
    let a = el(&mut b, &d, "div", &[("itemscope", ""), ("itemtype", "T")]);
    let child = el(&mut b, &d, "div", &[("itemscope", ""), ("itemprop", "p")]);
    let unknown = el(&mut b, &d, "span", &[]);
    cs(&mut b, &d, std::slice::from_ref(&a));
    cs(&mut b, &a, &[child.clone(), unknown.clone()]);
    cs(&mut b, &child, &[]);
    let c = b.finish().unwrap();
    let cx = context(&c);
    let a = c.subject_id(&a).unwrap();
    let child = c.subject_id(&child).unwrap();
    let missing = HtmlSemanticMissing::OrderedChildren(c.subject_id(&unknown).unwrap());
    let a_result = run(&c, &cx, &a, HtmlSemanticQuery::MicrodataItem);
    assert_eq!(
        item(&a_result).properties(),
        &HtmlSemanticValue::Missing(missing.clone())
    );
    let b_result = run(&c, &cx, &child, HtmlSemanticQuery::MicrodataItem);
    assert_eq!(
        item(&b_result).type_context().paths(),
        &HtmlSemanticValue::Missing(missing.clone())
    );
    assert_eq!(
        item(&b_result).type_context().typing(),
        &HtmlMicrodataTyping::Typed
    );
    let author_result = run(&c, &cx, &child, HtmlSemanticQuery::MicrodataAuthor);
    assert_eq!(
        author(&author_result).contribution(),
        &HtmlMicrodataCondition::Satisfied
    );
    assert_eq!(
        author(&author_result).receivers(),
        &HtmlSemanticValue::Missing(missing)
    );
}

#[test]
fn partial_participating_crawl_preserves_actual_item_cycle_witness() {
    // A receives B through its actual child list; B receives A through the
    // first proved ID. X makes A's collection incomplete but erases neither edge.
    let (mut b, d) = builder();
    let a = el(
        &mut b,
        &d,
        "div",
        &[("itemscope", ""), ("itemprop", "p"), ("id", "a")],
    );
    let child = el(
        &mut b,
        &d,
        "div",
        &[("itemscope", ""), ("itemprop", "p"), ("itemref", "a")],
    );
    let unknown = el(&mut b, &d, "span", &[]);
    cs(&mut b, &d, std::slice::from_ref(&a));
    cs(&mut b, &a, &[child.clone(), unknown.clone()]);
    cs(&mut b, &child, &[]);
    let c = b.finish().unwrap();
    let cx = context(&c);
    let missing = HtmlSemanticMissing::OrderedChildren(c.subject_id(&unknown).unwrap());
    let a_result = run(
        &c,
        &cx,
        &c.subject_id(&a).unwrap(),
        HtmlSemanticQuery::MicrodataItem,
    );
    assert_eq!(
        item(&a_result).properties(),
        &HtmlSemanticValue::Missing(missing.clone())
    );
    let doc_result = run(
        &c,
        &cx,
        &c.subject_id(&d).unwrap(),
        HtmlSemanticQuery::MicrodataDocumentAuthor,
    );
    // DFS starts at A and proves A→B→A, independently of incomplete X.
    assert_eq!(
        document(&doc_result).cycles(),
        &HtmlMicrodataCondition::Violated(c.subject_id(&a).unwrap())
    );
    assert_eq!(
        document(&doc_result).orphans(),
        &HtmlMicrodataCondition::Missing(missing)
    );
}

#[test]
fn incomplete_receiving_coverage_without_proved_edge_stays_missing() {
    let (mut b, d) = builder();
    let a = el(&mut b, &d, "div", &[("itemscope", ""), ("itemtype", "T")]);
    let unknown = el(&mut b, &d, "span", &[]);
    let child = el(&mut b, &d, "div", &[("itemscope", ""), ("itemprop", "p")]);
    cs(&mut b, &d, &[a.clone(), child.clone()]);
    cs(&mut b, &a, std::slice::from_ref(&unknown));
    cs(&mut b, &child, &[]);
    let c = b.finish().unwrap();
    let cx = context(&c);
    let child = c.subject_id(&child).unwrap();
    let missing = HtmlSemanticMissing::OrderedChildren(c.subject_id(&unknown).unwrap());
    let b_result = run(&c, &cx, &child, HtmlSemanticQuery::MicrodataItem);
    assert_eq!(
        item(&b_result).type_context().typing(),
        &HtmlMicrodataTyping::Missing(missing.clone())
    );
    assert_eq!(
        item(&b_result).type_context().paths(),
        &HtmlSemanticValue::Missing(missing.clone())
    );
    let author_result = run(&c, &cx, &child, HtmlSemanticQuery::MicrodataAuthor);
    assert_eq!(
        author(&author_result).contribution(),
        &HtmlMicrodataCondition::Missing(missing.clone())
    );
    let doc_result = run(
        &c,
        &cx,
        &c.subject_id(&d).unwrap(),
        HtmlSemanticQuery::MicrodataDocumentAuthor,
    );
    assert_eq!(
        document(&doc_result).cycles(),
        &HtmlMicrodataCondition::Missing(missing.clone())
    );
    assert_eq!(
        document(&doc_result).orphans(),
        &HtmlMicrodataCondition::Missing(missing)
    );
}
