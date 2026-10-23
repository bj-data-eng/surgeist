//! The ordinary zone reference owns its natural height and float frame.
use super::*;
use surgeist_text::{
    HyphenateCharacter, HyphenationProvenance, HyphenationResource, Hyphens, LineEnd, OverflowWrap,
    PreparedAlternativeDefinition, PreparedBreakKind, PreparedTermination, Range,
    SpacingAllocations, Text, WhiteSpace,
};

const AHEM: &[u8] = include_bytes!("../../../../surgeist-text/tests/fixtures/fonts/ahem/Ahem.ttf");

/// Actual rectangular shape x=0..40/y=10..30 in a leading 40x30 float.
/// In particular [0,10) does not intersect it; [0,20) does.
struct RecessedShape<S: LayoutScalar> {
    tree: PublicLayoutTreeOf<S>,
    queries: std::cell::RefCell<Vec<(FloatExclusionQueryOf<S>, bool)>>,
}

impl<S: LayoutScalar> Traverse for RecessedShape<S> {
    type Node = u32;
    type Scalar = S;
    type Children<'a>
        = <PublicLayoutTreeOf<S> as Traverse>::Children<'a>
    where
        Self: 'a;

    fn children(&self, node: u32) -> Self::Children<'_> {
        Traverse::children(&self.tree, node)
    }
    fn child_count(&self, node: u32) -> usize {
        self.tree.child_count(node)
    }
    fn child(&self, node: u32, index: usize) -> u32 {
        self.tree.child(node, index)
    }
}

impl<S: LayoutScalar> LayoutTree for RecessedShape<S> {
    type MeasureError = core::convert::Infallible;

    fn node_input(&self, node: u32) -> &NodeInputOf<S> {
        self.tree.node_input(node)
    }
    fn layout_input(&self, node: u32) -> LayoutInputViewOf<'_, S> {
        self.tree.layout_input(node)
    }
    fn float_exclusion_interval(
        &self,
        node: u32,
        query: FloatExclusionQueryOf<S>,
    ) -> Option<Result<Option<FloatExclusionIntervalOf<S>>, Self::MeasureError>> {
        assert_eq!(node, 30);
        let top = query.margin_box().origin().y + S::from_f64(10.0);
        let bottom = query.margin_box().origin().y + S::from_f64(30.0);
        let intersects = query.band_minimum() < bottom && top < query.band_maximum();
        self.queries.borrow_mut().push((query, intersects));
        let x = query.margin_box().origin().x;
        Some(Ok(if intersects {
            FloatExclusionIntervalOf::try_new(query, x, x + S::from_f64(40.0)).unwrap()
        } else {
            None
        }))
    }
}

fn paragraph() -> PreparedParagraph {
    let mut short = ahem_style(10.0, 10.0);
    short.hyphens = Hyphens::Auto;
    short.locale = Some("en-US".into());
    short.hyphenate_character = HyphenateCharacter::String(Text::from("A"));
    short.white_space = WhiteSpace::Collapse;
    short.wrap = Wrap::Word;
    short.overflow_wrap = OverflowWrap::Normal;
    let mut tall = short.clone();
    tall.line_height = surgeist_text::LineHeight::Absolute(20.0);
    PreparedParagraph::new(
        vec![
            ParagraphPart::text(1, Source::new("X "), short.clone()),
            ParagraphPart::text(2, Source::new("associate"), tall),
        ],
        short,
        ParagraphBaseDirection::LeftToRight,
    )
}

