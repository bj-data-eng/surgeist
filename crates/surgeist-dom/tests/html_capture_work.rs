//! `DomSnapshot::html_capture` plans and attaches a capture in work linear in
//! the number of captured nodes. The charged work is read through the public
//! `DomHtmlCaptureLimits::work` bound: the smallest bound that lets a capture
//! finish is exactly the work it charged, because each step charges before it
//! runs and refuses only when the bound would be exceeded.
use std::sync::atomic::AtomicBool;
use surgeist_dom::*;
use surgeist_html::{
    construction::HtmlDocumentMode,
    values::{Namespace as HtmlNamespace, OrderedAttributes, QualifiedName},
};
use surgeist_html_semantics::{
    HtmlCaptureBuilder, HtmlCaptureLimits, HtmlCaptureRegistryInput, HtmlCapturedIsValue,
    HtmlControlFactsError, HtmlElementReferenceFactsError, HtmlGlobalInheritanceError,
    HtmlHostFactsRevision, HtmlInertFactError, HtmlLanguageDirectionError, HtmlOutputDefaultError,
    HtmlScriptingStatus, HtmlSemanticContextError,
};

/// Four sizes, so three doublings. Large enough that a quadratic planner is far
/// from the linear growth bound and small enough that its RED run stays short.
const SIZES: [usize; 4] = [256, 512, 1024, 2048];
/// A linear count grows 2x per doubling; the slack covers per-document
/// constants and the logarithmic root find of the capture builder.
const GROWTH_BOUND: f64 = 2.2;

#[derive(Clone, Copy, Debug)]
enum Shape {
    /// One chain, each element created after its parent.
    ChainParentFirst,
    /// One chain, each element created before its parent.
    ChainLeafFirst,
    /// One parent with `n` children, the parent created first.
    WideParentFirst,
    /// One parent with `n` children, the parent created last.
    WideParentLast,
    /// `n` elements that no parent holds.
    Detached,
    /// `n` inputs owned by one form.
    ControlOwners,
    /// `n` buttons whose `commandfor` names one element.
    CommandTargets,
    /// `n` templates, each with its contents fragment.
    Templates,
    /// `n` hosts, each with a shadow root.
    ShadowRoots,
}

fn document(shape: Shape, n: usize) -> (DomSnapshot, DocumentId) {
    let names = DomNameLimits::new(16_777_216, 16_384, 4_000_000, 16_777_216).unwrap();
    let limits = DomLimits::new(3 * n + 64, 1 << 22, 8 * n + 256, 0, 0, 16, 131072, names).unwrap();
    let mut dom = Dom::new(limits).unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let mut made = Vec::new();
    match shape {
        Shape::ChainParentFirst | Shape::ChainLeafFirst => {
            for _ in 0..n {
                made.push(batch.create_element_ns(&document, Namespace::html(), "div"));
            }
            let at_depth = |depth: usize| match shape {
                Shape::ChainLeafFirst => &made[n - 1 - depth],
                _ => &made[depth],
            };
            for depth in 1..n {
                batch.insert_before(at_depth(depth), at_depth(depth - 1), None);
            }
            batch.insert_before(at_depth(0), &document, None);
        }
        Shape::Detached => {
            for _ in 0..n {
                batch.create_element_ns(&document, Namespace::html(), "div");
            }
        }
        Shape::ControlOwners | Shape::CommandTargets | Shape::Templates | Shape::ShadowRoots => {
            let top = batch.create_element_ns(&document, Namespace::html(), "div");
            batch.insert_before(&top, &document, None);
            let owner = match shape {
                Shape::ControlOwners => {
                    let form = batch.create_element_ns(&document, Namespace::html(), "form");
                    batch.insert_before(&form, &top, None);
                    form
                }
                _ => top.clone(),
            };
            let target = batch.create_element_ns(&document, Namespace::html(), "span");
            batch.insert_before(&target, &top, None);
            for _ in 0..n {
                let node = match shape {
                    Shape::ControlOwners => {
                        batch.create_element_ns(&document, Namespace::html(), "input")
                    }
                    Shape::CommandTargets => {
                        let button =
                            batch.create_element_ns(&document, Namespace::html(), "button");
                        batch.set_command_for_element(&button, Some((&target).into()));
                        button
                    }
                    Shape::Templates => {
                        batch.create_element_ns(&document, Namespace::html(), "template")
                    }
                    _ => {
                        let host = batch.create_element_ns(&document, Namespace::html(), "div");
                        batch
                            .attach_shadow(&host, DomShadowRootInit::new(HtmlShadowRootMode::Open));
                        host
                    }
                };
                batch.insert_before(&node, &owner, None);
            }
        }
        Shape::WideParentFirst | Shape::WideParentLast => {
            let parent_first = matches!(shape, Shape::WideParentFirst);
            let mut parent =
                parent_first.then(|| batch.create_element_ns(&document, Namespace::html(), "div"));
            for _ in 0..n {
                made.push(batch.create_element_ns(&document, Namespace::html(), "span"));
            }
            let parent = parent
                .take()
                .unwrap_or_else(|| batch.create_element_ns(&document, Namespace::html(), "div"));
            for child in &made {
                batch.insert_before(child, &parent, None);
            }
            batch.insert_before(&parent, &document, None);
        }
    }
    let commit = dom.apply(batch, &AtomicBool::new(false)).unwrap();
    let document = commit.document(&document).unwrap();
    (dom.snapshot(), document)
}

