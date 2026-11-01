//! Independently derived WM4 named margin-box baselines through public layout.
mod atomic_margin_edges;
mod atomic_receivers;
mod sideways_atomics;
mod upright;
mod used_alphabetic;

use std::{cell::Cell, convert::Infallible, slice, sync::Arc};
use surgeist_layout as l;
use surgeist_text as t;

#[derive(Clone)]
struct Published<S: l::LayoutScalar> {
    caches: Vec<l::CacheOf<S>>,
    unrounded: Vec<Option<l::NodeOutputOf<S>>>,
    batch: Option<l::CompletedLayoutBatchOf<u32, S>>,
}
struct Host<S: l::LayoutScalar> {
    inputs: Vec<l::LayoutInputOf<S>>,
    children: Vec<Vec<u32>>,
    non_box: l::NodeInputOf<S>,
    published: Published<S>,
    hits: Vec<Cell<usize>>,
    prepares: Cell<usize>,
    commits: usize,
}
impl<S: l::LayoutScalar> l::Traverse for Host<S> {
    type Node = u32;
    type Scalar = S;
    type Children<'a>
        = std::iter::Copied<slice::Iter<'a, u32>>
    where
        Self: 'a;
    fn children(&self, node: u32) -> Self::Children<'_> {
        self.children[node as usize].iter().copied()
    }
    fn child_count(&self, node: u32) -> usize {
        self.children[node as usize].len()
    }
    fn child(&self, node: u32, index: usize) -> u32 {
        self.children[node as usize][index]
    }
}
impl<S: l::LayoutScalar> l::LayoutTree for Host<S> {
    type MeasureError = Infallible;
    fn node_input(&self, node: u32) -> &l::NodeInputOf<S> {
        self.inputs[node as usize].as_box().unwrap_or(&self.non_box)
    }
    fn layout_input(&self, node: u32) -> l::LayoutInputViewOf<'_, S> {
        self.inputs[node as usize].as_view()
    }
    fn cache_get(
        &self,
        node: u32,
        input: &l::ComputeInputOf<S>,
        context: l::CacheKeyContext,
    ) -> Option<l::ComputeOutputOf<S>> {
        let result = self.published.caches[node as usize].get_with_context(input, context);
        if result.is_some() {
            self.hits[node as usize].set(self.hits[node as usize].get() + 1);
        }
        result
    }
    fn unrounded_layout(&self, node: u32) -> Option<l::NodeOutputOf<S>> {
        self.published.unrounded[node as usize]
    }
}
impl<S: l::LayoutScalar> l::LayoutBatchSink<u32, S> for Host<S> {
    type Error = Infallible;
    type Prepared = Published<S>;
    fn prepare_layout_batch(
        &self,
        batch: &l::CompletedLayoutBatchOf<u32, S>,
    ) -> Result<Self::Prepared, Self::Error> {
        self.prepares.set(self.prepares.get() + 1);
        let mut next = self.published.clone();
        for &node in batch.invalidated_nodes() {
            next.caches[node as usize].clear();
            next.unrounded[node as usize] = None;
        }
        for row in batch.cache_clear_entries() {
            next.caches[row.node() as usize].clear();
        }
        for row in batch.unrounded_entries() {
            next.unrounded[row.node() as usize] = Some(row.output());
        }
        for row in batch.cache_store_entries() {
            next.caches[row.node() as usize].store_with_context(
                row.input(),
                row.context(),
                row.output(),
            );
        }
        next.batch = Some(batch.clone());
        Ok(next)
    }
    fn commit_layout_batch(&mut self, prepared: Self::Prepared) {
        self.published = prepared;
        self.commits += 1;
    }
}
impl<S: l::LayoutScalar> Host<S> {
    fn new(root: l::NodeInputOf<S>) -> Self {
        let mut host = Self {
            inputs: vec![],
            children: vec![],
            non_box: l::NodeInputOf::non_box(),
            published: Published {
                caches: vec![],
                unrounded: vec![],
                batch: None,
            },
            hits: vec![],
            prepares: Cell::new(0),
            commits: 0,
        };
        assert_eq!(host.push(None, l::LayoutInputOf::box_input(root)), 0);
        host
    }
    fn push(&mut self, parent: Option<u32>, input: l::LayoutInputOf<S>) -> u32 {
        let node = u32::try_from(self.inputs.len()).unwrap();
        self.inputs.push(input);
        self.children.push(vec![]);
        self.published.caches.push(l::CacheOf::new());
        self.published.unrounded.push(None);
        self.hits.push(Cell::new(0));
        if let Some(parent) = parent {
            self.children[parent as usize].push(node);
        }
        node
    }
    fn change(&mut self, node: u32, f: impl FnOnce(&mut l::NodeInputOf<S>)) {
        let mut input = self.inputs[node as usize].as_box().unwrap().clone();
        f(&mut input);
        self.inputs[node as usize] = l::LayoutInputOf::box_input(input);
    }
    fn compute(
        &self,
        mode: l::WritingMode,
        changed: &[u32],
    ) -> Result<l::CompletedLayoutBatchOf<u32, S>, l::LayoutErrorOf<u32, S>> {
        l::compute_layout_invalidated(self, 0, request(mode), changed)
    }
}
fn n<S: l::LayoutScalar>(value: f64) -> S {
    S::from_f64(value)
}
fn px<S: l::LayoutScalar>(value: f64) -> l::PreferredSizeOf<S> {
    l::PreferredSizeOf::value(l::LengthPercentageOf::px(n(value)).unwrap())
}
fn request<S: l::LayoutScalar>(mode: l::WritingMode) -> l::LayoutRootRequestOf<S> {
    let available = if mode == l::WritingMode::HorizontalTb {
        l::Size::new(
            l::AvailableOf::definite(n(100.0)),
            l::AvailableOf::MAX_CONTENT,
        )
    } else {
        l::Size::new(
            l::AvailableOf::MAX_CONTENT,
            l::AvailableOf::definite(n(100.0)),
        )
    };
    l::LayoutRootRequestOf::viewport(available).unwrap()
}
fn physical_size<S: l::LayoutScalar>(mode: l::WritingMode, inline: f64, cross: f64) -> l::Size<S> {
    if mode == l::WritingMode::HorizontalTb {
        l::Size::new(n(inline), n(cross))
    } else {
        l::Size::new(n(cross), n(inline))
    }
}
fn root<S: l::LayoutScalar>(
    mode: l::WritingMode,
    direction: l::Direction,
    context: l::InlineBaselineContext,
) -> l::NodeInputOf<S> {
    l::NodeInputOf {
        display: l::Display::Block,
        writing_mode: mode,
        direction,
        inline_baseline_context: context,
        text_align: l::TextAlign::Start,
        text_align_last: l::TextAlignLast::Start,
        size: if mode == l::WritingMode::HorizontalTb {
            l::Size::new(px(100.0), l::PreferredSizeOf::AUTO)
        } else {
            l::Size::new(l::PreferredSizeOf::AUTO, px(100.0))
        },
        ..l::NodeInputOf::default()
    }
}
fn atomic<S: l::LayoutScalar>(
    mode: l::WritingMode,
    direction: l::Direction,
    cross: f64,
    margins: (f64, f64),
) -> l::NodeInputOf<S> {
    let mut input = l::NodeInputOf {
        display: l::Display::InlineBlock,
        writing_mode: mode,
        direction,
        vertical_align: l::VerticalAlignOf::Baseline,
        inline_baseline_context: l::InlineBaselineContext::Physical,
        atomic_inline_participation: Some(l::AtomicInlineParticipation::new(
            t::BidiLevel::try_new(u8::from(direction == l::Direction::Rtl)).unwrap(),
            l::InlineBreakOpportunity::Allowed,
        )),
        size: if mode == l::WritingMode::HorizontalTb {
            l::Size::new(px(20.0), px(cross))
        } else {
            l::Size::new(px(cross), px(20.0))
        },
        ..l::NodeInputOf::default()
    };
    let over = l::LengthAutoOf::value(l::LengthPercentageOf::px(n(margins.0)).unwrap());
    let under = l::LengthAutoOf::value(l::LengthPercentageOf::px(n(margins.1)).unwrap());
    match mode {
        l::WritingMode::HorizontalTb => {
            input.margin.top = over;
            input.margin.bottom = under;
        }
        l::WritingMode::VerticalRl | l::WritingMode::VerticalLr | l::WritingMode::SidewaysRl => {
            input.margin.right = over;
            input.margin.left = under;
        }
        l::WritingMode::SidewaysLr => {
            input.margin.left = over;
            input.margin.right = under;
        }
    }
    input
}
fn node<S: l::LayoutScalar>(
    entries: &[l::LayoutOutputEntryOf<u32, S>],
    id: u32,
) -> l::NodeOutputOf<S> {
    let mut matching = entries.iter().filter(|row| row.node() == id);
    let output = matching.next().unwrap().output();
    assert!(matching.next().is_none(), "one real output per node");
    output
}
fn named<S: l::LayoutScalar>(
    facts: l::NamedBaselinesOf<S>,
    name: l::BaselineName,
    axis: l::PhysicalAxis,
    offset: f64,
) {
    let pair = facts.get(name);
    for fact in [pair.first(), pair.last()] {
        match fact {
            l::BaselineFactOf::Available(coordinate) => {
                assert_eq!(coordinate.axis(), axis);
                assert_eq!(coordinate.offset(), n(offset));
            }
            other => panic!("expected genuine named coordinate, got {other:?}"),
        }
    }
}
fn absent<S: l::LayoutScalar>(facts: l::NamedBaselinesOf<S>) {
    for name in [l::BaselineName::Alphabetic, l::BaselineName::Central] {
        let pair = facts.get(name);
        assert_eq!(pair.first(), l::BaselineFactOf::Absent);
        assert_eq!(pair.last(), l::BaselineFactOf::Absent);
    }
}
fn missing<S: l::LayoutScalar>(
    result: Result<l::CompletedLayoutBatchOf<u32, S>, l::LayoutErrorOf<u32, S>>,
) {
    let error = result.unwrap_err();
    assert!(
        matches!(
            error.kind(),
            l::LayoutErrorKindOf::MissingContext(l::LayoutMissingContext::NamedInlineBaseline)
        ),
        "{error:?}"
    );
}
fn semantic<S: l::LayoutScalar>(
    batch: &l::CompletedLayoutBatchOf<u32, S>,
) -> Vec<(u32, l::NodeOutputOf<S>)> {
    let mut rows = batch
        .unrounded_entries()
        .iter()
        .map(|entry| (entry.node(), entry.output()))
        .collect::<Vec<_>>();
    rows.sort_by_key(|row| row.0);
    rows
}

fn margin_matrix<S: l::LayoutScalar>() {
    // Each row is authored independently: marginsA/B, root cross extent,
    // H/SLR A/B Alpha, H/SLR A/B Central, other-X A/B Alpha/Central.
    for (ma, mb, extent, forward_alpha, forward_central, reverse_alpha, reverse_central) in [
        (
            (2.0, 2.0),
            (2.0, 2.0),
            20.0,
            [2.0, 12.0],
            [2.0, 7.0],
            [2.0, 2.0],
            [2.0, 7.0],
        ),
        (
            (1.0, 5.0),
            (3.0, 1.0),
            22.0,
            [1.0, 15.0],
            [1.0, 9.0],
            [5.0, 1.0],
            [5.0, 7.0],
        ),
        (
            (-2.0, 6.0),
            (4.0, 0.0),
            20.0,
            [-2.0, 14.0],
            [-2.0, 9.0],
            [6.0, 0.0],
            [6.0, 5.0],
        ),
    ] {
        for mode in [
            l::WritingMode::HorizontalTb,
            l::WritingMode::VerticalRl,
            l::WritingMode::VerticalLr,
            l::WritingMode::SidewaysRl,
            l::WritingMode::SidewaysLr,
        ] {
            for direction in [l::Direction::Ltr, l::Direction::Rtl] {
                for name in [l::BaselineName::Alphabetic, l::BaselineName::Central] {
                    let mut host = Host::<S>::new(root(
                        mode,
                        direction,
                        l::InlineBaselineContext::Named(name),
                    ));
                    host.push(
                        Some(0),
                        l::LayoutInputOf::box_input(atomic(mode, direction, 16.0, ma)),
                    );
                    host.push(
                        Some(0),
                        l::LayoutInputOf::box_input(atomic(mode, direction, 6.0, mb)),
                    );
                    let batch = host.compute(mode, &[]).unwrap();
                    let alpha = name == l::BaselineName::Alphabetic;
                    let reverse = matches!(
                        mode,
                        l::WritingMode::VerticalRl
                            | l::WritingMode::VerticalLr
                            | l::WritingMode::SidewaysRl
                    );
                    let cross = match (reverse, alpha) {
                        (false, true) => forward_alpha,
                        (false, false) => forward_central,
                        (true, true) => reverse_alpha,
                        (true, false) => reverse_central,
                    };
                    let inline = match (mode, direction) {
                        (l::WritingMode::SidewaysLr, l::Direction::Ltr) => [80.0, 60.0],
                        (l::WritingMode::SidewaysLr, l::Direction::Rtl) => [0.0, 20.0],
                        (_, l::Direction::Ltr) => [0.0, 20.0],
                        (_, l::Direction::Rtl) => [80.0, 60.0],
                    };
                    for entries in [batch.unrounded_entries(), batch.final_entries()] {
                        for (index, id) in [1, 2].into_iter().enumerate() {
                            let expected = if mode == l::WritingMode::HorizontalTb {
                                l::Point::new(n(inline[index]), n(cross[index]))
                            } else {
                                l::Point::new(n(cross[index]), n(inline[index]))
                            };
                            assert_eq!(
                                node(entries, id).location,
                                expected,
                                "{mode:?}/{direction:?}/{name:?}/{ma:?}/{mb:?}"
                            );
                            assert_eq!(
                                node(entries, id).size,
                                physical_size(mode, 20.0, if id == 1 { 16.0 } else { 6.0 })
                            );
                            assert_eq!(node(entries, id).source_index, l::SourceIndex::new(index));
                            absent(node(entries, id).named_baselines);
                        }
                        assert_eq!(entries.len(), 3);
                        assert_eq!(node(entries, 0).size, physical_size(mode, 100.0, extent));
                        let offset = if alpha {
                            if reverse { 0.0 } else { extent }
                        } else {
                            extent / 2.0
                        };
                        named(
                            node(entries, 0).named_baselines,
                            name,
                            if mode == l::WritingMode::HorizontalTb {
                                l::PhysicalAxis::Vertical
                            } else {
                                l::PhysicalAxis::Horizontal
                            },
                            offset,
                        );
                        let other = if alpha {
                            l::BaselineName::Central
                        } else {
                            l::BaselineName::Alphabetic
                        };
                        let unknown = node(entries, 0).named_baselines.get(other);
                        assert_eq!(unknown.first(), l::BaselineFactOf::Unknown);
                        assert_eq!(unknown.last(), l::BaselineFactOf::Unknown);
                    }
                    assert!(batch.placed_text().is_empty());
                    assert!(batch.final_inline_fragments().is_empty());
                }
            }
        }
    }
}
#[test]
fn named_margin_boxes_use_literal_line_sides_both_directions_f32() {
    margin_matrix::<f32>();
}
#[test]
fn named_margin_boxes_use_literal_line_sides_both_directions_f64() {
    margin_matrix::<f64>();
}

