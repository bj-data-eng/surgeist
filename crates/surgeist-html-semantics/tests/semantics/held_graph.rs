//! The item graph a batch holds for its next query.
//!
//! A batch keeps the graph of the last tree a graph query read so the next query
//! of that tree does not build it again. The held graph's scratch stays
//! accounted, so it must never be what makes a batch need more scratch than its
//! largest query needs alone: it is released when a query of another tree or
//! kind begins, before a graph-kind query reserves scratch for anything but the
//! graph, and when the batch ends or fails. Each test fits a batch into the
//! scratch cap that admits its largest single query.

use super::constructor_work::{attributed, big_builder};
use super::lookup_work::roomy_limits;
use surgeist_html::{construction::HtmlDocumentMode, values::HtmlValueInterpreter};
use surgeist_html_semantics::{
    HtmlCapture, HtmlCaptureRegistryInput, HtmlCaptureSubjectId, HtmlControlFacts,
    HtmlControlFactsLimits, HtmlFormAssociation, HtmlFormAssociationState, HtmlScriptingStatus,
    HtmlSemanticCancellation, HtmlSemanticContext, HtmlSemanticContextInput, HtmlSemanticFailure,
    HtmlSemanticFailureKind, HtmlSemanticLimits, HtmlSemanticQuery, HtmlSemanticReport,
    HtmlSemanticRequest, HtmlSemanticResource, HtmlSemanticSession, HtmlSemanticUsage,
};

/// Subjects of a capture of several small documents, so that each query's
/// scratch comes from its own tree and the graph held for one is never part of
/// the tree of the next.
struct Page {
    capture: HtmlCapture,
    context: HtmlSemanticContext,
    /// An untyped item holding a property element, and a second like it, in the
    /// first document.
    item: HtmlCaptureSubjectId,
    other_item: HtmlCaptureSubjectId,
    /// The property element of `item`.
    prop: HtmlCaptureSubjectId,
    /// An element with an `itemref` and no `itemprop`, with 31 ids to find.
    refd: HtmlCaptureSubjectId,
    /// A property element that also has an `itemref`, with 31 ids to find.
    prop_ref: HtmlCaptureSubjectId,
    /// An item with its own type and an `itemref`, with 31 ids to find.
    typed: HtmlCaptureSubjectId,
    /// An input whose form owner is found by its `form` attribute.
    input: HtmlCaptureSubjectId,
}

