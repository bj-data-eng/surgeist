//! Native Layout consumes actual CW preparation, not horizontal geometry proxies.
use super::*;

fn qualify_preparation(paragraph: &Paragraph, orientation: t::LineOrientation) {
    let p = &paragraph.prepared;
    assert_eq!(p.orientation(), orientation);
    assert_eq!(
        p.captured_paragraph(),
        paragraph.capture.captured_paragraph(0)
    );
    assert_eq!(p.analyzed_source().source().text().code_units(), [65, 66]);
    assert_eq!(
        p.unit_views()
            .map(|u| (u.source_range(), u.level().get(), u.advance()))
            .collect::<Vec<_>>(),
        [
            (t::Range::new(0, 1), 0, 10.0),
            (t::Range::new(1, 2), 0, 10.0),
        ]
    );
    assert_eq!(p.control_views().count(), 0);
    assert_eq!(p.atomic_views().count(), 0);
    assert_eq!(paragraph.binding.container, 0);
    assert_eq!(paragraph.binding.nodes, [1]);
    assert_eq!(
        paragraph.binding.origins[0].origin,
        p.origin_views().next().unwrap().handle()
    );
    // Authenticate the already-supported Text producer before the Layout
    // consumer is asked to fit it. This whole-line receipt is not substituted
    // for the actual narrower selections that Layout must subsequently issue.
    let end = p
        .opportunity_views()
        .find(|o| o.kind() == t::PreparedBreakKind::ParagraphEnd)
        .unwrap()
        .handle();
    let line = p
        .select(t::LineSelectionRequest {
            start: p.boundary_views().next().unwrap().handle(),
            end: end.view().boundary(),
            termination: t::PreparedTermination::Ordinary(end),
            alternative: None,
            incoming: None,
            participants: p.participants().collect(),
            atomics: vec![],
            controls: vec![],
            boundary_context: vec![],
            line_box_struts: vec![],
            line_strut: None,
            context: t::LineContext {
                alignment_insets: t::LineAlignmentInsets::ZERO,
                formatted_line: t::FormattedLineContext::default(),
                base_direction: t::ParagraphBaseDirection::LeftToRight,
                line_box_direction: t::LineBoxDirection::LeftToRight,
                orientation,
                start: t::LineStart::Paragraph,
                end: t::LineEnd::Paragraph,
                available_inline: None,
                indent: 0.0,
                alignment: t::Alignment::Left,
                tabs: t::TabContext::NoTabs,
                spacing: t::SpacingPolicy::None,
            },
        })
        .unwrap()
        .finalize(t::SpacingAllocations::None)
        .unwrap();
    assert_eq!(
        (line.feedback().advance, line.feedback().line_height),
        (20.0, 10.0)
    );
    let mut glyphs = Vec::new();
    for run in line.run_views() {
        assert_eq!(run.face().bytes(), AHEM);
        assert_eq!(run.face().index(), 0);
        assert_eq!(run.face().catalogue_identity(), p.font_catalogue_identity());
        let instance = run.effective_instance().unwrap();
        let metrics = instance.metrics(p.font_catalogue_identity()).unwrap();
        assert_eq!(metrics.units_per_em(), 1000);
        assert_eq!(
            (metrics.ascent(), metrics.descent(), metrics.leading()),
            (8.0, -2.0, 0.0)
        );
        glyphs.extend(
            run.glyphs()
                .map(|g| (g.id(), g.pen(), g.x(), g.y(), g.advance(), g.range())),
        );
    }
    assert_eq!(
        glyphs,
        [
            (35, 0.0, 0.0, 0.0, 10.0, t::Range::new(0, 1)),
            (36, 10.0, 0.0, 0.0, 10.0, t::Range::new(1, 2))
        ]
    );
}

