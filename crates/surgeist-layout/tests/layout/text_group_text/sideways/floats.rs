//! Real incoming floats constrain clockwise Text in physical x bands.
//!
//! Independent WM4 Table11 oracles: root40x15, Ahem10 (8/2), clockwise
//! block-start/right and inline-start/top. Floats are genuine preceding native
//! boxes outside the captured Text window, never omitted captured atomics.
use super::*;
use std::cell::RefCell;

fn px<S: l::LayoutScalar>(value: f64) -> l::PreferredSizeOf<S> {
    l::PreferredSizeOf::value(l::LengthPercentageOf::px(S::from_f64(value)).unwrap())
}

fn host<S: l::LayoutScalar>() -> Host<S> {
    let mut host = Host::new(l::Direction::Ltr, l::TextGroupAlign::None);
    host.box_change(0, |root| {
        root.writing_mode = l::WritingMode::SidewaysRl;
        root.text_align = l::TextAlign::Start;
        root.inline_baseline_context = l::InlineBaselineContext::Physical;
        root.size = l::Size::new(px(40.0), px(15.0));
    });
    host
}

fn incoming<S: l::LayoutScalar>(host: &mut Host<S>, width: f64, height: f64) -> u32 {
    host.push(
        Some(0),
        l::LayoutInputOf::box_input(l::NodeInputOf {
            display: l::Display::Block,
            writing_mode: l::WritingMode::SidewaysRl,
            float: l::Float::Left,
            size: l::Size::new(px(width), px(height)),
            ..l::NodeInputOf::default()
        }),
    )
}