/// `Ok(())` when the capture finishes within `work`, `Err(())` when the bound
/// refuses it. Any other outcome is a defect of the fixture.
fn captures_within(snapshot: &DomSnapshot, document: DocumentId, n: usize, work: usize) -> bool {
    let result = snapshot.html_capture(
        document,
        None,
        HtmlCaptureLimits::new(
            4 * n + 16,
            16 * n + 64,
            4 * n + 16,
            100 * n + 100,
            4 * n + 16,
            0,
            0,
            1 << 30,
        ),
        DomHtmlCaptureLimits::new(4 * n + 16, work),
        &AtomicBool::new(false),
    );
    match result {
        Ok(_) => true,
        Err(DomHtmlCaptureError::ResourceLimit {
            resource: DomHtmlCaptureResource::Work,
            limit,
        }) => {
            assert_eq!(limit, work);
            false
        }
        // Each producer validates its facts under the same work bound, and
        // reports its own refusal when that bound runs out first.
        Err(
            DomHtmlCaptureError::Language(HtmlLanguageDirectionError::ValidationWork {
                limit, ..
            })
            | DomHtmlCaptureError::Inert(HtmlInertFactError::ValidationWork { limit, .. })
            | DomHtmlCaptureError::GlobalInheritance(HtmlGlobalInheritanceError::ValidationWork {
                limit,
                ..
            })
            | DomHtmlCaptureError::Controls(HtmlControlFactsError::ValidationWork { limit, .. })
            | DomHtmlCaptureError::OutputDefaults(HtmlOutputDefaultError::ValidationWork {
                limit,
                ..
            })
            | DomHtmlCaptureError::Context(HtmlSemanticContextError::ControlsFacts(
                HtmlControlFactsError::ValidationWork { limit, .. },
            ))
            | DomHtmlCaptureError::ElementReferences(
                HtmlElementReferenceFactsError::ValidationWork { limit, .. },
            ),
        ) => {
            assert_eq!(limit, work);
            false
        }
        Err(other) => panic!("capture of {n} nodes failed for another reason: {other:?}"),
    }
}

/// The least work bound under which the capture finishes.
fn charged_work(snapshot: &DomSnapshot, document: DocumentId, n: usize) -> usize {
    let mut high = 1;
    while !captures_within(snapshot, document, n, high) {
        high *= 2;
    }
    let mut low = high / 2;
    // `low` refuses (or is zero) and `high` finishes.
    while high - low > 1 {
        let middle = low + (high - low) / 2;
        if captures_within(snapshot, document, n, middle) {
            high = middle;
        } else {
            low = middle;
        }
    }
    high
}

fn assert_linear(shape: Shape) {
    let mut previous: Option<(usize, usize)> = None;
    for n in SIZES {
        let (snapshot, document) = document(shape, n);
        let work = charged_work(&snapshot, document, n);
        if let Some((before_n, before)) = previous {
            let growth = work as f64 / before as f64;
            assert!(
                growth <= GROWTH_BOUND,
                "{shape:?}: charged work grew {growth:.2}x from {before_n} nodes ({before}) to {n} nodes ({work})"
            );
        }
        previous = Some((n, work));
    }
}

#[test]
fn chain_capture_charges_work_linear_in_nodes_when_parents_are_created_first() {
    assert_linear(Shape::ChainParentFirst);
}

#[test]
fn chain_capture_charges_work_linear_in_nodes_when_leaves_are_created_first() {
    assert_linear(Shape::ChainLeafFirst);
}