fn constructors<S: l::LayoutScalar>(largest: S) {
    assert_eq!(
        l::InlineBaselineContext::default(),
        l::InlineBaselineContext::Physical
    );
    assert_eq!(
        l::InlineMetricsOf::<S>::default().reference(),
        l::InlineBaselineContext::Physical
    );
    // These are public value-constructor contracts, never fabricated outputs
    // returned by a LayoutTree or used as a measured baseline in the fitter.
    let size = l::Size::new(n(10.0), n(20.0));
    let raw_projection = l::Point::new(None, Some(n(8.0)));
    let projected = l::ComputeOutputOf::<S>::from_sizes_and_baselines(
        size,
        size,
        l::BaselinesOf::first(raw_projection),
    );
    assert_eq!(projected.first_baselines.y, Some(n(8.0)));
    for output in [
        l::ComputeOutputOf::<S>::DEFAULT,
        l::ComputeOutputOf::from_sizes(size, size),
        projected,
        l::ComputeOutputOf::from_sizes_and_first_baselines(size, size, raw_projection),
        l::ComputeOutputOf::from_outer_size(size),
    ] {
        for name in [l::BaselineName::Alphabetic, l::BaselineName::Central] {
            let facts = output.named_baselines.get(name);
            assert_eq!(facts.first(), l::BaselineFactOf::Unknown);
            assert_eq!(facts.last(), l::BaselineFactOf::Unknown);
        }
    }
    absent(l::ComputeOutputOf::<S>::HIDDEN.named_baselines);
    for number in [n(-2.0), n(-0.0), S::ZERO, largest] {
        let coordinate =
            l::BaselineCoordinateOf::try_new(l::PhysicalAxis::Horizontal, number).unwrap();
        assert_eq!(coordinate.axis(), l::PhysicalAxis::Horizontal);
        assert_eq!(coordinate.offset(), number);
        if number == S::ZERO {
            assert!(!coordinate.offset().to_f64().is_sign_negative());
        }
    }
    for number in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        let error: l::FiniteScalarErrorOf<S> =
            l::BaselineCoordinateOf::try_new(l::PhysicalAxis::Vertical, n::<S>(number))
                .unwrap_err();
        assert!(matches!(error, l::FiniteScalarErrorOf::NonFinite { .. }));
    }
}
#[test]
fn named_coordinates_retain_finite_signed_values_and_normalize_zero_f32() {
    constructors::<f32>(f32::MAX);
}
#[test]
fn named_coordinates_retain_finite_signed_values_and_normalize_zero_f64() {
    constructors::<f64>(f64::MAX);
}

fn wrapper<S: l::LayoutScalar>(
    host: &mut Host<S>,
    mode: l::WritingMode,
    direction: l::Direction,
    own: l::InlineBaselineContext,
    inner: l::InlineBaselineContext,
) -> u32 {
    let mut outer = atomic(mode, direction, 0.0, (0.0, 0.0));
    outer.inline_baseline_context = own;
    if mode == l::WritingMode::HorizontalTb {
        outer.size.height = l::PreferredSizeOf::AUTO;
    } else {
        outer.size.width = l::PreferredSizeOf::AUTO;
    }
    let over = l::LengthOf::value(l::LengthPercentageOf::px(n(2.0)).unwrap());
    let under = l::LengthOf::value(l::LengthPercentageOf::px(n(6.0)).unwrap());
    match mode {
        l::WritingMode::HorizontalTb => {
            outer.padding.top = over;
            outer.padding.bottom = under;
        }
        l::WritingMode::VerticalRl | l::WritingMode::VerticalLr | l::WritingMode::SidewaysRl => {
            outer.padding.right = over;
            outer.padding.left = under;
        }
        l::WritingMode::SidewaysLr => {
            outer.padding.left = over;
            outer.padding.right = under;
        }
    }
    let outer_id = host.push(Some(0), l::LayoutInputOf::box_input(outer));
    let mut block = root(mode, direction, inner);
    if mode == l::WritingMode::HorizontalTb {
        block.size.width = px(20.0);
    } else {
        block.size.height = px(20.0);
    }
    let block_id = host.push(Some(outer_id), l::LayoutInputOf::box_input(block));
    host.push(
        Some(block_id),
        l::LayoutInputOf::box_input(atomic(mode, direction, 10.0, (0.0, 0.0))),
    );
    outer_id
}
fn propagation<S: l::LayoutScalar>() {
    for mode in [
        l::WritingMode::HorizontalTb,
        l::WritingMode::VerticalRl,
        l::WritingMode::VerticalLr,
        l::WritingMode::SidewaysRl,
        l::WritingMode::SidewaysLr,
    ] {
        for direction in [l::Direction::Ltr, l::Direction::Rtl] {
            for name in [l::BaselineName::Alphabetic, l::BaselineName::Central] {
                for different_own_name in [false, true] {
                    let other = if name == l::BaselineName::Alphabetic {
                        l::BaselineName::Central
                    } else {
                        l::BaselineName::Alphabetic
                    };
                    let mut host = Host::<S>::new(root(
                        mode,
                        direction,
                        l::InlineBaselineContext::Named(name),
                    ));
                    host.push(
                        Some(0),
                        l::LayoutInputOf::box_input(atomic(mode, direction, 30.0, (0.0, 0.0))),
                    );
                    assert_eq!(
                        wrapper(
                            &mut host,
                            mode,
                            direction,
                            l::InlineBaselineContext::Named(if different_own_name {
                                other
                            } else {
                                name
                            }),
                            l::InlineBaselineContext::Named(name)
                        ),
                        2
                    );
                    let batch = host.compute(mode, &[]).unwrap();
                    let alpha = name == l::BaselineName::Alphabetic;
                    let reverse = matches!(
                        mode,
                        l::WritingMode::VerticalRl
                            | l::WritingMode::VerticalLr
                            | l::WritingMode::SidewaysRl
                    );
                    let (extent, peer_cross, wrapper_cross, baseline, wrapper_baseline) =
                        match (reverse, alpha) {
                            (false, true) => (36.0, 0.0, 18.0, 30.0, 12.0),
                            (false, false) => (30.0, 0.0, 8.0, 15.0, 7.0),
                            (true, true) => (36.0, 6.0, 0.0, 6.0, 6.0),
                            (true, false) => (30.0, 0.0, 4.0, 15.0, 11.0),
                        };
                    let inline = match (mode, direction) {
                        (l::WritingMode::SidewaysLr, l::Direction::Ltr) => [80.0, 60.0],
                        (l::WritingMode::SidewaysLr, l::Direction::Rtl) => [0.0, 20.0],
                        (_, l::Direction::Ltr) => [0.0, 20.0],
                        (_, l::Direction::Rtl) => [80.0, 60.0],
                    };
                    let axis = if mode == l::WritingMode::HorizontalTb {
                        l::PhysicalAxis::Vertical
                    } else {
                        l::PhysicalAxis::Horizontal
                    };
                    for entries in [batch.unrounded_entries(), batch.final_entries()] {
                        for (id, cross) in [(1, peer_cross), (2, wrapper_cross)] {
                            let location = if mode == l::WritingMode::HorizontalTb {
                                l::Point::new(n(inline[id as usize - 1]), n(cross))
                            } else {
                                l::Point::new(n(cross), n(inline[id as usize - 1]))
                            };
                            assert_eq!(
                                node(entries, id).location,
                                location,
                                "{mode:?}/{direction:?}/{name:?}/{different_own_name}"
                            );
                            assert_eq!(
                                node(entries, id).size,
                                physical_size(mode, 20.0, if id == 1 { 30.0 } else { 18.0 })
                            );
                        }
                        assert_eq!(node(entries, 0).size, physical_size(mode, 100.0, extent));
                        named(node(entries, 0).named_baselines, name, axis, baseline);
                        named(
                            node(entries, 2).named_baselines,
                            name,
                            axis,
                            wrapper_baseline,
                        );
                        named(
                            node(entries, 3).named_baselines,
                            name,
                            axis,
                            if alpha {
                                if reverse { 0.0 } else { 10.0 }
                            } else {
                                5.0
                            },
                        );
                        assert_eq!(
                            node(entries, 3).location,
                            if mode == l::WritingMode::HorizontalTb {
                                l::Point::new(S::ZERO, n(2.0))
                            } else {
                                l::Point::new(n(if reverse { 6.0 } else { 2.0 }), S::ZERO)
                            }
                        );
                        assert_eq!(node(entries, 3).size, physical_size(mode, 20.0, 10.0));
                        assert_eq!(node(entries, 4).size, physical_size(mode, 20.0, 10.0));
                        assert_eq!(node(entries, 2).source_index, l::SourceIndex::new(1));
                        assert_eq!(node(entries, 3).source_index, l::SourceIndex::ZERO);
                        assert_eq!(node(entries, 4).source_index, l::SourceIndex::ZERO);
                        absent(node(entries, 1).named_baselines);
                        absent(node(entries, 4).named_baselines);
                        assert_eq!(entries.len(), 5);
                    }
                }
            }
        }
    }
}
#[test]
fn genuine_child_named_facts_translate_through_boxes_independently_of_own_context_f32() {
    propagation::<f32>();
}
#[test]
fn genuine_child_named_facts_translate_through_boxes_independently_of_own_context_f64() {
    propagation::<f64>();
}

