//! Independently derived WM4 named margin-box baselines through public layout.
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
        vertical_align: l::VerticalAlign::Baseline,
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
    for alignment in [l::VerticalAlign::Top, l::VerticalAlign::Bottom] {
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
    let analyzed =
        t::AnalyzedSource::from_processed_paragraph(capture.paragraphs().next().unwrap()).unwrap();
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
