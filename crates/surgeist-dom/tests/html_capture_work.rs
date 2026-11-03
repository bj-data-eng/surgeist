//! `DomSnapshot::html_capture` plans and attaches a capture in work linear in
//! the number of captured nodes. The charged work is read through the public
//! `DomHtmlCaptureLimits::work` bound: the smallest bound that lets a capture
//! finish is exactly the work it charged, because each step charges before it
//! runs and refuses only when the bound would be exceeded.
use std::sync::atomic::AtomicBool;
use surgeist_dom::*;
use surgeist_html_semantics::HtmlCaptureLimits;

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
}

fn document(shape: Shape, n: usize) -> (DomSnapshot, DocumentId) {
    let names = DomNameLimits::new(16_777_216, 16_384, 4_000_000, 16_777_216).unwrap();
    let limits = DomLimits::new(2 * n + 64, 1 << 22, 8 * n + 256, 0, 0, 16, 131072, names).unwrap();
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
        HtmlCaptureLimits::new(n + 8, 16 * n + 64, 0, 100 * n + 100, n + 8, 0, 0, 1 << 30),
        DomHtmlCaptureLimits::new(n + 8, work),
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