fn authority_refusals<S: l::LayoutScalar>() {
    let h = l::WritingMode::HorizontalTb;
    let d = l::Direction::Ltr;
    let alpha = l::InlineBaselineContext::Named(l::BaselineName::Alphabetic);
    for child_context in [
        l::InlineBaselineContext::Physical,
        l::InlineBaselineContext::Named(l::BaselineName::Central),
    ] {
        let mut host = Host::<S>::new(root(h, d, alpha));
        wrapper(&mut host, h, d, child_context, child_context);
        missing(host.compute(h, &[]));
    }
    let mut orthogonal = Host::<S>::new(root(h, d, alpha));
    wrapper(&mut orthogonal, l::WritingMode::VerticalRl, d, alpha, alpha);
    let error = orthogonal.compute(h, &[]).unwrap_err();
    assert!(
        matches!(
            error.kind(),
            l::LayoutErrorKindOf::InvalidInput(l::LayoutInvalidInputOf::InlineBaseline(
                l::InlineBaselineError::AxisMismatch {
                    expected: l::PhysicalAxis::Vertical,
                    actual: l::PhysicalAxis::Horizontal
                }
            ))
        ),
        "{error:?}"
    );
    // Physical is a concrete valid domain, not missing named parent context.
    let mut physical = Host::<S>::new(root(h, d, l::InlineBaselineContext::Physical));
    wrapper(
        &mut physical,
        h,
        d,
        l::InlineBaselineContext::Physical,
        l::InlineBaselineContext::Physical,
    );
    let batch = physical.compute(h, &[]).unwrap();
    for name in [l::BaselineName::Alphabetic, l::BaselineName::Central] {
        assert_eq!(
            node(batch.unrounded_entries(), 0)
                .named_baselines
                .get(name)
                .first(),
            l::BaselineFactOf::Unknown
        );
    }
    for reference in [l::InlineBaselineContext::Physical, alpha] {
        let mut host = Host::<S>::new(root(h, d, alpha));
        let control = l::LineBreakInputOf::new()
            .with_metrics(l::InlineMetricsOf::try_new(reference, n(8.0), n(10.0)).unwrap());
        host.push(Some(0), l::LayoutInputOf::line_break(control));
        let result = host.compute(h, &[]);
        if reference == alpha {
            let batch = result.unwrap();
            assert_eq!(
                node(batch.unrounded_entries(), 0).size,
                l::Size::new(n(100.0), n(10.0))
            );
            named(
                node(batch.unrounded_entries(), 0).named_baselines,
                l::BaselineName::Alphabetic,
                l::PhysicalAxis::Vertical,
                8.0,
            );
        } else {
            let error = result.unwrap_err();
            assert!(
                matches!(error.kind(), l::LayoutErrorKindOf::InvalidInput(l::LayoutInvalidInputOf::InlineBaseline(l::InlineBaselineError::ReferenceMismatch { expected, actual })) if *expected == alpha && *actual == reference),
                "{error:?}"
            );
        }
    }
    for alignment in [l::VerticalAlignOf::Top, l::VerticalAlignOf::Bottom] {
        for has_tail in [false, true] {
            let control = l::LineBreakInputOf::new()
                .with_metrics(
                    l::InlineMetricsOf::try_new(
                        if has_tail {
                            alpha
                        } else {
                            l::InlineBaselineContext::Physical
                        },
                        n(8.0),
                        n(10.0),
                    )
                    .unwrap(),
                )
                .with_vertical_align(alignment);
            let mut host = Host::<S>::new(root(h, d, alpha));
            host.push(Some(0), l::LayoutInputOf::line_break(control));
            if has_tail {
                host.push(
                    Some(0),
                    l::LayoutInputOf::box_input(atomic(h, d, 6.0, (0.0, 0.0))),
                );
            }
            let top = host.compute(h, &[]).unwrap();
            assert_eq!(
                node(top.unrounded_entries(), 0).size.height,
                n(if has_tail { 20.0 } else { 10.0 })
            );
            let facts = node(top.unrounded_entries(), 0)
                .named_baselines
                .get(l::BaselineName::Alphabetic);
            assert_eq!(facts.first(), l::BaselineFactOf::Unknown);
            if has_tail {
                match facts.last() {
                    l::BaselineFactOf::Available(coordinate) => {
                        assert_eq!(coordinate.axis(), l::PhysicalAxis::Vertical);
                        assert_eq!(coordinate.offset(), n(18.0));
                    }
                    other => panic!("last real line has an actual named anchor, got {other:?}"),
                }
            } else {
                assert_eq!(facts.last(), l::BaselineFactOf::Unknown);
            }
            let mut parent = Host::<S>::new(root(h, d, alpha));
            let mut outer = atomic(h, d, 0.0, (0.0, 0.0));
            outer.size.height = l::PreferredSizeOf::AUTO;
            outer.inline_baseline_context = alpha;
            outer.baseline_source = l::BaselineSource::First;
            let wrapper_id = parent.push(Some(0), l::LayoutInputOf::box_input(outer));
            parent.push(Some(wrapper_id), l::LayoutInputOf::line_break(control));
            if has_tail {
                parent.push(
                    Some(wrapper_id),
                    l::LayoutInputOf::box_input(atomic(h, d, 6.0, (0.0, 0.0))),
                );
            }
            // Unknown first authority cannot be skipped to an available last.
            missing(parent.compute(h, &[]));
        }
        // Only extent is used on the first Top/Bottom line. The real next
        // line consumes the carried raw8/2 as baseline contributions, so its
        // reference must genuinely match the NamedAlpha receiving context.
        let mut physical_tail = Host::<S>::new(root(h, d, alpha));
        physical_tail.push(
            Some(0),
            l::LayoutInputOf::line_break(
                l::LineBreakInputOf::new()
                    .with_metrics(
                        l::InlineMetricsOf::try_new(
                            l::InlineBaselineContext::Physical,
                            n(8.0),
                            n(10.0),
                        )
                        .unwrap(),
                    )
                    .with_vertical_align(alignment),
            ),
        );
        physical_tail.push(
            Some(0),
            l::LayoutInputOf::box_input(atomic(h, d, 6.0, (0.0, 0.0))),
        );
        let error = physical_tail.compute(h, &[]).unwrap_err();
        assert!(
            matches!(error.kind(), l::LayoutErrorKindOf::InvalidInput(l::LayoutInvalidInputOf::InlineBaseline(l::InlineBaselineError::ReferenceMismatch { expected, actual })) if *expected == alpha && *actual == l::InlineBaselineContext::Physical),
            "{error:?}"
        );
    }
    for display in [l::Display::Flex, l::Display::Grid] {
        let mut parent = Host::<S>::new(root(h, d, alpha));
        let mut outer = atomic(h, d, 0.0, (0.0, 0.0));
        outer.size.height = l::PreferredSizeOf::AUTO;
        outer.inline_baseline_context = alpha;
        let wrapper_id = parent.push(Some(0), l::LayoutInputOf::box_input(outer));
        let mut generic = root(h, d, l::InlineBaselineContext::Physical);
        generic.display = display;
        generic.size.width = px(20.0);
        let generic_id = parent.push(Some(wrapper_id), l::LayoutInputOf::box_input(generic));
        let mut leaf = root(h, d, l::InlineBaselineContext::Physical);
        leaf.size = l::Size::new(px(10.0), px(10.0));
        parent.push(Some(generic_id), l::LayoutInputOf::box_input(leaf));
        missing(parent.compute(h, &[]));
    }
}
#[test]
fn genuine_unknown_axis_and_control_reference_mismatches_refuse_precisely_f32() {
    authority_refusals::<f32>();
}
#[test]
fn genuine_unknown_axis_and_control_reference_mismatches_refuse_precisely_f64() {
    authority_refusals::<f64>();
}

fn pair<S: l::LayoutScalar>(name: l::BaselineName) -> Host<S> {
    let mode = l::WritingMode::HorizontalTb;
    let direction = l::Direction::Ltr;
    let mut host = Host::new(root(mode, direction, l::InlineBaselineContext::Named(name)));
    host.push(
        Some(0),
        l::LayoutInputOf::box_input(atomic(mode, direction, 16.0, (2.0, 2.0))),
    );
    host.push(
        Some(0),
        l::LayoutInputOf::box_input(atomic(mode, direction, 6.0, (2.0, 2.0))),
    );
    host
}
fn cache_and_atomicity<S: l::LayoutScalar>() {
    let mode = l::WritingMode::HorizontalTb;
    let mut host = pair::<S>(l::BaselineName::Alphabetic);
    let alpha = host.compute(mode, &[]).unwrap();
    let original = semantic(&alpha);
    assert_eq!(
        node(alpha.unrounded_entries(), 2).location,
        l::Point::new(n(20.0), n(12.0))
    );
    alpha.apply_to(&mut host).unwrap();
    let before_hits: usize = host.hits.iter().map(Cell::get).sum();
    let warm = host.compute(mode, &[]).unwrap();
    assert_eq!(semantic(&warm), original);
    assert!(
        host.hits.iter().map(Cell::get).sum::<usize>() > before_hits,
        "actual eligible host cache must answer a repeated request"
    );
    warm.apply_to(&mut host).unwrap();
    host.change(0, |input| {
        input.inline_baseline_context = l::InlineBaselineContext::Named(l::BaselineName::Central)
    });
    let central = host.compute(mode, &[0]).unwrap();
    let fresh = pair::<S>(l::BaselineName::Central)
        .compute(mode, &[])
        .unwrap();
    assert_eq!(semantic(&central), semantic(&fresh));
    assert_eq!(
        node(central.unrounded_entries(), 2).location,
        l::Point::new(n(20.0), n(7.0))
    );
    named(
        node(central.unrounded_entries(), 0).named_baselines,
        l::BaselineName::Central,
        l::PhysicalAxis::Vertical,
        10.0,
    );
    central.apply_to(&mut host).unwrap();
    host.change(2, |input| {
        input.margin.top = l::LengthAutoOf::value(l::LengthPercentageOf::px(n(4.0)).unwrap());
        input.margin.bottom = l::LengthAutoOf::ZERO;
    });
    let changed_margin = host.compute(mode, &[2]).unwrap();
    let mut independent = pair::<S>(l::BaselineName::Central);
    independent.change(2, |input| {
        input.margin.top = l::LengthAutoOf::value(l::LengthPercentageOf::px(n(4.0)).unwrap());
        input.margin.bottom = l::LengthAutoOf::ZERO;
    });
    let fresh_margin = independent.compute(mode, &[]).unwrap();
    assert_eq!(semantic(&changed_margin), semantic(&fresh_margin));
    assert_eq!(
        node(changed_margin.unrounded_entries(), 2).location,
        l::Point::new(n(20.0), n(9.0))
    );
    changed_margin.apply_to(&mut host).unwrap();
    let accepted = semantic(host.published.batch.as_ref().unwrap());
    let counts = (host.prepares.get(), host.commits);
    // Actual previously baseline-less child now owns a real Physical numeric
    // line. Its named facts are Unknown, not genuine baseline absence.
    host.push(
        Some(1),
        l::LayoutInputOf::box_input(atomic(mode, l::Direction::Ltr, 10.0, (0.0, 0.0))),
    );
    missing(host.compute(mode, &[1]));
    assert_eq!((host.prepares.get(), host.commits), counts);
    assert_eq!(semantic(host.published.batch.as_ref().unwrap()), accepted);
    assert_eq!(semantic(&alpha), original);
    host.children[1].clear();
    let restored = host.compute(mode, &[1]).unwrap();
    assert_eq!(semantic(&restored), semantic(&fresh_margin));
    restored.apply_to(&mut host).unwrap();
    host.change(0, |input| {
        input.inline_baseline_context = l::InlineBaselineContext::Named(l::BaselineName::Alphabetic)
    });
    let back = host.compute(mode, &[0]).unwrap();
    independent.change(0, |input| {
        input.inline_baseline_context = l::InlineBaselineContext::Named(l::BaselineName::Alphabetic)
    });
    assert_eq!(
        semantic(&back),
        semantic(&independent.compute(mode, &[0]).unwrap())
    );
    assert_eq!(semantic(&alpha), original);
}
#[test]
fn genuine_cache_name_and_margin_changes_preserve_semantics_and_prior_batches_f32() {
    cache_and_atomicity::<f32>();
}
#[test]
fn genuine_cache_name_and_margin_changes_preserve_semantics_and_prior_batches_f64() {
    cache_and_atomicity::<f64>();
}

fn overflowing_synthesis<S: l::LayoutScalar>(largest: S) {
    let mode = l::WritingMode::HorizontalTb;
    let mut host = Host::<S>::new(root(
        mode,
        l::Direction::Ltr,
        l::InlineBaselineContext::Named(l::BaselineName::Alphabetic),
    ));
    host.push(
        Some(0),
        l::LayoutInputOf::box_input(atomic(
            mode,
            l::Direction::Ltr,
            largest.to_f64(),
            (0.0, 0.0),
        )),
    );
    // Genuine finite maximum control must reach the real owner first. If an
    // earlier capability refuses it, that is qualification evidence, not proof
    // of this later overflow criterion or grounds to weaken the input.
    let control = host.compute(mode, &[]).unwrap();
    assert_eq!(node(control.unrounded_entries(), 0).size.height, largest);
    named(
        node(control.unrounded_entries(), 0).named_baselines,
        l::BaselineName::Alphabetic,
        l::PhysicalAxis::Vertical,
        largest.to_f64(),
    );
    control.apply_to(&mut host).unwrap();
    let accepted = semantic(&control);
    let counts = (host.prepares.get(), host.commits);
    host.change(1, |input| {
        input.margin.bottom = l::LengthAutoOf::value(l::LengthPercentageOf::px(largest).unwrap())
    });
    let error = host.compute(mode, &[1]).unwrap_err();
    assert!(
        matches!(error.kind(), l::LayoutErrorKindOf::InvalidInput(l::LayoutInvalidInputOf::InvalidNumeric { value }) if !value.to_f64().is_finite()),
        "{error:?}"
    );
    assert_eq!((host.prepares.get(), host.commits), counts);
    assert_eq!(semantic(host.published.batch.as_ref().unwrap()), accepted);
    assert_eq!(semantic(&control), accepted);
}
#[test]
fn finite_maximum_control_precedes_generated_named_overflow_refusal_f32() {
    overflowing_synthesis::<f32>(f32::MAX);
}
#[test]
fn finite_maximum_control_precedes_generated_named_overflow_refusal_f64() {
    overflowing_synthesis::<f64>(f64::MAX);
}

