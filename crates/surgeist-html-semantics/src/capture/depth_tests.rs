//! Work observed on the admission seam the ordinary path shares with the
//! controlled one: ordinary assembly runs these same builder operations with a
//! no-op controller, so the `Action::Work` stream is the deterministic cost of
//! maintaining depths. The oracle is linear growth with the number of records,
//! whatever order the producer attaches edges in.

use super::preflight::Action;
use super::*;
use surgeist_html::{
    construction::{
        HtmlDocumentMode, HtmlShadowRootMode, HtmlShadowRootProperties, HtmlSlotAssignment,
    },
    values::{Namespace, OrderedAttributes, QualifiedName},
};

const SIZES: [usize; 4] = [500, 1000, 2000, 4000];

fn builder(n: usize) -> (HtmlCaptureBuilder, HtmlCaptureBuilderSubject) {
    // Room for `n` records of any kind, their edges and a chain `n` deep.
    let mut builder = HtmlCaptureBuilder::new(
        HtmlCaptureLimits::new(4 * n + 8, 16 * n + 16, 0, 100 * n, 4 * n + 8, 0, 0, 1 << 30),
        HtmlHostFactsRevision::fresh(),
    );
    let document = builder
        .add_document(
            HtmlDocumentMode::NoQuirks,
            HtmlScriptingStatus::Disabled,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    (builder, document)
}

fn element(
    builder: &mut HtmlCaptureBuilder,
    document: &HtmlCaptureBuilderSubject,
    name: &str,
) -> HtmlCaptureBuilderSubject {
    builder
        .add_element(
            document,
            QualifiedName::new(Namespace::html(), None, name),
            OrderedAttributes::default(),
            HtmlCapturedIsValue::Absent,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap()
}

/// An admission callback that adds up the work it is asked to admit.
fn tally(work: &mut usize) -> impl FnMut(Action) -> Result<(), ()> + '_ {
    move |action| {
        if let Action::Work(units) = action {
            *work += units;
        }
        Ok(())
    }
}

/// Each doubling of the records may at most double the work, with a small
/// allowance (2.2 to 2) for fixed costs and per-operation logarithmic terms.
fn assert_linear(what: &str, series: [usize; 4]) {
    for pair in series.windows(2) {
        assert!(
            pair[1] * 10 <= pair[0] * 22,
            "{what}: work must grow no faster than 2.2x per doubling, saw {series:?} for {SIZES:?} records"
        );
    }
}

/// A chain of `n` elements attached with `establish_children`, either from the
/// root down or from the leaf up; the work of the attachments alone.
fn children_chain(n: usize, leaf_first: bool) -> usize {
    let (mut builder, document) = builder(n);
    let chain: Vec<_> = (0..n)
        .map(|_| element(&mut builder, &document, "div"))
        .collect();
    let mut work = 0;
    let order: Vec<usize> = if leaf_first {
        (1..n).collect()
    } else {
        (1..n).rev().collect()
    };
    for i in order {
        builder
            .establish_children_controlled(
                &chain[i],
                std::slice::from_ref(&chain[i - 1]),
                &mut tally(&mut work),
            )
            .unwrap();
    }
    assert_eq!(builder.usage().depth(), n - 1);
    work
}

#[test]
fn leaf_first_children_attachment_work_is_linear() {
    assert_linear("leaf-first", SIZES.map(|n| children_chain(n, true)));
}

#[test]
fn root_first_children_attachment_work_is_linear() {
    assert_linear("root-first", SIZES.map(|n| children_chain(n, false)));
}

fn parent_chain(n: usize, leaf_first: bool) -> usize {
    let (mut builder, document) = builder(n);
    let chain: Vec<_> = (0..n)
        .map(|_| element(&mut builder, &document, "div"))
        .collect();
    let mut work = 0;
    let order: Vec<usize> = if leaf_first {
        (1..n).collect()
    } else {
        (1..n).rev().collect()
    };
    for i in order {
        builder
            .establish_parent_controlled(&chain[i - 1], &chain[i], &mut tally(&mut work))
            .unwrap();
    }
    assert_eq!(builder.usage().depth(), n - 1);
    work
}

#[test]
fn leaf_first_parent_attachment_work_is_linear() {
    assert_linear("leaf-first", SIZES.map(|n| parent_chain(n, true)));
}

#[test]
fn root_first_parent_attachment_work_is_linear() {
    assert_linear("root-first", SIZES.map(|n| parent_chain(n, false)));
}

/// Nested templates: each template's contents fragment holds the next template.
/// Leaf-first, every contents attachment lifts the whole nested chain below it.
fn template_chain(n: usize, leaf_first: bool) -> usize {
    let (mut builder, document) = builder(n);
    let templates: Vec<_> = (0..n)
        .map(|_| element(&mut builder, &document, "template"))
        .collect();
    let contents: Vec<_> = (0..n)
        .map(|_| builder.add_fragment(&document).unwrap())
        .collect();
    let mut work = 0;
    let order: Vec<usize> = if leaf_first {
        (0..n - 1).rev().collect()
    } else {
        (0..n - 1).collect()
    };
    for i in order {
        // The fragment holds the next template, then becomes this one's contents.
        // Root first attaches the contents of `i` before the next template's.
        builder
            .establish_children_controlled(
                &contents[i],
                std::slice::from_ref(&templates[i + 1]),
                &mut tally(&mut work),
            )
            .unwrap();
        builder
            .establish_template_contents_controlled(
                &templates[i],
                &contents[i],
                &mut tally(&mut work),
            )
            .unwrap();
    }
    assert_eq!(builder.usage().depth(), 2 * (n - 1));
    work
}

#[test]
fn leaf_first_template_contents_work_is_linear() {
    assert_linear("leaf-first", SIZES.map(|n| template_chain(n, true)));
}

#[test]
fn root_first_template_contents_work_is_linear() {
    assert_linear("root-first", SIZES.map(|n| template_chain(n, false)));
}

/// What a model node is, among the kinds that own or are owned by an edge.
#[derive(Clone, Copy, PartialEq)]
enum Kind {
    Element,
    Template,
    Fragment,
    Shadow,
}

/// The owning edges as the base semantics define them, with every depth
/// recomputed from the owner chain and no use of the builder's own structure.
struct Model {
    kind: Vec<Kind>,
    owner: Vec<Option<usize>>,
    /// The ordinary parent, when the edge is an ordinary one.
    ordinary: Vec<Option<usize>>,
    dependents: Vec<Vec<usize>>,
    complete: Vec<bool>,
    has_contents: Vec<bool>,
    has_shadow: Vec<bool>,
}
impl Model {
    fn new() -> Self {
        Self {
            kind: Vec::new(),
            owner: Vec::new(),
            ordinary: Vec::new(),
            dependents: Vec::new(),
            complete: Vec::new(),
            has_contents: Vec::new(),
            has_shadow: Vec::new(),
        }
    }
    fn push(&mut self, kind: Kind) -> usize {
        self.kind.push(kind);
        self.owner.push(None);
        self.ordinary.push(None);
        self.dependents.push(Vec::new());
        self.complete.push(false);
        self.has_contents.push(false);
        self.has_shadow.push(false);
        self.kind.len() - 1
    }
    fn depth_of(&self, mut at: usize) -> usize {
        let mut depth = 0;
        while let Some(up) = self.owner[at] {
            depth += 1;
            at = up;
        }
        depth
    }
    fn deepest(&self) -> usize {
        (0..self.kind.len())
            .map(|i| self.depth_of(i))
            .max()
            .unwrap_or(0)
    }
    fn within_limit(required: usize) -> Result<(), HtmlCaptureError> {
        if required > MODEL_LIMIT {
            Err(HtmlCaptureError::LimitExceeded {
                resource: HtmlCaptureResource::Depth,
                limit: MODEL_LIMIT,
                required,
            })
        } else {
            Ok(())
        }
    }
    /// The first refusal of attaching the roots in `edges` below `parent`: the
    /// new edge's own depth against the limit, then a breadth-first walk from
    /// the attached roots in which, at each record, reaching `parent` is a cycle
    /// that is reported before the record's depth is checked against the limit.
    fn refusal(
        &self,
        edges: &[(usize, usize)],
        parent: usize,
        single: bool,
    ) -> Result<(), HtmlCaptureError> {
        if single && edges[0].0 == parent {
            return Err(HtmlCaptureError::Cycle);
        }
        let first = self.depth_of(parent) + 1;
        Self::within_limit(first)?;
        let mut walk: Vec<(usize, usize)> = edges.iter().map(|&(c, _)| (c, first)).collect();
        let mut at = 0;
        while at < walk.len() {
            let (record, depth) = walk[at];
            if record == parent {
                return Err(HtmlCaptureError::Cycle);
            }
            Self::within_limit(depth)?;
            for &dependent in &self.dependents[record] {
                walk.push((dependent, depth + 1));
            }
            at += 1;
        }
        Ok(())
    }
}

const MODEL_LIMIT: usize = 6;

/// Counts of what one run of the model exercised.
#[derive(Default)]
struct Coverage {
    shadow_attached: usize,
    shadow_refused: usize,
    parent_attached: usize,
    cycles: usize,
    limits: usize,
    children_attached: usize,
    template_attached: usize,
    template_refused: usize,
}

fn model_run(seed: u64, coverage: &mut Coverage) {
    let mut state = seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1;
    let mut next = move |bound: usize| {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        usize::try_from(state % u64::try_from(bound).unwrap()).unwrap()
    };
    for _ in 0..4 {
        next(2);
    }
    let mut builder = HtmlCaptureBuilder::new(
        HtmlCaptureLimits::new(400, 5000, 0, 10000, MODEL_LIMIT, 0, 0, 1 << 20),
        HtmlHostFactsRevision::fresh(),
    );
    let document = builder
        .add_document(
            HtmlDocumentMode::NoQuirks,
            HtmlScriptingStatus::Disabled,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    let mut model = Model::new();
    let mut handles: Vec<HtmlCaptureBuilderSubject> = Vec::new();
    for _ in 0..250 {
        let n = model.kind.len();
        // Records are created between edges, in no particular order.
        match next(12) {
            0..=2 if n < 60 => {
                handles.push(element(&mut builder, &document, "div"));
                model.push(Kind::Element);
            }
            3 if n < 60 => {
                handles.push(element(&mut builder, &document, "template"));
                model.push(Kind::Template);
            }
            4 if n < 60 => {
                handles.push(builder.add_fragment(&document).unwrap());
                model.push(Kind::Fragment);
            }
            5 | 6 if n > 0 => {
                let host = next(n);
                if !matches!(model.kind[host], Kind::Element | Kind::Template)
                    || model.has_shadow[host]
                {
                    continue;
                }
                let properties = HtmlCapturedShadowProperties::new(
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
                );
                let result = builder.add_shadow_root(
                    &document,
                    &handles[host],
                    properties,
                    HtmlCaptureRegistryInput::Null,
                );
                match (result, Model::within_limit(model.depth_of(host) + 1)) {
                    (Ok(shadow), Ok(())) => {
                        model.has_shadow[host] = true;
                        let i = model.push(Kind::Shadow);
                        handles.push(shadow);
                        model.owner[i] = Some(host);
                        model.dependents[host].push(i);
                        coverage.shadow_attached += 1;
                    }
                    (Err(error), Err(expected)) => {
                        assert_eq!(error, expected, "shadow root, seed {seed}");
                        coverage.shadow_refused += 1;
                    }
                    (result, expected) => {
                        panic!("shadow root, seed {seed}: got {result:?}, model {expected:?}")
                    }
                }
            }
            7..=9 if n > 1 => {
                let (child, parent) = (next(n), next(n));
                if !matches!(model.kind[child], Kind::Element | Kind::Template)
                    || model.owner[child].is_some()
                    || model.complete[parent]
                {
                    continue;
                }
                let expected = model.refusal(&[(child, parent)], parent, true);
                let result = builder.establish_parent(&handles[child], &handles[parent]);
                match (&result, &expected) {
                    (Ok(()), Ok(())) => {
                        model.owner[child] = Some(parent);
                        model.ordinary[child] = Some(parent);
                        model.dependents[parent].push(child);
                        coverage.parent_attached += 1;
                    }
                    (Err(error), Err(expected)) => {
                        assert_eq!(error, expected, "parent, seed {seed}");
                        coverage.cycles += usize::from(*error == HtmlCaptureError::Cycle);
                        coverage.limits += usize::from(*error != HtmlCaptureError::Cycle);
                    }
                    _ => panic!("parent, seed {seed}: got {result:?}, model {expected:?}"),
                }
            }
            10 if n > 1 => {
                let parent = next(n);
                if model.complete[parent] {
                    continue;
                }
                let mut kids: Vec<usize> = (0..n)
                    .filter(|&i| model.ordinary[i] == Some(parent))
                    .collect();
                for _ in 0..1 + next(3) {
                    let child = next(n);
                    if matches!(model.kind[child], Kind::Element | Kind::Template)
                        && model.owner[child].is_none()
                        && !kids.contains(&child)
                    {
                        kids.push(child);
                    }
                }
                if kids.len() > 1 && next(2) == 0 {
                    kids.reverse();
                }
                if kids.is_empty() {
                    continue;
                }
                let edges: Vec<(usize, usize)> = kids
                    .iter()
                    .filter(|&&c| model.ordinary[c] != Some(parent))
                    .map(|&c| (c, parent))
                    .collect();
                let expected = if edges.is_empty() {
                    Ok(())
                } else {
                    model.refusal(&edges, parent, false)
                };
                let children: Vec<_> = kids.iter().map(|&c| handles[c].clone()).collect();
                let result = builder.establish_children(&handles[parent], &children);
                match (&result, &expected) {
                    (Ok(()), Ok(())) => {
                        for &(child, _) in &edges {
                            model.owner[child] = Some(parent);
                            model.ordinary[child] = Some(parent);
                            model.dependents[parent].push(child);
                        }
                        model.complete[parent] = true;
                        coverage.children_attached += 1;
                    }
                    (Err(error), Err(expected)) => {
                        assert_eq!(error, expected, "children, seed {seed}");
                        coverage.cycles += usize::from(*error == HtmlCaptureError::Cycle);
                        coverage.limits += usize::from(*error != HtmlCaptureError::Cycle);
                    }
                    _ => panic!(
                        "children, seed {seed}: got {result:?}, model {expected:?}, edges {edges:?}"
                    ),
                }
            }
            11 if n > 1 => {
                let (template, contents) = (next(n), next(n));
                if model.kind[template] != Kind::Template
                    || model.has_contents[template]
                    || model.kind[contents] != Kind::Fragment
                    || model.owner[contents].is_some()
                {
                    continue;
                }
                let expected = model.refusal(&[(contents, template)], template, true);
                let result =
                    builder.establish_template_contents(&handles[template], &handles[contents]);
                match (&result, &expected) {
                    (Ok(()), Ok(())) => {
                        model.owner[contents] = Some(template);
                        model.has_contents[template] = true;
                        model.dependents[template].push(contents);
                        coverage.template_attached += 1;
                    }
                    (Err(error), Err(expected)) => {
                        assert_eq!(error, expected, "template contents, seed {seed}");
                        coverage.template_refused += 1;
                    }
                    _ => {
                        panic!("template contents, seed {seed}: got {result:?}, model {expected:?}")
                    }
                }
            }
            _ => continue,
        }
        assert_eq!(
            builder.usage().depth(),
            model.deepest(),
            "usage depth, seed {seed}"
        );
    }
    let capture = builder.finish().unwrap();
    assert_eq!(
        capture.usage().depth(),
        model.deepest(),
        "finish depth, seed {seed}"
    );
}

/// Every owning edge kind, in arbitrary order with records created between
/// them, agrees with the base semantics on success, on the exact refusal
/// (cycle before limit at each record reached, the new edge's limit first), on
/// the depth in use after every operation and on the depth `finish` reports.
#[test]
fn random_edge_orders_agree_with_the_base_semantics() {
    let mut coverage = Coverage::default();
    for seed in 0..3000 {
        model_run(seed, &mut coverage);
    }
    assert!(
        coverage.shadow_attached > 1000
            && coverage.shadow_refused > 100
            && coverage.parent_attached > 1000
            && coverage.cycles > 100
            && coverage.limits > 100
            && coverage.children_attached > 1000
            && coverage.template_attached > 100
            && coverage.template_refused > 10,
        "the model must exercise every edge kind and refusal"
    );
}

/// One of the three metered edge operations, run on a fresh builder holding a
/// child and a parent to join, the parent already below a third record.
#[derive(Clone, Copy, Debug)]
enum Edge {
    Parent,
    Children,
    TemplateContents,
}

/// The builder, the child and the parent of the edge.
fn edge_builder(
    edge: Edge,
) -> (
    HtmlCaptureBuilder,
    HtmlCaptureBuilderSubject,
    HtmlCaptureBuilderSubject,
) {
    let (mut builder, document) = builder(8);
    let (child, parent) = match edge {
        Edge::Parent | Edge::Children => (
            element(&mut builder, &document, "div"),
            element(&mut builder, &document, "span"),
        ),
        Edge::TemplateContents => (
            builder.add_fragment(&document).unwrap(),
            element(&mut builder, &document, "template"),
        ),
    };
    let above = element(&mut builder, &document, "p");
    builder.establish_parent(&parent, &above).unwrap();
    (builder, child, parent)
}

/// The work the controlled seam reports for the edge.
fn reported_work(edge: Edge) -> usize {
    let (mut builder, child, parent) = edge_builder(edge);
    let mut work = 0;
    match edge {
        Edge::Parent => builder
            .establish_parent_controlled(&child, &parent, &mut tally(&mut work))
            .unwrap(),
        Edge::Children => builder
            .establish_children_controlled(
                &parent,
                std::slice::from_ref(&child),
                &mut tally(&mut work),
            )
            .unwrap(),
        Edge::TemplateContents => builder
            .establish_template_contents_controlled(&parent, &child, &mut tally(&mut work))
            .unwrap(),
    }
    work
}

/// The metered method on a fresh builder, refusing its `stop`-th step when
/// given: the steps taken, the result, and the usage before and after.
fn metered(
    edge: Edge,
    stop: Option<usize>,
) -> (
    usize,
    Result<(), HtmlCaptureEdgeError<usize>>,
    HtmlCaptureUsage,
    HtmlCaptureUsage,
) {
    let (mut builder, child, parent) = edge_builder(edge);
    let before = builder.usage();
    let mut steps = 0;
    let mut step = || {
        let at = steps;
        steps += 1;
        if Some(at) == stop { Err(at) } else { Ok(()) }
    };
    let result = match edge {
        Edge::Parent => builder.establish_parent_with(&child, &parent, &mut step),
        Edge::Children => {
            builder.establish_children_with(&parent, std::slice::from_ref(&child), &mut step)
        }
        Edge::TemplateContents => {
            builder.establish_template_contents_with(&parent, &child, &mut step)
        }
    };
    (steps, result, before, builder.usage())
}

#[test]
fn metered_edges_take_one_step_for_each_unit_the_builder_reports() {
    for edge in [Edge::Parent, Edge::Children, Edge::TemplateContents] {
        let work = reported_work(edge);
        assert!(work > 0, "{edge:?} reports work");
        let (steps, result, before, after) = metered(edge, None);
        assert_eq!(result, Ok(()), "{edge:?}");
        assert_eq!(steps, work, "{edge:?}");
        assert_ne!(before, after, "{edge:?}: the edge adds usage");
    }
}

#[test]
fn a_metered_edge_returns_the_first_refusal_unchanged_and_establishes_nothing() {
    for edge in [Edge::Parent, Edge::Children, Edge::TemplateContents] {
        for stop in 0..reported_work(edge) {
            let (steps, result, before, after) = metered(edge, Some(stop));
            assert_eq!(
                result,
                Err(HtmlCaptureEdgeError::Interrupted(stop)),
                "{edge:?} at {stop}"
            );
            assert_eq!(steps, stop + 1, "{edge:?}: no step after the refusal");
            assert_eq!(before, after, "{edge:?} at {stop}");
        }
    }
}

#[test]
fn a_refused_metered_edge_can_be_established_again_with_a_permissive_step() {
    for edge in [Edge::Parent, Edge::Children, Edge::TemplateContents] {
        for stop in 0..reported_work(edge) {
            let (mut builder, child, parent) = edge_builder(edge);
            let mut at = 0;
            let mut refuse = || {
                at += 1;
                if at == stop + 1 { Err(stop) } else { Ok(()) }
            };
            let refused = match edge {
                Edge::Parent => builder.establish_parent_with(&child, &parent, &mut refuse),
                Edge::Children => builder.establish_children_with(
                    &parent,
                    std::slice::from_ref(&child),
                    &mut refuse,
                ),
                Edge::TemplateContents => {
                    builder.establish_template_contents_with(&parent, &child, &mut refuse)
                }
            };
            assert_eq!(
                refused,
                Err(HtmlCaptureEdgeError::Interrupted(stop)),
                "{edge:?} at {stop}"
            );
            let mut permissive = || Ok::<_, usize>(());
            let again = match edge {
                Edge::Parent => builder.establish_parent_with(&child, &parent, &mut permissive),
                Edge::Children => builder.establish_children_with(
                    &parent,
                    std::slice::from_ref(&child),
                    &mut permissive,
                ),
                Edge::TemplateContents => {
                    builder.establish_template_contents_with(&parent, &child, &mut permissive)
                }
            };
            assert_eq!(again, Ok(()), "{edge:?} retried after {stop}");
        }
    }
}

#[test]
fn a_metered_edge_error_exposes_its_cause() {
    use std::error::Error as _;
    let capture: HtmlCaptureEdgeError<HtmlCaptureError> =
        HtmlCaptureEdgeError::Capture(HtmlCaptureError::Cycle);
    let interrupted: HtmlCaptureEdgeError<HtmlCaptureError> =
        HtmlCaptureEdgeError::Interrupted(HtmlCaptureError::DuplicateChild);
    assert_eq!(
        capture.source().unwrap().to_string(),
        HtmlCaptureError::Cycle.to_string()
    );
    assert_eq!(
        interrupted.source().unwrap().to_string(),
        HtmlCaptureError::DuplicateChild.to_string()
    );
}

#[test]
fn a_metered_edge_reports_a_capture_refusal_as_such() {
    let (mut builder, document) = builder(8);
    let element = element(&mut builder, &document, "div");
    let mut steps = 0;
    let result = builder.establish_parent_with(&element, &element, || -> Result<(), ()> {
        steps += 1;
        Ok(())
    });
    assert_eq!(
        result,
        Err(HtmlCaptureEdgeError::Capture(HtmlCaptureError::Cycle))
    );
    assert!(steps > 0, "the refusal was reached through metered work");
}
