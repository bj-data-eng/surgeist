//! The walk for the nearest ancestor form takes a step for each record it
//! examines, and a refused step ends it.

use crate::semantics::constructor_work::html;
use crate::semantics::support::builder_with;
use surgeist_html_semantics::{
    HtmlCapture, HtmlCaptureEdgeError, HtmlCaptureElementId, HtmlCaptureError, HtmlCaptureFact,
    HtmlCaptureLimits, HtmlFragmentContext,
};

/// A chain of `depth` elements below the document, named by `name_at`, and the
/// element ids from the top of the chain down.
fn chain(
    depth: usize,
    name_at: impl Fn(usize) -> &'static str,
) -> (HtmlCapture, Vec<HtmlCaptureElementId>) {
    let (mut b, d) = builder_with(HtmlCaptureLimits::new(
        depth + 10,
        4 * depth + 100,
        100,
        40 * depth + 100_000,
        depth + 10,
        0,
        0,
        16_777_216,
    ));
    let elements: Vec<_> = (0..depth).map(|n| html(&mut b, &d, name_at(n))).collect();
    for pair in elements.windows(2) {
        b.establish_children(&pair[0], std::slice::from_ref(&pair[1]))
            .unwrap();
    }
    b.establish_children(&elements[depth - 1], &[]).unwrap();
    b.establish_children(&d, std::slice::from_ref(&elements[0]))
        .unwrap();
    let capture = b.finish().unwrap();
    let ids = elements
        .iter()
        .map(|element| {
            capture
                .element_id(&capture.subject_id(element).unwrap())
                .unwrap()
        })
        .collect();
    (capture, ids)
}

/// Runs the walk from the bottom of the chain, refusing the step numbered
/// `refuse` (from 1), and reports how many steps it took with the result.
fn walk(
    capture: &HtmlCapture,
    from: &HtmlCaptureElementId,
    refuse: Option<usize>,
) -> (
    usize,
    Result<Option<HtmlCaptureElementId>, HtmlCaptureEdgeError<usize>>,
) {
    let mut steps = 0;
    let result = capture.nearest_form_with(from, || {
        steps += 1;
        if Some(steps) == refuse {
            Err(steps)
        } else {
            Ok(())
        }
    });
    (steps, result)
}

#[test]
fn the_walk_takes_one_step_per_record_up_to_the_form_or_the_top() {
    // 20 elements below the document, the form the 8th from the top: the walk from
    // the bottom examines the elements 20 down to 8, 13 records.
    let (capture, ids) = chain(20, |n| if n == 7 { "form" } else { "div" });
    let (steps, found) = walk(&capture, &ids[19], None);
    assert_eq!(steps, 13);
    assert_eq!(found.unwrap(), Some(ids[7].clone()));
    // Without a form it examines the 20 elements and the document: 21 records.
    let (capture, ids) = chain(20, |_| "div");
    let (steps, found) = walk(&capture, &ids[19], None);
    assert_eq!(steps, 21);
    assert_eq!(found.unwrap(), None);
}

#[test]
fn a_refused_step_ends_the_walk_unchanged_and_the_unmetered_walk_agrees_otherwise() {
    let (capture, ids) = chain(20, |n| if n == 7 { "form" } else { "div" });
    for refused in 1..=13 {
        let (steps, result) = walk(&capture, &ids[19], Some(refused));
        assert_eq!(steps, refused, "the walk stops at the refusal");
        assert_eq!(result, Err(HtmlCaptureEdgeError::Interrupted(refused)));
    }
    assert_eq!(
        capture.nearest_form(&ids[19]).unwrap(),
        walk(&capture, &ids[19], None).1.unwrap()
    );
}

#[test]
fn the_fragment_context_meters_its_nearest_form_walk() {
    let (capture, ids) = chain(20, |n| if n == 7 { "form" } else { "div" });
    let target = ids[19].subject().clone();
    let mut steps = 0;
    let context = HtmlFragmentContext::element_with(&capture, &target, || {
        steps += 1;
        Ok::<_, ()>(())
    })
    .unwrap();
    assert_eq!(steps, 13);
    assert_eq!(context.nearest_form(), Some(&ids[7]));
    let mut steps = 0;
    let refused = HtmlFragmentContext::element_with(&capture, &target, || {
        steps += 1;
        if steps == 5 { Err("cancelled") } else { Ok(()) }
    })
    .unwrap_err();
    assert_eq!(steps, 5);
    assert_eq!(refused, HtmlCaptureEdgeError::Interrupted("cancelled"));
    // The unmetered selection is the same context.
    assert_eq!(
        HtmlFragmentContext::element(&capture, &target)
            .unwrap()
            .nearest_form(),
        Some(&ids[7])
    );
}

#[test]
fn unavailable_ancestry_is_still_a_capture_error_through_the_metered_walk() {
    let (mut b, d) = builder_with(HtmlCaptureLimits::new(
        10, 100, 10, 1000, 10, 0, 0, 16_777_216,
    ));
    let orphan = html(&mut b, &d, "div");
    let capture = b.finish().unwrap();
    let id = capture
        .element_id(&capture.subject_id(&orphan).unwrap())
        .unwrap();
    let mut steps = 0;
    let result = capture.nearest_form_with(&id, || {
        steps += 1;
        Ok::<_, ()>(())
    });
    assert_eq!(steps, 1);
    assert!(matches!(
        result,
        Err(HtmlCaptureEdgeError::Capture(
            HtmlCaptureError::MissingFact {
                fact: HtmlCaptureFact::Parent,
                ..
            }
        ))
    ));
}