fn paragraph<S: l::LayoutScalar>(host: &mut Host<S>, parts: &[Part<'_>], height: f64) -> Paragraph {
    Paragraph::with_preparation(
        host,
        0,
        parts,
        t::Style {
            line_height: t::LineHeight::Absolute(height),
            word_break: t::WordBreak::BreakAll,
            ..style()
        },
        t::ParagraphBaseDirection::LeftToRight,
        t::PreparationOptions::default().with_orientation(t::LineOrientation::SidewaysClockwise),
    )
}

fn request<S: l::LayoutScalar>(width: f64) -> l::LayoutRootRequestOf<S> {
    l::LayoutRootRequestOf::viewport(l::Size::new(
        l::AvailableOf::definite(S::from_f64(width)),
        l::AvailableOf::definite(S::from_f64(15.0)),
    ))
    .unwrap()
}

fn compute_float<S: l::LayoutScalar>(
    host: &Host<S>,
    paragraph: &Paragraph,
    changed: &[u32],
    viewport_width: f64,
) -> l::CompletedLayoutBatchOf<u32, S> {
    let flow = l::LayoutFlowInput::try_new(
        host,
        0,
        vec![paragraph.binding.clone()],
        l::WrappingInput::default(),
        Vec::new(),
    )
    .unwrap();
    l::compute_layout_with_flow_invalidated(&flow, 0, request(viewport_width), changed).unwrap()
}

fn packets<S: l::LayoutScalar>(
    batch: &l::CompletedLayoutBatchOf<u32, S>,
) -> Vec<(u32, l::NodeOutputOf<S>)> {
    let mut rows = batch
        .final_entries()
        .iter()
        .map(|entry| (entry.node(), entry.output()))
        .collect::<Vec<_>>();
    rows.sort_by_key(|row| row.0);
    rows
}

fn qualify<S: l::LayoutScalar>(host: &Host<S>, paragraph: &Paragraph, float: u32, raw: &[u16]) {
    assert_eq!(host.children[0][0], float);
    assert_eq!(&host.children[0][1..], paragraph.binding.nodes.as_slice());
    assert!(!paragraph.binding.nodes.contains(&float));
    assert!(paragraph.binding.atomics.is_empty());
    assert_eq!(paragraph.prepared.atomic_views().count(), 0);
    assert_eq!(
        paragraph.prepared.orientation(),
        t::LineOrientation::SidewaysClockwise
    );
    assert_eq!(
        paragraph.prepared.captured_paragraph(),
        paragraph.capture.captured_paragraph(0)
    );
    assert_eq!(
        paragraph
            .prepared
            .analyzed_source()
            .source()
            .text()
            .code_units(),
        raw
    );
    for unit in paragraph.prepared.unit_views() {
        assert_eq!((unit.level().get(), unit.advance()), (0, 10.0));
    }
    let actual_float = host.inputs[float as usize].as_box().unwrap();
    assert_eq!(actual_float.float, l::Float::Left);
    assert_eq!(actual_float.writing_mode, l::WritingMode::SidewaysRl);
}

fn assert_rows<S: l::LayoutScalar>(
    batch: &l::CompletedLayoutBatchOf<u32, S>,
    paragraph: &Paragraph,
    capacity: &[f64],
    origins: &[(f64, f64)],
) {
    let root = node_output(batch, 0);
    assert_eq!(root.location, l::Point::ZERO);
    let layout = placed(batch, paragraph);
    assert_eq!(layout.orientation(), t::LineOrientation::SidewaysClockwise);
    let rows = receipts(batch, paragraph);
    assert_eq!(rows.len(), capacity.len());
    assert_eq!(layout.lines().len(), capacity.len());
    for (i, (row, &available)) in rows.iter().zip(capacity).enumerate() {
        assert_eq!(row.prepared_text(), paragraph.prepared);
        let selected = row.selected_line();
        let context = &selected.request().context;
        assert_eq!(context.orientation, t::LineOrientation::SidewaysClockwise);
        assert_eq!(context.available_inline, Some(available));
        assert_eq!(
            context.base_direction,
            t::ParagraphBaseDirection::LeftToRight
        );
        assert_eq!(context.line_box_direction, t::LineBoxDirection::LeftToRight);
        assert_eq!(row.feedback().advance, 10.0);
        let expected_end = if i + 1 == rows.len() {
            t::LineEnd::Paragraph
        } else {
            t::LineEnd::Soft
        };
        assert_eq!(context.end, expected_end);
        assert_eq!(
            row.selected_line()
                .request()
                .end
                .view()
                .source_after()
                .get(),
            i + 1
        );
    }
    let mut actual = Vec::new();
    for run in layout.run_views() {
        assert_eq!(run.face().bytes(), AHEM);
        assert_eq!(run.face().index(), 0);
        let metrics = run
            .effective_instance()
            .unwrap()
            .metrics(paragraph.prepared.font_catalogue_identity())
            .unwrap();
        assert_eq!(
            (metrics.units_per_em(), metrics.ascent(), metrics.descent()),
            (1000, 8.0, -2.0)
        );
        actual.push((run.origin().x, run.origin().y));
    }
    assert_eq!(actual, origins);
}

fn margin_and_replay<S: l::LayoutScalar>() {
    for signed in [false, true] {
        let mut host = host::<S>();
        let float = incoming(&mut host, 10.0, 5.0);
        if signed {
            // Signed inline margins keep the actual margin-box interval0..5,
            // while the border box moves to y−2. No nominal contour is guessed.
            host.box_change(float, |input| {
                input.margin.top =
                    l::LengthAutoOf::value(l::LengthPercentageOf::px(S::from_f64(-2.0)).unwrap());
                input.margin.bottom =
                    l::LengthAutoOf::value(l::LengthPercentageOf::px(S::from_f64(2.0)).unwrap());
            });
        }
        let paragraph = paragraph(&mut host, &[Part::Text("AB")], 10.0);
        qualify(&host, &paragraph, float, &[65, 66]);
        let old = compute_float(&host, &paragraph, &[], 40.0);
        assert_rows(&old, &paragraph, &[10.0, 15.0], &[(32.0, 5.0), (22.0, 0.0)]);
        let output = node_output(&old, float);
        assert_eq!(
            output.location,
            l::Point::new(
                S::from_f64(30.0),
                S::from_f64(if signed { -2.0 } else { 0.0 })
            )
        );
        assert_eq!(
            output.size,
            l::Size::new(S::from_f64(10.0), S::from_f64(5.0))
        );
        assert_eq!(output.source_index, l::SourceIndex::new(0));
        assert_eq!(
            node_output(&old, paragraph.binding.nodes[0]).source_index,
            l::SourceIndex::new(1)
        );
        old.apply_to(&mut host).unwrap();
        let warm = compute_float(&host, &paragraph, &[], 40.0);
        assert_rows(
            &warm,
            &paragraph,
            &[10.0, 15.0],
            &[(32.0, 5.0), (22.0, 0.0)],
        );
        assert!(
            host.hits[float as usize].get() > 0,
            "actual preceding-float cache was reused"
        );
        assert_eq!(packets(&old), packets(&warm));

        host.box_change(0, |root| root.size.width = px(50.0));
        // The viewport changes with the definite root, keeping actual GX0.
        // A width50 root in viewport40 would instead have global GX−10.
        let shifted = compute_float(&host, &paragraph, &[0], 50.0);
        assert_rows(
            &shifted,
            &paragraph,
            &[10.0, 15.0],
            &[(42.0, 5.0), (32.0, 0.0)],
        );
        assert_eq!(node_output(&shifted, float).location.x, S::from_f64(40.0));
        assert!(shifted.invalidated_nodes().contains(&0));
        // A cold solve uses the same genuine preparation/capture but no old
        // host cache or unrounded packet. These are inputs, not geometry oracles.
        for cache in &mut host.published.caches {
            cache.clear();
        }
        host.published.unrounded.fill(None);
        host.published.batch = None;
        let fresh = compute_float(&host, &paragraph, &[], 50.0);
        assert_rows(
            &fresh,
            &paragraph,
            &[10.0, 15.0],
            &[(42.0, 5.0), (32.0, 0.0)],
        );
        assert_eq!(packets(&shifted), packets(&fresh));
        assert_rows(&old, &paragraph, &[10.0, 15.0], &[(32.0, 5.0), (22.0, 0.0)]);
    }
}

#[test]
fn incoming_margin_float_expires_in_real_clockwise_bands_and_replays_changed_frame() {
    margin_and_replay::<f32>();
    margin_and_replay::<f64>();
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ShapeError {
    Refused,
}

struct ShapeHost<S: l::LayoutScalar> {
    inner: Host<S>,
    float: u32,
    fail: Cell<bool>,
    queries: RefCell<Vec<l::FloatExclusionQueryOf<S>>>,
}

impl<S: l::LayoutScalar> l::Traverse for ShapeHost<S> {
    type Node = u32;
    type Scalar = S;
    type Children<'a>
        = std::iter::Copied<slice::Iter<'a, u32>>
    where
        Self: 'a;
    fn children(&self, node: u32) -> Self::Children<'_> {
        l::Traverse::children(&self.inner, node)
    }
    fn child_count(&self, node: u32) -> usize {
        l::Traverse::child_count(&self.inner, node)
    }
    fn child(&self, node: u32, index: usize) -> u32 {
        l::Traverse::child(&self.inner, node, index)
    }
}

impl<S: l::LayoutScalar> l::LayoutTree for ShapeHost<S> {
    type MeasureError = ShapeError;
    fn node_input(&self, node: u32) -> &l::NodeInputOf<S> {
        l::LayoutTree::node_input(&self.inner, node)
    }
    fn layout_input(&self, node: u32) -> l::LayoutInputViewOf<'_, S> {
        l::LayoutTree::layout_input(&self.inner, node)
    }
    fn cache_get(
        &self,
        node: u32,
        input: &l::ComputeInputOf<S>,
        context: l::CacheKeyContext,
    ) -> Option<l::ComputeOutputOf<S>> {
        l::LayoutTree::cache_get(&self.inner, node, input, context)
    }
    fn unrounded_layout(&self, node: u32) -> Option<l::NodeOutputOf<S>> {
        l::LayoutTree::unrounded_layout(&self.inner, node)
    }
    fn float_exclusion_interval(
        &self,
        node: u32,
        query: l::FloatExclusionQueryOf<S>,
    ) -> Option<Result<Option<l::FloatExclusionIntervalOf<S>>, ShapeError>> {
        if node != self.float {
            return None;
        }
        self.queries.borrow_mut().push(query);
        assert_eq!(
            query.flow_axes(),
            l::FlowAxes::new(l::WritingMode::SidewaysRl, l::Direction::Ltr)
        );
        let box_ = query.margin_box();
        assert_eq!(box_.origin(), l::Point::new(S::from_f64(20.0), S::ZERO));
        assert_eq!(
            box_.size(),
            l::Size::new(S::from_f64(20.0), S::from_f64(4.0))
        );
        if self.fail.get() {
            return Some(Err(ShapeError::Refused));
        }
        // Fixed physical triangle (20,0),(40,0),(20,4). Across any intersecting
        // x-band its union's greatest y is (40−max(xmin,20))/5, clipped0..4.
        // The callback does not inspect source, row identity or query order.
        if query.band_maximum() <= S::from_f64(20.0) || query.band_minimum() >= S::from_f64(40.0) {
            return Some(Ok(None));
        }
        let height = ((S::from_f64(40.0) - query.band_minimum().max(S::from_f64(20.0)))
            / S::from_f64(5.0))
        .max(S::ZERO)
        .min(S::from_f64(4.0));
        Some(Ok(l::FloatExclusionIntervalOf::try_new(
            query,
            S::ZERO,
            height,
        )
        .unwrap()))
    }
}