fn fractional_references<S: l::LayoutScalar>() {
    for mode in [l::WritingMode::HorizontalTb, l::WritingMode::VerticalLr] {
        let context = l::InlineBaselineContext::Named(l::BaselineName::Central);
        let mut host = Host::<S>::new(root(mode, l::Direction::Ltr, context));
        host.push(
            Some(0),
            l::LayoutInputOf::box_input(atomic(mode, l::Direction::Ltr, 30.0, (0.0, 0.0))),
        );
        wrapper(&mut host, mode, l::Direction::Ltr, context, context);
        host.change(2, |input| {
            if mode == l::WritingMode::HorizontalTb {
                input.padding.top = l::LengthOf::value(l::LengthPercentageOf::px(n(2.5)).unwrap());
            } else {
                input.padding.left = l::LengthOf::value(l::LengthPercentageOf::px(n(6.5)).unwrap());
            }
        });
        let batch = host.compute(mode, &[]).unwrap();
        let (
            axis,
            raw_wrapper_origin,
            raw_block_origin,
            raw_offset,
            final_wrapper_origin,
            final_block_origin,
            final_offset,
        ) = if mode == l::WritingMode::HorizontalTb {
            (l::PhysicalAxis::Vertical, 7.5, 2.5, 7.5, 8.0, 3.0, 7.0)
        } else {
            (l::PhysicalAxis::Horizontal, 3.5, 6.5, 11.5, 4.0, 7.0, 11.0)
        };
        let coordinate = |point: l::Point<S>| {
            if mode == l::WritingMode::HorizontalTb {
                point.y
            } else {
                point.x
            }
        };
        let raw = batch.unrounded_entries();
        assert_eq!(coordinate(node(raw, 2).location), n(raw_wrapper_origin));
        assert_eq!(coordinate(node(raw, 3).location), n(raw_block_origin));
        assert_eq!(node(raw, 2).size, physical_size(mode, 20.0, 18.5));
        named(
            node(raw, 0).named_baselines,
            l::BaselineName::Central,
            axis,
            15.0,
        );
        named(
            node(raw, 2).named_baselines,
            l::BaselineName::Central,
            axis,
            raw_offset,
        );
        named(
            node(raw, 3).named_baselines,
            l::BaselineName::Central,
            axis,
            5.0,
        );
        let final_rows = batch.final_entries();
        assert_eq!(
            coordinate(node(final_rows, 2).location),
            n(final_wrapper_origin)
        );
        assert_eq!(
            coordinate(node(final_rows, 3).location),
            n(final_block_origin)
        );
        named(
            node(final_rows, 0).named_baselines,
            l::BaselineName::Central,
            axis,
            15.0,
        );
        named(
            node(final_rows, 2).named_baselines,
            l::BaselineName::Central,
            axis,
            final_offset,
        );
        named(
            node(final_rows, 3).named_baselines,
            l::BaselineName::Central,
            axis,
            4.0,
        );
        // Both published references reconstruct one once-snapped global15,
        // using actual final border origins rather than raw cumulative origins.
        assert_eq!(
            coordinate(node(final_rows, 2).location) + n::<S>(final_offset),
            n(15.0)
        );
        assert_eq!(
            coordinate(node(final_rows, 2).location)
                + coordinate(node(final_rows, 3).location)
                + n::<S>(4.0),
            n(15.0)
        );
    }
}
#[test]
fn fractional_nested_named_references_use_actual_final_border_origins_f32() {
    fractional_references::<f32>();
}
#[test]
fn fractional_nested_named_references_use_actual_final_border_origins_f64() {
    fractional_references::<f64>();
}