fn page() -> Page {
    let (mut b, first) = big_builder(300);
    let mut documents = vec![first];
    for _ in 0..4 {
        documents.push(
            b.add_document(
                HtmlDocumentMode::NoQuirks,
                HtmlScriptingStatus::Unavailable,
                HtmlCaptureRegistryInput::Null,
            )
            .unwrap(),
        );
    }
    let leaf = |b: &mut _, d, name: &str, attributes: &[(&str, &str)]| {
        let element = attributed(b, d, name, attributes);
        b.establish_children(&element, &[]).unwrap();
        element
    };
    let ids = |b: &mut _, d, holder: &_| {
        let mut children = vec![];
        for n in 0..31 {
            let id = if n == 0 {
                "x".to_string()
            } else {
                format!("p{n}")
            };
            children.push(leaf(b, d, "p", &[("id", id.as_str())]));
        }
        let _ = holder;
        children
    };
    // Document 0: two untyped items, each with a property element.
    let prop = leaf(&mut b, &documents[0], "span", &[("itemprop", "a")]);
    let item = attributed(&mut b, &documents[0], "div", &[("itemscope", "")]);
    b.establish_children(&item, std::slice::from_ref(&prop))
        .unwrap();
    let other_prop = leaf(&mut b, &documents[0], "span", &[("itemprop", "b")]);
    let other_item = attributed(&mut b, &documents[0], "div", &[("itemscope", "")]);
    b.establish_children(&other_item, std::slice::from_ref(&other_prop))
        .unwrap();
    b.establish_children(&documents[0], &[item.clone(), other_item.clone()])
        .unwrap();
    // Document 1: an itemref element and the ids it names.
    let refd = leaf(&mut b, &documents[1], "div", &[("itemref", "x")]);
    let mut children = vec![refd.clone()];
    children.extend(ids(&mut b, &documents[1], &refd));
    b.establish_children(&documents[1], &children).unwrap();
    // Document 2: an item whose property element has an itemref.
    let prop_ref = leaf(
        &mut b,
        &documents[2],
        "span",
        &[("itemprop", "a"), ("itemref", "x")],
    );
    let holder = attributed(&mut b, &documents[2], "div", &[("itemscope", "")]);
    b.establish_children(&holder, std::slice::from_ref(&prop_ref))
        .unwrap();
    let mut children = vec![holder.clone()];
    children.extend(ids(&mut b, &documents[2], &holder));
    b.establish_children(&documents[2], &children).unwrap();
    // Document 3: a typed item with an itemref.
    let typed = attributed(
        &mut b,
        &documents[3],
        "div",
        &[
            ("itemscope", ""),
            ("itemtype", "https://vocab.example/T"),
            ("itemref", "x"),
        ],
    );
    b.establish_children(&typed, &[]).unwrap();
    let mut children = vec![typed.clone()];
    children.extend(ids(&mut b, &documents[3], &typed));
    b.establish_children(&documents[3], &children).unwrap();
    // Document 4: a form and an input that names it.
    let form = leaf(&mut b, &documents[4], "form", &[("id", "f")]);
    let input = leaf(&mut b, &documents[4], "input", &[("form", "f")]);
    b.establish_children(&documents[4], &[form, input.clone()])
        .unwrap();
    let capture = b.finish().unwrap();
    let id = |node| capture.subject_id(node).unwrap();
    let facts = HtmlControlFacts::new(
        &capture,
        capture.host_facts_revision().clone(),
        vec![HtmlFormAssociation::new(
            id(&input),
            HtmlFormAssociationState::ResetAgainstCapture,
        )],
        vec![],
        vec![],
        vec![],
        HtmlControlFactsLimits::new(1, 1 << 40),
    )
    .unwrap();
    let context = HtmlSemanticContext::new(
        &capture,
        capture.host_facts_revision().clone(),
        HtmlSemanticContextInput::new(vec![]).with_controls(facts),
        0,
    )
    .unwrap();
    Page {
        item: id(&item),
        prop: id(&prop),
        other_item: id(&other_item),
        refd: id(&refd),
        prop_ref: id(&prop_ref),
        typed: id(&typed),
        input: id(&input),
        capture,
        context,
    }
}

fn request(subject: &HtmlCaptureSubjectId, query: HtmlSemanticQuery) -> HtmlSemanticRequest {
    HtmlSemanticRequest::new(subject.clone(), query)
}
fn run(
    page: &Page,
    limits: HtmlSemanticLimits,
    requests: &[HtmlSemanticRequest],
) -> (
    Result<HtmlSemanticReport, HtmlSemanticFailure>,
    HtmlSemanticUsage,
) {
    let mut session = HtmlSemanticSession::new(
        &page.capture,
        &page.context,
        HtmlValueInterpreter::new(10000),
        limits,
        HtmlSemanticCancellation::new(),
    )
    .unwrap();
    let result = session.query(requests);
    (result, session.usage())
}
fn scratch_cap(cap: usize) -> HtmlSemanticLimits {
    let big = 1 << 40;
    HtmlSemanticLimits::new(big, big, cap, 0, big, big)
}
fn peak_of(page: &Page, request: &HtmlSemanticRequest) -> usize {
    let (report, _) = run(page, roomy_limits(), std::slice::from_ref(request));
    report.unwrap().usage().peak_scratch_records()
}