fn qualify_reference(paragraph: &PreparedParagraph) {
    let p = &paragraph.prepared;
    assert_eq!(
        p.analyzed_source().source().text().code_units(),
        "X associate".encode_utf16().collect::<Vec<_>>()
    );
    let word = p
        .word_views()
        .find(|w| w.query() == Some("associate"))
        .unwrap();
    assert_eq!(word.resource(), HyphenationResource::EnUs2005);
    assert_eq!(
        word.provenance(),
        Some(HyphenationProvenance::SourceException { entry: "associate" })
    );
    assert_eq!(
        word.cuts()
            .map(|cut| {
                assert!(cut.opportunity().is_some());
                let counts = cut.counts().unwrap();
                (
                    cut.boundary().view().source_before().get(),
                    counts.total,
                    counts.before,
                    counts.after,
                )
            })
            .collect::<Vec<_>>(),
        [(4, 9, 2, 7), (6, 9, 4, 5)]
    );
    let offer = p
        .opportunity_views()
        .find(|o| {
            o.kind() == PreparedBreakKind::Allowed && o.boundary().view().source_before().get() == 2
        })
        .unwrap()
        .handle();
    let template = crate::test_support::text_fragment::fragment_for(p).finalized_line();
    let mut request = template.selected_line().request().clone();
    request.start = p.boundary_views().next().unwrap().handle();
    request.end = offer.view().boundary();
    request.participants = p
        .participants()
        .take(request.end.view().participant_offset())
        .collect();
    request.termination = PreparedTermination::Ordinary(offer);
    request.alternative = None;
    request.context.available_inline = Some(100.0);
    request.context.end = LineEnd::Soft;
    request.context.spacing = surgeist_text::SpacingPolicy::None;
    let line = p
        .select(request)
        .unwrap()
        .finalize(SpacingAllocations::None)
        .unwrap();
    // Ahem ascent8/descent2 at 10px; only the short prefix participates.
    // Its trailing SPACE is removed, leaving alignment10 and thus U=90.
    assert_eq!(
        (
            line.feedback().advance,
            line.feedback().baseline,
            line.feedback().line_height
        ),
        (10.0, 8.0, 10.0)
    );
    assert_eq!(line.measurements().alignment().advance(), 10.0);
    assert!(line.edge_units().any(|edge| edge.disposition()
        == surgeist_text::LineEdgeDisposition::Removed
        && edge.source_pieces().collect::<Vec<_>>() == [Range::new(1, 2)]));
    assert!(
        line.run_views().all(|run| run.face().bytes() == AHEM
            && run.effective_instance().unwrap().font_size() == 10.0)
    );
}