fn shape_and_error<S: l::LayoutScalar>() {
    let mut inner = host::<S>();
    let float = incoming(&mut inner, 20.0, 4.0);
    inner.box_change(float, |input| {
        input.float_exclusion = l::FloatExclusion::Shape
    });
    let paragraph = paragraph(&mut inner, &[Part::Text("AB")], 10.0);
    qualify(&inner, &paragraph, float, &[65, 66]);
    let mut host = ShapeHost {
        inner,
        float,
        fail: Cell::new(false),
        queries: RefCell::new(Vec::new()),
    };
    let run = |host: &ShapeHost<S>| {
        let flow = l::LayoutFlowInput::try_new(
            host,
            0,
            vec![paragraph.binding.clone()],
            l::WrappingInput::default(),
            Vec::new(),
        )
        .unwrap();
        l::compute_layout_with_flow(&flow, 0, request(40.0))
    };
    let old = run(&host).unwrap();
    assert_rows(&old, &paragraph, &[13.0, 11.0], &[(32.0, 2.0), (22.0, 4.0)]);
    assert!(host.queries.borrow().iter().any(|q| q.band_minimum() == S::from_f64(30.0) && q.band_maximum() == S::from_f64(40.0)));
    assert!(host.queries.borrow().iter().any(|q| q.band_minimum() == S::from_f64(20.0) && q.band_maximum() == S::from_f64(30.0)));
    old.apply_to(&mut host.inner).unwrap();
    let published = host.inner.published.batch.as_ref().unwrap().clone();
    host.fail.set(true);
    let error =
        run(&host).expect_err("actual shape refusal must not publish a partial Text packet");
    assert_eq!(error.operation(), l::LayoutOperation::FloatExclusionQuery);
    assert_eq!(
        error.site(),
        l::LayoutErrorSiteOf::ContainerSubject {
            container: 0,
            subject: float
        }
    );
    assert!(matches!(
        error.kind(),
        l::LayoutErrorKindOf::Measurement(ShapeError::Refused)
    ));
    assert_eq!(
        packets(host.inner.published.batch.as_ref().unwrap()),
        packets(&published)
    );
    assert_rows(&old, &paragraph, &[13.0, 11.0], &[(32.0, 2.0), (22.0, 4.0)]);
}