/// Both orders of the two queries fit the scratch cap that admits the larger of
/// them alone, and leave no scratch behind.
fn assert_pair_fits(
    label: &str,
    page: &Page,
    first: HtmlSemanticRequest,
    second: HtmlSemanticRequest,
) {
    let (a, b) = (peak_of(page, &first), peak_of(page, &second));
    assert!(
        a > 0 && b > 0,
        "{label}: both queries must reserve scratch ({a}, {b})"
    );
    let cap = a.max(b);
    for batch in [[first.clone(), second.clone()], [second, first]] {
        let (result, usage) = run(page, scratch_cap(cap), &batch);
        let report = result
            .unwrap_or_else(|failure| panic!("{label}: peaks {a} and {b}, cap {cap}: {failure:?}"));
        assert!(report.usage().peak_scratch_records() <= cap, "{label}");
        assert_eq!(report.usage().scratch_records(), 0, "{label}");
        assert_eq!(usage.scratch_records(), 0, "{label}");
    }
}

#[test]
fn a_held_graph_is_released_before_an_author_query_builds_its_id_index() {
    let p = page();
    assert_pair_fits(
        "property then itemref element",
        &p,
        request(&p.prop, HtmlSemanticQuery::MicrodataAuthor),
        request(&p.refd, HtmlSemanticQuery::MicrodataAuthor),
    );
}

#[test]
fn a_held_graph_is_released_before_a_graph_author_query_builds_its_id_index() {
    let p = page();
    assert_pair_fits(
        "property then property with itemref",
        &p,
        request(&p.prop, HtmlSemanticQuery::MicrodataAuthor),
        request(&p.prop_ref, HtmlSemanticQuery::MicrodataAuthor),
    );
}

#[test]
fn a_held_graph_is_released_before_an_item_with_its_own_types_reserves_scratch() {
    let p = page();
    assert_pair_fits(
        "untyped item then typed item with itemref",
        &p,
        request(&p.item, HtmlSemanticQuery::MicrodataItem),
        request(&p.typed, HtmlSemanticQuery::MicrodataItem),
    );
}

#[test]
fn a_held_graph_is_released_when_the_next_query_is_of_another_kind() {
    let p = page();
    assert_pair_fits(
        "item then form owner",
        &p,
        request(&p.item, HtmlSemanticQuery::MicrodataItem),
        request(&p.input, HtmlSemanticQuery::FormOwner),
    );
}