fn finite<S: l::LayoutScalar>() {
    for (writing_mode, orientation) in [
        (l::WritingMode::HorizontalTb, t::LineOrientation::Horizontal),
        (l::WritingMode::SidewaysRl, t::LineOrientation::Sideways),
    ] {
        let mut host = Host::<S>::new(l::Direction::Ltr, l::TextGroupAlign::None);
        host.box_change(0, |root| {
            root.writing_mode = writing_mode;
            root.text_align = l::TextAlign::Start;
            root.inline_baseline_context = l::InlineBaselineContext::Physical;
            root.size = l::Size::new(
                l::PreferredSizeOf::value(l::LengthPercentageOf::px(S::from_f64(40.0)).unwrap()),
                l::PreferredSizeOf::value(l::LengthPercentageOf::px(S::from_f64(15.0)).unwrap()),
            );
        });
        let paragraph = Paragraph::with_preparation(
            &mut host,
            0,
            &[Part::Text("AB")],
            t::Style {
                word_break: t::WordBreak::BreakAll,
                ..style()
            },
            t::ParagraphBaseDirection::LeftToRight,
            t::PreparationOptions::default().with_orientation(orientation),
        );
        qualify_preparation(&paragraph, orientation);
        let flow = l::LayoutFlowInput::try_new(
            &host,
            0,
            vec![paragraph.binding.clone()],
            l::WrappingInput::default(),
        )
        .unwrap();
        let request = l::LayoutRootRequestOf::viewport(l::Size::new(
            l::AvailableOf::definite(S::from_f64(40.0)),
            l::AvailableOf::definite(S::from_f64(15.0)),
        ))
        .unwrap();
        // Existing callable first divergence: the real CW producer above is
        // valid; the Srl consumer must select/retain/place it, not request Vertical.
        let batch = l::compute_layout_with_flow(&flow, 0, request)
            .expect("actual finite Layout must consume the matching prepared Text basis");
        let root = node_output(&batch, 0);
        assert_eq!(root.location, l::Point::ZERO);
        assert_eq!(
            root.size,
            l::Size::new(S::from_f64(40.0), S::from_f64(15.0))
        );
        assert_eq!(root.source_index, l::SourceIndex::new(0));
        let layout = placed(&batch, &paragraph);
        assert_eq!(layout.orientation(), orientation);
        assert_eq!(layout.source().text().code_units(), [65, 66]);
        assert_eq!(
            layout.analyzed_source().identity(),
            paragraph.prepared.analyzed_source().identity()
        );
        assert_eq!(layout.nonformatted_lines().count(), 0);
        let rows = receipts(&batch, &paragraph);
        assert_eq!(layout.metrics().size(), t::Size::new(20.0, 10.0));
        assert_eq!(layout.metrics().full_width(), 20.0);
        let advances = rows
            .iter()
            .map(|line| line.feedback().advance)
            .collect::<Vec<_>>();
        let mut glyphs = Vec::new();
        for (i, line) in rows.iter().enumerate() {
            assert_eq!(line.prepared_text(), paragraph.prepared);
            assert_eq!(
                line.selected_line().request().context.orientation,
                orientation
            );
            assert_eq!(
                line.selected_line().request().context.available_inline,
                Some(if orientation == t::LineOrientation::Horizontal {
                    40.0
                } else {
                    15.0
                })
            );
            assert_eq!(line.feedback().line_height, 10.0);
            assert_eq!(
                line.selected_line()
                    .request()
                    .end
                    .view()
                    .source_after()
                    .get(),
                if orientation == t::LineOrientation::Horizontal {
                    2
                } else {
                    i + 1
                }
            );
            assert_eq!(
                line.selected_line().request().context.end,
                if i + 1 == rows.len() {
                    t::LineEnd::Paragraph
                } else {
                    t::LineEnd::Soft
                }
            );
            for run in line.run_views() {
                assert_eq!(run.face().bytes(), AHEM);
                glyphs.extend(run.glyphs().map(|g| (g.id(), g.advance(), g.range())));
            }
        }
        assert_eq!(
            glyphs,
            [
                (35, 10.0, t::Range::new(0, 1)),
                (36, 10.0, t::Range::new(1, 2))
            ]
        );
        if orientation == t::LineOrientation::Horizontal {
            assert_eq!(advances, [20.0]);
            assert_eq!(
                layout.lines()[0].bounds(),
                t::Rect::new(0.0, 0.0, 40.0, 10.0)
            );
            assert_eq!(layout.bounds(), t::Rect::new(0.0, 0.0, 40.0, 10.0));
            assert_eq!(
                layout.run_views().next().unwrap().origin(),
                t::Point::new(0.0, 8.0)
            );
            assert_eq!(
                layout
                    .cursor(t::Cursor::new(0, t::Affinity::After))
                    .unwrap()
                    .rect(),
                t::Rect::new(0.0, 0.0, 1.0, 10.0)
            );
        } else {
            assert_eq!(advances, [10.0, 10.0]);
            assert_eq!(
                layout
                    .lines()
                    .iter()
                    .map(|line| (line.range(), line.baseline(), line.bounds()))
                    .collect::<Vec<_>>(),
                [
                    (
                        t::Range::new(0, 1),
                        32.0,
                        t::Rect::new(30.0, 0.0, 10.0, 15.0)
                    ),
                    (
                        t::Range::new(1, 2),
                        22.0,
                        t::Rect::new(20.0, 0.0, 10.0, 15.0)
                    ),
                ]
            );
            assert_eq!(layout.bounds(), t::Rect::new(20.0, 0.0, 20.0, 15.0));
            assert_eq!(
                layout
                    .run_views()
                    .map(|run| run.origin())
                    .collect::<Vec<_>>(),
                [t::Point::new(32.0, 0.0), t::Point::new(22.0, 0.0)]
            );
            assert_eq!(
                layout
                    .cursor(t::Cursor::new(0, t::Affinity::After))
                    .unwrap()
                    .rect(),
                t::Rect::new(30.0, 0.0, 10.0, 1.0)
            );
            assert_eq!(
                layout
                    .cursor(t::Cursor::new(1, t::Affinity::After))
                    .unwrap()
                    .rect(),
                t::Rect::new(20.0, 0.0, 10.0, 1.0)
            );
            let selection = layout
                .selection(t::Selection::new(
                    t::Cursor::new(0, t::Affinity::After),
                    t::Cursor::new(1, t::Affinity::Before),
                ))
                .unwrap();
            assert_eq!(selection.rects().len(), 1);
            assert_eq!(
                selection.rects()[0].rect(),
                t::Rect::new(30.0, 0.0, 10.0, 10.0)
            );
        }
    }
}

#[test]
fn real_finite_horizontal_then_sideways_rl_text_f32() {
    finite::<f32>();
}
#[test]
fn real_finite_horizontal_then_sideways_rl_text_f64() {
    finite::<f64>();
}

mod controls;
mod floats;
mod geometry;