#[test]
fn clockwise_shape_queries_use_physical_x_bands_and_preserve_provider_error_atomicity() {
    shape_and_error::<f32>();
    shape_and_error::<f64>();
}

fn clear<S: l::LayoutScalar>() {
    for clears in [false, true] {
        let mut host = host::<S>();
        let float = incoming(&mut host, 30.0, 5.0);
        let paragraph = paragraph(
            &mut host,
            &[Part::Text("A"), Part::Forced, Part::Text("B")],
            10.0,
        );
        qualify(&host, &paragraph, float, &[65, 66]);
        let control = paragraph.binding.controls[0].node;
        let input = host.inputs[control as usize]
            .as_line_break()
            .unwrap()
            .with_writing_mode(l::WritingMode::SidewaysRl)
            .with_clear(if clears {
                l::Clear::Left
            } else {
                l::Clear::None
            });
        host.inputs[control as usize] = l::LayoutInputOf::line_break(input);
        let batch = compute_float(&host, &paragraph, &[], 40.0);
        let lines = receipts(&batch, &paragraph);
        assert_eq!(lines.len(), 2);
        assert_eq!(
            lines[0].selected_line().request().context.end,
            t::LineEnd::Forced
        );
        assert_eq!(
            lines[1].selected_line().request().context.end,
            t::LineEnd::Paragraph
        );
        assert_eq!(
            lines[0].selected_line().request().controls[0].occurrence,
            paragraph.binding.controls[0].occurrence
        );
        assert_eq!(
            lines[0].selected_line().request().context.available_inline,
            Some(10.0)
        );
        assert_eq!(
            lines[1].selected_line().request().context.available_inline,
            Some(if clears { 15.0 } else { 10.0 })
        );
        let layout = placed(&batch, &paragraph);
        assert_eq!(layout.lines()[0].baseline(), 32.0);
        assert_eq!(
            layout.lines()[1].baseline(),
            if clears { 2.0 } else { 22.0 }
        );
        assert_eq!(
            layout.lines()[1].bounds().origin.y,
            if clears { 0.0 } else { 5.0 }
        );
        assert_eq!(layout.nonformatted_lines().count(), 0);
        assert_eq!(
            node_output(&batch, float).location,
            l::Point::new(S::from_f64(10.0), S::ZERO)
        );
    }
}