fn actual_text_unknown<S: l::LayoutScalar>() {
    const AHEM: &[u8] = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../surgeist-text/tests/fixtures/fonts/ahem/Ahem.ttf"
    ));
    let style = t::Style {
        font: t::Font::new().family("Ahem"),
        size: 10.0,
        line_height: t::LineHeight::Absolute(10.0),
        white_space: t::WhiteSpace::Preserve,
        wrap: t::Wrap::Word,
        ..t::Style::default()
    };
    let source = t::Source::identified("A", t::Id::from_u64(1489001), 4);
    use t::SourceProcessingEvent as E;
    let capture = Arc::new(
        t::process_contextual_source(t::ContextualSourceInput {
            joint_flows: Vec::new(),
            scope_alignment_limits: None,
            source: source.clone(),
            segment_breaks: t::DocumentSegmentBreaks::LineFeedDefault,
            envelope: t::SourceProcessingEnvelope::Complete,
            origins: vec![t::ParagraphOrigin {
                text_node: None,
                source: source.clone(),
                local: t::Range::new(0, 1),
                aggregate: t::Range::new(0, 1),
                kind: t::ParagraphOriginKind::Authored,
            }],
            controls: vec![],
            connections: vec![],
            boundary_policies: vec![],
            replacements: vec![],
            computed_line_boundaries: vec![],
            scopes: vec![t::SourceProcessingScope::Block {
                writing_axis: None,
                horizontal_inner_direction: None,
                font_reference: None,
                topology: t::SourceProcessingTopology::Complete,
                trim: Default::default(),
                base_style: style.clone(),
                used_direction: t::ParagraphBaseDirection::LeftToRight,
            }],
            paragraphs: vec![t::SourceProcessingParagraph {
                base_direction: t::ParagraphBaseDirection::LeftToRight,
                paragraph_strut: t::ParagraphStrut::DefaultFont,
            }],
            events: vec![
                E::EnterScope(0),
                E::EnterParagraph(0),
                E::Origin {
                    origin: 0,
                    affinity: t::Affinity::After,
                },
                E::ExitParagraph(0),
                E::ExitScope(0),
            ],
        })
        .unwrap(),
    );
    let analyzed = t::AnalyzedSource::from_processed_paragraph(
        capture.paragraphs().next().unwrap(),
        t::AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    let wrapping = analyzed
        .wrapping_relations()
        .filter(|r| r.ancestor_wrap().is_none())
        .map(|r| t::WrappingPolicy {
            relation: r.handle(),
            ancestor_wrap: t::Wrap::Word,
        })
        .collect();
    let mut system = t::System::new(t::SystemOptions {
        font_catalogue: t::FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(AHEM.to_vec()).unwrap();
    let prepared = system
        .prepare(
            &analyzed.with_wrapping_policies(wrapping).unwrap(),
            t::PreparationOptions::default(),
        )
        .unwrap();
    let origin = prepared.origin_views().next().unwrap().handle();
    let mut host = Host::<S>::new(root(
        l::WritingMode::HorizontalTb,
        l::Direction::Ltr,
        l::InlineBaselineContext::Physical,
    ));
    assert_eq!(
        host.push(
            Some(0),
            l::LayoutInputOf::inline_text(l::TextParticipation::new(origin.clone()))
        ),
        1
    );
    let binding = l::TextParagraphBinding {
        prepared: prepared.clone(),
        container: 0,
        nodes: vec![1],
        origins: vec![l::TextOriginBinding { origin, node: 1 }],
        controls: vec![],
        atomics: vec![],
        line_strut: None,
    };
    let physical = {
        let flow = l::LayoutFlowInput::try_new(
            &host,
            0,
            vec![binding.clone()],
            l::WrappingInput::default(),
            Vec::new(),
        )
        .unwrap();
        l::compute_layout_with_flow(&flow, 0, request(l::WritingMode::HorizontalTb)).unwrap()
    };
    assert_eq!(physical.placed_text().len(), 1);
    let text = physical.placed_text()[0].layout();
    assert_eq!(text.source().text().code_units(), [65]);
    assert_eq!(text.source(), prepared.analyzed_source().source());
    let original = prepared.origin_views().next().unwrap();
    assert_eq!(original.source().id(), source.id());
    assert_eq!(original.source().revision(), 4);
    assert_eq!(
        text.run_views()
            .flat_map(|r| r.glyphs().map(|g| g.id()).collect::<Vec<_>>())
            .collect::<Vec<_>>(),
        [35]
    );
    assert_eq!(
        node(physical.unrounded_entries(), 0).size,
        l::Size::new(n(100.0), n(10.0))
    );
    for name in [l::BaselineName::Alphabetic, l::BaselineName::Central] {
        assert_eq!(
            node(physical.unrounded_entries(), 0)
                .named_baselines
                .get(name)
                .first(),
            l::BaselineFactOf::Unknown
        );
        host.change(0, |input| {
            input.inline_baseline_context = l::InlineBaselineContext::Named(name)
        });
        let flow = l::LayoutFlowInput::try_new(
            &host,
            0,
            vec![binding.clone()],
            l::WrappingInput::default(),
            Vec::new(),
        )
        .unwrap();
        missing(l::compute_layout_with_flow_invalidated(
            &flow,
            0,
            request(l::WritingMode::HorizontalTb),
            &[0],
        ));
    }
    assert_eq!(text.source().text().code_units(), [65]);
    assert_eq!(text.lines()[0].baseline(), 8.0);
}
#[test]
fn actual_registered_text_is_valid_physical_but_not_named_origin_authority_f32() {
    actual_text_unknown::<f32>();
}
#[test]
fn actual_registered_text_is_valid_physical_but_not_named_origin_authority_f64() {
    actual_text_unknown::<f64>();
}

fn inline_block_default_last<S: l::LayoutScalar>() {
    let mode = l::WritingMode::HorizontalTb;
    let direction = l::Direction::Ltr;
    let context = l::InlineBaselineContext::Named(l::BaselineName::Alphabetic);
    let mut host = Host::<S>::new(root(mode, direction, context));
    assert_eq!(
        host.push(
            Some(0),
            l::LayoutInputOf::box_input(atomic(mode, direction, 30.0, (0.0, 0.0)))
        ),
        1
    );
    let mut outer = atomic(mode, direction, 0.0, (0.0, 0.0));
    outer.size.height = l::PreferredSizeOf::AUTO;
    outer.inline_baseline_context = context;
    assert_eq!(host.push(Some(0), l::LayoutInputOf::box_input(outer)), 2);
    let mut first = atomic(mode, direction, 10.0, (0.0, 0.0));
    first.size.width = px(10.0);
    first.atomic_inline_participation = Some(l::AtomicInlineParticipation::new(
        t::BidiLevel::try_new(0).unwrap(),
        l::InlineBreakOpportunity::Mandatory,
    ));
    assert_eq!(host.push(Some(2), l::LayoutInputOf::box_input(first)), 3);
    let mut last = atomic(mode, direction, 20.0, (0.0, 0.0));
    last.size.width = px(10.0);
    assert_eq!(host.push(Some(2), l::LayoutInputOf::box_input(last)), 4);
    let batch = host.compute(mode, &[]).unwrap();

    // Qualify actual distinct child facts before the outer default-selection
    // assertion. Mandatory-after-box carries authentic NamedAlpha10/0; the
    // second real20/0 line dominates it, producing extent30 and last30.
    for entries in [batch.unrounded_entries(), batch.final_entries()] {
        assert_eq!(node(entries, 2).size, l::Size::new(n(20.0), n(30.0)));
        assert_eq!(node(entries, 3).location, l::Point::ZERO);
        assert_eq!(node(entries, 3).size, l::Size::new(n(10.0), n(10.0)));
        assert_eq!(node(entries, 4).location, l::Point::new(S::ZERO, n(10.0)));
        assert_eq!(node(entries, 4).size, l::Size::new(n(10.0), n(20.0)));
        let facts = node(entries, 2)
            .named_baselines
            .get(l::BaselineName::Alphabetic);
        for (fact, expected) in [(facts.first(), 10.0), (facts.last(), 30.0)] {
            match fact {
                l::BaselineFactOf::Available(coordinate) => {
                    assert_eq!(coordinate.axis(), l::PhysicalAxis::Vertical);
                    assert_eq!(coordinate.offset(), n(expected));
                }
                other => panic!("actual two-line child must provide named facts, got {other:?}"),
            }
        }
    }
    // CSS Inline3 4.2.1 initial auto prefers the last baseline of an
    // InlineBlock. Peer30 and actual wrapper-last30 share one30-unit line.
    assert_eq!(
        node(batch.unrounded_entries(), 0).size,
        l::Size::new(n(100.0), n(30.0)),
        "InlineBlock default must select its actual last named baseline"
    );
    for entries in [batch.unrounded_entries(), batch.final_entries()] {
        assert_eq!(node(entries, 2).location, l::Point::new(n(20.0), S::ZERO));
        assert_eq!(node(entries, 1).location, l::Point::ZERO);
        assert_eq!(node(entries, 1).size, l::Size::new(n(20.0), n(30.0)));
        assert_eq!(node(entries, 0).size, l::Size::new(n(100.0), n(30.0)));
        named(
            node(entries, 0).named_baselines,
            l::BaselineName::Alphabetic,
            l::PhysicalAxis::Vertical,
            30.0,
        );
        assert_eq!(entries.len(), 5);
        for (id, source) in [(0, 0), (1, 0), (2, 1), (3, 0), (4, 1)] {
            assert_eq!(node(entries, id).source_index, l::SourceIndex::new(source));
        }
        for id in [1, 3, 4] {
            absent(node(entries, id).named_baselines);
        }
    }
    assert!(batch.placed_text().is_empty());
    assert!(batch.final_inline_fragments().is_empty());
}

#[test]
fn inline_block_default_auto_uses_last_named_baseline_f32() {
    inline_block_default_last::<f32>();
}

#[test]
fn inline_block_default_auto_uses_last_named_baseline_f64() {
    inline_block_default_last::<f64>();
}

fn source_fact<S: l::LayoutScalar>(fact: l::BaselineFactOf<S>, axis: l::PhysicalAxis, value: f64) {
    match fact {
        l::BaselineFactOf::Available(coordinate) => {
            assert_eq!(coordinate.axis(), axis);
            assert_eq!(coordinate.offset(), n(value));
        }
        other => panic!("expected actual named coordinate {value}, got {other:?}"),
    }
}
fn source_block<S: l::LayoutScalar>(
    name: l::BaselineName,
    height: Option<f64>,
) -> l::NodeInputOf<S> {
    let mut input = root(
        l::WritingMode::HorizontalTb,
        l::Direction::Ltr,
        l::InlineBaselineContext::Named(name),
    );
    input.size.width = px(20.0);
    input.size.height = height.map_or(l::PreferredSizeOf::AUTO, px);
    input
}
fn source_line<S: l::LayoutScalar>(
    host: &mut Host<S>,
    parent: u32,
    name: l::BaselineName,
    height: f64,
) -> u32 {
    let line = host.push(
        Some(parent),
        l::LayoutInputOf::box_input(source_block(name, None)),
    );
    host.push(
        Some(line),
        l::LayoutInputOf::box_input(atomic(
            l::WritingMode::HorizontalTb,
            l::Direction::Ltr,
            height,
            (0.0, 0.0),
        )),
    );
    line
}
fn source_parent<S: l::LayoutScalar>(
    name: l::BaselineName,
    peer: f64,
    source: l::BaselineSource,
    overflow: l::ComputedOverflow,
    margin: f64,
) -> Host<S> {
    let mode = l::WritingMode::HorizontalTb;
    let context = l::InlineBaselineContext::Named(name);
    let mut host = Host::new(root(mode, l::Direction::Ltr, context));
    assert_eq!(
        host.push(
            Some(0),
            l::LayoutInputOf::box_input(atomic(mode, l::Direction::Ltr, peer, (0.0, 0.0)))
        ),
        1
    );
    let mut wrapper = atomic(mode, l::Direction::Ltr, 40.0, (0.0, margin));
    wrapper.inline_baseline_context = context;
    wrapper.baseline_source = source;
    wrapper.overflow = overflow;
    assert_eq!(host.push(Some(0), l::LayoutInputOf::box_input(wrapper)), 2);
    host
}
fn source_content<S: l::LayoutScalar>(
    host: &mut Host<S>,
    name: l::BaselineName,
    spacer: f64,
    last: f64,
) {
    source_line(host, 2, name, 10.0);
    host.push(
        Some(2),
        l::LayoutInputOf::box_input(source_block(name, Some(spacer))),
    );
    source_line(host, 2, name, last);
}
fn source_rectangles<S: l::LayoutScalar>(
    batch: &l::CompletedLayoutBatchOf<u32, S>,
    height: f64,
    peer: f64,
    wrapper: f64,
) {
    for entries in [batch.unrounded_entries(), batch.final_entries()] {
        assert_eq!(
            node(entries, 0).size,
            l::Size::new(n(100.0), n(height)),
            "expected root height{height}, peer y{peer}, wrapper y{wrapper}"
        );
        assert_eq!(node(entries, 1).location, l::Point::new(S::ZERO, n(peer)));
        assert_eq!(
            node(entries, 2).location,
            l::Point::new(n(20.0), n(wrapper))
        );
        assert_eq!(node(entries, 2).size, l::Size::new(n(20.0), n(40.0)));
        for id in 0..u32::try_from(entries.len()).unwrap() {
            node(entries, id);
        }
        assert_eq!(node(entries, 1).source_index, l::SourceIndex::new(0));
        assert_eq!(node(entries, 2).source_index, l::SourceIndex::new(1));
    }
    assert!(batch.placed_text().is_empty());
    assert!(batch.final_inline_fragments().is_empty());
}

fn source_override_cache<S: l::LayoutScalar>() {
    let mode = l::WritingMode::HorizontalTb;
    let name = l::BaselineName::Alphabetic;
    let visible = l::ComputedOverflow::try_new(l::Overflow::Visible, l::Overflow::Visible).unwrap();
    let fixture = |source| {
        let mut host = source_parent::<S>(name, 30.0, source, visible, 0.0);
        host.change(2, |input| input.size.height = l::PreferredSizeOf::AUTO);
        let mut first = atomic(mode, l::Direction::Ltr, 10.0, (0.0, 0.0));
        first.size.width = px(10.0);
        first.atomic_inline_participation = Some(l::AtomicInlineParticipation::new(
            t::BidiLevel::try_new(0).unwrap(),
            l::InlineBreakOpportunity::Mandatory,
        ));
        host.push(Some(2), l::LayoutInputOf::box_input(first));
        let mut last = atomic(mode, l::Direction::Ltr, 20.0, (0.0, 0.0));
        last.size.width = px(10.0);
        host.push(Some(2), l::LayoutInputOf::box_input(last));
        host
    };
    assert_eq!(l::BaselineSource::default(), l::BaselineSource::Auto);
    let mut host = fixture(l::BaselineSource::Auto);
    let initial = host.compute(mode, &[]).unwrap();
    let initial_semantics = semantic(&initial);
    initial.apply_to(&mut host).unwrap();
    let old_hits: usize = host.hits.iter().map(Cell::get).sum();
    let warm = host.compute(mode, &[]).unwrap();
    assert_eq!(semantic(&warm), initial_semantics);
    assert!(host.hits.iter().map(Cell::get).sum::<usize>() > old_hits);
    warm.apply_to(&mut host).unwrap();
    for (source, height, y) in [
        (l::BaselineSource::First, 50.0, 20.0),
        (l::BaselineSource::Last, 30.0, 0.0),
        (l::BaselineSource::Auto, 30.0, 0.0),
    ] {
        host.change(2, |input| input.baseline_source = source);
        let changed = host.compute(mode, &[2]).unwrap();
        let fresh = fixture(source).compute(mode, &[]).unwrap();
        assert_eq!(semantic(&changed), semantic(&fresh));
        for entries in [changed.unrounded_entries(), changed.final_entries()] {
            assert_eq!(node(entries, 0).size, l::Size::new(n(100.0), n(height)));
            assert_eq!(node(entries, 2).location, l::Point::new(n(20.0), n(y)));
            let pair = node(entries, 2).named_baselines.get(name);
            source_fact(pair.first(), l::PhysicalAxis::Vertical, 10.0);
            source_fact(pair.last(), l::PhysicalAxis::Vertical, 30.0);
        }
        changed.apply_to(&mut host).unwrap();
        // This declaration is not inherited: changing ONLY the parent must
        // preserve the real child's chosen source and intrinsic10/30 table.
        for parent_source in [l::BaselineSource::First, l::BaselineSource::Last] {
            host.change(0, |input| input.baseline_source = parent_source);
            let parent_changed = host.compute(mode, &[0]).unwrap();
            assert_eq!(semantic(&parent_changed), semantic(&changed));
            parent_changed.apply_to(&mut host).unwrap();
        }
        assert_eq!(semantic(&initial), initial_semantics);
    }
}
#[test]
fn named_source_overrides_noninheritance_and_real_cache_f32() {
    source_override_cache::<f32>();
}
#[test]
fn named_source_overrides_noninheritance_and_real_cache_f64() {
    source_override_cache::<f64>();
}

fn source_unknown_selection<S: l::LayoutScalar>() {
    let mode = l::WritingMode::HorizontalTb;
    let name = l::BaselineName::Alphabetic;
    let visible = l::ComputedOverflow::try_new(l::Overflow::Visible, l::Overflow::Visible).unwrap();
    for unknown_first in [true, false] {
        for source in [
            l::BaselineSource::Auto,
            l::BaselineSource::First,
            l::BaselineSource::Last,
        ] {
            let mut host = source_parent::<S>(name, 30.0, source, visible, 0.0);
            host.change(2, |input| input.size.height = l::PreferredSizeOf::AUTO);
            let top_only = |host: &mut Host<S>| {
                let child = host.push(
                    Some(2),
                    l::LayoutInputOf::box_input(source_block(name, None)),
                );
                host.push(
                    Some(child),
                    l::LayoutInputOf::line_break(
                        l::LineBreakInputOf::new()
                            .with_metrics(
                                l::InlineMetricsOf::try_new(
                                    l::InlineBaselineContext::Physical,
                                    n(8.0),
                                    n(10.0),
                                )
                                .unwrap(),
                            )
                            .with_vertical_align(l::VerticalAlignOf::Top),
                    ),
                );
            };
            if unknown_first {
                top_only(&mut host);
                source_line(&mut host, 2, name, 6.0);
            } else {
                source_line(&mut host, 2, name, 6.0);
                top_only(&mut host);
            }
            let chooses_first = source == l::BaselineSource::First;
            if chooses_first == unknown_first {
                // Publish the genuine available opposite set first. A refused
                // source change may not replace that completed packet or caches.
                host.change(2, |input| {
                    input.baseline_source = if unknown_first {
                        l::BaselineSource::Last
                    } else {
                        l::BaselineSource::First
                    };
                });
                let prior = host.compute(mode, &[]).unwrap();
                let prior_semantics = semantic(&prior);
                prior.apply_to(&mut host).unwrap();
                let counts = (host.prepares.get(), host.commits);
                host.change(2, |input| input.baseline_source = source);
                missing(host.compute(mode, &[2]));
                assert_eq!((host.prepares.get(), host.commits), counts);
                assert_eq!(
                    semantic(host.published.batch.as_ref().unwrap()),
                    prior_semantics
                );
                assert_eq!(semantic(&prior), prior_semantics);
            } else {
                let batch = host.compute(mode, &[]).unwrap();
                for entries in [batch.unrounded_entries(), batch.final_entries()] {
                    assert_eq!(node(entries, 2).size.height, n(16.0));
                    let pair = node(entries, 2).named_baselines.get(name);
                    if unknown_first {
                        assert_eq!(pair.first(), l::BaselineFactOf::Unknown);
                        source_fact(pair.last(), l::PhysicalAxis::Vertical, 16.0);
                        assert_eq!(node(entries, 0).size.height, n(30.0));
                        assert_eq!(node(entries, 2).location.y, n(14.0));
                    } else {
                        source_fact(pair.first(), l::PhysicalAxis::Vertical, 6.0);
                        assert_eq!(pair.last(), l::BaselineFactOf::Unknown);
                        assert_eq!(node(entries, 0).size.height, n(40.0));
                        assert_eq!(node(entries, 2).location.y, n(24.0));
                    }
                }
            }
        }
    }
}
#[test]
fn named_source_selected_unknown_never_borrows_opposite_fact_f32() {
    source_unknown_selection::<f32>();
}
#[test]
fn named_source_selected_unknown_never_borrows_opposite_fact_f64() {
    source_unknown_selection::<f64>();
}

fn source_absent_and_empty_scroll<S: l::LayoutScalar>() {
    let mode = l::WritingMode::HorizontalTb;
    let name = l::BaselineName::Central;
    let visible = l::ComputedOverflow::try_new(l::Overflow::Visible, l::Overflow::Visible).unwrap();
    let hidden = l::ComputedOverflow::try_new(l::Overflow::Hidden, l::Overflow::Hidden).unwrap();
    for child_overflow in [l::Overflow::Hidden, l::Overflow::Scroll, l::Overflow::Auto] {
        for (source, height, peer) in [
            (l::BaselineSource::First, 40.0, 10.0),
            (l::BaselineSource::Last, 50.0, 30.0),
            (l::BaselineSource::Auto, 50.0, 30.0),
        ] {
            let mut host = source_parent::<S>(name, 20.0, source, visible, 0.0);
            let mut child = source_block(name, Some(40.0));
            child.overflow = l::ComputedOverflow::try_new(child_overflow, child_overflow).unwrap();
            host.push(Some(2), l::LayoutInputOf::box_input(child));
            let batch = host.compute(mode, &[]).unwrap();
            source_rectangles(&batch, height, peer, 0.0);
            for entries in [batch.unrounded_entries(), batch.final_entries()] {
                for id in [2, 3] {
                    let pair = node(entries, id).named_baselines.get(name);
                    assert_eq!(pair.first(), l::BaselineFactOf::Absent);
                    source_fact(pair.last(), l::PhysicalAxis::Vertical, 40.0);
                }
            }
        }
    }
    // Explicit source disables the exceptional Auto set. Its selected genuine
    // Absent must synthesize from the margin box, THEN clamp that synthesized
    // result for a scroll container; merely clamping Available exports misses it.
    for source in [l::BaselineSource::First, l::BaselineSource::Last] {
        for (name, margin, height, y, coordinate) in [
            (l::BaselineName::Alphabetic, 8.0, 68.0, 20.0, 40.0),
            (l::BaselineName::Alphabetic, -6.0, 60.0, 26.0, 34.0),
            (l::BaselineName::Central, 8.0, 60.0, 6.0, 24.0),
            (l::BaselineName::Central, -6.0, 60.0, 13.0, 17.0),
        ] {
            let host = source_parent::<S>(name, 60.0, source, hidden, margin);
            let batch = host.compute(mode, &[]).unwrap();
            source_rectangles(&batch, height, 0.0, y);
            for entries in [batch.unrounded_entries(), batch.final_entries()] {
                absent(node(entries, 2).named_baselines);
                // The actual parent anchor proves the used clamped coordinate
                // without replacing the child's genuine Absent facts.
                let parent = if name == l::BaselineName::Alphabetic {
                    60.0
                } else {
                    30.0
                };
                named(
                    node(entries, 0).named_baselines,
                    name,
                    l::PhysicalAxis::Vertical,
                    parent,
                );
                assert_eq!(node(entries, 2).location.y + n::<S>(coordinate), n(parent));
            }
        }
    }
}
#[test]
fn named_source_absent_slot_and_empty_scroll_fallback_clamp_f32() {
    source_absent_and_empty_scroll::<f32>();
}
#[test]
fn named_source_absent_slot_and_empty_scroll_fallback_clamp_f64() {
    source_absent_and_empty_scroll::<f64>();
}

fn source_scroll_content<S: l::LayoutScalar>() {
    let mode = l::WritingMode::HorizontalTb;
    let hidden = l::ComputedOverflow::try_new(l::Overflow::Hidden, l::Overflow::Hidden).unwrap();
    for (name, source, margin, height, peer, y, last) in [
        (
            l::BaselineName::Alphabetic,
            l::BaselineSource::Auto,
            8.0,
            68.0,
            0.0,
            20.0,
            40.0,
        ),
        (
            l::BaselineName::Alphabetic,
            l::BaselineSource::Auto,
            -6.0,
            60.0,
            0.0,
            26.0,
            34.0,
        ),
        (
            l::BaselineName::Alphabetic,
            l::BaselineSource::Auto,
            -50.0,
            60.0,
            0.0,
            60.0,
            0.0,
        ),
        (
            l::BaselineName::Central,
            l::BaselineSource::Auto,
            8.0,
            70.0,
            10.0,
            0.0,
            40.0,
        ),
        (
            l::BaselineName::Central,
            l::BaselineSource::Auto,
            -6.0,
            64.0,
            4.0,
            0.0,
            34.0,
        ),
        (
            l::BaselineName::Central,
            l::BaselineSource::Auto,
            -50.0,
            60.0,
            0.0,
            30.0,
            0.0,
        ),
        (
            l::BaselineName::Alphabetic,
            l::BaselineSource::First,
            0.0,
            90.0,
            0.0,
            50.0,
            24.0,
        ),
        (
            l::BaselineName::Alphabetic,
            l::BaselineSource::Last,
            0.0,
            76.0,
            0.0,
            36.0,
            24.0,
        ),
        (
            l::BaselineName::Central,
            l::BaselineSource::First,
            0.0,
            65.0,
            0.0,
            25.0,
            19.0,
        ),
        (
            l::BaselineName::Central,
            l::BaselineSource::Last,
            0.0,
            60.0,
            0.0,
            11.0,
            19.0,
        ),
    ] {
        let mut host = source_parent::<S>(name, 60.0, source, hidden, margin);
        source_content(&mut host, name, 4.0, 10.0);
        let batch = host.compute(mode, &[]).unwrap();
        source_rectangles(&batch, height, peer, y);
        for entries in [batch.unrounded_entries(), batch.final_entries()] {
            assert_eq!(node(entries, 3).location.y, S::ZERO);
            assert_eq!(node(entries, 5).location.y, n(10.0));
            assert_eq!(node(entries, 6).location.y, n(14.0));
            let pair = node(entries, 2).named_baselines.get(name);
            source_fact(
                pair.first(),
                l::PhysicalAxis::Vertical,
                if name == l::BaselineName::Alphabetic {
                    10.0
                } else {
                    5.0
                },
            );
            source_fact(pair.last(), l::PhysicalAxis::Vertical, last);
            if source == l::BaselineSource::Auto {
                for exceptional_name in [l::BaselineName::Alphabetic, l::BaselineName::Central] {
                    source_fact(
                        node(entries, 2)
                            .named_baselines
                            .get(exceptional_name)
                            .last(),
                        l::PhysicalAxis::Vertical,
                        last,
                    );
                }
            }
        }
    }
}
#[test]
fn named_source_actual_scroll_sets_and_signed_margins_f32() {
    source_scroll_content::<f32>();
}
#[test]
fn named_source_actual_scroll_sets_and_signed_margins_f64() {
    source_scroll_content::<f64>();
}

fn source_clip_and_nested<S: l::LayoutScalar>() {
    let mode = l::WritingMode::HorizontalTb;
    let hidden = l::ComputedOverflow::try_new(l::Overflow::Hidden, l::Overflow::Hidden).unwrap();
    for overflow in [
        l::ComputedOverflow::try_new(l::Overflow::Clip, l::Overflow::Clip).unwrap(),
        l::ComputedOverflow::try_new(l::Overflow::Visible, l::Overflow::Clip).unwrap(),
        hidden,
    ] {
        let scroll = overflow == hidden;
        for (name, height, peer, y, last) in [
            (
                l::BaselineName::Alphabetic,
                60.0,
                0.0,
                if scroll { 20.0 } else { 5.0 },
                if scroll { 40.0 } else { 55.0 },
            ),
            (
                l::BaselineName::Central,
                if scroll { 70.0 } else { 75.0 },
                if scroll { 10.0 } else { 15.0 },
                0.0,
                if scroll { 40.0 } else { 45.0 },
            ),
        ] {
            for source in [l::BaselineSource::Last, l::BaselineSource::Auto] {
                let mut host = source_parent::<S>(name, 60.0, source, overflow, 0.0);
                source_content(&mut host, name, 25.0, 20.0);
                let batch = host.compute(mode, &[]).unwrap();
                source_rectangles(&batch, height, peer, y);
                for entries in [batch.unrounded_entries(), batch.final_entries()] {
                    source_fact(
                        node(entries, 2).named_baselines.get(name).last(),
                        l::PhysicalAxis::Vertical,
                        last,
                    );
                    assert_eq!(node(entries, 6).location.y, n(35.0));
                }
            }
        }
    }
    // Only Alphabetic exceeds40: Central35 must not be replaced by40.
    for (name, last) in [
        (l::BaselineName::Alphabetic, 40.0),
        (l::BaselineName::Central, 35.0),
    ] {
        let mut host = source_parent::<S>(name, 60.0, l::BaselineSource::Last, hidden, 0.0);
        source_content(&mut host, name, 15.0, 20.0);
        let batch = host.compute(mode, &[]).unwrap();
        for entries in [batch.unrounded_entries(), batch.final_entries()] {
            source_fact(
                node(entries, 2).named_baselines.get(name).last(),
                l::PhysicalAxis::Vertical,
                last,
            );
        }
    }
    for name in [l::BaselineName::Alphabetic, l::BaselineName::Central] {
        for outer_scroll in [false, true] {
            let mut host = Host::<S>::new(source_block(name, None));
            host.change(0, |input| {
                input.size.width = px(100.0);
                input.size.height = px(if outer_scroll { 45.0 } else { 50.0 });
                input.baseline_source = l::BaselineSource::Last;
                if outer_scroll {
                    input.overflow = hidden;
                }
            });
            host.push(
                Some(0),
                l::LayoutInputOf::box_input(source_block(name, Some(10.0))),
            );
            let mut child = source_block(name, Some(40.0));
            child.overflow = hidden;
            child.baseline_source = l::BaselineSource::Last;
            assert_eq!(host.push(Some(0), l::LayoutInputOf::box_input(child)), 2);
            source_content(&mut host, name, 25.0, 20.0);
            let batch = host.compute(mode, &[]).unwrap();
            for entries in [batch.unrounded_entries(), batch.final_entries()] {
                assert_eq!(node(entries, 2).location.y, n(10.0));
                source_fact(
                    node(entries, 2).named_baselines.get(name).last(),
                    l::PhysicalAxis::Vertical,
                    40.0,
                );
                source_fact(
                    node(entries, 0).named_baselines.get(name).last(),
                    l::PhysicalAxis::Vertical,
                    if outer_scroll { 45.0 } else { 50.0 },
                );
            }
        }
    }
}
#[test]
fn named_source_clip_individual_clamp_and_nested_initial_frames_f32() {
    source_clip_and_nested::<f32>();
}
#[test]
fn named_source_clip_individual_clamp_and_nested_initial_frames_f64() {
    source_clip_and_nested::<f64>();
}

fn source_vertical_lr<S: l::LayoutScalar>() {
    let mode = l::WritingMode::VerticalLr;
    for (name, extent, peer_x) in [
        (l::BaselineName::Alphabetic, 100.0, 40.0),
        (l::BaselineName::Central, 70.0, 10.0),
    ] {
        let mut host = Host::<S>::new(root(
            mode,
            l::Direction::Ltr,
            l::InlineBaselineContext::Named(name),
        ));
        host.push(
            Some(0),
            l::LayoutInputOf::box_input(atomic(mode, l::Direction::Ltr, 60.0, (0.0, 0.0))),
        );
        let mut wrapper = atomic(mode, l::Direction::Ltr, 40.0, (0.0, 0.0));
        // VerticalLr block-end is physical RIGHT; alphabetic under is LEFT.
        wrapper.margin.right = l::LengthAutoOf::value(l::LengthPercentageOf::px(n(8.0)).unwrap());
        wrapper.overflow =
            l::ComputedOverflow::try_new(l::Overflow::Hidden, l::Overflow::Hidden).unwrap();
        host.push(Some(0), l::LayoutInputOf::box_input(wrapper));
        let batch = host.compute(mode, &[]).unwrap();
        for entries in [batch.unrounded_entries(), batch.final_entries()] {
            assert_eq!(node(entries, 0).size, l::Size::new(n(extent), n(100.0)));
            assert_eq!(node(entries, 1).location, l::Point::new(n(peer_x), S::ZERO));
            assert_eq!(node(entries, 2).location, l::Point::new(S::ZERO, n(20.0)));
            assert_eq!(node(entries, 2).size, l::Size::new(n(40.0), n(20.0)));
            for exceptional_name in [l::BaselineName::Alphabetic, l::BaselineName::Central] {
                let pair = node(entries, 2).named_baselines.get(exceptional_name);
                assert_eq!(pair.first(), l::BaselineFactOf::Absent);
                source_fact(pair.last(), l::PhysicalAxis::Horizontal, 40.0);
            }
        }
    }
}
#[test]
fn named_source_vertical_lr_empty_scroll_uses_block_end_for_both_names_f32() {
    source_vertical_lr::<f32>();
}
#[test]
fn named_source_vertical_lr_empty_scroll_uses_block_end_for_both_names_f64() {
    source_vertical_lr::<f64>();
}

fn packet_child<S: l::LayoutScalar>(host: &mut Host<S>, parent: u32) -> (u32, u32) {
    let child = host.push(
        Some(parent),
        l::LayoutInputOf::box_input(source_block(l::BaselineName::Alphabetic, None)),
    );
    let mut inner = atomic(
        l::WritingMode::HorizontalTb,
        l::Direction::Ltr,
        10.0,
        (0.0, 0.0),
    );
    inner.size.width = px(10.0);
    let inner = host.push(Some(child), l::LayoutInputOf::box_input(inner));
    (child, inner)
}
fn packet_producer_control<S: l::LayoutScalar>() {
    // A genuine root writer exposes the same actual native Block producer.
    // Its sole real10-high baseline-less atom synthesizes under-edge Alpha10.
    let mut host = Host::<S>::new(source_block(l::BaselineName::Alphabetic, None));
    let mut inner = atomic(
        l::WritingMode::HorizontalTb,
        l::Direction::Ltr,
        10.0,
        (0.0, 0.0),
    );
    inner.size.width = px(10.0);
    host.push(Some(0), l::LayoutInputOf::box_input(inner));
    let batch = host.compute(l::WritingMode::HorizontalTb, &[]).unwrap();
    for entries in [batch.unrounded_entries(), batch.final_entries()] {
        assert_eq!(node(entries, 0).size, l::Size::new(n(20.0), n(10.0)));
        assert_eq!(node(entries, 1).location, l::Point::ZERO);
        assert_eq!(node(entries, 1).size, l::Size::new(n(10.0), n(10.0)));
        absent(node(entries, 1).named_baselines);
        named(
            node(entries, 0).named_baselines,
            l::BaselineName::Alphabetic,
            l::PhysicalAxis::Vertical,
            10.0,
        );
    }
}
fn packet_computed_alpha<S: l::LayoutScalar>(
    batch: &l::CompletedLayoutBatchOf<u32, S>,
    child: u32,
) {
    // Undemanded size queries may leave facts Unknown. Any actual Available
    // fact still names this genuine10-high Alpha line; Absent would be false.
    // Performed packet geometry and Alpha10 are asserted independently below.
    for row in batch
        .cache_store_entries()
        .iter()
        .filter(|row| row.node() == child)
    {
        let pair = row
            .output()
            .named_baselines
            .get(l::BaselineName::Alphabetic);
        for fact in [pair.first(), pair.last()] {
            match fact {
                l::BaselineFactOf::Unknown => {}
                l::BaselineFactOf::Available(_) => {
                    source_fact(fact, l::PhysicalAxis::Vertical, 10.0)
                }
                l::BaselineFactOf::Absent => {
                    panic!("actual Alpha line is not absent; request={:?}", row.input())
                }
            }
        }
    }
}

fn packet_alpha<S: l::LayoutScalar>(batch: &l::CompletedLayoutBatchOf<u32, S>, child: u32) {
    for entries in [batch.unrounded_entries(), batch.final_entries()] {
        named(
            node(entries, child).named_baselines,
            l::BaselineName::Alphabetic,
            l::PhysicalAxis::Vertical,
            10.0,
        );
    }
}
fn packet_writer<S: l::LayoutScalar>(display: l::Display, positioned: bool, floating: bool) {
    packet_producer_control::<S>();
    let mode = l::WritingMode::HorizontalTb;
    let mut input = root(mode, l::Direction::Ltr, l::InlineBaselineContext::Physical);
    input.display = display;
    input.align_items = Some(l::AlignItems::Start);
    if positioned {
        input.size.height = px(40.0);
    }
    let mut host = Host::<S>::new(input);
    let (child, inner) = packet_child(&mut host, 0);
    host.change(child, |input| {
        if positioned {
            input.position = l::Position::Absolute;
            input.inset.left = l::LengthAutoOf::value(l::LengthPercentageOf::px(n(9.0)).unwrap());
            input.inset.top = l::LengthAutoOf::value(l::LengthPercentageOf::px(n(7.0)).unwrap());
        }
        if floating {
            input.float = l::Float::Left;
        }
    });
    let cold = host.compute(mode, &[]).unwrap();
    // Geometry and authentic child ComputeOutput facts qualify the producer
    // BEFORE the packet's missing named-field assertion, separately per writer.
    for entries in [cold.unrounded_entries(), cold.final_entries()] {
        assert_eq!(entries.len(), 3);
        assert_eq!(
            node(entries, 0).size,
            l::Size::new(n(100.0), n(if positioned { 40.0 } else { 10.0 }))
        );
        assert_eq!(
            node(entries, child).location,
            if positioned {
                l::Point::new(n(9.0), n(7.0))
            } else {
                l::Point::ZERO
            }
        );
        assert_eq!(node(entries, child).size, l::Size::new(n(20.0), n(10.0)));
        assert_eq!(node(entries, inner).location, l::Point::ZERO);
        assert_eq!(node(entries, inner).size, l::Size::new(n(10.0), n(10.0)));
        assert_eq!(node(entries, child).source_index, l::SourceIndex::new(0));
        assert_eq!(node(entries, inner).source_index, l::SourceIndex::new(0));
        absent(node(entries, inner).named_baselines);
    }
    packet_computed_alpha(&cold, child);
    packet_alpha(&cold, child);
    assert!(cold.placed_text().is_empty());
    let prior = semantic(&cold);
    cold.apply_to(&mut host).unwrap();
    let warm = host.compute(mode, &[]).unwrap();
    assert_eq!(semantic(&warm), prior);
    assert_eq!(warm.final_entries(), cold.final_entries());
    packet_alpha(&warm, child);
    warm.apply_to(&mut host).unwrap();
    // Replace genuine baseline absence by a real Physical line inside the
    // formerly empty inner atomic. Its Unknown cannot satisfy the named child.
    let mut nested = atomic(mode, l::Direction::Ltr, 6.0, (0.0, 0.0));
    nested.size.width = px(6.0);
    host.push(Some(inner), l::LayoutInputOf::box_input(nested));
    let counts = (host.prepares.get(), host.commits);
    missing(host.compute(mode, &[inner]));
    assert_eq!((host.prepares.get(), host.commits), counts);
    assert_eq!(semantic(host.published.batch.as_ref().unwrap()), prior);
    assert_eq!(semantic(&cold), prior);
}
#[test]
fn flex_in_flow_packet_retains_actual_child_named_baselines_f32() {
    packet_writer::<f32>(l::Display::Flex, false, false);
}
#[test]
fn flex_in_flow_packet_retains_actual_child_named_baselines_f64() {
    packet_writer::<f64>(l::Display::Flex, false, false);
}
#[test]
fn flex_absolute_packet_retains_actual_child_named_baselines_f32() {
    packet_writer::<f32>(l::Display::Flex, true, false);
}
#[test]
fn flex_absolute_packet_retains_actual_child_named_baselines_f64() {
    packet_writer::<f64>(l::Display::Flex, true, false);
}
#[test]
fn block_absolute_packet_retains_actual_child_named_baselines_f32() {
    packet_writer::<f32>(l::Display::Block, true, false);
}
#[test]
fn block_absolute_packet_retains_actual_child_named_baselines_f64() {
    packet_writer::<f64>(l::Display::Block, true, false);
}
#[test]
fn block_normal_float_packet_retains_actual_child_named_baselines_f32() {
    packet_writer::<f32>(l::Display::Block, false, true);
}
#[test]
fn block_normal_float_packet_retains_actual_child_named_baselines_f64() {
    packet_writer::<f64>(l::Display::Block, false, true);
}

fn packet_grid_known_size<S: l::LayoutScalar>() {
    let mode = l::WritingMode::HorizontalTb;
    let mut host = Host::<S>::new(root(
        mode,
        l::Direction::Ltr,
        l::InlineBaselineContext::Physical,
    ));
    let mut float = source_block(l::BaselineName::Alphabetic, Some(10.0));
    float.inline_baseline_context = l::InlineBaselineContext::Physical;
    float.size.width = px(10.0);
    float.float = l::Float::Left;
    assert_eq!(host.push(Some(0), l::LayoutInputOf::box_input(float)), 1);
    let mut child = source_block(l::BaselineName::Alphabetic, Some(20.0));
    child.inline_baseline_context = l::InlineBaselineContext::Physical;
    child.block_flow_context = l::BlockFlowContext::Independent;
    assert_eq!(host.push(Some(0), l::LayoutInputOf::box_input(child)), 2);
    let grid = l::NodeInputOf {
        display: l::Display::Grid,
        size: l::Size::new(px(10.0), px(10.0)),
        ..l::NodeInputOf::default()
    };
    assert_eq!(host.push(Some(2), l::LayoutInputOf::box_input(grid)), 3);
    let batch = host.compute(mode, &[]).unwrap();
    for entries in [batch.unrounded_entries(), batch.final_entries()] {
        assert_eq!(entries.len(), 4);
        assert_eq!(node(entries, 0).size, l::Size::new(n(100.0), n(20.0)));
        assert_eq!(node(entries, 1).location, l::Point::ZERO);
        assert_eq!(node(entries, 1).size, l::Size::new(n(10.0), n(10.0)));
        assert_eq!(node(entries, 2).location, l::Point::new(n(10.0), S::ZERO));
        assert_eq!(node(entries, 2).size, l::Size::new(n(20.0), n(20.0)));
        assert_eq!(node(entries, 3).location, l::Point::ZERO);
        assert_eq!(node(entries, 3).size, l::Size::new(n(10.0), n(10.0)));
        for (id, source) in [(0, 0), (1, 0), (2, 1), (3, 0)] {
            assert_eq!(node(entries, id).source_index, l::SourceIndex::new(source));
        }
        for id in [2, 3] {
            for name in [l::BaselineName::Alphabetic, l::BaselineName::Central] {
                let pair = node(entries, id).named_baselines.get(name);
                assert_eq!(pair.first(), l::BaselineFactOf::Unknown);
                assert_eq!(pair.last(), l::BaselineFactOf::Unknown);
            }
        }
    }
    // Every actual published receipt must be truthful; no particular probe
    // count, query pattern, or private run-mode representation is required.
    for row in batch
        .cache_store_entries()
        .iter()
        .filter(|row| row.node() == 2)
    {
        assert_eq!(row.output().size, l::Size::new(n(20.0), n(20.0)));
        assert_eq!(
            row.output().named_baselines,
            node(batch.unrounded_entries(), 2).named_baselines,
            "a real compute/cache packet must retain the actual Grid descendant's Unknown authority, not affirmative absence; request={:?}",
            row.input()
        );
    }
    let prior = semantic(&batch);
    batch.apply_to(&mut host).unwrap();
    let warm = host.compute(mode, &[]).unwrap();
    assert_eq!(semantic(&warm), prior);
    assert_eq!(warm.final_entries(), batch.final_entries());
    assert_eq!(semantic(&batch), prior);
}
#[test]
fn known_size_float_avoidance_cache_preserves_real_grid_unknown_baselines_f32() {
    packet_grid_known_size::<f32>();
}
#[test]
fn known_size_float_avoidance_cache_preserves_real_grid_unknown_baselines_f64() {
    packet_grid_known_size::<f64>();
}

fn packet_text_float_binding() -> l::TextParagraphBinding<u32> {
    const AHEM: &[u8] = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../surgeist-text/tests/fixtures/fonts/ahem/Ahem.ttf"
    ));
    let style = t::Style {
        font: t::Font::new().family("Ahem"),
        size: 10.0,
        line_height: t::LineHeight::Absolute(10.0),
        white_space: t::WhiteSpace::Preserve,
        wrap: t::Wrap::Word,
        ..t::Style::default()
    };
    let source = t::Source::identified("AA", t::Id::from_u64(1489041), 1);
    let origins = [0, 1]
        .map(|index| t::ParagraphOrigin {
            text_node: None,
            source: t::Source::new("A"),
            local: t::Range::new(0, 1),
            aggregate: t::Range::new(index, index + 1),
            kind: t::ParagraphOriginKind::Authored,
        })
        .to_vec();
    use t::SourceProcessingEvent as E;
    let capture = Arc::new(
        t::process_contextual_source(t::ContextualSourceInput {
            joint_flows: Vec::new(),
            scope_alignment_limits: None,
            source,
            segment_breaks: t::DocumentSegmentBreaks::LineFeedDefault,
            envelope: t::SourceProcessingEnvelope::Complete,
            origins,
            controls: vec![],
            connections: vec![t::SourceProcessingConnection {
                before_origin: 0,
                after_origin: 1,
                kind: t::ParagraphConnectionKind::Continuous,
            }],
            boundary_policies: vec![t::ParagraphBoundaryPolicy {
                definition: t::ParagraphBoundaryDefinition::Connection(0),
                preceding_wrap: t::Wrap::Word,
                following_wrap: t::Wrap::Word,
                ancestor_wrap: t::Wrap::Word,
            }],
            replacements: vec![],
            computed_line_boundaries: vec![],
            scopes: vec![t::SourceProcessingScope::Block {
                writing_axis: None,
                horizontal_inner_direction: None,
                font_reference: None,
                topology: t::SourceProcessingTopology::Complete,
                trim: Default::default(),
                base_style: style,
                used_direction: t::ParagraphBaseDirection::LeftToRight,
            }],
            paragraphs: vec![t::SourceProcessingParagraph {
                base_direction: t::ParagraphBaseDirection::LeftToRight,
                paragraph_strut: t::ParagraphStrut::DefaultFont,
            }],
            events: vec![
                E::EnterScope(0),
                E::EnterParagraph(0),
                E::Origin {
                    origin: 0,
                    affinity: t::Affinity::After,
                },
                E::Connection(0),
                E::Origin {
                    origin: 1,
                    affinity: t::Affinity::After,
                },
                E::ExitParagraph(0),
                E::ExitScope(0),
            ],
        })
        .unwrap(),
    );
    let analyzed = t::AnalyzedSource::from_processed_paragraph(
        capture.paragraphs().next().unwrap(),
        t::AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    let wrapping = analyzed
        .wrapping_relations()
        .filter(|relation| relation.ancestor_wrap().is_none())
        .map(|relation| t::WrappingPolicy {
            relation: relation.handle(),
            ancestor_wrap: t::Wrap::Word,
        })
        .collect();
    let mut system = t::System::new(t::SystemOptions {
        font_catalogue: t::FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(AHEM.to_vec()).unwrap();
    let prepared = system
        .prepare(
            &analyzed.with_wrapping_policies(wrapping).unwrap(),
            t::PreparationOptions::default(),
        )
        .unwrap();
    let origins = prepared
        .origin_views()
        .zip([1, 3])
        .map(|(view, node)| l::TextOriginBinding {
            origin: view.handle(),
            node,
        })
        .collect();
    // The real transparent floating box belongs in the captured native window,
    // not in Text's atomic/origin tables. Its own actual child stays native.
    l::TextParagraphBinding {
        prepared,
        container: 0,
        nodes: vec![1, 2, 3],
        origins,
        controls: vec![],
        atomics: vec![],
        line_strut: None,
    }
}
fn packet_text_float<S: l::LayoutScalar>() {
    packet_producer_control::<S>();
    let mode = l::WritingMode::HorizontalTb;
    let binding = packet_text_float_binding();
    assert_eq!(binding.origins.len(), 2);
    let mut host = Host::<S>::new(root(
        mode,
        l::Direction::Ltr,
        l::InlineBaselineContext::Physical,
    ));
    assert_eq!(
        host.push(
            Some(0),
            l::LayoutInputOf::inline_text(l::TextParticipation::new(
                binding.origins[0].origin.clone()
            ))
        ),
        1
    );
    let mut float = source_block(l::BaselineName::Alphabetic, None);
    float.float = l::Float::Left;
    assert_eq!(host.push(Some(0), l::LayoutInputOf::box_input(float)), 2);
    assert_eq!(
        host.push(
            Some(0),
            l::LayoutInputOf::inline_text(l::TextParticipation::new(
                binding.origins[1].origin.clone()
            ))
        ),
        3
    );
    let mut inner = atomic(mode, l::Direction::Ltr, 10.0, (0.0, 0.0));
    inner.size.width = px(10.0);
    assert_eq!(host.push(Some(2), l::LayoutInputOf::box_input(inner)), 4);
    let compute = |host: &Host<S>, changed: &[u32]| {
        let flow = l::LayoutFlowInput::try_new(
            host,
            0,
            vec![binding.clone()],
            l::WrappingInput::default(),
            Vec::new(),
        )
        .unwrap();
        l::compute_layout_with_flow_invalidated(&flow, 0, request(mode), changed)
    };
    let cold = compute(&host, &[]).unwrap();
    assert_eq!(cold.placed_text().len(), 1);
    let text = cold.placed_text()[0].layout();
    assert_eq!(text.source(), binding.prepared.analyzed_source().source());
    assert_eq!(text.source().text().code_units(), [65, 65]);
    assert_eq!(text.lines().len(), 1);
    assert_eq!(text.lines()[0].range(), t::Range::new(0, 2));
    assert_eq!(text.lines()[0].baseline(), 8.0);
    assert_eq!(
        text.glyph_runs()
            .iter()
            .map(|run| run.advance())
            .sum::<f64>(),
        20.0
    );
    assert_eq!(
        text.run_views()
            .flat_map(|run| run.glyphs().map(|glyph| glyph.id()).collect::<Vec<_>>())
            .collect::<Vec<_>>(),
        [35, 35]
    );
    for entries in [cold.unrounded_entries(), cold.final_entries()] {
        assert_eq!(entries.len(), 5);
        assert_eq!(node(entries, 0).size, l::Size::new(n(100.0), n(10.0)));
        assert_eq!(node(entries, 2).location, l::Point::ZERO);
        assert_eq!(node(entries, 2).size, l::Size::new(n(20.0), n(10.0)));
        assert_eq!(node(entries, 4).location, l::Point::ZERO);
        assert_eq!(node(entries, 4).size, l::Size::new(n(10.0), n(10.0)));
        absent(node(entries, 4).named_baselines);
        for (id, source) in [(0, 0), (1, 0), (2, 1), (3, 2), (4, 0)] {
            assert_eq!(node(entries, id).source_index, l::SourceIndex::new(source));
        }
    }
    packet_computed_alpha(&cold, 2);
    packet_alpha(&cold, 2);
    let prior = semantic(&cold);
    let positions = text.positions().collect::<Vec<_>>();
    cold.apply_to(&mut host).unwrap();
    let warm = compute(&host, &[]).unwrap();
    assert_eq!(warm.final_entries(), cold.final_entries());
    packet_alpha(&warm, 2);
    warm.apply_to(&mut host).unwrap();
    let mut nested = atomic(mode, l::Direction::Ltr, 6.0, (0.0, 0.0));
    nested.size.width = px(6.0);
    host.push(Some(4), l::LayoutInputOf::box_input(nested));
    let counts = (host.prepares.get(), host.commits);
    missing(compute(&host, &[4]));
    assert_eq!((host.prepares.get(), host.commits), counts);
    assert_eq!(semantic(host.published.batch.as_ref().unwrap()), prior);
    assert_eq!(semantic(&cold), prior);
    assert_eq!(text.positions().collect::<Vec<_>>(), positions);
    assert_eq!(text.source().text().code_units(), [65, 65]);
}
#[test]
fn captured_text_float_packet_retains_actual_child_named_baselines_f32() {
    packet_text_float::<f32>();
}
#[test]
fn captured_text_float_packet_retains_actual_child_named_baselines_f64() {
    packet_text_float::<f64>();
}

#[derive(Clone, Copy, Debug)]
enum DemandContent {
    ZeroForced,
    Decorated,
    UnsupportedGrid,
}
fn sizing_demand_host<S: l::LayoutScalar>(
    context: l::InlineBaselineContext,
    content: DemandContent,
) -> Host<S> {
    let mode = l::WritingMode::HorizontalTb;
    let alpha = l::InlineBaselineContext::Named(l::BaselineName::Alphabetic);
    let mut outer = root(mode, l::Direction::Ltr, l::InlineBaselineContext::Physical);
    outer.display = l::Display::Flex;
    outer.align_items = Some(l::AlignItems::Start);
    let mut host = Host::<S>::new(outer);
    let mut line = source_block(l::BaselineName::Alphabetic, None);
    line.inline_baseline_context = context;
    line.size.width = px(40.0);
    assert_eq!(host.push(Some(0), l::LayoutInputOf::box_input(line)), 1);
    assert_eq!(
        host.push(
            Some(1),
            l::LayoutInputOf::box_input(atomic(mode, l::Direction::Ltr, 20.0, (0.0, 0.0)))
        ),
        2
    );
    let mut wrapper = atomic(mode, l::Direction::Ltr, 20.0, (0.0, 0.0));
    wrapper.inline_baseline_context = alpha;
    assert_eq!(host.push(Some(1), l::LayoutInputOf::box_input(wrapper)), 3);
    let metrics = l::InlineMetricsOf::try_new(alpha, S::ZERO, S::ZERO).unwrap();
    match content {
        DemandContent::ZeroForced => {
            host.push(
                Some(3),
                l::LayoutInputOf::line_break(l::LineBreakInputOf::new().with_metrics(metrics)),
            );
        }
        DemandContent::Decorated => {
            // A real ordinary Block export separates the atomic consumer from
            // this zero-height line, so demand must propagate through it.
            assert_eq!(
                host.push(
                    Some(3),
                    l::LayoutInputOf::box_input(source_block(
                        l::BaselineName::Alphabetic,
                        Some(0.0)
                    ))
                ),
                4
            );
            host.push(
                Some(4),
                l::LayoutInputOf::inline_boundary(l::InlineBoundaryInputOf::contribution(
                    l::InlineBoundaryKind::Start,
                    metrics,
                    l::InlineBoundaryEdgeOf::try_new(n(1.0), S::ZERO).unwrap(),
                )),
            );
            host.push(
                Some(4),
                l::LayoutInputOf::inline_boundary(l::InlineBoundaryInputOf::contribution(
                    l::InlineBoundaryKind::End,
                    metrics,
                    l::InlineBoundaryEdgeOf::ZERO,
                )),
            );
        }
        DemandContent::UnsupportedGrid => {
            host.push(
                Some(3),
                l::LayoutInputOf::box_input(l::NodeInputOf {
                    display: l::Display::Grid,
                    size: l::Size::new(px(10.0), px(10.0)),
                    ..l::NodeInputOf::default()
                }),
            );
        }
    }
    host
}
fn sizing_demand_facts<S: l::LayoutScalar>(
    batch: &l::CompletedLayoutBatchOf<u32, S>,
    named_context: bool,
    content: DemandContent,
) {
    for entries in [batch.unrounded_entries(), batch.final_entries()] {
        named(
            node(entries, 3).named_baselines,
            l::BaselineName::Alphabetic,
            l::PhysicalAxis::Vertical,
            0.0,
        );
        assert_eq!(node(entries, 3).size, l::Size::new(n(20.0), n(20.0)));
        assert_eq!(node(entries, 4).location, l::Point::ZERO);
        assert_eq!(
            entries.len(),
            if matches!(content, DemandContent::Decorated) {
                7
            } else {
                5
            }
        );
        for (id, source) in [(0, 0), (1, 0), (2, 0), (3, 1), (4, 0)] {
            assert_eq!(node(entries, id).source_index, l::SourceIndex::new(source));
        }
        if matches!(content, DemandContent::Decorated) {
            assert_eq!(node(entries, 4).size, l::Size::new(n(20.0), S::ZERO));
            named(
                node(entries, 4).named_baselines,
                l::BaselineName::Alphabetic,
                l::PhysicalAxis::Vertical,
                0.0,
            );
            assert_eq!(node(entries, 5).source_index, l::SourceIndex::new(0));
            assert_eq!(node(entries, 5).location, l::Point::ZERO);
            assert_eq!(node(entries, 5).size, l::Size::new(n(1.0), S::ZERO));
            assert_eq!(node(entries, 6).source_index, l::SourceIndex::new(1));
            assert_eq!(node(entries, 6).location, l::Point::new(n(1.0), S::ZERO));
            assert_eq!(node(entries, 6).size, l::Size::ZERO);
        } else {
            assert_eq!(node(entries, 4).size, l::Size::ZERO);
        }
        if named_context {
            // Peer contributes before20/after0. Actual WAlpha0 in its20-high
            // border contributes before0/after20: one40-high named line.
            assert_eq!(node(entries, 0).size, l::Size::new(n(100.0), n(40.0)));
            assert_eq!(node(entries, 1).location, l::Point::ZERO);
            assert_eq!(node(entries, 1).size, l::Size::new(n(40.0), n(40.0)));
            assert_eq!(node(entries, 2).location, l::Point::ZERO);
            assert_eq!(node(entries, 2).size, l::Size::new(n(20.0), n(20.0)));
            assert_eq!(node(entries, 3).location, l::Point::new(n(20.0), n(20.0)));
            named(
                node(entries, 1).named_baselines,
                l::BaselineName::Alphabetic,
                l::PhysicalAxis::Vertical,
                20.0,
            );
        }
    }
    // Knowledge is phase-local. Unknown is truthful for unobserved size-only
    // fields; a genuine Available must retain Alpha0. Absent is never true here.
    for row in batch.cache_store_entries().iter().filter(|row| {
        row.node() == 3 || matches!(content, DemandContent::Decorated) && row.node() == 4
    }) {
        let pair = row
            .output()
            .named_baselines
            .get(l::BaselineName::Alphabetic);
        for fact in [pair.first(), pair.last()] {
            match fact {
                l::BaselineFactOf::Unknown => {}
                l::BaselineFactOf::Available(_) => {
                    source_fact(fact, l::PhysicalAxis::Vertical, 0.0)
                }
                l::BaselineFactOf::Absent => panic!(
                    "real zero/decorated Alpha line is not absent; request={:?}",
                    row.input()
                ),
            }
        }
    }
    assert!(batch.placed_text().is_empty());
    assert!(batch.final_inline_fragments().is_empty());
}
fn sizing_unknown_keys<S: l::LayoutScalar>(
    host: &Host<S>,
    batch: &l::CompletedLayoutBatchOf<u32, S>,
) -> Vec<(l::ComputeInputOf<S>, l::CacheKeyContext)> {
    batch
        .cache_store_entries()
        .iter()
        .filter(|row| row.node() == 3)
        .filter_map(|row| {
            let output = host.published.caches[3].get_with_context(row.input(), row.context())?;
            (output
                .named_baselines
                .get(l::BaselineName::Alphabetic)
                .last()
                == l::BaselineFactOf::Unknown)
                .then_some((*row.input(), row.context()))
        })
        .fold(Vec::new(), |mut keys, key| {
            if !keys.contains(&key) {
                keys.push(key);
            }
            keys
        })
}
fn sizing_demand_cache<S: l::LayoutScalar>() {
    let mode = l::WritingMode::HorizontalTb;
    let physical = l::InlineBaselineContext::Physical;
    let alpha = l::InlineBaselineContext::Named(l::BaselineName::Alphabetic);
    for content in [DemandContent::ZeroForced, DemandContent::Decorated] {
        for starts_named in [false, true] {
            let first_context = if starts_named { alpha } else { physical };
            let mut host = sizing_demand_host::<S>(first_context, content);
            let first = host
                .compute(mode, &[])
                .unwrap_or_else(|error| panic!("{content:?} {first_context:?}: {error:?}"));
            sizing_demand_facts(&first, starts_named, content);
            let first_semantics = semantic(&first);
            first.apply_to(&mut host).unwrap();
            let warm = host.compute(mode, &[]).unwrap();
            assert_eq!(semantic(&warm), first_semantics);
            assert_eq!(warm.final_entries(), first.final_entries());
            sizing_demand_facts(&warm, starts_named, content);
            warm.apply_to(&mut host).unwrap();
            let mut unknown_keys = if starts_named {
                Vec::new()
            } else {
                sizing_unknown_keys(&host, &first)
            };
            for named_context in [!starts_named, starts_named] {
                let context = if named_context { alpha } else { physical };
                // Only this actual consumer's name changes. W and its real
                // zero/decoration producer, declared size and source stay intact.
                host.change(1, |input| input.inline_baseline_context = context);
                let changed = host
                    .compute(mode, &[1])
                    .unwrap_or_else(|error| panic!("{content:?} {context:?}: {error:?}"));
                sizing_demand_facts(&changed, named_context, content);
                let fresh = sizing_demand_host::<S>(context, content)
                    .compute(mode, &[])
                    .unwrap();
                assert_eq!(semantic(&changed), semantic(&fresh));
                assert_eq!(changed.final_entries(), fresh.final_entries());
                if named_context {
                    // Actual semantic contrast diagnostics, not a forced count
                    // or private-demand/run-mode Debug parser. Extra genuine
                    // knowledge on an undemanded query is allowed.
                    let mut available_keys = Vec::new();
                    let mut current_unknown_keys = Vec::new();
                    for row in changed
                        .cache_store_entries()
                        .iter()
                        .filter(|row| row.node() == 3)
                    {
                        let key = (*row.input(), row.context());
                        let pair = row
                            .output()
                            .named_baselines
                            .get(l::BaselineName::Alphabetic);
                        let keys = if matches!(pair.last(), l::BaselineFactOf::Available(_)) {
                            &mut available_keys
                        } else {
                            &mut current_unknown_keys
                        };
                        if !keys.contains(&key) {
                            keys.push(key);
                        }
                    }
                    eprintln!(
                        "named sizing contrast scalar={} profile={content:?} starts_named={starts_named} retained_undemanded_unknown_keys={} new_available_keys={} new_unknown_keys={}",
                        std::any::type_name::<S>(),
                        unknown_keys.len(),
                        available_keys.len(),
                        current_unknown_keys.len()
                    );
                    // Only actual Available receipts are relevant here;
                    // unrelated additional undemanded Unknown rows are allowed.
                    for key in &available_keys {
                        assert!(
                            !unknown_keys.contains(key),
                            "named-demand receipt must not alias an actually warmed size-only Unknown key"
                        );
                    }
                }
                let changed_semantics = semantic(&changed);
                changed.apply_to(&mut host).unwrap();
                let repeated = host.compute(mode, &[]).unwrap();
                assert_eq!(semantic(&repeated), changed_semantics);
                assert_eq!(repeated.final_entries(), changed.final_entries());
                sizing_demand_facts(&repeated, named_context, content);
                repeated.apply_to(&mut host).unwrap();
                if !named_context {
                    unknown_keys = sizing_unknown_keys(&host, &changed);
                }
                assert_eq!(semantic(&first), first_semantics);
                assert_eq!(semantic(&changed), changed_semantics);
            }
        }
    }
    // A genuine unsupported descendant stays Unknown at the actual consumer;
    // warming an undemanded operation must not turn it into known absence.
    let mut unsupported = sizing_demand_host::<S>(physical, DemandContent::UnsupportedGrid);
    let accepted = unsupported.compute(mode, &[]).unwrap();
    for entries in [accepted.unrounded_entries(), accepted.final_entries()] {
        let pair = node(entries, 3)
            .named_baselines
            .get(l::BaselineName::Alphabetic);
        assert_eq!(pair.first(), l::BaselineFactOf::Unknown);
        assert_eq!(pair.last(), l::BaselineFactOf::Unknown);
    }
    let accepted_semantics = semantic(&accepted);
    accepted.apply_to(&mut unsupported).unwrap();
    let counts = (unsupported.prepares.get(), unsupported.commits);
    unsupported.change(1, |input| input.inline_baseline_context = alpha);
    missing(unsupported.compute(mode, &[1]));
    missing(sizing_demand_host::<S>(alpha, DemandContent::UnsupportedGrid).compute(mode, &[]));
    assert_eq!((unsupported.prepares.get(), unsupported.commits), counts);
    assert_eq!(
        semantic(unsupported.published.batch.as_ref().unwrap()),
        accepted_semantics
    );
    assert_eq!(semantic(&accepted), accepted_semantics);
}
#[test]
fn real_named_sizing_consumer_separates_warm_undemanded_cache_f32() {
    sizing_demand_cache::<f32>();
}
#[test]
fn real_named_sizing_consumer_separates_warm_undemanded_cache_f64() {
    sizing_demand_cache::<f64>();
}