/// Two documents, each with an untyped item holding a property element.
#[test]
fn a_held_graph_is_released_when_the_next_query_is_of_another_tree() {
    let (mut b, first) = big_builder(8);
    let second = b
        .add_document(
            HtmlDocumentMode::NoQuirks,
            HtmlScriptingStatus::Unavailable,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    let mut items = Vec::new();
    for (count, document) in [(3, &first), (5, &second)] {
        let item = attributed(&mut b, document, "div", &[("itemscope", "")]);
        let props: Vec<_> = (0..count)
            .map(|_| attributed(&mut b, document, "span", &[("itemprop", "name")]))
            .collect();
        for prop in &props {
            b.establish_children(prop, &[]).unwrap();
        }
        b.establish_children(&item, &props).unwrap();
        b.establish_children(document, std::slice::from_ref(&item))
            .unwrap();
        items.push(item);
    }
    let capture = b.finish().unwrap();
    let context = HtmlSemanticContext::new(
        &capture,
        capture.host_facts_revision().clone(),
        HtmlSemanticContextInput::new(vec![]),
        0,
    )
    .unwrap();
    let page = Page {
        item: capture.subject_id(&items[0]).unwrap(),
        other_item: capture.subject_id(&items[1]).unwrap(),
        prop: capture.subject_id(&items[0]).unwrap(),
        refd: capture.subject_id(&items[0]).unwrap(),
        prop_ref: capture.subject_id(&items[0]).unwrap(),
        typed: capture.subject_id(&items[0]).unwrap(),
        input: capture.subject_id(&items[0]).unwrap(),
        capture,
        context,
    };
    // The trees differ in size, so holding the first graph under the second
    // would exceed the cap that admits the larger alone.
    assert_pair_fits(
        "items of two trees",
        &page,
        request(&page.item, HtmlSemanticQuery::MicrodataItem),
        request(&page.other_item, HtmlSemanticQuery::MicrodataItem),
    );
}

/// A batch of graph queries that fails midway leaves no scratch in the session,
/// and its failure still records the graph scratch the batch held when it
/// stopped. By hand: the first document holds two untyped items, each with one
/// property element, so its graph holds 2 items and 2 (item, property) pairs and
/// no edge, and a graph vector reserves exactly one record per push: 4 records.
#[test]
fn a_failed_batch_of_graph_queries_leaves_no_scratch_and_records_the_held_graph() {
    let p = page();
    let queries = [
        request(&p.item, HtmlSemanticQuery::MicrodataItem),
        request(&p.other_item, HtmlSemanticQuery::MicrodataItem),
    ];
    let (one, _) = run(&p, roomy_limits(), &queries[..1]);
    let work_of_first = one.unwrap().usage().work();
    // Stopped by the work cap just after the first query finished, during the
    // second, whose graph is the one held.
    let big = 1 << 40;
    let limits = HtmlSemanticLimits::new(work_of_first + 2, big, big, 0, big, big);
    let (result, usage) = run(&p, limits, &queries);
    let failure = result.unwrap_err();
    assert!(matches!(
        failure.kind(),
        HtmlSemanticFailureKind::LimitExceeded {
            resource: HtmlSemanticResource::Work,
            ..
        }
    ));
    assert_eq!(usage.scratch_records(), 0, "the session releases the graph");
    assert_eq!(failure.usage().scratch_records(), 4);
}

/// An untyped item whose `itemref` names 31 ids, then ten untyped items of
/// three property elements each. The graph of the document holds the 11 items
/// and the 30 (item, property) pairs, and no edge, so G = 41 records; the first
/// item's crawl builds an id index of the 31 ids.
fn crawl_heavy_page() -> (Page, HtmlCaptureSubjectId) {
    let (mut b, d) = big_builder(200);
    let item = attributed(&mut b, &d, "div", &[("itemscope", ""), ("itemref", "x")]);
    b.establish_children(&item, &[]).unwrap();
    let mut children = vec![item.clone()];
    for n in 0..31 {
        let id = if n == 0 {
            "x".to_string()
        } else {
            format!("p{n}")
        };
        let element = attributed(&mut b, &d, "p", &[("id", id.as_str())]);
        b.establish_children(&element, &[]).unwrap();
        children.push(element);
    }
    for _ in 0..10 {
        let props: Vec<_> = (0..3)
            .map(|_| attributed(&mut b, &d, "span", &[("itemprop", "n")]))
            .collect();
        for prop in &props {
            b.establish_children(prop, &[]).unwrap();
        }
        let holder = attributed(&mut b, &d, "div", &[("itemscope", "")]);
        b.establish_children(&holder, &props).unwrap();
        children.push(holder);
    }
    b.establish_children(&d, &children).unwrap();
    let capture = b.finish().unwrap();
    let context = HtmlSemanticContext::new(
        &capture,
        capture.host_facts_revision().clone(),
        HtmlSemanticContextInput::new(vec![]),
        0,
    )
    .unwrap();
    let subject = capture.subject_id(&item).unwrap();
    let page = Page {
        item: subject.clone(),
        other_item: subject.clone(),
        prop: subject.clone(),
        refd: subject.clone(),
        prop_ref: subject.clone(),
        typed: subject.clone(),
        input: subject.clone(),
        capture,
        context,
    };
    (page, subject)
}

/// A single untyped item query holds its tree's graph while it crawls the item,
/// as the batch needs it for the next item, where the previous code had released
/// it before the crawl. Its peak is therefore at most the graph's G records
/// above what it was, G + crawl where it was max(G + type context, crawl). On
/// this page, where G = 41, the graph-building crawl of the same item already
/// reached the peak (measured 106 before the graph was held and 106 after), so
/// the rise is 0. A rise is bounded by G and appears only if the item's own crawl
/// needed more scratch than building the graph did.
#[test]
fn a_single_item_query_peak_is_at_most_the_graph_above_its_previous_peak() {
    let (page, item) = crawl_heavy_page();
    let peak = peak_of(&page, &request(&item, HtmlSemanticQuery::MicrodataItem));
    assert_eq!(peak, 106);
}