#[test]
fn actual_forced_clear_advances_clockwise_block_to_float_expiry_without_changing_source() {
    clear::<f32>();
    clear::<f64>();
}

fn fractional_minimum<S: l::LayoutScalar>() {
    for constrained in [false, true] {
        let mut host = host::<S>();
        if constrained {
            incoming(&mut host, 10.0, 15.0);
        }
        let paragraph = paragraph(&mut host, &[Part::Text("A")], 11.0);
        assert_eq!(paragraph.prepared.atomic_views().count(), 0);
        let batch = compute_float(&host, &paragraph, &[], 40.0);
        let rows = receipts(&batch, &paragraph);
        assert_eq!(rows.len(), 1);
        assert_eq!(
            (rows[0].feedback().baseline, rows[0].feedback().line_height),
            (8.5, 11.0)
        );
        assert_eq!(
            rows[0].selected_line().request().context.available_inline,
            Some(15.0)
        );
        assert_eq!(
            rows[0].selected_line().request().context.end,
            t::LineEnd::Paragraph
        );
        assert_eq!(
            rows[0]
                .selected_line()
                .request()
                .end
                .view()
                .source_after()
                .get(),
            1
        );
        // Natural half-leading(11−8−2)/2 supplies actual baseline8.5.
        // Without a bound: physical31.5 rounds nearest32. The real full-inline
        // float forbids block<10; max physical21.5 therefore floors21, not22.
        let layout = placed(&batch, &paragraph);
        assert_eq!(
            layout.lines()[0].baseline(),
            if constrained { 21.0 } else { 32.0 }
        );
        assert_eq!(layout.lines()[0].range(), t::Range::new(0, 1));
        assert_eq!(layout.run_views().next().unwrap().face().bytes(), AHEM);
        if constrained {
            assert_eq!(
                node_output(&batch, 1).location,
                l::Point::new(S::from_f64(30.0), S::ZERO)
            );
            assert!(
                layout.lines()[0].bounds().origin.x + layout.lines()[0].bounds().size.width <= 30.0,
                "accepted line stays beyond the genuine float's block-end"
            );
        }
    }
}

#[test]
fn fractional_clockwise_minimum_uses_floor_of_real_float_bound_while_unconstrained_uses_nearest() {
    fractional_minimum::<f32>();
    fractional_minimum::<f64>();
}
