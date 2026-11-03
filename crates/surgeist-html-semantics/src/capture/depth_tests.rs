//! Work observed on the admission seam the ordinary path shares with the
//! controlled one: ordinary assembly runs these same builder operations with a
//! no-op controller, so the `Action::Work` stream is the deterministic cost of
//! maintaining depths. The oracle is linear growth with the number of records,
//! whatever order the producer attaches edges in.

use super::preflight::Action;
use super::*;
use surgeist_html::{
    construction::HtmlDocumentMode,
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