fn check<S: LayoutScalar>(zone: f64, allowed: bool) {
    let paragraph = paragraph();
    qualify_reference(&paragraph);
    let host = RecessedShape {
        tree: PublicLayoutTreeOf::<S>::new()
            .children(0, [30, 1, 2])
            .children(30, [])
            .children(1, [])
            .children(2, [])
            .style(
                0,
                NodeInputOf {
                    display: Display::Block,
                    size: Size::new(
                        PreferredSizeOf::px(S::from_f64(100.0)),
                        PreferredSizeOf::AUTO,
                    ),
                    hyphenate_limit_zone: LengthPercentageOf::px(S::from_f64(zone)).unwrap(),
                    ..NodeInputOf::default()
                },
            )
            .style(
                30,
                NodeInputOf {
                    display: Display::Block,
                    float: Float::Left,
                    float_exclusion: FloatExclusion::Shape,
                    size: Size::new(
                        PreferredSizeOf::px(S::from_f64(40.0)),
                        PreferredSizeOf::px(S::from_f64(30.0)),
                    ),
                    ..NodeInputOf::default()
                },
            )
            .input(1, paragraph.text_input(1))
            .input(2, paragraph.text_input(2)),
        queries: Default::default(),
    };
    let margin =
        ScrollRectOf::try_new(Point::ZERO, Size::new(S::from_f64(40.0), S::from_f64(30.0)))
            .unwrap();
    for (height, expected) in [
        (10.0, None),
        (20.0, Some((0.0, 40.0))),
        (30.0, Some((0.0, 40.0))),
    ] {
        let query = FloatExclusionQueryOf::try_new(
            margin,
            FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
            S::ZERO,
            S::from_f64(height),
        )
        .unwrap();
        let interval = host.float_exclusion_interval(30, query).unwrap().unwrap();
        assert_eq!(
            interval.map(|i| (i.minimum().to_f64(), i.maximum().to_f64())),
            expected
        );
    }
    host.queries.borrow_mut().clear();
    let capture = TextLayoutInput::try_new(&host, 0, vec![paragraph.binding(0)]).unwrap();
    let batch = compute_layout_with_text(&capture, 0, request(100.0)).unwrap();
    let layout = batch.placed_text()[0].layout();
    let expected = if allowed {
        // Fallback U90 > Z75, although the candidate's 60px band would give
        // only50. Its own 20px height still encounters the real shape.
        [
            (Range::new(0, 4), 50.0, 0.0, 13.0, 40.0, 60.0, 20.0),
            (Range::new(4, 6), 30.0, 20.0, 33.0, 40.0, 60.0, 20.0),
            (Range::new(6, 11), 50.0, 40.0, 53.0, 0.0, 100.0, 20.0),
        ]
    } else {
        // U90 == Z90 suppresses only the first-line automatic variants.
        [
            (Range::new(0, 2), 10.0, 0.0, 8.0, 0.0, 100.0, 10.0),
            (Range::new(2, 6), 50.0, 10.0, 23.0, 40.0, 60.0, 20.0),
            (Range::new(6, 11), 50.0, 30.0, 43.0, 0.0, 100.0, 20.0),
        ]
    };
    assert_eq!(layout.lines().len(), expected.len());
    assert_eq!(
        layout.source().text().code_units(),
        "X associate".encode_utf16().collect::<Vec<_>>()
    );
    for (index, (range, advance, top, baseline, x, width, height)) in
        expected.into_iter().enumerate()
    {
        let placed = &layout.lines()[index];
        assert_eq!(placed.range(), range);
        assert_eq!(placed.baseline(), baseline);
        assert_eq!(
            placed.bounds(),
            surgeist_text::Rect::new(x, top, width, height)
        );
        let finalized = batch
            .final_inline_fragments()
            .iter()
            .find(|row| row.fragment().line_index() == index)
            .unwrap()
            .fragment()
            .finalized_fragment()
            .finalized_line();
        assert_eq!(finalized.prepared_text(), paragraph.prepared);
        assert_eq!(finalized.feedback().advance, advance);
        assert_eq!(finalized.feedback().line_height, height);
        assert_eq!(
            finalized.selected_line().request().context.available_inline,
            Some(width)
        );
        let selected = finalized.selected_line();
        if index == 2 || (!allowed && index == 0) {
            assert!(selected.request().alternative.is_none());
        } else {
            assert!(matches!(
                selected
                    .request()
                    .alternative
                    .as_ref()
                    .unwrap()
                    .view()
                    .definition(),
                PreparedAlternativeDefinition::Automatic { .. }
            ));
        }
        assert!(finalized.run_views().all(|run| run.face().bytes() == AHEM
            && run.effective_instance().unwrap().font_size() == 10.0));
    }
    let float = batch
        .final_entries()
        .iter()
        .find(|row| row.node() == 30)
        .unwrap()
        .output();
    assert_eq!(float.location, Point::ZERO);
    assert_eq!(float.size, Size::new(S::from_f64(40.0), S::from_f64(30.0)));
    let root = batch
        .final_entries()
        .iter()
        .find(|row| row.node() == 0)
        .unwrap()
        .output();
    assert_eq!(root.size.height.to_f64(), if allowed { 60.0 } else { 50.0 });
    let queries = host.queries.borrow();
    assert!(queries.iter().any(|(q, hit)| !hit
        && q.band_minimum() == S::ZERO
        && q.band_maximum() == S::from_f64(10.0)));
    assert!(queries.iter().any(|(q, hit)| *hit
        && q.band_minimum() == S::ZERO
        && q.band_maximum() == S::from_f64(20.0)));
}

#[test]
fn zone_reference_converges_with_its_own_short_height_before_the_candidate_shape_frame() {
    for zone in [75.0, 90.0] {
        check::<f32>(zone, zone < 90.0);
        check::<f64>(zone, zone < 90.0);
    }
}