#[test]
fn wide_capture_charges_work_linear_in_nodes_when_the_parent_is_created_first() {
    assert_linear(Shape::WideParentFirst);
}

#[test]
fn wide_capture_charges_work_linear_in_nodes_when_the_parent_is_created_last() {
    assert_linear(Shape::WideParentLast);
}

#[test]
fn control_capture_charges_work_linear_in_nodes_when_one_form_owns_every_input() {
    assert_linear(Shape::ControlOwners);
}

#[test]
fn command_target_capture_charges_work_linear_in_nodes_when_every_button_names_one_target() {
    assert_linear(Shape::CommandTargets);
}

#[test]
fn template_capture_charges_work_linear_in_nodes_when_every_template_has_contents() {
    assert_linear(Shape::Templates);
}

#[test]
fn shadow_capture_charges_work_linear_in_nodes_when_every_host_has_a_root() {
    assert_linear(Shape::ShadowRoots);
}

/// The steps the capture builder reports when it establishes the children of
/// the document and of each of `n` elements in order: one chain when `chained`,
/// no edge at all otherwise. This is the capture's own attach order, measured
/// on a builder the adapter is not part of.
fn builder_edge_steps(n: usize, chained: bool) -> usize {
    let limits = HtmlCaptureLimits::new(n + 8, 16 * n + 64, 0, 100 * n + 100, n + 8, 0, 0, 1 << 30);
    let mut builder = HtmlCaptureBuilder::new(limits, HtmlHostFactsRevision::fresh());
    let document = builder
        .add_document(
            HtmlDocumentMode::NoQuirks,
            HtmlScriptingStatus::Disabled,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    let elements: Vec<_> = (0..n)
        .map(|_| {
            builder
                .add_element(
                    &document,
                    QualifiedName::new(HtmlNamespace::html(), None, "div"),
                    OrderedAttributes::default(),
                    HtmlCapturedIsValue::Absent,
                    HtmlCaptureRegistryInput::Null,
                )
                .unwrap()
        })
        .collect();
    let mut steps = 0;
    let mut count = || {
        steps += 1;
        Ok::<(), ()>(())
    };
    let first = if chained { &elements[..1] } else { &[] };
    builder
        .establish_children_with(&document, first, &mut count)
        .unwrap();
    for (at, element) in elements.iter().enumerate() {
        let next = if chained && at + 1 < n {
            &elements[at + 1..at + 2]
        } else {
            &[]
        };
        builder
            .establish_children_with(element, next, &mut count)
            .unwrap();
    }
    steps
}

#[test]
fn the_steps_the_builder_takes_to_attach_edges_are_charged_to_the_work_limit() {
    for n in [64, 128] {
        let (chain, chain_document) = document(Shape::ChainParentFirst, n);
        let (loose, loose_document) = document(Shape::Detached, n);
        // Both documents plan, stage and export the same `n` divs, and both
        // establish the children of every element; only the chain has edges.
        let attached = charged_work(&chain, chain_document, n);
        let baseline = charged_work(&loose, loose_document, n);
        let edges = builder_edge_steps(n, true) - builder_edge_steps(n, false);
        assert!(edges > n, "the chain's edges take builder steps");
        assert!(
            attached >= baseline + edges,
            "{n} nodes: the chain charged {attached}, the same nodes without edges {baseline}, \
             and the builder reports {edges} steps for the edges alone"
        );
        // One step below the minimum is refused for want of work.
        assert!(!captures_within(&chain, chain_document, n, attached - 1));
    }
}

#[test]
fn every_work_limit_on_an_attach_heavy_document_refuses_for_work_or_finishes() {
    let n = 24;
    let (snapshot, document) = document(Shape::ChainParentFirst, n);
    let needed = charged_work(&snapshot, document, n);
    let stride = (needed / 300).max(1);
    for limit in (0..needed + stride)
        .step_by(stride)
        .chain([needed - 1, needed])
    {
        assert_eq!(
            captures_within(&snapshot, document, n, limit),
            limit >= needed,
            "limit {limit} of {needed}"
        );
    }
}

#[test]
fn a_cancelled_capture_is_refused_as_cancelled() {
    let (snapshot, document) = document(Shape::ChainParentFirst, 8);
    let result = snapshot.html_capture(
        document,
        None,
        HtmlCaptureLimits::new(64, 256, 0, 1024, 64, 0, 0, 1 << 30),
        DomHtmlCaptureLimits::new(64, usize::MAX),
        &AtomicBool::new(true),
    );
    assert!(matches!(result, Err(DomHtmlCaptureError::Cancelled)));
}
